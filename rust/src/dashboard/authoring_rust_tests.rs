//! Runtime tests for dashboard live authoring commands.
use super::authoring::{
    clone_live_dashboard_to_file_with_request, get_live_dashboard_to_file_with_request,
};
use crate::common::GrafanaCliError;
use serde_json::{json, Value};
use std::fs;
use tempfile::tempdir;

type TestRequestResult = crate::common::Result<Option<Value>>;

fn read_json_output_file(path: &std::path::Path) -> Value {
    let raw = fs::read_to_string(path).unwrap();
    assert!(
        raw.ends_with('\n'),
        "expected output file {} to end with a newline",
        path.display()
    );
    serde_json::from_str(&raw).unwrap()
}

fn live_dashboard_payload() -> Value {
    json!({
        "dashboard": {
            "id": 42,
            "uid": "cpu-main",
            "title": "CPU Main",
            "schemaVersion": 39,
            "tags": ["ops"]
        },
        "meta": {
            "folderUid": "infra",
            "folderTitle": "Infra",
            "slug": "cpu-main"
        }
    })
}

#[allow(clippy::type_complexity)]
fn dashboard_request_fixture(
    payload: Value,
) -> impl FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult {
    move |method, path, _params, _payload| match (method, path) {
        (reqwest::Method::GET, "/api/dashboards/uid/cpu-main") => Ok(Some(payload.clone())),
        _ => Err(crate::common::message(format!("unexpected request {path}"))),
    }
}

#[test]
fn get_live_dashboard_to_file_with_request_writes_id_null_and_preserves_meta() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("cpu-main.json");

    let document = get_live_dashboard_to_file_with_request(
        dashboard_request_fixture(live_dashboard_payload()),
        "cpu-main",
        &output,
    )
    .unwrap();

    assert_eq!(document["dashboard"]["id"], Value::Null);
    assert_eq!(document["dashboard"]["uid"], "cpu-main");
    assert_eq!(document["dashboard"]["title"], "CPU Main");
    assert_eq!(document["meta"]["folderUid"], "infra");
    assert_eq!(read_json_output_file(&output), document);
}

#[test]
fn clone_live_dashboard_to_file_with_request_applies_overrides() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("cpu-main-clone.json");

    let document = clone_live_dashboard_to_file_with_request(
        dashboard_request_fixture(live_dashboard_payload()),
        "cpu-main",
        &output,
        Some("CPU Clone"),
        Some("cpu-main-clone"),
        Some("ops"),
    )
    .unwrap();

    assert_eq!(document["dashboard"]["id"], Value::Null);
    assert_eq!(document["dashboard"]["uid"], "cpu-main-clone");
    assert_eq!(document["dashboard"]["title"], "CPU Clone");
    assert_eq!(document["meta"]["folderUid"], "ops");
    assert_eq!(document["meta"]["folderTitle"], "Infra");
    assert_eq!(read_json_output_file(&output), document);
}

#[test]
fn get_live_dashboard_to_file_with_request_errors_on_missing_dashboard() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("missing.json");

    let error = get_live_dashboard_to_file_with_request(
        |_method, _path, _params, _payload| Ok(None),
        "cpu-main",
        &output,
    )
    .unwrap_err();

    assert!(matches!(error, GrafanaCliError::Message(_)));
    assert!(!output.exists());
}
