use serde_json::{Map, Value};
use std::collections::BTreeSet;

use crate::common::{message, Result};
use crate::http::JsonHttpClient;
use crate::project_status::{
    ProjectDomainStatus, PROJECT_STATUS_BLOCKED, PROJECT_STATUS_PARTIAL, PROJECT_STATUS_READY,
};
use crate::project_status_model::{merge_status_record_counts, StatusReading, StatusRecordCount};
use crate::project_status_support::build_live_project_status_client_from_api;

use super::{
    build_live_overall_freshness, PROJECT_STATUS_LIVE_ALL_ORGS_AGGREGATE,
    PROJECT_STATUS_LIVE_ALL_ORGS_MODE_SUFFIX,
};

fn project_status_severity_rank(status: &str) -> usize {
    match status {
        PROJECT_STATUS_BLOCKED => 0,
        PROJECT_STATUS_PARTIAL => 1,
        PROJECT_STATUS_READY => 2,
        _ => 3,
    }
}

fn org_id_from_record(org: &Map<String, Value>) -> Result<i64> {
    org.get("id")
        .and_then(Value::as_i64)
        .ok_or_else(|| message("Grafana org payload did not include a usable numeric id."))
}

pub(super) struct ScopedLiveOrgClient {
    #[cfg(test)]
    org_id: i64,
    client: JsonHttpClient,
}

impl ScopedLiveOrgClient {
    #[cfg(test)]
    pub(super) fn org_id(&self) -> i64 {
        self.org_id
    }

    fn client(&self) -> &JsonHttpClient {
        &self.client
    }
}

pub(super) fn build_scoped_live_org_clients(
    api: &crate::grafana_api::GrafanaApiClient,
    orgs: &[Map<String, Value>],
) -> Result<Vec<ScopedLiveOrgClient>> {
    orgs.iter()
        .map(|org| {
            let org_id = org_id_from_record(org)?;
            let client = build_live_project_status_client_from_api(api, Some(org_id))?;
            Ok(ScopedLiveOrgClient {
                #[cfg(test)]
                org_id,
                client,
            })
        })
        .collect()
}

fn merge_live_domain_statuses(statuses: Vec<ProjectDomainStatus>) -> Result<ProjectDomainStatus> {
    let aggregate = statuses
        .iter()
        .min_by_key(|status| {
            (
                project_status_severity_rank(&status.status),
                usize::MAX - status.blocker_count,
                usize::MAX - status.warning_count,
            )
        })
        .ok_or_else(|| message("Expected at least one per-org domain status to aggregate."))?;
    let blockers = merge_status_record_counts(
        statuses
            .iter()
            .flat_map(|status| status.blockers.iter().cloned().map(StatusRecordCount::from)),
    );
    let warnings = merge_status_record_counts(
        statuses
            .iter()
            .flat_map(|status| status.warnings.iter().cloned().map(StatusRecordCount::from)),
    );
    let freshness = build_live_overall_freshness(&statuses);
    let reason_code = if statuses
        .iter()
        .all(|status| status.reason_code == aggregate.reason_code)
    {
        aggregate.reason_code.clone()
    } else {
        PROJECT_STATUS_LIVE_ALL_ORGS_AGGREGATE.to_string()
    };
    let mode = if statuses.iter().all(|status| status.mode == aggregate.mode) {
        format!(
            "{}{}",
            aggregate.mode, PROJECT_STATUS_LIVE_ALL_ORGS_MODE_SUFFIX
        )
    } else {
        PROJECT_STATUS_LIVE_ALL_ORGS_AGGREGATE.to_string()
    };
    let source_kinds = statuses
        .iter()
        .flat_map(|status| status.source_kinds.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let signal_keys = statuses
        .iter()
        .flat_map(|status| status.signal_keys.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let next_actions = statuses
        .iter()
        .flat_map(|status| status.next_actions.iter().cloned())
        .fold(Vec::<String>::new(), |mut acc, item| {
            if !acc.iter().any(|existing| existing == &item) {
                acc.push(item);
            }
            acc
        });

    Ok(StatusReading {
        id: aggregate.id.clone(),
        scope: aggregate.scope.clone(),
        mode,
        status: aggregate.status.clone(),
        reason_code,
        primary_count: statuses.iter().map(|status| status.primary_count).sum(),
        source_kinds,
        signal_keys,
        blockers,
        warnings,
        next_actions,
        freshness,
    }
    .into_project_domain_status())
}

#[cfg(test)]
pub(super) fn build_live_multi_org_domain_status_with_orgs<F>(
    orgs: &[Map<String, Value>],
    mut build_org_status: F,
) -> Result<ProjectDomainStatus>
where
    F: FnMut(i64) -> Result<ProjectDomainStatus>,
{
    let mut statuses = Vec::new();
    for org in orgs {
        statuses.push(build_org_status(org_id_from_record(org)?)?);
    }
    merge_live_domain_statuses(statuses)
}

pub(super) fn build_live_multi_org_domain_status<F>(
    clients: &[ScopedLiveOrgClient],
    mut build_status: F,
) -> Result<ProjectDomainStatus>
where
    F: FnMut(&JsonHttpClient) -> ProjectDomainStatus,
{
    let mut statuses = Vec::new();
    for client in clients {
        statuses.push(build_status(client.client()));
    }
    merge_live_domain_statuses(statuses)
}

pub(super) fn build_live_multi_org_domain_status_pair<F>(
    clients: &[ScopedLiveOrgClient],
    mut build_statuses: F,
) -> Result<(ProjectDomainStatus, ProjectDomainStatus)>
where
    F: FnMut(&JsonHttpClient) -> (ProjectDomainStatus, ProjectDomainStatus),
{
    let mut first_statuses = Vec::new();
    let mut second_statuses = Vec::new();
    for client in clients {
        let (first, second) = build_statuses(client.client());
        first_statuses.push(first);
        second_statuses.push(second);
    }
    Ok((
        merge_live_domain_statuses(first_statuses)?,
        merge_live_domain_statuses(second_statuses)?,
    ))
}

#[cfg(test)]
pub(super) fn build_live_multi_org_domain_status_with_clients<F>(
    clients: &[ScopedLiveOrgClient],
    mut build_status: F,
) -> Result<ProjectDomainStatus>
where
    F: FnMut(&ScopedLiveOrgClient) -> ProjectDomainStatus,
{
    let mut statuses = Vec::new();
    for client in clients {
        statuses.push(build_status(client));
    }
    merge_live_domain_statuses(statuses)
}

#[cfg(test)]
pub(super) fn scoped_live_org_client_for_test(
    org_id: i64,
    client: JsonHttpClient,
) -> ScopedLiveOrgClient {
    ScopedLiveOrgClient { org_id, client }
}
