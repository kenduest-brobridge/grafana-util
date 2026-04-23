//! Shared foundation for all Rust CLI domains.
//!
//! Responsibilities:
//! - Provide one canonical `Result` and `GrafanaCliError` API shared by all modules.
//! - Centralize auth/header derivation, interactive credential prompting, and input parsing.
//! - Own generic JSON helpers, FS helpers, and output serializers that keep command behavior uniform.
use serde_json::{Map, Value};

mod auth;
mod diff_document;
mod error;
mod io;
mod json_output;
mod normalize;
#[allow(unused_imports)]
pub use auth::{env_value, resolve_auth_headers};
#[allow(unused_imports)]
pub use diff_document::{build_shared_diff_document, DiffOutputFormat, SharedDiffSummary};
#[allow(unused_imports)]
pub use error::{
    api_response, editor, invalid_header_name, invalid_header_value, invalid_url, message,
    parse_error, tui, validation, GrafanaCliError, Result,
};
#[allow(unused_imports)]
pub use io::{
    emit_plain_output, load_json_object_file, should_print_stdout, write_json_file,
    write_plain_output_file,
};
#[allow(unused_imports)]
pub use json_output::{
    json_color_choice, json_color_enabled, render_json_value, render_json_value_with_choice,
    set_json_color_choice, strip_ansi_codes, CliColorChoice,
};
#[allow(unused_imports)]
pub use normalize::sanitize_path_component;

/// Print one supported column id per line for discoverability-style CLI flags.
pub fn print_supported_columns(columns: &[&str]) {
    println!("{}", columns.join("\n"));
}

/// Return whether a selected column list asked for the full human-readable set.
pub fn requested_columns_include_all(columns: &[String]) -> bool {
    columns.iter().any(|value| value == "all")
}

/// Canonical grafana-util version embedded in emitted JSON documents.
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Schema version for machine-readable CLI version output.
pub const TOOL_VERSION_SCHEMA_VERSION: i64 = 1;
/// UTC timestamp recorded when this binary was built.
pub const TOOL_BUILD_TIME: &str = env!("GRAFANA_UTIL_BUILD_TIME");
/// Short Git commit recorded for this binary build.
pub const TOOL_GIT_COMMIT: &str = env!("GRAFANA_UTIL_GIT_COMMIT");
/// Canonical version payload handed to Clap for `--version`.
pub const TOOL_VERSION_DETAILS: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\ncommit: ",
    env!("GRAFANA_UTIL_GIT_COMMIT"),
    "\nbuild time: ",
    env!("GRAFANA_UTIL_BUILD_TIME")
);
/// Canonical user-facing version text for CLI version output.
pub const TOOL_VERSION_TEXT: &str = concat!(
    "grafana-util ",
    env!("CARGO_PKG_VERSION"),
    "\ncommit: ",
    env!("GRAFANA_UTIL_GIT_COMMIT"),
    "\nbuild time: ",
    env!("GRAFANA_UTIL_BUILD_TIME"),
    "\n"
);

/// Return the current grafana-util version for staged/export/status metadata.
pub fn tool_version() -> &'static str {
    TOOL_VERSION
}

/// Return the UTC build timestamp recorded for this binary.
pub fn tool_build_time() -> &'static str {
    TOOL_BUILD_TIME
}

/// Return the short Git commit recorded for this binary.
pub fn tool_git_commit() -> &'static str {
    TOOL_GIT_COMMIT
}

/// Require a JSON value to be an object and return a borrowed map view.
pub fn value_as_object<'a>(
    value: &'a Value,
    error_message: &str,
) -> Result<&'a Map<String, Value>> {
    match value.as_object() {
        Some(object) => Ok(object),
        None => Err(message(error_message)),
    }
}

/// Read one nested object field if present.
pub fn object_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Option<&'a Map<String, Value>> {
    object.get(key).and_then(Value::as_object)
}

/// Read a non-empty string field or fall back to the provided default.
pub fn string_field(object: &Map<String, Value>, key: &str, default: &str) -> String {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .to_string()
}

#[cfg(test)]
#[path = "rust_tests.rs"]
mod common_rust_tests;
