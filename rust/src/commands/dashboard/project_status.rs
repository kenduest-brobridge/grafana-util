//! Shared dashboard domain-status producer.
//!
//! Maintainer note:
//! - This module derives one dashboard-owned domain-status row from the staged
//!   inspect summary document.
//! - Keep the producer document-driven and conservative; this module may
//!   surface staged governance warnings from summary fields and the existing
//!   detail arrays, but deeper heuristics should still layer on top of this
//!   base instead of being re-parsed inside overview.

mod details;

use serde_json::Value;

use crate::project_status::{
    ProjectDomainStatus, PROJECT_STATUS_BLOCKED, PROJECT_STATUS_PARTIAL, PROJECT_STATUS_READY,
};
use crate::project_status_model::{StatusProducer, StatusReading, StatusRecordCount};

use self::details::{
    count_import_ready_dashboard_dependencies, count_import_ready_dashboard_dependency_details,
    push_detail_findings, push_detail_findings_with_count, push_detail_signal_keys, push_warning,
    summary_number, top_level_array,
};

const DASHBOARD_DOMAIN_ID: &str = "dashboard";
const DASHBOARD_SCOPE: &str = "staged";
const DASHBOARD_MODE: &str = "inspect-summary";
const DASHBOARD_REASON_READY: &str = PROJECT_STATUS_READY;
const DASHBOARD_REASON_PARTIAL_NO_DATA: &str = "partial-no-data";
const DASHBOARD_REASON_BLOCKED_BY_BLOCKERS: &str = "blocked-by-blockers";

const DASHBOARD_SOURCE_KINDS: &[&str] = &["dashboard-export"];
const DASHBOARD_SIGNAL_KEYS: &[&str] = &[
    "summary.dashboardCount",
    "summary.queryCount",
    "summary.orphanedDatasourceCount",
    "summary.mixedDatasourceDashboardCount",
];
const DASHBOARD_ORPHANED_DATASOURCES_KEY: &str = "orphanedDatasources";
const DASHBOARD_MIXED_DASHBOARDS_KEY: &str = "mixedDatasourceDashboards";
const DASHBOARD_DASHBOARD_GOVERNANCE_KEY: &str = "dashboardGovernance";
const DASHBOARD_DATASOURCE_GOVERNANCE_KEY: &str = "datasourceGovernance";
const DASHBOARD_DASHBOARD_DEPENDENCIES_KEY: &str = "dashboardDependencies";

const DASHBOARD_BLOCKER_ORPHANED_DATASOURCES: &str = "orphaned-datasources";
const DASHBOARD_BLOCKER_MIXED_DASHBOARDS: &str = "mixed-dashboards";
const DASHBOARD_WARNING_RISK_RECORDS: &str = "risk-records";
const DASHBOARD_WARNING_HIGH_BLAST_RADIUS_DATASOURCES: &str = "high-blast-radius-datasources";
const DASHBOARD_WARNING_QUERY_AUDITS: &str = "query-audits";
const DASHBOARD_WARNING_DASHBOARD_AUDITS: &str = "dashboard-audits";
const DASHBOARD_WARNING_IMPORT_READINESS_GAPS: &str = "import-readiness-gaps";
const DASHBOARD_WARNING_IMPORT_READINESS_DETAIL_GAPS: &str = "import-readiness-detail-gaps";

const DASHBOARD_EXPORT_AT_LEAST_ONE_ACTIONS: &[&str] = &["export at least one dashboard"];
const DASHBOARD_REVIEW_GOVERNANCE_WARNINGS_ACTIONS: &[&str] =
    &["review dashboard governance warnings before promotion or apply"];
const DASHBOARD_RESOLVE_BLOCKERS_ACTIONS: &[&str] =
    &["resolve orphaned datasources, then mixed dashboards"];

#[derive(Debug, Clone, Copy)]
struct DashboardDomainStatusInputs<'a> {
    summary_document: Option<&'a Value>,
}

pub(crate) fn build_dashboard_domain_status(
    summary_document: Option<&Value>,
) -> Option<ProjectDomainStatus> {
    DashboardDomainStatusInputs { summary_document }.project_domain_status()
}

impl<'a> StatusProducer for DashboardDomainStatusInputs<'a> {
    fn status_reading(self) -> Option<StatusReading> {
        let document = self.summary_document?;
        let dashboards = summary_number(document, "dashboardCount");
        let queries = summary_number(document, "queryCount");
        let orphaned = summary_number(document, "orphanedDatasourceCount");
        let mixed = summary_number(document, "mixedDatasourceDashboardCount");
        let risk_records = summary_number(document, "riskRecordCount");
        let high_blast_radius_datasources =
            summary_number(document, "highBlastRadiusDatasourceCount");
        let query_audits = summary_number(document, "queryAuditCount");
        let dashboard_audits = summary_number(document, "dashboardAuditCount");
        let orphaned_datasource_items = document
            .get(DASHBOARD_ORPHANED_DATASOURCES_KEY)
            .and_then(Value::as_array)
            .map(Vec::as_slice);
        let mixed_dashboard_items = top_level_array(document, DASHBOARD_MIXED_DASHBOARDS_KEY);
        let dashboard_governance_items =
            top_level_array(document, DASHBOARD_DASHBOARD_GOVERNANCE_KEY);
        let datasource_governance_items =
            top_level_array(document, DASHBOARD_DATASOURCE_GOVERNANCE_KEY);
        let dashboard_dependency_items =
            top_level_array(document, DASHBOARD_DASHBOARD_DEPENDENCIES_KEY);

        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        let mut signal_keys = DASHBOARD_SIGNAL_KEYS
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<String>>();
        push_detail_findings(
            &mut blockers,
            &mut signal_keys,
            DASHBOARD_BLOCKER_ORPHANED_DATASOURCES,
            DASHBOARD_ORPHANED_DATASOURCES_KEY,
            orphaned_datasource_items,
            &["uid", "name"],
            orphaned,
            "summary.orphanedDatasourceCount",
        );
        push_detail_findings(
            &mut blockers,
            &mut signal_keys,
            DASHBOARD_BLOCKER_MIXED_DASHBOARDS,
            DASHBOARD_MIXED_DASHBOARDS_KEY,
            mixed_dashboard_items,
            &["uid", "title"],
            mixed,
            "summary.mixedDatasourceDashboardCount",
        );
        push_detail_findings_with_count(
            &mut warnings,
            &mut signal_keys,
            DASHBOARD_WARNING_RISK_RECORDS,
            DASHBOARD_DASHBOARD_GOVERNANCE_KEY,
            dashboard_governance_items,
            &["dashboardUid", "dashboardTitle"],
            risk_records,
            "summary.riskRecordCount",
            |item| item.get("riskCount").and_then(Value::as_u64).unwrap_or(0) as usize,
        );
        push_detail_findings_with_count(
            &mut warnings,
            &mut signal_keys,
            DASHBOARD_WARNING_HIGH_BLAST_RADIUS_DATASOURCES,
            DASHBOARD_DATASOURCE_GOVERNANCE_KEY,
            datasource_governance_items,
            &["datasourceUid", "datasource"],
            high_blast_radius_datasources,
            "summary.highBlastRadiusDatasourceCount",
            |item| {
                item.get("highBlastRadius")
                    .and_then(Value::as_bool)
                    .unwrap_or(false) as usize
            },
        );
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DASHBOARD_WARNING_QUERY_AUDITS,
            query_audits,
            "summary.queryAuditCount",
        );
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DASHBOARD_WARNING_DASHBOARD_AUDITS,
            dashboard_audits,
            "summary.dashboardAuditCount",
        );
        if dashboard_dependency_items.is_some() {
            let import_ready_dependency_count =
                count_import_ready_dashboard_dependencies(dashboard_dependency_items);
            let import_readiness_gap = dashboards.saturating_sub(import_ready_dependency_count);
            if import_readiness_gap > 0 {
                warnings.push(StatusRecordCount::new(
                    DASHBOARD_WARNING_IMPORT_READINESS_GAPS,
                    import_readiness_gap,
                    DASHBOARD_DASHBOARD_DEPENDENCIES_KEY,
                ));
            }

            let import_ready_detail_count =
                count_import_ready_dashboard_dependency_details(dashboard_dependency_items);
            let import_readiness_detail_gap =
                import_ready_dependency_count.saturating_sub(import_ready_detail_count);
            if import_readiness_detail_gap > 0 {
                warnings.push(StatusRecordCount::new(
                    DASHBOARD_WARNING_IMPORT_READINESS_DETAIL_GAPS,
                    import_readiness_detail_gap,
                    DASHBOARD_DASHBOARD_DEPENDENCIES_KEY,
                ));
            }
        }

        let (status, reason_code, mut next_actions) = if !blockers.is_empty() {
            (
                PROJECT_STATUS_BLOCKED,
                DASHBOARD_REASON_BLOCKED_BY_BLOCKERS,
                DASHBOARD_RESOLVE_BLOCKERS_ACTIONS
                    .iter()
                    .map(|item| (*item).to_string())
                    .collect::<Vec<String>>(),
            )
        } else if dashboards == 0 {
            (
                PROJECT_STATUS_PARTIAL,
                DASHBOARD_REASON_PARTIAL_NO_DATA,
                DASHBOARD_EXPORT_AT_LEAST_ONE_ACTIONS
                    .iter()
                    .map(|item| (*item).to_string())
                    .collect::<Vec<String>>(),
            )
        } else {
            (PROJECT_STATUS_READY, DASHBOARD_REASON_READY, Vec::new())
        };
        if !warnings.is_empty() && blockers.is_empty() {
            next_actions.extend(
                DASHBOARD_REVIEW_GOVERNANCE_WARNINGS_ACTIONS
                    .iter()
                    .map(|item| (*item).to_string()),
            );
        }

        push_detail_signal_keys(
            &mut signal_keys,
            DASHBOARD_DASHBOARD_DEPENDENCIES_KEY,
            dashboard_dependency_items,
            &["dashboardUid", "dashboardTitle"],
        );

        let reading = StatusReading {
            id: DASHBOARD_DOMAIN_ID.to_string(),
            scope: DASHBOARD_SCOPE.to_string(),
            mode: DASHBOARD_MODE.to_string(),
            status: status.to_string(),
            reason_code: reason_code.to_string(),
            primary_count: dashboards.max(queries),
            source_kinds: DASHBOARD_SOURCE_KINDS
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            signal_keys,
            blockers,
            warnings,
            next_actions,
            freshness: Default::default(),
        };

        Some(reading)
    }
}

#[cfg(test)]
#[path = "project_status_tests.rs"]
mod dashboard_project_status_rust_tests;
