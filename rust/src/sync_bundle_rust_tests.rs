use crate::sync_bundle_preflight::{
    build_sync_bundle_preflight_document, render_sync_bundle_preflight_text,
    SYNC_BUNDLE_PREFLIGHT_KIND,
};
use crate::sync_workbench::build_sync_source_bundle_document;
use serde_json::json;

#[test]
fn build_sync_bundle_preflight_document_aggregates_sync_and_provider_checks() {
    let source_bundle = json!({
        "dashboards": [
            {
                "kind": "dashboard",
                "uid": "cpu-main",
                "title": "CPU Main",
                "body": {"datasourceUids": ["prom-main"]}
            }
        ],
        "datasources": [
            {
                "kind": "datasource",
                "uid": "prom-main",
                "name": "Prometheus Main",
                "body": {"type": "prometheus"},
                "secureJsonDataProviders": {
                    "httpHeaderValue1": "${provider:vault:secret/data/prom/token}"
                }
            }
        ],
        "alerts": [
            {
                "kind": "alert",
                "uid": "cpu-high",
                "title": "CPU High",
                "managedFields": ["condition", "contactPoints"],
                "body": {"condition": "A > 90", "contactPoints": ["pagerduty-primary"]}
            }
        ]
    });
    let target_inventory = json!({"dashboards": [], "datasources": []});
    let availability = json!({
        "pluginIds": [],
        "datasourceUids": [],
        "contactPoints": [],
        "providerNames": []
    });

    let document = build_sync_bundle_preflight_document(
        &source_bundle,
        &target_inventory,
        Some(&availability),
    )
    .unwrap();

    assert_eq!(document["kind"], json!(SYNC_BUNDLE_PREFLIGHT_KIND));
    assert!(document["summary"]["syncBlockingCount"].as_i64().unwrap() >= 1);
    assert_eq!(document["summary"]["providerBlockingCount"], json!(1));
    assert_eq!(
        document["providerAssessment"]["plans"][0]["providerKind"],
        json!("external-provider-reference")
    );
}

#[test]
fn render_sync_bundle_preflight_text_renders_summary() {
    let document = build_sync_bundle_preflight_document(
        &json!({"folders": [{"kind": "folder", "uid": "ops", "title": "Operations"}]}),
        &json!({}),
        None,
    )
    .unwrap();

    let output = render_sync_bundle_preflight_text(&document)
        .unwrap()
        .join("\n");

    assert!(output.contains("Sync bundle preflight summary"));
    assert!(output.contains("Resources: 1 total"));
    assert!(output.contains("Sync blocking:"));
    assert!(output.contains("Provider blocking:"));
}

#[test]
fn render_sync_bundle_preflight_rejects_wrong_kind() {
    let error = render_sync_bundle_preflight_text(&json!({"kind": "wrong"}))
        .unwrap_err()
        .to_string();

    assert!(error.contains("kind is not supported"));
}

#[test]
fn build_sync_bundle_preflight_document_reads_provider_metadata_from_source_bundle_document() {
    let source_bundle = build_sync_source_bundle_document(
        &[json!({
            "kind": "dashboard",
            "uid": "cpu-main",
            "title": "CPU Main",
            "body": {"datasourceUids": ["loki-main"]},
        })],
        &[json!({
            "kind": "datasource",
            "uid": "loki-main",
            "name": "Loki Main",
            "title": "Loki Main",
            "body": {"uid": "loki-main", "name": "Loki Main", "type": "loki"},
            "secureJsonDataProviders": {
                "httpHeaderValue1": "${provider:vault:secret/data/loki/token}"
            },
            "secureJsonDataPlaceholders": {
                "basicAuthPassword": "${secret:loki-basic-auth}"
            }
        })],
        &[],
        None,
        None,
    )
    .unwrap();
    let target_inventory = json!({"dashboards": [], "datasources": []});
    let availability = json!({
        "pluginIds": ["loki"],
        "datasourceUids": [],
        "datasourceNames": [],
        "contactPoints": [],
        "providerNames": ["vault"],
    });

    let document = build_sync_bundle_preflight_document(
        &source_bundle,
        &target_inventory,
        Some(&availability),
    )
    .unwrap();

    assert_eq!(document["summary"]["providerBlockingCount"], json!(0));
    assert_eq!(
        document["providerAssessment"]["plans"][0]["providers"][0]["providerName"],
        json!("vault")
    );
}
