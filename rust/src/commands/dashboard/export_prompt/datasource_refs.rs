use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};

use super::helpers::{
    build_resolved_datasource, datasource_plugin_version, format_plugin_name,
    is_builtin_datasource_ref, is_placeholder_string, lookup_datasource,
    resolve_datasource_type_alias, DatasourceCatalog, ResolvedDatasource,
};

pub(super) fn resolve_string_datasource_ref(
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

pub(super) fn resolve_object_datasource_ref(
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

pub(super) fn resolve_datasource_ref(
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

pub(super) fn should_promote_generic_datasource_ref(
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
