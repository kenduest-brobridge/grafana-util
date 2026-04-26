use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::Result;

use super::datasource_refs::resolve_datasource_ref;
use super::helpers::{
    datasource_type_alias, extract_placeholder_name, is_placeholder_string, DatasourceCatalog,
    InputMapping,
};
use super::inputs::allocate_input_mapping;

#[derive(Clone, Debug)]
pub(super) struct ConstantInputMapping {
    pub(super) input_name: String,
    pub(super) label: String,
    pub(super) value: Value,
}

pub(super) fn make_constant_input_name(name: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_underscore = false;
    for character in name.chars().flat_map(|character| character.to_uppercase()) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
            last_was_underscore = false;
        } else if !last_was_underscore {
            normalized.push('_');
            last_was_underscore = true;
        }
    }
    let normalized = normalized.trim_matches('_');
    format!(
        "VAR_{}",
        if normalized.is_empty() {
            "CONSTANT"
        } else {
            normalized
        }
    )
}

pub(super) fn collect_and_rewrite_constant_variables(
    dashboard: &mut Value,
) -> Vec<ConstantInputMapping> {
    let Some(variables) = dashboard
        .get_mut("templating")
        .and_then(Value::as_object_mut)
        .and_then(|templating| templating.get_mut("list"))
        .and_then(Value::as_array_mut)
    else {
        return Vec::new();
    };

    let mut constants = Vec::new();
    let mut name_counts = BTreeMap::<String, usize>::new();
    for variable in variables {
        let Some(variable_object) = variable.as_object_mut() else {
            continue;
        };
        if variable_object.get("type").and_then(Value::as_str) != Some("constant") {
            continue;
        }
        let variable_name = variable_object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("constant");
        let input_base = make_constant_input_name(variable_name);
        let count = name_counts.get(&input_base).copied().unwrap_or(0) + 1;
        name_counts.insert(input_base.clone(), count);
        let input_name = if count == 1 {
            input_base
        } else {
            format!("{input_base}_{count}")
        };
        let label = variable_object
            .get("label")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(variable_name)
            .to_string();
        let value = variable_object
            .get("query")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new()));
        let placeholder = format!("${{{input_name}}}");
        variable_object.insert("query".to_string(), Value::String(placeholder.clone()));
        let current = json!({
            "value": placeholder,
            "text": placeholder,
            "selected": false,
        });
        variable_object.insert("current".to_string(), current.clone());
        variable_object.insert("options".to_string(), Value::Array(vec![current]));
        constants.push(ConstantInputMapping {
            input_name,
            label,
            value,
        });
    }
    constants
}

pub(super) fn collect_datasource_variable_reference_names(
    node: &Value,
    names: &mut BTreeSet<String>,
) {
    match node {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "datasource" {
                    collect_placeholder_reference_name(value, names);
                }
                collect_datasource_variable_reference_names(value, names);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_datasource_variable_reference_names(item, names);
            }
        }
        _ => {}
    }
}

fn collect_placeholder_reference_name(reference: &Value, names: &mut BTreeSet<String>) {
    match reference {
        Value::String(text) if is_placeholder_string(text) => {
            names.insert(extract_placeholder_name(text));
        }
        Value::Object(object) => {
            for key in ["uid", "name"] {
                if let Some(text) = object.get(key).and_then(Value::as_str) {
                    if is_placeholder_string(text) {
                        names.insert(extract_placeholder_name(text));
                    }
                }
            }
        }
        _ => {}
    }
}

fn datasource_variable_current_reference(variable: &Map<String, Value>) -> Option<Value> {
    let current = variable.get("current").and_then(Value::as_object)?;
    let current_value = current
        .get("value")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && !is_placeholder_string(value))?;
    let current_text = current
        .get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let query_type = variable
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && !is_placeholder_string(value))
        .map(datasource_type_alias);

    let mut reference = Map::new();
    reference.insert("uid".to_string(), Value::String(current_value.to_string()));
    if let Some(current_text) = current_text.filter(|value| *value != current_value) {
        reference.insert("name".to_string(), Value::String(current_text.to_string()));
    }
    if let Some(query_type) = query_type {
        reference.insert("type".to_string(), Value::String(query_type.to_string()));
    }
    Some(Value::Object(reference))
}

pub(super) fn map_datasource_variable_current_inputs(
    dashboard: &mut Value,
    used_variable_names: &BTreeSet<String>,
    ref_mapping: &mut BTreeMap<String, InputMapping>,
    type_counts: &mut BTreeMap<String, usize>,
    datasource_catalog: &DatasourceCatalog,
) -> Result<()> {
    let Some(variables) = dashboard
        .get_mut("templating")
        .and_then(Value::as_object_mut)
        .and_then(|templating| templating.get_mut("list"))
        .and_then(Value::as_array_mut)
    else {
        return Ok(());
    };

    for variable in variables {
        let Some(variable_object) = variable.as_object_mut() else {
            continue;
        };
        if variable_object.get("type").and_then(Value::as_str) != Some("datasource") {
            continue;
        }
        let variable_is_used = variable_object
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| used_variable_names.contains(name));
        if !variable_is_used {
            continue;
        }
        let Some(reference) = datasource_variable_current_reference(variable_object) else {
            continue;
        };
        let Some(resolved) = resolve_datasource_ref(&reference, datasource_catalog)? else {
            continue;
        };
        let mapping = allocate_input_mapping(&resolved, ref_mapping, type_counts, None);
        variable_object.insert(
            "current".to_string(),
            json!({
                "text": "",
                "value": format!("${{{}}}", mapping.input_name),
                "selected": true,
            }),
        );
        variable_object.insert("options".to_string(), Value::Array(Vec::new()));
    }
    Ok(())
}
