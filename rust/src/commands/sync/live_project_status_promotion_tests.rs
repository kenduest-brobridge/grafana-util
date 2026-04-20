use super::live_project_status_promotion::{
    build_live_promotion_project_status, LivePromotionProjectStatusInputs,
};
use serde_json::json;

#[test]
fn build_live_promotion_project_status_returns_none_without_inputs() {
    assert!(
        build_live_promotion_project_status(LivePromotionProjectStatusInputs::default()).is_none()
    );
}

#[test]
fn build_live_promotion_project_status_reports_blocked_when_summary_has_blockers() {
    let summary = json!({
        "kind": "grafana-utils-sync-promotion-summary",
        "summary": {
            "resourceCount": 4,
            "missingMappingCount": 2,
            "bundleBlockingCount": 3,
            "blockingCount": 5,
        }
    });
    let mapping = json!({
        "kind": "grafana-utils-sync-promotion-mapping",
        "folders": {"ops-src": "ops-prod"},
        "datasources": {
            "uids": {"prom-src": "prom-prod"},
            "names": {"Prometheus Source": "Prometheus Prod"}
        }
    });
    let availability = json!({
        "pluginIds": ["prometheus"],
        "datasourceUids": ["prom-prod"],
        "contactPoints": ["pagerduty-primary"]
    });

    let status = build_live_promotion_project_status(LivePromotionProjectStatusInputs {
        promotion_summary_document: Some(&summary),
        promotion_mapping_document: Some(&mapping),
        availability_document: Some(&availability),
    })
    .unwrap();

    assert_eq!(status.id, "promotion");
    assert_eq!(status.scope, "live");
    assert_eq!(status.mode, "live-promotion-surfaces");
    assert_eq!(status.status, "blocked");
    assert_eq!(status.reason_code, "blocked-by-blockers");
    assert_eq!(status.primary_count, 4);
    assert_eq!(status.blocker_count, 5);
    assert_eq!(
        status.source_kinds,
        vec![
            "live-promotion-summary".to_string(),
            "live-promotion-mapping".to_string(),
            "live-promotion-availability".to_string(),
        ]
    );
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.missingMappingCount".to_string(),
            "summary.bundleBlockingCount".to_string(),
            "summary.blockingCount".to_string(),
            "mapping.entryCount".to_string(),
            "availability.entryCount".to_string(),
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
            "continuationSummary.continuationInstruction".to_string(),
        ]
    );
    assert_eq!(status.blockers.len(), 2);
    assert_eq!(status.blockers[0].kind, "missing-mappings");
    assert_eq!(status.blockers[0].count, 2);
    assert_eq!(status.blockers[0].source, "summary.missingMappingCount");
    assert_eq!(status.blockers[1].kind, "bundle-blocking");
    assert_eq!(status.blockers[1].count, 3);
    assert_eq!(status.blockers[1].source, "summary.bundleBlockingCount");
    assert_eq!(
        status.next_actions,
        vec!["resolve promotion blockers in the fixed order: missing-mapping, bundle-blocking, blocking".to_string()]
    );
}

#[test]
fn build_live_promotion_project_status_reports_partial_when_inputs_are_incomplete() {
    let summary = json!({
        "kind": "grafana-utils-sync-promotion-summary",
        "summary": {
            "resourceCount": 0,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        }
    });
    let mapping = json!({
        "kind": "grafana-utils-sync-promotion-mapping",
        "folders": {},
        "datasources": {
            "uids": {},
            "names": {}
        }
    });

    let status = build_live_promotion_project_status(LivePromotionProjectStatusInputs {
        promotion_summary_document: Some(&summary),
        promotion_mapping_document: Some(&mapping),
        availability_document: None,
    })
    .unwrap();

    assert_eq!(status.status, "partial");
    assert_eq!(status.reason_code, "partial-no-data");
    assert_eq!(status.primary_count, 0);
    assert_eq!(status.blocker_count, 0);
    assert_eq!(
        status.next_actions,
        vec![
            "stage at least one promotable resource before promotion".to_string(),
            "provide explicit promotion mappings before promotion".to_string(),
            "provide live availability hints before promotion".to_string(),
        ]
    );
}

#[test]
fn build_live_promotion_project_status_reports_ready_from_consistent_inputs() {
    let summary = json!({
        "kind": "grafana-utils-sync-promotion-summary",
        "summary": {
            "resourceCount": 3,
            "missingMappingCount": 0,
            "bundleBlockingCount": 0,
            "blockingCount": 0,
        }
    });
    let mapping = json!({
        "kind": "grafana-utils-sync-promotion-mapping",
        "folders": {"ops-src": "ops-prod"},
        "datasources": {
            "uids": {"prom-src": "prom-prod"},
            "names": {"Prometheus Source": "Prometheus Prod"}
        }
    });
    let availability = json!({
        "pluginIds": ["prometheus", "timeseries"],
        "datasourceUids": ["prom-prod"],
        "datasourceNames": ["Prometheus Prod"],
        "contactPoints": ["pagerduty-primary"],
        "providerNames": ["vault"],
        "secretPlaceholderNames": ["prom-basic-auth"]
    });

    let status = build_live_promotion_project_status(LivePromotionProjectStatusInputs {
        promotion_summary_document: Some(&summary),
        promotion_mapping_document: Some(&mapping),
        availability_document: Some(&availability),
    })
    .unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.primary_count, 3);
    assert!(status.blockers.is_empty());
    assert!(status.next_actions.is_empty());
}

#[test]
fn build_live_promotion_project_status_reports_review_ready_handoff_and_apply_waiting_continuation()
{
    let summary = json!({
        "kind": "grafana-utils-sync-promotion-summary",
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
            "readyForContinuation": false,
            "nextStage": "resolve-blockers",
            "blockingCount": 0,
            "continuationInstruction": "keep the promotion staged until the apply continuation is ready",
        }
    });
    let mapping = json!({
        "kind": "grafana-utils-sync-promotion-mapping",
        "folders": {"ops-src": "ops-prod"},
        "datasources": {
            "uids": {"prom-src": "prom-prod"},
            "names": {"Prometheus Source": "Prometheus Prod"}
        }
    });
    let availability = json!({
        "pluginIds": ["prometheus", "timeseries"],
        "datasourceUids": ["prom-prod"],
        "datasourceNames": ["Prometheus Prod"],
        "contactPoints": ["pagerduty-primary"],
        "providerNames": ["vault"],
        "secretPlaceholderNames": ["prom-basic-auth"]
    });

    let status = build_live_promotion_project_status(LivePromotionProjectStatusInputs {
        promotion_summary_document: Some(&summary),
        promotion_mapping_document: Some(&mapping),
        availability_document: Some(&availability),
    })
    .unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.warning_count, 2);
    assert_eq!(
        status.next_actions,
        vec![
            "promotion handoff is review-ready".to_string(),
            "keep the promotion staged until the apply continuation is ready".to_string(),
        ]
    );
}

#[test]
fn build_live_promotion_project_status_reports_apply_ready_continuation_after_review_ready_handoff()
{
    let summary = json!({
        "kind": "grafana-utils-sync-promotion-summary",
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
            "resolvedCount": 1,
            "blockingCount": 0,
            "continuationInstruction": "reviewed remaps can continue into a staged apply continuation without enabling live mutation",
        }
    });
    let mapping = json!({
        "kind": "grafana-utils-sync-promotion-mapping",
        "folders": {"ops-src": "ops-prod"},
        "datasources": {
            "uids": {"prom-src": "prom-prod"},
            "names": {"Prometheus Source": "Prometheus Prod"}
        }
    });
    let availability = json!({
        "pluginIds": ["prometheus", "timeseries"],
        "datasourceUids": ["prom-prod"],
        "datasourceNames": ["Prometheus Prod"],
        "contactPoints": ["pagerduty-primary"],
        "providerNames": ["vault"],
        "secretPlaceholderNames": ["prom-basic-auth"]
    });

    let status = build_live_promotion_project_status(LivePromotionProjectStatusInputs {
        promotion_summary_document: Some(&summary),
        promotion_mapping_document: Some(&mapping),
        availability_document: Some(&availability),
    })
    .unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.warning_count, 2);
    assert_eq!(
        status.next_actions,
        vec![
            "promotion handoff is review-ready".to_string(),
            "promotion is apply-ready in the staged continuation".to_string(),
        ]
    );
}
