use super::project_status::{build_sync_domain_status, SyncDomainStatusInputs};
use crate::project_status::status_finding;
use serde_json::json;

#[test]
fn build_sync_domain_status_reports_bundle_preflight_artifact_evidence() {
    let summary = json!({
        "kind": "grafana-utils-sync-summary",
        "summary": {
            "resourceCount": 4,
        }
    });
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 4,
            "syncBlockingCount": 1,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 2,
            "alertArtifactPlanOnlyCount": 3,
            "alertArtifactCount": 5,
        }
    });

    let status = build_sync_domain_status(SyncDomainStatusInputs {
        summary_document: Some(&summary),
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(status.id, "sync");
    assert_eq!(status.scope, "staged");
    assert_eq!(status.mode, "staged-documents");
    assert_eq!(status.status, "blocked");
    assert_eq!(status.reason_code, "blocked-by-blockers");
    assert_eq!(status.primary_count, 4);
    assert_eq!(
        status.source_kinds,
        vec!["sync-summary".to_string(), "package-test".to_string()]
    );
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.syncBlockingCount".to_string(),
            "summary.providerBlockingCount".to_string(),
            "summary.secretPlaceholderBlockingCount".to_string(),
            "summary.alertArtifactBlockedCount".to_string(),
            "summary.alertArtifactPlanOnlyCount".to_string(),
            "summary.alertArtifactCount".to_string(),
        ]
    );
    assert_eq!(status.blockers.len(), 2);
    assert_eq!(status.blockers[0].kind, "sync-blocking");
    assert_eq!(status.blockers[0].count, 1);
    assert_eq!(status.blockers[0].source, "summary.syncBlockingCount");
    assert_eq!(status.blockers[1].kind, "alert-artifact-blocking");
    assert_eq!(status.blockers[1].count, 2);
    assert_eq!(
        status.blockers[1].source,
        "summary.alertArtifactBlockedCount"
    );
    assert_eq!(status.warnings.len(), 1);
    assert_eq!(status.warnings[0].kind, "alert-artifact-plan-only");
    assert_eq!(status.warnings[0].count, 3);
    assert_eq!(
        status.warnings[0].source,
        "summary.alertArtifactPlanOnlyCount"
    );
    assert_eq!(
        status.next_actions,
        vec!["resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact".to_string()]
    );
}

#[test]
fn build_sync_domain_status_reports_bundle_preflight_resource_source_when_summary_is_missing() {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 4,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
            "alertArtifactCount": 0,
        }
    });

    let status = build_sync_domain_status(SyncDomainStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(status.id, "sync");
    assert_eq!(status.scope, "staged");
    assert_eq!(status.mode, "staged-documents");
    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.primary_count, 4);
    assert_eq!(status.source_kinds, vec!["package-test".to_string()]);
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.syncBlockingCount".to_string(),
            "summary.providerBlockingCount".to_string(),
            "summary.secretPlaceholderBlockingCount".to_string(),
            "summary.alertArtifactBlockedCount".to_string(),
            "summary.alertArtifactPlanOnlyCount".to_string(),
            "summary.alertArtifactCount".to_string(),
        ]
    );
    assert_eq!(status.blockers.len(), 0);
    assert_eq!(status.warnings.len(), 0);
    assert_eq!(
        status.next_actions,
        vec!["re-run sync summary after staged changes".to_string()]
    );
}

#[test]
fn build_sync_domain_status_surfaces_alert_artifact_review_evidence_without_blocking_or_plan_only()
{
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 4,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
            "alertArtifactCount": 3,
        }
    });

    let status = build_sync_domain_status(SyncDomainStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.blocker_count, 0);
    assert_eq!(status.warning_count, 3);
    assert_eq!(status.warnings.len(), 1);
    assert_eq!(status.warnings[0].kind, "alert-artifact-review");
    assert_eq!(status.warnings[0].count, 3);
    assert_eq!(status.warnings[0].source, "summary.alertArtifactCount");
    assert_eq!(
        status.next_actions,
        vec!["review non-blocking sync findings before promotion or apply".to_string()]
    );
}

#[test]
fn build_sync_domain_status_uses_nested_alert_artifact_evidence_when_summary_counts_are_missing() {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 2,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
            "alertArtifactCount": 0,
        },
        "alertArtifactAssessment": {
            "summary": {
                "resourceCount": 2,
                "blockedCount": 1,
                "planOnlyCount": 2,
            }
        }
    });

    let status = build_sync_domain_status(SyncDomainStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(status.status, "blocked");
    assert_eq!(status.reason_code, "blocked-by-blockers");
    assert_eq!(status.blocker_count, 1);
    assert_eq!(status.warning_count, 2);
    assert_eq!(status.blockers.len(), 1);
    assert_eq!(status.blockers[0].kind, "alert-artifact-blocking");
    assert_eq!(status.blockers[0].count, 1);
    assert_eq!(
        status.blockers[0].source,
        "alertArtifactAssessment.summary.blockedCount"
    );
    assert_eq!(status.warnings.len(), 1);
    assert_eq!(status.warnings[0].kind, "alert-artifact-plan-only");
    assert_eq!(status.warnings[0].count, 2);
    assert_eq!(
        status.warnings[0].source,
        "alertArtifactAssessment.summary.planOnlyCount"
    );
    assert_eq!(
        status.next_actions,
        vec!["resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact".to_string()]
    );
}

#[test]
fn build_sync_domain_status_uses_nested_provider_assessment_evidence_when_summary_counts_are_missing(
) {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 2,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
            "alertArtifactCount": 0,
        },
        "providerAssessment": {
            "summary": {
                "blockingCount": 3
            }
        }
    });

    let status = build_sync_domain_status(SyncDomainStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(status.status, "blocked");
    assert_eq!(status.reason_code, "blocked-by-blockers");
    assert_eq!(status.blocker_count, 3);
    assert_eq!(status.blockers.len(), 1);
    assert_eq!(status.blockers[0].kind, "provider-blocking");
    assert_eq!(status.blockers[0].count, 3);
    assert_eq!(
        status.blockers[0].source,
        "providerAssessment.summary.blockingCount"
    );
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.syncBlockingCount".to_string(),
            "summary.providerBlockingCount".to_string(),
            "summary.secretPlaceholderBlockingCount".to_string(),
            "summary.alertArtifactBlockedCount".to_string(),
            "summary.alertArtifactPlanOnlyCount".to_string(),
            "summary.alertArtifactCount".to_string(),
            "providerAssessment.summary.blockingCount".to_string(),
            "providerAssessment.plans".to_string(),
        ]
    );
    assert_eq!(status.warnings.len(), 0);
    assert_eq!(
        status.next_actions,
        vec!["resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact".to_string()]
    );
}

#[test]
fn build_sync_domain_status_surfaces_provider_and_secret_review_evidence_without_blocking() {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 2,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
            "alertArtifactCount": 0,
        },
        "providerAssessment": {
            "summary": {
                "blockingCount": 0
            },
            "plans": [
                {"providerKind": "vault"},
                {"providerKind": "aws-secrets-manager"}
            ]
        },
        "secretPlaceholderAssessment": {
            "summary": {
                "blockingCount": 0
            },
            "plans": [
                {"providerKind": "vault"}
            ]
        }
    });

    let status = build_sync_domain_status(SyncDomainStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(status.status, "ready");
    assert_eq!(status.reason_code, "ready");
    assert_eq!(status.blocker_count, 0);
    assert_eq!(status.warning_count, 3);
    assert_eq!(
        status.warnings,
        vec![
            status_finding("provider-review", 2, "providerAssessment.plans"),
            status_finding(
                "secret-placeholder-review",
                1,
                "secretPlaceholderAssessment.plans"
            ),
        ]
    );
    assert_eq!(
        status.next_actions,
        vec!["review non-blocking sync findings before promotion or apply".to_string()]
    );
}

#[test]
fn build_sync_domain_status_uses_nested_secret_placeholder_assessment_evidence_when_summary_counts_are_missing(
) {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 2,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
            "alertArtifactCount": 0,
        },
        "secretPlaceholderAssessment": {
            "summary": {
                "blockingCount": 2
            }
        }
    });

    let status = build_sync_domain_status(SyncDomainStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(status.status, "blocked");
    assert_eq!(status.reason_code, "blocked-by-blockers");
    assert_eq!(status.blocker_count, 2);
    assert_eq!(status.blockers.len(), 1);
    assert_eq!(status.blockers[0].kind, "secret-placeholder-blocking");
    assert_eq!(status.blockers[0].count, 2);
    assert_eq!(
        status.blockers[0].source,
        "secretPlaceholderAssessment.summary.blockingCount"
    );
    assert_eq!(
        status.signal_keys,
        vec![
            "summary.resourceCount".to_string(),
            "summary.syncBlockingCount".to_string(),
            "summary.providerBlockingCount".to_string(),
            "summary.secretPlaceholderBlockingCount".to_string(),
            "summary.alertArtifactBlockedCount".to_string(),
            "summary.alertArtifactPlanOnlyCount".to_string(),
            "summary.alertArtifactCount".to_string(),
            "secretPlaceholderAssessment.summary.blockingCount".to_string(),
            "secretPlaceholderAssessment.plans".to_string(),
        ]
    );
    assert_eq!(status.warnings.len(), 0);
    assert_eq!(
        status.next_actions,
        vec!["resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact".to_string()]
    );
}
