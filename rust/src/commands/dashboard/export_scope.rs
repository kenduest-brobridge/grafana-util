use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};

use super::super::history::build_dashboard_history_export_document_with_request;
use super::super::list::{
    attach_dashboard_org_metadata, collect_dashboard_source_metadata,
    fetch_current_org_with_request,
};
use super::super::prompt::build_external_export_document_with_library_panels;
use super::super::{
    build_dashboard_index_item, build_datasource_catalog, build_datasource_inventory_record,
    build_export_metadata, build_preserved_web_import_document, build_root_export_index,
    build_variant_index, collect_folder_inventory_with_request, fetch_dashboard_with_request,
    list_dashboard_summaries_with_request, list_datasources_with_request, write_dashboard,
    write_json_document, ExportArgs, ExportOrgSummary, RootExportIndex,
    DASHBOARD_PERMISSION_BUNDLE_FILENAME, DATASOURCE_INVENTORY_FILENAME, DEFAULT_UNKNOWN_UID,
    EXPORT_METADATA_FILENAME, FOLDER_INVENTORY_FILENAME, PROMPT_EXPORT_SUBDIR,
    PROVISIONING_EXPORT_SUBDIR, RAW_EXPORT_SUBDIR,
};
use super::export_paths::{
    build_export_variant_dirs, build_folder_paths_by_inventory_key, build_history_output_path,
    build_output_path, build_output_path_with_folder_path, export_folder_inventory_key,
};
use super::export_render::{format_export_progress_line, format_export_verbose_line};
use super::{
    build_all_orgs_output_dir, build_permission_bundle_document, build_provisioning_config,
    build_used_datasource_summaries, collect_library_panel_exports_with_request,
    collect_permission_export_documents, write_yaml_document,
};

pub(crate) struct ScopeExportResult {
    pub(crate) exported_count: usize,
    pub(crate) root_index: Option<RootExportIndex>,
    pub(crate) org_summary: Option<ExportOrgSummary>,
}

pub(crate) fn export_dashboards_in_scope_with_request<F>(
    request_json: &mut F,
    args: &ExportArgs,
    org: Option<&Map<String, Value>>,
    org_id_override: Option<i64>,
) -> Result<ScopeExportResult>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if args.without_dashboard_raw
        && args.without_dashboard_prompt
        && args.without_dashboard_provisioning
    {
        return Err(message(
            "Nothing to export. Remove one of --without-raw, --without-prompt, or --without-provisioning.",
        ));
    }
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
    let current_org = match org {
        Some(org) => org.clone(),
        None => fetch_current_org_with_request(&mut scoped_request)?,
    };
    let current_org_name = string_field(&current_org, "name", "org");
    let current_org_id = current_org
        .get("id")
        .map(|value| value.to_string())
        .unwrap_or_else(|| DEFAULT_UNKNOWN_UID.to_string());
    let scope_output_dir = if args.all_orgs {
        build_all_orgs_output_dir(&args.output_dir, &current_org)
    } else {
        args.output_dir.clone()
    };
    let (raw_dir, prompt_dir, provisioning_dir) = build_export_variant_dirs(&scope_output_dir);
    let history_dir = scope_output_dir.join("history");
    let provisioning_dashboards_dir = provisioning_dir.join("dashboards");
    let provisioning_config_dir = provisioning_dir.join("provisioning");
    if !args.dry_run && !args.without_dashboard_raw {
        fs::create_dir_all(&raw_dir)?;
    }
    if !args.dry_run && !args.without_dashboard_prompt {
        fs::create_dir_all(&prompt_dir)?;
    }
    if !args.dry_run && !args.without_dashboard_provisioning {
        fs::create_dir_all(&provisioning_dashboards_dir)?;
        fs::create_dir_all(&provisioning_config_dir)?;
    }
    if !args.dry_run && args.include_history {
        fs::create_dir_all(&history_dir)?;
    }
    let datasource_list = list_datasources_with_request(&mut scoped_request)?;
    let datasource_inventory = datasource_list
        .iter()
        .map(|datasource| build_datasource_inventory_record(datasource, &current_org))
        .collect::<Vec<_>>();
    let datasource_catalog = build_datasource_catalog(&datasource_list);

    let summaries = list_dashboard_summaries_with_request(&mut scoped_request, args.page_size)?;
    if summaries.is_empty() {
        return Ok(ScopeExportResult {
            exported_count: 0,
            root_index: None,
            org_summary: None,
        });
    }
    let summaries = attach_dashboard_org_metadata(&summaries, &current_org);
    let folder_inventory = collect_folder_inventory_with_request(&mut scoped_request, &summaries)?;
    let folder_paths_by_key = build_folder_paths_by_inventory_key(&folder_inventory);
    let permission_documents = if args.without_dashboard_raw || args.dry_run {
        Vec::new()
    } else {
        collect_permission_export_documents(&mut scoped_request, &summaries, &folder_inventory)?
    };

    let mut exported_count = 0;
    let mut index_items = Vec::new();
    let mut used_source_names = BTreeSet::new();
    let mut used_source_uids = BTreeSet::new();
    let total = summaries.len();
    for (index, summary) in summaries.into_iter().enumerate() {
        let uid = string_field(&summary, "uid", "");
        if uid.is_empty() {
            continue;
        }
        if args.verbose {
            // Verbose mode prints per-variant details after each write, which already
            // includes status for this iteration; suppressing progress lines avoids
            // duplicated/noisy output and keeps verbose logs ordered.
        } else if args.progress {
            println!(
                "{}",
                format_export_progress_line(index + 1, total, &uid, args.dry_run)
            );
        }
        let payload = fetch_dashboard_with_request(&mut scoped_request, &uid)?;
        let (source_names, source_uids) =
            collect_dashboard_source_metadata(&payload, &datasource_catalog)?;
        used_source_names.extend(source_names);
        used_source_uids.extend(source_uids);
        if args.include_history {
            let history_document = build_dashboard_history_export_document_with_request(
                &mut scoped_request,
                &uid,
                20,
            )?;
            let history_path = build_history_output_path(&history_dir, &uid);
            if !args.dry_run {
                write_history_document(&history_document, &history_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line("history", &uid, &history_path, args.dry_run)
                );
            }
        }
        let mut item = build_dashboard_index_item(&summary, &uid);
        let folder_path = export_folder_inventory_key(&summary)
            .and_then(|key| folder_paths_by_key.get(&key))
            .map(String::as_str);
        item.folder_path = folder_path.unwrap_or("").to_string();
        if !args.without_dashboard_raw {
            let raw_document = build_preserved_web_import_document(&payload)?;
            let raw_path =
                build_output_path_with_folder_path(&raw_dir, &summary, folder_path, args.flat);
            if !args.dry_run {
                write_dashboard(&raw_document, &raw_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line("raw", &uid, &raw_path, args.dry_run)
                );
            }
            item.raw_path = Some(raw_path.display().to_string());
        }
        if !args.without_dashboard_prompt {
            let (library_panel_exports, library_panel_warnings) =
                collect_library_panel_exports_with_request(&mut scoped_request, &payload)?;
            for warning in library_panel_warnings {
                eprintln!("Dashboard export prompt warning: {warning}");
            }
            let prompt_document = build_external_export_document_with_library_panels(
                &payload,
                &datasource_catalog,
                Some(&library_panel_exports),
            )?;
            let prompt_path =
                build_output_path_with_folder_path(&prompt_dir, &summary, folder_path, args.flat);
            if !args.dry_run {
                write_dashboard(&prompt_document, &prompt_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line("prompt", &uid, &prompt_path, args.dry_run)
                );
            }
            item.prompt_path = Some(prompt_path.display().to_string());
        }
        if !args.without_dashboard_provisioning {
            let provisioning_document = build_preserved_web_import_document(&payload)?;
            let provisioning_path =
                build_output_path(&provisioning_dashboards_dir, &summary, false);
            if !args.dry_run {
                write_dashboard(&provisioning_document, &provisioning_path, args.overwrite)?;
            }
            if args.verbose {
                println!(
                    "{}",
                    format_export_verbose_line(
                        "provisioning",
                        &uid,
                        &provisioning_path,
                        args.dry_run
                    )
                );
            }
            item.provisioning_path = Some(provisioning_path.display().to_string());
        }
        exported_count += 1;
        index_items.push(item);
    }

    let mut raw_index_path = None;
    if !args.without_dashboard_raw {
        let index_path = raw_dir.join("index.json");
        let metadata_path = raw_dir.join(EXPORT_METADATA_FILENAME);
        let folder_inventory_path = raw_dir.join(FOLDER_INVENTORY_FILENAME);
        let datasource_inventory_path = raw_dir.join(DATASOURCE_INVENTORY_FILENAME);
        let permission_bundle_path = raw_dir.join(DASHBOARD_PERMISSION_BUNDLE_FILENAME);
        if !args.dry_run {
            write_json_document(
                &build_variant_index(
                    &index_items,
                    |item| item.raw_path.as_deref(),
                    "grafana-web-import-preserve-uid",
                ),
                &index_path,
            )?;
            write_json_document(
                &build_export_metadata(
                    RAW_EXPORT_SUBDIR,
                    index_items
                        .iter()
                        .filter(|item| item.raw_path.is_some())
                        .count(),
                    Some("grafana-web-import-preserve-uid"),
                    Some(FOLDER_INVENTORY_FILENAME),
                    Some(DATASOURCE_INVENTORY_FILENAME),
                    Some(DASHBOARD_PERMISSION_BUNDLE_FILENAME),
                    Some(&current_org_name),
                    Some(&current_org_id),
                    None,
                    "live",
                    Some(&args.common.url),
                    None,
                    args.common.profile.as_deref(),
                    raw_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
            write_json_document(&folder_inventory, &folder_inventory_path)?;
            write_json_document(&datasource_inventory, &datasource_inventory_path)?;
            write_json_document(
                &build_permission_bundle_document(&permission_documents),
                &permission_bundle_path,
            )?;
        }
        raw_index_path = Some(index_path);
    }
    let mut prompt_index_path = None;
    if !args.without_dashboard_prompt {
        let index_path = prompt_dir.join("index.json");
        let metadata_path = prompt_dir.join(EXPORT_METADATA_FILENAME);
        if !args.dry_run {
            write_json_document(
                &build_variant_index(
                    &index_items,
                    |item| item.prompt_path.as_deref(),
                    "grafana-web-import-with-datasource-inputs",
                ),
                &index_path,
            )?;
            write_json_document(
                &build_export_metadata(
                    PROMPT_EXPORT_SUBDIR,
                    index_items
                        .iter()
                        .filter(|item| item.prompt_path.is_some())
                        .count(),
                    Some("grafana-web-import-with-datasource-inputs"),
                    None,
                    None,
                    None,
                    Some(&current_org_name),
                    Some(&current_org_id),
                    None,
                    "live",
                    Some(&args.common.url),
                    None,
                    args.common.profile.as_deref(),
                    prompt_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
        }
        prompt_index_path = Some(index_path);
    }
    let mut provisioning_index_path = None;
    if !args.without_dashboard_provisioning {
        let index_path = provisioning_dir.join("index.json");
        let metadata_path = provisioning_dir.join(EXPORT_METADATA_FILENAME);
        let config_path = provisioning_config_dir.join("dashboards.yaml");
        let provisioning_provider_org_id = args
            .provisioning_provider_org_id
            .unwrap_or_else(|| current_org_id.parse::<i64>().unwrap_or(1));
        let provisioning_provider_path = args
            .provisioning_provider_path
            .as_deref()
            .unwrap_or(&provisioning_dashboards_dir);
        if !args.dry_run {
            write_json_document(
                &build_variant_index(
                    &index_items,
                    |item| item.provisioning_path.as_deref(),
                    "grafana-file-provisioning-dashboard",
                ),
                &index_path,
            )?;
            write_json_document(
                &build_export_metadata(
                    PROVISIONING_EXPORT_SUBDIR,
                    index_items
                        .iter()
                        .filter(|item| item.provisioning_path.is_some())
                        .count(),
                    Some("grafana-file-provisioning-dashboard"),
                    None,
                    None,
                    None,
                    Some(&current_org_name),
                    Some(&current_org_id),
                    None,
                    "live",
                    Some(&args.common.url),
                    None,
                    args.common.profile.as_deref(),
                    provisioning_dir.as_path(),
                    &metadata_path,
                ),
                &metadata_path,
            )?;
            write_yaml_document(
                &build_provisioning_config(
                    provisioning_provider_org_id,
                    &args.provisioning_provider_name,
                    provisioning_provider_path,
                    args.provisioning_provider_disable_deletion,
                    args.provisioning_provider_allow_ui_updates,
                    args.provisioning_provider_update_interval_seconds,
                ),
                &config_path,
                args.overwrite,
            )?;
        }
        provisioning_index_path = Some(index_path);
    }
    let root_index = build_root_export_index(
        &index_items,
        raw_index_path.as_deref(),
        prompt_index_path.as_deref(),
        provisioning_index_path.as_deref(),
        &folder_inventory,
    );
    let used_datasources = build_used_datasource_summaries(
        &datasource_inventory,
        &used_source_names,
        &used_source_uids,
    );
    let org_summary = ExportOrgSummary {
        org: current_org_name.clone(),
        org_id: current_org_id.clone(),
        dashboard_count: index_items.len() as u64,
        datasource_count: Some(datasource_inventory.len() as u64),
        used_datasource_count: Some(used_datasources.len() as u64),
        used_datasources: Some(used_datasources),
        output_dir: if args.all_orgs {
            Some(scope_output_dir.display().to_string())
        } else {
            None
        },
    };
    if !args.dry_run {
        write_json_document(&root_index, &scope_output_dir.join("index.json"))?;
        write_json_document(
            &build_export_metadata(
                "root",
                index_items.len(),
                None,
                None,
                None,
                None,
                Some(&current_org_name),
                Some(&current_org_id),
                None,
                "live",
                Some(&args.common.url),
                None,
                args.common.profile.as_deref(),
                scope_output_dir.as_path(),
                &scope_output_dir.join(EXPORT_METADATA_FILENAME),
            ),
            &scope_output_dir.join(EXPORT_METADATA_FILENAME),
        )?;
    }
    Ok(ScopeExportResult {
        exported_count,
        root_index: Some(root_index),
        org_summary: Some(org_summary),
    })
}

fn write_history_document<T: serde::Serialize>(
    payload: &T,
    output_path: &Path,
    overwrite: bool,
) -> Result<()> {
    if output_path.exists() && !overwrite {
        return Err(message(format!(
            "Refusing to overwrite existing file: {}. Use --overwrite.",
            output_path.display()
        )));
    }
    write_json_document(payload, output_path)
}
