use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{message, Result};
use crate::dashboard::list;
use crate::dashboard::source_loader::resolve_dashboard_workspace_variant_dir;
use crate::dashboard::{
    load_datasource_inventory, load_export_metadata, load_folder_inventory,
    DashboardImportInputFormat, DashboardSourceKind, ExportMetadata, ImportArgs, VariantIndexEntry,
    RAW_EXPORT_SUBDIR,
};
use crate::grafana_api::DashboardResourceClient;

use super::super::lookup::{
    list_orgs_cached, resolve_import_target_org_id_with_request, ImportLookupCache,
};

fn load_export_org_ids(
    input_dir: &Path,
    metadata: Option<&ExportMetadata>,
) -> Result<BTreeSet<String>> {
    let mut org_ids = BTreeSet::new();
    let index_file = metadata
        .map(|item| item.index_file.clone())
        .unwrap_or_else(|| "index.json".to_string());
    let index_path = input_dir.join(&index_file);
    if index_path.is_file() {
        let raw = fs::read_to_string(&index_path)?;
        let entries: Vec<VariantIndexEntry> = serde_json::from_str(&raw)?;
        for entry in entries {
            let org_id = entry.org_id.trim();
            if !org_id.is_empty() {
                org_ids.insert(org_id.to_string());
            }
        }
    }

    for folder in load_folder_inventory(input_dir, metadata)? {
        let org_id = folder.org_id.trim();
        if !org_id.is_empty() {
            org_ids.insert(org_id.to_string());
        }
    }
    for datasource in load_datasource_inventory(input_dir, metadata)? {
        let org_id = datasource.org_id.trim();
        if !org_id.is_empty() {
            org_ids.insert(org_id.to_string());
        }
    }
    Ok(org_ids)
}

fn load_export_org_names(
    input_dir: &Path,
    metadata: Option<&ExportMetadata>,
) -> Result<BTreeSet<String>> {
    let mut org_names = BTreeSet::new();
    let index_file = metadata
        .map(|item| item.index_file.clone())
        .unwrap_or_else(|| "index.json".to_string());
    let index_path = input_dir.join(&index_file);
    if index_path.is_file() {
        let raw = fs::read_to_string(&index_path)?;
        let entries: Vec<VariantIndexEntry> = serde_json::from_str(&raw)?;
        for entry in entries {
            let org_name = entry.org.trim();
            if !org_name.is_empty() {
                org_names.insert(org_name.to_string());
            }
        }
    }

    for folder in load_folder_inventory(input_dir, metadata)? {
        let org_name = folder.org.trim();
        if !org_name.is_empty() {
            org_names.insert(org_name.to_string());
        }
    }
    for datasource in load_datasource_inventory(input_dir, metadata)? {
        let org_name = datasource.org.trim();
        if !org_name.is_empty() {
            org_names.insert(org_name.to_string());
        }
    }
    Ok(org_names)
}

#[derive(Debug, Clone)]
pub(crate) struct ExportOrgImportScope {
    pub source_org_id: i64,
    pub source_org_name: String,
    pub input_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExportOrgTargetPlan {
    pub source_org_id: i64,
    pub source_org_name: String,
    pub target_org_id: Option<i64>,
    pub org_action: &'static str,
    pub input_dir: PathBuf,
}

fn org_id_string_from_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.trim().to_string(),
        Some(Value::Number(number)) => number.to_string(),
        _ => String::new(),
    }
}

fn parse_export_org_scope(import_root: &Path, raw_dir: &Path) -> Result<ExportOrgImportScope> {
    parse_export_org_scope_for_variant(import_root, raw_dir, RAW_EXPORT_SUBDIR)
}

fn use_export_org_variant_dir(input_format: DashboardImportInputFormat) -> &'static str {
    DashboardSourceKind::from_import_input_format(input_format)
        .expected_variant()
        .expect("dashboard import input format must map to an export variant")
}

fn has_export_org_scopes(input_dir: &Path, variant_dir_name: &str) -> Result<bool> {
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|item| item.to_str()) else {
            continue;
        };
        if name.starts_with("org_") && path.join(variant_dir_name).is_dir() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn normalize_export_org_scopes_root(input_dir: &Path, variant_dir_name: &str) -> Result<PathBuf> {
    if has_export_org_scopes(input_dir, variant_dir_name)? {
        return Ok(input_dir.to_path_buf());
    }
    if let Some(workspace_variant_dir) =
        resolve_dashboard_workspace_variant_dir(input_dir, variant_dir_name)
    {
        if has_export_org_scopes(&workspace_variant_dir, variant_dir_name)? {
            return Ok(workspace_variant_dir);
        }
    }
    Ok(input_dir.to_path_buf())
}

fn parse_export_org_scope_for_variant(
    import_root: &Path,
    variant_dir: &Path,
    expected_variant: &'static str,
) -> Result<ExportOrgImportScope> {
    let metadata = load_export_metadata(variant_dir, Some(expected_variant))?;
    let export_org_ids = load_export_org_ids(variant_dir, metadata.as_ref())?;
    if export_org_ids.is_empty() {
        return Err(message(format!(
            "Cannot route import by export org for {}: {expected_variant} export orgId metadata was not found in index.json, folders.json, or datasources.json.",
            variant_dir.display()
        )));
    }
    if export_org_ids.len() > 1 {
        return Err(message(format!(
            "Cannot route import by export org for {}: found multiple export orgIds ({}).",
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
            "Cannot route import by export org for {}: export orgId '{}' is not a valid integer.",
            variant_dir.display(),
            source_org_id_text
        ))
    })?;
    let org_names = load_export_org_names(variant_dir, metadata.as_ref())?;
    if org_names.len() > 1 {
        return Err(message(format!(
            "Cannot route import by export org for {}: found multiple export org names ({}).",
            variant_dir.display(),
            org_names.into_iter().collect::<Vec<String>>().join(", ")
        )));
    }
    let source_org_name = org_names.into_iter().next().unwrap_or_else(|| {
        import_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("org")
            .to_string()
    });
    Ok(ExportOrgImportScope {
        source_org_id,
        source_org_name,
        input_dir: variant_dir.to_path_buf(),
    })
}

pub(crate) fn discover_export_org_import_scopes(
    args: &ImportArgs,
) -> Result<Vec<ExportOrgImportScope>> {
    if !args.use_export_org {
        return Ok(Vec::new());
    }
    let variant_dir_name = use_export_org_variant_dir(args.input_format);
    let scan_root = normalize_export_org_scopes_root(&args.input_dir, variant_dir_name)?;
    let selected_org_ids: BTreeSet<i64> = args.only_org_id.iter().copied().collect();
    let mut scopes = Vec::new();
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
        let scope = if variant_dir_name == RAW_EXPORT_SUBDIR {
            parse_export_org_scope(&path, &variant_dir)?
        } else {
            parse_export_org_scope_for_variant(&path, &variant_dir, variant_dir_name)?
        };
        if !selected_org_ids.is_empty() && !selected_org_ids.contains(&scope.source_org_id) {
            continue;
        }
        scopes.push(scope);
    }
    scopes.sort_by_key(|scope| scope.source_org_id);
    if scopes.is_empty() {
        if let Some(metadata) = load_export_metadata(&scan_root, None)? {
            if metadata.variant != "root" {
                return Err(message(format!(
                    "Dashboard import with --use-export-org expects the combined export root, not one {variant_dir_name}/ export directory."
                )));
            }
        }
        if selected_org_ids.is_empty() {
            return Err(message(format!(
                "Dashboard import with --use-export-org did not find any org-specific {variant_dir_name} exports under {}.",
                scan_root.display(),
            )));
        }
        return Err(message(format!(
            "Dashboard import with --use-export-org did not find the selected exported org IDs ({}) under {}.",
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
            "Dashboard import with --use-export-org did not find the selected exported org IDs ({}).",
            missing_org_ids.join(", ")
        )));
    }
    Ok(scopes)
}

pub(crate) fn validate_matching_export_org_with_request<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
    args: &ImportArgs,
    input_dir: &Path,
    metadata: Option<&ExportMetadata>,
    target_org_id_override: Option<i64>,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if !args.require_matching_export_org {
        return Ok(());
    }
    let export_org_ids = load_export_org_ids(input_dir, metadata)?;
    if export_org_ids.is_empty() {
        return Err(message(
            "Cannot verify exported org for import: export orgId metadata was not found in index.json, folders.json, or datasources.json.",
        ));
    }
    if export_org_ids.len() > 1 {
        return Err(message(format!(
            "Cannot verify exported org for import: found multiple export orgIds ({}).",
            export_org_ids
                .into_iter()
                .collect::<Vec<String>>()
                .join(", ")
        )));
    }
    let export_org_id = export_org_ids.into_iter().next().unwrap_or_default();
    let target_org_id = match target_org_id_override {
        Some(org_id) => org_id.to_string(),
        None => resolve_import_target_org_id_with_request(request_json, cache, args)?,
    };
    if export_org_id != target_org_id {
        return Err(message(format!(
            "Dashboard import export org mismatch: export orgId {export_org_id} does not match target org {target_org_id}. Use matching credentials/org selection or omit --require-matching-export-org."
        )));
    }
    Ok(())
}

fn current_org_id_with_client(client: &DashboardResourceClient<'_>) -> Result<String> {
    let org = client.fetch_current_org()?;
    list::org_id_value(&org).map(|value| value.to_string())
}

pub(crate) fn validate_matching_export_org_with_client(
    client: &DashboardResourceClient<'_>,
    args: &ImportArgs,
    input_dir: &Path,
    metadata: Option<&ExportMetadata>,
    target_org_id_override: Option<i64>,
) -> Result<()> {
    if !args.require_matching_export_org {
        return Ok(());
    }
    let export_org_ids = load_export_org_ids(input_dir, metadata)?;
    if export_org_ids.is_empty() {
        return Err(message(
            "Cannot verify exported org for import: export orgId metadata was not found in index.json, folders.json, or datasources.json.",
        ));
    }
    if export_org_ids.len() > 1 {
        return Err(message(format!(
            "Cannot verify exported org for import: found multiple export orgIds ({}).",
            export_org_ids
                .into_iter()
                .collect::<Vec<String>>()
                .join(", ")
        )));
    }
    let export_org_id = export_org_ids.into_iter().next().unwrap_or_default();
    let target_org_id = match target_org_id_override {
        Some(org_id) => org_id.to_string(),
        None => current_org_id_with_client(client)?,
    };
    if export_org_id != target_org_id {
        return Err(message(format!(
            "Dashboard import export org mismatch: export orgId {export_org_id} does not match target org {target_org_id}. Use matching credentials/org selection or omit --require-matching-export-org."
        )));
    }
    Ok(())
}

pub(crate) fn resolve_target_org_plan_for_export_scope_with_request<F>(
    request_json: &mut F,
    cache: &mut ImportLookupCache,
    args: &super::super::ImportArgs,
    scope: &ExportOrgImportScope,
) -> Result<ExportOrgTargetPlan>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let orgs = list_orgs_cached(request_json, cache)?;
    for org in &orgs {
        let org_id_text = org_id_string_from_value(org.get("id"));
        if org_id_text == scope.source_org_id.to_string() {
            return Ok(ExportOrgTargetPlan {
                source_org_id: scope.source_org_id,
                source_org_name: scope.source_org_name.clone(),
                target_org_id: Some(scope.source_org_id),
                org_action: "exists",
                input_dir: scope.input_dir.clone(),
            });
        }
    }
    if args.dry_run && !args.create_missing_orgs {
        return Ok(ExportOrgTargetPlan {
            source_org_id: scope.source_org_id,
            source_org_name: scope.source_org_name.clone(),
            target_org_id: None,
            org_action: "missing",
            input_dir: scope.input_dir.clone(),
        });
    }
    if !args.create_missing_orgs {
        return Err(message(format!(
            "Dashboard import could not find destination Grafana org {} ({}) for --use-export-org. Use --create-missing-orgs to create it first.",
            scope.source_org_id, scope.source_org_name
        )));
    }
    if scope.source_org_name.trim().is_empty() {
        return Err(message(format!(
            "Dashboard import with --create-missing-orgs could not determine an exported org name for source orgId {}.",
            scope.source_org_id
        )));
    }
    if args.dry_run {
        return Ok(ExportOrgTargetPlan {
            source_org_id: scope.source_org_id,
            source_org_name: scope.source_org_name.clone(),
            target_org_id: None,
            org_action: "would-create",
            input_dir: scope.input_dir.clone(),
        });
    }
    let created = create_org_with_request(&mut *request_json, &scope.source_org_name)?;
    let created_org_id =
        org_id_string_from_value(created.get("orgId").or_else(|| created.get("id")));
    if created_org_id.is_empty() {
        return Err(message(format!(
            "Grafana did not return a usable orgId after creating destination org '{}' for exported org {}.",
            scope.source_org_name, scope.source_org_id
        )));
    }
    let parsed_org_id = created_org_id.parse::<i64>().map_err(|_| {
        message(format!(
            "Grafana returned non-numeric orgId '{}' after creating destination org '{}' for exported org {}.",
            created_org_id, scope.source_org_name, scope.source_org_id
        ))
    })?;
    Ok(ExportOrgTargetPlan {
        source_org_id: scope.source_org_id,
        source_org_name: scope.source_org_name.clone(),
        target_org_id: Some(parsed_org_id),
        org_action: "created",
        input_dir: scope.input_dir.clone(),
    })
}

fn create_org_with_request<F>(mut request_json: F, org_name: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(
        Method::POST,
        "/api/orgs",
        &[],
        Some(&Value::Object(Map::from_iter(vec![(
            "name".to_string(),
            Value::String(org_name.to_string()),
        )]))),
    )? {
        Some(Value::Object(object)) => Ok(object),
        _ => Err(message(
            "Unexpected organization create response from Grafana during dashboard import.",
        )),
    }
}
