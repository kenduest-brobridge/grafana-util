use serde_json::Value;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::common::{message, Result};

use super::{load_json_file, PatchFileArgs, PublishArgs};

const STDIN_PATH_TOKEN: &str = "-";
const STDIN_DISPLAY_LABEL: &str = "<stdin>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardInputDocument {
    pub(crate) document: Value,
    pub(crate) display_label: String,
    pub(crate) validation_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FileWatchFingerprint {
    modified_millis: u128,
    len: u64,
}

fn is_stdin_path(path: &Path) -> bool {
    path.to_str() == Some(STDIN_PATH_TOKEN)
}

fn parse_dashboard_input_json(raw: &str, source_label: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        message(format!(
            "Failed to parse dashboard JSON from {source_label}: {error}"
        ))
    })?;
    if !value.is_object() {
        return Err(message(format!(
            "Dashboard input must contain a JSON object: {source_label}"
        )));
    }
    Ok(value)
}

pub(crate) fn load_dashboard_input_document_from_reader<R: Read>(
    mut reader: R,
    source_label: &str,
    validation_path: PathBuf,
) -> Result<DashboardInputDocument> {
    let mut raw = String::new();
    reader.read_to_string(&mut raw)?;
    let document = parse_dashboard_input_json(&raw, source_label)?;
    Ok(DashboardInputDocument {
        document,
        display_label: source_label.to_string(),
        validation_path,
    })
}

pub(super) fn load_dashboard_input_document(input: &Path) -> Result<DashboardInputDocument> {
    if is_stdin_path(input) {
        return load_dashboard_input_document_from_reader(
            io::stdin(),
            STDIN_DISPLAY_LABEL,
            PathBuf::from(STDIN_DISPLAY_LABEL),
        );
    }

    Ok(DashboardInputDocument {
        document: load_json_file(input)?,
        display_label: input.display().to_string(),
        validation_path: input.to_path_buf(),
    })
}

pub(super) fn validate_publish_args(args: &PublishArgs) -> Result<()> {
    if args.watch && is_stdin_path(&args.input) {
        return Err(message(
            "--watch cannot be combined with --input -. Point --input at a local file.",
        ));
    }
    Ok(())
}

pub(super) fn validate_patch_file_args(args: &PatchFileArgs) -> Result<()> {
    if is_stdin_path(&args.input) && args.output.is_none() {
        return Err(message(
            "patch --input - requires --output because standard input cannot be overwritten in place.",
        ));
    }
    Ok(())
}

pub(super) fn current_file_watch_fingerprint(path: &Path) -> Result<Option<FileWatchFingerprint>> {
    match fs::metadata(path) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .map_err(|error| {
                    message(format!(
                        "Failed to read file modification time for {}: {error}",
                        path.display()
                    ))
                })?
                .duration_since(UNIX_EPOCH)
                .map_err(|error| {
                    message(format!(
                        "File modification time is before UNIX_EPOCH for {}: {error}",
                        path.display()
                    ))
                })?;
            Ok(Some(FileWatchFingerprint {
                modified_millis: modified.as_millis(),
                len: metadata.len(),
            }))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn watch_event_targets_input_path(paths: &[PathBuf], input_path: &Path) -> bool {
    paths.iter().any(|path| {
        path == input_path
            || (path.file_name() == input_path.file_name() && path.parent() == input_path.parent())
    })
}

pub(crate) fn watch_start_message(path: &Path) -> String {
    format!(
        "Watching {} for dashboard publish changes. Press Ctrl-C to stop.",
        path.display()
    )
}

pub(crate) fn watch_change_detected_message(path: &Path) -> String {
    format!(
        "Detected dashboard input change for {}; waiting for a stable save.",
        path.display()
    )
}

pub(crate) fn watch_change_unstable_message(path: &Path) -> String {
    format!(
        "Dashboard input changed again before it stabilized; still watching {}.",
        path.display()
    )
}
