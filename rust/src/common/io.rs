use serde_json::Value;
use std::fs;
use std::path::Path;

use super::{strip_ansi_codes, validation, Result};

/// Write user-facing rendered output to disk without ANSI color codes.
pub fn write_plain_output_file(path: &Path, output: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let normalized = strip_ansi_codes(output).trim_end_matches('\n').to_string();
    fs::write(path, format!("{normalized}\n"))?;
    Ok(())
}

pub fn should_print_stdout(output_file: Option<&Path>, also_stdout: bool) -> bool {
    output_file.is_none() || also_stdout
}

/// Persist plain-text artifacts and optionally mirror them to stdout.
pub fn emit_plain_output(
    output: &str,
    output_file: Option<&Path>,
    also_stdout: bool,
) -> Result<()> {
    let normalized = output.trim_end_matches('\n');
    if normalized.is_empty() {
        return Ok(());
    }
    if let Some(path) = output_file {
        write_plain_output_file(path, normalized)?;
    }
    if should_print_stdout(output_file, also_stdout) {
        println!("{normalized}");
    }
    Ok(())
}

/// Load a JSON file and require the top-level value to be an object.
pub fn load_json_object_file(path: &Path, object_label: &str) -> Result<Value> {
    let raw = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&raw)?;
    if !value.is_object() {
        return Err(validation(format!(
            "{object_label} file must contain a JSON object: {}",
            path.display()
        )));
    }
    Ok(value)
}

/// Write JSON to disk with an explicit overwrite gate.
pub fn write_json_file(path: &Path, payload: &Value, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(validation(format!(
            "Refusing to overwrite existing file: {}. Use --overwrite.",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(payload)?),
    )?;
    Ok(())
}
