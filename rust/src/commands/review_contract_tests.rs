use super::*;
use serde_json::json;

#[test]
fn review_apply_result_preserves_common_evidence_shape_with_domain_fields() {
    let mut result = ReviewApplyResult::new("apply");
    result.push_result(review_apply_result_entry(
        "grafana-alert-rule",
        "cpu-high",
        "create",
        json!({"uid": "cpu-high"}),
    ));

    let document = result.into_value_with_fields([
        ("kind", json!("grafana-util-alert-apply-result")),
        ("allowPolicyReset", json!(false)),
    ]);

    assert_eq!(
        document,
        json!({
            "kind": "grafana-util-alert-apply-result",
            "mode": "apply",
            "allowPolicyReset": false,
            "appliedCount": 1,
            "results": [{
                "kind": "grafana-alert-rule",
                "identity": "cpu-high",
                "action": "create",
                "response": {"uid": "cpu-high"}
            }]
        })
    );
}

#[test]
fn review_mutation_summary_rows_project_counts_and_blocked_reasons() {
    let envelope = build_review_mutation_envelope(
        vec![
            ReviewMutationActionInput {
                action_id: "dashboard:create:latency".to_string(),
                action: REVIEW_ACTION_WOULD_CREATE.to_string(),
                domain: "dashboard".to_string(),
                resource_kind: "grafana-dashboard".to_string(),
                identity: "latency".to_string(),
                status: REVIEW_STATUS_READY.to_string(),
                blocked_reason: None,
                details: None,
                review_hints: Vec::new(),
                raw: json!({}),
            }
            .into(),
            ReviewMutationActionInput {
                action_id: "datasource:extra:prometheus".to_string(),
                action: REVIEW_ACTION_EXTRA_REMOTE.to_string(),
                domain: "datasource".to_string(),
                resource_kind: "grafana-datasource".to_string(),
                identity: "prometheus".to_string(),
                status: REVIEW_STATUS_WARNING.to_string(),
                blocked_reason: None,
                details: None,
                review_hints: Vec::new(),
                raw: json!({}),
            }
            .into(),
            ReviewMutationActionInput {
                action_id: "access:blocked:viewer".to_string(),
                action: REVIEW_ACTION_BLOCKED.to_string(),
                domain: "access".to_string(),
                resource_kind: "grafana-user".to_string(),
                identity: "viewer@example.com".to_string(),
                status: REVIEW_STATUS_BLOCKED.to_string(),
                blocked_reason: Some("externally synced user".to_string()),
                details: None,
                review_hints: Vec::new(),
                raw: json!({}),
            }
            .into(),
        ],
        &["dashboard", "datasource", "access"],
    );

    let rows = build_review_mutation_summary_rows(&envelope);

    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.action_count == 3));
    assert!(rows.iter().all(|row| row.domain_count == 3));
    assert!(rows.iter().all(|row| row.blocked_count == 1));
    assert!(rows.iter().all(|row| row.warning_count == 1));
    assert!(rows
        .iter()
        .all(|row| row.blocked_reasons == vec!["externally synced user".to_string()]));
}

#[test]
fn review_mutation_action_detail_lines_project_generic_review_evidence() {
    let blocked = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:alice".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "alice".to_string(),
        status: REVIEW_STATUS_BLOCKED.to_string(),
        blocked_reason: Some("externally synced user".to_string()),
        details: Some("fields=orgRole".to_string()),
        review_hints: Vec::new(),
        raw: json!({}),
    });
    let warning = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:team:ops".to_string(),
        action: REVIEW_ACTION_WOULD_CREATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "team".to_string(),
        identity: "ops".to_string(),
        status: REVIEW_STATUS_WARNING.to_string(),
        blocked_reason: None,
        details: None,
        review_hints: Vec::new(),
        raw: json!({}),
    });

    let blocked_lines = build_review_mutation_action_detail_lines(&blocked);
    let warning_lines = build_review_mutation_action_detail_lines(&warning);

    assert!(blocked_lines
        .iter()
        .any(|line| line == "Review action: would-update (status=blocked)"));
    assert!(blocked_lines
        .iter()
        .any(|line| line == "Review identity: user alice"));
    assert!(blocked_lines
        .iter()
        .any(|line| line == "Review details: fields=orgRole"));
    assert!(blocked_lines
        .iter()
        .any(|line| line == "Review blocker status: blocked by externally synced user"));
    assert!(warning_lines
        .iter()
        .any(|line| line == "Review action: would-create (status=warning)"));
    assert!(warning_lines
        .iter()
        .any(|line| line == "Review identity: team ops"));
    assert!(!warning_lines
        .iter()
        .any(|line| line.starts_with("Review blocker status:")));
}

#[test]
fn review_mutation_action_next_check_lines_project_hints_and_default_guidance() {
    let blocked = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:alice".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "alice".to_string(),
        status: REVIEW_STATUS_BLOCKED.to_string(),
        blocked_reason: Some("externally synced user".to_string()),
        details: Some("fields=orgRole".to_string()),
        review_hints: vec!["review identity source".to_string()],
        raw: json!({}),
    });
    let remote_only = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:bob".to_string(),
        action: REVIEW_ACTION_EXTRA_REMOTE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "bob".to_string(),
        status: REVIEW_STATUS_WARNING.to_string(),
        blocked_reason: None,
        details: None,
        review_hints: vec![REVIEW_HINT_REMOTE_ONLY.to_string()],
        raw: json!({}),
    });

    assert_eq!(
        build_review_mutation_action_next_check_lines(&blocked),
        vec![
            "Check next: review identity source.".to_string(),
            "Check next: confirm the blocker in Grafana and adjust the bundle or remote ownership before retrying.".to_string(),
        ]
    );
    assert_eq!(
        build_review_mutation_action_next_check_lines(&remote_only),
        vec![
            "Check next: decide whether this live-only record should stay unmanaged or be deleted."
                .to_string(),
            "Check next: review the warning evidence and verify operator intent.".to_string(),
        ]
    );
}

#[test]
fn review_mutation_action_diff_preview_lines_hide_secret_like_fields() {
    let action = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:alice".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "alice".to_string(),
        status: REVIEW_STATUS_WARNING.to_string(),
        blocked_reason: None,
        details: Some("fields=email".to_string()),
        review_hints: Vec::new(),
        raw: json!({
            "changes": [
                {
                    "field": "email",
                    "before": "alice@example.com",
                    "after": "alice-old@example.com"
                },
                {
                    "field": "password",
                    "before": "new-secret",
                    "after": "old-secret"
                }
            ]
        }),
    });

    let lines = build_review_mutation_action_diff_preview_lines(&action);
    let rendered = lines.join("\n");

    assert!(rendered.contains("Shared Diff: user alice [would-update]"));
    assert!(rendered.contains("email"));
    assert!(!rendered.contains("password"));
    assert!(!rendered.contains("new-secret"));
    assert!(!rendered.contains("old-secret"));
}

#[test]
fn review_mutation_action_change_detail_lines_hide_secret_like_fields() {
    let action = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:alice".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "alice".to_string(),
        status: REVIEW_STATUS_WARNING.to_string(),
        blocked_reason: None,
        details: Some("fields=email".to_string()),
        review_hints: Vec::new(),
        raw: json!({
            "changes": [
                {
                    "field": "email",
                    "before": "alice@example.com",
                    "after": "alice-old@example.com"
                },
                {
                    "field": "password",
                    "before": "new-secret",
                    "after": "old-secret"
                }
            ]
        }),
    });

    let lines = build_review_mutation_action_change_detail_lines(&action);
    let rendered = lines.join("\n");

    assert_eq!(
        lines,
        vec!["Change: email bundle=alice@example.com live=alice-old@example.com"]
    );
    assert!(!rendered.contains("password"));
    assert!(!rendered.contains("new-secret"));
    assert!(!rendered.contains("old-secret"));
}

#[test]
fn review_mutation_action_target_evidence_lines_project_known_live_target_fields() {
    let action = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:alice".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "alice".to_string(),
        status: REVIEW_STATUS_BLOCKED.to_string(),
        blocked_reason: Some("externally synced user".to_string()),
        details: Some("fields=orgRole".to_string()),
        review_hints: Vec::new(),
        raw: json!({
            "target": {
                "login": "alice",
                "orgRole": "Viewer",
                "isExternal": false,
                "ignored": "not shown"
            }
        }),
    });

    assert_eq!(
        build_review_mutation_action_target_evidence_lines(&action),
        vec![
            "Live target: login=alice".to_string(),
            "Live target: orgRole=Viewer".to_string(),
        ]
    );
}

#[test]
fn review_mutation_action_context_lines_project_warning_and_blocker_evidence() {
    let action = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:alice".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "alice".to_string(),
        status: REVIEW_STATUS_BLOCKED.to_string(),
        blocked_reason: Some("externally synced user".to_string()),
        details: Some("fields=orgRole,password".to_string()),
        review_hints: Vec::new(),
        raw: json!({
            "changedFields": ["orgRole", "password"],
            "target": {
                "isExternal": true,
                "isProvisioned": false,
                "disabled": false,
                "ignored": true
            }
        }),
    });

    assert_eq!(
        build_review_mutation_action_context_lines(&action),
        vec![
            "Blocked context: externally synced user.".to_string(),
            "Blocked evidence: live target flags isExternal=true isProvisioned=false disabled=false.".to_string(),
        ]
    );

    let warning = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:bob".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "bob".to_string(),
        status: REVIEW_STATUS_WARNING.to_string(),
        blocked_reason: None,
        details: Some("fields=orgRole,password".to_string()),
        review_hints: Vec::new(),
        raw: json!({
            "changedFields": ["orgRole", "password"]
        }),
    });

    assert_eq!(
        build_review_mutation_action_context_lines(&warning),
        vec![
            "Warning context: verify bundle fields orgRole against the live target before approving.".to_string(),
        ]
    );
}

#[test]
fn review_mutation_action_narrative_and_impact_lines_project_action_guidance() {
    let update = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:alice".to_string(),
        action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "alice".to_string(),
        status: REVIEW_STATUS_WARNING.to_string(),
        blocked_reason: None,
        details: Some("fields=orgRole".to_string()),
        review_hints: Vec::new(),
        raw: json!({
            "changedFields": ["orgRole", "password"]
        }),
    });
    let delete = ReviewMutationAction::from(ReviewMutationActionInput {
        action_id: "access:user:bob".to_string(),
        action: REVIEW_ACTION_WOULD_DELETE.to_string(),
        domain: "access".to_string(),
        resource_kind: "user".to_string(),
        identity: "bob".to_string(),
        status: REVIEW_STATUS_WARNING.to_string(),
        blocked_reason: None,
        details: None,
        review_hints: Vec::new(),
        raw: json!({}),
    });

    assert_eq!(
        build_review_mutation_action_narrative_line(&update),
        "Narrative: changes this live user so it matches the reviewed bundle."
    );
    assert_eq!(
        build_review_mutation_action_impact_line(&update),
        Some("Why this matters: permission or administrative reach would change.".to_string())
    );
    assert_eq!(
        build_review_mutation_action_narrative_line(&delete),
        "Narrative: removes this live-only user because prune review marked it for deletion."
    );
    assert_eq!(
        build_review_mutation_action_impact_line(&delete),
        Some("Why this matters: the live record would disappear after apply.".to_string())
    );
}

#[test]
fn append_review_evidence_section_adds_heading_only_for_non_empty_lines() {
    let mut lines = vec!["Name: Prometheus".to_string()];

    append_review_evidence_section(&mut lines, Vec::new());
    assert_eq!(lines, vec!["Name: Prometheus".to_string()]);

    append_review_evidence_section(
        &mut lines,
        vec![
            "Review action: would-update (status=ready)".to_string(),
            "Review changed fields: url".to_string(),
        ],
    );

    assert_eq!(
        lines,
        vec![
            "Name: Prometheus".to_string(),
            "Review evidence:".to_string(),
            "Review action: would-update (status=ready)".to_string(),
            "Review changed fields: url".to_string(),
        ]
    );
}
