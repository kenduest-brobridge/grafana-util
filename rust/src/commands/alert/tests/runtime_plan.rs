use super::{
    build_alert_plan_with_request, write_new_rule_scaffold, write_new_template_scaffold,
    CONTACT_POINT_KIND, MUTE_TIMING_KIND, POLICIES_KIND, RULE_KIND, TEMPLATE_KIND,
    TOOL_SCHEMA_VERSION,
};
use reqwest::Method;
use serde_json::json;
use tempfile::tempdir;

#[test]
fn build_alert_plan_with_request_generates_create_update_noop_and_blocked_rows() {
    let temp = tempdir().unwrap();
    write_new_rule_scaffold(
        &temp.path().join("rules/create-rule.json"),
        "create-rule",
        true,
    )
    .unwrap();
    super::write_pretty_json(
        &temp.path().join("contact-points/update-contact-point.yaml"),
        &json!({
            "kind": CONTACT_POINT_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "uid": "cp-update",
                "name": "Update Me",
                "type": "webhook",
                "settings": {"url": "http://127.0.0.1/new"}
            }
        }),
    );
    write_new_template_scaffold(
        &temp.path().join("templates/example-template.json"),
        "example-template",
        true,
    )
    .unwrap();

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/alert-rules/create-rule") => Ok(None),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                {
                    "uid": "cp-update",
                    "name": "Update Me",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/old"}
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/templates/example-template") => Ok(Some(json!({
                "name": "example-template",
                "template": "{{ define \"example-template\" }}replace me{{ end }}"
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([
                {
                    "name": "off-hours",
                    "time_intervals": []
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([
                {
                    "name": "example-template",
                    "template": "{{ define \"example-template\" }}replace me{{ end }}"
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({
                "receiver": "grafana-default-email"
            }))),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        false,
    )
    .unwrap();

    assert_eq!(plan["summary"]["create"], json!(1));
    assert_eq!(plan["summary"]["update"], json!(1));
    assert_eq!(plan["summary"]["noop"], json!(1));
    assert_eq!(plan["summary"]["blocked"], json!(2));
    assert_eq!(plan["summary"]["warning"], json!(0));

    let rows = plan["rows"].as_array().unwrap();
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(RULE_KIND)
            && row["identity"] == json!("create-rule")
            && row["action"] == json!("create")
            && row["status"] == json!("ready")
            && row["actionId"].as_str().unwrap_or("").ends_with("::create")
            && row["changedFields"].is_array()
            && row["changes"].is_array()
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(CONTACT_POINT_KIND)
            && row["identity"] == json!("cp-update")
            && row["action"] == json!("update")
            && row["status"] == json!("ready")
            && row["changedFields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "settings")
            && row["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|change| change["field"] == json!("settings"))
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(TEMPLATE_KIND)
            && row["identity"] == json!("example-template")
            && row["action"] == json!("noop")
            && row["status"] == json!("same")
            && row["reviewHints"] == json!([])
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(MUTE_TIMING_KIND)
            && row["identity"] == json!("off-hours")
            && row["action"] == json!("blocked")
            && row["status"] == json!("blocked")
            && row["reason"] == json!("prune-required")
            && row["blockedReason"] == json!("prune-required")
    }));
    assert!(rows.iter().any(|row| {
        row["kind"] == json!(POLICIES_KIND)
            && row["identity"] == json!("grafana-default-email")
            && row["action"] == json!("blocked")
            && row["status"] == json!("blocked")
    }));
}

#[test]
fn build_alert_plan_with_request_surfaces_linked_rule_review_hints() {
    let temp = tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("rules")).unwrap();
    std::fs::write(
        temp.path().join("rules/linked-rule.json"),
        serde_json::to_string_pretty(&json!({
            "uid": "linked-rule",
            "title": "Linked Rule",
            "folderUID": "alerts",
            "ruleGroup": "linked",
            "condition": "A",
            "data": [],
            "annotations": {
                "__dashboardUid__": "src-dash",
                "__panelId__": "7"
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/alert-rules/linked-rule") => Ok(Some(json!({
                "uid": "linked-rule",
                "title": "Linked Rule",
                "folderUID": "alerts",
                "ruleGroup": "linked",
                "condition": "A",
                "data": [],
                "annotations": {
                    "__dashboardUid__": "src-dash",
                    "__panelId__": "7"
                }
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({"receiver": "root"}))),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        false,
    )
    .unwrap();

    assert_eq!(plan["summary"]["warning"], json!(1));
    let row = plan["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == json!("linked-rule"))
        .unwrap();
    assert_eq!(row["action"], json!("noop"));
    assert_eq!(row["status"], json!("warning"));
    assert!(row["actionId"]
        .as_str()
        .unwrap_or("")
        .contains("linked-rule"));
    let hints = row["reviewHints"].as_array().unwrap();
    assert!(hints
        .iter()
        .any(|hint| hint["code"] == json!("linked-dashboard-reference")));
    assert!(hints
        .iter()
        .any(|hint| hint["code"] == json!("linked-panel-reference")));
    assert!(row["changedFields"].as_array().unwrap().is_empty());
}

#[test]
fn build_alert_plan_with_request_marks_live_only_resources_delete_when_prune_enabled() {
    let temp = tempdir().unwrap();
    write_new_rule_scaffold(
        &temp.path().join("rules/create-rule.json"),
        "create-rule",
        true,
    )
    .unwrap();

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/alert-rules/create-rule") => Ok(None),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                {
                    "uid": "cp-delete",
                    "name": "Delete Me",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/delete"}
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(None),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        true,
    )
    .unwrap();

    assert!(plan["rows"].as_array().unwrap().iter().any(|row| {
        row["kind"] == json!(CONTACT_POINT_KIND)
            && row["identity"] == json!("cp-delete")
            && row["action"] == json!("delete")
    }));
}

#[test]
fn build_alert_plan_with_request_treats_authoring_round_trip_defaults_as_noop() {
    let temp = tempdir().unwrap();
    super::write_pretty_json(
        &temp
            .path()
            .join("contact-points/authoring-contact-point.json"),
        &json!({
            "kind": CONTACT_POINT_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "uid": "cp-authoring",
                "name": "Authoring Webhook",
                "type": "webhook",
                "settings": {"url": "http://127.0.0.1/notify"}
            }
        }),
    );
    super::write_pretty_json(
        &temp.path().join("policies/notification-policies.json"),
        &json!({
            "kind": POLICIES_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "receiver": "pagerduty-primary",
                "group_by": ["grafana_folder", "alertname"],
                "routes": [{
                    "receiver": "pagerduty-primary",
                    "continue": false,
                    "group_by": ["grafana_folder", "alertname"],
                    "object_matchers": [
                        ["team", "=", "platform"],
                        ["severity", "=", "critical"],
                        ["grafana_utils_route", "=", "pagerduty-primary"]
                    ]
                }]
            }
        }),
    );
    super::write_pretty_json(
        &temp.path().join("rules/cpu-high.json"),
        &json!({
            "kind": RULE_KIND,
            "apiVersion": super::TOOL_API_VERSION,
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "spec": {
                "uid": "cpu-high",
                "title": "cpu-high",
                "folderUID": "platform-alerts",
                "ruleGroup": "cpu",
                "condition": "A",
                "for": "5m",
                "noDataState": "NoData",
                "execErrState": "Alerting",
                "labels": {
                    "grafana_utils_route": "pagerduty-primary",
                    "severity": "critical",
                    "team": "platform"
                },
                "annotations": {},
                "data": [{
                    "refId": "A",
                    "datasourceUid": "__expr__",
                    "relativeTimeRange": {"from": 0, "to": 0},
                    "model": {
                        "refId": "A",
                        "type": "classic_conditions",
                        "datasource": {"type": "__expr__", "uid": "__expr__"},
                        "expression": "A",
                        "conditions": [{
                            "type": "query",
                            "query": {"params": ["A"]},
                            "reducer": {"type": "last", "params": []},
                            "evaluator": {"type": "gt", "params": [80.0]},
                            "operator": {"type": "and"}
                        }],
                        "intervalMs": 1000,
                        "maxDataPoints": 43200
                    }
                }]
            }
        }),
    );

    let plan = build_alert_plan_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                {
                    "uid": "cp-authoring",
                    "name": "Authoring Webhook",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/notify"},
                    "disableResolveMessage": false
                }
            ]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({
                "receiver": "pagerduty-primary",
                "group_by": ["grafana_folder", "alertname"],
                "routes": [{
                    "receiver": "pagerduty-primary",
                    "group_by": ["grafana_folder", "alertname"],
                    "object_matchers": [
                        ["grafana_utils_route", "=", "pagerduty-primary"],
                        ["severity", "=", "critical"],
                        ["team", "=", "platform"]
                    ]
                }]
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules/cpu-high") => Ok(Some(json!({
                "uid": "cpu-high",
                "title": "cpu-high",
                "folderUID": "platform-alerts",
                "ruleGroup": "cpu",
                "condition": "A",
                "for": "5m",
                "noDataState": "NoData",
                "execErrState": "Alerting",
                "isPaused": false,
                "keep_firing_for": "0s",
                "notification_settings": null,
                "record": null,
                "orgID": 1,
                "data": [{
                    "refId": "A",
                    "queryType": "",
                    "datasourceUid": "__expr__",
                    "relativeTimeRange": {"from": 0, "to": 0},
                    "model": {
                        "refId": "A",
                        "type": "classic_conditions",
                        "datasource": {"type": "__expr__", "uid": "__expr__"},
                        "expression": "A",
                        "conditions": [{
                            "type": "query",
                            "query": {"params": ["A"]},
                            "reducer": {"type": "last", "params": []},
                            "evaluator": {"type": "gt", "params": [80.0]},
                            "operator": {"type": "and"}
                        }],
                        "intervalMs": 1000,
                        "maxDataPoints": 43200
                    }
                }],
                "labels": {
                    "grafana_utils_route": "pagerduty-primary",
                    "severity": "critical",
                    "team": "platform"
                },
                "annotations": {}
            }))),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([
                {"uid": "cpu-high"}
            ]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([]))),
            _ => panic!("unexpected request {method:?} {path}"),
        },
        temp.path(),
        true,
    )
    .unwrap();

    assert_eq!(plan["summary"]["create"], json!(0));
    assert_eq!(plan["summary"]["update"], json!(0));
    assert_eq!(plan["summary"]["noop"], json!(3));
    assert_eq!(plan["summary"]["delete"], json!(0));
}
