//! Dashboard governance gate evaluator.
//! Direct live/local review is the common path; governance-json and query-report artifacts stay available for advanced reuse.
use serde::Serialize;
use serde_json::Value;
#[cfg(test)]
use std::path::Path;

use crate::common::{message, Result};

use super::governance_gate_rules as rules;

#[cfg(any(feature = "tui", test))]
mod items;
#[cfg(any(feature = "tui", test))]
pub(crate) use items::{build_browser_item, finding_sort_key};

mod runner;
pub(crate) use runner::run_dashboard_governance_gate;

#[cfg(feature = "tui")]
pub(crate) mod tui;
#[cfg(test)]
pub(crate) use tui::{build_governance_gate_tui_groups, build_governance_gate_tui_items};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DashboardGovernanceGateSummary {
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "queryRecordCount")]
    pub(crate) query_record_count: usize,
    #[serde(rename = "violationCount")]
    pub(crate) violation_count: usize,
    #[serde(rename = "warningCount")]
    pub(crate) warning_count: usize,
    #[serde(rename = "checkedRules")]
    pub(crate) checked_rules: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DashboardGovernanceGateFinding {
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) message: String,
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "dashboardTitle")]
    pub(crate) dashboard_title: String,
    #[serde(rename = "panelId")]
    pub(crate) panel_id: String,
    #[serde(rename = "panelTitle")]
    pub(crate) panel_title: String,
    #[serde(rename = "refId")]
    pub(crate) ref_id: String,
    pub(crate) datasource: String,
    #[serde(rename = "datasourceUid")]
    pub(crate) datasource_uid: String,
    #[serde(rename = "datasourceFamily")]
    pub(crate) datasource_family: String,
    #[serde(rename = "riskKind", skip_serializing_if = "String::is_empty")]
    pub(crate) risk_kind: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DashboardGovernanceGateResult {
    pub(crate) ok: bool,
    pub(crate) summary: DashboardGovernanceGateSummary,
    pub(crate) violations: Vec<DashboardGovernanceGateFinding>,
    pub(crate) warnings: Vec<DashboardGovernanceGateFinding>,
}

pub(crate) fn evaluate_dashboard_governance_gate(
    policy: &Value,
    governance_document: &Value,
    query_document: &Value,
) -> Result<DashboardGovernanceGateResult> {
    let policy = rules::parse_query_threshold_policy(policy)?;
    let queries = query_document
        .get("queries")
        .and_then(Value::as_array)
        .ok_or_else(|| message("Dashboard query report JSON must contain a queries array."))?;
    let dashboard_count = query_document
        .get("summary")
        .and_then(|summary| summary.get("dashboardCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let query_record_count = query_document
        .get("summary")
        .and_then(|summary| summary.get("queryRecordCount"))
        .or_else(|| {
            query_document
                .get("summary")
                .and_then(|summary| summary.get("reportRowCount"))
        })
        .and_then(Value::as_u64)
        .unwrap_or(queries.len() as u64) as usize;

    let violations = rules::evaluate_dashboard_governance_gate_violations(
        &policy,
        governance_document,
        queries,
    )?;
    let warnings = rules::build_governance_warning_findings(governance_document)?;

    let ok = violations.is_empty() && (!policy.fail_on_warnings || warnings.is_empty());
    Ok(DashboardGovernanceGateResult {
        ok,
        summary: DashboardGovernanceGateSummary {
            dashboard_count,
            query_record_count,
            violation_count: violations.len(),
            warning_count: warnings.len(),
            checked_rules: rules::build_checked_rules(&policy),
        },
        violations,
        warnings,
    })
}

pub(crate) fn render_dashboard_governance_gate_result(
    result: &DashboardGovernanceGateResult,
) -> String {
    let mut lines = vec![
        format!(
            "Dashboard governance gate: {}",
            if result.ok { "PASS" } else { "FAIL" }
        ),
        format!(
            "Dashboards: {}  Queries: {}  Violations: {}  Warnings: {}",
            result.summary.dashboard_count,
            result.summary.query_record_count,
            result.summary.violation_count,
            result.summary.warning_count
        ),
    ];
    if !result.violations.is_empty() {
        lines.push(String::new());
        lines.push("Violations:".to_string());
        for record in &result.violations {
            lines.push(format!(
                "  ERROR [{}] dashboard={} panel={} datasource={}: {}",
                record.code,
                if record.dashboard_uid.is_empty() {
                    "-"
                } else {
                    &record.dashboard_uid
                },
                if record.panel_id.is_empty() {
                    "-"
                } else {
                    &record.panel_id
                },
                if record.datasource_uid.is_empty() {
                    "-"
                } else {
                    &record.datasource_uid
                },
                record.message
            ));
        }
    }
    if !result.warnings.is_empty() {
        lines.push(String::new());
        lines.push("Warnings:".to_string());
        for record in &result.warnings {
            lines.push(format!(
                "  WARN [{}] dashboard={} panel={} datasource={}: {}",
                if record.risk_kind.is_empty() {
                    &record.code
                } else {
                    &record.risk_kind
                },
                if record.dashboard_uid.is_empty() {
                    "-"
                } else {
                    &record.dashboard_uid
                },
                if record.panel_id.is_empty() {
                    "-"
                } else {
                    &record.panel_id
                },
                if record.datasource.is_empty() {
                    "-"
                } else {
                    &record.datasource
                },
                record.message
            ));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::CliColorChoice;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    use super::super::cli_defs::{CommonCliArgs, InspectExportInputType};
    use super::super::governance_policy::built_in_governance_policy;
    use super::super::review_source::{
        resolve_dashboard_review_artifacts, DashboardReviewSourceArgs,
    };

    fn make_common_args() -> CommonCliArgs {
        CommonCliArgs {
            color: CliColorChoice::Never,
            profile: None,
            url: "http://127.0.0.1:3000".to_string(),
            api_token: None,
            username: None,
            password: None,
            prompt_password: false,
            prompt_token: false,
            timeout: 30,
            verify_ssl: false,
        }
    }

    fn write_basic_git_sync_raw_export(raw_dir: &Path) {
        fs::create_dir_all(raw_dir).unwrap();
        fs::write(
            raw_dir.join("export-metadata.json"),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": 1,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid",
                "foldersFile": "folders.json",
                "datasourcesFile": "datasources.json",
                "org": "Main Org",
                "orgId": "1"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("folders.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": "general",
                    "title": "General",
                    "path": "General",
                    "org": "Main Org",
                    "orgId": "1"
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("datasources.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": "prom-main",
                    "name": "Prometheus Main",
                    "type": "prometheus",
                    "access": "proxy",
                    "url": "http://grafana.example.internal",
                    "isDefault": "true",
                    "org": "Main Org",
                    "orgId": "1"
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "path": "dash.json",
                    "format": "grafana-web-import-preserve-uid",
                    "org": "Main Org",
                    "orgId": "1"
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("dash.json"),
            serde_json::to_string_pretty(&json!({
                "dashboard": {
                    "id": null,
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "schemaVersion": 38,
                    "templating": {
                        "list": []
                    },
                    "panels": [{
                        "id": 7,
                        "title": "CPU",
                        "type": "timeseries",
                        "datasource": {"uid": "prom-main", "type": "prometheus"},
                        "targets": [{
                            "refId": "A",
                            "expr": "sum(rate(cpu_seconds_total[5m]))"
                        }]
                    }]
                },
                "meta": {
                    "folderUid": "general",
                    "folderTitle": "General"
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn evaluate_dashboard_governance_gate_supports_git_sync_repo_layout() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path();
        fs::create_dir_all(repo_root.join(".git")).unwrap();
        let raw_dir = repo_root.join("dashboards/git-sync/raw/org_1/raw");
        write_basic_git_sync_raw_export(&raw_dir);

        let artifacts = resolve_dashboard_review_artifacts(&DashboardReviewSourceArgs {
            common: &make_common_args(),
            page_size: 100,
            org_id: None,
            all_orgs: false,
            input_dir: Some(repo_root),
            input_format: crate::dashboard::DashboardImportInputFormat::Raw,
            input_type: Some(InspectExportInputType::Raw),
            governance: None,
            queries: None,
            require_queries: true,
        })
        .unwrap();

        let result = evaluate_dashboard_governance_gate(
            &built_in_governance_policy(),
            &artifacts.governance,
            &artifacts.queries,
        )
        .unwrap();

        assert!(result.ok);
        assert_eq!(result.summary.dashboard_count, 1);
        assert_eq!(result.summary.query_record_count, 1);
    }
}
