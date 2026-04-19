//! Dashboard prompt output transformation.
//! Translates query panels into external dashboard import payload shape and keeps datasource mapping helpers.
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{message, string_field, value_as_object, Result};

use super::build_preserved_web_import_document;
use super::prompt_datasource_refs::resolve_datasource_ref;
use super::prompt_inputs::{allocate_input_mapping, build_input_definitions, build_requires_block};
use super::prompt_variables::{
    collect_and_rewrite_constant_variables, collect_datasource_variable_reference_names,
    map_datasource_variable_current_inputs,
};

#[allow(unused_imports)]
pub(crate) use super::prompt_helpers::{
    build_datasource_catalog, build_resolved_datasource, collect_datasource_refs,
    datasource_plugin_version, datasource_type_alias, extract_placeholder_name,
    format_panel_plugin_name, format_plugin_name, is_builtin_datasource_ref, is_placeholder_string,
    lookup_datasource, make_input_label, make_input_name, resolve_datasource_type_alias,
    DatasourceCatalog, InputMapping, ResolvedDatasource,
};

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
