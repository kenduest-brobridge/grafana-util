//! Sync workspace review view regression tests.

use super::review_tui;
use super::workspace_preview_review_view::build_workspace_review_view;
use serde_json::json;

#[test]
fn build_workspace_review_view_normalizes_actions_domains_and_blockers() {
    let document = json!({
        "kind": "grafana-utils-sync-plan",
        "summary": {
            "would_create": 1,
            "would_update": 0,
            "would_delete": 0,
            "noop": 0,
            "unmanaged": 0,
            "alert_candidate": 0,
            "alert_plan_only": 0,
            "alert_blocked": 1
        },
        "operations": [
            {
                "action": "would-create",
                "resourceKind": "folder",
                "identity": "folder-main"
            },
            {
                "action": "blocked-read-only",
                "resourceKind": "dashboard",
                "identity": "blocked-main",
                "status": "blocked",
                "reason": "target-read-only",
                "ownership": "git-sync-managed",
                "provenance": ["ownership=git-sync-managed"]
            }
        ]
    });

    let view = build_workspace_review_view(&document).unwrap();

    assert_eq!(view.actions.len(), 2);
    assert_eq!(view.actions[0].domain, "folder");
    assert_eq!(view.actions[0].resource_kind, "folder");
    assert_eq!(view.actions[0].status, "ready");
    assert_eq!(view.actions[1].status, "blocked");
    assert_eq!(view.actions[1].raw["ownership"], json!("git-sync-managed"));
    assert_eq!(
        view.actions[1].raw["provenance"],
        json!(["ownership=git-sync-managed"])
    );
    assert_eq!(view.blocked_reasons, vec!["target-read-only"]);
    assert!(view.domains.iter().any(|domain| domain.id == "dashboard"));
    assert!(view.domains.iter().any(|domain| domain.id == "folder"));
    assert_eq!(view.summary.action_count, 2);
    assert_eq!(view.summary.domain_count, 5);
    assert_eq!(view.summary.blocked_count, 1);
}

#[test]
fn build_workspace_review_view_orders_actions_and_preserves_raw_payload() {
    let document = json!({
        "kind": "grafana-utils-sync-plan",
        "summary": {},
        "operations": [
            {
                "action": "blocked-read-only",
                "kind": "alert-contact-point",
                "identity": "ops-email",
                "reason": "target-read-only",
                "detail": "managed by provisioning",
                "customEvidence": {"owner": "platform"}
            },
            {
                "action": "would-delete",
                "kind": "datasource",
                "uid": "remote-prom",
                "name": "Remote Prometheus"
            },
            {
                "action": "same",
                "kind": "team",
                "identity": "viewers"
            },
            {
                "action": "would-update",
                "kind": "dashboard",
                "identity": "ops-overview"
            },
            {
                "action": "would-create",
                "kind": "folder",
                "identity": "platform"
            }
        ]
    });

    let view = build_workspace_review_view(&document).unwrap();

    let ordered = view
        .actions
        .iter()
        .map(|action| {
            (
                action.domain.as_str(),
                action.identity.as_str(),
                action.action.as_str(),
                action.status.as_str(),
                action.blocked_reason.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ordered,
        vec![
            ("folder", "platform", "would-create", "ready", None),
            ("dashboard", "ops-overview", "would-update", "ready", None),
            ("datasource", "remote-prom", "would-delete", "ready", None),
            (
                "alert",
                "ops-email",
                "blocked-read-only",
                "blocked",
                Some("target-read-only")
            ),
            ("access", "viewers", "same", "same", None),
        ]
    );

    let alert = view
        .actions
        .iter()
        .find(|action| action.identity == "ops-email")
        .unwrap();
    assert_eq!(
        alert.action_id,
        "alert:alert-contact-point:identity:ops-email"
    );
    assert_eq!(alert.resource_kind, "alert-contact-point");
    assert_eq!(alert.order_group, "review");
    assert_eq!(alert.kind_order, 3);
    assert_eq!(alert.details.as_deref(), Some("managed by provisioning"));
    assert_eq!(alert.raw["blockedReason"], json!("target-read-only"));
    assert_eq!(alert.raw["details"], json!("managed by provisioning"));
    assert_eq!(alert.raw["customEvidence"], json!({"owner": "platform"}));

    let datasource = view
        .actions
        .iter()
        .find(|action| action.identity == "remote-prom")
        .unwrap();
    assert_eq!(
        datasource.action_id,
        "datasource:datasource:uid:remote-prom"
    );
    assert_eq!(datasource.raw["name"], json!("Remote Prometheus"));
    assert_eq!(view.blocked_reasons, vec!["target-read-only"]);
}

#[test]
fn filter_review_plan_operations_uses_workspace_review_view_contract() {
    let plan = json!({
        "kind": "grafana-utils-sync-plan",
        "traceId": "sync-trace-review",
        "summary": {
            "would_create": 2,
            "would_update": 1,
            "would_delete": 0,
            "noop": 0,
            "unmanaged": 0,
            "alert_candidate": 1,
            "alert_plan_only": 0,
            "alert_blocked": 0
        },
        "reviewRequired": true,
        "reviewed": false,
        "operations": [
            {"kind":"datasource","identity":"prom-main","action":"would-update"},
            {"kind":"alert-contact-point","identity":"ops-email","action":"would-create"},
            {
                "kind":"dashboard",
                "identity":"blocked-main",
                "action":"blocked-read-only",
                "status":"blocked",
                "reason":"target-read-only"
            }
        ]
    });
    let selected = ["alert-contact-point::ops-email".to_string()]
        .into_iter()
        .collect();

    let filtered = review_tui::filter_review_plan_operations(&plan, &selected).unwrap();

    assert_eq!(filtered["summary"]["would_create"], json!(1));
    assert_eq!(filtered["summary"]["blockedCount"], json!(1));
    assert_eq!(filtered["domains"].as_array().unwrap().len(), 4);
    assert_eq!(filtered["blockedReasons"], json!(["target-read-only"]));
    assert_eq!(filtered["operations"].as_array().unwrap().len(), 2);
}
