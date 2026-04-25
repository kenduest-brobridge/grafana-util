use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{message, validation, Result};

use super::super::cli_defs::InspectLiveArgs;
use super::super::files::{
    build_export_metadata, load_export_metadata, resolve_dashboard_export_root, write_json_document,
};
use super::super::{
    DATASOURCE_INVENTORY_FILENAME, EXPORT_METADATA_FILENAME, FOLDER_INVENTORY_FILENAME,
    RAW_EXPORT_SUBDIR,
};

static LIVE_INSPECT_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TempInspectDir {
    pub(crate) path: PathBuf,
}

impl TempInspectDir {
    pub(crate) fn new(prefix: &str) -> Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| validation(format!("Failed to build {prefix} temp path: {error}")))?
            .as_nanos();
        let sequence = LIVE_INSPECT_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "grafana-utils-{prefix}-{}-{timestamp}-{sequence}",
            process::id(),
        ));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempInspectDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn prepare_live_review_import_dir(temp_root: &Path, all_orgs: bool) -> Result<PathBuf> {
    if !all_orgs {
        return Ok(temp_root.join(RAW_EXPORT_SUBDIR));
    }

    let inspect_raw_dir = temp_root
        .join("summary-live-all-orgs")
        .join(RAW_EXPORT_SUBDIR);
    let org_raw_dirs = discover_org_variant_export_dirs(temp_root, RAW_EXPORT_SUBDIR)?;
    merge_org_variant_exports_into_dir(&org_raw_dirs, &inspect_raw_dir, None, RAW_EXPORT_SUBDIR)?;
    Ok(inspect_raw_dir)
}

pub(crate) fn load_variant_index_entries(
    input_dir: &Path,
    metadata: Option<&super::super::models::ExportMetadata>,
) -> Result<Vec<super::super::models::VariantIndexEntry>> {
    let index_file = metadata
        .map(|item| item.index_file.clone())
        .unwrap_or_else(|| "index.json".to_string());
    let index_path = input_dir.join(&index_file);
    if !index_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&index_path)?;
    let entries: Vec<super::super::models::VariantIndexEntry> = serde_json::from_str(&raw)?;
    Ok(entries)
}

fn load_json_array_file(path: &Path, error_context: &str) -> Result<Vec<Value>> {
    let raw = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&raw)?;
    match value {
        Value::Array(items) => Ok(items),
        _ => Err(message(format!(
            "{error_context} must be a JSON array: {}",
            path.display()
        ))),
    }
}

fn discover_org_variant_export_dirs(
    input_dir: &Path,
    expected_variant: &'static str,
) -> Result<Vec<(String, PathBuf)>> {
    if !input_dir.exists() {
        return Err(message(format!(
            "Import directory does not exist: {}",
            input_dir.display()
        )));
    }
    if !input_dir.is_dir() {
        return Err(message(format!(
            "Import path is not a directory: {}",
            input_dir.display()
        )));
    }
    let mut org_raw_dirs = Vec::new();
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let org_root = entry.path();
        if !org_root.is_dir() {
            continue;
        }
        let org_name = entry.file_name().to_string_lossy().to_string();
        if !org_name.starts_with("org_") {
            continue;
        }
        let org_variant_dir = org_root.join(expected_variant);
        if org_variant_dir.is_dir() {
            org_raw_dirs.push((org_name, org_variant_dir));
        }
    }
    org_raw_dirs.sort_by(|left, right| left.0.cmp(&right.0));
    if org_raw_dirs.is_empty() {
        return Err(message(format!(
            "Import path {} does not contain any org-scoped {expected_variant}/ exports. Point --input-dir at a combined multi-org dashboard export root that includes that variant.",
            input_dir.display(),
        )));
    }
    Ok(org_raw_dirs)
}

fn merge_org_variant_exports_into_dir(
    org_variant_dirs: &[(String, PathBuf)],
    inspect_variant_dir: &Path,
    source_root: Option<&Path>,
    expected_variant: &'static str,
) -> Result<()> {
    // Preserve each org's raw export content and rewrite the index paths so the merged
    // tree still looks like one export root to the downstream inspect readers.
    fs::create_dir_all(inspect_variant_dir)?;

    let mut folder_inventory = Vec::new();
    let mut datasource_inventory = Vec::new();
    let mut index_entries = Vec::new();
    let mut dashboard_count = 0usize;

    for (org_name, org_variant_dir) in org_variant_dirs {
        let relative_target = inspect_variant_dir.join(org_name);
        copy_dir_recursive(org_variant_dir, &relative_target)?;

        let metadata = load_export_metadata(org_variant_dir, Some(expected_variant))?;
        let org_index_entries = load_variant_index_entries(org_variant_dir, metadata.as_ref())?;
        for mut entry in org_index_entries.clone() {
            entry.path = format!("{org_name}/{}", entry.path);
            index_entries.push(entry);
        }

        let folder_path = org_variant_dir.join(FOLDER_INVENTORY_FILENAME);
        if folder_path.is_file() {
            folder_inventory.extend(load_json_array_file(
                &folder_path,
                "Dashboard folder inventory",
            )?);
        }
        let datasource_path = org_variant_dir.join(DATASOURCE_INVENTORY_FILENAME);
        if datasource_path.is_file() {
            datasource_inventory.extend(load_json_array_file(
                &datasource_path,
                "Dashboard datasource inventory",
            )?);
        }

        dashboard_count += metadata
            .as_ref()
            .map(|item| item.dashboard_count as usize)
            .unwrap_or(org_index_entries.len());
    }

    write_json_document(
        &build_export_metadata(
            expected_variant,
            dashboard_count,
            Some("grafana-web-import-preserve-uid"),
            Some(FOLDER_INVENTORY_FILENAME),
            Some(DATASOURCE_INVENTORY_FILENAME),
            None,
            None,
            None,
            None,
            "local",
            None,
            source_root,
            None,
            inspect_variant_dir,
            &inspect_variant_dir.join(EXPORT_METADATA_FILENAME),
        ),
        &inspect_variant_dir.join(EXPORT_METADATA_FILENAME),
    )?;
    write_json_document(&index_entries, &inspect_variant_dir.join("index.json"))?;
    write_json_document(
        &folder_inventory,
        &inspect_variant_dir.join(FOLDER_INVENTORY_FILENAME),
    )?;
    write_json_document(
        &datasource_inventory,
        &inspect_variant_dir.join(DATASOURCE_INVENTORY_FILENAME),
    )?;
    if let Some(source_root) = source_root {
        fs::write(
            inspect_variant_dir.join(".inspect-source-root"),
            source_root.display().to_string(),
        )?;
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

pub(crate) fn prepare_inspect_live_import_dir(
    temp_root: &Path,
    args: &InspectLiveArgs,
) -> Result<PathBuf> {
    prepare_live_review_import_dir(temp_root, args.all_orgs)
}

#[cfg(test)]
pub(crate) fn prepare_inspect_export_import_dir(
    temp_root: &Path,
    input_dir: &Path,
) -> Result<PathBuf> {
    prepare_inspect_export_import_dir_for_variant(temp_root, input_dir, RAW_EXPORT_SUBDIR)
}

pub(crate) fn prepare_inspect_export_import_dir_for_variant(
    temp_root: &Path,
    input_dir: &Path,
    expected_variant: &'static str,
) -> Result<PathBuf> {
    // Root exports need one synthetic raw tree so offline inspect sees the same layout
    // whether the source was a live all-org fetch or a prebuilt export archive.
    let root_manifest = resolve_dashboard_export_root(input_dir)?;
    if root_manifest
        .as_ref()
        .map(|resolved| resolved.manifest.scope_kind.is_root())
        .unwrap_or(false)
    {
        let export_root = root_manifest
            .as_ref()
            .map(|resolved| resolved.metadata_dir.as_path())
            .unwrap_or(input_dir);
        let inspect_variant_dir = temp_root
            .join("summary-export-all-orgs")
            .join(expected_variant);
        let org_variant_dirs = discover_org_variant_export_dirs(export_root, expected_variant)?;
        merge_org_variant_exports_into_dir(
            &org_variant_dirs,
            &inspect_variant_dir,
            Some(export_root),
            expected_variant,
        )?;
        return Ok(inspect_variant_dir);
    }
    Ok(input_dir.to_path_buf())
}
