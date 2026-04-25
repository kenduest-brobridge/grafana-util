//! Live project status runtime orchestration.
//!
//! Responsibilities:
//! - Build per-domain collectors for dashboard, datasource, alert, access, and sync status.
//! - Aggregate live findings across orgs and score combined freshness/severity.
//! - Emit a stable status document for `project-status` reporting.

#[path = "live_discovery.rs"]
mod live_discovery;
#[path = "live_domains.rs"]
mod live_domains;
#[path = "live_multi_org.rs"]
mod live_multi_org;
#[cfg(test)]
#[path = "live_tests.rs"]
mod tests;

use serde_json::Value;
use std::fs::Metadata;
use std::path::PathBuf;

use crate::access::build_access_live_read_failed_domain_status;
use crate::alert::build_alert_live_read_failed_domain_status;
use crate::common::{load_json_object_file, message, Result};
use crate::dashboard::build_live_dashboard_read_failed_domain_status;
use crate::datasource_live_project_status::build_live_datasource_read_failed_domain_status;
use crate::project_status::{
    build_project_status, ProjectDomainStatus, ProjectStatus, ProjectStatusFreshness,
};
use crate::project_status_command::{ProjectStatusLiveArgs, PROJECT_STATUS_DOMAIN_COUNT};
use crate::project_status_freshness::{
    build_live_project_status_freshness_from_parts, project_status_freshness_parts_from_ages,
    project_status_freshness_parts_from_samples, ProjectStatusFreshnessSample,
};
use crate::project_status_support::{build_live_project_status_api_client, project_status_live};

use self::live_discovery::build_live_status_discovery;
use self::live_domains::{
    build_live_access_status, build_live_alert_status,
    build_live_dashboard_and_datasource_statuses, build_live_promotion_status,
    build_live_sync_status,
};
use self::live_multi_org::{
    build_live_multi_org_domain_status, build_live_multi_org_domain_status_pair,
    build_scoped_live_org_clients, ScopedLiveOrgClient,
};

const PROJECT_STATUS_LIVE_SCOPE: &str = "live";
const PROJECT_STATUS_LIVE_ALL_ORGS_MODE_SUFFIX: &str = "-all-orgs";
const PROJECT_STATUS_LIVE_ALL_ORGS_AGGREGATE: &str = "multi-org-aggregate";
const PROJECT_STATUS_LIVE_INSTANCE_SOURCE: &str = "api-health";

enum AllOrgScopedClients {
    OrgListFailed,
    Scoped(Result<Vec<ScopedLiveOrgClient>>),
}

fn build_status_with_optional_all_org_scope<F, G>(
    root_client: &crate::http::JsonHttpClient,
    all_org_scope: Option<&AllOrgScopedClients>,
    mut build_status: F,
    build_read_failed: G,
    domain_source: &str,
    domain_action: &str,
) -> ProjectDomainStatus
where
    F: FnMut(&crate::http::JsonHttpClient) -> ProjectDomainStatus,
    G: Fn(&str, &str) -> ProjectDomainStatus,
{
    match all_org_scope {
        Some(AllOrgScopedClients::Scoped(Ok(clients))) if !clients.is_empty() => {
            build_live_multi_org_domain_status(clients, build_status)
                .unwrap_or_else(|_| build_read_failed(domain_source, domain_action))
        }
        Some(AllOrgScopedClients::Scoped(Ok(_))) | None => build_status(root_client),
        Some(AllOrgScopedClients::Scoped(Err(_))) => {
            build_read_failed(domain_source, domain_action)
        }
        Some(AllOrgScopedClients::OrgListFailed) => build_read_failed(
            "live-org-list",
            "restore org list access, then re-run live status --all-orgs",
        ),
    }
}

fn load_optional_project_status_document_with_metadata(
    path: Option<&PathBuf>,
    label: &str,
) -> Result<Option<(Value, Metadata)>> {
    path.map(|path| {
        let document = load_json_object_file(path, label)?;
        let metadata = std::fs::metadata(path)
            .map_err(|error| message(format!("Failed to stat {}: {}", path.display(), error)))?;
        Ok((document, metadata))
    })
    .transpose()
}

fn stamp_live_domain_freshness(
    mut domain: ProjectDomainStatus,
    samples: &[ProjectStatusFreshnessSample<'_>],
) -> ProjectDomainStatus {
    let freshness = if samples.is_empty() {
        project_status_freshness_parts_from_ages(domain.source_kinds.len(), &[])
    } else {
        project_status_freshness_parts_from_samples(samples)
    };
    domain.freshness = build_live_project_status_freshness_from_parts(freshness);
    domain
}

fn build_live_overall_freshness(domains: &[ProjectDomainStatus]) -> ProjectStatusFreshness {
    let mut ages = Vec::new();
    let mut source_count = 0usize;
    for domain in domains {
        source_count += domain.freshness.source_count;
        if let Some(age) = domain.freshness.newest_age_seconds {
            ages.push(age);
        }
        if let Some(age) = domain.freshness.oldest_age_seconds {
            ages.push(age);
        }
    }
    build_live_project_status_freshness_from_parts(project_status_freshness_parts_from_ages(
        source_count,
        &ages,
    ))
}

pub(crate) fn build_live_project_status(args: &ProjectStatusLiveArgs) -> Result<ProjectStatus> {
    let api = build_live_project_status_api_client(args)?;
    let client = api.http_client().clone();
    let sync_summary_document = load_optional_project_status_document_with_metadata(
        args.sync_summary_file.as_ref(),
        "Project status sync summary input",
    )?;
    let bundle_preflight_document = load_optional_project_status_document_with_metadata(
        args.bundle_preflight_file.as_ref(),
        "Project status bundle preflight input",
    )?;
    let promotion_summary_document = load_optional_project_status_document_with_metadata(
        args.promotion_summary_file.as_ref(),
        "Project status promotion summary input",
    )?;
    let promotion_mapping_document = load_optional_project_status_document_with_metadata(
        args.mapping_file.as_ref(),
        "Project status mapping input",
    )?;
    let availability_document = load_optional_project_status_document_with_metadata(
        args.availability_file.as_ref(),
        "Project status availability input",
    )?;
    let all_org_scoped_clients = if args.all_orgs {
        Some(match project_status_live::list_visible_orgs(&client) {
            Ok(orgs) if orgs.is_empty() => AllOrgScopedClients::Scoped(Ok(Vec::new())),
            Ok(orgs) => AllOrgScopedClients::Scoped(build_scoped_live_org_clients(&api, &orgs)),
            Err(_) => AllOrgScopedClients::OrgListFailed,
        })
    } else {
        None
    };
    let (dashboard_status, datasource_status) = match all_org_scoped_clients.as_ref() {
        Some(AllOrgScopedClients::Scoped(Ok(clients))) if !clients.is_empty() => {
            build_live_multi_org_domain_status_pair(
                clients,
                build_live_dashboard_and_datasource_statuses,
            )
            .unwrap_or_else(|_| {
                (
                    build_live_dashboard_read_failed_domain_status(
                        "live-dashboard-search",
                        "restore dashboard/org read access, then re-run live status --all-orgs",
                    ),
                    build_live_datasource_read_failed_domain_status(
                        "live-datasource-list",
                        "restore datasource/org read access, then re-run live status --all-orgs",
                    ),
                )
            })
        }
        Some(AllOrgScopedClients::Scoped(Ok(_))) | None => {
            build_live_dashboard_and_datasource_statuses(&client)
        }
        Some(AllOrgScopedClients::Scoped(Err(_))) => (
            build_live_dashboard_read_failed_domain_status(
                "live-dashboard-search",
                "restore dashboard/org read access, then re-run live status --all-orgs",
            ),
            build_live_datasource_read_failed_domain_status(
                "live-datasource-list",
                "restore datasource/org read access, then re-run live status --all-orgs",
            ),
        ),
        Some(AllOrgScopedClients::OrgListFailed) => (
            build_live_dashboard_read_failed_domain_status(
                "live-org-list",
                "restore org list access, then re-run live status --all-orgs",
            ),
            build_live_datasource_read_failed_domain_status(
                "live-org-list",
                "restore org list access, then re-run live status --all-orgs",
            ),
        ),
    };
    let alert_status = build_status_with_optional_all_org_scope(
        &client,
        all_org_scoped_clients.as_ref(),
        build_live_alert_status,
        build_alert_live_read_failed_domain_status,
        "alert",
        "restore alert/org read access, then re-run live status --all-orgs",
    );
    let access_status = build_status_with_optional_all_org_scope(
        &client,
        all_org_scoped_clients.as_ref(),
        build_live_access_status,
        build_access_live_read_failed_domain_status,
        "grafana-utils-access-live-org-users",
        "restore access/org read access, then re-run live status --all-orgs",
    );
    let domains = vec![
        dashboard_status,
        datasource_status,
        alert_status,
        access_status,
        build_live_sync_status(
            sync_summary_document.as_ref().map(|(document, _)| document),
            bundle_preflight_document
                .as_ref()
                .map(|(document, _)| document),
            sync_summary_document.as_ref().map(|(_, metadata)| metadata),
            bundle_preflight_document
                .as_ref()
                .map(|(_, metadata)| metadata),
        ),
        build_live_promotion_status(
            promotion_summary_document
                .as_ref()
                .map(|(document, _)| document),
            promotion_mapping_document
                .as_ref()
                .map(|(document, _)| document),
            availability_document.as_ref().map(|(document, _)| document),
            promotion_summary_document
                .as_ref()
                .map(|(_, metadata)| metadata),
            promotion_mapping_document
                .as_ref()
                .map(|(_, metadata)| metadata),
            availability_document.as_ref().map(|(_, metadata)| metadata),
        ),
    ];
    let overall_freshness = build_live_overall_freshness(&domains);
    let mut status = build_project_status(
        PROJECT_STATUS_LIVE_SCOPE,
        PROJECT_STATUS_DOMAIN_COUNT,
        overall_freshness,
        domains,
    );
    status.discovery = Some(build_live_status_discovery(&client));
    Ok(status)
}
