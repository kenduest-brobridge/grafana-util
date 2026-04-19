//! Export dashboards into staged artifact trees.
//!
//! Responsibilities:
//! - Resolve dashboards from live/org metadata and normalize folder hierarchy.
//! - Render staged export variants (`raw`, `prompt`, `provisioning`) using shared helpers.
//! - Build dashboard export metadata/index artifacts and stream operator-facing progress/output.
use reqwest::Method;
use serde_json::Value;

use crate::common::Result;
use crate::grafana_api::DashboardResourceClient;
use crate::http::JsonHttpClient;

use super::list::{list_orgs_with_request, org_id_value};
use super::{
    build_api_client, build_http_client, build_http_client_for_org,
    build_http_client_for_org_from_api, ExportArgs,
};
#[path = "export_paths.rs"]
mod export_paths;
#[path = "export_provisioning.rs"]
mod export_provisioning;
#[path = "export_render.rs"]
mod export_render;
#[path = "export_root_bundle.rs"]
mod export_root_bundle;
#[path = "export_scope.rs"]
mod export_scope;
#[path = "export_support.rs"]
mod export_support;

pub use self::export_paths::{build_export_variant_dirs, build_output_path};
use self::export_provisioning::{build_provisioning_config, write_yaml_document};
#[allow(unused_imports)]
pub(crate) use self::export_render::{format_export_progress_line, format_export_verbose_line};
use self::export_root_bundle::write_all_orgs_root_export_bundle;
#[allow(unused_imports)]
pub(crate) use self::export_scope::{export_dashboards_in_scope_with_request, ScopeExportResult};
use self::export_support::{
    build_all_orgs_output_dir, build_permission_bundle_document, build_used_datasource_summaries,
    collect_library_panel_exports_with_request, collect_permission_export_documents,
};

pub(crate) fn export_dashboards_with_request<F>(
    mut request_json: F,
    args: &ExportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if args.all_orgs {
        let mut total = 0usize;
        let mut root_items = Vec::new();
        let mut root_folders = Vec::new();
        let mut org_summaries = Vec::new();
        for org in list_orgs_with_request(&mut request_json)? {
            let org_id = org_id_value(&org)?;
            let scope_result = export_dashboards_in_scope_with_request(
                &mut request_json,
                args,
                Some(&org),
                Some(org_id),
            )?;
            total += scope_result.exported_count;
            if let Some(root_index) = scope_result.root_index {
                root_items.extend(root_index.items);
                root_folders.extend(root_index.folders);
            }
            if let Some(org_summary) = scope_result.org_summary {
                org_summaries.push(org_summary);
            }
        }
        if !args.dry_run && !root_items.is_empty() {
            write_all_orgs_root_export_bundle(
                &args.output_dir,
                &root_items,
                &root_folders,
                org_summaries,
                &args.common.url,
                args.common.profile.as_deref(),
            )?;
        }
        Ok(total)
    } else {
        Ok(
            export_dashboards_in_scope_with_request(&mut request_json, args, None, args.org_id)?
                .exported_count,
        )
    }
}

pub fn export_dashboards_with_client(client: &JsonHttpClient, args: &ExportArgs) -> Result<usize> {
    export_dashboards_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

pub(crate) fn export_dashboards_with_org_clients(args: &ExportArgs) -> Result<usize> {
    if args.all_orgs {
        let admin_api = build_api_client(&args.common)?;
        let admin_dashboard = DashboardResourceClient::new(admin_api.http_client());
        let mut total = 0usize;
        let mut root_items = Vec::new();
        let mut root_folders = Vec::new();
        let mut org_summaries = Vec::new();
        for org in admin_dashboard.list_orgs()? {
            let org_id = org_id_value(&org)?;
            let org_client = build_http_client_for_org_from_api(&admin_api, org_id)?;
            let scope_result = export_dashboards_in_scope_with_request(
                &mut |method, path, params, payload| {
                    org_client.request_json(method, path, params, payload)
                },
                args,
                Some(&org),
                None,
            )?;
            total += scope_result.exported_count;
            if let Some(root_index) = scope_result.root_index {
                root_items.extend(root_index.items);
                root_folders.extend(root_index.folders);
            }
            if let Some(org_summary) = scope_result.org_summary {
                org_summaries.push(org_summary);
            }
        }
        if !args.dry_run && !root_items.is_empty() {
            write_all_orgs_root_export_bundle(
                &args.output_dir,
                &root_items,
                &root_folders,
                org_summaries,
                &args.common.url,
                args.common.profile.as_deref(),
            )?;
        }
        Ok(total)
    } else if let Some(org_id) = args.org_id {
        let org_client = build_http_client_for_org(&args.common, org_id)?;
        Ok(export_dashboards_in_scope_with_request(
            &mut |method, path, params, payload| {
                org_client.request_json(method, path, params, payload)
            },
            args,
            None,
            None,
        )?
        .exported_count)
    } else {
        let client = build_http_client(&args.common)?;
        let dashboard = DashboardResourceClient::new(&client);
        let current_org = dashboard.fetch_current_org()?;
        Ok(export_dashboards_in_scope_with_request(
            &mut |method, path, params, payload| client.request_json(method, path, params, payload),
            args,
            Some(&current_org),
            None,
        )?
        .exported_count)
    }
}

#[cfg(test)]
mod export_tests {
    use super::*;
    use crate::dashboard::CommonCliArgs;
    use crate::dashboard::{build_root_export_index, build_variant_index};
    use serde_json::json;
    use serde_json::Map;
    use std::fs;
    use tempfile::tempdir;

    fn make_common_args(base_url: String) -> CommonCliArgs {
        CommonCliArgs {
            color: crate::common::CliColorChoice::Auto,
            profile: None,
            url: base_url,
            api_token: Some("token".to_string()),
            username: None,
            password: None,
            prompt_password: false,
            prompt_token: false,
            timeout: 30,
            verify_ssl: false,
        }
    }

    #[test]
    fn dashboard_export_indexes_include_folder_identity_metadata() {
        let mut summary = Map::new();
        summary.insert("uid".to_string(), json!("cpu-main"));
        summary.insert("title".to_string(), json!("CPU Main"));
        summary.insert("folderTitle".to_string(), json!("Infra"));
        summary.insert("folderUid".to_string(), json!("infra"));
        summary.insert("orgName".to_string(), json!("Main Org"));
        summary.insert("orgId".to_string(), json!("1"));
        let mut item = super::super::build_dashboard_index_item(&summary, "cpu-main");
        item.folder_path = "Platform / Team / Infra".to_string();
        item.raw_path = Some("/tmp/export/raw/Platform/Team/Infra/CPU.json".to_string());

        let variant_index = build_variant_index(
            &[item.clone()],
            |item| item.raw_path.as_deref(),
            "grafana-web-import-preserve-uid",
        );
        let root_index = build_root_export_index(&[item], None, None, None, &[]);

        assert_eq!(variant_index[0].folder_uid, "infra");
        assert_eq!(variant_index[0].folder_path, "Platform / Team / Infra");
        assert_eq!(root_index.items[0].folder_uid, "infra");
        assert_eq!(root_index.items[0].folder_path, "Platform / Team / Infra");
    }

    #[test]
    fn export_dashboards_with_request_includes_live_library_panel_elements_in_prompt_variant() {
        let temp = tempdir().unwrap();
        let args = ExportArgs {
            common: make_common_args("http://127.0.0.1:3000".to_string()),
            output_dir: temp.path().join("dashboards"),
            run: None,
            run_id: None,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            flat: false,
            overwrite: true,
            without_dashboard_raw: true,
            without_dashboard_prompt: false,
            without_dashboard_provisioning: true,
            include_history: false,
            provisioning_provider_name: "grafana-utils-dashboards".to_string(),
            provisioning_provider_org_id: None,
            provisioning_provider_path: None,
            provisioning_provider_disable_deletion: false,
            provisioning_provider_allow_ui_updates: false,
            provisioning_provider_update_interval_seconds: 30,
            dry_run: false,
            progress: false,
            verbose: false,
        };

        let count = export_dashboards_with_request(
            |method, path, params, payload| {
                let scoped_org = params
                    .iter()
                    .find(|(key, _)| key == "orgId")
                    .map(|(_, value)| value.as_str());
                match (method.as_str(), path, scoped_org) {
                    ("GET", "/api/org", None) => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
                    ("GET", "/api/search", None) => Ok(Some(json!([
                        {
                            "uid": "cpu-main",
                            "title": "CPU Main",
                            "folderTitle": "General"
                        }
                    ]))),
                    ("GET", "/api/datasources", None) => Ok(Some(json!([
                        {
                            "uid": "prom-main",
                            "name": "Prometheus Main",
                            "type": "prometheus",
                            "url": "http://prometheus:9090",
                            "access": "proxy",
                            "isDefault": true
                        }
                    ]))),
                    ("GET", "/api/dashboards/uid/cpu-main", None) => Ok(Some(json!({
                        "dashboard": {
                            "id": 7,
                            "uid": "cpu-main",
                            "title": "CPU Main",
                            "panels": [
                                {
                                    "id": 1,
                                    "type": "graph",
                                    "datasource": {"uid": "prom-main", "type": "prometheus"},
                                    "libraryPanel": {"uid": "shared-panel", "name": "Shared Panel"},
                                    "targets": []
                                }
                            ]
                        }
                    }))),
                    ("GET", "/api/library-elements/shared-panel", None) => Ok(Some(json!({
                        "result": {
                            "uid": "shared-panel",
                            "name": "Shared Panel",
                            "kind": 1,
                            "type": "graph",
                            "model": {
                                "type": "graph",
                                "datasource": {"uid": "prom-main", "type": "prometheus"}
                            }
                        }
                    }))),
                    _ => Err(crate::common::message(format!(
                        "unexpected request path={path} payload={payload:?}"
                    ))),
                }
            },
            &args,
        )
        .unwrap();

        assert_eq!(count, 1);
        let prompt_path = args
            .output_dir
            .join("prompt/General/CPU_Main__cpu-main.json");
        assert!(prompt_path.is_file());
        let prompt: Value =
            serde_json::from_str(&fs::read_to_string(prompt_path).unwrap()).unwrap();
        assert_eq!(
            prompt["__elements"]["shared-panel"]["name"],
            json!("Shared Panel")
        );
        assert_eq!(prompt["__elements"]["shared-panel"]["kind"], json!(1));
        assert_eq!(prompt["__elements"]["shared-panel"]["type"], json!("graph"));
        assert!(
            prompt["__elements"]["shared-panel"]["model"]["datasource"]["uid"]
                .as_str()
                .is_some_and(|value| value.starts_with("${DS_"))
        );
    }
}
