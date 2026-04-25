//! Live dashboard domain-status producer.
//!
//! Maintainer note:
//! - This module derives one dashboard-owned domain-status row directly from
//!   live dashboard read surfaces.
//! - Keep the producer conservative and cheap: dashboard summaries and
//!   datasource inventory are enough for readiness and drift signals, and the
//!   dashboard search summaries already carry the folder/title metadata needed
//!   for conservative import-readiness evidence without adding more live
//!   requests.

use reqwest::Method;
use serde_json::{Map, Value};

mod details;
mod status_row;

use details::live_dashboard_status_details;
use status_row::{
    build_live_dashboard_domain_status_from_row_details,
    build_live_dashboard_read_failed_status_row,
};

use crate::common::Result;
use crate::grafana_api::project_status_live as project_status_live_support;
use crate::project_status::ProjectDomainStatus;
use crate::project_status_model::{StatusProducer, StatusReading};

use super::DEFAULT_PAGE_SIZE;

#[derive(Debug, Clone, Default)]
pub(crate) struct LiveDashboardProjectStatusInputs {
    pub dashboard_summaries: Vec<Map<String, Value>>,
    pub datasources: Vec<Map<String, Value>>,
}

pub(crate) fn collect_live_dashboard_project_status_inputs_with_request<F>(
    request_json: &mut F,
) -> Result<LiveDashboardProjectStatusInputs>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    project_status_live_support::collect_live_dashboard_project_status_inputs_with_request(
        request_json,
        DEFAULT_PAGE_SIZE,
    )
}

pub(crate) fn build_live_dashboard_domain_status(
    dashboard_summaries: &[Map<String, Value>],
    datasources: &[Map<String, Value>],
) -> ProjectDomainStatus {
    let details = live_dashboard_status_details(dashboard_summaries, datasources);
    build_live_dashboard_domain_status_from_row_details(
        details.dashboard_count,
        details.datasource_count,
        details.warning_signal_keys,
        details.warnings,
    )
}

pub(crate) fn build_live_dashboard_domain_status_from_inputs(
    inputs: &LiveDashboardProjectStatusInputs,
) -> ProjectDomainStatus {
    build_live_dashboard_domain_status(&inputs.dashboard_summaries, &inputs.datasources)
}

impl StatusProducer for &LiveDashboardProjectStatusInputs {
    fn status_reading(self) -> Option<StatusReading> {
        Some(build_live_dashboard_domain_status_from_inputs(self).into())
    }
}

pub(crate) fn build_live_dashboard_read_failed_domain_status(
    source_kind: &str,
    action: &str,
) -> ProjectDomainStatus {
    build_live_dashboard_read_failed_status_row(source_kind, action).into_project_domain_status()
}

#[allow(dead_code)]
pub(crate) fn build_live_dashboard_domain_status_with_request<F>(
    request_json: F,
) -> Result<ProjectDomainStatus>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut request_json = request_json;
    let inputs = collect_live_dashboard_project_status_inputs_with_request(&mut request_json)?;
    Ok(build_live_dashboard_domain_status_from_inputs(&inputs))
}

#[cfg(test)]
mod live_project_status_rust_tests {
    use super::build_live_dashboard_domain_status_with_request;
    use super::{
        build_live_dashboard_domain_status_from_inputs,
        build_live_dashboard_read_failed_domain_status,
        collect_live_dashboard_project_status_inputs_with_request,
    };
    use crate::project_status::{status_finding, PROJECT_STATUS_PARTIAL, PROJECT_STATUS_READY};
    use serde_json::json;
    use serde_json::Value;

    type TestRequestResult = crate::common::Result<Option<Value>>;

    fn request_fixture(
    ) -> impl FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult
    {
        move |method, path, _params, _payload| {
            let method_name = method.to_string();
            match (method, path) {
                (reqwest::Method::GET, "/api/search") => Ok(Some(json!([
                    {
                        "uid": "cpu-main",
                        "title": "CPU Main",
                        "type": "dash-db",
                        "folderUid": "infra",
                        "folderTitle": "Infra"
                    },
                    {
                        "uid": "logs-main",
                        "title": "Logs Main",
                        "type": "dash-db",
                        "folderUid": "platform",
                        "folderTitle": "Platform"
                    },
                    {
                        "uid": "cpu-main",
                        "title": "CPU Main",
                        "type": "dash-db",
                        "folderUid": "infra",
                        "folderTitle": "Infra"
                    }
                ]))),
                (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([
                    {"uid": "prom-main", "name": "Prometheus Main"},
                    {"uid": "loki-main", "name": "Loki Main"},
                    {"uid": "tempo-main", "name": "Tempo Main"}
                ]))),
                _ => Err(crate::common::message(format!(
                    "unexpected request {:?} {path}",
                    method_name
                ))),
            }
        }
    }

    #[test]
    fn build_live_dashboard_domain_status_tracks_live_summary_and_datasource_reads() {
        let mut request = request_fixture();
        let inputs =
            collect_live_dashboard_project_status_inputs_with_request(&mut request).unwrap();
        let domain = build_live_dashboard_domain_status_from_inputs(&inputs);

        assert_eq!(domain.id, "dashboard");
        assert_eq!(domain.scope, "live");
        assert_eq!(domain.mode, "live-dashboard-read");
        assert_eq!(domain.status, PROJECT_STATUS_READY);
        assert_eq!(domain.reason_code, "ready");
        assert_eq!(domain.primary_count, 2);
        assert_eq!(domain.blocker_count, 0);
        assert_eq!(domain.warning_count, 1);
        assert_eq!(
            domain.source_kinds,
            vec![
                "live-dashboard-search".to_string(),
                "live-datasource-list".to_string(),
            ]
        );
        assert_eq!(
            domain.signal_keys,
            vec![
                "live.dashboardCount".to_string(),
                "live.datasourceCount".to_string(),
                "live.folderCount".to_string(),
                "live.dashboardTitleGapCount".to_string(),
                "live.folderTitleGapCount".to_string(),
                "live.importReadyDashboardCount".to_string(),
                "live.folderSpreadCount".to_string(),
            ]
        );
        assert!(domain.blockers.is_empty());
        assert_eq!(
            domain.warnings,
            vec![status_finding(
                "live-dashboard-folder-spread",
                1,
                "live.folderSpreadCount",
            )]
        );
        assert_eq!(
            domain.next_actions,
            vec![
                "re-run live dashboard read after dashboard, folder, or datasource changes"
                    .to_string(),
                "review live dashboard governance and import-readiness warnings before re-running live dashboard read"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn build_live_dashboard_read_failed_domain_status_preserves_dashboard_contract() {
        let domain = build_live_dashboard_read_failed_domain_status(
            "live-dashboard-search",
            "restore dashboard search access, then re-run live status",
        );

        assert_eq!(domain.id, "dashboard");
        assert_eq!(domain.scope, "live");
        assert_eq!(domain.mode, "live-dashboard-read");
        assert_eq!(domain.status, PROJECT_STATUS_PARTIAL);
        assert_eq!(domain.reason_code, "live-read-failed");
        assert_eq!(domain.primary_count, 0);
        assert_eq!(domain.blocker_count, 1);
        assert_eq!(domain.warning_count, 0);
        assert_eq!(domain.source_kinds, vec!["live-dashboard-search"]);
        assert_eq!(domain.signal_keys, vec!["live.dashboardCount"]);
        assert_eq!(
            domain.blockers,
            vec![status_finding("live-read-failed", 1, "live.dashboardCount")]
        );
        assert_eq!(
            domain.next_actions,
            vec!["restore dashboard search access, then re-run live status".to_string()]
        );
    }

    #[test]
    fn collect_live_dashboard_project_status_inputs_with_request_reads_dashboard_and_datasource_surfaces(
    ) {
        let mut request = request_fixture();
        let inputs =
            collect_live_dashboard_project_status_inputs_with_request(&mut request).unwrap();

        assert_eq!(inputs.dashboard_summaries.len(), 2);
        assert_eq!(inputs.datasources.len(), 3);
        assert_eq!(
            inputs
                .dashboard_summaries
                .first()
                .and_then(|summary| summary.get("uid"))
                .and_then(Value::as_str),
            Some("cpu-main")
        );
        assert_eq!(
            inputs
                .datasources
                .first()
                .and_then(|datasource| datasource.get("uid"))
                .and_then(Value::as_str),
            Some("prom-main")
        );
    }

    #[test]
    fn build_live_dashboard_domain_status_reports_root_scope_warning_for_general_dashboards() {
        let domain = build_live_dashboard_domain_status_with_request(
            move |method, path, _params, _payload| {
                let method_name = method.to_string();
                match (method, path) {
                    (reqwest::Method::GET, "/api/search") => Ok(Some(json!([
                        {
                            "uid": "cpu-main",
                            "title": "CPU Main",
                            "type": "dash-db",
                            "folderUid": "",
                            "folderTitle": "General"
                        },
                        {
                            "uid": "logs-main",
                            "title": "Logs Main",
                            "type": "dash-db",
                            "folderUid": "platform",
                            "folderTitle": "Platform"
                        }
                    ]))),
                    (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([
                        {"uid": "prom-main", "name": "Prometheus Main"}
                    ]))),
                    _ => Err(crate::common::message(format!(
                        "unexpected request {:?} {path}",
                        method_name
                    ))),
                }
            },
        )
        .unwrap();

        assert_eq!(domain.status, PROJECT_STATUS_READY);
        assert_eq!(domain.warning_count, 1);
        assert_eq!(
            domain.warnings,
            vec![status_finding(
                "live-dashboard-root-scope",
                1,
                "live.rootDashboardCount",
            )]
        );
        assert_eq!(
            domain.signal_keys,
            vec![
                "live.dashboardCount".to_string(),
                "live.datasourceCount".to_string(),
                "live.folderCount".to_string(),
                "live.dashboardTitleGapCount".to_string(),
                "live.folderTitleGapCount".to_string(),
                "live.importReadyDashboardCount".to_string(),
                "live.rootDashboardCount".to_string(),
            ]
        );
        assert_eq!(
            domain.next_actions,
            vec![
                "re-run live dashboard read after dashboard, folder, or datasource changes"
                    .to_string(),
                "review live dashboard governance and import-readiness warnings before re-running live dashboard read"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn build_live_dashboard_domain_status_reports_partial_when_no_dashboards_exist() {
        let domain = build_live_dashboard_domain_status_with_request(
            move |method, path, _params, _payload| {
                let method_name = method.to_string();
                match (method, path) {
                    (reqwest::Method::GET, "/api/search") => Ok(Some(json!([]))),
                    (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([
                        {"uid": "prom-main", "name": "Prometheus Main"}
                    ]))),
                    _ => Err(crate::common::message(format!(
                        "unexpected request {:?} {path}",
                        method_name
                    ))),
                }
            },
        )
        .unwrap();

        assert_eq!(domain.status, PROJECT_STATUS_PARTIAL);
        assert_eq!(domain.reason_code, "partial-no-data");
        assert_eq!(domain.primary_count, 0);
        assert_eq!(
            domain.next_actions,
            vec![
                "create or import at least one dashboard, then re-run live dashboard read"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn build_live_dashboard_domain_status_reports_import_readiness_gaps_for_live_dashboards() {
        let domain = build_live_dashboard_domain_status_with_request(
            move |method, path, _params, _payload| {
                let method_name = method.to_string();
                match (method, path) {
                    (reqwest::Method::GET, "/api/search") => Ok(Some(json!([
                        {
                            "uid": "cpu-main",
                            "title": "",
                            "type": "dash-db",
                            "folderUid": "infra",
                            "folderTitle": ""
                        },
                        {
                            "uid": "ops-main",
                            "title": "Ops Main",
                            "type": "dash-db",
                            "folderUid": "",
                            "folderTitle": "General"
                        }
                    ]))),
                    (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([
                        {"uid": "prom-main", "name": "Prometheus Main"}
                    ]))),
                    _ => Err(crate::common::message(format!(
                        "unexpected request {:?} {path}",
                        method_name
                    ))),
                }
            },
        )
        .unwrap();

        assert_eq!(domain.status, PROJECT_STATUS_READY);
        assert_eq!(domain.reason_code, "ready");
        assert_eq!(domain.primary_count, 2);
        assert_eq!(domain.warning_count, 4);
        assert_eq!(
            domain.warnings,
            vec![
                status_finding("live-dashboard-root-scope", 1, "live.rootDashboardCount"),
                status_finding("live-dashboard-title-gap", 1, "live.dashboardTitleGapCount"),
                status_finding(
                    "live-dashboard-folder-title-gap",
                    1,
                    "live.folderTitleGapCount"
                ),
                status_finding(
                    "live-dashboard-import-ready-gap",
                    1,
                    "live.importReadyDashboardCount"
                ),
            ]
        );
        assert_eq!(
            domain.signal_keys,
            vec![
                "live.dashboardCount".to_string(),
                "live.datasourceCount".to_string(),
                "live.folderCount".to_string(),
                "live.dashboardTitleGapCount".to_string(),
                "live.folderTitleGapCount".to_string(),
                "live.importReadyDashboardCount".to_string(),
                "live.rootDashboardCount".to_string(),
            ]
        );
        assert_eq!(
            domain.next_actions,
            vec![
                "re-run live dashboard read after dashboard, folder, or datasource changes"
                    .to_string(),
                "review live dashboard governance and import-readiness warnings before re-running live dashboard read"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn build_live_dashboard_domain_status_reports_partial_when_dashboard_has_no_datasources() {
        let domain = build_live_dashboard_domain_status_with_request(
            move |method, path, _params, _payload| {
                let method_name = method.to_string();
                match (method, path) {
                    (reqwest::Method::GET, "/api/search") => Ok(Some(json!([
                        {
                            "uid": "cpu-main",
                            "title": "CPU Main",
                            "type": "dash-db",
                            "folderUid": "infra",
                            "folderTitle": "Infra"
                        }
                    ]))),
                    (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([]))),
                    _ => Err(crate::common::message(format!(
                        "unexpected request {:?} {path}",
                        method_name
                    ))),
                }
            },
        )
        .unwrap();

        assert_eq!(domain.status, PROJECT_STATUS_PARTIAL);
        assert_eq!(domain.reason_code, "partial-no-datasources");
        assert_eq!(domain.primary_count, 1);
        assert_eq!(domain.warning_count, 1);
        assert_eq!(
            domain.warnings,
            vec![status_finding(
                "live-datasource-inventory-empty",
                1,
                "live.datasourceInventoryEmpty",
            )]
        );
        assert_eq!(
            domain.signal_keys,
            vec![
                "live.dashboardCount".to_string(),
                "live.datasourceCount".to_string(),
                "live.folderCount".to_string(),
                "live.dashboardTitleGapCount".to_string(),
                "live.folderTitleGapCount".to_string(),
                "live.importReadyDashboardCount".to_string(),
                "live.datasourceInventoryEmpty".to_string(),
            ]
        );
        assert_eq!(
            domain.next_actions,
            vec![
                "create or import at least one datasource, then re-run live dashboard read"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn build_live_dashboard_domain_status_reports_datasource_inventory_drift() {
        let domain = build_live_dashboard_domain_status_with_request(
            move |method, path, _params, _payload| {
                let method_name = method.to_string();
                match (method, path) {
                    (reqwest::Method::GET, "/api/search") => Ok(Some(json!([
                        {
                            "uid": "cpu-main",
                            "title": "CPU Main",
                            "type": "dash-db",
                            "folderUid": "infra",
                            "folderTitle": "Infra"
                        }
                    ]))),
                    (reqwest::Method::GET, "/api/datasources") => Ok(Some(json!([
                        {"uid": "prom-main", "name": "Prometheus Main"},
                        {"uid": "prom-main", "name": "Prometheus Main"},
                        {"uid": "loki-main", "name": "Loki Main"}
                    ]))),
                    _ => Err(crate::common::message(format!(
                        "unexpected request {:?} {path}",
                        method_name
                    ))),
                }
            },
        )
        .unwrap();

        assert_eq!(domain.status, PROJECT_STATUS_READY);
        assert_eq!(domain.reason_code, "ready");
        assert_eq!(domain.primary_count, 1);
        assert_eq!(domain.warning_count, 1);
        assert_eq!(
            domain.warnings,
            vec![status_finding(
                "live-datasource-inventory-drift",
                1,
                "live.datasourceDriftCount",
            )]
        );
        assert_eq!(
            domain.signal_keys,
            vec![
                "live.dashboardCount".to_string(),
                "live.datasourceCount".to_string(),
                "live.folderCount".to_string(),
                "live.dashboardTitleGapCount".to_string(),
                "live.folderTitleGapCount".to_string(),
                "live.importReadyDashboardCount".to_string(),
                "live.datasourceDriftCount".to_string(),
            ]
        );
        assert_eq!(
            domain.next_actions,
            vec![
                "re-run live dashboard read after dashboard, folder, or datasource changes"
                    .to_string(),
                "review live dashboard governance and import-readiness warnings before re-running live dashboard read"
                    .to_string(),
            ]
        );
    }
}
