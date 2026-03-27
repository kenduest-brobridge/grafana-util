//! Sync apply-intent parsing helpers.
//!
//! This module owns the shape of a staged apply intent document so request
//! execution can stay focused on Grafana transport wiring.

use crate::common::Result;
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyncApplyOperation {
    pub kind: String,
    pub identity: String,
    pub action: String,
    pub desired: Map<String, Value>,
}

impl SyncApplyOperation {
    pub(crate) fn from_value(operation: &Value) -> Result<Self> {
        let object = crate::sync::require_json_object(operation, "Sync apply operation")?;
        Ok(Self {
            kind: object
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string(),
            identity: object
                .get("identity")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string(),
            action: object
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string(),
            desired: object
                .get("desired")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default(),
        })
    }
}

pub(crate) fn load_apply_intent_operations(document: &Value) -> Result<Vec<SyncApplyOperation>> {
    let object = crate::sync::require_json_object(document, "Sync apply intent document")?;
    let operations = super::super::json::require_json_array_field(
        object,
        "operations",
        "Sync apply intent document",
    )?;
    operations
        .iter()
        .map(SyncApplyOperation::from_value)
        .collect::<Result<Vec<_>>>()
}
