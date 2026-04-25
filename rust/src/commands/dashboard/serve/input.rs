use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::Serialize;
use serde_json::Value;

use crate::common::{message, string_field, value_as_object, Result};

use super::super::{
    extract_dashboard_object, load_json_file, DashboardServeScriptFormat, ServeArgs,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PathFingerprint {
    modified_millis: u128,
    len: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DashboardServeItem {
    pub(super) title: String,
    pub(super) uid: String,
    pub(super) source: String,
    pub(super) document_kind: String,
    pub(super) dashboard: Value,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DashboardServeDocument {
    pub(super) item_count: usize,
    pub(super) items: Vec<DashboardServeItem>,
    pub(super) last_reload_millis: u128,
    pub(super) last_error: Option<String>,
}

fn walk_dashboard_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if root.is_file() {
        files.push(root.to_path_buf());
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dashboard_files(&path, files)?;
            continue;
        }
        let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if matches!(ext, "json" | "yaml" | "yml") {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_dashboard_value(value: Value, source: String) -> Result<DashboardServeItem> {
    let object = value_as_object(&value, "Dashboard serve expects JSON/YAML objects.")?;
    let dashboard = extract_dashboard_object(object)?;
    Ok(DashboardServeItem {
        title: string_field(dashboard, "title", "dashboard"),
        uid: string_field(dashboard, "uid", source.as_str()),
        source,
        document_kind: if object.contains_key("dashboard") {
            "wrapped".to_string()
        } else {
            "bare".to_string()
        },
        dashboard: value,
    })
}

fn parse_script_output(raw: &str, yaml: bool) -> Result<Vec<Value>> {
    let value = if yaml {
        serde_yaml::from_str::<Value>(raw).map_err(|error| {
            message(format!(
                "Failed to parse dashboard serve YAML script output: {error}"
            ))
        })?
    } else {
        serde_json::from_str::<Value>(raw).map_err(|error| {
            message(format!(
                "Failed to parse dashboard serve JSON script output: {error}"
            ))
        })?
    };
    match value {
        Value::Array(items) => Ok(items),
        other => Ok(vec![other]),
    }
}

pub(super) fn load_input_items(args: &ServeArgs) -> Result<Vec<DashboardServeItem>> {
    if let Some(script) = args.script.as_ref() {
        let output = Command::new("/bin/sh")
            .arg("-lc")
            .arg(script)
            .output()
            .map_err(|error| message(format!("Failed to run dashboard serve script: {error}")))?;
        if !output.status.success() {
            return Err(message(format!(
                "Dashboard serve script exited with status {}.",
                output.status
            )));
        }
        return parse_script_output(
            std::str::from_utf8(&output.stdout).map_err(|error| {
                message(format!(
                    "Dashboard serve script stdout is not UTF-8: {error}"
                ))
            })?,
            matches!(args.script_format, DashboardServeScriptFormat::Yaml),
        )?
        .into_iter()
        .enumerate()
        .map(|(index, value)| parse_dashboard_value(value, format!("script:{index}")))
        .collect();
    }

    let input = args
        .input
        .as_ref()
        .ok_or_else(|| message("dashboard serve requires --input or --script."))?;
    let mut files = Vec::new();
    walk_dashboard_files(input, &mut files)?;
    if files.is_empty() {
        return Err(message(format!(
            "No dashboard files found under {}.",
            input.display()
        )));
    }
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let value = match path.extension().and_then(|value| value.to_str()) {
                Some("yaml") | Some("yml") => {
                    serde_yaml::from_str::<Value>(&fs::read_to_string(&path)?).map_err(|error| {
                        message(format!(
                            "Failed to parse dashboard YAML file {}: {error}",
                            path.display()
                        ))
                    })?
                }
                _ => load_json_file(&path)?,
            };
            parse_dashboard_value(value, path.display().to_string())
        })
        .collect()
}

pub(super) fn current_fingerprint(path: &Path) -> Result<Option<PathFingerprint>> {
    match fs::metadata(path) {
        Ok(metadata) => {
            let modified = metadata
                .modified()?
                .duration_since(UNIX_EPOCH)
                .map_err(|error| {
                    message(format!(
                        "Dashboard serve file timestamp is before UNIX_EPOCH for {}: {error}",
                        path.display()
                    ))
                })?;
            Ok(Some(PathFingerprint {
                modified_millis: modified.as_millis(),
                len: metadata.len(),
            }))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
