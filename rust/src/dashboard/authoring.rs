//! Live dashboard authoring helpers.
//!
//! These commands fetch one live dashboard, clear the numeric ID, and write a local
//! draft that can be edited and later imported with the existing dashboard import flow.
use reqwest::Method;
use serde_json::{Map, Value};
use std::fs;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{message, value_as_object, Result};
use crate::http::JsonHttpClient;

#[cfg(test)]
use super::fetch_dashboard_with_request;
use super::validate::validate_dashboard_import_document;
use super::{
    extract_dashboard_object, fetch_dashboard, load_json_file, write_dashboard,
    write_json_document, CloneLiveArgs, GetArgs, ImportArgs, PatchFileArgs, PublishArgs,
};

fn set_meta_folder_uid(document: &mut Map<String, Value>, folder_uid: &str) -> Result<()> {
    if folder_uid.is_empty() {
        return Ok(());
    }
    let meta = document
        .entry("meta".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let meta_object = meta.as_object_mut().ok_or_else(|| {
        message("Unexpected dashboard payload from Grafana: meta must be a JSON object.")
    })?;
    meta_object.insert(
        "folderUid".to_string(),
        Value::String(folder_uid.to_string()),
    );
    Ok(())
}

fn build_authoring_document(
    payload: &Value,
    title_override: Option<&str>,
    uid_override: Option<&str>,
    folder_uid_override: Option<&str>,
) -> Result<Value> {
    let object = value_as_object(payload, "Unexpected dashboard payload from Grafana.")?;
    let mut document = object.clone();
    let mut dashboard = extract_dashboard_object(&document)?.clone();
    dashboard.insert("id".to_string(), Value::Null);
    if let Some(title) = title_override {
        dashboard.insert("title".to_string(), Value::String(title.to_string()));
    }
    if let Some(uid) = uid_override {
        dashboard.insert("uid".to_string(), Value::String(uid.to_string()));
    }
    document.insert("dashboard".to_string(), Value::Object(dashboard));
    if let Some(folder_uid) = folder_uid_override {
        set_meta_folder_uid(&mut document, folder_uid)?;
    }
    Ok(Value::Object(document))
}

fn patch_dashboard_document(document: &mut Value, args: &PatchFileArgs) -> Result<()> {
    let object = document.as_object_mut().ok_or_else(|| {
        message("Dashboard patch-file expects a JSON object at the document root.")
    })?;
    let has_wrapper = object.contains_key("dashboard");
    if has_wrapper {
        let dashboard = object
            .get_mut("dashboard")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| {
                message("Dashboard patch-file expects dashboard to be a JSON object.")
            })?;
        if let Some(name) = args.name.as_ref() {
            dashboard.insert("title".to_string(), Value::String(name.clone()));
        }
        if let Some(uid) = args.uid.as_ref() {
            dashboard.insert("uid".to_string(), Value::String(uid.clone()));
        }
        if !args.tags.is_empty() {
            dashboard.insert(
                "tags".to_string(),
                Value::Array(args.tags.iter().cloned().map(Value::String).collect()),
            );
        }
    } else {
        if let Some(name) = args.name.as_ref() {
            object.insert("title".to_string(), Value::String(name.clone()));
        }
        if let Some(uid) = args.uid.as_ref() {
            object.insert("uid".to_string(), Value::String(uid.clone()));
        }
        if !args.tags.is_empty() {
            object.insert(
                "tags".to_string(),
                Value::Array(args.tags.iter().cloned().map(Value::String).collect()),
            );
        }
    }

    if args.folder_uid.is_some() || args.message.is_some() {
        let meta = object
            .entry("meta".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let meta = meta
            .as_object_mut()
            .ok_or_else(|| message("Dashboard patch-file expects meta to be a JSON object."))?;
        if let Some(folder_uid) = args.folder_uid.as_ref() {
            meta.insert("folderUid".to_string(), Value::String(folder_uid.clone()));
        }
        if let Some(message_text) = args.message.as_ref() {
            meta.insert("message".to_string(), Value::String(message_text.clone()));
        }
    }
    Ok(())
}

pub(crate) fn patch_dashboard_file(args: &PatchFileArgs) -> Result<()> {
    let mut document = load_json_file(&args.input)?;
    validate_dashboard_import_document(&document, &args.input, false, None)?;
    patch_dashboard_document(&mut document, args)?;
    let output_path = args.output.as_ref().unwrap_or(&args.input);
    write_json_document(&document, output_path)
}

fn build_publish_import_args(temp_input: PathBuf, args: &PublishArgs) -> ImportArgs {
    ImportArgs {
        common: args.common.clone(),
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        import_dir: temp_input,
        input_format: super::DashboardImportInputFormat::Raw,
        import_folder_uid: args.folder_uid.clone(),
        ensure_folders: false,
        replace_existing: args.replace_existing,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: args.message.clone(),
        interactive: false,
        dry_run: args.dry_run,
        table: args.table,
        json: args.json,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    }
}

pub(crate) fn publish_dashboard_with_request<F>(
    mut request_json: F,
    args: &PublishArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let document = load_json_file(&args.input)?;
    validate_dashboard_import_document(&document, &args.input, false, None)?;
    let temp_dir = std::env::temp_dir().join(format!(
        "grafana-dashboard-publish-{}-{}",
        process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| message(format!("Failed to build publish temp path: {error}")))?
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir)?;
    let result = (|| -> Result<()> {
        let staged_path = temp_dir.join("dashboard.json");
        fs::copy(&args.input, &staged_path)?;
        let import_args = build_publish_import_args(temp_dir.clone(), args);
        let _ = super::import::import_dashboards_with_request(&mut request_json, &import_args)?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

pub(crate) fn publish_dashboard_with_client(
    client: &JsonHttpClient,
    args: &PublishArgs,
) -> Result<()> {
    publish_dashboard_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )?;
    Ok(())
}

pub(crate) fn build_live_dashboard_authoring_document(
    payload: &Value,
    title_override: Option<&str>,
    uid_override: Option<&str>,
    folder_uid_override: Option<&str>,
) -> Result<Value> {
    build_authoring_document(payload, title_override, uid_override, folder_uid_override)
}

#[cfg(test)]
pub(crate) fn build_live_dashboard_authoring_document_with_request<F>(
    mut request_json: F,
    uid: &str,
    title_override: Option<&str>,
    uid_override: Option<&str>,
    folder_uid_override: Option<&str>,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let payload = fetch_dashboard_with_request(&mut request_json, uid)?;
    build_live_dashboard_authoring_document(
        &payload,
        title_override,
        uid_override,
        folder_uid_override,
    )
}

pub(crate) fn get_live_dashboard_to_file_with_client(
    client: &JsonHttpClient,
    args: &GetArgs,
) -> Result<()> {
    let payload = fetch_dashboard(client, &args.dashboard_uid)?;
    let document = build_live_dashboard_authoring_document(&payload, None, None, None)?;
    write_dashboard(&document, &args.output, false)
}

pub(crate) fn clone_live_dashboard_to_file_with_client(
    client: &JsonHttpClient,
    args: &CloneLiveArgs,
) -> Result<()> {
    let payload = fetch_dashboard(client, &args.source_uid)?;
    let document = build_live_dashboard_authoring_document(
        &payload,
        args.name.as_deref(),
        args.uid.as_deref(),
        args.folder_uid.as_deref(),
    )?;
    write_dashboard(&document, &args.output, false)
}

#[cfg(test)]
pub(crate) fn get_live_dashboard_to_file_with_request<F>(
    request_json: F,
    uid: &str,
    output: &Path,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let document =
        build_live_dashboard_authoring_document_with_request(request_json, uid, None, None, None)?;
    write_dashboard(&document, output, false)?;
    Ok(document)
}

#[cfg(test)]
pub(crate) fn clone_live_dashboard_to_file_with_request<F>(
    request_json: F,
    source_uid: &str,
    output: &Path,
    title_override: Option<&str>,
    uid_override: Option<&str>,
    folder_uid_override: Option<&str>,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let document = build_live_dashboard_authoring_document_with_request(
        request_json,
        source_uid,
        title_override,
        uid_override,
        folder_uid_override,
    )?;
    write_dashboard(&document, output, false)?;
    Ok(document)
}
