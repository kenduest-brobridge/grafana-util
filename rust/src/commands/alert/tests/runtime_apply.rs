use super::{
    build_alert_plan_document, determine_import_action_with_request,
    execute_alert_plan_with_request, fetch_live_compare_document_with_request,
    import_resource_document_with_request, load_alert_recreate_contract_fixture,
    run_alert_recreate_case, CONTACT_POINT_KIND, POLICIES_KIND, RULE_KIND, TEMPLATE_KIND,
};
use crate::common::{message, Result};
use reqwest::Method;
use serde_json::{json, Value};
use std::cell::RefCell;

#[test]
fn execute_alert_plan_with_request_applies_create_update_and_delete_rows() {
    let plan = build_alert_plan_document(
        &[
            json!({
                "kind": RULE_KIND,
                "identity": "rule-create",
                "action": "create",
                "actionId": "grafana-alert-rule::rule-create::create",
                "status": "ready",
                "blockedReason": null,
                "reviewHints": [],
                "changedFields": ["title"],
                "changes": [{
                    "field": "title",
                    "before": null,
                    "after": "Create Me"
                }],
                "desired": {
                    "uid": "rule-create",
                    "title": "Create Me",
                    "folderUID": "general",
                    "ruleGroup": "default",
                    "condition": "A",
                    "data": []
                },
                "live": null
            }),
            json!({
                "kind": CONTACT_POINT_KIND,
                "identity": "cp-update",
                "action": "update",
                "actionId": "grafana-alert-contact-point::cp-update::update",
                "status": "ready",
                "blockedReason": null,
                "reviewHints": [],
                "changedFields": ["settings"],
                "changes": [{
                    "field": "settings",
                    "before": {"url": "http://127.0.0.1/old"},
                    "after": {"url": "http://127.0.0.1/new"}
                }],
                "desired": {
                    "uid": "cp-update",
                    "name": "Update Me",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/new"}
                },
                "live": {
                    "uid": "cp-update",
                    "name": "Update Me",
                    "type": "webhook",
                    "settings": {"url": "http://127.0.0.1/old"}
                }
            }),
            json!({
                "kind": TEMPLATE_KIND,
                "identity": "template-delete",
                "action": "delete",
                "actionId": "grafana-message-template::template-delete::delete",
                "status": "ready",
                "blockedReason": null,
                "reviewHints": [],
                "changedFields": ["name"],
                "changes": [{
                    "field": "name",
                    "before": "template-delete",
                    "after": null
                }],
                "desired": null,
                "live": {
                    "name": "template-delete"
                }
            }),
            json!({
                "kind": RULE_KIND,
                "identity": "rule-noop",
                "action": "noop",
                "actionId": "grafana-alert-rule::rule-noop::noop",
                "status": "same",
                "blockedReason": null,
                "reviewHints": [],
                "changedFields": [],
                "changes": [],
                "desired": null,
                "live": null
            }),
        ],
        true,
    );
    let calls = RefCell::new(Vec::new());

    let result = execute_alert_plan_with_request(
        |method, path, _params, payload| {
            calls
                .borrow_mut()
                .push((method.clone(), path.to_string(), payload.cloned()));
            match (method.clone(), path) {
                (Method::POST, "/api/v1/provisioning/alert-rules") => {
                    Ok(Some(json!({"uid": "rule-create"})))
                }
                (Method::PUT, "/api/v1/provisioning/contact-points/cp-update") => {
                    Ok(Some(json!({"uid": "cp-update"})))
                }
                (Method::DELETE, "/api/v1/provisioning/templates/template-delete") => Ok(None),
                _ => panic!("unexpected request {method:?} {path}"),
            }
        },
        &plan,
        false,
    )
    .unwrap();

    assert_eq!(result["appliedCount"], json!(3));
    let calls = calls.borrow();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].0, Method::POST);
    assert_eq!(calls[0].1, "/api/v1/provisioning/alert-rules");
    assert_eq!(calls[1].0, Method::PUT);
    assert_eq!(calls[1].1, "/api/v1/provisioning/contact-points/cp-update");
    assert_eq!(calls[2].0, Method::DELETE);
    assert_eq!(calls[2].1, "/api/v1/provisioning/templates/template-delete");
}

#[test]
fn execute_alert_plan_with_request_rejects_policy_delete_without_guard() {
    let plan = build_alert_plan_document(
        &[json!({
            "kind": POLICIES_KIND,
            "identity": "grafana-default-email",
            "action": "delete",
            "desired": null
        })],
        true,
    );

    let error =
        execute_alert_plan_with_request(|_method, _path, _params, _payload| Ok(None), &plan, false)
            .unwrap_err()
            .to_string();

    assert!(error.contains("--allow-policy-reset"));
}

#[test]
fn alert_recreate_matrix_with_request_covers_rule_contact_point_mute_timing_and_template() {
    let fixture = load_alert_recreate_contract_fixture();
    for case in fixture["recreateCases"].as_array().unwrap_or(&Vec::new()) {
        let kind = case["kind"].as_str().unwrap_or("");
        let identity = case["identity"].as_str().unwrap_or("");
        let expected_dry_run_action = case["expectedDryRunAction"]
            .as_str()
            .unwrap_or("would-create");
        let expected_replay_action = case["expectedReplayAction"].as_str().unwrap_or("created");
        let request_contract = case["requestContract"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        let payload = case["payload"].as_object().cloned().unwrap_or_default();
        run_alert_recreate_case(
            kind,
            payload,
            identity,
            expected_dry_run_action,
            expected_replay_action,
            request_contract,
        );
    }
}

#[test]
fn policies_with_request_stay_update_only_and_return_to_same_state() {
    let fixture = load_alert_recreate_contract_fixture();
    let expected_identity = fixture["policiesCase"]["identity"].as_str().unwrap_or("");
    let expected_dry_run_action = fixture["policiesCase"]["expectedDryRunAction"]
        .as_str()
        .unwrap_or("would-update");
    let expected_replay_action = fixture["policiesCase"]["expectedReplayAction"]
        .as_str()
        .unwrap_or("updated");
    let request_contract = fixture["policiesCase"]["requestContract"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    let local_payload = fixture["policiesCase"]["payload"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    let remote_policy = RefCell::new(json!({
        "receiver": "legacy-email",
        "routes": [{"receiver": "legacy-email"}]
    }));
    let request_log = RefCell::new(Vec::<String>::new());

    let initial_compare = fetch_live_compare_document_with_request(
        |method, path, _params, payload| -> Result<Option<Value>> {
            let method_name = method.as_str().to_string();
            request_log
                .borrow_mut()
                .push(format!("{} {}", method_name, path));
            match (method, path) {
                (Method::GET, "/api/v1/provisioning/policies") => {
                    Ok(Some(remote_policy.borrow().clone()))
                }
                (Method::PUT, "/api/v1/provisioning/policies") => {
                    let next = payload.cloned().ok_or_else(|| {
                        message("policies update payload must be present".to_string())
                    })?;
                    *remote_policy.borrow_mut() = next.clone();
                    Ok(Some(next))
                }
                _ => Err(message(format!(
                    "unexpected alert runtime request {} {}",
                    method_name, path
                ))),
            }
        },
        POLICIES_KIND,
        &local_payload,
    )
    .unwrap()
    .unwrap();
    assert_ne!(
        super::serialize_compare_document(&initial_compare).unwrap(),
        super::serialize_compare_document(&json!({
            "kind": POLICIES_KIND,
            "spec": local_payload,
        }))
        .unwrap()
    );

    assert_eq!(
        determine_import_action_with_request(
            |method, path, _params, _payload| -> Result<Option<Value>> {
                let method_name = method.as_str().to_string();
                request_log
                    .borrow_mut()
                    .push(format!("{} {}", method_name, path));
                match (method, path) {
                    (Method::GET, "/api/v1/provisioning/policies") => {
                        Ok(Some(remote_policy.borrow().clone()))
                    }
                    _ => Err(message(format!(
                        "unexpected alert runtime request {} {}",
                        method_name, path
                    ))),
                }
            },
            POLICIES_KIND,
            &local_payload,
            true,
        )
        .unwrap(),
        expected_dry_run_action
    );

    let (action, identity) = import_resource_document_with_request(
        |method, path, _params, payload| -> Result<Option<Value>> {
            let method_name = method.as_str().to_string();
            request_log
                .borrow_mut()
                .push(format!("{} {}", method_name, path));
            match (method, path) {
                (Method::PUT, "/api/v1/provisioning/policies") => {
                    let next = payload.cloned().ok_or_else(|| {
                        message("policies update payload must be present".to_string())
                    })?;
                    *remote_policy.borrow_mut() = next.clone();
                    Ok(Some(next))
                }
                _ => Err(message(format!(
                    "unexpected alert runtime request {} {}",
                    method_name, path
                ))),
            }
        },
        POLICIES_KIND,
        &local_payload,
        true,
    )
    .unwrap();
    assert_eq!(action, expected_replay_action);
    assert_eq!(identity, expected_identity);

    let update_request_prefix = request_contract
        .get("updateRequestPrefix")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let update_request_count = request_contract
        .get("updateRequestCount")
        .and_then(Value::as_u64)
        .unwrap_or(1) as usize;
    let actual_update_count = request_log
        .borrow()
        .iter()
        .filter(|entry| entry.starts_with(update_request_prefix))
        .count();
    assert_eq!(
        actual_update_count, update_request_count,
        "expected {update_request_count} update request(s) for policies"
    );
    assert!(
        !update_request_prefix.is_empty(),
        "policies request contract is missing updateRequestPrefix"
    );

    let live_compare = fetch_live_compare_document_with_request(
        |method, path, _params, _payload| -> Result<Option<Value>> {
            let method_name = method.as_str().to_string();
            request_log
                .borrow_mut()
                .push(format!("{} {}", method_name, path));
            match (method, path) {
                (Method::GET, "/api/v1/provisioning/policies") => {
                    Ok(Some(remote_policy.borrow().clone()))
                }
                _ => Err(message(format!(
                    "unexpected alert runtime request {} {}",
                    method_name, path
                ))),
            }
        },
        POLICIES_KIND,
        &local_payload,
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        super::serialize_compare_document(&live_compare).unwrap(),
        super::serialize_compare_document(&json!({
            "kind": POLICIES_KIND,
            "spec": local_payload,
        }))
        .unwrap()
    );
}
