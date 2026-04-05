//! External-editor live dashboard edit flow with a safe local-draft default.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::common::{message, string_field, value_as_object, Result};
use crate::http::JsonHttpClient;

use super::{
    extract_dashboard_object, fetch_dashboard, import_dashboard_request, EditLiveArgs,
    DEFAULT_IMPORT_MESSAGE,
};

fn temp_edit_path(uid: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    env::temp_dir().join(format!(
        "grafana-util-dashboard-edit-{uid}-{timestamp}.json"
    ))
}

fn default_output_path(uid: &str) -> PathBuf {
    PathBuf::from(format!("{uid}.edited.json"))
}

fn build_wrapped_live_document(payload: &Value) -> Result<Value> {
    let object = value_as_object(payload, "Unexpected dashboard payload from Grafana.")?;
    let dashboard = extract_dashboard_object(object)?.clone();
    let mut document = Map::new();
    document.insert("dashboard".to_string(), Value::Object(dashboard));
    if let Some(folder_uid) = object
        .get("meta")
        .and_then(Value::as_object)
        .map(|meta| string_field(meta, "folderUid", ""))
        .filter(|value| !value.is_empty())
    {
        document.insert("folderUid".to_string(), Value::String(folder_uid));
    }
    Ok(Value::Object(document))
}

fn validate_edited_document(value: &Value) -> Result<()> {
    let object = value_as_object(value, "Edited dashboard payload must be a JSON object.")?;
    let dashboard = extract_dashboard_object(object)?;
    if dashboard
        .get("uid")
        .and_then(Value::as_str)
        .filter(|uid| !uid.trim().is_empty())
        .is_none()
    {
        return Err(message(
            "Edited dashboard payload must include dashboard.uid.",
        ));
    }
    Ok(())
}

fn write_temp_payload(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

fn run_editor_command(path: &Path) -> Result<()> {
    let editor = env::var("VISUAL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("EDITOR")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "vi".to_string());
    let mut parts = editor.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| message("Could not resolve an external editor command."))?;
    let mut command = Command::new(program);
    for part in parts {
        command.arg(part);
    }
    let status = command.arg(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(message(format!(
            "External editor exited with status {status}."
        )))
    }
}

fn edit_payload_in_external_editor(uid: &str, value: &Value) -> Result<Option<Value>> {
    let temp_path = temp_edit_path(uid);
    write_temp_payload(&temp_path, value)?;
    let result = (|| -> Result<Option<Value>> {
        run_editor_command(&temp_path)?;
        let edited = fs::read_to_string(&temp_path)?;
        let parsed: Value = serde_json::from_str(&edited).map_err(|error| {
            message(format!(
                "Edited dashboard JSON is invalid in {}: {error}",
                temp_path.display()
            ))
        })?;
        validate_edited_document(&parsed)?;
        if parsed == *value {
            Ok(None)
        } else {
            Ok(Some(parsed))
        }
    })();
    let _ = fs::remove_file(&temp_path);
    result
}

fn summarize_changes(original: &Value, edited: &Value) -> Result<Vec<String>> {
    let original_object = value_as_object(
        original,
        "Original dashboard edit payload must be an object.",
    )?;
    let edited_object = value_as_object(edited, "Edited dashboard payload must be an object.")?;
    let original_dashboard = extract_dashboard_object(original_object)?;
    let edited_dashboard = extract_dashboard_object(edited_object)?;
    Ok(vec![
        format!(
            "Dashboard edit review uid={} title={} -> {}",
            string_field(edited_dashboard, "uid", ""),
            string_field(original_dashboard, "title", ""),
            string_field(edited_dashboard, "title", "")
        ),
        format!(
            "Folder UID: {} -> {}",
            string_field(original_object, "folderUid", "-"),
            string_field(edited_object, "folderUid", "-"),
        ),
    ])
}

pub(crate) fn run_dashboard_edit_live(
    client: Option<&JsonHttpClient>,
    args: &EditLiveArgs,
) -> Result<()> {
    let client =
        client.ok_or_else(|| message("Dashboard edit-live requires a live Grafana client."))?;
    if args.apply_live && !args.yes {
        return Err(message(
            "--apply-live requires --yes because it writes the edited dashboard back to Grafana.",
        ));
    }

    let live_payload = fetch_dashboard(client, &args.dashboard_uid)?;
    let wrapped = build_wrapped_live_document(&live_payload)?;
    let Some(edited) = edit_payload_in_external_editor(&args.dashboard_uid, &wrapped)? else {
        println!(
            "No dashboard changes detected for {}. Nothing written.",
            args.dashboard_uid
        );
        return Ok(());
    };

    for line in summarize_changes(&wrapped, &edited)? {
        println!("{line}");
    }

    if args.apply_live {
        let payload = value_as_object(&edited, "Edited dashboard payload must be an object.")?;
        let mut import_payload = payload.clone();
        import_payload.insert("overwrite".to_string(), Value::Bool(true));
        import_payload.insert(
            "message".to_string(),
            Value::String(if args.message.is_empty() {
                DEFAULT_IMPORT_MESSAGE.to_string()
            } else {
                args.message.clone()
            }),
        );
        let _ = import_dashboard_request(client, &Value::Object(import_payload))?;
        println!(
            "Applied edited dashboard {} back to Grafana.",
            args.dashboard_uid
        );
        return Ok(());
    }

    let output = args
        .output
        .clone()
        .unwrap_or_else(|| default_output_path(&args.dashboard_uid));
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, serde_json::to_string_pretty(&edited)? + "\n")?;
    println!(
        "Wrote edited dashboard draft for {} to {}.",
        args.dashboard_uid,
        output.display()
    );
    Ok(())
}
