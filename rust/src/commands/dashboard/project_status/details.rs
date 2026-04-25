use serde_json::Value;

use crate::project_status_model::StatusRecordCount;

pub(super) fn summary_number(document: &Value, key: &str) -> usize {
    document
        .get("summary")
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

pub(super) fn top_level_array<'a>(document: &'a Value, key: &str) -> Option<&'a [Value]> {
    document
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
}

pub(super) fn push_warning(
    warnings: &mut Vec<StatusRecordCount>,
    signal_keys: &mut Vec<String>,
    kind: &str,
    count: usize,
    source: &str,
) {
    if count > 0 {
        warnings.push(StatusRecordCount::new(kind, count, source));
        signal_keys.push(source.to_string());
    }
}

pub(super) fn count_import_ready_dashboard_dependencies(items: Option<&[Value]>) -> usize {
    items
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    item.get("file")
                        .and_then(Value::as_str)
                        .map(|value| !value.trim().is_empty())
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn count_import_ready_dashboard_dependency_details(items: Option<&[Value]>) -> usize {
    items
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    item.get("file")
                        .and_then(Value::as_str)
                        .map(|value| !value.trim().is_empty())
                        .unwrap_or(false)
                        && item
                            .get("panelIds")
                            .and_then(Value::as_array)
                            .map(|value| !value.is_empty())
                            .unwrap_or(false)
                        && item
                            .get("queryFields")
                            .and_then(Value::as_array)
                            .map(|value| !value.is_empty())
                            .unwrap_or(false)
                        && item
                            .get("datasources")
                            .and_then(Value::as_array)
                            .map(|value| !value.is_empty())
                            .unwrap_or(false)
                        && item
                            .get("datasourceFamilies")
                            .and_then(Value::as_array)
                            .map(|value| !value.is_empty())
                            .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn push_detail_signal_keys(
    signal_keys: &mut Vec<String>,
    document_key: &str,
    items: Option<&[Value]>,
    fields: &[&str],
) {
    if let Some(items) = items {
        if !items.is_empty() {
            push_signal_key(signal_keys, document_key);
            for (index, item) in items.iter().enumerate() {
                let source = detail_source_key(document_key, item, index, fields);
                push_signal_key(signal_keys, &source);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_detail_findings_with_count<F>(
    findings: &mut Vec<StatusRecordCount>,
    signal_keys: &mut Vec<String>,
    kind: &str,
    document_key: &str,
    items: Option<&[Value]>,
    fields: &[&str],
    fallback_count: usize,
    fallback_source: &str,
    mut count_for_item: F,
) where
    F: FnMut(&Value) -> usize,
{
    if let Some(items) = items {
        if !items.is_empty() {
            push_signal_key(signal_keys, document_key);
            let mut used_detail_rows = false;
            for (index, item) in items.iter().enumerate() {
                let count = count_for_item(item);
                if count == 0 {
                    continue;
                }
                let source = detail_source_key(document_key, item, index, fields);
                findings.push(StatusRecordCount::new(kind, count, &source));
                push_signal_key(signal_keys, &source);
                used_detail_rows = true;
            }
            if used_detail_rows {
                return;
            }
        }
    }

    if fallback_count > 0 {
        findings.push(StatusRecordCount::new(
            kind,
            fallback_count,
            fallback_source,
        ));
        push_signal_key(signal_keys, fallback_source);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_detail_findings(
    findings: &mut Vec<StatusRecordCount>,
    signal_keys: &mut Vec<String>,
    kind: &str,
    document_key: &str,
    items: Option<&[Value]>,
    fields: &[&str],
    fallback_count: usize,
    fallback_source: &str,
) {
    if let Some(items) = items {
        if !items.is_empty() {
            push_signal_key(signal_keys, document_key);
            for (index, item) in items.iter().enumerate() {
                let source = detail_source_key(document_key, item, index, fields);
                findings.push(StatusRecordCount::new(kind, 1, &source));
                push_signal_key(signal_keys, &source);
            }
            return;
        }
    }

    if fallback_count > 0 {
        findings.push(StatusRecordCount::new(
            kind,
            fallback_count,
            fallback_source,
        ));
        push_signal_key(signal_keys, fallback_source);
    }
}

fn push_signal_key(signal_keys: &mut Vec<String>, source: &str) {
    if !signal_keys.iter().any(|item| item == source) {
        signal_keys.push(source.to_string());
    }
}

fn detail_source_key(document_key: &str, item: &Value, index: usize, fields: &[&str]) -> String {
    for field in fields {
        if let Some(value) = item.get(field).and_then(Value::as_str) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return format!("{document_key}.{trimmed}");
            }
        }
    }
    format!("{document_key}[{index}]")
}
