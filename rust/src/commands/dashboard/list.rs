//! Read model for dashboard and datasource listing.
//! This module keeps orchestration and org/path enrichment while summary rendering lives in a
//! focused sibling helper.
use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

use crate::common::{message, render_json_value, string_field, value_as_object, Result};
use crate::grafana_api::{DashboardResourceClient, DatasourceResourceClient};
use crate::http::JsonHttpClient;
use crate::tabular_output::render_yaml;

use super::{
    build_api_client, build_datasource_catalog, build_folder_path, build_http_client_for_org,
    build_http_client_for_org_from_api, fetch_dashboard_with_request,
    fetch_folder_if_exists_with_request, list_dashboard_summaries_with_request,
    list_datasources_with_request, ListArgs, DEFAULT_DASHBOARD_TITLE, DEFAULT_FOLDER_TITLE,
    DEFAULT_FOLDER_UID, DEFAULT_UNKNOWN_UID,
};

#[path = "list_render.rs"]
mod list_render;
#[path = "list/source_metadata.rs"]
mod source_metadata;

#[allow(unused_imports)]
pub(crate) use list_render::{
    format_dashboard_summary_line, render_dashboard_summary_csv, render_dashboard_summary_json,
    render_dashboard_summary_table, render_dashboard_summary_text,
};
pub(crate) use source_metadata::collect_dashboard_source_metadata;

struct DashboardListResourceClients<'a> {
    dashboard: DashboardResourceClient<'a>,
    datasource: DatasourceResourceClient<'a>,
}

impl<'a> DashboardListResourceClients<'a> {
    fn new(client: &'a JsonHttpClient) -> Self {
        Self {
            dashboard: DashboardResourceClient::new(client),
            datasource: DatasourceResourceClient::new(client),
        }
    }

    fn list_dashboard_summaries(&self, page_size: usize) -> Result<Vec<Map<String, Value>>> {
        self.dashboard.list_dashboard_summaries(page_size)
    }

    fn attach_dashboard_folder_paths(
        &self,
        summaries: &[Map<String, Value>],
    ) -> Result<Vec<Map<String, Value>>> {
        let mut folder_paths = BTreeMap::new();
        for summary in summaries {
            let folder_uid = string_field(summary, "folderUid", "");
            let folder_title = string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE);
            if folder_uid.is_empty() || folder_paths.contains_key(&folder_uid) {
                continue;
            }
            let folder = self.dashboard.fetch_folder_if_exists(&folder_uid)?;
            let folder_path = match folder {
                Some(folder) => build_folder_path(&folder, &folder_title),
                None => folder_title,
            };
            folder_paths.insert(folder_uid, folder_path);
        }

        Ok(summaries
            .iter()
            .map(|summary| {
                let mut item = summary.clone();
                let folder_uid = string_field(summary, "folderUid", "");
                let folder_title = string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE);
                item.insert(
                    "folderPath".to_string(),
                    Value::String(
                        folder_paths
                            .get(&folder_uid)
                            .cloned()
                            .unwrap_or(folder_title),
                    ),
                );
                item
            })
            .collect())
    }

    fn attach_dashboard_sources(
        &self,
        summaries: &[Map<String, Value>],
    ) -> Result<Vec<Map<String, Value>>> {
        let datasource_catalog = build_datasource_catalog(&self.datasource.list_datasources()?);
        summaries
            .iter()
            .map(|summary| {
                let uid = string_field(summary, "uid", "");
                let mut item = summary.clone();
                if uid.is_empty() {
                    item.insert("sources".to_string(), Value::Array(Vec::new()));
                    item.insert("sourceUids".to_string(), Value::Array(Vec::new()));
                    return Ok(item);
                }
                let payload = self.dashboard.fetch_dashboard(&uid)?;
                let (sources, source_uids) =
                    collect_dashboard_source_metadata(&payload, &datasource_catalog)?;
                item.insert(
                    "sources".to_string(),
                    Value::Array(sources.into_iter().map(Value::String).collect()),
                );
                item.insert(
                    "sourceUids".to_string(),
                    Value::Array(source_uids.into_iter().map(Value::String).collect()),
                );
                Ok(item)
            })
            .collect()
    }
}

/// attach dashboard folder paths with request.
pub(crate) fn attach_dashboard_folder_paths_with_request<F>(
    mut request_json: F,
    summaries: &[Map<String, Value>],
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut folder_paths = BTreeMap::new();
    for summary in summaries {
        let folder_uid = string_field(summary, "folderUid", "");
        let folder_title = string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE);
        if folder_uid.is_empty() || folder_paths.contains_key(&folder_uid) {
            continue;
        }
        let folder = fetch_folder_if_exists_with_request(&mut request_json, &folder_uid)?;
        let folder_path = match folder {
            Some(folder) => build_folder_path(&folder, &folder_title),
            None => folder_title,
        };
        folder_paths.insert(folder_uid, folder_path);
    }

    Ok(summaries
        .iter()
        .map(|summary| {
            let mut item = summary.clone();
            let folder_uid = string_field(summary, "folderUid", "");
            let folder_title = string_field(summary, "folderTitle", DEFAULT_FOLDER_TITLE);
            item.insert(
                "folderPath".to_string(),
                Value::String(
                    folder_paths
                        .get(&folder_uid)
                        .cloned()
                        .unwrap_or(folder_title),
                ),
            );
            item
        })
        .collect())
}

/// fetch current org with request.
pub(crate) fn fetch_current_org_with_request<F>(mut request_json: F) -> Result<Map<String, Value>>
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

/// Purpose: implementation note.
pub(crate) fn list_orgs_with_request<F>(mut request_json: F) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, "/api/orgs", &[], None)? {
        Some(Value::Array(values)) => values
            .into_iter()
            .map(|value| {
                let object = value_as_object(&value, "Unexpected org list payload from Grafana.")?;
                Ok(object.clone())
            })
            .collect(),
        Some(_) => Err(message("Unexpected org list payload from Grafana.")),
        None => Ok(Vec::new()),
    }
}

/// attach dashboard org metadata.
pub(crate) fn attach_dashboard_org_metadata(
    summaries: &[Map<String, Value>],
    org: &Map<String, Value>,
) -> Vec<Map<String, Value>> {
    let org_name = string_field(org, "name", "");
    let org_id = org.get("id").cloned().unwrap_or(Value::Null);
    summaries
        .iter()
        .map(|summary| {
            let mut item = summary.clone();
            item.insert("orgName".to_string(), Value::String(org_name.clone()));
            item.insert("orgId".to_string(), org_id.clone());
            item
        })
        .collect()
}

/// org id value.
pub(crate) fn org_id_value(org: &Map<String, Value>) -> Result<i64> {
    org.get("id")
        .and_then(Value::as_i64)
        .ok_or_else(|| message("Grafana org payload did not include a usable id."))
}

fn dashboard_list_needs_sources(args: &ListArgs) -> bool {
    args.show_sources
        || args.text
        || args.json
        || args.yaml
        || args
            .output_columns
            .iter()
            .any(|column| matches!(column.as_str(), "sources" | "source_uids"))
}

fn attach_dashboard_sources_with_request<F>(
    mut request_json: F,
    summaries: &[Map<String, Value>],
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let datasource_catalog =
        build_datasource_catalog(&list_datasources_with_request(&mut request_json)?);
    summaries
        .iter()
        .map(|summary| {
            let uid = string_field(summary, "uid", "");
            let mut item = summary.clone();
            if uid.is_empty() {
                item.insert("sources".to_string(), Value::Array(Vec::new()));
                item.insert("sourceUids".to_string(), Value::Array(Vec::new()));
                return Ok(item);
            }
            let payload = fetch_dashboard_with_request(&mut request_json, &uid)?;
            let (sources, source_uids) =
                collect_dashboard_source_metadata(&payload, &datasource_catalog)?;
            item.insert(
                "sources".to_string(),
                Value::Array(sources.into_iter().map(Value::String).collect()),
            );
            item.insert(
                "sourceUids".to_string(),
                Value::Array(source_uids.into_iter().map(Value::String).collect()),
            );
            Ok(item)
        })
        .collect()
}

fn collect_list_dashboards_with_client(
    client: &JsonHttpClient,
    args: &ListArgs,
    org: Option<&Map<String, Value>>,
) -> Result<Vec<Map<String, Value>>> {
    let resources = DashboardListResourceClients::new(client);
    let dashboard_summaries = resources.list_dashboard_summaries(args.page_size)?;
    let current_org = match org {
        Some(org) => org.clone(),
        None => resources.dashboard.fetch_current_org()?,
    };
    let summaries = resources.attach_dashboard_folder_paths(&dashboard_summaries)?;
    let summaries = attach_dashboard_org_metadata(&summaries, &current_org);
    let summaries = if dashboard_list_needs_sources(args) && !summaries.is_empty() {
        resources.attach_dashboard_sources(&summaries)?
    } else {
        summaries
    };
    Ok(summaries)
}

pub(crate) fn collect_list_dashboards_with_request<F>(
    request_json: &mut F,
    args: &ListArgs,
    org: Option<&Map<String, Value>>,
    org_id_override: Option<i64>,
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut scoped_request = |method: Method,
                              path: &str,
                              params: &[(String, String)],
                              payload: Option<&Value>|
     -> Result<Option<Value>> {
        let mut scoped_params = params.to_vec();
        if let Some(org_id) = org_id_override {
            scoped_params.push(("orgId".to_string(), org_id.to_string()));
        }
        request_json(method, path, &scoped_params, payload)
    };

    let dashboard_summaries =
        list_dashboard_summaries_with_request(&mut scoped_request, args.page_size)?;
    let current_org = match org {
        Some(org) => org.clone(),
        None => fetch_current_org_with_request(&mut scoped_request)?,
    };
    let summaries =
        attach_dashboard_folder_paths_with_request(&mut scoped_request, &dashboard_summaries)?;
    let summaries = attach_dashboard_org_metadata(&summaries, &current_org);
    let summaries = if dashboard_list_needs_sources(args) && !summaries.is_empty() {
        attach_dashboard_sources_with_request(&mut scoped_request, &summaries)?
    } else {
        summaries
    };
    Ok(summaries)
}

fn render_dashboard_list_output(
    summaries: &[Map<String, Value>],
    args: &ListArgs,
) -> Result<usize> {
    if args.json {
        println!(
            "{}",
            render_json_value(&render_dashboard_summary_json(
                summaries,
                &args.output_columns,
            ))?
        );
    } else if args.yaml {
        print!(
            "{}",
            render_yaml(&render_dashboard_summary_json(
                summaries,
                &args.output_columns
            ))?
        );
    } else if args.csv {
        for line in render_dashboard_summary_csv(summaries, &args.output_columns) {
            println!("{line}");
        }
    } else if args.text {
        for line in render_dashboard_summary_text(summaries) {
            println!("{line}");
        }
    } else {
        for line in render_dashboard_summary_table(summaries, &args.output_columns, !args.no_header)
        {
            println!("{line}");
        }
    }
    if !args.csv && !args.json && !args.yaml {
        println!();
        println!("Listed {} dashboard(s).", summaries.len());
    }
    Ok(summaries.len())
}

/// Purpose: implementation note.
#[allow(dead_code)]
pub(crate) fn list_dashboards_with_request<F>(mut request_json: F, args: &ListArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut summaries = Vec::new();
    if args.all_orgs {
        for org in list_orgs_with_request(&mut request_json)? {
            let org_id = org_id_value(&org)?;
            let mut scoped = collect_list_dashboards_with_request(
                &mut request_json,
                args,
                Some(&org),
                Some(org_id),
            )?;
            summaries.append(&mut scoped);
        }
    } else {
        summaries =
            collect_list_dashboards_with_request(&mut request_json, args, None, args.org_id)?;
    }
    render_dashboard_list_output(&summaries, args)
}

/// Purpose: implementation note.
///
/// Args: see function signature.
/// Returns: see implementation.
pub fn list_dashboards_with_client(client: &JsonHttpClient, args: &ListArgs) -> Result<usize> {
    let summaries = collect_list_dashboards_with_client(client, args, None)?;
    render_dashboard_list_output(&summaries, args)
}

/// Purpose: implementation note.
pub(crate) fn list_dashboards_with_org_clients(args: &ListArgs) -> Result<usize> {
    let admin_api = build_api_client(&args.common)?;
    let admin_client = admin_api.http_client();
    let orgs = if args.all_orgs {
        DashboardResourceClient::new(admin_client).list_orgs()?
    } else {
        Vec::new()
    };
    let mut summaries = Vec::new();
    if args.all_orgs {
        for org in orgs {
            let org_id = org_id_value(&org)?;
            let org_client = build_http_client_for_org_from_api(&admin_api, org_id)?;
            let mut scoped = collect_list_dashboards_with_client(&org_client, args, Some(&org))?;
            summaries.append(&mut scoped);
        }
    } else if let Some(org_id) = args.org_id {
        let org_client = build_http_client_for_org(&args.common, org_id)?;
        summaries = collect_list_dashboards_with_client(&org_client, args, None)?;
    } else {
        summaries = collect_list_dashboards_with_client(admin_client, args, None)?;
    }
    render_dashboard_list_output(&summaries, args)
}
