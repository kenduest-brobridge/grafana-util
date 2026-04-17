//! Dashboard prompt output transformation.
//! Translates query panels into external dashboard import payload shape and keeps datasource mapping helpers.
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{message, string_field, value_as_object, Result};

use super::build_preserved_web_import_document;

pub(crate) use super::prompt_helpers::{
    build_datasource_catalog, build_resolved_datasource, collect_datasource_refs,
    datasource_plugin_version, datasource_type_alias, extract_placeholder_name,
    format_panel_plugin_name, format_plugin_name, is_builtin_datasource_ref, is_placeholder_string,
    lookup_datasource, make_input_label, make_input_name, resolve_datasource_type_alias,
    DatasourceCatalog, InputMapping, ResolvedDatasource,
};

fn resolve_string_datasource_ref(
    reference: &str,
    datasource_catalog: &DatasourceCatalog,
) -> Result<ResolvedDatasource> {
    if let Some(datasource) =
        lookup_datasource(datasource_catalog, Some(reference), Some(reference))
    {
        let uid = string_field(&datasource, "uid", reference);
        let ds_type = string_field(&datasource, "type", "");
        if ds_type.is_empty() {
            return Err(message(format!(
                "Datasource {reference:?} does not have a usable type."
            )));
        }
        let label = string_field(&datasource, "name", reference);
        let mut resolved = build_resolved_datasource(format!("uid:{uid}"), ds_type, label);
        resolved.plugin_version = datasource_plugin_version(&datasource);
        return Ok(resolved);
    }

    if let Some(datasource_type) = resolve_datasource_type_alias(reference, datasource_catalog) {
        return Ok(build_resolved_datasource(
            format!("type:{datasource_type}"),
            datasource_type.clone(),
            format_plugin_name(&datasource_type),
        ));
    }

    Err(message(format!(
        "Cannot resolve datasource name or uid {reference:?} for prompt export."
    )))
}

fn resolve_object_datasource_ref(
    reference: &Map<String, Value>,
    datasource_catalog: &DatasourceCatalog,
) -> Result<Option<ResolvedDatasource>> {
    let uid = reference.get("uid").and_then(Value::as_str);
    let name = reference.get("name").and_then(Value::as_str);
    let ds_type = reference.get("type").and_then(Value::as_str);
    if ds_type == Some("datasource") && !should_promote_generic_datasource_ref(uid, name, ds_type) {
        return Ok(None);
    }
    if should_promote_generic_datasource_ref(uid, name, ds_type) {
        return Ok(Some(build_resolved_datasource(
            "type:datasource".to_string(),
            "datasource".to_string(),
            "datasource".to_string(),
        )));
    }
    if uid.is_some_and(is_placeholder_string) || name.is_some_and(is_placeholder_string) {
        return Ok(None);
    }

    let datasource = lookup_datasource(datasource_catalog, uid, name);
    let mut resolved_type = ds_type.unwrap_or_default().to_string();
    let mut resolved_label = name.unwrap_or(uid.unwrap_or_default()).to_string();
    let mut resolved_uid = uid.unwrap_or(name.unwrap_or_default()).to_string();
    if let Some(ref datasource) = datasource {
        if resolved_type.is_empty() {
            resolved_type = string_field(datasource, "type", "");
        }
        let datasource_name = string_field(datasource, "name", "");
        if !datasource_name.is_empty() {
            resolved_label = datasource_name;
        }
        if resolved_uid.is_empty() {
            resolved_uid = string_field(datasource, "uid", "");
        }
    }

    if resolved_type.is_empty() {
        return Err(message(format!(
            "Cannot resolve datasource type from reference {:?}.",
            Value::Object(reference.clone())
        )));
    }
    if resolved_uid.is_empty() {
        resolved_uid = resolved_type.clone();
    }
    if resolved_label.is_empty() {
        resolved_label = resolved_type.clone();
    }

    let mut resolved =
        build_resolved_datasource(format!("uid:{resolved_uid}"), resolved_type, resolved_label);
    if let Some(datasource) = datasource {
        resolved.plugin_version = datasource_plugin_version(&datasource);
    }
    Ok(Some(resolved))
}

fn resolve_datasource_ref(
    reference: &Value,
    datasource_catalog: &DatasourceCatalog,
) -> Result<Option<ResolvedDatasource>> {
    let allow_generic_object = reference.as_object().is_some_and(|object| {
        should_promote_generic_datasource_ref(
            object.get("uid").and_then(Value::as_str),
            object.get("name").and_then(Value::as_str),
            object.get("type").and_then(Value::as_str),
        )
    });
    if reference.is_null() || (is_builtin_datasource_ref(reference) && !allow_generic_object) {
        return Ok(None);
    }
    match reference {
        Value::String(text) => {
            if is_placeholder_string(text) {
                Ok(None)
            } else {
                resolve_string_datasource_ref(text, datasource_catalog).map(Some)
            }
        }
        Value::Object(object) => resolve_object_datasource_ref(object, datasource_catalog),
        _ => Ok(None),
    }
}

fn should_promote_generic_datasource_ref(
    uid: Option<&str>,
    name: Option<&str>,
    ds_type: Option<&str>,
) -> bool {
    if ds_type != Some("datasource") {
        return false;
    }
    let uid = uid.unwrap_or_default();
    let name = name.unwrap_or_default();
    (uid.is_empty() && name.is_empty())
        || uid == "-- Mixed --"
        || name == "-- Mixed --"
        || is_placeholder_string(uid)
        || is_placeholder_string(name)
}

fn allocate_input_mapping(
    resolved: &ResolvedDatasource,
    ref_mapping: &mut BTreeMap<String, InputMapping>,
    type_counts: &mut BTreeMap<String, usize>,
    key_override: Option<String>,
) -> InputMapping {
    let mapping_key = key_override.unwrap_or_else(|| resolved.key.clone());
    if let Some(mapping) = ref_mapping.get(&mapping_key) {
        return mapping.clone();
    }
    let input_label = if resolved.input_label.is_empty() {
        resolved.plugin_name.clone()
    } else {
        resolved.input_label.clone()
    };
    let input_base = make_input_name(&input_label);
    let count = type_counts.get(&input_base).copied().unwrap_or(0) + 1;
    type_counts.insert(input_base.clone(), count);
    let mapping = InputMapping {
        input_name: if count == 1 {
            input_base
        } else {
            format!("{input_base}_{count}")
        },
        label: if resolved.input_label.is_empty() {
            make_input_label(&resolved.ds_type, count)
        } else {
            resolved.input_label.clone()
        },
        plugin_name: resolved.plugin_name.clone(),
        ds_type: resolved.ds_type.clone(),
        plugin_version: resolved.plugin_version.clone(),
    };
    ref_mapping.insert(mapping_key, mapping.clone());
    mapping
}

#[derive(Clone, Debug)]
struct ConstantInputMapping {
    input_name: String,
    label: String,
    value: Value,
}

fn make_constant_input_name(name: &str) -> String {
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

fn collect_and_rewrite_constant_variables(dashboard: &mut Value) -> Vec<ConstantInputMapping> {
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

fn replace_datasource_refs_in_dashboard(
    node: &mut Value,
    ref_mapping: &BTreeMap<String, InputMapping>,
    datasource_catalog: &DatasourceCatalog,
) -> Result<()> {
    match node {
        Value::Object(object) => {
            if let Some(datasource_value) = object.get_mut("datasource") {
                if let Some(resolved) =
                    resolve_datasource_ref(datasource_value, datasource_catalog)?
                {
                    let mapping = ref_mapping.get(&resolved.key).ok_or_else(|| {
                        message(format!(
                            "Missing datasource input mapping for {}",
                            resolved.key
                        ))
                    })?;
                    let placeholder = format!("${{{}}}", mapping.input_name);
                    let replacement = if datasource_value.is_object() {
                        let mut replacement = Map::new();
                        replacement.insert("uid".to_string(), Value::String(placeholder));
                        if !resolved.ds_type.is_empty() {
                            replacement.insert("type".to_string(), Value::String(resolved.ds_type));
                        }
                        Value::Object(replacement)
                    } else {
                        Value::String(placeholder)
                    };
                    *datasource_value = replacement;
                }
            }
            let keys = object.keys().cloned().collect::<Vec<String>>();
            for key in keys {
                if key == "datasource" {
                    continue;
                }
                if let Some(value) = object.get_mut(&key) {
                    replace_datasource_refs_in_dashboard(value, ref_mapping, datasource_catalog)?;
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                replace_datasource_refs_in_dashboard(item, ref_mapping, datasource_catalog)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn collect_panel_types(panels: &[Value], panel_types: &mut BTreeSet<String>) {
    for panel in panels {
        collect_panel_types_from_value(panel, panel_types);
    }
}

fn collect_panel_types_from_value(panel: &Value, panel_types: &mut BTreeSet<String>) {
    let Some(panel_object) = panel.as_object() else {
        if let Some(items) = panel.as_array() {
            collect_panel_types(items, panel_types);
        }
        return;
    };
    let panel_type = string_field(panel_object, "type", "");
    if !panel_type.is_empty() {
        panel_types.insert(panel_type);
    }
    if let Some(nested) = panel_object.get("panels").and_then(Value::as_array) {
        collect_panel_types(nested, panel_types);
    }
}

fn build_input_definitions(
    ref_mapping: &BTreeMap<String, InputMapping>,
    constants: &[ConstantInputMapping],
) -> Value {
    let mut mappings = ref_mapping.values().cloned().collect::<Vec<InputMapping>>();
    mappings.sort_by(|left, right| left.input_name.cmp(&right.input_name));
    let mut inputs = mappings
        .into_iter()
        .map(|mapping| {
            json!({
                "name": mapping.input_name,
                "label": mapping.label,
                "description": "",
                "type": "datasource",
                "pluginId": mapping.ds_type,
                "pluginName": mapping.plugin_name,
            })
        })
        .collect::<Vec<Value>>();
    let mut constants = constants.to_vec();
    constants.sort_by(|left, right| left.input_name.cmp(&right.input_name));
    inputs.extend(constants.into_iter().map(|constant| {
        json!({
            "name": constant.input_name,
            "type": "constant",
            "label": constant.label,
            "value": constant.value,
            "description": "",
        })
    }));
    Value::Array(inputs)
}

fn collect_datasource_variable_reference_names(node: &Value, names: &mut BTreeSet<String>) {
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

fn map_datasource_variable_current_inputs(
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

fn build_requires_block(
    ref_mapping: &BTreeMap<String, InputMapping>,
    panel_types: &BTreeSet<String>,
) -> Value {
    let mut requires = vec![json!({
        "type": "grafana",
        "id": "grafana",
        "name": "Grafana",
        "version": "",
    })];
    let mut datasource_mappings = ref_mapping.values().cloned().collect::<Vec<_>>();
    datasource_mappings.sort_by(|left, right| left.input_name.cmp(&right.input_name));
    for mapping in datasource_mappings {
        requires.push(json!({
            "type": "datasource",
            "id": mapping.ds_type,
            "name": mapping.plugin_name,
            "version": mapping.plugin_version,
        }));
    }
    for panel_type in panel_types {
        requires.push(json!({
            "type": "panel",
            "id": panel_type,
            "name": format_panel_plugin_name(panel_type),
            "version": "",
        }));
    }
    Value::Array(requires)
}

pub(crate) fn collect_library_panel_uids(document: &Value) -> BTreeSet<String> {
    let mut uids = BTreeSet::new();
    collect_library_panel_uids_recursive(document, &mut uids);
    uids
}

fn collect_library_panel_uids_recursive(node: &Value, uids: &mut BTreeSet<String>) {
    match node {
        Value::Object(object) => {
            if let Some(library_panel) = object.get("libraryPanel").and_then(Value::as_object) {
                if let Some(uid) = library_panel.get("uid").and_then(Value::as_str) {
                    let uid = uid.trim();
                    if !uid.is_empty() {
                        uids.insert(uid.to_string());
                    }
                }
            }
            for value in object.values() {
                collect_library_panel_uids_recursive(value, uids);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_library_panel_uids_recursive(item, uids);
            }
        }
        _ => {}
    }
}

pub(crate) fn build_library_panel_export(payload: &Value) -> Result<(String, Value)> {
    let object = value_as_object(payload, "Unexpected library panel payload from Grafana.")?;
    let result = object
        .get("result")
        .ok_or_else(|| message("Unexpected library panel payload from Grafana."))?;
    let result_object = value_as_object(result, "Unexpected library panel payload from Grafana.")?;

    let uid = string_field(result_object, "uid", "");
    if uid.is_empty() {
        return Err(message("Unexpected library panel payload from Grafana."));
    }
    let name = string_field(result_object, "name", &uid);
    let kind = result_object
        .get("kind")
        .and_then(Value::as_i64)
        .unwrap_or(1);
    let panel_type = string_field(result_object, "type", "");
    let mut model = result_object
        .get("model")
        .cloned()
        .ok_or_else(|| message("Unexpected library panel payload from Grafana."))?;
    let model_object = model
        .as_object_mut()
        .ok_or_else(|| message("Unexpected library panel model from Grafana."))?;
    model_object.remove("gridPos");
    model_object.remove("id");
    model_object.remove("libraryPanel");
    let model_type = model_object
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    Ok((
        uid.clone(),
        json!({
            "name": name,
            "uid": uid,
            "kind": kind,
            "type": if panel_type.is_empty() {
                model_type
            } else {
                panel_type
            },
            "model": model,
        }),
    ))
}

pub fn build_external_export_document(
    payload: &Value,
    datasource_catalog: &DatasourceCatalog,
) -> Result<Value> {
    build_external_export_document_with_library_panels(payload, datasource_catalog, None)
}

pub(crate) fn build_external_export_document_with_library_panels(
    payload: &Value,
    datasource_catalog: &DatasourceCatalog,
    library_panels: Option<&BTreeMap<String, Value>>,
) -> Result<Value> {
    let mut dashboard = build_preserved_web_import_document(payload)?;
    let dashboard_object = dashboard
        .as_object_mut()
        .ok_or_else(|| message("Unexpected dashboard payload from Grafana."))?;

    let mut refs = Vec::new();
    collect_datasource_refs(&Value::Object(dashboard_object.clone()), &mut refs);
    let mut used_datasource_variable_names = BTreeSet::new();
    collect_datasource_variable_reference_names(
        &Value::Object(dashboard_object.clone()),
        &mut used_datasource_variable_names,
    );
    let mut library_panel_models = Vec::new();
    if let Some(library_panels) = library_panels {
        for library_panel in library_panels.values() {
            let Some(model) = library_panel.get("model") else {
                continue;
            };
            collect_datasource_refs(model, &mut refs);
            library_panel_models.push(model.clone());
        }
    }

    let mut ref_mapping = BTreeMap::new();
    let mut type_counts = BTreeMap::new();
    for reference in refs {
        let Some(resolved) = resolve_datasource_ref(&reference, datasource_catalog)? else {
            continue;
        };
        if ref_mapping.contains_key(&resolved.key) {
            continue;
        }
        allocate_input_mapping(&resolved, &mut ref_mapping, &mut type_counts, None);
    }

    map_datasource_variable_current_inputs(
        &mut dashboard,
        &used_datasource_variable_names,
        &mut ref_mapping,
        &mut type_counts,
        datasource_catalog,
    )?;

    replace_datasource_refs_in_dashboard(&mut dashboard, &ref_mapping, datasource_catalog)?;
    let constants = collect_and_rewrite_constant_variables(&mut dashboard);

    let mut panel_types = BTreeSet::new();
    if let Some(panels) = dashboard.get("panels").and_then(Value::as_array) {
        collect_panel_types(panels, &mut panel_types);
    }
    for model in &library_panel_models {
        collect_panel_types_from_value(model, &mut panel_types);
    }
    let mut elements = Map::new();
    if let Some(library_panels) = library_panels {
        for (uid, library_panel) in library_panels {
            let library_panel_object = value_as_object(
                library_panel,
                "Unexpected library panel export payload from Grafana.",
            )?;
            let name = string_field(library_panel_object, "name", uid);
            let kind = library_panel_object
                .get("kind")
                .and_then(Value::as_i64)
                .unwrap_or(1);
            let panel_type = string_field(library_panel_object, "type", "");
            let mut model = library_panel_object
                .get("model")
                .cloned()
                .ok_or_else(|| message("Unexpected library panel export payload from Grafana."))?;
            let model_type = model
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            replace_datasource_refs_in_dashboard(&mut model, &ref_mapping, datasource_catalog)?;
            elements.insert(
                uid.clone(),
                json!({
                    "name": name,
                    "uid": uid,
                    "kind": kind,
                    "type": if panel_type.is_empty() {
                        model_type
                    } else {
                        panel_type
                    },
                    "model": model,
                }),
            );
        }
    }
    let dashboard_object = dashboard
        .as_object_mut()
        .ok_or_else(|| message("Unexpected dashboard payload from Grafana."))?;
    dashboard_object.insert(
        "__inputs".to_string(),
        build_input_definitions(&ref_mapping, &constants),
    );
    dashboard_object.insert(
        "__requires".to_string(),
        build_requires_block(&ref_mapping, &panel_types),
    );
    dashboard_object.insert("__elements".to_string(), Value::Object(elements));
    Ok(dashboard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn datasource_catalog() -> DatasourceCatalog {
        build_datasource_catalog(&[json!({
            "uid": "prom-main",
            "name": "Prometheus Main",
            "type": "prometheus",
            "pluginId": "prometheus",
            "pluginName": "Prometheus",
        })
        .as_object()
        .unwrap()
        .clone()])
    }

    #[test]
    fn build_library_panel_export_strips_identity_fields() {
        let (uid, export) = build_library_panel_export(&json!({
            "result": {
                "uid": "shared-panel",
                "name": "Shared Panel",
                "kind": 1,
                "type": "graph",
                "model": {
                    "id": 11,
                    "gridPos": {"x": 0, "y": 0, "w": 12, "h": 8},
                    "libraryPanel": {"uid": "shared-panel", "name": "Shared Panel"},
                    "type": "graph",
                    "datasource": {"uid": "prom-main", "type": "prometheus"}
                }
            }
        }))
        .unwrap();

        assert_eq!(uid, "shared-panel");
        assert_eq!(export["name"], json!("Shared Panel"));
        assert_eq!(export["kind"], json!(1));
        assert_eq!(export["type"], json!("graph"));
        assert!(export["model"].get("id").is_none());
        assert!(export["model"].get("gridPos").is_none());
        assert!(export["model"].get("libraryPanel").is_none());
    }

    #[test]
    fn build_external_export_document_includes_library_panel_elements_when_available() {
        let library_panels = BTreeMap::from([(
            "shared-panel".to_string(),
            json!({
                "name": "Shared Panel",
                "uid": "shared-panel",
                "kind": 1,
                "type": "graph",
                "model": {
                    "type": "graph",
                    "datasource": {"uid": "prom-main", "type": "prometheus"}
                }
            }),
        )]);

        let document = build_external_export_document_with_library_panels(
            &json!({
                "uid": "cpu-main",
                "title": "CPU Main",
                "panels": [{
                    "id": 1,
                    "type": "graph",
                    "datasource": {"uid": "prom-main", "type": "prometheus"},
                    "libraryPanel": {"uid": "shared-panel", "name": "Shared Panel"},
                    "targets": []
                }]
            }),
            &datasource_catalog(),
            Some(&library_panels),
        )
        .unwrap();

        assert_eq!(
            document["__elements"]["shared-panel"]["name"],
            json!("Shared Panel")
        );
        assert_eq!(document["__elements"]["shared-panel"]["kind"], json!(1));
        assert_eq!(
            document["__elements"]["shared-panel"]["type"],
            json!("graph")
        );
        assert!(
            document["__elements"]["shared-panel"]["model"]["datasource"]["uid"]
                .as_str()
                .is_some_and(|value| value.starts_with("${DS_"))
        );
        assert_eq!(
            document["panels"][0]["libraryPanel"]["uid"],
            json!("shared-panel")
        );
        assert!(!document["__inputs"].as_array().unwrap().is_empty());
    }
}
