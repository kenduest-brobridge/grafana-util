use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::common::{message, Result};
use crate::dashboard::{
    load_export_metadata, load_folder_inventory, ExportMetadata, FolderInventoryItem, ImportArgs,
    FOLDER_INVENTORY_FILENAME,
};

pub(super) struct PreparedImportRun {
    pub(super) resolved_import: super::super::LoadedImportSource,
    pub(super) metadata: Option<ExportMetadata>,
    pub(super) folders_by_uid: BTreeMap<String, FolderInventoryItem>,
    pub(super) dashboard_files: Vec<PathBuf>,
}

pub(super) fn validate_import_args(args: &ImportArgs) -> Result<()> {
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
    Ok(())
}

pub(super) fn prepare_import_run<F>(
    select_dashboard_files: &mut F,
    args: &ImportArgs,
) -> Result<PreparedImportRun>
where
    F: FnMut(&super::super::LoadedImportSource, Vec<PathBuf>) -> Result<Option<Vec<PathBuf>>>,
{
    let resolved_import = super::super::resolve_import_source(args)?;
    let metadata = load_export_metadata(
        resolved_import.metadata_dir(),
        Some(super::super::import_metadata_variant(args)),
    )?;
    let folder_inventory = if args.ensure_folders || args.dry_run {
        load_folder_inventory(resolved_import.metadata_dir(), metadata.as_ref())?
    } else {
        Vec::new()
    };
    if args.ensure_folders && folder_inventory.is_empty() {
        let folders_file = metadata
            .as_ref()
            .and_then(|item| item.folders_file.as_deref())
            .unwrap_or(FOLDER_INVENTORY_FILENAME);
        return Err(message(format!(
            "Folder inventory file not found for --ensure-folders: {}. Re-export dashboards with raw folder inventory or omit --ensure-folders.",
            resolved_import.metadata_dir().join(folders_file).display()
        )));
    }
    let folders_by_uid = folder_inventory
        .into_iter()
        .map(|item| (item.uid.clone(), item))
        .collect::<BTreeMap<String, FolderInventoryItem>>();
    let discovered_dashboard_files =
        super::super::dashboard_files_for_import(resolved_import.dashboard_dir())?;
    let dashboard_files =
        match select_dashboard_files(&resolved_import, discovered_dashboard_files.clone())? {
            Some(selected) => selected,
            None if args.interactive => {
                println!(
                    "{} cancelled.",
                    if args.dry_run {
                        "Interactive dry-run"
                    } else {
                        "Import"
                    }
                );
                return Ok(PreparedImportRun {
                    resolved_import,
                    metadata,
                    folders_by_uid,
                    dashboard_files: Vec::new(),
                });
            }
            None => discovered_dashboard_files,
        };
    Ok(PreparedImportRun {
        resolved_import,
        metadata,
        folders_by_uid,
        dashboard_files,
    })
}
