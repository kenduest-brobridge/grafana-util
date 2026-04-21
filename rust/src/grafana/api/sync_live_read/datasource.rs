#[cfg(test)]
use reqwest::Method;
use serde_json::{Map, Value};

#[cfg(test)]
use crate::common::message;
use crate::common::Result;
use crate::sync::append_unique_strings;
#[cfg(test)]
use crate::sync::require_json_object;

use super::super::{availability_key, SyncLiveClient};
use super::availability;

pub(super) fn append_datasource_resource_specs_from_client(
    client: &SyncLiveClient<'_>,
    specs: &mut Vec<Value>,
) -> Result<()> {
    for datasource in client.list_datasources()? {
        append_datasource_resource_spec(&datasource, specs);
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn append_datasource_resource_specs_with_request<F>(
    request_json: &mut F,
    specs: &mut Vec<Value>,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, "/api/datasources", &[], None)? {
        Some(Value::Array(datasources)) => {
            for datasource in datasources {
                let object = require_json_object(&datasource, "Grafana datasource payload")?;
                append_datasource_resource_spec(object, specs);
            }
        }
        Some(_) => return Err(message("Unexpected datasource list response from Grafana.")),
        None => {}
    }
    Ok(())
}

pub(super) fn append_datasource_availability_from_client(
    client: &SyncLiveClient<'_>,
    availability: &mut Map<String, Value>,
) -> Result<()> {
    let datasources = client.list_datasources()?;
    append_datasource_availability(datasources.iter(), availability)
}

#[cfg(test)]
pub(super) fn append_datasource_availability_with_request<F>(
    request_json: &mut F,
    availability: &mut Map<String, Value>,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, "/api/datasources", &[], None)? {
        Some(Value::Array(datasources)) => {
            let mut objects = Vec::with_capacity(datasources.len());
            for datasource in datasources {
                objects
                    .push(require_json_object(&datasource, "Grafana datasource payload")?.clone());
            }
            append_datasource_availability(objects.iter(), availability)?;
        }
        Some(_) => return Err(message("Unexpected datasource list response from Grafana.")),
        None => {}
    }
    Ok(())
}

fn append_datasource_resource_spec(datasource: &Map<String, Value>, specs: &mut Vec<Value>) {
    let uid = datasource
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let name = datasource
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if uid.is_empty() && name.is_empty() {
        return;
    }

    let title = if name.is_empty() { uid } else { name };
    let mut body = Map::new();
    body.insert("uid".to_string(), Value::String(uid.to_string()));
    body.insert("name".to_string(), Value::String(title.to_string()));
    body.insert(
        "type".to_string(),
        datasource
            .get("type")
            .cloned()
            .unwrap_or(Value::String(String::new())),
    );
    body.insert(
        "access".to_string(),
        datasource
            .get("access")
            .cloned()
            .unwrap_or(Value::String(String::new())),
    );
    body.insert(
        "url".to_string(),
        datasource
            .get("url")
            .cloned()
            .unwrap_or(Value::String(String::new())),
    );
    body.insert(
        "isDefault".to_string(),
        datasource
            .get("isDefault")
            .cloned()
            .unwrap_or(Value::Bool(false)),
    );
    if let Some(json_data) = datasource.get("jsonData").and_then(Value::as_object) {
        if !json_data.is_empty() {
            body.insert("jsonData".to_string(), Value::Object(json_data.clone()));
        }
    }

    specs.push(serde_json::json!({
        "kind": "datasource",
        "uid": uid,
        "name": title,
        "title": title,
        "body": body,
    }));
}

fn append_datasource_availability<'a, I>(
    datasources: I,
    availability: &mut Map<String, Value>,
) -> Result<()>
where
    I: IntoIterator<Item = &'a Map<String, Value>>,
{
    let mut uids = Vec::new();
    let mut names = Vec::new();
    for datasource in datasources {
        if let Some(uid) = datasource
            .get("uid")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value: &&str| !value.is_empty())
        {
            uids.push(uid.to_string());
        }
        if let Some(name) = datasource
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value: &&str| !value.is_empty())
        {
            names.push(name.to_string());
        }
    }
    append_unique_strings(
        availability::array_mut(availability, availability_key::DATASOURCE_UIDS),
        &uids,
    );
    append_unique_strings(
        availability::array_mut(availability, availability_key::DATASOURCE_NAMES),
        &names,
    );
    Ok(())
}
