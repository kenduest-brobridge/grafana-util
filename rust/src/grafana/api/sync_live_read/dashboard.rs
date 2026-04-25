use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
#[cfg(test)]
use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, Result};
#[cfg(test)]
use crate::sync::require_json_object;

use super::SyncLiveClient;

const DASHBOARD_DETAIL_FETCH_CONCURRENCY: usize = 8;

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
    append_dashboard_specs_from_summaries(
        specs,
        client.list_dashboard_summaries(page_size)?,
        |uid| client.fetch_dashboard(uid),
    )
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
            append_dashboard_spec(specs, uid, dashboard_wrapper)?;
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
    dashboard_wrapper: Value,
) -> Result<()> {
    let mut wrapper = match dashboard_wrapper {
        Value::Object(wrapper) => wrapper,
        _ => return Err(message("Grafana dashboard payload")),
    };
    let dashboard = wrapper
        .remove("dashboard")
        .ok_or_else(|| message(format!("Unexpected dashboard payload for UID {uid}.")))?;
    let mut normalized = match dashboard {
        Value::Object(body) => body,
        _ => return Err(message("Grafana dashboard body")),
    };
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

fn append_dashboard_specs_from_summaries<F>(
    specs: &mut Vec<Value>,
    summaries: Vec<Map<String, Value>>,
    fetch_dashboard: F,
) -> Result<()>
where
    F: Fn(&str) -> Result<Value> + Sync,
{
    let uids = summaries
        .iter()
        .filter_map(|summary| {
            let uid = summary_uid(summary);
            (!uid.is_empty()).then(|| uid.to_string())
        })
        .collect::<Vec<_>>();
    if uids.is_empty() {
        return Ok(());
    }

    let pool = ThreadPoolBuilder::new()
        .num_threads(DASHBOARD_DETAIL_FETCH_CONCURRENCY)
        .build()
        .map_err(|error| {
            message(format!(
                "Failed to build dashboard detail read worker pool: {error}"
            ))
        })?;
    let reads = pool.install(|| {
        uids.par_iter()
            .map(|uid| (uid.clone(), fetch_dashboard(uid)))
            .collect::<Vec<_>>()
    });

    let failures = reads
        .iter()
        .filter_map(|(uid, result)| {
            result
                .as_ref()
                .err()
                .map(|error| format!("uid={uid}: {error}"))
        })
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        return Err(message(format!(
            "Failed to fetch {} dashboard detail(s) after /api/search: {}",
            failures.len(),
            failures.join("; ")
        )));
    }

    let mut dashboard_specs = Vec::new();
    for (uid, result) in reads {
        let dashboard_wrapper = result?;
        append_dashboard_spec(&mut dashboard_specs, &uid, dashboard_wrapper)?;
    }
    specs.extend(dashboard_specs);
    Ok(())
}

fn summary_uid(summary: &Map<String, Value>) -> &str {
    summary
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(uid: &str) -> Map<String, Value> {
        let mut object = Map::new();
        object.insert("uid".to_string(), Value::String(uid.to_string()));
        object
    }

    fn dashboard_payload(title: &str) -> Value {
        serde_json::json!({
            "dashboard": {
                "id": 123,
                "title": title,
                "panels": []
            }
        })
    }

    #[test]
    fn append_dashboard_specs_from_summaries_preserves_search_order() {
        let mut specs = Vec::new();

        append_dashboard_specs_from_summaries(
            &mut specs,
            vec![summary("dash-b"), summary("dash-a"), summary("dash-c")],
            |uid| Ok(dashboard_payload(&format!("title-{uid}"))),
        )
        .unwrap();

        let uids = specs
            .iter()
            .map(|spec| spec["uid"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(uids, vec!["dash-b", "dash-a", "dash-c"]);
    }

    #[test]
    fn append_dashboard_specs_from_summaries_aggregates_detail_failures() {
        let mut specs = vec![serde_json::json!({"kind": "folder", "uid": "f"})];

        let error = append_dashboard_specs_from_summaries(
            &mut specs,
            vec![summary("dash-a"), summary("dash-b"), summary("dash-c")],
            |uid| {
                if uid == "dash-b" || uid == "dash-c" {
                    Err(message(format!("boom {uid}")))
                } else {
                    Ok(dashboard_payload(&format!("title-{uid}")))
                }
            },
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("Failed to fetch 2 dashboard detail(s) after /api/search"));
        assert!(error.contains("uid=dash-b: boom dash-b"));
        assert!(error.contains("uid=dash-c: boom dash-c"));
        assert_eq!(
            specs,
            vec![serde_json::json!({"kind": "folder", "uid": "f"})]
        );
    }
}
