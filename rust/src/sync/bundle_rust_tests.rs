//! Sync bundle preflight contract tests.
//! Validates sync bundle planning output and sync-source dependency assembly.
use super::SyncBundlePreflightSummary;
use super::SYNC_BUNDLE_PREFLIGHT_KIND;
use super::{build_sync_bundle_preflight_document, render_sync_bundle_preflight_text};
use crate::dashboard::CommonCliArgs;
use crate::sync::workbench::{build_sync_source_bundle_document, render_sync_source_bundle_text};
use crate::sync::{
    render_sync_apply_intent_text, run_sync_cli, SyncBundleArgs, SyncBundlePreflightArgs,
    SyncGroupCommand, SyncOutputFormat,
};
use serde_json::json;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn sync_common_args() -> CommonCliArgs {
    CommonCliArgs {
        url: "http://127.0.0.1:3000".to_string(),
        api_token: Some("test-token".to_string()),
        username: None,
        password: None,
        prompt_password: false,
        prompt_token: false,
        timeout: 30,
        verify_ssl: false,
    }
}

fn load_alert_export_contract_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/alert_export_contract_cases.json"
    ))
    .unwrap()
}

fn load_sync_source_bundle_contract_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/sync_source_bundle_contract_cases.json"
    ))
    .unwrap()
}

fn build_alert_replay_artifact_fixture(include_rules: bool) -> Value {
    let fixture = load_alert_export_contract_fixture();
    let mut summary = fixture["syncAlertingArtifacts"]["summary"].clone();
    let artifacts = fixture["syncAlertingArtifacts"]["artifacts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut alerting = serde_json::Map::new();
    if !include_rules {
        if let Some(summary_object) = summary.as_object_mut() {
            summary_object.insert("ruleCount".to_string(), json!(0));
        }
    }
    alerting.insert("summary".to_string(), summary);
    for artifact in artifacts {
        let section = artifact["section"].as_str().unwrap_or_default();
        if !include_rules && section == "rules" {
            continue;
        }
        let identity_field = artifact["identityField"].as_str().unwrap_or_default();
        let identity = artifact["identity"].clone();
        let source_path = artifact["sourcePath"].clone();
        let kind = match section {
            "rules" => "grafana-alert-rule",
            "contactPoints" => "grafana-contact-point",
            "muteTimings" => "grafana-mute-timing",
            "policies" => "grafana-notification-policies",
            "templates" => "grafana-notification-template",
            _ => "",
        };
        let mut spec = serde_json::Map::new();
        spec.insert(identity_field.to_string(), identity);
        let mut document = serde_json::Map::new();
        document.insert("kind".to_string(), json!(kind));
        document.insert("spec".to_string(), Value::Object(spec));
        let entry = json!({
            "sourcePath": source_path,
            "document": Value::Object(document)
        });
        alerting
            .entry(section.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        alerting
            .get_mut(section)
            .and_then(Value::as_array_mut)
            .unwrap()
            .push(entry);
    }
    Value::Object(alerting)
}

#[test]
fn build_sync_source_bundle_document_matches_cross_domain_summary_contract() {
    let fixture = load_sync_source_bundle_contract_fixture();
    let source_bundle = build_sync_source_bundle_document(
        &[
            json!({"kind": "dashboard", "uid": "cpu-main", "title": "CPU Main"}),
            json!({"kind": "dashboard", "uid": "logs-main", "title": "Logs Main"}),
        ],
        &[json!({
            "kind": "datasource",
            "uid": "prom-main",
            "name": "Prometheus Main",
            "title": "Prometheus Main"
        })],
        &[json!({
            "kind": "folder",
            "uid": "ops",
            "title": "Operations"
        })],
        &[json!({
            "kind": "alert",
            "uid": "cpu-high",
            "title": "CPU High",
            "managedFields": ["condition"],
            "body": {"condition": "A"}
        })],
        Some(&build_alert_replay_artifact_fixture(true)),
        Some(&json!({"bundleLabel": "sync-smoke"})),
    )
    .unwrap();

    assert_eq!(
        source_bundle["summary"],
        fixture["crossDomainSummaryCase"]["summary"]
    );
    assert_eq!(
        source_bundle["dashboards"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(
        source_bundle["datasources"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(source_bundle["folders"].as_array().map(Vec::len), Some(1));
    assert_eq!(source_bundle["alerts"].as_array().map(Vec::len), Some(1));

    let text = render_sync_source_bundle_text(&source_bundle).unwrap();
    assert_eq!(
        text,
        fixture["crossDomainSummaryCase"]["textLines"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect::<Vec<String>>()
    );
}

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
fn sync_bundle_preflight_summary_reads_counts_from_document() {
    let document = build_sync_bundle_preflight_document(
        &json!({
            "folders": [
                {"kind": "folder", "uid": "ops", "title": "Operations"}
            ]
        }),
        &json!({}),
        None,
    )
    .unwrap();

    let summary = SyncBundlePreflightSummary::from_document(&document).unwrap();

    assert_eq!(summary.resource_count, 1);
    assert_eq!(summary.sync_blocking_count, 0);
    assert_eq!(summary.provider_blocking_count, 0);
    assert_eq!(summary.alert_artifact_count, 0);
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
    assert!(output.contains("Alert artifacts: 0 total"));
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

#[test]
fn build_sync_source_bundle_document_keeps_normalized_alert_specs() {
    let source_bundle = build_sync_source_bundle_document(
        &[],
        &[],
        &[],
        &[json!({
            "kind": "alert",
            "uid": "cpu-high",
            "title": "CPU High",
            "managedFields": ["condition", "datasourceUids"],
            "body": {
                "condition": "A",
                "datasourceUids": ["prom-main"]
            },
            "sourcePath": "rules/infra/cpu-high.json"
        })],
        Some(&json!({"summary": {"ruleCount": 1}})),
        None,
    )
    .unwrap();

    assert_eq!(source_bundle["alerts"].as_array().unwrap().len(), 1);
    assert_eq!(source_bundle["alerts"][0]["uid"], json!("cpu-high"));
    assert_eq!(
        source_bundle["alerts"][0]["body"]["datasourceUids"][0],
        json!("prom-main")
    );
}

#[test]
fn build_sync_source_bundle_document_includes_alert_contract() {
    let source_bundle = build_sync_source_bundle_document(
        &[
            json!({
                "kind": "dashboard",
                "uid": "cpu-main",
                "title": "CPU Main",
                "body": {"datasourceUids": ["prom-main"]},
            }),
            json!({
                "kind": "dashboard",
                "uid": "logs-main",
                "title": "Logs Main",
                "body": {"datasourceUids": ["loki-main"]},
            }),
        ],
        &[json!({
            "kind": "datasource",
            "uid": "prom-main",
            "name": "Prometheus Main",
            "title": "Prometheus Main",
            "body": {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus"},
        })],
        &[],
        &[],
        Some(&json!({
            "rules": [{
                "sourcePath": "rules/infra/cpu-high.json",
                "document": {
                    "kind": "grafana-alert-rule",
                    "metadata": {
                        "uid": "cpu-high",
                        "title": "CPU High",
                    },
                    "spec": {
                        "uid": "cpu-high",
                        "title": "CPU High",
                    },
                },
            }],
            "contactPoints": [{"uid": "pagerduty-primary", "name": "PagerDuty Primary"}],
            "summary": {"ruleCount": 1, "contactPointCount": 1},
        })),
        None,
    )
    .unwrap();

    let contract = &source_bundle["alertContract"];
    assert_eq!(contract["kind"], json!("grafana-utils-sync-alert-contract"));
    assert_eq!(contract["summary"]["total"], json!(2));
    assert_eq!(contract["summary"]["safeForSync"], json!(2));
    assert_eq!(contract["resources"].as_array().unwrap().len(), 2);
}

#[test]
fn build_sync_source_bundle_document_preserves_full_alert_contract_sections() {
    let source_bundle = build_sync_source_bundle_document(
        &[],
        &[],
        &[],
        &[],
        Some(&build_alert_replay_artifact_fixture(true)),
        None,
    )
    .unwrap();

    let contract = &source_bundle["alertContract"];
    assert_eq!(
        contract["summary"],
        json!({
            "total": 5,
            "safeForSync": 2
        })
    );
    assert_eq!(
        contract["countsByKind"],
        json!([
            {"kind": "grafana-alert-rule", "count": 1},
            {"kind": "grafana-contact-point", "count": 1},
            {"kind": "grafana-mute-timing", "count": 1},
            {"kind": "grafana-notification-policies", "count": 1},
            {"kind": "grafana-notification-template", "count": 1}
        ])
    );
    assert_eq!(contract["resources"].as_array().unwrap().len(), 5);
    assert!(contract["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| {
            resource["kind"] == "grafana-contact-point" && resource["safeForSync"] == json!(true)
        }));
    assert!(contract["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| {
            resource["kind"] == "grafana-mute-timing" && resource["safeForSync"] == json!(false)
        }));
    assert!(contract["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| {
            resource["kind"] == "grafana-notification-policies"
                && resource["safeForSync"] == json!(false)
        }));
    assert!(contract["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| {
            resource["kind"] == "grafana-notification-template"
                && resource["safeForSync"] == json!(false)
        }));
}

#[test]
fn build_sync_bundle_preflight_document_falls_back_to_alerting_rule_documents() {
    let source_bundle = json!({
        "dashboards": [],
        "datasources": [],
        "folders": [],
        "alerting": {
            "rules": [
                {
                    "sourcePath": "rules/infra/cpu-high.json",
                    "document": {
                        "kind": "grafana-alert-rule",
                        "metadata": {
                            "uid": "cpu-high",
                            "title": "CPU High"
                        },
                        "spec": {
                            "uid": "cpu-high",
                            "title": "CPU High",
                            "folderUID": "infra",
                            "ruleGroup": "CPU Alerts",
                            "condition": "A",
                            "data": [
                                {
                                    "refId": "A",
                                    "datasourceUid": "prom-main",
                                    "datasourceName": "Prometheus Main"
                                }
                            ],
                            "notificationSettings": {
                                "receiver": "pagerduty-primary"
                            }
                        }
                    }
                }
            ]
        }
    });
    let availability = json!({
        "pluginIds": [],
        "datasourceUids": [],
        "datasourceNames": [],
        "contactPoints": [],
        "providerNames": []
    });

    let document =
        build_sync_bundle_preflight_document(&source_bundle, &json!({}), Some(&availability))
            .unwrap();

    assert_eq!(document["summary"]["resourceCount"], json!(1));
    assert!(document["syncPreflight"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-datasource"
            && item["identity"] == "cpu-high->prom-main"
            && item["status"] == "missing"));
    assert!(document["syncPreflight"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-contact-point"
            && item["identity"] == "cpu-high->pagerduty-primary"
            && item["status"] == "missing"));
}

#[test]
fn build_sync_bundle_preflight_document_reports_non_rule_alert_export_artifacts_from_source_bundle()
{
    let source_bundle = json!({
        "dashboards": [],
        "datasources": [],
        "folders": [],
        "alerting": build_alert_replay_artifact_fixture(false)
    });
    let availability = json!({
        "pluginIds": [],
        "datasourceUids": [],
        "datasourceNames": [],
        "contactPoints": [],
        "providerNames": []
    });

    let document =
        build_sync_bundle_preflight_document(&source_bundle, &json!({}), Some(&availability))
            .unwrap();

    assert_eq!(document["summary"]["resourceCount"], json!(0));
    assert_eq!(document["syncPreflight"]["checks"], json!([]));
}

#[test]
fn build_sync_source_bundle_document_preserves_alert_replay_artifact_summary_and_paths() {
    let fixture = load_alert_export_contract_fixture();
    let source_bundle = build_sync_source_bundle_document(
        &[],
        &[],
        &[],
        &[],
        Some(&build_alert_replay_artifact_fixture(true)),
        None,
    )
    .unwrap();

    let expected_summary = &fixture["syncAlertingArtifacts"]["summary"];
    assert_eq!(source_bundle["alerting"]["summary"], *expected_summary);
    for artifact in fixture["syncAlertingArtifacts"]["artifacts"]
        .as_array()
        .unwrap_or(&Vec::new())
    {
        let section = artifact["section"].as_str().unwrap_or_default();
        let identity_field = artifact["identityField"].as_str().unwrap_or_default();
        let identity = artifact["identity"].clone();
        let source_path = artifact["sourcePath"].clone();
        let empty_entries = Vec::new();
        let entries = source_bundle["alerting"][section]
            .as_array()
            .unwrap_or(&empty_entries);
        assert!(entries.iter().any(|entry| {
            entry["sourcePath"] == source_path
                && entry["document"]["spec"][identity_field] == identity
        }));
    }
}

#[test]
fn render_sync_source_bundle_text_reports_alert_replay_artifact_counts() {
    let fixture = load_alert_export_contract_fixture();
    let source_bundle = build_sync_source_bundle_document(
        &[],
        &[],
        &[],
        &[],
        Some(&build_alert_replay_artifact_fixture(true)),
        None,
    )
    .unwrap();

    let lines = render_sync_source_bundle_text(&source_bundle).unwrap();
    let expected = format!(
        "Alerting: rules={} contact-points={} mute-timings={} policies={} templates={}",
        fixture["syncAlertingArtifacts"]["summary"]["ruleCount"],
        fixture["syncAlertingArtifacts"]["summary"]["contactPointCount"],
        fixture["syncAlertingArtifacts"]["summary"]["muteTimingCount"],
        fixture["syncAlertingArtifacts"]["summary"]["policyCount"],
        fixture["syncAlertingArtifacts"]["summary"]["templateCount"]
    );
    assert!(lines.iter().any(|line| line == &expected));
}

#[test]
fn build_sync_bundle_preflight_document_reports_alert_replay_artifacts_and_keeps_sync_checks_zero()
{
    let alerting = build_alert_replay_artifact_fixture(false);
    let source_bundle = json!({
        "dashboards": [],
        "datasources": [],
        "folders": [],
        "alerting": alerting
    });
    let availability = json!({
        "pluginIds": [],
        "datasourceUids": [],
        "datasourceNames": [],
        "contactPoints": [],
        "providerNames": []
    });

    let document =
        build_sync_bundle_preflight_document(&source_bundle, &json!({}), Some(&availability))
            .unwrap();

    assert_eq!(document["summary"]["alertArtifactCount"], json!(4));
    assert_eq!(document["summary"]["alertArtifactPlanOnlyCount"], json!(1));
    assert_eq!(document["summary"]["alertArtifactBlockedCount"], json!(3));
    assert_eq!(
        document["alertArtifactAssessment"]["summary"]["contactPointCount"],
        json!(1)
    );
    assert_eq!(
        document["alertArtifactAssessment"]["summary"]["muteTimingCount"],
        json!(1)
    );
    assert_eq!(
        document["alertArtifactAssessment"]["summary"]["policyCount"],
        json!(1)
    );
    assert_eq!(
        document["alertArtifactAssessment"]["summary"]["templateCount"],
        json!(1)
    );
    assert!(document["alertArtifactAssessment"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-contact-point"
            && item["identity"] == "smoke-webhook"
            && item["status"] == "plan-only"));
    assert!(document["alertArtifactAssessment"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-mute-timing"
            && item["identity"] == "Off Hours"
            && item["status"] == "blocked"));
    assert!(document["alertArtifactAssessment"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-policy"
            && item["identity"] == "grafana-default-email"
            && item["status"] == "blocked"));
    assert!(document["alertArtifactAssessment"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-template"
            && item["identity"] == "slack.default"
            && item["status"] == "blocked"));
    assert_eq!(document["summary"]["syncBlockingCount"], json!(0));
    assert_eq!(document["syncPreflight"]["checks"], json!([]));
}

#[test]
fn build_sync_bundle_preflight_document_counts_top_level_alert_specs_from_source_bundle() {
    let source_bundle = build_sync_source_bundle_document(
        &[json!({
            "kind": "dashboard",
            "uid": "cpu-main",
            "title": "CPU Main",
            "body": {"datasourceUids": ["prom-main"]},
        })],
        &[json!({
            "kind": "datasource",
            "uid": "prom-main",
            "name": "Prometheus Main",
            "title": "Prometheus Main",
            "body": {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus"},
        })],
        &[],
        &[json!({
            "kind": "alert",
            "uid": "cpu-high",
            "title": "CPU High",
            "managedFields": ["condition", "contactPoints", "datasourceUids"],
            "body": {
                "condition": "A",
                "contactPoints": ["pagerduty-primary"],
                "datasourceUids": ["prom-main"]
            }
        })],
        Some(&json!({
            "rules": [{
                "sourcePath": "rules/cpu-high.json",
                "document": {
                    "groups": [{
                        "name": "CPU Alerts",
                        "rules": [{"uid": "cpu-high", "title": "CPU High"}]
                    }]
                }
            }],
            "summary": {"ruleCount": 1}
        })),
        None,
    )
    .unwrap();
    let target_inventory = json!({"dashboards": [], "datasources": []});
    let availability = json!({
        "pluginIds": ["prometheus"],
        "datasourceUids": ["prom-main"],
        "datasourceNames": [],
        "contactPoints": ["pagerduty-primary"],
        "providerNames": []
    });

    let document = build_sync_bundle_preflight_document(
        &source_bundle,
        &target_inventory,
        Some(&availability),
    )
    .unwrap();

    assert_eq!(document["summary"]["resourceCount"], json!(3));
    assert!(document["syncPreflight"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-datasource"
            && item["identity"] == "cpu-high->prom-main"
            && item["status"] == "ok"));
    assert!(document["syncPreflight"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-contact-point"
            && item["identity"] == "cpu-high->pagerduty-primary"
            && item["status"] == "ok"));
}

#[test]
fn run_sync_cli_bundle_writes_source_bundle_artifact() {
    let temp = tempdir().unwrap();
    let dashboard_export_dir = temp.path().join("dashboards").join("raw");
    let alert_export_dir = temp.path().join("alerts").join("raw");
    fs::create_dir_all(&dashboard_export_dir).unwrap();
    fs::create_dir_all(alert_export_dir.join("rules")).unwrap();
    fs::write(
        dashboard_export_dir.join("cpu.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "cpu-main",
                "title": "CPU Main",
                "panels": []
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        dashboard_export_dir.join("folders.json"),
        serde_json::to_string_pretty(&json!([
            {"uid": "ops", "title": "Operations", "path": "Operations"}
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        dashboard_export_dir.join("datasources.json"),
        serde_json::to_string_pretty(&json!([
            {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus"}
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        alert_export_dir.join("rules").join("cpu-high.json"),
        serde_json::to_string_pretty(&json!({
            "groups": [{
                "name": "CPU Alerts",
                "folderUid": "general",
                "rules": [{
                    "uid": "cpu-high",
                    "title": "CPU High",
                    "condition": "A",
                    "data": [{
                        "refId": "A",
                        "datasourceUid": "prom-main",
                        "model": {
                            "expr": "up",
                            "refId": "A"
                        }
                    }],
                    "for": "5m",
                    "noDataState": "NoData",
                    "execErrState": "Alerting",
                    "annotations": {
                        "__dashboardUid__": "cpu-main",
                        "__panelId__": "1"
                    },
                    "notification_settings": {
                        "receiver": "pagerduty-primary"
                    }
                }]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let metadata_file = temp.path().join("metadata.json");
    fs::write(
        &metadata_file,
        serde_json::to_string_pretty(&json!({
            "bundleLabel": "smoke-bundle"
        }))
        .unwrap(),
    )
    .unwrap();
    let output_file = temp.path().join("bundle.json");

    let result = run_sync_cli(SyncGroupCommand::Bundle(SyncBundleArgs {
        dashboard_export_dir: Some(dashboard_export_dir.clone()),
        alert_export_dir: Some(alert_export_dir.clone()),
        datasource_export_file: None,
        metadata_file: Some(metadata_file.clone()),
        output_file: Some(output_file.clone()),
        output: SyncOutputFormat::Json,
    }));

    assert!(result.is_ok());
    let bundle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_file).unwrap()).unwrap();
    assert_eq!(bundle["kind"], json!("grafana-utils-sync-source-bundle"));
    assert_eq!(bundle["summary"]["dashboardCount"], json!(1));
    assert_eq!(bundle["summary"]["datasourceCount"], json!(1));
    assert_eq!(bundle["summary"]["folderCount"], json!(1));
    assert_eq!(bundle["summary"]["alertRuleCount"], json!(1));
    assert_eq!(bundle["alerts"].as_array().unwrap().len(), 1);
    assert_eq!(bundle["alerts"][0]["kind"], json!("alert"));
    assert_eq!(bundle["alerts"][0]["uid"], json!("cpu-high"));
    assert_eq!(bundle["alerts"][0]["title"], json!("CPU High"));
    assert_eq!(
        bundle["alerts"][0]["managedFields"],
        json!([
            "condition",
            "annotations",
            "contactPoints",
            "datasourceUids",
            "data"
        ])
    );
    assert_eq!(bundle["alerts"][0]["body"]["condition"], json!("A"));
    assert_eq!(
        bundle["alerts"][0]["body"]["contactPoints"],
        json!(["pagerduty-primary"])
    );
    assert_eq!(
        bundle["alerts"][0]["body"]["datasourceUids"],
        json!(["prom-main"])
    );
    assert_eq!(
        bundle["alerts"][0]["body"]["annotations"]["__dashboardUid__"],
        json!("cpu-main")
    );
    assert_eq!(bundle["metadata"]["bundleLabel"], json!("smoke-bundle"));
    assert_eq!(
        bundle["metadata"]["dashboardExportDir"],
        json!(dashboard_export_dir.display().to_string())
    );
    assert_eq!(
        bundle["alerting"]["exportDir"],
        json!(alert_export_dir.display().to_string())
    );
    assert_eq!(bundle["alerting"]["summary"]["ruleCount"], json!(1));
    assert_eq!(bundle["alerting"]["summary"]["contactPointCount"], json!(0));
    assert_eq!(bundle["alerting"]["summary"]["policyCount"], json!(0));
    assert_eq!(
        bundle["metadata"]["alertExportDir"],
        json!(alert_export_dir.display().to_string())
    );
}

#[test]
fn run_sync_cli_bundle_preserves_alert_export_artifact_metadata() {
    let temp = tempdir().unwrap();
    let alert_export_dir = temp.path().join("alerts").join("raw");
    fs::create_dir_all(
        alert_export_dir
            .join("contact-points")
            .join("Smoke_Webhook"),
    )
    .unwrap();
    fs::create_dir_all(alert_export_dir.join("mute-timings")).unwrap();
    fs::create_dir_all(alert_export_dir.join("policies")).unwrap();
    fs::create_dir_all(alert_export_dir.join("templates")).unwrap();
    fs::write(
        alert_export_dir
            .join("contact-points")
            .join("Smoke_Webhook")
            .join("Smoke_Webhook__smoke-webhook.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-contact-point",
            "apiVersion": 1,
            "schemaVersion": 1,
            "spec": {
                "uid": "smoke-webhook",
                "name": "Smoke Webhook",
                "type": "webhook",
                "settings": {"url": "http://127.0.0.1/notify"}
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        alert_export_dir.join("contact-points").join("index.json"),
        serde_json::to_string_pretty(&json!([
            {
                "kind": "grafana-contact-point",
                "uid": "smoke-webhook",
                "name": "Smoke Webhook",
                "type": "webhook",
                "path": "contact-points/Smoke_Webhook/Smoke_Webhook__smoke-webhook.json"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        alert_export_dir.join("mute-timings").join("Off_Hours.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-mute-timing",
            "apiVersion": 1,
            "schemaVersion": 1,
            "spec": {
                "name": "Off Hours",
                "time_intervals": [{
                    "times": [{
                        "start_time": "00:00",
                        "end_time": "06:00"
                    }]
                }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        alert_export_dir
            .join("policies")
            .join("notification-policies.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-notification-policies",
            "apiVersion": 1,
            "schemaVersion": 1,
            "spec": {"receiver": "grafana-default-email"}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        alert_export_dir
            .join("templates")
            .join("slack.default.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-notification-template",
            "apiVersion": 1,
            "schemaVersion": 1,
            "spec": {
                "name": "slack.default",
                "template": "{{ define \"slack.default\" }}ok{{ end }}"
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        alert_export_dir.join("export-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-util-alert-export-index",
            "apiVersion": 1,
            "schemaVersion": 1
        }))
        .unwrap(),
    )
    .unwrap();
    let output_file = temp.path().join("bundle.json");

    let result = run_sync_cli(SyncGroupCommand::Bundle(SyncBundleArgs {
        dashboard_export_dir: None,
        alert_export_dir: Some(alert_export_dir.clone()),
        datasource_export_file: None,
        metadata_file: None,
        output_file: Some(output_file.clone()),
        output: SyncOutputFormat::Json,
    }));

    assert!(result.is_ok());
    let bundle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_file).unwrap()).unwrap();
    assert_eq!(bundle["alerting"]["summary"]["contactPointCount"], json!(1));
    assert_eq!(bundle["alerting"]["summary"]["muteTimingCount"], json!(1));
    assert_eq!(bundle["alerting"]["summary"]["policyCount"], json!(1));
    assert_eq!(bundle["alerting"]["summary"]["templateCount"], json!(1));
    assert_eq!(bundle["alerts"].as_array().unwrap().len(), 4);
    assert!(bundle["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-contact-point" && item["uid"] == "smoke-webhook"));
    assert!(bundle["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-mute-timing" && item["name"] == "Off Hours"));
    assert!(bundle["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-policy" && item["title"] == "grafana-default-email"));
    assert!(bundle["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "alert-template" && item["name"] == "slack.default"));
    assert_eq!(
        bundle["alerting"]["exportMetadata"]["kind"],
        json!("grafana-util-alert-export-index")
    );
    assert_eq!(
        bundle["alerting"]["muteTimings"][0]["sourcePath"],
        json!("mute-timings/Off_Hours.json")
    );
    assert_eq!(
        bundle["alerting"]["contactPoints"][0]["sourcePath"],
        json!("contact-points/Smoke_Webhook/Smoke_Webhook__smoke-webhook.json")
    );
    assert_eq!(
        bundle["alerting"]["policies"][0]["sourcePath"],
        json!("policies/notification-policies.json")
    );
    assert_eq!(
        bundle["alerting"]["templates"][0]["sourcePath"],
        json!("templates/slack.default.json")
    );
}

#[test]
fn run_sync_cli_bundle_ignores_dashboard_permissions_bundle() {
    let temp = tempdir().unwrap();
    let dashboard_export_dir = temp.path().join("dashboards").join("raw");
    fs::create_dir_all(&dashboard_export_dir).unwrap();
    fs::write(
        dashboard_export_dir.join("cpu.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "cpu-main",
                "title": "CPU Main",
                "panels": []
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        dashboard_export_dir.join("permissions.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "dashboard-permissions",
            "permissions": [{
                "uid": "cpu-main",
                "role": "Viewer"
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let output_file = temp.path().join("bundle.json");

    let result = run_sync_cli(SyncGroupCommand::Bundle(SyncBundleArgs {
        dashboard_export_dir: Some(dashboard_export_dir.clone()),
        alert_export_dir: None,
        datasource_export_file: None,
        metadata_file: None,
        output_file: Some(output_file.clone()),
        output: SyncOutputFormat::Json,
    }));

    assert!(result.is_ok(), "{result:?}");
    let bundle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_file).unwrap()).unwrap();
    assert_eq!(bundle["summary"]["dashboardCount"], json!(1));
    assert_eq!(bundle["dashboards"].as_array().unwrap().len(), 1);
    assert_eq!(bundle["dashboards"][0]["uid"], json!("cpu-main"));
}

#[test]
fn run_sync_cli_bundle_preserves_datasource_provider_metadata_from_inventory_file() {
    let temp = tempdir().unwrap();
    let datasource_export_file = temp.path().join("datasources.json");
    fs::write(
        &datasource_export_file,
        serde_json::to_string_pretty(&json!([
            {
                "uid": "loki-main",
                "name": "Loki Main",
                "type": "loki",
                "secureJsonDataProviders": {
                    "httpHeaderValue1": "${provider:vault:secret/data/loki/token}"
                },
                "secureJsonDataPlaceholders": {
                    "basicAuthPassword": "${secret:loki-basic-auth}"
                }
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    let output_file = temp.path().join("bundle.json");

    let result = run_sync_cli(SyncGroupCommand::Bundle(SyncBundleArgs {
        dashboard_export_dir: None,
        alert_export_dir: None,
        datasource_export_file: Some(datasource_export_file.clone()),
        metadata_file: None,
        output_file: Some(output_file.clone()),
        output: SyncOutputFormat::Json,
    }));

    assert!(result.is_ok());
    let bundle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_file).unwrap()).unwrap();
    assert_eq!(bundle["summary"]["datasourceCount"], json!(1));
    assert_eq!(
        bundle["metadata"]["datasourceExportFile"],
        json!(datasource_export_file.display().to_string())
    );
    assert_eq!(
        bundle["datasources"][0]["secureJsonDataProviders"]["httpHeaderValue1"],
        json!("${provider:vault:secret/data/loki/token}")
    );
    assert_eq!(
        bundle["datasources"][0]["secureJsonDataPlaceholders"]["basicAuthPassword"],
        json!("${secret:loki-basic-auth}")
    );
}

#[test]
fn run_sync_cli_bundle_normalizes_tool_rule_export_into_top_level_alert_spec() {
    let temp = tempdir().unwrap();
    let alert_export_dir = temp.path().join("alerts").join("raw");
    fs::create_dir_all(alert_export_dir.join("rules")).unwrap();
    fs::write(
        alert_export_dir.join("rules").join("cpu-high.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "apiVersion": 1,
            "kind": "grafana-alert-rule",
            "metadata": {
                "uid": "cpu-high",
                "title": "CPU High",
                "folderUID": "general",
                "ruleGroup": "CPU Alerts"
            },
            "spec": {
                "uid": "cpu-high",
                "title": "CPU High",
                "folderUID": "general",
                "ruleGroup": "CPU Alerts",
                "condition": "A",
                "data": [{
                    "refId": "A",
                    "datasourceUid": "prom-main",
                    "model": {
                        "datasource": {
                            "uid": "prom-main",
                            "name": "Prometheus Main",
                            "type": "prometheus"
                        },
                        "expr": "up",
                        "refId": "A"
                    }
                }],
                "notificationSettings": {
                    "receiver": "pagerduty-primary"
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let output_file = temp.path().join("bundle.json");

    let result = run_sync_cli(SyncGroupCommand::Bundle(SyncBundleArgs {
        dashboard_export_dir: None,
        alert_export_dir: Some(alert_export_dir.clone()),
        datasource_export_file: None,
        metadata_file: None,
        output_file: Some(output_file.clone()),
        output: SyncOutputFormat::Json,
    }));

    assert!(result.is_ok());
    let bundle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_file).unwrap()).unwrap();
    assert_eq!(bundle["summary"]["alertRuleCount"], json!(1));
    assert_eq!(bundle["alerts"].as_array().unwrap().len(), 1);
    assert_eq!(
        bundle["alerts"][0]["managedFields"],
        json!([
            "condition",
            "contactPoints",
            "datasourceUids",
            "datasourceNames",
            "pluginIds",
            "data"
        ])
    );
    assert_eq!(
        bundle["alerts"][0]["body"]["contactPoints"],
        json!(["pagerduty-primary"])
    );
    assert_eq!(
        bundle["alerts"][0]["body"]["datasourceNames"],
        json!(["Prometheus Main"])
    );
    assert_eq!(
        bundle["alerts"][0]["body"]["pluginIds"],
        json!(["prometheus"])
    );
    assert_eq!(
        bundle["metadata"]["alertExportDir"],
        json!(alert_export_dir.display().to_string())
    );
}

#[test]
fn run_sync_cli_bundle_preflight_accepts_local_bundle_inputs() {
    let temp = tempdir().unwrap();
    let source_bundle = temp.path().join("source.json");
    let target_inventory = temp.path().join("target.json");
    fs::write(
        &source_bundle,
        serde_json::to_string_pretty(&json!({
            "dashboards": [],
            "datasources": [],
            "folders": [{"kind":"folder","uid":"ops","title":"Operations"}],
            "alerts": []
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &target_inventory,
        serde_json::to_string_pretty(&json!({})).unwrap(),
    )
    .unwrap();

    let result = run_sync_cli(SyncGroupCommand::BundlePreflight(SyncBundlePreflightArgs {
        source_bundle,
        target_inventory,
        availability_file: None,
        fetch_live: false,
        common: sync_common_args(),
        org_id: None,
        output: SyncOutputFormat::Json,
    }));

    assert!(result.is_ok());
}

#[test]
fn render_sync_apply_intent_text_includes_alert_artifact_bundle_counts() {
    let lines = render_sync_apply_intent_text(&json!({
        "kind": "grafana-utils-sync-apply-intent",
        "stage": "apply",
        "stepIndex": 3,
        "traceId": "sync-trace-demo",
        "parentTraceId": "sync-trace-demo",
        "mode": "apply",
        "reviewed": true,
        "reviewRequired": true,
        "allowPrune": false,
        "approved": true,
        "summary": {
            "would_create": 1,
            "would_update": 0,
            "would_delete": 0,
            "noop": 0,
            "unmanaged": 0,
            "alert_candidate": 0,
            "alert_plan_only": 0,
            "alert_blocked": 0
        },
        "operations": [],
        "bundlePreflightSummary": {
            "resourceCount": 4,
            "syncBlockingCount": 0,
            "providerBlockingCount": 0,
            "alertArtifactCount": 4,
            "alertArtifactPlanOnlyCount": 1,
            "alertArtifactBlockingCount": 3
        }
    }))
    .unwrap();

    let output = lines.join("\n");
    assert!(output.contains("alert-artifacts=4"));
    assert!(output.contains("plan-only=1"));
    assert!(output.contains("blocking=3"));
}
