use clap::ValueEnum;
use serde::Serialize;
use serde_json::{json, Value};

use super::TOOL_VERSION;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum DiffOutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SharedDiffSummary {
    pub checked: usize,
    pub same: usize,
    pub different: usize,
    pub missing_remote: usize,
    pub extra_remote: usize,
    pub ambiguous: usize,
}

pub fn build_shared_diff_document(
    kind: &str,
    schema_version: i64,
    summary: SharedDiffSummary,
    rows: &[Value],
) -> Value {
    json!({
        "kind": kind,
        "schemaVersion": schema_version,
        "toolVersion": TOOL_VERSION,
        "summary": summary,
        "rows": rows,
    })
}
