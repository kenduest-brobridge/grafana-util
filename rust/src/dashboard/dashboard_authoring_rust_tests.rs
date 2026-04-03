//! Dashboard authoring regression tests for local patch and publish wrappers.
use super::make_common_args;
use crate::dashboard::authoring::publish_dashboard_with_request;
use crate::dashboard::{patch_dashboard_file, PatchFileArgs, PublishArgs};
use serde_json::{json, Value};
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

fn read_json_output_file(path: &std::path::Path) -> Value {
    let raw = fs::read_to_string(path).unwrap();
    serde_json::from_str(&raw).unwrap()
}

#[test]
fn patch_dashboard_file_updates_in_place() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("cpu-main.json");
    fs::write(
        &input,
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "id": 7,
                "uid": "cpu-old",
                "title": "CPU",
                "schemaVersion": 38,
                "tags": ["legacy"]
            },
            "meta": {
                "folderUid": "old-folder"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    patch_dashboard_file(&PatchFileArgs {
        input: input.clone(),
        output: None,
        name: Some("CPU Overview".to_string()),
        uid: Some("cpu-main".to_string()),
        folder_uid: Some("infra".to_string()),
        message: Some("Promote CPU dashboard".to_string()),
        tags: vec!["prod".to_string(), "sre".to_string()],
    })
    .unwrap();

    let patched: Value = read_json_output_file(&input);
    assert_eq!(patched["dashboard"]["title"], "CPU Overview");
    assert_eq!(patched["dashboard"]["uid"], "cpu-main");
    assert_eq!(patched["dashboard"]["tags"], json!(["prod", "sre"]));
    assert_eq!(patched["meta"]["folderUid"], "infra");
    assert_eq!(patched["meta"]["message"], "Promote CPU dashboard");
}

#[test]
fn patch_dashboard_file_writes_to_explicit_output_path() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("cpu-main.json");
    let output = temp.path().join("cpu-main-patched.json");
    fs::write(
        &input,
        serde_json::to_string_pretty(&json!({
            "uid": "cpu-main",
            "title": "CPU",
            "schemaVersion": 38,
            "tags": ["legacy"]
        }))
        .unwrap(),
    )
    .unwrap();

    patch_dashboard_file(&PatchFileArgs {
        input: input.clone(),
        output: Some(output.clone()),
        name: Some("CPU Overview".to_string()),
        uid: None,
        folder_uid: Some("infra".to_string()),
        message: None,
        tags: vec!["prod".to_string()],
    })
    .unwrap();

    let input_document: Value = serde_json::from_str(&fs::read_to_string(&input).unwrap()).unwrap();
    let output_document: Value = read_json_output_file(&output);
    assert_eq!(input_document["title"], "CPU");
    assert_eq!(output_document["title"], "CPU Overview");
    assert_eq!(output_document["uid"], "cpu-main");
    assert_eq!(output_document["tags"], json!(["prod"]));
    assert_eq!(output_document["meta"]["folderUid"], "infra");
}

#[test]
fn publish_dashboard_with_request_posts_staged_single_file() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("cpu-main.json");
    fs::write(
        &input,
        serde_json::to_string_pretty(&json!({
            "uid": "cpu-main",
            "title": "CPU Overview",
            "schemaVersion": 38,
            "panels": []
        }))
        .unwrap(),
    )
    .unwrap();

    let payloads = Arc::new(Mutex::new(Vec::<Value>::new()));
    let recorded = payloads.clone();

    let args = PublishArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        input: input.clone(),
        replace_existing: true,
        folder_uid: Some("infra".to_string()),
        message: "Promote CPU dashboard".to_string(),
        dry_run: false,
        table: false,
        json: false,
    };

    publish_dashboard_with_request(
        move |method, path, _params, payload: Option<&Value>| {
            let method_name = method.to_string();
            match (method, path) {
                (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([
                    {"uid": "prom-main", "type": "prometheus"}
                ]))),
                (reqwest::Method::GET, "/api/plugins") => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                (reqwest::Method::POST, "/api/dashboards/db") => {
                    recorded
                        .lock()
                        .unwrap()
                        .push(payload.cloned().unwrap_or(Value::Null));
                    Ok(Some(json!({"status": "success"})))
                }
                _ => Err(crate::common::message(format!(
                    "unexpected request {method_name} {path}"
                ))),
            }
        },
        &args,
    )
    .unwrap();

    let payloads = payloads.lock().unwrap();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["dashboard"]["uid"], "cpu-main");
    assert_eq!(payloads[0]["dashboard"]["title"], "CPU Overview");
    assert_eq!(payloads[0]["folderUid"], "infra");
    assert_eq!(payloads[0]["overwrite"], true);
    assert_eq!(payloads[0]["message"], "Promote CPU dashboard");
}
