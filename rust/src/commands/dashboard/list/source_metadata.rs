use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{string_field, value_as_object, Result};
use crate::dashboard::import::build_dashboard_target_review;
use crate::dashboard::import::target::dashboard_target_review_ownership_label;

use super::super::{
    collect_datasource_refs, datasource_type_alias, extract_dashboard_object,
    is_builtin_datasource_ref, is_placeholder_string, lookup_datasource,
    resolve_datasource_type_alias,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardOwnershipProvenance {
    pub(crate) ownership: String,
    pub(crate) provenance: Vec<String>,
}

fn lookup_unique_datasource_name_by_type(
    datasources_by_uid: &BTreeMap<String, Map<String, Value>>,
    datasource_type: &str,
) -> Option<String> {
    let matches: BTreeSet<String> = datasources_by_uid
        .values()
        .filter(|datasource| {
            string_field(datasource, "type", "").eq_ignore_ascii_case(datasource_type)
        })
        .map(|datasource| {
            let name = string_field(datasource, "name", "");
            if name.is_empty() {
                string_field(datasource, "uid", datasource_type)
            } else {
                name
            }
        })
        .collect();
    if matches.len() == 1 {
        matches.iter().next().cloned()
    } else {
        None
    }
}

fn resolve_datasource_source_name(
    reference: &Value,
    datasource_catalog: &super::super::prompt::DatasourceCatalog,
) -> Option<String> {
    if reference.is_null() || is_builtin_datasource_ref(reference) {
        return None;
    }
    match reference {
        Value::String(text) => {
            if is_placeholder_string(text) {
                return None;
            }
            if let Some(datasource) = lookup_datasource(datasource_catalog, Some(text), Some(text))
            {
                let name = string_field(&datasource, "name", text);
                return Some(name);
            }
            resolve_datasource_type_alias(text, datasource_catalog)
                .and_then(|datasource_type| {
                    lookup_unique_datasource_name_by_type(
                        &datasource_catalog.by_uid,
                        &datasource_type,
                    )
                    .or_else(|| Some(datasource_type_alias(&datasource_type).to_string()))
                })
                .or_else(|| Some(text.to_string()))
        }
        Value::Object(object) => {
            let uid = object.get("uid").and_then(Value::as_str);
            let name = object.get("name").and_then(Value::as_str);
            let datasource_type = object.get("type").and_then(Value::as_str);
            let has_placeholder =
                uid.is_some_and(is_placeholder_string) || name.is_some_and(is_placeholder_string);
            if has_placeholder {
                return None;
            }
            if let Some(datasource) = lookup_datasource(datasource_catalog, uid, name) {
                let resolved_name = string_field(
                    &datasource,
                    "name",
                    uid.or(name)
                        .unwrap_or_else(|| datasource_type.unwrap_or("")),
                );
                if !resolved_name.is_empty() {
                    return Some(resolved_name);
                }
            }
            name.map(str::to_string)
                .or_else(|| uid.map(str::to_string))
                .or_else(|| {
                    datasource_type.and_then(|value| {
                        lookup_unique_datasource_name_by_type(&datasource_catalog.by_uid, value)
                            .or_else(|| Some(datasource_type_alias(value).to_string()))
                    })
                })
        }
        _ => None,
    }
}

fn resolve_datasource_source_uid(
    reference: &Value,
    datasource_catalog: &super::super::prompt::DatasourceCatalog,
) -> Option<String> {
    if reference.is_null() || is_builtin_datasource_ref(reference) {
        return None;
    }
    match reference {
        Value::String(text) => {
            if is_placeholder_string(text) {
                return None;
            }
            lookup_datasource(datasource_catalog, Some(text), Some(text))
                .map(|datasource| string_field(&datasource, "uid", ""))
                .filter(|uid| !uid.is_empty())
        }
        Value::Object(object) => {
            let uid = object.get("uid").and_then(Value::as_str);
            let name = object.get("name").and_then(Value::as_str);
            let has_placeholder =
                uid.is_some_and(is_placeholder_string) || name.is_some_and(is_placeholder_string);
            if has_placeholder {
                return None;
            }
            if let Some(datasource) = lookup_datasource(datasource_catalog, uid, name) {
                let resolved_uid = string_field(&datasource, "uid", "");
                if !resolved_uid.is_empty() {
                    return Some(resolved_uid);
                }
            }
            uid.filter(|value| !value.is_empty()).map(str::to_string)
        }
        _ => None,
    }
}

pub(crate) fn collect_dashboard_ownership_provenance(
    payload: &Value,
) -> Result<DashboardOwnershipProvenance> {
    let review = build_dashboard_target_review(payload)?;
    let ownership = dashboard_target_review_ownership_label(&review).to_string();
    let ownership_note = format!("ownership={ownership}");
    let mut provenance = review.evidence;
    if !provenance.iter().any(|value| value == &ownership_note) {
        provenance.insert(0, ownership_note);
    }
    Ok(DashboardOwnershipProvenance {
        ownership,
        provenance,
    })
}

/// collect dashboard source metadata.
pub(crate) fn collect_dashboard_source_metadata(
    payload: &Value,
    datasource_catalog: &super::super::prompt::DatasourceCatalog,
) -> Result<(Vec<String>, Vec<String>)> {
    let payload_object = value_as_object(payload, "Unexpected dashboard payload from Grafana.")?;
    let dashboard_object = extract_dashboard_object(payload_object)?;
    let mut refs = Vec::new();
    collect_datasource_refs(&Value::Object(dashboard_object.clone()), &mut refs);
    let mut names = BTreeSet::new();
    let mut uids = BTreeSet::new();
    for reference in refs {
        if let Some(name) = resolve_datasource_source_name(&reference, datasource_catalog) {
            names.insert(name);
        }
        if let Some(uid) = resolve_datasource_source_uid(&reference, datasource_catalog) {
            uids.insert(uid);
        }
    }
    Ok((names.into_iter().collect(), uids.into_iter().collect()))
}
