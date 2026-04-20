//! Small JSON accessors shared by sync-owned project-status producers.

use serde_json::Value;

pub(super) fn summary_number(document: &Value, key: &str) -> usize {
    document
        .get("summary")
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

pub(super) fn section_object<'a>(document: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    document
        .and_then(Value::as_object)
        .and_then(|object| object.get(key))
}

pub(super) fn section_bool(document: Option<&Value>, section: &str, key: &str) -> bool {
    section_object(document, section)
        .and_then(|value| value.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(super) fn section_text(document: Option<&Value>, section: &str, key: &str) -> Option<String> {
    section_object(document, section)
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn section_number(document: Option<&Value>, section: &str, key: &str) -> usize {
    section_object(document, section)
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

pub(super) fn section_summary_number(document: &Value, section: &str, key: &str) -> usize {
    section_object(Some(document), section)
        .and_then(Value::as_object)
        .and_then(|object| object.get("summary"))
        .and_then(Value::as_object)
        .and_then(|object| object.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

pub(super) fn section_array_count(document: &Value, section: &str, key: &str) -> usize {
    section_object(Some(document), section)
        .and_then(Value::as_object)
        .and_then(|object| object.get(key))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

pub(super) fn value_array_count(document: Option<&Value>) -> usize {
    document
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

pub(super) fn push_unique(next_actions: &mut Vec<String>, action: &str) {
    if !next_actions.iter().any(|item| item == action) {
        next_actions.push(action.to_string());
    }
}
