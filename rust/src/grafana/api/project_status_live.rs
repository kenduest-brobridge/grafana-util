//! Shared live reads for the project-status workflow.
//!
//! Keep this module workflow-level: it should gather the live documents needed
//! by project-status without turning into a generic endpoint SDK.

use crate::common::{message, value_as_object, Result};
use crate::dashboard::LiveDashboardProjectStatusInputs;
use crate::grafana_api::{
    AccessResourceClient, AlertingResourceClient, DashboardResourceClient, DatasourceResourceClient,
};
use crate::http::JsonHttpClient;
use crate::project_status_freshness::ProjectStatusFreshnessSample;
use reqwest::Method;
use serde_json::{Map, Value};

#[cfg(test)]
use crate::dashboard::build_live_dashboard_domain_status_from_inputs;
#[cfg(test)]
use crate::project_status::{ProjectDomainStatus, PROJECT_STATUS_PARTIAL};
#[cfg(test)]
use crate::project_status_freshness::{
    build_live_project_status_freshness_from_samples,
    build_live_project_status_freshness_from_source_count,
};
#[cfg(test)]
use crate::project_status_model::{StatusReading, StatusRecordCount};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProjectStatusAlertSurfaceDocuments {
    pub(crate) rules: Option<Value>,
    pub(crate) contact_points: Option<Value>,
    pub(crate) mute_timings: Option<Value>,
    pub(crate) policies: Option<Value>,
    pub(crate) templates: Option<Value>,
}

fn request_json_best_effort_with_request<F>(
    request_json: &mut F,
    path: &str,
    params: &[(String, String)],
) -> Option<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, path, params, None).ok().flatten() {
        Some(Value::Null) => None,
        other => other,
    }
}

fn request_object_list_with_request<F>(
    request_json: &mut F,
    path: &str,
    params: &[(String, String)],
    error_message: &str,
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, path, params, None)? {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| Ok(value_as_object(item, error_message)?.clone()))
            .collect(),
        Some(_) => Err(message(error_message)),
        None => Ok(Vec::new()),
    }
}

pub(crate) fn project_status_timestamp_from_object(object: &Map<String, Value>) -> Option<&str> {
    for key in ["updated", "updatedAt", "modified", "createdAt", "created"] {
        if let Some(observed_at) = object.get(key).and_then(Value::as_str) {
            let observed_at = observed_at.trim();
            if !observed_at.is_empty() {
                return Some(observed_at);
            }
        }
    }
    None
}

pub(crate) fn project_status_freshness_samples_from_value<'a>(
    source: &'static str,
    value: &'a Value,
) -> Vec<ProjectStatusFreshnessSample<'a>> {
    match value {
        Value::Array(items) => items
            .iter()
            .flat_map(|item| project_status_freshness_samples_from_value(source, item))
            .collect(),
        Value::Object(object) => project_status_timestamp_from_object(object)
            .map(|observed_at| {
                vec![ProjectStatusFreshnessSample::ObservedAtRfc3339 {
                    source,
                    observed_at,
                }]
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

pub(crate) fn project_status_freshness_samples_from_records<'a>(
    source: &'static str,
    records: &'a [Map<String, Value>],
) -> Vec<ProjectStatusFreshnessSample<'a>> {
    records
        .iter()
        .filter_map(|record| {
            project_status_timestamp_from_object(record).map(|observed_at| {
                ProjectStatusFreshnessSample::ObservedAtRfc3339 {
                    source,
                    observed_at,
                }
            })
        })
        .collect()
}

pub(crate) fn dashboard_project_status_freshness_samples<'a>(
    inputs: &'a LiveDashboardProjectStatusInputs,
) -> Vec<ProjectStatusFreshnessSample<'a>> {
    let mut freshness_samples = project_status_freshness_samples_from_records(
        "dashboard-search",
        &inputs.dashboard_summaries,
    );
    freshness_samples.extend(project_status_freshness_samples_from_records(
        "datasource-list",
        &inputs.datasources,
    ));
    freshness_samples
}

pub(crate) fn alert_project_status_freshness_samples<'a>(
    documents: &'a ProjectStatusAlertSurfaceDocuments,
) -> Vec<ProjectStatusFreshnessSample<'a>> {
    let mut freshness_samples = Vec::new();
    if let Some(document) = documents.rules.as_ref() {
        freshness_samples.extend(project_status_freshness_samples_from_value(
            "alert-rules",
            document,
        ));
    }
    if let Some(document) = documents.contact_points.as_ref() {
        freshness_samples.extend(project_status_freshness_samples_from_value(
            "alert-contact-points",
            document,
        ));
    }
    if let Some(document) = documents.mute_timings.as_ref() {
        freshness_samples.extend(project_status_freshness_samples_from_value(
            "alert-mute-timings",
            document,
        ));
    }
    if let Some(document) = documents.policies.as_ref() {
        freshness_samples.extend(project_status_freshness_samples_from_value(
            "alert-policies",
            document,
        ));
    }
    if let Some(document) = documents.templates.as_ref() {
        freshness_samples.extend(project_status_freshness_samples_from_value(
            "alert-templates",
            document,
        ));
    }
    freshness_samples
}

#[cfg(test)]
fn stamp_live_domain_freshness(
    mut domain: ProjectDomainStatus,
    samples: &[ProjectStatusFreshnessSample<'_>],
) -> ProjectDomainStatus {
    domain.freshness = if samples.is_empty() {
        build_live_project_status_freshness_from_source_count(domain.source_kinds.len())
    } else {
        build_live_project_status_freshness_from_samples(samples)
    };
    domain
}

#[cfg(test)]
pub(crate) fn build_live_dashboard_status_with_request<F>(
    mut request_json: F,
) -> ProjectDomainStatus
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match collect_live_dashboard_project_status_inputs_with_request(&mut request_json, 500) {
        Ok(inputs) => {
            let status = build_live_dashboard_domain_status_from_inputs(&inputs);
            let mut freshness_samples = project_status_freshness_samples_from_records(
                "dashboard-search",
                &inputs.dashboard_summaries,
            );
            freshness_samples.extend(project_status_freshness_samples_from_records(
                "datasource-list",
                &inputs.datasources,
            ));
            let dashboard_version_timestamp = if freshness_samples.is_empty() {
                latest_dashboard_version_timestamp_with_request(
                    &mut request_json,
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
        Err(_) => StatusReading {
            id: "dashboard".to_string(),
            scope: "live".to_string(),
            mode: "live-dashboard-read".to_string(),
            status: PROJECT_STATUS_PARTIAL.to_string(),
            reason_code: "live-read-failed".to_string(),
            primary_count: 0,
            source_kinds: vec!["live-dashboard-search".to_string()],
            signal_keys: vec!["live.dashboardCount".to_string()],
            blockers: vec![StatusRecordCount::new(
                "live-read-failed",
                1,
                "live.dashboardCount",
            )],
            warnings: Vec::new(),
            next_actions: vec![
                "restore dashboard search access, then re-run live status".to_string()
            ],
            freshness: Default::default(),
        }
        .into_project_domain_status(),
    }
}

#[cfg(test)]
fn first_dashboard_uid(dashboard_summaries: &[Map<String, Value>]) -> Option<&str> {
    dashboard_summaries.iter().find_map(|summary| {
        summary
            .get("uid")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

pub(crate) fn list_visible_orgs(client: &JsonHttpClient) -> Result<Vec<Map<String, Value>>> {
    AccessResourceClient::new(client).list_orgs()
}

pub(crate) fn collect_live_dashboard_project_status_inputs(
    client: &JsonHttpClient,
    page_size: usize,
) -> Result<LiveDashboardProjectStatusInputs> {
    let dashboard_client = DashboardResourceClient::new(client);
    let datasource_client = DatasourceResourceClient::new(client);
    Ok(LiveDashboardProjectStatusInputs {
        dashboard_summaries: dashboard_client.list_dashboard_summaries(page_size)?,
        datasources: datasource_client.list_datasources()?,
    })
}

pub(crate) fn collect_live_dashboard_project_status_inputs_with_request<F>(
    request_json: &mut F,
    page_size: usize,
) -> Result<LiveDashboardProjectStatusInputs>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut dashboard_summaries = Vec::new();
    let mut seen_uids = std::collections::BTreeSet::new();
    let mut page = 1usize;
    loop {
        let params = vec![
            ("type".to_string(), "dash-db".to_string()),
            ("limit".to_string(), page_size.to_string()),
            ("page".to_string(), page.to_string()),
        ];
        let response = request_json(Method::GET, "/api/search", &params, None)?;
        let batch = match response {
            Some(Value::Array(batch)) => batch,
            Some(_) => return Err(message("Unexpected search response from Grafana.")),
            None => Vec::new(),
        };
        if batch.is_empty() {
            break;
        }
        let batch_len = batch.len();
        for item in batch {
            let object =
                value_as_object(&item, "Unexpected dashboard summary payload from Grafana.")?;
            let uid = object
                .get("uid")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            if uid.is_empty() || seen_uids.contains(uid) {
                continue;
            }
            seen_uids.insert(uid.to_string());
            dashboard_summaries.push(object.clone());
        }
        if batch_len < page_size {
            break;
        }
        page += 1;
    }

    let datasources = match request_json(Method::GET, "/api/datasources", &[], None)? {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                Ok(value_as_object(item, "Unexpected datasource payload from Grafana.")?.clone())
            })
            .collect::<Result<Vec<Map<String, Value>>>>()?,
        Some(_) => return Err(message("Unexpected datasource list response from Grafana.")),
        None => Vec::new(),
    };

    Ok(LiveDashboardProjectStatusInputs {
        dashboard_summaries,
        datasources,
    })
}

pub(crate) fn list_visible_orgs_with_request<F>(
    request_json: &mut F,
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object_list_with_request(
        request_json,
        "/api/orgs",
        &[],
        "Unexpected /api/orgs payload from Grafana.",
    )
}

#[cfg(test)]
pub(crate) fn fetch_current_org(client: &JsonHttpClient) -> Result<Map<String, Value>> {
    AccessResourceClient::new(client).fetch_current_org()
}

pub(crate) fn fetch_current_org_with_request<F>(request_json: &mut F) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, "/api/org", &[], None)? {
        Some(value) => {
            let object = value_as_object(&value, "Unexpected current-org payload from Grafana.")?;
            Ok(object.clone())
        }
        None => Err(message("Grafana did not return current-org metadata.")),
    }
}

pub(crate) fn latest_dashboard_version_timestamp(
    client: &JsonHttpClient,
    dashboard_summaries: &[Map<String, Value>],
) -> Option<String> {
    DashboardResourceClient::new(client).latest_dashboard_version_timestamp(dashboard_summaries)
}

#[cfg(test)]
pub(crate) fn latest_dashboard_version_timestamp_with_request<F>(
    mut request_json: F,
    dashboard_summaries: &[Map<String, Value>],
) -> Option<String>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let uid = first_dashboard_uid(dashboard_summaries)?;
    let path = format!("/api/dashboards/uid/{uid}/versions");
    let params = vec![("limit".to_string(), "1".to_string())];
    let response = request_json(Method::GET, &path, &params, None)
        .ok()
        .flatten()?;
    let versions = match response {
        Value::Array(items) => items,
        Value::Object(object) => object
            .get("versions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    versions
        .first()
        .and_then(Value::as_object)
        .and_then(project_status_timestamp_from_object)
        .map(str::to_string)
}

pub(crate) fn load_alert_surface_documents(
    client: &JsonHttpClient,
) -> ProjectStatusAlertSurfaceDocuments {
    let alerting = AlertingResourceClient::new(client);
    ProjectStatusAlertSurfaceDocuments {
        rules: request_json_best_effort_with_request(
            &mut |method, path, params, payload| {
                alerting.request_json(method, path, params, payload)
            },
            "/api/v1/provisioning/alert-rules",
            &[],
        ),
        contact_points: request_json_best_effort_with_request(
            &mut |method, path, params, payload| {
                alerting.request_json(method, path, params, payload)
            },
            "/api/v1/provisioning/contact-points",
            &[],
        ),
        mute_timings: request_json_best_effort_with_request(
            &mut |method, path, params, payload| {
                alerting.request_json(method, path, params, payload)
            },
            "/api/v1/provisioning/mute-timings",
            &[],
        ),
        policies: request_json_best_effort_with_request(
            &mut |method, path, params, payload| {
                alerting.request_json(method, path, params, payload)
            },
            "/api/v1/provisioning/policies",
            &[],
        ),
        templates: request_json_best_effort_with_request(
            &mut |method, path, params, payload| {
                alerting.request_json(method, path, params, payload)
            },
            "/api/v1/provisioning/templates",
            &[],
        ),
    }
}

#[cfg(test)]
pub(crate) fn load_alert_surface_documents_with_request<F>(
    request_json: &mut F,
) -> ProjectStatusAlertSurfaceDocuments
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    ProjectStatusAlertSurfaceDocuments {
        rules: request_json_best_effort_with_request(
            request_json,
            "/api/v1/provisioning/alert-rules",
            &[],
        ),
        contact_points: request_json_best_effort_with_request(
            request_json,
            "/api/v1/provisioning/contact-points",
            &[],
        ),
        mute_timings: request_json_best_effort_with_request(
            request_json,
            "/api/v1/provisioning/mute-timings",
            &[],
        ),
        policies: request_json_best_effort_with_request(
            request_json,
            "/api/v1/provisioning/policies",
            &[],
        ),
        templates: request_json_best_effort_with_request(
            request_json,
            "/api/v1/provisioning/templates",
            &[],
        ),
    }
}

#[cfg(test)]
#[path = "project_status_live_tests.rs"]
mod tests;
