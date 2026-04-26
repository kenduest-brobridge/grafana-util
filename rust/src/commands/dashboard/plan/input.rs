//! Dashboard plan input discovery and live-state collection.

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{message, string_field, Result};

use super::super::cli_defs::{DashboardImportInputFormat, InspectExportInputType};
use super::super::files::resolve_dashboard_export_root;
use super::super::import_lookup::ImportLookupCache;
use super::super::import_validation::{
    resolve_target_org_plan_for_export_scope_with_request, ExportOrgImportScope,
};
use super::super::source_loader::resolve_dashboard_workspace_variant_dir;
use super::super::{
    build_auth_context, build_http_client, build_http_client_for_org, discover_dashboard_files,
    load_dashboard_source, load_datasource_inventory, load_export_metadata, load_folder_inventory,
    load_json_file, DEFAULT_PAGE_SIZE, PROMPT_EXPORT_SUBDIR, RAW_EXPORT_SUBDIR,
};
use super::reconcile::{build_live_dashboard_with_request, build_local_dashboard};
use super::types::{DashboardPlanInput, OrgPlanInput, PlanLiveState};
use super::{
    permissions::{collect_live_folder_permission_resources, load_folder_permission_resources},
    types,
};

pub(super) fn plan_export_org_variant_dir(input_type: InspectExportInputType) -> &'static str {
    match input_type {
        InspectExportInputType::Raw => RAW_EXPORT_SUBDIR,
        InspectExportInputType::Source => PROMPT_EXPORT_SUBDIR,
    }
}

fn has_plan_export_org_scopes(input_dir: &Path, variant_dir_name: &str) -> Result<bool> {
    if !input_dir.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with("org_") && path.join(variant_dir_name).is_dir() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn normalize_plan_export_org_scopes_root(
    input_dir: &Path,
    variant_dir_name: &str,
) -> Result<PathBuf> {
    if let Some(resolved_root) = resolve_dashboard_export_root(input_dir)? {
        if resolved_root.manifest.scope_kind.is_aggregate()
            || has_plan_export_org_scopes(&resolved_root.metadata_dir, variant_dir_name)?
        {
            return Ok(resolved_root.metadata_dir);
        }
    }
    if has_plan_export_org_scopes(input_dir, variant_dir_name)? {
        return Ok(input_dir.to_path_buf());
    }
    if let Some(workspace_variant_dir) =
        resolve_dashboard_workspace_variant_dir(input_dir, variant_dir_name)
    {
        if has_plan_export_org_scopes(&workspace_variant_dir, variant_dir_name)? {
            return Ok(workspace_variant_dir);
        }
    }
    Ok(input_dir.to_path_buf())
}

fn load_plan_export_org_index_entries(
    input_dir: &Path,
    metadata: Option<&super::super::ExportMetadata>,
) -> Result<Vec<super::super::VariantIndexEntry>> {
    let index_file = metadata
        .map(|item| item.index_file.clone())
        .unwrap_or_else(|| "index.json".to_string());
    let index_path = input_dir.join(index_file);
    if !index_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&index_path)?;
    serde_json::from_str(&raw).map_err(|error| {
        message(format!(
            "Invalid dashboard export index in {}: {error}",
            index_path.display()
        ))
    })
}

fn org_id_text_from_value(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => Some(text.trim().to_string()),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    }
}

fn collect_plan_export_org_ids(
    input_dir: &Path,
    metadata: Option<&super::super::ExportMetadata>,
) -> Result<BTreeSet<String>> {
    let mut org_ids = BTreeSet::new();
    if let Some(metadata) = metadata {
        if let Some(org_id) = metadata
            .org_id
            .as_ref()
            .map(|value| value.trim().to_string())
        {
            if !org_id.is_empty() {
                org_ids.insert(org_id);
            }
        }
        if let Some(orgs) = metadata.orgs.as_ref() {
            for org in orgs {
                let org_id = org.org_id.trim();
                if !org_id.is_empty() {
                    org_ids.insert(org_id.to_string());
                }
            }
        }
    }
    for entry in load_plan_export_org_index_entries(input_dir, metadata)? {
        if !entry.org_id.trim().is_empty() {
            org_ids.insert(entry.org_id.trim().to_string());
        }
    }
    for folder in load_folder_inventory(input_dir, metadata)? {
        if !folder.org_id.trim().is_empty() {
            org_ids.insert(folder.org_id.trim().to_string());
        }
    }
    for datasource in load_datasource_inventory(input_dir, metadata)? {
        if !datasource.org_id.trim().is_empty() {
            org_ids.insert(datasource.org_id.trim().to_string());
        }
    }
    Ok(org_ids)
}

fn collect_plan_export_org_names(
    input_dir: &Path,
    metadata: Option<&super::super::ExportMetadata>,
) -> Result<BTreeSet<String>> {
    let mut org_names = BTreeSet::new();
    if let Some(metadata) = metadata {
        if let Some(org) = metadata.org.as_ref().map(|value| value.trim().to_string()) {
            if !org.is_empty() {
                org_names.insert(org);
            }
        }
        if let Some(orgs) = metadata.orgs.as_ref() {
            for org in orgs {
                let org_name = org.org.trim();
                if !org_name.is_empty() {
                    org_names.insert(org_name.to_string());
                }
            }
        }
    }
    for entry in load_plan_export_org_index_entries(input_dir, metadata)? {
        if !entry.org.trim().is_empty() {
            org_names.insert(entry.org.trim().to_string());
        }
    }
    for folder in load_folder_inventory(input_dir, metadata)? {
        if !folder.org.trim().is_empty() {
            org_names.insert(folder.org.trim().to_string());
        }
    }
    for datasource in load_datasource_inventory(input_dir, metadata)? {
        if !datasource.org.trim().is_empty() {
            org_names.insert(datasource.org.trim().to_string());
        }
    }
    Ok(org_names)
}

fn parse_plan_export_org_scope_for_variant(
    import_root: &Path,
    variant_dir: &Path,
    expected_variant: &'static str,
) -> Result<ExportOrgImportScope> {
    let metadata = load_export_metadata(variant_dir, Some(expected_variant))?;
    let export_org_ids = collect_plan_export_org_ids(variant_dir, metadata.as_ref())?;
    if export_org_ids.is_empty() {
        return Err(message(format!(
            "Dashboard plan with --use-export-org could not find {expected_variant} export orgId metadata in {}.",
            variant_dir.display()
        )));
    }
    if export_org_ids.len() > 1 {
        return Err(message(format!(
            "Dashboard plan with --use-export-org found multiple export orgIds in {}: {}",
            variant_dir.display(),
            export_org_ids
                .into_iter()
                .collect::<Vec<String>>()
                .join(", ")
        )));
    }
    let source_org_id_text = export_org_ids.into_iter().next().unwrap_or_default();
    let source_org_id = source_org_id_text.parse::<i64>().map_err(|_| {
        message(format!(
            "Dashboard plan with --use-export-org found a non-numeric export orgId '{}' in {}.",
            source_org_id_text,
            variant_dir.display()
        ))
    })?;
    let export_org_names = collect_plan_export_org_names(variant_dir, metadata.as_ref())?;
    if export_org_names.len() > 1 {
        return Err(message(format!(
            "Dashboard plan with --use-export-org found multiple export org names in {}: {}",
            variant_dir.display(),
            export_org_names
                .into_iter()
                .collect::<Vec<String>>()
                .join(", ")
        )));
    }
    let source_org_name = export_org_names.into_iter().next().unwrap_or_else(|| {
        variant_dir
            .file_name()
            .and_then(|value| value.to_str())
            .or_else(|| import_root.file_name().and_then(|value| value.to_str()))
            .unwrap_or("org")
            .to_string()
    });
    Ok(ExportOrgImportScope {
        source_org_id,
        source_org_name,
        input_dir: variant_dir.to_path_buf(),
    })
}

fn discover_plan_export_org_scopes(
    args: &super::super::PlanArgs,
) -> Result<Vec<ExportOrgImportScope>> {
    if !args.use_export_org {
        return Ok(Vec::new());
    }
    let variant_dir_name = plan_export_org_variant_dir(args.input_type);
    let scan_root = normalize_plan_export_org_scopes_root(&args.input_dir, variant_dir_name)?;
    let selected_org_ids: BTreeSet<i64> = args.only_org_id.iter().copied().collect();
    let mut scopes = Vec::new();
    if scan_root.is_dir() {
        for entry in fs::read_dir(&scan_root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|item| item.to_str()) else {
                continue;
            };
            if !name.starts_with("org_") {
                continue;
            }
            let variant_dir = path.join(variant_dir_name);
            if !variant_dir.is_dir() {
                continue;
            }
            let scope = parse_plan_export_org_scope_for_variant(
                &scan_root,
                &variant_dir,
                variant_dir_name,
            )?;
            if !selected_org_ids.is_empty() && !selected_org_ids.contains(&scope.source_org_id) {
                continue;
            }
            scopes.push(scope);
        }
    }
    scopes.sort_by_key(|scope| scope.source_org_id);
    if scopes.is_empty() {
        if selected_org_ids.is_empty() {
            return Err(message(format!(
                "Dashboard plan with --use-export-org did not find any org-specific {variant_dir_name} exports under {}.",
                scan_root.display()
            )));
        }
        return Err(message(format!(
            "Dashboard plan with --use-export-org did not find the selected exported org IDs ({}) under {}.",
            selected_org_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            scan_root.display()
        )));
    }
    let found_org_ids: BTreeSet<i64> = scopes.iter().map(|scope| scope.source_org_id).collect();
    let missing_org_ids: Vec<String> = selected_org_ids
        .difference(&found_org_ids)
        .map(|id| id.to_string())
        .collect();
    if !missing_org_ids.is_empty() {
        return Err(message(format!(
            "Dashboard plan with --use-export-org did not find the selected exported org IDs ({}).",
            missing_org_ids.join(", ")
        )));
    }
    Ok(scopes)
}

fn export_org_target_org_name(
    target_org_id: Option<i64>,
    lookup_cache: &ImportLookupCache,
    fallback_name: &str,
) -> String {
    let Some(target_org_id) = target_org_id else {
        return "<new>".to_string();
    };
    if let Some(orgs) = lookup_cache.orgs.as_ref() {
        let target_id_text = target_org_id.to_string();
        for org in orgs {
            if org_id_text_from_value(org.get("id")).as_deref() == Some(target_id_text.as_str()) {
                if let Some(name) = org.get("name").and_then(Value::as_str) {
                    return name.to_string();
                }
            }
        }
    }
    fallback_name.to_string()
}

fn build_export_routing_import_args(args: &super::super::PlanArgs) -> super::super::ImportArgs {
    super::super::ImportArgs {
        common: args.common.clone(),
        org_id: None,
        use_export_org: true,
        only_org_id: args.only_org_id.clone(),
        create_missing_orgs: args.create_missing_orgs,
        input_dir: args.input_dir.clone(),
        local: false,
        run: None,
        run_id: None,
        input_format: DashboardImportInputFormat::Raw,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: false,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: String::new(),
        interactive: false,
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        list_columns: false,
        progress: false,
        verbose: false,
    }
}

fn load_local_org_plan_input(
    input_dir: &Path,
    expected_variant: &'static str,
    source_org_id: Option<String>,
    source_org_name: String,
    target_org_id: Option<String>,
    target_org_name: String,
    org_action: String,
) -> Result<OrgPlanInput> {
    let metadata = load_export_metadata(input_dir, Some(expected_variant))?;
    let folder_inventory = load_folder_inventory(input_dir, metadata.as_ref())?;
    let mut local_dashboards = Vec::new();
    for file in discover_dashboard_files(input_dir)? {
        let document = load_json_file(&file)?;
        local_dashboards.push(build_local_dashboard(&document, &file, &folder_inventory)?);
    }
    let folder_permission_resources = load_folder_permission_resources(
        input_dir,
        metadata
            .as_ref()
            .and_then(|item| item.permissions_file.as_deref()),
        &folder_inventory,
    )?;
    Ok(OrgPlanInput {
        source_org_id,
        source_org_name,
        target_org_id,
        target_org_name,
        org_action,
        input_dir: input_dir.to_path_buf(),
        local_dashboards,
        folder_inventory,
        live_dashboards: Vec::new(),
        live_datasources: Vec::new(),
        live_folders: Vec::new(),
        folder_permission_resources,
        live_folder_permission_resources: Vec::new(),
    })
}

fn collect_live_org_state_with_request<F>(
    request_json: &mut F,
    collect_folders: bool,
) -> Result<PlanLiveState>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let live_datasources = super::super::list_datasources_with_request(&mut *request_json)?;
    let mut live_dashboards = Vec::new();
    let summaries =
        super::super::list_dashboard_summaries_with_request(&mut *request_json, DEFAULT_PAGE_SIZE)?;
    let live_folders = if collect_folders {
        super::super::collect_folder_inventory_with_request(&mut *request_json, &summaries)?
    } else {
        Vec::new()
    };
    let mut seen_uids = BTreeSet::new();
    for summary in summaries {
        let uid = string_field(&summary, "uid", "");
        if uid.is_empty() || !seen_uids.insert(uid.clone()) {
            continue;
        }
        let payload = super::super::fetch_dashboard_with_request(&mut *request_json, &uid)?;
        live_dashboards.push(build_live_dashboard_with_request(request_json, &payload)?);
    }
    Ok(types::PlanLiveState {
        live_datasources,
        live_dashboards,
        live_folders,
    })
}

fn attach_live_state<F>(
    org: &mut OrgPlanInput,
    live_state: PlanLiveState,
    request_json: &mut F,
    include_folder_permissions: bool,
    folder_permission_match: &str,
) -> Result<()>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    org.live_datasources = live_state.live_datasources;
    org.live_dashboards = live_state.live_dashboards;
    org.live_folders = live_state.live_folders;
    if include_folder_permissions {
        org.live_folder_permission_resources = collect_live_folder_permission_resources(
            request_json,
            &org.folder_permission_resources,
            &org.live_folders,
            folder_permission_match,
        )?;
    }
    Ok(())
}

fn collect_single_scope_with_request<F>(
    args: &super::super::PlanArgs,
    request_json: &mut F,
) -> Result<DashboardPlanInput>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let expected_variant = plan_export_org_variant_dir(args.input_type);
    let resolved = load_dashboard_source(
        &args.input_dir,
        DashboardImportInputFormat::Raw,
        Some(args.input_type),
        false,
    )?;
    let current_org = super::super::list::fetch_current_org_with_request(&mut *request_json)?;
    let target_org_id = current_org.get("id").map(|value| match value {
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    });
    let target_org_name = string_field(&current_org, "name", "org");
    let metadata = load_export_metadata(&resolved.input_dir, Some(expected_variant))?;
    let source_org_id = metadata
        .as_ref()
        .and_then(|item| item.org_id.as_ref())
        .cloned();
    let source_org_name = metadata
        .as_ref()
        .and_then(|item| item.org.as_ref())
        .cloned()
        .unwrap_or_else(|| target_org_name.clone());
    let mut org = load_local_org_plan_input(
        &resolved.input_dir,
        expected_variant,
        source_org_id,
        source_org_name,
        target_org_id.clone(),
        target_org_name.clone(),
        if args.org_id.is_some() {
            "explicit-org".to_string()
        } else {
            "current-org".to_string()
        },
    )?;
    let live_state =
        collect_live_org_state_with_request(request_json, args.include_folder_permissions)?;
    attach_live_state(
        &mut org,
        live_state,
        request_json,
        args.include_folder_permissions,
        args.folder_permission_match.as_str(),
    )?;
    Ok(DashboardPlanInput {
        scope: if args.org_id.is_some() {
            "explicit-org".to_string()
        } else {
            "current-org".to_string()
        },
        input_type: match args.input_type {
            InspectExportInputType::Raw => "raw".to_string(),
            InspectExportInputType::Source => "source".to_string(),
        },
        input_layout: resolved.layout_kind.as_str().to_string(),
        prune: args.prune,
        include_folder_permissions: args.include_folder_permissions,
        folder_permission_match: args.folder_permission_match.as_str().to_string(),
        orgs: vec![org],
    })
}

#[cfg(test)]
pub(super) fn collect_export_org_scope_with_request<F>(
    args: &super::super::PlanArgs,
    request_json: &mut F,
) -> Result<DashboardPlanInput>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    collect_export_org_scope_with_live_collector(args, request_json, |_, request_json| {
        collect_live_org_state_with_request(request_json, args.include_folder_permissions)
    })
}

fn collect_export_org_scope_with_live_collector<F, G>(
    args: &super::super::PlanArgs,
    request_json: &mut F,
    mut collect_live_for_org: G,
) -> Result<DashboardPlanInput>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
    G: FnMut(i64, &mut F) -> Result<PlanLiveState>,
{
    let export_args = build_export_routing_import_args(args);
    let mut lookup_cache = ImportLookupCache::default();
    let scopes = discover_plan_export_org_scopes(args)?;
    let layout_kind = load_dashboard_source(
        &args.input_dir,
        DashboardImportInputFormat::Raw,
        Some(args.input_type),
        false,
    )?
    .layout_kind;
    let mut orgs = Vec::new();
    for scope in scopes {
        let target_plan = resolve_target_org_plan_for_export_scope_with_request(
            request_json,
            &mut lookup_cache,
            &export_args,
            &scope,
        )?;
        let target_org_name = export_org_target_org_name(
            target_plan.target_org_id,
            &lookup_cache,
            &target_plan.source_org_name,
        );
        let mut org = load_local_org_plan_input(
            &target_plan.input_dir,
            plan_export_org_variant_dir(args.input_type),
            Some(target_plan.source_org_id.to_string()),
            target_plan.source_org_name.clone(),
            target_plan.target_org_id.map(|value| value.to_string()),
            target_org_name,
            target_plan.org_action.to_string(),
        )?;
        if let Some(target_org_id) = target_plan.target_org_id {
            let live_state = collect_live_for_org(target_org_id, request_json)?;
            attach_live_state(
                &mut org,
                live_state,
                request_json,
                args.include_folder_permissions,
                args.folder_permission_match.as_str(),
            )?;
        }
        orgs.push(org);
    }
    Ok(DashboardPlanInput {
        scope: "export-org".to_string(),
        input_type: match args.input_type {
            InspectExportInputType::Raw => "raw".to_string(),
            InspectExportInputType::Source => "source".to_string(),
        },
        input_layout: layout_kind.as_str().to_string(),
        prune: args.prune,
        include_folder_permissions: args.include_folder_permissions,
        folder_permission_match: args.folder_permission_match.as_str().to_string(),
        orgs,
    })
}

#[cfg(test)]
pub(super) fn collect_plan_input_with_request<F>(
    args: &super::super::PlanArgs,
    request_json: &mut F,
) -> Result<DashboardPlanInput>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if args.use_export_org {
        let context = build_auth_context(&args.common)?;
        if context.auth_mode != "basic" {
            return Err(message(
                "Dashboard plan with --use-export-org requires Basic auth (--basic-user / --basic-password).",
            ));
        }
        return collect_export_org_scope_with_request(args, request_json);
    }
    collect_single_scope_with_request(args, request_json)
}

pub(super) fn collect_plan_input(args: &super::super::PlanArgs) -> Result<DashboardPlanInput> {
    let client = if let Some(org_id) = args.org_id {
        build_http_client_for_org(&args.common, org_id)?
    } else {
        build_http_client(&args.common)?
    };
    let mut request_json =
        |method: reqwest::Method,
         path: &str,
         params: &[(String, String)],
         payload: Option<&Value>| { client.request_json(method, path, params, payload) };
    if args.use_export_org {
        let context = build_auth_context(&args.common)?;
        if context.auth_mode != "basic" {
            return Err(message(
                "Dashboard plan with --use-export-org requires Basic auth (--basic-user / --basic-password).",
            ));
        }
        return collect_export_org_scope_with_live_collector(
            args,
            &mut request_json,
            |target_org_id, _request_json| {
                let scoped_client = build_http_client_for_org(&args.common, target_org_id)?;
                let mut scoped_request_json =
                    |method: reqwest::Method,
                     path: &str,
                     params: &[(String, String)],
                     payload: Option<&Value>| {
                        scoped_client.request_json(method, path, params, payload)
                    };
                collect_live_org_state_with_request(
                    &mut scoped_request_json,
                    args.include_folder_permissions,
                )
            },
        );
    }
    collect_single_scope_with_request(args, &mut request_json)
}
