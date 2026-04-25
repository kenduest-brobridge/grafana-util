use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::common::string_field;
use crate::project_status::{status_finding, ProjectStatusFinding};

const DASHBOARD_DATASOURCE_EMPTY_SIGNAL_KEY: &str = "live.datasourceInventoryEmpty";
const DASHBOARD_DATASOURCE_DRIFT_SIGNAL_KEY: &str = "live.datasourceDriftCount";
const DASHBOARD_FOLDER_SPREAD_SIGNAL_KEY: &str = "live.folderSpreadCount";
const DASHBOARD_ROOT_SCOPE_SIGNAL_KEY: &str = "live.rootDashboardCount";
const DASHBOARD_TITLE_GAP_SIGNAL_KEY: &str = "live.dashboardTitleGapCount";
const DASHBOARD_FOLDER_TITLE_GAP_SIGNAL_KEY: &str = "live.folderTitleGapCount";
const DASHBOARD_IMPORT_READY_SIGNAL_KEY: &str = "live.importReadyDashboardCount";

const DASHBOARD_WARNING_KIND_EMPTY_DATASOURCE_INVENTORY: &str = "live-datasource-inventory-empty";
const DASHBOARD_WARNING_KIND_DATASOURCE_DRIFT: &str = "live-datasource-inventory-drift";
const DASHBOARD_WARNING_KIND_FOLDER_SPREAD: &str = "live-dashboard-folder-spread";
const DASHBOARD_WARNING_KIND_ROOT_SCOPE: &str = "live-dashboard-root-scope";
const DASHBOARD_WARNING_KIND_TITLE_GAP: &str = "live-dashboard-title-gap";
const DASHBOARD_WARNING_KIND_FOLDER_TITLE_GAP: &str = "live-dashboard-folder-title-gap";

pub(super) struct LiveDashboardStatusDetails {
    pub dashboard_count: usize,
    pub datasource_count: usize,
    pub warning_signal_keys: Vec<String>,
    pub warnings: Vec<ProjectStatusFinding>,
}

pub(super) fn live_dashboard_status_details(
    dashboard_summaries: &[Map<String, Value>],
    datasources: &[Map<String, Value>],
) -> LiveDashboardStatusDetails {
    let dashboard_count = count_unique_dashboard_uids(dashboard_summaries);
    let root_scoped_dashboard_count = count_root_scoped_dashboards(dashboard_summaries);
    let folder_count = count_unique_folder_uids(dashboard_summaries);
    let dashboard_title_gap_count = count_dashboard_title_gaps(dashboard_summaries);
    let folder_title_gap_count = count_folder_title_gaps(dashboard_summaries);
    let import_ready_dashboard_count = count_import_ready_dashboards(dashboard_summaries);
    let datasource_count = count_unique_datasource_uids(datasources);
    let raw_datasource_count = datasources.len();

    let mut warning_signal_keys = Vec::new();
    let warnings = build_live_dashboard_warnings(
        dashboard_count,
        folder_count,
        root_scoped_dashboard_count,
        dashboard_title_gap_count,
        folder_title_gap_count,
        import_ready_dashboard_count,
        datasource_count,
        raw_datasource_count,
        &mut warning_signal_keys,
    );

    LiveDashboardStatusDetails {
        dashboard_count,
        datasource_count,
        warning_signal_keys,
        warnings,
    }
}

fn count_unique_dashboard_uids(dashboard_summaries: &[Map<String, Value>]) -> usize {
    let mut seen = BTreeSet::new();
    for summary in dashboard_summaries {
        let uid = string_field(summary, "uid", "");
        if !uid.is_empty() {
            seen.insert(uid);
        }
    }
    seen.len()
}

fn count_unique_folder_uids(dashboard_summaries: &[Map<String, Value>]) -> usize {
    let mut seen = BTreeSet::new();
    for summary in dashboard_summaries {
        let folder_uid = string_field(summary, "folderUid", "");
        if !folder_uid.is_empty() {
            seen.insert(folder_uid);
        }
    }
    seen.len()
}

fn count_root_scoped_dashboards(dashboard_summaries: &[Map<String, Value>]) -> usize {
    dashboard_summaries
        .iter()
        .filter(|summary| string_field(summary, "folderUid", "").is_empty())
        .count()
}

fn count_dashboard_title_gaps(dashboard_summaries: &[Map<String, Value>]) -> usize {
    dashboard_summaries
        .iter()
        .filter(|summary| string_field(summary, "title", "").trim().is_empty())
        .count()
}

fn count_folder_title_gaps(dashboard_summaries: &[Map<String, Value>]) -> usize {
    dashboard_summaries
        .iter()
        .filter(|summary| {
            !string_field(summary, "folderUid", "").trim().is_empty()
                && string_field(summary, "folderTitle", "").trim().is_empty()
        })
        .count()
}

fn count_import_ready_dashboards(dashboard_summaries: &[Map<String, Value>]) -> usize {
    let mut seen = BTreeSet::new();
    for summary in dashboard_summaries {
        let uid = string_field(summary, "uid", "");
        if uid.is_empty() {
            continue;
        }

        let title = string_field(summary, "title", "");
        let folder_uid = string_field(summary, "folderUid", "");
        let folder_title = string_field(summary, "folderTitle", "");
        let folder_is_ready = folder_uid.trim().is_empty() || !folder_title.trim().is_empty();
        if !title.trim().is_empty() && folder_is_ready {
            seen.insert(uid);
        }
    }
    seen.len()
}

fn count_unique_datasource_uids(datasources: &[Map<String, Value>]) -> usize {
    let mut seen = BTreeSet::new();
    for datasource in datasources {
        let uid = string_field(datasource, "uid", "");
        if !uid.is_empty() {
            seen.insert(uid);
        }
    }
    seen.len()
}

fn push_warning(
    warnings: &mut Vec<ProjectStatusFinding>,
    signal_keys: &mut Vec<String>,
    kind: &str,
    count: usize,
    source: &str,
) {
    if count == 0 {
        return;
    }
    warnings.push(status_finding(kind, count, source));
    if !signal_keys.iter().any(|item| item == source) {
        signal_keys.push(source.to_string());
    }
}

#[allow(clippy::too_many_arguments)]
fn build_live_dashboard_warnings(
    dashboard_count: usize,
    folder_count: usize,
    root_scoped_dashboard_count: usize,
    dashboard_title_gap_count: usize,
    folder_title_gap_count: usize,
    import_ready_dashboard_count: usize,
    datasource_count: usize,
    raw_datasource_count: usize,
    signal_keys: &mut Vec<String>,
) -> Vec<ProjectStatusFinding> {
    let mut warnings = Vec::new();
    if dashboard_count > 0 && datasource_count == 0 {
        push_warning(
            &mut warnings,
            signal_keys,
            DASHBOARD_WARNING_KIND_EMPTY_DATASOURCE_INVENTORY,
            1,
            DASHBOARD_DATASOURCE_EMPTY_SIGNAL_KEY,
        );
        return warnings;
    }

    let drift_count = raw_datasource_count.saturating_sub(datasource_count);
    push_warning(
        &mut warnings,
        signal_keys,
        DASHBOARD_WARNING_KIND_DATASOURCE_DRIFT,
        drift_count,
        DASHBOARD_DATASOURCE_DRIFT_SIGNAL_KEY,
    );
    push_warning(
        &mut warnings,
        signal_keys,
        DASHBOARD_WARNING_KIND_FOLDER_SPREAD,
        folder_count.saturating_sub(1),
        DASHBOARD_FOLDER_SPREAD_SIGNAL_KEY,
    );
    if folder_count > 0 {
        push_warning(
            &mut warnings,
            signal_keys,
            DASHBOARD_WARNING_KIND_ROOT_SCOPE,
            root_scoped_dashboard_count,
            DASHBOARD_ROOT_SCOPE_SIGNAL_KEY,
        );
    }
    push_warning(
        &mut warnings,
        signal_keys,
        DASHBOARD_WARNING_KIND_TITLE_GAP,
        dashboard_title_gap_count,
        DASHBOARD_TITLE_GAP_SIGNAL_KEY,
    );
    push_warning(
        &mut warnings,
        signal_keys,
        DASHBOARD_WARNING_KIND_FOLDER_TITLE_GAP,
        folder_title_gap_count,
        DASHBOARD_FOLDER_TITLE_GAP_SIGNAL_KEY,
    );
    push_warning(
        &mut warnings,
        signal_keys,
        "live-dashboard-import-ready-gap",
        dashboard_count.saturating_sub(import_ready_dashboard_count),
        DASHBOARD_IMPORT_READY_SIGNAL_KEY,
    );
    warnings
}
