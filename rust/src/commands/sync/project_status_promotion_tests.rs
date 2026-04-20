use super::project_status_promotion::build_promotion_domain_status;
use serde_json::json;

#[test]
fn build_promotion_domain_status_reports_blockers_and_explicit_signals() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 3,
            "missingMappingCount": 2,
            "bundleBlockingCount": 5,
            "blockingCount": 7,
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.id, "promotion");
    assert_eq!(status.scope, "staged");
    assert_eq!(status.mode, "artifact-summary");
    assert_eq!(status.status, "blocked");
    assert_eq!(status.reason_code, "blocked-by-blockers");
    assert_eq!(status.primary_count, 3);
    assert_eq!(status.blocker_count, 7);
    assert_eq!(status.source_kinds, vec!["promotion-preflight".to_string()]);
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.missingMappingCount".to_string(),
            "summary.bundleBlockingCount".to_string(),
            "summary.blockingCount".to_string(),
        ]
    );
    assert_eq!(status.blockers.len(), 2);
    assert_eq!(status.blockers[0].kind, "missing-mappings");
    assert_eq!(status.blockers[0].count, 2);
    assert_eq!(status.blockers[0].source, "summary.missingMappingCount");
    assert_eq!(status.blockers[1].kind, "bundle-blocking");
    assert_eq!(status.blockers[1].count, 5);
    assert_eq!(status.blockers[1].source, "summary.bundleBlockingCount");
    assert!(status.next_actions.contains(&"resolve promotion blockers in the fixed order: missing-mapping, bundle-blocking, blocking".to_string()));
}

#[test]
fn build_promotion_domain_status_reports_partial_when_no_resources() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 0,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.status, "partial");
    assert_eq!(status.reason_code, "partial-no-data");
    assert_eq!(status.primary_count, 0);
    assert_eq!(status.blocker_count, 0);
    assert_eq!(
        status.next_actions,
        vec!["stage at least one promotable resource before promotion".to_string()]
    );
}

#[test]
fn build_promotion_domain_status_reports_review_and_apply_evidence_when_ready() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 3,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        },
        "handoffSummary": {
            "reviewRequired": true,
            "readyForReview": true,
            "nextStage": "review",
            "blockingCount": 0,
            "reviewInstruction": "promotion handoff is ready to move into review",
        },
        "continuationSummary": {
            "stagedOnly": true,
            "liveMutationAllowed": false,
            "readyForContinuation": true,
            "nextStage": "staged-apply-continuation",
            "blockingCount": 0,
            "continuationInstruction": "reviewed remaps can continue into a staged apply continuation without enabling live mutation",
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.id, "promotion");
    assert_eq!(status.scope, "staged");
    assert_eq!(status.mode, "artifact-summary");
    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.primary_count, 3);
    assert_eq!(status.blocker_count, 0);
    assert_eq!(status.warning_count, 2);
    assert_eq!(status.source_kinds, vec!["promotion-preflight".to_string()]);
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.missingMappingCount".to_string(),
            "summary.bundleBlockingCount".to_string(),
            "summary.blockingCount".to_string(),
            "handoffSummary.reviewRequired".to_string(),
            "handoffSummary.readyForReview".to_string(),
            "handoffSummary.nextStage".to_string(),
            "handoffSummary.blockingCount".to_string(),
            "handoffSummary.reviewInstruction".to_string(),
            "continuationSummary.stagedOnly".to_string(),
            "continuationSummary.liveMutationAllowed".to_string(),
            "continuationSummary.readyForContinuation".to_string(),
            "continuationSummary.nextStage".to_string(),
            "continuationSummary.blockingCount".to_string(),
            "continuationSummary.resolvedCount".to_string(),
            "continuationSummary.continuationInstruction".to_string(),
        ]
    );
    assert_eq!(status.blockers.len(), 0);
    assert_eq!(status.warnings.len(), 2);
    assert_eq!(status.warnings[0].kind, "review-handoff");
    assert_eq!(status.warnings[0].count, 1);
    assert_eq!(status.warnings[0].source, "handoffSummary.readyForReview");
    assert_eq!(status.warnings[1].kind, "apply-continuation");
    assert_eq!(status.warnings[1].count, 1);
    assert_eq!(
        status.warnings[1].source,
        "continuationSummary.readyForContinuation"
    );
    assert_eq!(
        status.next_actions,
        vec![
            "promotion handoff is review-ready".to_string(),
            "promotion is apply-ready in the staged continuation".to_string(),
        ]
    );
}

#[test]
fn build_promotion_domain_status_prefers_resolved_apply_evidence_when_continuation_reports_it() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 3,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        },
        "handoffSummary": {
            "reviewRequired": true,
            "readyForReview": true,
            "nextStage": "review",
            "blockingCount": 0,
            "reviewInstruction": "promotion handoff is ready to move into review",
        },
        "continuationSummary": {
            "stagedOnly": true,
            "liveMutationAllowed": false,
            "readyForContinuation": true,
            "nextStage": "staged-apply-continuation",
            "resolvedCount": 4,
            "blockingCount": 0,
            "continuationInstruction": "reviewed remaps can continue into a staged apply continuation without enabling live mutation",
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.warning_count, 5);
    assert_eq!(status.warnings[1].kind, "apply-continuation");
    assert_eq!(status.warnings[1].count, 4);
    assert_eq!(
        status.warnings[1].source,
        "continuationSummary.resolvedCount"
    );
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.missingMappingCount".to_string(),
            "summary.bundleBlockingCount".to_string(),
            "summary.blockingCount".to_string(),
            "handoffSummary.reviewRequired".to_string(),
            "handoffSummary.readyForReview".to_string(),
            "handoffSummary.nextStage".to_string(),
            "handoffSummary.blockingCount".to_string(),
            "handoffSummary.reviewInstruction".to_string(),
            "continuationSummary.stagedOnly".to_string(),
            "continuationSummary.liveMutationAllowed".to_string(),
            "continuationSummary.readyForContinuation".to_string(),
            "continuationSummary.nextStage".to_string(),
            "continuationSummary.blockingCount".to_string(),
            "continuationSummary.resolvedCount".to_string(),
            "continuationSummary.continuationInstruction".to_string(),
        ]
    );
    assert_eq!(
        status.next_actions,
        vec![
            "promotion handoff is review-ready".to_string(),
            "promotion is apply-ready in the staged continuation".to_string(),
        ]
    );
}

#[test]
fn build_promotion_domain_status_uses_next_stage_evidence_when_instructions_are_missing() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 3,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        },
        "handoffSummary": {
            "reviewRequired": true,
            "readyForReview": true,
            "nextStage": "review",
            "blockingCount": 0,
        },
        "continuationSummary": {
            "stagedOnly": true,
            "liveMutationAllowed": false,
            "readyForContinuation": true,
            "nextStage": "staged-apply-continuation",
            "blockingCount": 0,
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.warning_count, 2);
    assert_eq!(status.warnings[0].source, "handoffSummary.nextStage");
    assert_eq!(status.warnings[1].source, "continuationSummary.nextStage");
    assert_eq!(
        status.next_actions,
        vec![
            "promotion handoff is review-ready".to_string(),
            "promotion is apply-ready in the staged continuation".to_string(),
        ]
    );
}

#[test]
fn build_promotion_domain_status_reports_instruction_sources_when_not_ready() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 3,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        },
        "handoffSummary": {
            "reviewRequired": true,
            "readyForReview": false,
            "nextStage": "review",
            "blockingCount": 0,
            "reviewInstruction": "promotion handoff is blocked until the staged review step is ready",
        },
        "continuationSummary": {
            "stagedOnly": true,
            "liveMutationAllowed": false,
            "readyForContinuation": false,
            "nextStage": "staged-apply-continuation",
            "blockingCount": 0,
            "continuationInstruction": "keep the promotion staged until the apply continuation is ready",
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.warning_count, 2);
    assert_eq!(
        status.warnings[0].source,
        "handoffSummary.reviewInstruction"
    );
    assert_eq!(
        status.warnings[1].source,
        "continuationSummary.continuationInstruction"
    );
    assert_eq!(
        status.next_actions,
        vec![
            "promotion handoff is blocked until the staged review step is ready".to_string(),
            "keep the promotion staged until the apply continuation is ready".to_string(),
        ]
    );
}

#[test]
fn build_promotion_domain_status_uses_nested_blocking_evidence_when_summary_is_clear() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 3,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        },
        "handoffSummary": {
            "reviewRequired": true,
            "readyForReview": false,
            "nextStage": "review",
            "blockingCount": 0,
            "reviewInstruction": "promotion handoff is blocked until the staged review step is ready",
        },
        "continuationSummary": {
            "stagedOnly": true,
            "liveMutationAllowed": false,
            "readyForContinuation": false,
            "nextStage": "staged-apply-continuation",
            "blockingCount": 2,
            "continuationInstruction": "keep the promotion staged until the apply continuation is ready",
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.status, "blocked");
    assert_eq!(status.reason_code, "blocked-by-blockers");
    assert_eq!(status.blocker_count, 2);
    assert_eq!(status.blockers.len(), 1);
    assert_eq!(status.blockers[0].kind, "blocking");
    assert_eq!(status.blockers[0].count, 2);
    assert_eq!(
        status.blockers[0].source,
        "continuationSummary.blockingCount"
    );
    assert_eq!(status.warning_count, 0);
    assert!(status.warnings.is_empty());
    assert_eq!(
        status.next_actions,
        vec!["resolve promotion blockers in the fixed order: missing-mapping, bundle-blocking, blocking".to_string()]
    );
}

#[test]
fn build_promotion_domain_status_surfaces_remap_complexity_signals_from_check_summary() {
    let document = json!({
        "kind": "grafana-utils-sync-promotion-preflight",
        "summary": {
            "resourceCount": 4,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        },
        "checkSummary": {
            "folderRemapCount": 1,
            "datasourceUidRemapCount": 2,
            "datasourceNameRemapCount": 3,
        },
        "handoffSummary": {
            "reviewRequired": true,
            "readyForReview": true,
            "nextStage": "review",
            "blockingCount": 0,
            "reviewInstruction": "promotion handoff is ready to move into review",
        },
        "continuationSummary": {
            "stagedOnly": true,
            "liveMutationAllowed": false,
            "readyForContinuation": true,
            "nextStage": "staged-apply-continuation",
            "blockingCount": 0,
            "continuationInstruction": "reviewed remaps can continue into a staged apply continuation without enabling live mutation",
        }
    });

    let status = build_promotion_domain_status(Some(&document)).unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.blocker_count, 0);
    assert_eq!(status.warning_count, 8);
    assert!(status
        .signal_keys
        .contains(&"checkSummary.folderRemapCount".to_string()));
    assert!(status
        .signal_keys
        .contains(&"checkSummary.datasourceUidRemapCount".to_string()));
    assert!(status
        .signal_keys
        .contains(&"checkSummary.datasourceNameRemapCount".to_string()));
    assert!(status
        .signal_keys
        .contains(&"checkSummary.resolvedCount".to_string()));
    assert!(status
        .signal_keys
        .contains(&"checkSummary.directCount".to_string()));
    assert!(status
        .signal_keys
        .contains(&"checkSummary.mappedCount".to_string()));
    assert!(status
        .signal_keys
        .contains(&"checkSummary.missingTargetCount".to_string()));
    assert!(status.warnings.iter().any(|warning| {
        warning.kind == "folder-remaps"
            && warning.count == 1
            && warning.source == "checkSummary.folderRemapCount"
    }));
    assert!(status.warnings.iter().any(|warning| {
        warning.kind == "datasource-uid-remaps"
            && warning.count == 2
            && warning.source == "checkSummary.datasourceUidRemapCount"
    }));
    assert!(status.warnings.iter().any(|warning| {
        warning.kind == "datasource-name-remaps"
            && warning.count == 3
            && warning.source == "checkSummary.datasourceNameRemapCount"
    }));
    assert_eq!(
        status.next_actions,
        vec![
            "promotion handoff is review-ready".to_string(),
            "review folder remaps before promotion review".to_string(),
            "review datasource remaps before promotion review".to_string(),
            "promotion is apply-ready in the staged continuation".to_string(),
        ]
    );
}
