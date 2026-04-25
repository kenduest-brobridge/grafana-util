//! Shared access import dry-run review document builder.

use serde_json::{Map, Value};
use std::path::Path;

use crate::common::tool_version;

const ACCESS_IMPORT_DRY_RUN_KIND: &str = "grafana-utils-access-import-dry-run";
const ACCESS_IMPORT_DRY_RUN_SCHEMA_VERSION: i64 = 1;

pub(crate) fn build_access_import_dry_run_document(
    resource_kind: &str,
    rows: &[Map<String, Value>],
    processed: usize,
    created: usize,
    updated: usize,
    skipped: usize,
    source: &Path,
) -> Value {
    let blocked = rows
        .iter()
        .filter(|row| {
            matches!(row.get("blocked"), Some(Value::Bool(true)))
                || matches!(row.get("status"), Some(Value::String(status)) if status == "blocked")
        })
        .count();
    Value::Object(Map::from_iter(vec![
        (
            "kind".to_string(),
            Value::String(ACCESS_IMPORT_DRY_RUN_KIND.to_string()),
        ),
        (
            "schemaVersion".to_string(),
            Value::Number(ACCESS_IMPORT_DRY_RUN_SCHEMA_VERSION.into()),
        ),
        (
            "toolVersion".to_string(),
            Value::String(tool_version().to_string()),
        ),
        ("reviewRequired".to_string(), Value::Bool(true)),
        ("reviewed".to_string(), Value::Bool(false)),
        (
            "resourceKind".to_string(),
            Value::String(resource_kind.to_string()),
        ),
        (
            "rows".to_string(),
            Value::Array(rows.iter().cloned().map(Value::Object).collect()),
        ),
        (
            "summary".to_string(),
            Value::Object(Map::from_iter(vec![
                (
                    "processed".to_string(),
                    Value::Number((processed as i64).into()),
                ),
                (
                    "created".to_string(),
                    Value::Number((created as i64).into()),
                ),
                (
                    "updated".to_string(),
                    Value::Number((updated as i64).into()),
                ),
                (
                    "skipped".to_string(),
                    Value::Number((skipped as i64).into()),
                ),
                (
                    "blocked".to_string(),
                    Value::Number((blocked as i64).into()),
                ),
                (
                    "source".to_string(),
                    Value::String(source.to_string_lossy().to_string()),
                ),
            ])),
        ),
    ]))
}
