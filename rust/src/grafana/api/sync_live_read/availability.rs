use serde_json::{Map, Value};

use super::super::availability_key;
#[cfg(test)]
use crate::common::Result;
use crate::sync::append_unique_strings;
#[cfg(test)]
use crate::sync::require_json_object;

pub(super) fn empty_document() -> Map<String, Value> {
    Map::from_iter(vec![
        (
            availability_key::DATASOURCE_UIDS.to_string(),
            Value::Array(Vec::new()),
        ),
        (
            availability_key::DATASOURCE_NAMES.to_string(),
            Value::Array(Vec::new()),
        ),
        (
            availability_key::PLUGIN_IDS.to_string(),
            Value::Array(Vec::new()),
        ),
        (
            availability_key::CONTACT_POINTS.to_string(),
            Value::Array(Vec::new()),
        ),
    ])
}

pub(super) fn array_mut<'a>(
    availability: &'a mut Map<String, Value>,
    key: &'static str,
) -> &'a mut Vec<Value> {
    availability
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .unwrap_or_else(|| panic!("{key} should be array"))
}

pub(super) fn append_plugin_ids_from_slice(
    plugins: &[Map<String, Value>],
    availability: &mut Map<String, Value>,
) {
    append_plugin_ids(plugins.iter(), availability);
}

#[cfg(test)]
pub(super) fn append_plugin_ids_from_values(
    plugins: &[Value],
    availability: &mut Map<String, Value>,
) {
    let ids = plugins
        .iter()
        .filter_map(|plugin| plugin.get("id").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    append_unique_strings(array_mut(availability, availability_key::PLUGIN_IDS), &ids);
}

pub(super) fn append_plugin_ids<'a, I>(plugins: I, availability: &mut Map<String, Value>)
where
    I: IntoIterator<Item = &'a Map<String, Value>>,
{
    let ids = plugins
        .into_iter()
        .filter_map(|plugin| plugin.get("id").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    append_unique_strings(array_mut(availability, availability_key::PLUGIN_IDS), &ids);
}

pub(super) fn append_contact_point_identifiers_from_slice(
    contact_points: &[Map<String, Value>],
    availability: &mut Map<String, Value>,
) {
    append_contact_point_identifiers(contact_points.iter(), availability);
}

pub(super) fn append_contact_point_identifiers<'a, I>(
    contact_points: I,
    availability: &mut Map<String, Value>,
) where
    I: IntoIterator<Item = &'a Map<String, Value>>,
{
    let mut names = Vec::new();
    for contact_point in contact_points {
        if let Some(name) = contact_point
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value: &&str| !value.is_empty())
        {
            names.push(name.to_string());
        }
        if let Some(uid) = contact_point
            .get("uid")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value: &&str| !value.is_empty())
        {
            names.push(uid.to_string());
        }
    }
    append_unique_strings(
        array_mut(availability, availability_key::CONTACT_POINTS),
        &names,
    );
}

#[cfg(test)]
pub(super) fn append_contact_point_identifiers_from_values(
    contact_points: &[Value],
    availability: &mut Map<String, Value>,
) -> Result<()> {
    let mut objects = Vec::with_capacity(contact_points.len());
    for contact_point in contact_points {
        objects.push(require_json_object(contact_point, "Grafana contact-point payload")?.clone());
    }
    append_contact_point_identifiers(objects.iter(), availability);
    Ok(())
}
