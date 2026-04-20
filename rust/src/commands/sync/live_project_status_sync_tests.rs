use super::live_project_status_sync::{build_live_sync_domain_status, SyncLiveProjectStatusInputs};
use serde_json::json;

#[test]
fn build_live_sync_domain_status_returns_none_without_any_surfaces() {
    assert!(build_live_sync_domain_status(SyncLiveProjectStatusInputs::default()).is_none());
}

#[test]
fn build_live_sync_domain_status_reports_blockers_from_bundle_preflight() {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 4,
            "syncBlockingCount": 1,
            "providerBlockingCount": 2,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactCount": 4,
            "alertArtifactBlockedCount": 3,
            "alertArtifactPlanOnlyCount": 1,
            "blockedCount": 2,
            "planOnlyCount": 1,
        }
    });

    let domain = build_live_sync_domain_status(SyncLiveProjectStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();
    let value = serde_json::to_value(domain).unwrap();

    assert_eq!(value["id"], json!("sync"));
    assert_eq!(value["scope"], json!("live"));
    assert_eq!(value["mode"], json!("live-sync-surfaces"));
    assert_eq!(value["status"], json!("blocked"));
    assert_eq!(value["reasonCode"], json!("blocked-by-blockers"));
    assert_eq!(value["primaryCount"], json!(4));
    assert_eq!(value["blockerCount"], json!(6));
    assert_eq!(value["warningCount"], json!(1));
    assert_eq!(value["sourceKinds"], json!(["package-test"]));
    assert_eq!(
        value["signalKeys"],
        json!([
            "summary.resourceCount",
            "summary.syncBlockingCount",
            "summary.providerBlockingCount",
            "summary.secretPlaceholderBlockingCount",
            "summary.alertArtifactCount",
            "summary.alertArtifactBlockedCount",
            "summary.alertArtifactPlanOnlyCount",
            "summary.blockedCount",
            "summary.planOnlyCount",
        ])
    );
    assert_eq!(value["blockers"].as_array().unwrap().len(), 3);
    assert_eq!(
        value["warnings"],
        json!([{
            "kind": "alert-artifact-plan-only",
            "count": 1,
            "source": "summary.alertArtifactPlanOnlyCount"
        }])
    );
    assert!(value["nextActions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact"));
}

#[test]
fn build_live_sync_domain_status_is_partial_when_only_summary_exists() {
    let summary = json!({
        "kind": "grafana-utils-sync-summary",
        "summary": {
            "resourceCount": 2,
        }
    });

    let domain = build_live_sync_domain_status(SyncLiveProjectStatusInputs {
        summary_document: Some(&summary),
        bundle_preflight_document: None,
    })
    .unwrap();
    let value = serde_json::to_value(domain).unwrap();

    assert_eq!(value["status"], json!("partial"));
    assert_eq!(value["reasonCode"], json!("partial-missing-surfaces"));
    assert_eq!(value["primaryCount"], json!(2));
    assert_eq!(value["sourceKinds"], json!(["sync-summary"]));
    assert_eq!(value["signalKeys"], json!(["summary.resourceCount"]));
    assert_eq!(
        value["nextActions"],
        json!(["provide a staged package-test document before interpreting live sync readiness"])
    );
}

#[test]
fn build_live_sync_domain_status_keeps_summary_and_bundle_sources_additive() {
    let summary = json!({
        "kind": "grafana-utils-sync-summary",
        "summary": {
            "resourceCount": 2,
        }
    });
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 2,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactCount": 1,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
            "blockedCount": 0,
            "planOnlyCount": 0,
        }
    });

    let domain = build_live_sync_domain_status(SyncLiveProjectStatusInputs {
        summary_document: Some(&summary),
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();
    let value = serde_json::to_value(domain).unwrap();

    assert_eq!(value["status"], json!("ready"));
    assert_eq!(
        value["sourceKinds"],
        json!(["sync-summary", "package-test"])
    );
    assert_eq!(
        value["signalKeys"],
        json!([
            "summary.resourceCount",
            "summary.syncBlockingCount",
            "summary.providerBlockingCount",
            "summary.secretPlaceholderBlockingCount",
            "summary.alertArtifactCount",
            "summary.alertArtifactBlockedCount",
            "summary.alertArtifactPlanOnlyCount",
            "summary.blockedCount",
            "summary.planOnlyCount",
        ])
    );
    assert_eq!(value["warningCount"], json!(0));
    assert_eq!(
        value["nextActions"],
        json!(["re-run sync summary after staged changes"])
    );
}

#[test]
fn build_live_sync_domain_status_is_partial_when_bundle_preflight_has_no_resources() {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 0,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "secretPlaceholderBlockingCount": 0,
            "alertArtifactBlockedCount": 0,
            "alertArtifactPlanOnlyCount": 0,
        }
    });

    let domain = build_live_sync_domain_status(SyncLiveProjectStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();

    assert_eq!(domain.status, "partial");
    assert_eq!(domain.reason_code, "partial-no-data");
    assert_eq!(domain.primary_count, 0);
    assert_eq!(
        domain.next_actions,
        vec!["stage at least one dashboard, datasource, or alert resource".to_string()]
    );
}

#[test]
fn build_live_sync_domain_status_falls_back_to_generic_bundle_summary_keys() {
    let bundle_preflight = json!({
        "kind": "grafana-utils-sync-bundle-preflight",
        "summary": {
            "resourceCount": 3,
            "blockedCount": 2,
            "planOnlyCount": 1
        }
    });

    let domain = build_live_sync_domain_status(SyncLiveProjectStatusInputs {
        summary_document: None,
        bundle_preflight_document: Some(&bundle_preflight),
    })
    .unwrap();
    let value = serde_json::to_value(domain).unwrap();

    assert_eq!(value["status"], json!("blocked"));
    assert_eq!(value["reasonCode"], json!("blocked-by-blockers"));
    assert_eq!(
        value["blockers"],
        json!([{
            "kind": "bundle-blocking",
            "count": 2,
            "source": "summary.blockedCount"
        }])
    );
}
