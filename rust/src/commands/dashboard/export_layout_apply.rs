use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::common::{message, Result};

use super::{
    build_export_metadata, build_root_export_index, entry_relative_path, load_export_metadata,
    load_folder_inventory, load_json_file, load_variant_index_entries, write_json_document,
    DashboardIndexItem, ExportLayoutArgs, ExportLayoutPlan, ExportOrgSummary, FolderInventoryItem,
    LayoutVariant, RootExportIndex, DEFAULT_ORG_ID, EXPORT_METADATA_FILENAME, PROMPT_EXPORT_SUBDIR,
    RAW_EXPORT_SUBDIR,
};

pub(super) fn apply_export_layout_plan(
    args: &ExportLayoutArgs,
    execution_root: &Path,
    plan: &ExportLayoutPlan,
) -> Result<bool> {
    let mut changed_variants = BTreeSet::new();
    for operation in &plan.operations {
        if operation.action != "move" {
            continue;
        }
        let variant_dir = execution_root.join(&operation.variant_scope);
        let source = variant_dir.join(&operation.from_relative);
        let target = variant_dir.join(&operation.to_relative);
        if args.in_place {
            backup_file(args, execution_root, &source)?;
            if target.exists() {
                backup_file(args, execution_root, &target)?;
            }
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        if target.exists() && args.overwrite {
            fs::remove_file(&target)?;
        }
        fs::rename(&source, &target)?;
        changed_variants.insert((operation.variant_scope.clone(), operation.variant.clone()));
    }

    for (variant_scope, variant) in changed_variants {
        let variant_dir = execution_root.join(&variant_scope);
        update_variant_index(&variant_dir, &variant_scope, &variant, plan)?;
        update_root_index(&variant_dir, &variant_scope, &variant, plan)?;
    }
    update_aggregate_root_index(execution_root, plan)?;
    rebuild_all_orgs_aggregate_root(execution_root)
}

fn backup_file(args: &ExportLayoutArgs, input_root: &Path, path: &Path) -> Result<()> {
    let Some(backup_dir) = args.backup_dir.as_ref() else {
        return Ok(());
    };
    if !path.is_file() {
        return Ok(());
    }
    let relative = path.strip_prefix(input_root).unwrap_or(path);
    let target = backup_dir.join(relative);
    if target.exists() && !args.overwrite {
        return Err(message(format!(
            "Refusing to overwrite backup file: {}. Use --overwrite.",
            target.display()
        )));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(path, target)?;
    Ok(())
}

fn update_variant_index(
    variant_dir: &Path,
    variant_scope: &Path,
    variant: &str,
    plan: &ExportLayoutPlan,
) -> Result<()> {
    let index_path = variant_dir.join("index.json");
    if !index_path.is_file() {
        return Ok(());
    }
    let mut entries = load_variant_index_entries(variant_dir)?;
    for entry in &mut entries {
        let Some(operation) = plan.operations.iter().find(|operation| {
            operation.variant == variant
                && operation.uid == entry.uid
                && operation.org_id == entry.org_id
                && operation.action == "move"
                && operation.variant_scope == variant_scope
        }) else {
            continue;
        };
        entry.path = variant_dir
            .join(&operation.to_relative)
            .display()
            .to_string();
    }
    write_json_document(&entries, &index_path)
}

fn update_root_index(
    variant_dir: &Path,
    variant_scope: &Path,
    variant: &str,
    plan: &ExportLayoutPlan,
) -> Result<()> {
    let Some(root_dir) = variant_dir.parent() else {
        return Ok(());
    };
    let index_path = root_dir.join("index.json");
    if !index_path.is_file() {
        return Ok(());
    }
    let value = load_json_file(&index_path)?;
    let mut root_index: RootExportIndex = match serde_json::from_value(value) {
        Ok(index) => index,
        Err(_) => return Ok(()),
    };
    if variant == RAW_EXPORT_SUBDIR {
        root_index.variants.raw = Some(variant_dir.join("index.json").display().to_string());
    } else {
        root_index.variants.prompt = Some(variant_dir.join("index.json").display().to_string());
    }
    for item in &mut root_index.items {
        let Some(operation) = plan.operations.iter().find(|operation| {
            operation.variant == variant
                && operation.uid == item.uid
                && operation.org_id == item.org_id
                && operation.action == "move"
                && operation.variant_scope == variant_scope
        }) else {
            continue;
        };
        let variant = if variant == RAW_EXPORT_SUBDIR {
            LayoutVariant::Raw
        } else {
            LayoutVariant::Prompt
        };
        *variant.index_field(item) = Some(
            variant_dir
                .join(&operation.to_relative)
                .display()
                .to_string(),
        );
    }
    write_json_document(&root_index, &index_path)
}

fn update_aggregate_root_index(execution_root: &Path, plan: &ExportLayoutPlan) -> Result<()> {
    let index_path = execution_root.join("index.json");
    if !index_path.is_file() {
        return Ok(());
    }
    let value = load_json_file(&index_path)?;
    let mut root_index: RootExportIndex = match serde_json::from_value(value) {
        Ok(index) => index,
        Err(_) => return Ok(()),
    };
    let mut changed = false;
    for item in &mut root_index.items {
        for operation in plan.operations.iter().filter(|operation| {
            operation.uid == item.uid
                && operation.org_id == item.org_id
                && operation.action == "move"
        }) {
            let variant_dir = execution_root.join(&operation.variant_scope);
            let repaired_path = variant_dir
                .join(&operation.to_relative)
                .display()
                .to_string();
            if operation.variant == RAW_EXPORT_SUBDIR {
                item.raw_path = Some(repaired_path);
                changed = true;
            } else if operation.variant == PROMPT_EXPORT_SUBDIR {
                item.prompt_path = Some(repaired_path);
                changed = true;
            }
        }
    }
    if changed {
        write_json_document(&root_index, &index_path)?;
    }
    Ok(())
}

fn rebuild_all_orgs_aggregate_root(execution_root: &Path) -> Result<bool> {
    let mut root_items_by_key = BTreeMap::<String, DashboardIndexItem>::new();
    let mut root_folders = Vec::<FolderInventoryItem>::new();
    let mut org_summaries = Vec::<ExportOrgSummary>::new();

    for entry in fs::read_dir(execution_root)? {
        let entry = entry?;
        let org_root = entry.path();
        if !org_root.is_dir() {
            continue;
        }
        let Some(name) = org_root.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !name.starts_with("org_") {
            continue;
        }

        let raw_dir = org_root.join(RAW_EXPORT_SUBDIR);
        let prompt_dir = org_root.join(PROMPT_EXPORT_SUBDIR);
        let raw_metadata = load_export_metadata(&raw_dir, Some(RAW_EXPORT_SUBDIR))?;
        let prompt_metadata = load_export_metadata(&prompt_dir, Some(PROMPT_EXPORT_SUBDIR))?;
        let org_name = raw_metadata
            .as_ref()
            .and_then(|metadata| metadata.org.as_deref())
            .or_else(|| {
                prompt_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.org.as_deref())
            })
            .unwrap_or(name)
            .to_string();
        let org_id = raw_metadata
            .as_ref()
            .and_then(|metadata| metadata.org_id.as_deref())
            .or_else(|| {
                prompt_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.org_id.as_deref())
            })
            .unwrap_or(DEFAULT_ORG_ID)
            .to_string();

        if raw_dir.join("index.json").is_file() {
            merge_variant_entries_into_root(
                &raw_dir,
                LayoutVariant::Raw,
                &org_name,
                &org_id,
                &mut root_items_by_key,
            )?;
            root_folders.extend(load_folder_inventory(&raw_dir, raw_metadata.as_ref())?);
        }
        if prompt_dir.join("index.json").is_file() {
            merge_variant_entries_into_root(
                &prompt_dir,
                LayoutVariant::Prompt,
                &org_name,
                &org_id,
                &mut root_items_by_key,
            )?;
        }

        let dashboard_count = root_items_by_key
            .values()
            .filter(|item| item.org_id == org_id)
            .count() as u64;
        if dashboard_count > 0 {
            org_summaries.push(ExportOrgSummary {
                org: org_name,
                org_id,
                dashboard_count,
                datasource_count: None,
                used_datasource_count: None,
                used_datasources: None,
                output_dir: Some(org_root.display().to_string()),
            });
        }
    }

    if org_summaries.len() < 2 {
        return Ok(false);
    }

    let root_items = root_items_by_key.into_values().collect::<Vec<_>>();
    let root_index = build_root_export_index(&root_items, None, None, None, &root_folders);
    write_json_document(&root_index, &execution_root.join("index.json"))?;
    write_json_document(
        &build_export_metadata(
            "root",
            root_items.len(),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(org_summaries),
            "local-repair",
            None,
            Some(execution_root),
            None,
            execution_root,
            &execution_root.join(EXPORT_METADATA_FILENAME),
        ),
        &execution_root.join(EXPORT_METADATA_FILENAME),
    )?;
    Ok(true)
}

fn merge_variant_entries_into_root(
    variant_dir: &Path,
    variant: LayoutVariant,
    org_name: &str,
    org_id: &str,
    items_by_key: &mut BTreeMap<String, DashboardIndexItem>,
) -> Result<()> {
    for entry in load_variant_index_entries(variant_dir)? {
        let entry_org_id = if entry.org_id.trim().is_empty() {
            org_id.to_string()
        } else {
            entry.org_id.clone()
        };
        let key = format!("{}:{}", entry_org_id, entry.uid);
        let relative = entry_relative_path(&entry.path, variant_dir, variant)?;
        let path = variant_dir.join(relative).display().to_string();
        let item = items_by_key
            .entry(key)
            .or_insert_with(|| DashboardIndexItem {
                uid: entry.uid.clone(),
                title: entry.title.clone(),
                folder_title: entry.folder_title.clone(),
                folder_uid: entry.folder_uid.clone(),
                folder_path: entry.folder_path.clone(),
                org: if entry.org.trim().is_empty() {
                    org_name.to_string()
                } else {
                    entry.org.clone()
                },
                org_id: entry_org_id,
                ownership: entry.ownership.clone(),
                provenance: entry.provenance.clone(),
                raw_path: None,
                prompt_path: None,
                provisioning_path: None,
            });
        if item.folder_title.is_empty() {
            item.folder_title = entry.folder_title;
        }
        if item.folder_uid.is_empty() {
            item.folder_uid = entry.folder_uid;
        }
        if item.folder_path.is_empty() {
            item.folder_path = entry.folder_path;
        }
        if item.ownership.is_empty() {
            item.ownership = entry.ownership;
        }
        if item.provenance.is_empty() {
            item.provenance = entry.provenance;
        }
        match variant {
            LayoutVariant::Raw => item.raw_path = Some(path),
            LayoutVariant::Prompt => item.prompt_path = Some(path),
        }
    }
    Ok(())
}

pub(super) fn copy_dir_recursive(source: &Path, target: &Path, overwrite: bool) -> Result<()> {
    if target.exists() && !overwrite {
        return Err(message(format!(
            "Refusing to overwrite existing output directory: {}. Use --overwrite.",
            target.display()
        )));
    }
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path, overwrite)?;
        } else {
            if target_path.exists() && !overwrite {
                return Err(message(format!(
                    "Refusing to overwrite existing file: {}. Use --overwrite.",
                    target_path.display()
                )));
            }
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source_path, target_path)?;
        }
    }
    Ok(())
}
