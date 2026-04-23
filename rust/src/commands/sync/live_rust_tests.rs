//! Sync live-oriented test suite.
//! Covers live flag parsing and request-backed fetch/apply helpers.
use super::live::{
    execute_live_apply_with_request, fetch_live_availability_with_request,
    fetch_live_resource_specs_with_request, load_apply_intent_operations,
};
use super::{SyncAdvancedCommand, SyncCliArgs, SyncGroupCommand};
use clap::Parser;
use reqwest::Method;
use serde_json::json;
use std::path::Path;

fn load_apply_operations(items: Vec<serde_json::Value>) -> Vec<super::live::SyncApplyOperation> {
    load_apply_intent_operations(&json!({ "operations": items })).unwrap()
}

#[test]
fn load_apply_intent_operations_requires_operations_array() {
    let error = load_apply_intent_operations(&json!({
        "kind": "grafana-utils-sync-apply-intent"
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("missing operations"));
}

#[test]
fn load_apply_intent_operations_rejects_wrong_kind() {
    let error = load_apply_intent_operations(&json!({
        "kind": "wrong-kind",
        "operations": []
    }))
    .unwrap_err()
    .to_string();

    assert!(error.contains("kind is not supported"));
}

#[test]
fn parse_sync_cli_supports_plan_fetch_live_mode() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "ci",
        "plan",
        "--desired-file",
        "./desired.json",
        "--fetch-live",
        "--org-id",
        "7",
        "--page-size",
        "250",
        "--token",
        "test-token",
    ]);

    match args.command {
        SyncGroupCommand::Advanced(inner) => match inner.command {
            SyncAdvancedCommand::Plan(inner) => {
                assert_eq!(inner.desired_file, Path::new("./desired.json"));
                assert_eq!(inner.live_file, None);
                assert!(inner.fetch_live);
                assert_eq!(inner.org_id, Some(7));
                assert_eq!(inner.page_size, 250);
            }
            _ => panic!("expected plan"),
        },
        _ => panic!("expected plan"),
    }
}

#[test]
fn parse_sync_cli_supports_apply_execute_live_flags() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "apply",
        "--plan-file",
        "./plan.json",
        "--approve",
        "--execute-live",
        "--allow-folder-delete",
        "--allow-policy-reset",
        "--org-id",
        "9",
        "--token",
        "test-token",
    ]);

    match args.command {
        SyncGroupCommand::Apply(inner) => {
            assert!(inner.execute_live);
            assert!(inner.allow_folder_delete);
            assert!(inner.allow_policy_reset);
            assert_eq!(inner.org_id, Some(9));
        }
        _ => panic!("expected apply"),
    }
}

#[test]
fn parse_sync_cli_supports_input_test_fetch_live_mode() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "ci",
        "input-test",
        "--desired-file",
        "./desired.json",
        "--fetch-live",
        "--org-id",
        "3",
        "--token",
        "test-token",
    ]);

    match args.command {
        SyncGroupCommand::Advanced(inner) => match inner.command {
            SyncAdvancedCommand::Preflight(inner) => {
                assert!(inner.fetch_live);
                assert_eq!(inner.org_id, Some(3));
            }
            _ => panic!("expected input-test"),
        },
        _ => panic!("expected workspace ci"),
    }
}

#[test]
fn parse_sync_cli_supports_package_test_fetch_live_mode() {
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "ci",
        "package-test",
        "--source-bundle",
        "./bundle.json",
        "--target-inventory",
        "./target.json",
        "--fetch-live",
        "--org-id",
        "5",
        "--token",
        "test-token",
    ]);

    match args.command {
        SyncGroupCommand::Advanced(inner) => match inner.command {
            SyncAdvancedCommand::BundlePreflight(inner) => {
                assert!(inner.fetch_live);
                assert_eq!(inner.org_id, Some(5));
            }
            _ => panic!("expected package-test"),
        },
        _ => panic!("expected workspace ci"),
    }
}

#[test]
fn fetch_live_resource_specs_with_request_collects_alerts_and_alerting_resources() {
    let mut calls = Vec::new();
    let specs = fetch_live_resource_specs_with_request(
        |method, path, params, payload| {
            calls.push((
                method.clone(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match (method.clone(), path) {
                (Method::GET, "/api/folders") => Ok(Some(json!([
                    {"uid": "ops", "title": "Operations"}
                ]))),
                (Method::GET, "/api/search") => {
                    let page = params
                        .iter()
                        .find(|(key, _)| key == "page")
                        .map(|(_, value)| value.as_str())
                        .unwrap_or("1");
                    if page == "1" {
                        Ok(Some(json!([
                            {"uid": "cpu-main", "title": "CPU Main"}
                        ])))
                    } else {
                        Ok(Some(json!([])))
                    }
                }
                (Method::GET, "/api/dashboards/uid/cpu-main") => Ok(Some(json!({
                    "dashboard": {"uid": "cpu-main", "title": "CPU Main", "panels": []}
                }))),
                (Method::GET, "/api/datasources") => Ok(Some(json!([
                    {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "access": "proxy", "url": "http://prometheus:9090"}
                ]))),
                (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([
                    {
                        "uid": "cpu-high",
                        "title": "CPU High",
                        "folderUID": "general",
                        "ruleGroup": "CPU Alerts",
                        "condition": "A",
                        "data": [{"refId": "A"}]
                    }
                ]))),
                (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                    {
                        "uid": "cp-main",
                        "name": "PagerDuty Primary",
                        "type": "webhook",
                        "settings": {"url": "http://127.0.0.1/notify"}
                    }
                ]))),
                (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([
                    {
                        "name": "Off Hours",
                        "time_intervals": [{"times": [{"start_time": "00:00", "end_time": "06:00"}]}]
                    }
                ]))),
                (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({
                    "receiver": "grafana-default-email"
                }))),
                (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(json!([
                    {"name": "slack.default"}
                ]))),
                (Method::GET, "/api/v1/provisioning/templates/slack.default") => Ok(Some(json!({
                    "name": "slack.default",
                    "template": "{{ define \"slack.default\" }}ok{{ end }}"
                }))),
                _ => Err(crate::common::message(format!("unexpected {method} {path}"))),
            }
        },
        500,
    )
    .unwrap();

    assert!(specs.iter().any(|item| item["kind"] == "folder"));
    assert!(specs.iter().any(|item| item["kind"] == "dashboard"));
    assert!(specs.iter().any(|item| item["kind"] == "datasource"));
    assert!(specs.iter().any(|item| item["kind"] == "alert"));
    assert!(specs
        .iter()
        .any(|item| item["kind"] == "alert-contact-point" && item["uid"] == "cp-main"));
    assert!(specs
        .iter()
        .any(|item| item["kind"] == "alert-mute-timing" && item["name"] == "Off Hours"));
    assert!(specs
        .iter()
        .any(|item| item["kind"] == "alert-policy" && item["title"] == "grafana-default-email"));
    assert!(specs
        .iter()
        .any(|item| item["kind"] == "alert-template" && item["name"] == "slack.default"));
    assert!(calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/v1/provisioning/alert-rules"));
    assert!(calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/v1/provisioning/contact-points"));
    assert!(calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/v1/provisioning/templates/slack.default"));
}

#[test]
fn fetch_live_resource_specs_with_request_ignores_null_template_list() {
    let specs = fetch_live_resource_specs_with_request(
        |method, path, params, _| match (method.clone(), path) {
            (Method::GET, "/api/folders") => Ok(Some(json!([]))),
            (Method::GET, "/api/search") => {
                let page = params
                    .iter()
                    .find(|(key, _)| key == "page")
                    .map(|(_, value)| value.as_str())
                    .unwrap_or("1");
                let _ = page;
                Ok(Some(json!([])))
            }
            (Method::GET, "/api/datasources") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/alert-rules") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/mute-timings") => Ok(Some(json!([]))),
            (Method::GET, "/api/v1/provisioning/policies") => Ok(Some(json!({}))),
            (Method::GET, "/api/v1/provisioning/templates") => Ok(Some(serde_json::Value::Null)),
            _ => Err(crate::common::message(format!(
                "unexpected {method} {path}"
            ))),
        },
        500,
    )
    .unwrap();

    assert!(!specs.iter().any(|item| item["kind"] == "alert-template"));
}

#[test]
fn fetch_live_availability_with_request_collects_contact_points_and_plugins() {
    let availability =
        fetch_live_availability_with_request(|method, path, _, _| match (method, path) {
            (Method::GET, "/api/datasources") => Ok(Some(json!([
                {"uid": "prom-main", "name": "Prometheus Main"}
            ]))),
            (Method::GET, "/api/plugins") => Ok(Some(json!([
                {"id": "prometheus"}
            ]))),
            (Method::GET, "/api/v1/provisioning/contact-points") => Ok(Some(json!([
                {"uid": "cp-1", "name": "pagerduty-primary"}
            ]))),
            _ => Err(crate::common::message("unexpected request")),
        })
        .unwrap();

    assert_eq!(availability["datasourceUids"], json!(["prom-main"]));
    assert_eq!(availability["pluginIds"], json!(["prometheus"]));
    assert_eq!(
        availability["contactPoints"],
        json!(["pagerduty-primary", "cp-1"])
    );
}

#[test]
fn execute_live_apply_with_request_supports_alert_create() {
    let mut calls = Vec::new();
    let operations = load_apply_operations(vec![json!({
        "kind": "alert",
        "identity": "cpu-high",
        "action": "would-create",
        "desired": {
            "uid": "cpu-high",
            "title": "CPU High",
            "folderUID": "general",
            "ruleGroup": "CPU Alerts",
            "condition": "A",
            "data": [{"refId": "A"}]
        }
    })]);
    let result = execute_live_apply_with_request(
        |method, path, _, payload| {
            calls.push((method.clone(), path.to_string(), payload.cloned()));
            match (method, path) {
                (Method::POST, "/api/v1/provisioning/alert-rules") => {
                    Ok(Some(json!({"uid": "cpu-high", "status": "created"})))
                }
                _ => Err(crate::common::message("unexpected request")),
            }
        },
        &operations,
        false,
        false,
    )
    .unwrap();

    assert_eq!(result["mode"], json!("live-apply"));
    assert_eq!(result["appliedCount"], json!(1));
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, Method::POST);
    assert_eq!(calls[0].1, "/api/v1/provisioning/alert-rules");
}

#[test]
fn execute_live_apply_with_request_supports_folder_create_update_delete_guards() {
    let operations = load_apply_operations(vec![json!({
        "kind": "folder",
        "identity": "platform",
        "action": "would-delete"
    })]);
    let blocked = execute_live_apply_with_request(
        |_, _, _, _| {
            Err(crate::common::message(
                "request handler should not be called",
            ))
        },
        &operations,
        false,
        false,
    )
    .unwrap_err()
    .to_string();
    assert!(blocked.contains("--allow-folder-delete"));

    let mut calls = Vec::new();
    let operations = load_apply_operations(vec![
        json!({
            "kind": "folder",
            "identity": "platform",
            "action": "would-create",
            "desired": {"title": "Platform", "parentUid": "root"}
        }),
        json!({
            "kind": "folder",
            "identity": "platform",
            "action": "would-update",
            "desired": {"title": "Platform Renamed"}
        }),
        json!({
            "kind": "folder",
            "identity": "platform",
            "action": "would-delete"
        }),
    ]);

    let result = execute_live_apply_with_request(
        |method, path, params, payload| {
            calls.push((
                method.clone(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match (method, path) {
                (Method::POST, "/api/folders") => Ok(Some(json!({"uid": "platform"}))),
                (Method::PUT, "/api/folders/platform") => Ok(Some(json!({"uid": "platform"}))),
                (Method::DELETE, "/api/folders/platform") => Ok(None),
                _ => Err(crate::common::message("unexpected request")),
            }
        },
        &operations,
        true,
        false,
    )
    .unwrap();

    assert_eq!(result["appliedCount"], json!(3));
    assert_eq!(calls[0].0, Method::POST);
    assert_eq!(calls[0].1, "/api/folders");
    assert_eq!(
        calls[0].3.as_ref().and_then(|value| value.get("title")),
        Some(&json!("Platform"))
    );
    assert_eq!(
        calls[0].3.as_ref().and_then(|value| value.get("parentUid")),
        Some(&json!("root"))
    );
    assert_eq!(calls[1].0, Method::PUT);
    assert_eq!(calls[1].1, "/api/folders/platform");
    assert_eq!(calls[2].0, Method::DELETE);
    assert_eq!(calls[2].1, "/api/folders/platform");
    assert!(calls[2]
        .2
        .iter()
        .any(|(key, value)| key == "forceDeleteRules" && value == "false"));
}

#[test]
fn execute_live_apply_with_request_supports_dashboard_upsert_and_delete() {
    let mut calls = Vec::new();
    let operations = load_apply_operations(vec![
        json!({
            "kind": "dashboard",
            "identity": "cpu-main",
            "action": "would-create",
            "desired": {
                "id": 42,
                "title": "CPU Main",
                "folderUid": "platform",
                "panels": []
            }
        }),
        json!({
            "kind": "dashboard",
            "identity": "cpu-main",
            "action": "would-update",
            "desired": {
                "title": "CPU Main Updated",
                "folderUID": "platform",
                "panels": []
            }
        }),
        json!({
            "kind": "dashboard",
            "identity": "cpu-main",
            "action": "would-delete"
        }),
    ]);

    let result = execute_live_apply_with_request(
        |method, path, _, payload| {
            calls.push((method.clone(), path.to_string(), payload.cloned()));
            match (method, path) {
                (Method::POST, "/api/dashboards/db") => Ok(Some(json!({"status": "success"}))),
                (Method::DELETE, "/api/dashboards/uid/cpu-main") => Ok(None),
                _ => Err(crate::common::message("unexpected request")),
            }
        },
        &operations,
        false,
        false,
    )
    .unwrap();

    assert_eq!(result["appliedCount"], json!(3));
    assert_eq!(calls[0].0, Method::POST);
    assert_eq!(calls[0].1, "/api/dashboards/db");
    let create_payload = calls[0].2.as_ref().unwrap();
    assert_eq!(create_payload["overwrite"], json!(false));
    assert_eq!(create_payload["folderUid"], json!("platform"));
    assert_eq!(create_payload["dashboard"]["uid"], json!("cpu-main"));
    assert!(create_payload["dashboard"].get("id").is_none());

    let update_payload = calls[1].2.as_ref().unwrap();
    assert_eq!(update_payload["overwrite"], json!(true));
    assert_eq!(update_payload["folderUid"], json!("platform"));
    assert_eq!(update_payload["dashboard"]["uid"], json!("cpu-main"));
    assert_eq!(calls[2].0, Method::DELETE);
    assert_eq!(calls[2].1, "/api/dashboards/uid/cpu-main");
}

#[test]
fn execute_live_apply_with_request_supports_datasource_update_and_delete_uid_with_id_fallback() {
    let mut calls = Vec::new();
    let operations = load_apply_operations(vec![
        json!({
            "kind": "datasource",
            "identity": "prom-main",
            "action": "would-update",
            "desired": {
                "type": "prometheus",
                "url": "http://prometheus:9090"
            }
        }),
        json!({
            "kind": "datasource",
            "identity": "prom-main",
            "action": "would-delete"
        }),
    ]);

    let result = execute_live_apply_with_request(
        |method, path, _, payload| {
            calls.push((method.clone(), path.to_string(), payload.cloned()));
            match (method, path) {
                (Method::GET, "/api/datasources") => Ok(Some(json!([
                    {"id": 7, "uid": "prom-main", "name": "Prometheus Main"}
                ]))),
                (Method::PUT, "/api/datasources/uid/prom-main") => {
                    Err(crate::common::api_response(
                        404,
                        "http://grafana.local/api/datasources/uid/prom-main",
                        "not found",
                    ))
                }
                (Method::PUT, "/api/datasources/7") => {
                    Ok(Some(json!({"uid": "prom-main", "status": "updated"})))
                }
                (Method::DELETE, "/api/datasources/uid/prom-main") => {
                    Err(crate::common::api_response(
                        404,
                        "http://grafana.local/api/datasources/uid/prom-main",
                        "not found",
                    ))
                }
                (Method::DELETE, "/api/datasources/7") => Ok(None),
                _ => Err(crate::common::message("unexpected request")),
            }
        },
        &operations,
        false,
        false,
    )
    .unwrap();

    assert_eq!(result["appliedCount"], json!(2));
    assert!(calls.iter().any(|(method, path, payload)| {
        *method == Method::PUT
            && path == "/api/datasources/uid/prom-main"
            && payload
                .as_ref()
                .and_then(|value| value.get("uid"))
                .is_some_and(|value| value == "prom-main")
            && payload
                .as_ref()
                .and_then(|value| value.get("name"))
                .is_some_and(|value| value == "prom-main")
    }));
    assert!(calls
        .iter()
        .any(|(method, path, _)| *method == Method::PUT && path == "/api/datasources/7"));
    assert!(calls.iter().any(|(method, path, _)| {
        *method == Method::DELETE && path == "/api/datasources/uid/prom-main"
    }));
    assert!(calls
        .iter()
        .any(|(method, path, _)| *method == Method::DELETE && path == "/api/datasources/7"));
}

#[test]
fn execute_live_apply_with_request_supports_non_rule_alert_resources() {
    let mut calls = Vec::new();
    let operations = load_apply_operations(vec![
        json!({
            "kind": "alert-contact-point",
            "identity": "cp-main",
            "action": "would-update",
            "desired": {
                "uid": "cp-main",
                "name": "PagerDuty Primary",
                "type": "webhook",
                "settings": {"url": "http://127.0.0.1/notify"}
            }
        }),
        json!({
            "kind": "alert-mute-timing",
            "identity": "Off Hours",
            "action": "would-update",
            "desired": {
                "name": "Off Hours",
                "time_intervals": [{"times": [{"start_time": "00:00", "end_time": "06:00"}]}]
            }
        }),
        json!({
            "kind": "alert-policy",
            "identity": "grafana-default-email",
            "action": "would-update",
            "desired": {
                "receiver": "grafana-default-email"
            }
        }),
        json!({
            "kind": "alert-template",
            "identity": "slack.default",
            "action": "would-update",
            "desired": {
                "name": "slack.default",
                "template": "{{ define \"slack.default\" }}ok{{ end }}"
            }
        }),
    ]);
    let result = execute_live_apply_with_request(
        |method, path, _, payload| {
            calls.push((method.clone(), path.to_string(), payload.cloned()));
            match (method, path) {
                (Method::PUT, "/api/v1/provisioning/contact-points/cp-main") => {
                    Ok(Some(json!({"uid": "cp-main", "status": "updated"})))
                }
                (Method::PUT, "/api/v1/provisioning/mute-timings/Off Hours") => {
                    Ok(Some(json!({"name": "Off Hours", "status": "updated"})))
                }
                (Method::PUT, "/api/v1/provisioning/policies") => Ok(Some(
                    json!({"receiver": "grafana-default-email", "status": "updated"}),
                )),
                (Method::PUT, "/api/v1/provisioning/templates/slack.default") => {
                    Ok(Some(json!({"name": "slack.default", "status": "updated"})))
                }
                _ => Err(crate::common::message("unexpected request")),
            }
        },
        &operations,
        false,
        false,
    )
    .unwrap();

    assert_eq!(result["mode"], json!("live-apply"));
    assert_eq!(result["appliedCount"], json!(4));
    assert!(calls.iter().any(|(method, path, _)| *method == Method::PUT
        && path == "/api/v1/provisioning/contact-points/cp-main"));
    assert!(calls.iter().any(|(method, path, _)| *method == Method::PUT
        && path == "/api/v1/provisioning/mute-timings/Off Hours"));
    assert!(
        calls
            .iter()
            .any(|(method, path, _)| *method == Method::PUT
                && path == "/api/v1/provisioning/policies")
    );
    assert!(calls
        .iter()
        .any(|(method, path, payload)| *method == Method::PUT
            && path == "/api/v1/provisioning/templates/slack.default"
            && payload
                .as_ref()
                .and_then(|value| value.get("name"))
                .is_none()));
}

#[test]
fn execute_live_apply_with_request_supports_non_rule_alert_deletes() {
    let mut calls = Vec::new();
    let operations = load_apply_operations(vec![
        json!({
            "kind": "alert-contact-point",
            "identity": "cp-main",
            "action": "would-delete"
        }),
        json!({
            "kind": "alert-mute-timing",
            "identity": "Off Hours",
            "action": "would-delete"
        }),
        json!({
            "kind": "alert-template",
            "identity": "slack.default",
            "action": "would-delete"
        }),
    ]);
    let result = execute_live_apply_with_request(
        |method, path, params, _| {
            calls.push((method.clone(), path.to_string(), params.to_vec()));
            match (method, path) {
                (Method::DELETE, "/api/v1/provisioning/contact-points/cp-main") => Ok(None),
                (Method::DELETE, "/api/v1/provisioning/mute-timings/Off Hours") => Ok(None),
                (Method::DELETE, "/api/v1/provisioning/templates/slack.default") => Ok(None),
                _ => Err(crate::common::message("unexpected request")),
            }
        },
        &operations,
        false,
        false,
    )
    .unwrap();

    assert_eq!(result["appliedCount"], json!(3));
    assert!(calls
        .iter()
        .any(|(method, path, _)| *method == Method::DELETE
            && path == "/api/v1/provisioning/contact-points/cp-main"));
    assert!(calls
        .iter()
        .any(|(method, path, params)| *method == Method::DELETE
            && path == "/api/v1/provisioning/mute-timings/Off Hours"
            && params
                .iter()
                .any(|(key, value)| key == "version" && value.is_empty())));
    assert!(calls
        .iter()
        .any(|(method, path, params)| *method == Method::DELETE
            && path == "/api/v1/provisioning/templates/slack.default"
            && params
                .iter()
                .any(|(key, value)| key == "version" && value.is_empty())));
}

#[test]
fn execute_live_apply_with_request_rejects_alert_policy_delete_without_reset_flag() {
    let operations = load_apply_operations(vec![json!({
        "kind": "alert-policy",
        "identity": "grafana-default-email",
        "action": "would-delete"
    })]);
    let result = execute_live_apply_with_request(
        |_, _, _, _| {
            Err(crate::common::message(
                "request handler should not be called",
            ))
        },
        &operations,
        false,
        false,
    );

    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("--allow-policy-reset"));
}

#[test]
fn execute_live_apply_with_request_supports_alert_policy_reset_when_allowed() {
    let mut calls = Vec::new();
    let operations = load_apply_operations(vec![json!({
        "kind": "alert-policy",
        "identity": "grafana-default-email",
        "action": "would-delete"
    })]);
    let result = execute_live_apply_with_request(
        |method, path, params, _| {
            calls.push((method.clone(), path.to_string(), params.to_vec()));
            match (method, path) {
                (Method::DELETE, "/api/v1/provisioning/policies") => Ok(None),
                _ => Err(crate::common::message("unexpected request")),
            }
        },
        &operations,
        false,
        true,
    )
    .unwrap();

    assert_eq!(result["appliedCount"], json!(1));
    assert!(calls
        .iter()
        .any(|(method, path, _)| *method == Method::DELETE
            && path == "/api/v1/provisioning/policies"));
}
