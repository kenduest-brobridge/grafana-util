//! Dashboard domain orchestrator for the unified dashboard CLI.
//!
//! This file is the boundary between unified command parsing and the lower-level
//! dashboard modules that implement export/import/live/inspect/screenshot logic.
use crate::common::Result;
use crate::http::JsonHttpClient;
use serde::Serialize;
use serde_json::Value;

// Keep the dashboard surface area split by concern. This file should stay focused
// on re-exports, shared constants, and top-level command dispatch.
mod authoring;
mod browse;
mod browse_actions;
mod browse_edit_dialog;
mod browse_external_edit_dialog;
mod browse_history_dialog;
mod browse_input;
mod browse_live_detail;
mod browse_load;
mod browse_render;
mod browse_state;
mod browse_support;
mod browse_terminal;
mod browse_tui;
mod cli_defs;
mod command_artifacts;
mod command_runner;
mod delete;
mod delete_interactive;
mod delete_render;
mod delete_support;
mod edit;
mod edit_external;
mod edit_live;
mod export;
mod export_layout;
mod facade_exports;
mod facade_support;
mod files;
mod governance_gate;
mod governance_gate_rules;
mod governance_gate_tui;
mod governance_policy;
mod help;
mod history;
mod impact_tui;
mod import;
mod import_interactive;
mod import_interactive_context;
mod import_interactive_loader;
mod import_interactive_render;
mod import_interactive_review;
mod import_interactive_state;
mod inspect;
mod inspect_analyzer_flux;
mod inspect_analyzer_loki;
mod inspect_analyzer_prometheus;
mod inspect_analyzer_search;
mod inspect_analyzer_sql;
mod inspect_dependency_render;
mod inspect_family;
mod inspect_governance;
mod inspect_live;
mod inspect_live_tui;
mod inspect_query;
mod inspect_render;
mod inspect_report;
mod inspect_summary;
mod inspect_workbench;
mod inspect_workbench_render;
mod inspect_workbench_state;
mod inspect_workbench_support;
mod list;
mod live;
mod live_project_status;
mod models;
mod plan;
mod plan_types;
mod project_status;
mod prompt;
mod prompt_datasource_refs;
mod prompt_helpers;
mod prompt_inputs;
mod prompt_variables;
mod raw_to_prompt;
mod raw_to_prompt_datasource_resolution;
mod raw_to_prompt_output;
mod raw_to_prompt_plan;
mod raw_to_prompt_prompt_paths;
mod raw_to_prompt_resolution;
mod raw_to_prompt_types;
mod review_source;
mod run_inspect;
mod run_list;
mod screenshot;
mod serve;
mod source_loader;
mod topology;
mod topology_tui;
mod validate;
mod vars;

pub(crate) use facade_exports::crate_exports::*;
pub use facade_exports::pub_exports::*;
pub(crate) use import::{
    compare as import_compare, lookup as import_lookup, render as import_render,
    target as import_target, validation as import_validation,
};

#[cfg(not(feature = "tui"))]
pub(crate) fn tui_not_built<T>(action: &str) -> Result<T> {
    Err(crate::common::message(format!(
        "Dashboard {action} requires TUI support, but it was not built in."
    )))
}

// Shared dashboard defaults and export filenames used across export/import/live flows.
pub const DEFAULT_URL: &str = "http://localhost:3000";
pub const DEFAULT_TIMEOUT: u64 = 30;
pub const DEFAULT_PAGE_SIZE: usize = 500;
pub const DEFAULT_EXPORT_DIR: &str = "dashboards";
pub const RAW_EXPORT_SUBDIR: &str = "raw";
pub const PROMPT_EXPORT_SUBDIR: &str = "prompt";
pub const PROVISIONING_EXPORT_SUBDIR: &str = "provisioning";
pub const DEFAULT_IMPORT_MESSAGE: &str = "Imported by grafana-utils";
pub const DEFAULT_DASHBOARD_TITLE: &str = "dashboard";
pub const DEFAULT_FOLDER_TITLE: &str = "General";
pub const DEFAULT_FOLDER_UID: &str = "general";
pub const DEFAULT_ORG_ID: &str = "1";
pub const DEFAULT_ORG_NAME: &str = "Main Org.";
pub const DEFAULT_UNKNOWN_UID: &str = "unknown";
pub const EXPORT_METADATA_FILENAME: &str = "export-metadata.json";
pub const TOOL_SCHEMA_VERSION: i64 = 1;
pub const ROOT_INDEX_KIND: &str = "grafana-utils-dashboard-export-index";
pub const FOLDER_INVENTORY_FILENAME: &str = "folders.json";
pub const DATASOURCE_INVENTORY_FILENAME: &str = "datasources.json";
pub const DASHBOARD_PERMISSION_BUNDLE_FILENAME: &str = "permissions.json";
const BUILTIN_DATASOURCE_TYPES: &[&str] = &["__expr__", "grafana"];
const BUILTIN_DATASOURCE_NAMES: &[&str] = &[
    "-- Dashboard --",
    "-- Grafana --",
    "-- Mixed --",
    "grafana",
    "expr",
    "__expr__",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum FolderInventoryStatusKind {
    Missing,
    Matches,
    Mismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FolderInventoryStatus {
    pub uid: String,
    pub expected_title: String,
    pub expected_parent_uid: Option<String>,
    pub expected_path: String,
    pub actual_title: Option<String>,
    pub actual_parent_uid: Option<String>,
    pub actual_path: Option<String>,
    pub kind: FolderInventoryStatusKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DashboardWebRunOutput {
    pub document: Value,
    pub text_lines: Vec<String>,
}

/// Run the dashboard CLI with an already configured client.
/// Thin wrapper that keeps the public module surface stable while the runtime lives in `command_runner`.
pub fn run_dashboard_cli_with_client(
    client: &JsonHttpClient,
    args: DashboardCliArgs,
) -> Result<()> {
    command_runner::run_dashboard_cli_with_client(client, args)
}

/// Run the dashboard CLI after normalizing args and creating clients as needed.
/// Thin wrapper that exposes the dashboard runtime boundary from the module root.
pub fn run_dashboard_cli(args: DashboardCliArgs) -> Result<()> {
    command_runner::run_dashboard_cli(args)
}

#[cfg(test)]
#[path = "authoring/rust_tests.rs"]
mod authoring_rust_tests;
#[cfg(test)]
#[path = "dashboard_artifact_workflow_rust_tests.rs"]
mod dashboard_artifact_workflow_rust_tests;
#[cfg(test)]
#[path = "dashboard_cli_rust_tests.rs"]
mod dashboard_cli_rust_tests;
#[cfg(test)]
#[path = "rust_tests.rs"]
mod dashboard_rust_tests;
#[cfg(test)]
#[path = "history_cli_rust_tests.rs"]
mod history_cli_rust_tests;
#[cfg(test)]
#[path = "import_rust_tests.rs"]
mod import_rust_tests;
#[cfg(test)]
#[path = "inspect_export_rust_tests.rs"]
mod inspect_export_rust_tests;
#[cfg(test)]
#[path = "inspect_governance_document_rust_tests.rs"]
mod inspect_governance_document_rust_tests;
#[cfg(test)]
#[path = "inspect_governance_rust_tests.rs"]
mod inspect_governance_rust_tests;
#[cfg(test)]
#[path = "inspect_live_rust_tests.rs"]
mod inspect_live_rust_tests;
#[cfg(test)]
#[path = "inspect_vars_rust_tests.rs"]
mod inspect_vars_rust_tests;
#[cfg(test)]
#[path = "raw_to_prompt_rust_tests.rs"]
mod raw_to_prompt_rust_tests;
#[cfg(test)]
#[path = "screenshot_rust_tests.rs"]
mod screenshot_rust_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
#[path = "topology_impact_document_rust_tests.rs"]
mod topology_impact_document_rust_tests;
#[cfg(test)]
#[path = "topology_impact_rust_tests.rs"]
mod topology_impact_rust_tests;
