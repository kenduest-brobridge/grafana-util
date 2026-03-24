//! Import orchestration for dashboards.
//! Loads local export artifacts, computes target orgs, and applies idempotent upsert behavior
//! through the shared dashboard HTTP/auth context.
use reqwest::Method;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::common::{message, string_field, value_as_object, Result};
use crate::http::{JsonHttpClient, JsonHttpClientConfig};

use super::import_lookup::{
    apply_folder_path_guard_to_action, collect_folder_inventory_statuses_cached,
    determine_dashboard_import_action_with_request,
    determine_import_folder_uid_override_with_request, ensure_folder_inventory_entry_cached,
    resolve_dashboard_import_folder_path_with_request,
    resolve_existing_dashboard_folder_path_with_request, build_folder_path_match_result,
    ImportLookupCache,
};
use super::import_render::{
    build_folder_inventory_dry_run_record, build_import_dry_run_json_value,
    build_import_dry_run_record,
    build_routed_import_dry_run_json_document, build_routed_import_org_row,
};
use super::import_validation::{
    discover_export_org_import_scopes,
    resolve_target_org_plan_for_export_scope_with_request,
    validate_dashboard_import_dependencies_with_request, validate_matching_export_org_with_request,
};

pub(crate) use super::import_compare::diff_dashboards_with_request;

pub(crate) use super::import_render::{
    describe_dashboard_import_mode, format_import_progress_line, format_import_verbose_line,
    format_routed_import_scope_summary_fields, render_folder_inventory_dry_run_table,
    render_import_dry_run_json, render_import_dry_run_table, render_routed_import_org_table,
    ImportDryRunReport,
};
#[cfg(test)]
pub(crate) use super::import_render::format_routed_import_target_org_label;
pub(crate) use super::import_validation::build_import_auth_context;

fn dashboard_files_for_import(import_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut dashboard_files = super::discover_dashboard_files(import_dir)?;
    dashboard_files.retain(|path| {
        path.file_name().and_then(|name| name.to_str()) != Some(super::FOLDER_INVENTORY_FILENAME)
    });
    Ok(dashboard_files)
}

/// Purpose: implementation note.
///
/// Args: see function signature.
/// Returns: see implementation.
pub fn diff_dashboards_with_client(
    client: &JsonHttpClient,
    args: &super::DiffArgs,
) -> Result<usize> {
    diff_dashboards_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

/// collect import dry run report with request.
pub(crate) fn collect_import_dry_run_report_with_request<F>(
    mut request_json: F,
    args: &super::ImportArgs,
) -> Result<ImportDryRunReport>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut lookup_cache = ImportLookupCache::default();
    let metadata = super::load_export_metadata(&args.import_dir, Some(super::RAW_EXPORT_SUBDIR))?;
    validate_matching_export_org_with_request(
        &mut request_json,
        &mut lookup_cache,
        args,
        &args.import_dir,
        metadata.as_ref(),
        None,
    )?;
    let folder_inventory = if args.ensure_folders || args.dry_run {
        super::load_folder_inventory(&args.import_dir, metadata.as_ref())?
    } else {
        Vec::new()
    };
    if args.ensure_folders && folder_inventory.is_empty() {
        let folders_file = metadata
            .as_ref()
            .and_then(|item| item.folders_file.as_deref())
            .unwrap_or(super::FOLDER_INVENTORY_FILENAME);
        return Err(message(format!(
            "Folder inventory file not found for --ensure-folders: {}. Re-export dashboards with raw folder inventory or omit --ensure-folders.",
            args.import_dir.join(folders_file).display()
        )));
    }
    let folder_statuses = if args.ensure_folders {
        collect_folder_inventory_statuses_cached(
            &mut request_json,
            &mut lookup_cache,
            &folder_inventory,
        )?
    } else {
        Vec::new()
    };
    let folders_by_uid: BTreeMap<String, super::FolderInventoryItem> = folder_inventory
        .into_iter()
        .map(|item| (item.uid.clone(), item))
        .collect();
    let dashboard_files = dashboard_files_for_import(&args.import_dir)?;
    let effective_replace_existing = args.replace_existing || args.update_existing_only;
    let mut dashboard_records: Vec<[String; 8]> = Vec::new();
    for dashboard_file in &dashboard_files {
        let document = super::load_json_file(dashboard_file)?;
        if args.strict_schema {
            super::validate::validate_dashboard_import_document(
                &document,
                dashboard_file,
                true,
                args.target_schema_version,
            )?;
        }
        let document_object =
            value_as_object(&document, "Dashboard payload must be a JSON object.")?;
        let dashboard = super::extract_dashboard_object(document_object)?;
        let uid = string_field(dashboard, "uid", "");
        let source_folder_path = if args.require_matching_folder_path {
            Some(super::import_lookup::resolve_source_dashboard_folder_path(
                &document,
                dashboard_file,
                &args.import_dir,
                &folders_by_uid,
            )?)
        } else {
            None
        };
        let folder_uid_override = determine_import_folder_uid_override_with_request(
            &mut request_json,
            &mut lookup_cache,
            &uid,
            args.import_folder_uid.as_deref(),
            effective_replace_existing,
        )?;
        let payload = super::build_import_payload(
            &document,
            folder_uid_override.as_deref(),
            effective_replace_existing,
            &args.import_message,
        )?;
        let action = determine_dashboard_import_action_with_request(
            &mut request_json,
            &mut lookup_cache,
            &payload,
            args.replace_existing,
            args.update_existing_only,
        )?;
        let destination_folder_path = if args.require_matching_folder_path {
            resolve_existing_dashboard_folder_path_with_request(
                &mut request_json,
                &mut lookup_cache,
                &uid,
            )?
        } else {
            None
        };
        let (
            folder_paths_match,
            folder_match_reason,
            normalized_source_folder_path,
            normalized_destination_folder_path,
        ) = if args.require_matching_folder_path {
            build_folder_path_match_result(
                source_folder_path.as_deref(),
                destination_folder_path.as_deref(),
                destination_folder_path.is_some(),
                true,
            )
        } else {
            (true, "", String::new(), None)
        };
        let action = apply_folder_path_guard_to_action(action, folder_paths_match);
        let prefer_live_folder_path =
            folder_uid_override.is_some() && args.import_folder_uid.is_none() && !uid.is_empty();
        let folder_path = resolve_dashboard_import_folder_path_with_request(
            &mut request_json,
            &mut lookup_cache,
            &payload,
            &folders_by_uid,
            prefer_live_folder_path,
        )?;
        dashboard_records.push(build_import_dry_run_record(
            dashboard_file,
            &uid,
            action,
            &folder_path,
            &normalized_source_folder_path,
            normalized_destination_folder_path.as_deref(),
            folder_match_reason,
        ));
    }
    Ok(ImportDryRunReport {
        mode: describe_dashboard_import_mode(args.replace_existing, args.update_existing_only)
            .to_string(),
        import_dir: args.import_dir.clone(),
        folder_statuses,
        skipped_missing_count: if args.update_existing_only {
            dashboard_records
                .iter()
                .filter(|record| record[2] == "skip-missing")
                .count()
        } else {
            0
        },
        skipped_folder_mismatch_count: dashboard_records
            .iter()
            .filter(|record| record[2] == "skip-folder-mismatch")
            .count(),
        dashboard_records,
    })
}

fn folder_inventory_status_output_lines(
    folder_statuses: &[super::FolderInventoryStatus],
    no_header: bool,
    json_output: bool,
    table_output: bool,
) {
    if json_output {
        return;
    }
    if table_output {
        let records: Vec<[String; 6]> = folder_statuses
            .iter()
            .map(build_folder_inventory_dry_run_record)
            .collect();
        for line in render_folder_inventory_dry_run_table(&records, !no_header) {
            println!("{line}");
        }
    } else {
        for status in folder_statuses {
            println!("{}", super::format_folder_inventory_status_line(status));
        }
    }
}

/// Purpose: implementation note.
pub(crate) fn import_dashboards_with_request<F>(
    mut request_json: F,
    args: &super::ImportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut lookup_cache = ImportLookupCache::default();
    if args.table && !args.dry_run {
        return Err(message(
            "--table is only supported with --dry-run for import-dashboard.",
        ));
    }
    if args.json && !args.dry_run {
        return Err(message(
            "--json is only supported with --dry-run for import-dashboard.",
        ));
    }
    if args.table && args.json {
        return Err(message(
            "--table and --json are mutually exclusive for import-dashboard.",
        ));
    }
    if args.no_header && !args.table {
        return Err(message(
            "--no-header is only supported with --dry-run --table for import-dashboard.",
        ));
    }
    if !args.output_columns.is_empty() && !args.table {
        return Err(message(
            "--output-columns is only supported with --dry-run --table or table-like --output-format for import-dashboard.",
        ));
    }
    if args.require_matching_folder_path && args.import_folder_uid.is_some() {
        return Err(message(
            "--require-matching-folder-path cannot be combined with --import-folder-uid.",
        ));
    }
    if args.ensure_folders && args.import_folder_uid.is_some() {
        return Err(message(
            "--ensure-folders cannot be combined with --import-folder-uid.",
        ));
    }
    let metadata = super::load_export_metadata(&args.import_dir, Some(super::RAW_EXPORT_SUBDIR))?;
    validate_matching_export_org_with_request(
        &mut request_json,
        &mut lookup_cache,
        args,
        &args.import_dir,
        metadata.as_ref(),
        None,
    )?;
    let folder_inventory = if args.ensure_folders || args.dry_run {
        super::load_folder_inventory(&args.import_dir, metadata.as_ref())?
    } else {
        Vec::new()
    };
    if args.ensure_folders && folder_inventory.is_empty() {
        let folders_file = metadata
            .as_ref()
            .and_then(|item| item.folders_file.as_deref())
            .unwrap_or(super::FOLDER_INVENTORY_FILENAME);
        return Err(message(format!(
            "Folder inventory file not found for --ensure-folders: {}. Re-export dashboards with raw folder inventory or omit --ensure-folders.",
            args.import_dir.join(folders_file).display()
        )));
    }
    let folder_statuses = if args.dry_run && args.ensure_folders {
        collect_folder_inventory_statuses_cached(
            &mut request_json,
            &mut lookup_cache,
            &folder_inventory,
        )?
    } else {
        Vec::new()
    };
    let folders_by_uid: BTreeMap<String, super::FolderInventoryItem> = folder_inventory
        .into_iter()
        .map(|item| (item.uid.clone(), item))
        .collect();
    if !args.dry_run {
        validate_dashboard_import_dependencies_with_request(
            &mut request_json,
            &args.import_dir,
            args.strict_schema,
            args.target_schema_version,
        )?;
    }
    let dashboard_files = dashboard_files_for_import(&args.import_dir)?;
    let total = dashboard_files.len();
    let effective_replace_existing = args.replace_existing || args.update_existing_only;
    let mut dry_run_records: Vec<[String; 8]> = Vec::new();
    let mut imported_count = 0usize;
    let mut skipped_missing_count = 0usize;
    let mut skipped_folder_mismatch_count = 0usize;
    let mode = describe_dashboard_import_mode(args.replace_existing, args.update_existing_only);
    if !args.json {
        println!("Import mode: {}", mode);
    }
    if args.dry_run && args.ensure_folders {
        folder_inventory_status_output_lines(&folder_statuses, args.no_header, args.json, args.table);
        let missing_folder_count = folder_statuses
            .iter()
            .filter(|status| status.kind == super::FolderInventoryStatusKind::Missing)
            .count();
        let mismatched_folder_count = folder_statuses
            .iter()
            .filter(|status| status.kind == super::FolderInventoryStatusKind::Mismatch)
            .count();
        let folders_file = metadata
            .as_ref()
            .and_then(|item| item.folders_file.as_deref())
            .unwrap_or(super::FOLDER_INVENTORY_FILENAME);
        if !args.json {
            println!(
                "Dry-run checked {} folder(s) from {}; {} missing, {} mismatched",
                folder_statuses.len(),
                args.import_dir.join(folders_file).display(),
                missing_folder_count,
                mismatched_folder_count
            );
        }
    }
    for (index, dashboard_file) in dashboard_files.iter().enumerate() {
        let document = super::load_json_file(dashboard_file)?;
        if args.strict_schema {
            super::validate::validate_dashboard_import_document(
                &document,
                dashboard_file,
                true,
                args.target_schema_version,
            )?;
        }
        let document_object =
            value_as_object(&document, "Dashboard payload must be a JSON object.")?;
        let dashboard = super::extract_dashboard_object(document_object)?;
        let uid = string_field(dashboard, "uid", "");
        let source_folder_path = if args.require_matching_folder_path {
            Some(super::import_lookup::resolve_source_dashboard_folder_path(
                &document,
                dashboard_file,
                &args.import_dir,
                &folders_by_uid,
            )?)
        } else {
            None
        };
        let folder_uid_override = determine_import_folder_uid_override_with_request(
            &mut request_json,
            &mut lookup_cache,
            &uid,
            args.import_folder_uid.as_deref(),
            effective_replace_existing,
        )?;
        let payload = super::build_import_payload(
            &document,
            folder_uid_override.as_deref(),
            effective_replace_existing,
            &args.import_message,
        )?;
        let action = if args.dry_run
            || args.update_existing_only
            || args.ensure_folders
            || args.require_matching_folder_path
        {
            Some(determine_dashboard_import_action_with_request(
                &mut request_json,
                &mut lookup_cache,
                &payload,
                args.replace_existing,
                args.update_existing_only,
            )?)
        } else {
            None
        };
        let destination_folder_path = if args.require_matching_folder_path {
            resolve_existing_dashboard_folder_path_with_request(
                &mut request_json,
                &mut lookup_cache,
                &uid,
            )?
        } else {
            None
        };
        let (
            folder_paths_match,
            folder_match_reason,
            normalized_source_folder_path,
            normalized_destination_folder_path,
        ) = if args.require_matching_folder_path {
            build_folder_path_match_result(
                source_folder_path.as_deref(),
                destination_folder_path.as_deref(),
                destination_folder_path.is_some(),
                true,
            )
        } else {
            (true, "", String::new(), None)
        };
        let action =
            action.map(|value| apply_folder_path_guard_to_action(value, folder_paths_match));
        if args.dry_run {
            let needs_dry_run_folder_path =
                args.table || args.json || args.verbose || args.progress;
            let folder_path = if needs_dry_run_folder_path {
                let prefer_live_folder_path = folder_uid_override.is_some()
                    && args.import_folder_uid.is_none()
                    && !uid.is_empty();
                Some(resolve_dashboard_import_folder_path_with_request(
                    &mut request_json,
                    &mut lookup_cache,
                    &payload,
                    &folders_by_uid,
                    prefer_live_folder_path,
                )?)
            } else {
                None
            };
            let payload_object =
                value_as_object(&payload, "Dashboard import payload must be a JSON object.")?;
            let dashboard = payload_object
                .get("dashboard")
                .and_then(Value::as_object)
                .ok_or_else(|| message("Dashboard import payload is missing dashboard."))?;
            let uid = string_field(dashboard, "uid", super::DEFAULT_UNKNOWN_UID);
            if args.table || args.json {
                dry_run_records.push(build_import_dry_run_record(
                    dashboard_file,
                    &uid,
                    action.unwrap_or(super::DEFAULT_UNKNOWN_UID),
                    folder_path.as_deref().unwrap_or(""),
                    &normalized_source_folder_path,
                    normalized_destination_folder_path.as_deref(),
                    folder_match_reason,
                ));
            } else if args.verbose {
                println!(
                    "{}",
                    format_import_verbose_line(
                        dashboard_file,
                        true,
                        Some(&uid),
                        Some(action.unwrap_or(super::DEFAULT_UNKNOWN_UID)),
                        folder_path.as_deref(),
                    )
                );
            } else if args.progress {
                println!(
                    "{}",
                    format_import_progress_line(
                        index + 1,
                        total,
                        &uid,
                        true,
                        Some(action.unwrap_or(super::DEFAULT_UNKNOWN_UID)),
                        folder_path.as_deref(),
                    )
                );
            }
            continue;
        }
        if args.update_existing_only || args.require_matching_folder_path {
            let payload_object =
                value_as_object(&payload, "Dashboard import payload must be a JSON object.")?;
            let dashboard = payload_object
                .get("dashboard")
                .and_then(Value::as_object)
                .ok_or_else(|| message("Dashboard import payload is missing dashboard."))?;
            let uid = string_field(dashboard, "uid", super::DEFAULT_UNKNOWN_UID);
            if action == Some("would-skip-missing") {
                skipped_missing_count += 1;
                if args.verbose {
                    println!(
                        "Skipped import uid={} dest=missing action=skip-missing file={}",
                        uid,
                        dashboard_file.display()
                    );
                } else if args.progress {
                    println!(
                        "Skipping dashboard {}/{}: {} dest=missing action=skip-missing",
                        index + 1,
                        total,
                        uid
                    );
                }
                continue;
            }
            if action == Some("would-skip-folder-mismatch") {
                skipped_folder_mismatch_count += 1;
                if args.verbose {
                    println!(
                        "Skipped import uid={} dest=exists action=skip-folder-mismatch sourceFolderPath={} destinationFolderPath={} file={}",
                        uid,
                        normalized_source_folder_path,
                        normalized_destination_folder_path.as_deref().unwrap_or("-"),
                        dashboard_file.display()
                    );
                } else if args.progress {
                    println!(
                        "Skipping dashboard {}/{}: {} dest=exists action=skip-folder-mismatch",
                        index + 1,
                        total,
                        uid
                    );
                }
                continue;
            }
        }
        if args.ensure_folders {
            let payload_object =
                value_as_object(&payload, "Dashboard import payload must be a JSON object.")?;
            let folder_uid = payload_object
                .get("folderUid")
                .and_then(Value::as_str)
                .unwrap_or("");
            if !folder_uid.is_empty() && action != Some("would-fail-existing") {
                ensure_folder_inventory_entry_cached(
                    &mut request_json,
                    &mut lookup_cache,
                    &folders_by_uid,
                    folder_uid,
                )?;
            }
        }
        let _result = super::import_dashboard_request_with_request(&mut request_json, &payload)?;
        imported_count += 1;
        if args.verbose {
            println!(
                "{}",
                format_import_verbose_line(dashboard_file, false, None, None, None)
            );
        } else if args.progress {
            println!(
                "{}",
                format_import_progress_line(
                    index + 1,
                    total,
                    &dashboard_file.display().to_string(),
                    false,
                    None,
                    None,
                )
            );
        }
    }
    if args.dry_run {
        if args.update_existing_only {
            skipped_missing_count = dry_run_records
                .iter()
                .filter(|record| record[2] == "skip-missing")
                .count();
        }
        skipped_folder_mismatch_count = dry_run_records
            .iter()
            .filter(|record| record[2] == "skip-folder-mismatch")
            .count();
        if args.json {
            println!(
                "{}",
                render_import_dry_run_json(
                    mode,
                    &folder_statuses,
                    &dry_run_records,
                    &args.import_dir,
                    skipped_missing_count,
                    skipped_folder_mismatch_count,
                )?
            );
        } else if args.table {
            for line in render_import_dry_run_table(
                &dry_run_records,
                !args.no_header,
                if args.output_columns.is_empty() {
                    None
                } else {
                    Some(args.output_columns.as_slice())
                },
            ) {
                println!("{line}");
            }
        }
        if args.json {
        } else if args.update_existing_only
            && skipped_missing_count > 0
            && skipped_folder_mismatch_count > 0
        {
            println!(
                "Dry-run checked {} dashboard(s) from {}; would skip {} missing dashboards and {} folder-mismatched dashboards",
                dashboard_files.len(),
                args.import_dir.display(),
                skipped_missing_count,
                skipped_folder_mismatch_count
            );
        } else if args.update_existing_only && skipped_missing_count > 0 {
            println!(
                "Dry-run checked {} dashboard(s) from {}; would skip {} missing dashboards",
                dashboard_files.len(),
                args.import_dir.display(),
                skipped_missing_count
            );
        } else if skipped_folder_mismatch_count > 0 {
            println!(
                "Dry-run checked {} dashboard(s) from {}; would skip {} folder-mismatched dashboards",
                dashboard_files.len(),
                args.import_dir.display(),
                skipped_folder_mismatch_count
            );
        } else {
            println!(
                "Dry-run checked {} dashboard(s) from {}",
                dashboard_files.len(),
                args.import_dir.display()
            );
        }
        return Ok(dashboard_files.len());
    }
    if args.update_existing_only && skipped_missing_count > 0 && skipped_folder_mismatch_count > 0 {
        println!(
            "Imported {} dashboard files from {}; skipped {} missing dashboards and {} folder-mismatched dashboards",
            imported_count,
            args.import_dir.display(),
            skipped_missing_count,
            skipped_folder_mismatch_count
        );
    } else if args.update_existing_only && skipped_missing_count > 0 {
        println!(
            "Imported {} dashboard files from {}; skipped {} missing dashboards",
            imported_count,
            args.import_dir.display(),
            skipped_missing_count
        );
    } else if skipped_folder_mismatch_count > 0 {
        println!(
            "Imported {} dashboard files from {}; skipped {} folder-mismatched dashboards",
            imported_count,
            args.import_dir.display(),
            skipped_folder_mismatch_count
        );
    }
    Ok(imported_count)
}

/// Purpose: implementation note.
///
/// Args: see function signature.
/// Returns: see implementation.
pub fn import_dashboards_with_client(client: &JsonHttpClient, args: &super::ImportArgs) -> Result<usize> {
    import_dashboards_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

fn count_dashboard_files(import_dir: &Path) -> Result<usize> {
    Ok(dashboard_files_for_import(import_dir)?.len())
}

/// Purpose: implementation note.
pub(crate) fn build_routed_import_dry_run_json_with_request<F, G>(
    mut request_json: F,
    mut collect_preview_for_org: G,
    args: &super::ImportArgs,
) -> Result<String>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
    G: FnMut(i64, &super::ImportArgs) -> Result<ImportDryRunReport>,
{
    let scopes = discover_export_org_import_scopes(args)?;
    let mut lookup_cache = ImportLookupCache::default();
    let mut orgs = Vec::new();
    let mut imports = Vec::new();
    for scope in scopes {
        let target_plan = resolve_target_org_plan_for_export_scope_with_request(
            &mut request_json,
            &mut lookup_cache,
            args,
            &scope,
        )?;
        let dashboard_count = count_dashboard_files(&target_plan.import_dir)?;
        orgs.push(serde_json::json!({
            "sourceOrgId": target_plan.source_org_id,
            "sourceOrgName": target_plan.source_org_name,
            "orgAction": target_plan.org_action,
            "targetOrgId": target_plan.target_org_id,
            "dashboardCount": dashboard_count,
            "importDir": target_plan.import_dir.display().to_string(),
        }));
        let preview = if let Some(target_org_id) = target_plan.target_org_id {
            let mut scoped_args = args.clone();
            scoped_args.org_id = Some(target_org_id);
            scoped_args.use_export_org = false;
            scoped_args.only_org_id = Vec::new();
            scoped_args.create_missing_orgs = false;
            scoped_args.import_dir = target_plan.import_dir.clone();
            let report = collect_preview_for_org(target_org_id, &scoped_args)?;
            build_import_dry_run_json_value(&report)
        } else {
            serde_json::json!({
                "mode": describe_dashboard_import_mode(args.replace_existing, args.update_existing_only),
                "folders": [],
                "dashboards": [],
                "summary": {
                    "importDir": target_plan.import_dir.display().to_string(),
                    "folderCount": 0,
                    "missingFolders": 0,
                    "mismatchedFolders": 0,
                    "dashboardCount": dashboard_count,
                    "missingDashboards": 0,
                    "skippedMissingDashboards": 0,
                    "skippedFolderMismatchDashboards": 0,
                }
            })
        };
        let mut import_entry = serde_json::Map::new();
        import_entry.insert(
            "sourceOrgId".to_string(),
            Value::from(target_plan.source_org_id),
        );
        import_entry.insert(
            "sourceOrgName".to_string(),
            Value::from(target_plan.source_org_name.clone()),
        );
        import_entry.insert("orgAction".to_string(), Value::from(target_plan.org_action));
        import_entry.insert(
            "targetOrgId".to_string(),
            target_plan
                .target_org_id
                .map(Value::from)
                .unwrap_or(Value::Null),
        );
        if let Some(preview_object) = preview.as_object() {
            for (key, value) in preview_object {
                import_entry.insert(key.clone(), value.clone());
            }
        }
        imports.push(Value::Object(import_entry));
    }
    build_routed_import_dry_run_json_document(&orgs, &imports)
}

/// Purpose: implementation note.
pub(crate) fn import_dashboards_by_export_org_with_request<F, G, H>(
    mut request_json: F,
    mut import_for_org: G,
    collect_preview_for_org: H,
    args: &super::ImportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
    G: FnMut(i64, &super::ImportArgs) -> Result<usize>,
    H: FnMut(i64, &super::ImportArgs) -> Result<ImportDryRunReport>,
{
    let scopes = discover_export_org_import_scopes(args)?;
    let mut lookup_cache = ImportLookupCache::default();
    if args.dry_run && args.json {
        println!(
            "{}",
            build_routed_import_dry_run_json_with_request(
                request_json,
                collect_preview_for_org,
                args,
            )?
        );
        return Ok(0);
    }
    let mut imported_count = 0;
    let mut org_rows = Vec::new();
    let mut resolved_plans = Vec::new();
    for scope in scopes {
        let target_plan = resolve_target_org_plan_for_export_scope_with_request(
            &mut request_json,
            &mut lookup_cache,
            args,
            &scope,
        )?;
        let dashboard_count = count_dashboard_files(&target_plan.import_dir)?;
        org_rows.push(build_routed_import_org_row(&target_plan, dashboard_count));
        resolved_plans.push(target_plan);
    }
    if args.dry_run && args.table {
        for line in render_routed_import_org_table(&org_rows, !args.no_header) {
            println!("{line}");
        }
        return Ok(0);
    }
    for target_plan in resolved_plans {
        if !args.table {
            println!(
                "Importing {}",
                format_routed_import_scope_summary_fields(
                    target_plan.source_org_id,
                    &target_plan.source_org_name,
                    target_plan.org_action,
                    target_plan.target_org_id,
                    &target_plan.import_dir,
                )
            );
        }
        let Some(target_org_id) = target_plan.target_org_id else {
            continue;
        };
        let mut scoped_args = args.clone();
        scoped_args.org_id = Some(target_org_id);
        scoped_args.use_export_org = false;
        scoped_args.only_org_id = Vec::new();
        scoped_args.create_missing_orgs = false;
        scoped_args.import_dir = target_plan.import_dir.clone();
        imported_count += import_for_org(target_org_id, &scoped_args).map_err(|error| {
            message(format!(
                "Dashboard routed import failed for {}: {}",
                format_routed_import_scope_summary_fields(
                    target_plan.source_org_id,
                    &target_plan.source_org_name,
                    target_plan.org_action,
                    target_plan.target_org_id,
                    &target_plan.import_dir,
                ),
                error
            ))
        })?;
    }
    Ok(imported_count)
}

/// Purpose: implementation note.
pub(crate) fn import_dashboards_with_org_clients(args: &super::ImportArgs) -> Result<usize> {
    let context = build_import_auth_context(args)?;
    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: context.url.clone(),
        headers: context.headers.clone(),
        timeout_secs: context.timeout,
        verify_ssl: context.verify_ssl,
    })?;
    if !args.use_export_org {
        return import_dashboards_with_request(
            |method, path, params, payload| client.request_json(method, path, params, payload),
            args,
        );
    }
    import_dashboards_by_export_org_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        |target_org_id, scoped_args| {
            let scoped_client = super::build_http_client_for_org(&args.common, target_org_id)?;
            import_dashboards_with_client(&scoped_client, scoped_args)
        },
        |target_org_id, scoped_args| {
            let scoped_client = super::build_http_client_for_org(&args.common, target_org_id)?;
            collect_import_dry_run_report_with_request(
                |method, path, params, payload| {
                    scoped_client.request_json(method, path, params, payload)
                },
                scoped_args,
            )
        },
        args,
    )
}
