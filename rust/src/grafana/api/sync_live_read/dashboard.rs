#[cfg(test)]
use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, Result};
use crate::sync::require_json_object;

use super::SyncLiveClient;

pub(crate) fn append_folder_specs(
    client: &SyncLiveClient<'_>,
    specs: &mut Vec<Value>,
) -> Result<()> {
    for folder in client.list_folders()? {
        append_folder_spec(specs, &folder)?;
    }
    Ok(())
}

pub(crate) fn append_dashboard_specs(
    client: &SyncLiveClient<'_>,
    specs: &mut Vec<Value>,
    page_size: usize,
) -> Result<()> {
    for summary in client.list_dashboard_summaries(page_size)? {
        let uid = summary_uid(&summary);
        if uid.is_empty() {
            continue;
        }
        let dashboard_wrapper = client.fetch_dashboard(uid)?;
        append_dashboard_spec(specs, uid, &dashboard_wrapper)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn append_folder_specs_with_request<F>(
    request_json: &mut F,
    specs: &mut Vec<Value>,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, "/api/folders", &[], None)? {
        Some(Value::Array(folders)) => {
            for folder in folders {
                let object = require_json_object(&folder, "Grafana folder payload")?;
                append_folder_spec(specs, object)?;
            }
        }
        Some(_) => return Err(message("Unexpected folder list response from Grafana.")),
        None => {}
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn append_dashboard_specs_with_request<F>(
    request_json: &mut F,
    specs: &mut Vec<Value>,
    page_size: usize,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut page = 1usize;
    loop {
        let params = vec![
            ("type".to_string(), "dash-db".to_string()),
            ("limit".to_string(), page_size.to_string()),
            ("page".to_string(), page.to_string()),
        ];
        let batch = match request_json(Method::GET, "/api/search", &params, None)? {
            Some(Value::Array(items)) => items,
            Some(_) => return Err(message("Unexpected search response from Grafana.")),
            None => Vec::new(),
        };
        if batch.is_empty() {
            break;
        }
        let batch_len = batch.len();
        for item in batch {
            let summary = require_json_object(&item, "Grafana dashboard summary")?;
            let uid = summary_uid(summary);
            if uid.is_empty() {
                continue;
            }
            let dashboard_wrapper = match request_json(
                Method::GET,
                &format!("/api/dashboards/uid/{uid}"),
                &[],
                None,
            )? {
                Some(value) => value,
                None => continue,
            };
            append_dashboard_spec(specs, uid, &dashboard_wrapper)?;
        }
        if batch_len < page_size {
            break;
        }
        page += 1;
    }
    Ok(())
}

fn append_folder_spec(specs: &mut Vec<Value>, folder: &Map<String, Value>) -> Result<()> {
    let uid = folder
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if uid.is_empty() {
        return Ok(());
    }
    let title = folder
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
        .unwrap_or(uid);
    let mut body = Map::new();
    body.insert("title".to_string(), Value::String(title.to_string()));
    if let Some(parent_uid) = folder
        .get("parentUid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
    {
        body.insert(
            "parentUid".to_string(),
            Value::String(parent_uid.to_string()),
        );
    }
    specs.push(serde_json::json!({
        "kind": "folder",
        "uid": uid,
        "title": title,
        "body": body,
    }));
    Ok(())
}

fn append_dashboard_spec(
    specs: &mut Vec<Value>,
    uid: &str,
    dashboard_wrapper: &Value,
) -> Result<()> {
    let wrapper = require_json_object(dashboard_wrapper, "Grafana dashboard payload")?;
    let dashboard = wrapper
        .get("dashboard")
        .ok_or_else(|| message(format!("Unexpected dashboard payload for UID {uid}.")))?;
    let body = require_json_object(dashboard, "Grafana dashboard body")?;
    let mut normalized = body.clone();
    normalized.remove("id");
    let title = normalized
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())
        .unwrap_or(uid);
    specs.push(serde_json::json!({
        "kind": "dashboard",
        "uid": uid,
        "title": title,
        "body": normalized,
    }));
    Ok(())
}

fn summary_uid(summary: &Map<String, Value>) -> &str {
    summary
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
}
