use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

use super::helpers::{format_panel_plugin_name, make_input_label, make_input_name, InputMapping};
use super::variables::ConstantInputMapping;

pub(super) fn allocate_input_mapping(
    resolved: &super::helpers::ResolvedDatasource,
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

pub(super) fn build_input_definitions(
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
            "label": constant.label,
            "type": "constant",
            "value": constant.value,
            "description": "",
        })
    }));
    Value::Array(inputs)
}

pub(super) fn build_requires_block(
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
