use crate::project_status::{
    ProjectDomainStatus, ProjectStatusFinding, PROJECT_STATUS_PARTIAL, PROJECT_STATUS_READY,
};
use crate::project_status_model::{StatusReading, StatusRecordCount};

const DASHBOARD_DOMAIN_ID: &str = "dashboard";
const DASHBOARD_SCOPE: &str = "live";
const DASHBOARD_MODE: &str = "live-dashboard-read";
const DASHBOARD_REASON_READY: &str = PROJECT_STATUS_READY;
const DASHBOARD_REASON_PARTIAL_NO_DATA: &str = "partial-no-data";
const DASHBOARD_REASON_PARTIAL_NO_DATASOURCES: &str = "partial-no-datasources";
const DASHBOARD_REASON_LIVE_READ_FAILED: &str = "live-read-failed";

const DASHBOARD_SOURCE_KINDS: &[&str] = &["live-dashboard-search", "live-datasource-list"];
const DASHBOARD_SIGNAL_KEYS: &[&str] = &[
    "live.dashboardCount",
    "live.datasourceCount",
    "live.folderCount",
    "live.dashboardTitleGapCount",
    "live.folderTitleGapCount",
    "live.importReadyDashboardCount",
];
const DASHBOARD_PRIMARY_SIGNAL_KEY: &str = "live.dashboardCount";

const DASHBOARD_READY_NEXT_ACTIONS: &[&str] =
    &["re-run live dashboard read after dashboard, folder, or datasource changes"];
const DASHBOARD_NO_DATA_NEXT_ACTIONS: &[&str] =
    &["create or import at least one dashboard, then re-run live dashboard read"];
const DASHBOARD_NO_DATASOURCES_NEXT_ACTIONS: &[&str] =
    &["create or import at least one datasource, then re-run live dashboard read"];
const DASHBOARD_REVIEW_WARNING_NEXT_ACTIONS: &[&str] = &[
    "review live dashboard governance and import-readiness warnings before re-running live dashboard read",
];

pub(super) fn build_live_dashboard_domain_status_from_row_details(
    dashboard_count: usize,
    datasource_count: usize,
    warning_signal_keys: Vec<String>,
    warnings: Vec<ProjectStatusFinding>,
) -> ProjectDomainStatus {
    let warning_count = warnings.len();
    let mut signal_keys = DASHBOARD_SIGNAL_KEYS
        .iter()
        .map(|item| (*item).to_string())
        .collect::<Vec<String>>();
    for signal_key in warning_signal_keys {
        if !signal_keys.iter().any(|item| item == &signal_key) {
            signal_keys.push(signal_key);
        }
    }

    let (status, reason_code, next_actions) =
        live_dashboard_status_summary(dashboard_count, datasource_count, warning_count);

    StatusReading {
        id: DASHBOARD_DOMAIN_ID.to_string(),
        scope: DASHBOARD_SCOPE.to_string(),
        mode: DASHBOARD_MODE.to_string(),
        status: status.to_string(),
        reason_code: reason_code.to_string(),
        primary_count: dashboard_count,
        source_kinds: DASHBOARD_SOURCE_KINDS
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        signal_keys,
        blockers: Vec::new(),
        warnings: warnings.into_iter().map(Into::into).collect(),
        next_actions,
        freshness: Default::default(),
    }
    .into_project_domain_status()
}

pub(super) fn build_live_dashboard_read_failed_status_row(
    source_kind: &str,
    action: &str,
) -> StatusReading {
    StatusReading {
        id: DASHBOARD_DOMAIN_ID.to_string(),
        scope: DASHBOARD_SCOPE.to_string(),
        mode: DASHBOARD_MODE.to_string(),
        status: PROJECT_STATUS_PARTIAL.to_string(),
        reason_code: DASHBOARD_REASON_LIVE_READ_FAILED.to_string(),
        primary_count: 0,
        source_kinds: vec![source_kind.to_string()],
        signal_keys: vec![DASHBOARD_PRIMARY_SIGNAL_KEY.to_string()],
        blockers: vec![StatusRecordCount::new(
            DASHBOARD_REASON_LIVE_READ_FAILED,
            1,
            DASHBOARD_PRIMARY_SIGNAL_KEY,
        )],
        warnings: Vec::new(),
        next_actions: vec![action.to_string()],
        freshness: Default::default(),
    }
}

fn live_dashboard_status_summary(
    dashboard_count: usize,
    datasource_count: usize,
    warning_count: usize,
) -> (&'static str, &'static str, Vec<String>) {
    let (status, reason_code, mut next_actions) = if dashboard_count == 0 {
        (
            PROJECT_STATUS_PARTIAL,
            DASHBOARD_REASON_PARTIAL_NO_DATA,
            string_list(DASHBOARD_NO_DATA_NEXT_ACTIONS),
        )
    } else if datasource_count == 0 {
        (
            PROJECT_STATUS_PARTIAL,
            DASHBOARD_REASON_PARTIAL_NO_DATASOURCES,
            string_list(DASHBOARD_NO_DATASOURCES_NEXT_ACTIONS),
        )
    } else {
        (
            PROJECT_STATUS_READY,
            DASHBOARD_REASON_READY,
            string_list(DASHBOARD_READY_NEXT_ACTIONS),
        )
    };
    if warning_count > 0 && status == PROJECT_STATUS_READY {
        next_actions.extend(string_list(DASHBOARD_REVIEW_WARNING_NEXT_ACTIONS));
    }
    (status, reason_code, next_actions)
}

fn string_list(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
