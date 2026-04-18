use serde_json::Value;

use crate::common::{message, Result};
use crate::dashboard::{
    build_import_payload, extract_dashboard_object, load_json_file, validate, ImportArgs,
    DEFAULT_UNKNOWN_UID,
};

use super::super::super::import_lookup::{
    apply_folder_path_guard_to_action, build_folder_path_match_result, ImportLookupCache,
};
use super::super::super::import_render::{format_import_progress_line, format_import_verbose_line};
use super::super::super::import_target::{
    build_dashboard_target_review, build_dashboard_target_review_reason,
    dashboard_target_review_is_blocked, dashboard_target_review_is_warning,
};
use super::import_apply_backend::LiveImportBackend;
use super::import_apply_prepare::PreparedImportRun;

pub(super) fn run_live_import<B: LiveImportBackend>(
    backend: &mut B,
    args: &ImportArgs,
    prepared: &PreparedImportRun,
    lookup_cache: &mut ImportLookupCache,
) -> Result<usize> {
    let total = prepared.dashboard_files.len();
    let effective_replace_existing = args.replace_existing || args.update_existing_only;
    let mut imported_count = 0usize;
    let mut skipped_missing_count = 0usize;
    let mut skipped_folder_mismatch_count = 0usize;
    let mode = super::super::super::import_render::describe_dashboard_import_mode(
        args.replace_existing,
        args.update_existing_only,
    );
    if !args.json {
        println!("Import mode: {}", mode);
    }
    for (index, dashboard_file) in prepared.dashboard_files.iter().enumerate() {
        let document = load_json_file(dashboard_file)?;
        if args.strict_schema {
            validate::validate_dashboard_import_document(
                &document,
                dashboard_file,
                true,
                args.target_schema_version,
            )?;
        }
        let document_object =
            crate::common::value_as_object(&document, "Dashboard payload must be a JSON object.")?;
        let dashboard = extract_dashboard_object(document_object)?;
        let uid = crate::common::string_field(dashboard, "uid", "");
        let source_folder_path = if args.require_matching_folder_path {
            Some(
                super::super::super::import_lookup::resolve_source_dashboard_folder_path(
                    &document,
                    dashboard_file,
                    prepared.resolved_import.dashboard_dir(),
                    &prepared.folders_by_uid,
                )?,
            )
        } else {
            None
        };
        let folder_uid_override = backend.determine_import_folder_uid_override(
            lookup_cache,
            &uid,
            args.import_folder_uid.as_deref(),
            effective_replace_existing,
        )?;
        let payload = build_import_payload(
            &document,
            folder_uid_override.as_deref(),
            effective_replace_existing,
            &args.import_message,
        )?;
        let action = if args.update_existing_only
            || args.ensure_folders
            || args.require_matching_folder_path
        {
            Some(backend.determine_dashboard_import_action(
                lookup_cache,
                &payload,
                args.replace_existing,
                args.update_existing_only,
            )?)
        } else {
            None
        };
        let destination_folder_path = if args.require_matching_folder_path {
            backend.resolve_existing_dashboard_folder_path(lookup_cache, &uid)?
        } else {
            None
        };
        let (
            folder_paths_match,
            _folder_match_reason,
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
            (true, "", String::new(), None::<String>)
        };
        let action =
            action.map(|value| apply_folder_path_guard_to_action(value, folder_paths_match));
        if args.update_existing_only || args.require_matching_folder_path {
            let payload_object = crate::common::value_as_object(
                &payload,
                "Dashboard import payload must be a JSON object.",
            )?;
            let dashboard = payload_object
                .get("dashboard")
                .and_then(Value::as_object)
                .ok_or_else(|| message("Dashboard import payload is missing dashboard."))?;
            let uid = crate::common::string_field(dashboard, "uid", DEFAULT_UNKNOWN_UID);
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
        if effective_replace_existing && !uid.is_empty() {
            let existing_dashboard = backend.fetch_existing_dashboard(lookup_cache, &uid)?;
            if let Some(existing_dashboard) = existing_dashboard {
                let review = build_dashboard_target_review(&existing_dashboard)?;
                if dashboard_target_review_is_blocked(&review) {
                    return Err(message(format!(
                        "Refusing to overwrite provisioned dashboard uid={uid}: {}",
                        build_dashboard_target_review_reason(&review)
                    )));
                }
                if dashboard_target_review_is_warning(&review) {
                    eprintln!(
                        "Warning: overwriting managed dashboard uid={uid}: {}",
                        build_dashboard_target_review_reason(&review)
                    );
                }
            }
        }
        if args.ensure_folders {
            let payload_object = crate::common::value_as_object(
                &payload,
                "Dashboard import payload must be a JSON object.",
            )?;
            let folder_uid = payload_object
                .get("folderUid")
                .and_then(Value::as_str)
                .unwrap_or("");
            if !folder_uid.is_empty() && action != Some("would-fail-existing") {
                backend.ensure_folder_inventory_entry(
                    lookup_cache,
                    &prepared.folders_by_uid,
                    folder_uid,
                )?;
            }
        }
        backend.import_dashboard(&payload)?;
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
    if args.update_existing_only && skipped_missing_count > 0 && skipped_folder_mismatch_count > 0 {
        println!(
            "Imported {} dashboard files from {}; skipped {} missing dashboards and {} folder-mismatched dashboards",
            imported_count,
            args.input_dir.display(),
            skipped_missing_count,
            skipped_folder_mismatch_count
        );
    } else if args.update_existing_only && skipped_missing_count > 0 {
        println!(
            "Imported {} dashboard files from {}; skipped {} missing dashboards",
            imported_count,
            args.input_dir.display(),
            skipped_missing_count
        );
    } else if skipped_folder_mismatch_count > 0 {
        println!(
            "Imported {} dashboard files from {}; skipped {} folder-mismatched dashboards",
            imported_count,
            args.input_dir.display(),
            skipped_folder_mismatch_count
        );
    } else {
        println!(
            "Imported {} dashboard files from {}",
            imported_count,
            args.input_dir.display()
        );
    }
    Ok(imported_count)
}
