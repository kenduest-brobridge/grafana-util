#[path = "datasource_diff.rs"]
mod datasource_diff;

use datasource_diff::{
    build_datasource_diff_report, normalize_export_records, normalize_live_records,
    DatasourceDiffStatus,
};
use serde_json::{json, Value};

fn load_contract_cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../../tests/fixtures/datasource_contract_cases.json"))
        .unwrap()
}

#[test]
fn normalize_export_records_handles_string_bools_and_org_ids() {
    let records = normalize_export_records(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": "true",
        "orgId": 7
    })]);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].uid, "prom-main");
    assert!(records[0].is_default);
    assert_eq!(records[0].org_id, "7");
}

#[test]
fn normalize_export_records_matches_shared_contract_fixtures() {
    for case in load_contract_cases() {
        let object = case.as_object().unwrap();
        let raw_datasource = object.get("rawDatasource").cloned().unwrap();
        let expected = object
            .get("expectedNormalizedRecord")
            .and_then(Value::as_object)
            .unwrap();
        let records = normalize_export_records(&[raw_datasource]);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].uid, expected.get("uid").and_then(Value::as_str).unwrap());
        assert_eq!(records[0].name, expected.get("name").and_then(Value::as_str).unwrap());
        assert_eq!(
            records[0].datasource_type,
            expected.get("type").and_then(Value::as_str).unwrap()
        );
        assert_eq!(
            records[0].access,
            expected.get("access").and_then(Value::as_str).unwrap()
        );
        assert_eq!(records[0].url, expected.get("url").and_then(Value::as_str).unwrap());
        assert_eq!(
            records[0].is_default,
            expected.get("isDefault").and_then(Value::as_str).unwrap() == "true"
        );
        assert_eq!(records[0].org_id, expected.get("orgId").and_then(Value::as_str).unwrap());
    }
}

#[test]
fn diff_report_marks_matching_records_by_uid() {
    let export_records = normalize_export_records(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    let live_records = normalize_live_records(&[json!({
        "id": 9,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": 1
    })]);

    let report = build_datasource_diff_report(&export_records, &live_records);

    assert_eq!(report.summary.compared_count, 1);
    assert_eq!(report.summary.matches_count, 1);
    assert_eq!(report.entries[0].status, DatasourceDiffStatus::Matches);
    assert!(report.entries[0].differences.is_empty());
}

#[test]
fn diff_report_captures_field_level_differences() {
    let export_records = normalize_export_records(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    let live_records = normalize_live_records(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "loki",
        "access": "direct",
        "url": "http://loki:3100",
        "isDefault": false,
        "orgId": "1"
    })]);

    let report = build_datasource_diff_report(&export_records, &live_records);

    assert_eq!(report.summary.different_count, 1);
    assert_eq!(report.entries[0].status, DatasourceDiffStatus::Different);
    assert_eq!(report.entries[0].differences.len(), 4);
    assert_eq!(report.entries[0].differences[0].field, "type");
}

#[test]
fn diff_report_marks_missing_live_records() {
    let export_records = normalize_export_records(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus"
    })]);
    let live_records = normalize_live_records(&[]);

    let report = build_datasource_diff_report(&export_records, &live_records);

    assert_eq!(report.summary.missing_in_live_count, 1);
    assert_eq!(
        report.entries[0].status,
        DatasourceDiffStatus::MissingInLive
    );
}

#[test]
fn diff_report_marks_unmatched_live_records_as_missing_in_export() {
    let export_records = normalize_export_records(&[]);
    let live_records = normalize_live_records(&[json!({
        "uid": "logs-main",
        "name": "Logs Main",
        "type": "loki"
    })]);

    let report = build_datasource_diff_report(&export_records, &live_records);

    assert_eq!(report.summary.missing_in_export_count, 1);
    assert_eq!(
        report.entries[0].status,
        DatasourceDiffStatus::MissingInExport
    );
}

#[test]
fn diff_report_marks_ambiguous_name_matches_without_uid() {
    let export_records = normalize_export_records(&[json!({
        "name": "Shared Name",
        "type": "prometheus"
    })]);
    let live_records = normalize_live_records(&[
        json!({"uid": "a", "name": "Shared Name", "type": "prometheus"}),
        json!({"uid": "b", "name": "Shared Name", "type": "prometheus"}),
    ]);

    let report = build_datasource_diff_report(&export_records, &live_records);

    assert_eq!(report.summary.ambiguous_live_match_count, 1);
    assert_eq!(
        report.entries[0].status,
        DatasourceDiffStatus::AmbiguousLiveMatch
    );
}
