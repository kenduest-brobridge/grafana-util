//! Dashboard governance output contract regressions.
use super::super::test_support;
use super::super::{
    render_dashboard_governance_gate_result, GovernanceGateArgs, GovernanceGateOutputFormat,
};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn read_json_output_file(path: &Path) -> Value {
    let raw = fs::read_to_string(path).unwrap();
    assert!(
        raw.ends_with('\n'),
        "expected output file {} to end with a newline",
        path.display()
    );
    serde_json::from_str(&raw).unwrap()
}

#[test]
fn render_dashboard_governance_gate_result_lists_violations_and_warnings() {
    let result = test_support::DashboardGovernanceGateResult {
        ok: false,
        summary: test_support::DashboardGovernanceGateSummary {
            dashboard_count: 1,
            query_record_count: 2,
            violation_count: 1,
            warning_count: 1,
            checked_rules: json!({
                "datasourceAllowedFamilies": [],
                "datasourceAllowedUids": [],
                "allowedFolderPrefixes": [],
                "forbidUnknown": false,
                "forbidMixedFamilies": false,
                "forbidSelectStar": false,
                "requireSqlTimeFilter": false,
                "forbidBroadLokiRegex": false,
                "forbidBroadPrometheusSelectors": false,
                "forbidRegexHeavyPrometheus": false,
                "forbidHighCardinalityRegex": false,
                "maxPrometheusRangeWindowSeconds": null,
                "maxPrometheusAggregationDepth": null,
                "maxPrometheusCostScore": null,
                "forbidUnscopedLokiSearch": false,
                "maxPanelsPerDashboard": null,
                "minRefreshIntervalSeconds": null,
                "maxAuditScore": null,
                "maxReasonCount": null,
                "blockReasons": [],
                "maxDashboardLoadScore": null,
                "maxQueryComplexityScore": null,
                "maxDashboardComplexityScore": null,
                "maxQueriesPerDashboard": 1,
                "maxQueriesPerPanel": null,
                "failOnWarnings": false
            }),
        },
        violations: vec![test_support::DashboardGovernanceGateFinding {
            severity: "error".to_string(),
            code: "max-queries-per-dashboard".to_string(),
            message: "Dashboard query count 2 exceeds policy maxQueriesPerDashboard=1.".to_string(),
            dashboard_uid: "core-main".to_string(),
            dashboard_title: "Core Main".to_string(),
            panel_id: String::new(),
            panel_title: String::new(),
            ref_id: String::new(),
            datasource: String::new(),
            datasource_uid: String::new(),
            datasource_family: String::new(),
            risk_kind: String::new(),
        }],
        warnings: vec![test_support::DashboardGovernanceGateFinding {
            severity: "warning".to_string(),
            code: "broad-loki-selector".to_string(),
            message: "Narrow the Loki stream selector.".to_string(),
            dashboard_uid: "core-main".to_string(),
            dashboard_title: String::new(),
            panel_id: "7".to_string(),
            panel_title: String::new(),
            ref_id: String::new(),
            datasource: "Logs Main".to_string(),
            datasource_uid: String::new(),
            datasource_family: String::new(),
            risk_kind: "broad-loki-selector".to_string(),
        }],
    };

    let output = render_dashboard_governance_gate_result(&result);
    assert!(output.contains("Dashboard governance gate: FAIL"));
    assert!(output.contains("Violations:"));
    assert!(output.contains("Warnings:"));
    assert!(output.contains("max-queries-per-dashboard"));
    assert!(output.contains("broad-loki-selector"));
}

#[test]
fn run_dashboard_governance_gate_writes_json_output_file() {
    let temp = tempdir().unwrap();
    let policy_path = temp.path().join("policy.json");
    let governance_path = temp.path().join("governance.json");
    let queries_path = temp.path().join("queries.json");
    let json_output = temp.path().join("governance-check.json");

    fs::write(
        &policy_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "datasources": {
                "allowedFamilies": [],
                "allowedUids": []
            },
            "queries": {
                "maxQueriesPerDashboard": 4,
                "maxQueriesPerPanel": 2
            },
            "enforcement": {
                "failOnWarnings": false
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &governance_path,
        serde_json::to_string_pretty(&json!({
            "summary": {
                "dashboardCount": 1,
                "queryRecordCount": 2
            },
            "riskRecords": []
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &queries_path,
        serde_json::to_string_pretty(&json!({
            "summary": {
                "dashboardCount": 1,
                "queryRecordCount": 2
            },
            "queries": [
                {
                    "dashboardUid": "core-main",
                    "dashboardTitle": "Core Main",
                    "panelId": "7",
                    "panelTitle": "Errors",
                    "refId": "A"
                },
                {
                    "dashboardUid": "core-main",
                    "dashboardTitle": "Core Main",
                    "panelId": "8",
                    "panelTitle": "Latency",
                    "refId": "B"
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let args = GovernanceGateArgs {
        policy: policy_path,
        governance: governance_path,
        queries: queries_path,
        output_format: GovernanceGateOutputFormat::Json,
        json_output: Some(json_output.clone()),
        interactive: false,
    };

    test_support::run_dashboard_governance_gate(&args).unwrap();
    let output = read_json_output_file(&json_output);
    assert_eq!(output["ok"], json!(true));
    assert_eq!(output["summary"]["violationCount"], json!(0));
    assert_eq!(output["summary"]["warningCount"], json!(0));
    assert_eq!(
        output["summary"]["checkedRules"],
        json!({
            "datasourceAllowedFamilies": [],
            "datasourceAllowedUids": [],
            "allowedFolderPrefixes": [],
            "forbidUnknown": false,
            "forbidMixedFamilies": false,
            "forbidSelectStar": false,
            "requireSqlTimeFilter": false,
            "forbidBroadLokiRegex": false,
            "forbidBroadPrometheusSelectors": false,
            "forbidRegexHeavyPrometheus": false,
            "forbidHighCardinalityRegex": false,
            "maxPrometheusRangeWindowSeconds": null,
            "maxPrometheusAggregationDepth": null,
            "maxPrometheusCostScore": null,
            "forbidUnscopedLokiSearch": false,
            "maxPanelsPerDashboard": null,
            "minRefreshIntervalSeconds": null,
            "maxAuditScore": null,
            "maxReasonCount": null,
            "blockReasons": [],
            "maxDashboardLoadScore": null,
            "maxQueryComplexityScore": null,
            "maxDashboardComplexityScore": null,
            "maxQueriesPerDashboard": 4,
            "maxQueriesPerPanel": 2,
            "failOnWarnings": false
        })
    );
    assert_eq!(output["violations"], json!([]));
    assert_eq!(output["warnings"], json!([]));
}
