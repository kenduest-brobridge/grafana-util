use super::build_dashboard_domain_status;
use crate::project_status::{status_finding, PROJECT_STATUS_BLOCKED};
use serde_json::json;

#[test]
fn build_dashboard_domain_status_uses_detail_rows_for_blockers_when_available() {
    let document = json!({
        "summary": {
            "dashboardCount": 2,
            "queryCount": 5,
            "orphanedDatasourceCount": 2,
            "mixedDatasourceDashboardCount": 2,
        },
        "orphanedDatasources": [
            {
                "uid": "unused-main",
                "name": "Unused Main",
            },
            {
                "uid": "unused-ops",
                "name": "Unused Ops",
            }
        ],
        "mixedDatasourceDashboards": [
            {
                "uid": "cpu-main",
                "title": "CPU Main",
            },
            {
                "uid": "logs-main",
                "title": "Logs Main",
            }
        ]
    });

    let domain = build_dashboard_domain_status(Some(&document)).unwrap();

    assert_eq!(domain.status, PROJECT_STATUS_BLOCKED);
    assert_eq!(domain.reason_code, "blocked-by-blockers");
    assert_eq!(domain.primary_count, 5);
    assert_eq!(domain.blocker_count, 4);
    assert_eq!(domain.warning_count, 0);
    assert_eq!(
        domain.signal_keys,
        vec![
            "summary.dashboardCount".to_string(),
            "summary.queryCount".to_string(),
            "summary.orphanedDatasourceCount".to_string(),
            "summary.mixedDatasourceDashboardCount".to_string(),
            "orphanedDatasources".to_string(),
            "orphanedDatasources.unused-main".to_string(),
            "orphanedDatasources.unused-ops".to_string(),
            "mixedDatasourceDashboards".to_string(),
            "mixedDatasourceDashboards.cpu-main".to_string(),
            "mixedDatasourceDashboards.logs-main".to_string(),
        ]
    );
    assert_eq!(
        domain.blockers,
        vec![
            status_finding("orphaned-datasources", 1, "orphanedDatasources.unused-main",),
            status_finding("orphaned-datasources", 1, "orphanedDatasources.unused-ops",),
            status_finding("mixed-dashboards", 1, "mixedDatasourceDashboards.cpu-main"),
            status_finding("mixed-dashboards", 1, "mixedDatasourceDashboards.logs-main"),
        ]
    );
    assert_eq!(
        domain.next_actions,
        vec!["resolve orphaned datasources, then mixed dashboards".to_string()]
    );
}

#[test]
fn build_dashboard_domain_status_prefers_detail_governance_rows_when_available() {
    let document = json!({
        "summary": {
            "dashboardCount": 2,
            "queryCount": 5,
            "orphanedDatasourceCount": 0,
            "mixedDatasourceDashboardCount": 0,
            "riskRecordCount": 0,
            "highBlastRadiusDatasourceCount": 0,
            "queryAuditCount": 0,
            "dashboardAuditCount": 0,
        },
        "dashboardGovernance": [
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "folderPath": "General",
                "panelCount": 3,
                "queryCount": 4,
                "datasourceCount": 2,
                "datasourceFamilyCount": 2,
                "datasources": ["Prometheus Main", "Loki Main"],
                "datasourceFamilies": ["prometheus", "loki"],
                "mixedDatasource": true,
                "riskCount": 2,
                "riskKinds": ["mixed-datasource-dashboard", "dashboard-panel-pressure"],
            },
            {
                "dashboardUid": "ops-main",
                "dashboardTitle": "Ops Main",
                "folderPath": "Platform",
                "panelCount": 2,
                "queryCount": 1,
                "datasourceCount": 1,
                "datasourceFamilyCount": 1,
                "datasources": ["Prometheus Main"],
                "datasourceFamilies": ["prometheus"],
                "mixedDatasource": false,
                "riskCount": 1,
                "riskKinds": ["empty-query-analysis"],
            }
        ],
        "datasourceGovernance": [
            {
                "datasourceUid": "prom-main",
                "datasource": "Prometheus Main",
                "family": "prometheus",
                "queryCount": 4,
                "dashboardCount": 2,
                "panelCount": 5,
                "mixedDashboardCount": 1,
                "riskCount": 1,
                "riskKinds": ["datasource-high-blast-radius", "mixed-datasource-dashboard"],
                "folderCount": 2,
                "highBlastRadius": true,
                "crossFolder": true,
                "folderPaths": ["General", "Platform"],
                "dashboardUids": ["core-main", "ops-main"],
                "dashboardTitles": ["Core Main", "Ops Main"],
                "orphaned": false,
            }
        ],
        "dashboardDependencies": [
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "folderPath": "General",
                "file": "/tmp/raw/core-main.json",
                "panelCount": 3,
                "queryCount": 4,
                "datasourceCount": 2,
                "datasourceFamilyCount": 2,
                "panelIds": ["1", "2", "3"],
                "datasources": ["Prometheus Main", "Loki Main"],
                "datasourceFamilies": ["prometheus", "loki"],
                "queryFields": ["expr", "query"],
                "panelVariables": [],
                "queryVariables": [],
                "metrics": ["http_requests_total"],
                "functions": ["rate"],
                "measurements": ["job=\"grafana\""],
                "buckets": ["5m"],
            }
        ]
    });

    let domain = build_dashboard_domain_status(Some(&document)).unwrap();

    assert_eq!(domain.status, "ready");
    assert_eq!(domain.reason_code, "ready");
    assert_eq!(domain.primary_count, 5);
    assert_eq!(domain.blocker_count, 0);
    assert_eq!(domain.warning_count, 5);
    assert_eq!(
        domain.signal_keys,
        vec![
            "summary.dashboardCount".to_string(),
            "summary.queryCount".to_string(),
            "summary.orphanedDatasourceCount".to_string(),
            "summary.mixedDatasourceDashboardCount".to_string(),
            "dashboardGovernance".to_string(),
            "dashboardGovernance.core-main".to_string(),
            "dashboardGovernance.ops-main".to_string(),
            "datasourceGovernance".to_string(),
            "datasourceGovernance.prom-main".to_string(),
            "dashboardDependencies".to_string(),
            "dashboardDependencies.core-main".to_string(),
        ]
    );
    assert_eq!(
        domain.warnings,
        vec![
            status_finding("risk-records", 2, "dashboardGovernance.core-main"),
            status_finding("risk-records", 1, "dashboardGovernance.ops-main"),
            status_finding(
                "high-blast-radius-datasources",
                1,
                "datasourceGovernance.prom-main"
            ),
            status_finding("import-readiness-gaps", 1, "dashboardDependencies"),
        ]
    );
    assert_eq!(
        domain.next_actions,
        vec!["review dashboard governance warnings before promotion or apply".to_string()]
    );
}

#[test]
fn build_dashboard_domain_status_reports_import_readiness_gaps_from_dependency_coverage() {
    let document = json!({
        "summary": {
            "dashboardCount": 2,
            "queryCount": 5,
            "orphanedDatasourceCount": 0,
            "mixedDatasourceDashboardCount": 0,
            "riskRecordCount": 0,
            "highBlastRadiusDatasourceCount": 0,
            "queryAuditCount": 0,
            "dashboardAuditCount": 0,
        },
        "dashboardDependencies": [
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "folderPath": "General",
                "file": "/tmp/raw/core-main.json",
                "panelCount": 3,
                "queryCount": 4,
                "datasourceCount": 2,
                "datasourceFamilyCount": 2,
                "panelIds": ["1", "2", "3"],
                "datasources": ["Prometheus Main", "Loki Main"],
                "datasourceFamilies": ["prometheus", "loki"],
                "queryFields": ["expr", "query"],
                "panelVariables": [],
                "queryVariables": [],
                "metrics": ["http_requests_total"],
                "functions": ["rate"],
                "measurements": ["job=\"grafana\""],
                "buckets": ["5m"],
            }
        ]
    });

    let domain = build_dashboard_domain_status(Some(&document)).unwrap();

    assert_eq!(domain.status, "ready");
    assert_eq!(domain.reason_code, "ready");
    assert_eq!(domain.primary_count, 5);
    assert_eq!(domain.blocker_count, 0);
    assert_eq!(domain.warning_count, 1);
    assert_eq!(
        domain.warnings,
        vec![status_finding(
            "import-readiness-gaps",
            1,
            "dashboardDependencies"
        )]
    );
    assert_eq!(
        domain.signal_keys,
        vec![
            "summary.dashboardCount".to_string(),
            "summary.queryCount".to_string(),
            "summary.orphanedDatasourceCount".to_string(),
            "summary.mixedDatasourceDashboardCount".to_string(),
            "dashboardDependencies".to_string(),
            "dashboardDependencies.core-main".to_string(),
        ]
    );
    assert_eq!(
        domain.next_actions,
        vec!["review dashboard governance warnings before promotion or apply".to_string()]
    );
}

#[test]
fn build_dashboard_domain_status_reports_import_readiness_detail_gaps_from_richer_dependency_rows()
{
    let document = json!({
        "summary": {
            "dashboardCount": 2,
            "queryCount": 5,
            "orphanedDatasourceCount": 0,
            "mixedDatasourceDashboardCount": 0,
            "riskRecordCount": 0,
            "highBlastRadiusDatasourceCount": 0,
            "queryAuditCount": 0,
            "dashboardAuditCount": 0,
        },
        "dashboardDependencies": [
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "folderPath": "General",
                "file": "/tmp/raw/core-main.json",
                "panelCount": 3,
                "queryCount": 4,
                "datasourceCount": 2,
                "datasourceFamilyCount": 2,
                "panelIds": ["1", "2", "3"],
                "datasources": ["Prometheus Main", "Loki Main"],
                "datasourceFamilies": ["prometheus", "loki"],
                "queryFields": ["expr", "query"],
                "panelVariables": [],
                "queryVariables": [],
                "metrics": ["http_requests_total"],
                "functions": ["rate"],
                "measurements": ["job=\"grafana\""],
                "buckets": ["5m"],
            },
            {
                "dashboardUid": "ops-main",
                "dashboardTitle": "Ops Main",
                "folderPath": "Platform",
                "file": "/tmp/raw/ops-main.json",
                "panelCount": 2,
                "queryCount": 1,
                "datasourceCount": 1,
                "datasourceFamilyCount": 1,
                "panelIds": [],
                "datasources": ["Prometheus Main"],
                "datasourceFamilies": ["prometheus"],
                "queryFields": [],
                "panelVariables": [],
                "queryVariables": [],
                "metrics": ["up"],
                "functions": ["rate"],
                "measurements": ["service.name"],
                "buckets": ["5m"],
            }
        ]
    });

    let domain = build_dashboard_domain_status(Some(&document)).unwrap();

    assert_eq!(domain.status, "ready");
    assert_eq!(domain.reason_code, "ready");
    assert_eq!(domain.primary_count, 5);
    assert_eq!(domain.blocker_count, 0);
    assert_eq!(domain.warning_count, 1);
    assert_eq!(
        domain.warnings,
        vec![status_finding(
            "import-readiness-detail-gaps",
            1,
            "dashboardDependencies"
        )]
    );
    assert_eq!(
        domain.signal_keys,
        vec![
            "summary.dashboardCount".to_string(),
            "summary.queryCount".to_string(),
            "summary.orphanedDatasourceCount".to_string(),
            "summary.mixedDatasourceDashboardCount".to_string(),
            "dashboardDependencies".to_string(),
            "dashboardDependencies.core-main".to_string(),
            "dashboardDependencies.ops-main".to_string(),
        ]
    );
    assert_eq!(
        domain.next_actions,
        vec!["review dashboard governance warnings before promotion or apply".to_string()]
    );
}

#[test]
fn build_dashboard_domain_status_reports_import_readiness_detail_gaps_when_datasource_coverage_is_missing(
) {
    let document = json!({
        "summary": {
            "dashboardCount": 1,
            "queryCount": 4,
            "orphanedDatasourceCount": 0,
            "mixedDatasourceDashboardCount": 0,
            "riskRecordCount": 0,
            "highBlastRadiusDatasourceCount": 0,
            "queryAuditCount": 0,
            "dashboardAuditCount": 0,
        },
        "dashboardDependencies": [
            {
                "dashboardUid": "core-main",
                "dashboardTitle": "Core Main",
                "folderPath": "General",
                "file": "/tmp/raw/core-main.json",
                "panelCount": 3,
                "queryCount": 4,
                "datasourceCount": 2,
                "datasourceFamilyCount": 2,
                "panelIds": ["1", "2", "3"],
                "datasources": [],
                "datasourceFamilies": [],
                "queryFields": ["expr", "query"],
                "panelVariables": [],
                "queryVariables": [],
                "metrics": ["http_requests_total"],
                "functions": ["rate"],
                "measurements": ["job=\"grafana\""],
                "buckets": ["5m"],
            }
        ]
    });

    let domain = build_dashboard_domain_status(Some(&document)).unwrap();

    assert_eq!(domain.status, "ready");
    assert_eq!(domain.reason_code, "ready");
    assert_eq!(domain.primary_count, 4);
    assert_eq!(domain.blocker_count, 0);
    assert_eq!(domain.warning_count, 1);
    assert_eq!(
        domain.warnings,
        vec![status_finding(
            "import-readiness-detail-gaps",
            1,
            "dashboardDependencies"
        )]
    );
    assert_eq!(
        domain.signal_keys,
        vec![
            "summary.dashboardCount".to_string(),
            "summary.queryCount".to_string(),
            "summary.orphanedDatasourceCount".to_string(),
            "summary.mixedDatasourceDashboardCount".to_string(),
            "dashboardDependencies".to_string(),
            "dashboardDependencies.core-main".to_string(),
        ]
    );
}

#[test]
fn build_dashboard_domain_status_falls_back_to_summary_counts_without_detail_rows() {
    let document = json!({
        "summary": {
            "dashboardCount": 2,
            "queryCount": 5,
            "orphanedDatasourceCount": 1,
            "mixedDatasourceDashboardCount": 2,
            "riskRecordCount": 2,
            "highBlastRadiusDatasourceCount": 1,
            "queryAuditCount": 3,
            "dashboardAuditCount": 1,
        }
    });

    let domain = build_dashboard_domain_status(Some(&document)).unwrap();

    assert_eq!(domain.status, PROJECT_STATUS_BLOCKED);
    assert_eq!(domain.reason_code, "blocked-by-blockers");
    assert_eq!(domain.blocker_count, 3);
    assert_eq!(domain.warning_count, 7);
    assert_eq!(
        domain.blockers,
        vec![
            status_finding("orphaned-datasources", 1, "summary.orphanedDatasourceCount",),
            status_finding(
                "mixed-dashboards",
                2,
                "summary.mixedDatasourceDashboardCount"
            ),
        ]
    );
    assert_eq!(
        domain.signal_keys,
        vec![
            "summary.dashboardCount".to_string(),
            "summary.queryCount".to_string(),
            "summary.orphanedDatasourceCount".to_string(),
            "summary.mixedDatasourceDashboardCount".to_string(),
            "summary.riskRecordCount".to_string(),
            "summary.highBlastRadiusDatasourceCount".to_string(),
            "summary.queryAuditCount".to_string(),
            "summary.dashboardAuditCount".to_string(),
        ]
    );
}
