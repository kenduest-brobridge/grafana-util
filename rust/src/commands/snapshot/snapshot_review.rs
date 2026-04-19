use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::common::Result;
use crate::dashboard::TempInspectDir;
use crate::overview::{OverviewArgs, OverviewOutputFormat};
use crate::staged_export_scopes::resolve_datasource_export_scope_dirs;

use super::snapshot_artifacts::{build_snapshot_paths, resolve_snapshot_review_input_dir};
use super::snapshot_metadata::export_scope_kind_from_metadata_value;

#[allow(dead_code)]
pub fn build_snapshot_overview_args(args: &super::super::SnapshotReviewArgs) -> OverviewArgs {
    let input_dir =
        resolve_snapshot_review_input_dir(args).unwrap_or_else(|_| args.input_dir.clone());
    let paths = build_snapshot_paths(&input_dir);
    OverviewArgs {
        dashboard_export_dir: Some(paths.dashboards),
        dashboard_provisioning_dir: None,
        datasource_export_dir: Some(paths.datasources),
        datasource_provisioning_file: None,
        access_user_export_dir: Some(paths.access_users),
        access_team_export_dir: Some(paths.access_teams),
        access_org_export_dir: Some(paths.access_orgs),
        access_service_account_export_dir: Some(paths.access_service_accounts),
        desired_file: None,
        source_bundle: None,
        target_inventory: None,
        alert_export_dir: None,
        availability_file: None,
        mapping_file: None,
        output_format: args.output_format,
    }
}

fn normalize_snapshot_datasource_dir(temp_root: &Path, datasource_dir: &Path) -> Result<PathBuf> {
    let metadata_path = datasource_dir.join(super::super::SNAPSHOT_DATASOURCE_EXPORT_METADATA_FILENAME);
    if !metadata_path.is_file() {
        return Ok(datasource_dir.to_path_buf());
    }

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&metadata_path)?)?;
    let kind = metadata
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let resource = metadata
        .get("resource")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if kind != super::super::SNAPSHOT_DATASOURCE_ROOT_INDEX_KIND
        || resource != "datasource"
        || !matches!(
            export_scope_kind_from_metadata_value(&metadata),
            "all-orgs-root" | "workspace-root"
        )
    {
        return Ok(datasource_dir.to_path_buf());
    }

    let mut merged = Vec::new();
    let mut seen_rows = BTreeMap::<String, ()>::new();
    let mut append_rows = |rows: Vec<Value>| -> Result<()> {
        for row in rows {
            let key = serde_json::to_string(&row)?;
            if seen_rows.insert(key, ()).is_none() {
                merged.push(row);
            }
        }
        Ok(())
    };

    let root_datasources_path = datasource_dir.join(super::super::SNAPSHOT_DATASOURCE_EXPORT_FILENAME);
    if root_datasources_path.is_file() {
        let rows: Vec<Value> = serde_json::from_str(&fs::read_to_string(&root_datasources_path)?)?;
        append_rows(rows)?;
    }

    let scope_dirs = resolve_datasource_export_scope_dirs(datasource_dir);
    for path in scope_dirs {
        if path == datasource_dir {
            continue;
        }
        let datasources_path = path.join(super::super::SNAPSHOT_DATASOURCE_EXPORT_FILENAME);
        if !datasources_path.is_file() {
            continue;
        }
        let rows: Vec<Value> = serde_json::from_str(&fs::read_to_string(&datasources_path)?)?;
        append_rows(rows)?;
    }

    let normalized_dir = temp_root.join("snapshot-review-datasources");
    fs::create_dir_all(&normalized_dir)?;
    fs::write(
        normalized_dir.join(super::super::SNAPSHOT_DATASOURCE_EXPORT_FILENAME),
        serde_json::to_string_pretty(&merged)?,
    )?;
    fs::write(
        normalized_dir.join(super::super::SNAPSHOT_DATASOURCE_EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": super::super::SNAPSHOT_DATASOURCE_TOOL_SCHEMA_VERSION,
            "kind": super::super::SNAPSHOT_DATASOURCE_ROOT_INDEX_KIND,
            "variant": "root",
            "resource": "datasource",
            "datasourceCount": merged.len(),
            "datasourcesFile": super::super::SNAPSHOT_DATASOURCE_EXPORT_FILENAME,
            "indexFile": "index.json",
            "format": "grafana-datasource-inventory-v1",
        }))?,
    )?;
    Ok(normalized_dir)
}

// Build normalized snapshot review documents once, then route to a caller-provided renderer.
pub(crate) fn run_snapshot_review_document_with_handler<FO>(
    args: super::super::SnapshotReviewArgs,
    mut run_review: FO,
) -> Result<()>
where
    FO: FnMut(Value) -> Result<()>,
{
    let input_dir = resolve_snapshot_review_input_dir(&args)?;
    let paths = build_snapshot_paths(&input_dir);
    let temp_dir = TempInspectDir::new("snapshot-review")?;
    let datasource_dir = normalize_snapshot_datasource_dir(&temp_dir.path, &paths.datasources)?;
    let document = super::super::build_snapshot_review_document(
        &paths.dashboards,
        &datasource_dir,
        &paths.datasources,
    )?;
    run_review(document)
}

#[allow(dead_code)]
pub fn run_snapshot_review(args: super::super::SnapshotReviewArgs) -> Result<()> {
    // Review path is output-format routing over a prebuilt snapshot document;
    // it intentionally does not mutate source artifacts.
    let output = if args.interactive {
        #[cfg(feature = "tui")]
        {
            OverviewOutputFormat::Interactive
        }
        #[cfg(not(feature = "tui"))]
        {
            OverviewOutputFormat::Text
        }
    } else {
        args.output_format
    };
    run_snapshot_review_document_with_handler(args, move |document| {
        super::super::snapshot_review::emit_snapshot_review_output(&document, output)
    })
}
