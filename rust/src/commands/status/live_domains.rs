use serde_json::{Map, Value};
use std::fs::Metadata;

use crate::access::{build_access_live_domain_status, build_access_live_read_failed_domain_status};
use crate::alert::{
    build_alert_live_project_status_domain, build_alert_live_read_failed_domain_status,
    AlertLiveProjectStatusInputs,
};
use crate::common::Result;
use crate::dashboard::{
    build_live_dashboard_domain_status_from_inputs, build_live_dashboard_read_failed_domain_status,
    LiveDashboardProjectStatusInputs, DEFAULT_PAGE_SIZE,
};
use crate::datasource_live_project_status::{
    build_datasource_live_project_status_from_inputs,
    build_live_datasource_read_failed_domain_status, LiveDatasourceProjectStatusInputs,
};
use crate::grafana_api::{DashboardResourceClient, DatasourceResourceClient};
use crate::http::JsonHttpClient;
use crate::project_status::ProjectDomainStatus;
use crate::project_status_freshness::ProjectStatusFreshnessSample;
use crate::project_status_support::project_status_live;
use crate::sync::{
    build_live_promotion_domain_status_transport, build_live_promotion_project_status,
    build_live_sync_domain_status, build_live_sync_domain_status_transport,
    LivePromotionProjectStatusInputs, SyncLiveProjectStatusInputs,
};

use super::stamp_live_domain_freshness;

struct LiveDashboardDatasourceReadPass {
    dashboard_summaries: Result<Vec<Map<String, Value>>>,
    datasources: Result<Vec<Map<String, Value>>>,
    datasource_org_list: Vec<Map<String, Value>>,
    datasource_current_org: Option<Map<String, Value>>,
}

fn collect_live_dashboard_datasource_read_pass(
    client: &JsonHttpClient,
) -> LiveDashboardDatasourceReadPass {
    let dashboard_client = DashboardResourceClient::new(client);
    let datasource_client = DatasourceResourceClient::new(client);
    let dashboard_summaries = dashboard_client.list_dashboard_summaries(DEFAULT_PAGE_SIZE);
    let datasources = datasource_client.list_datasources();
    let (datasource_org_list, datasource_current_org) = if datasources.is_ok() {
        (
            datasource_client.list_orgs().unwrap_or_default(),
            datasource_client.fetch_current_org().ok(),
        )
    } else {
        (Vec::new(), None)
    };

    LiveDashboardDatasourceReadPass {
        dashboard_summaries,
        datasources,
        datasource_org_list,
        datasource_current_org,
    }
}

fn build_live_dashboard_status_from_read_pass(
    client: &JsonHttpClient,
    read_pass: &LiveDashboardDatasourceReadPass,
) -> ProjectDomainStatus {
    match (
        read_pass.dashboard_summaries.as_ref(),
        read_pass.datasources.as_ref(),
    ) {
        (Ok(dashboard_summaries), Ok(datasources)) => {
            let inputs = LiveDashboardProjectStatusInputs {
                dashboard_summaries: dashboard_summaries.clone(),
                datasources: datasources.clone(),
            };
            let status = build_live_dashboard_domain_status_from_inputs(&inputs);
            let mut freshness_samples =
                project_status_live::dashboard_project_status_freshness_samples(&inputs);
            let dashboard_version_timestamp = if freshness_samples.is_empty() {
                project_status_live::latest_dashboard_version_timestamp(
                    client,
                    &inputs.dashboard_summaries,
                )
            } else {
                None
            };
            if let Some(observed_at) = dashboard_version_timestamp.as_deref() {
                freshness_samples.push(ProjectStatusFreshnessSample::ObservedAtRfc3339 {
                    source: "dashboard-version-history",
                    observed_at,
                });
            }
            stamp_live_domain_freshness(status, &freshness_samples)
        }
        _ => build_live_dashboard_read_failed_domain_status(
            "live-dashboard-search",
            "restore dashboard search access, then re-run live status",
        ),
    }
}

fn build_live_datasource_status_from_read_pass(
    read_pass: &LiveDashboardDatasourceReadPass,
) -> ProjectDomainStatus {
    let status = match read_pass.datasources.as_ref() {
        Ok(datasources) => {
            let inputs = LiveDatasourceProjectStatusInputs {
                datasource_list: datasources.clone(),
                org_list: read_pass.datasource_org_list.clone(),
                current_org: read_pass.datasource_current_org.clone(),
            };
            build_datasource_live_project_status_from_inputs(&inputs).unwrap_or_else(|| {
                build_live_datasource_read_failed_domain_status(
                    "live-datasource-list",
                    "restore datasource inventory access, then re-run live status",
                )
            })
        }
        Err(_) => build_live_datasource_read_failed_domain_status(
            "live-datasource-list",
            "restore datasource inventory access, then re-run live status",
        ),
    };
    stamp_live_domain_freshness(status, &[])
}

pub(super) fn build_live_dashboard_and_datasource_statuses(
    client: &JsonHttpClient,
) -> (ProjectDomainStatus, ProjectDomainStatus) {
    let read_pass = collect_live_dashboard_datasource_read_pass(client);
    (
        build_live_dashboard_status_from_read_pass(client, &read_pass),
        build_live_datasource_status_from_read_pass(&read_pass),
    )
}

pub(super) fn build_live_alert_status(client: &JsonHttpClient) -> ProjectDomainStatus {
    let documents = project_status_live::load_alert_surface_documents(client);
    let status = build_alert_live_project_status_domain(AlertLiveProjectStatusInputs {
        rules_document: documents.rules.as_ref(),
        contact_points_document: documents.contact_points.as_ref(),
        mute_timings_document: documents.mute_timings.as_ref(),
        policies_document: documents.policies.as_ref(),
        templates_document: documents.templates.as_ref(),
    })
    .unwrap_or_else(|| {
        build_alert_live_read_failed_domain_status(
            "alert",
            "restore alert read access, then re-run live status",
        )
    });
    let freshness_samples = project_status_live::alert_project_status_freshness_samples(&documents);
    stamp_live_domain_freshness(status, &freshness_samples)
}

pub(super) fn build_live_access_status(client: &JsonHttpClient) -> ProjectDomainStatus {
    let status = build_access_live_domain_status(client).unwrap_or_else(|| {
        build_access_live_read_failed_domain_status(
            "grafana-utils-access-live-org-users",
            "restore access read scopes, then re-run live status",
        )
    });
    stamp_live_domain_freshness(status, &[])
}

pub(super) fn build_live_sync_status(
    sync_summary_document: Option<&Value>,
    bundle_preflight_document: Option<&Value>,
    sync_summary_metadata: Option<&Metadata>,
    bundle_preflight_metadata: Option<&Metadata>,
) -> ProjectDomainStatus {
    let status = build_live_sync_domain_status(SyncLiveProjectStatusInputs {
        summary_document: sync_summary_document,
        bundle_preflight_document,
    })
    .unwrap_or_else(build_live_sync_domain_status_transport);
    let mut samples = Vec::new();
    if let Some(metadata) = sync_summary_metadata {
        samples.push(ProjectStatusFreshnessSample::ObservedAtMetadata {
            source: "sync-summary",
            metadata,
        });
    }
    if let Some(metadata) = bundle_preflight_metadata {
        samples.push(ProjectStatusFreshnessSample::ObservedAtMetadata {
            source: "bundle-preflight",
            metadata,
        });
    }
    stamp_live_domain_freshness(status, &samples)
}

pub(super) fn build_live_promotion_status(
    promotion_summary_document: Option<&Value>,
    promotion_mapping_document: Option<&Value>,
    availability_document: Option<&Value>,
    promotion_summary_metadata: Option<&Metadata>,
    promotion_mapping_metadata: Option<&Metadata>,
    availability_metadata: Option<&Metadata>,
) -> ProjectDomainStatus {
    let status = build_live_promotion_project_status(LivePromotionProjectStatusInputs {
        promotion_summary_document,
        promotion_mapping_document,
        availability_document,
    })
    .unwrap_or_else(build_live_promotion_domain_status_transport);
    let mut samples = Vec::new();
    if let Some(metadata) = promotion_summary_metadata {
        samples.push(ProjectStatusFreshnessSample::ObservedAtMetadata {
            source: "promotion-summary",
            metadata,
        });
    }
    if let Some(metadata) = promotion_mapping_metadata {
        samples.push(ProjectStatusFreshnessSample::ObservedAtMetadata {
            source: "promotion-mapping",
            metadata,
        });
    }
    if let Some(metadata) = availability_metadata {
        samples.push(ProjectStatusFreshnessSample::ObservedAtMetadata {
            source: "availability",
            metadata,
        });
    }
    stamp_live_domain_freshness(status, &samples)
}
