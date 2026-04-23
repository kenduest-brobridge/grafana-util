use serde_json::Value;
use std::fs::Metadata;

use crate::access::{build_access_live_domain_status, build_access_live_read_failed_domain_status};
use crate::alert::{
    build_alert_live_project_status_domain, build_alert_live_read_failed_domain_status,
    AlertLiveProjectStatusInputs,
};
use crate::dashboard::{
    build_live_dashboard_domain_status_from_inputs, build_live_dashboard_read_failed_domain_status,
    DEFAULT_PAGE_SIZE,
};
use crate::datasource_live_project_status::{
    build_datasource_live_project_status_from_inputs,
    build_live_datasource_read_failed_domain_status,
    collect_live_datasource_project_status_inputs_with_request,
};
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

pub(super) fn build_live_dashboard_status(client: &JsonHttpClient) -> ProjectDomainStatus {
    match project_status_live::collect_live_dashboard_project_status_inputs(
        client,
        DEFAULT_PAGE_SIZE,
    ) {
        Ok(inputs) => {
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
        Err(_) => build_live_dashboard_read_failed_domain_status(
            "live-dashboard-search",
            "restore dashboard search access, then re-run live status",
        ),
    }
}

pub(super) fn build_live_datasource_status(client: &JsonHttpClient) -> ProjectDomainStatus {
    let status = match collect_live_datasource_project_status_inputs_with_request(
        &mut |method, path, params, payload| client.request_json(method, path, params, payload),
    ) {
        Ok(inputs) => {
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
