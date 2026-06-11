#![cfg_attr(not(feature = "tui"), allow(dead_code))]

use serde_json::{Map, Value};

use crate::datasource_provider::{collect_provider_references, iter_provider_names};
use crate::datasource_secret::{collect_secret_placeholders, iter_secret_placeholder_names};
use crate::review_diff::is_safe_review_changed_field;

pub(crate) fn review_lines_from_datasource_details(details: &Map<String, Value>) -> Vec<String> {
    secret_review_lines(details)
}

fn secret_review_lines(details: &Map<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(placeholders) = details
        .get("secureJsonDataPlaceholders")
        .and_then(Value::as_object)
    {
        lines.extend(placeholder_review_lines(placeholders));
    }
    if let Some(secure_json_fields) = details.get("secureJsonFields").and_then(Value::as_object) {
        lines.extend(live_secure_json_fields_review_lines(secure_json_fields));
    }
    if let Some(secure_json_data) = details.get("secureJsonData").and_then(Value::as_object) {
        lines.extend(resolved_secure_json_data_review_lines(secure_json_data));
    }
    if let Some(providers) = details
        .get("secureJsonDataProviders")
        .and_then(Value::as_object)
    {
        lines.extend(provider_reference_review_lines(providers));
    }
    if details
        .get("readOnly")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        lines.extend(read_only_review_lines());
    }
    lines.extend(local_review_evidence_lines(details));
    lines
}

fn placeholder_review_lines(placeholders: &Map<String, Value>) -> Vec<String> {
    match collect_secret_placeholders(Some(placeholders)) {
        Ok(placeholders) if placeholders.is_empty() => Vec::new(),
        Ok(placeholders) => {
            let field_names = placeholders
                .iter()
                .map(|placeholder| placeholder.field_name.clone())
                .collect::<Vec<_>>();
            let placeholder_names = iter_secret_placeholder_names(&placeholders)
                .map(str::to_string)
                .collect::<Vec<_>>();
            vec![
                format!(
                    "Secret placeholders: available ({} field(s): {})",
                    field_names.len(),
                    field_names.join(", ")
                ),
                format!("Secret placeholder names: {}", placeholder_names.join(", ")),
                "Secret blocker status: blocked until --secret-values resolves placeholders"
                    .to_string(),
                "Secret review required: true (placeholder-backed secureJsonData)".to_string(),
            ]
        }
        Err(error) => vec![
            format!(
                "Secret placeholders: invalid placeholder contract ({} field(s): {})",
                placeholders.len(),
                sorted_object_keys(placeholders).join(", ")
            ),
            format!("Secret blocker status: blocked by placeholder contract error: {error}"),
            "Secret review required: true (placeholder contract error)".to_string(),
        ],
    }
}

fn live_secure_json_fields_review_lines(secure_json_fields: &Map<String, Value>) -> Vec<String> {
    let field_names = secure_json_fields
        .iter()
        .filter(|&(_, value)| value.as_bool().unwrap_or(false))
        .map(|(field_name, _)| field_name.to_string())
        .collect::<Vec<_>>();
    if field_names.is_empty() {
        return Vec::new();
    }
    vec![
        format!(
            "Secret placeholders: unavailable from live secureJsonFields ({} field(s): {})",
            field_names.len(),
            field_names.join(", ")
        ),
        "Secret blocker status: review-required; resolved values are hidden by Grafana".to_string(),
        "Secret review required: true (secure fields present)".to_string(),
    ]
}

fn resolved_secure_json_data_review_lines(secure_json_data: &Map<String, Value>) -> Vec<String> {
    let field_names = sorted_object_keys(secure_json_data);
    if field_names.is_empty() {
        return Vec::new();
    }
    vec![
        format!(
            "Secret material: hidden ({} secureJsonData field(s): {})",
            field_names.len(),
            field_names.join(", ")
        ),
        "Secret blocker status: review-required; resolved credential values are never displayed"
            .to_string(),
        "Secret review required: true (resolved secureJsonData hidden)".to_string(),
    ]
}

fn provider_reference_review_lines(providers: &Map<String, Value>) -> Vec<String> {
    match collect_provider_references(Some(providers)) {
        Ok(references) if references.is_empty() => Vec::new(),
        Ok(references) => {
            let field_names = references
                .iter()
                .map(|reference| reference.field_name.clone())
                .collect::<Vec<_>>();
            let provider_names = iter_provider_names(&references)
                .map(str::to_string)
                .collect::<Vec<_>>();
            vec![
                format!(
                    "Secret providers: external ({} field(s): {})",
                    field_names.len(),
                    field_names.join(", ")
                ),
                format!("Secret provider names: {}", provider_names.join(", ")),
                "Secret blocker status: review-required; provider resolution is external"
                    .to_string(),
                "Secret review required: true (provider-backed secureJsonData)".to_string(),
            ]
        }
        Err(error) => vec![
            format!(
                "Secret providers: invalid provider contract ({} field(s): {})",
                providers.len(),
                sorted_object_keys(providers).join(", ")
            ),
            format!("Secret blocker status: blocked by provider contract error: {error}"),
            "Secret review required: true (provider contract error)".to_string(),
        ],
    }
}

fn read_only_review_lines() -> Vec<String> {
    vec![
        "Datasource blocker status: blocked; read-only datasource requires external update"
            .to_string(),
        "Datasource review required: true (read-only datasource)".to_string(),
    ]
}

fn local_review_evidence_lines(details: &Map<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();
    let action = text_field(details, &["action"]);
    let status = text_field(details, &["status"]);
    if let Some(action) = action {
        if let Some(status) = status {
            lines.push(format!("Review action: {action} (status={status})"));
        } else {
            lines.push(format!("Review action: {action}"));
        }
    } else if let Some(status) = status {
        lines.push(format!("Review status: {status}"));
    }
    if let Some(blocked_reason) = text_field(details, &["blockedReason", "blocked_reason"]) {
        lines.push(format!(
            "Review blocker status: blocked by {blocked_reason}"
        ));
    }
    if let Some(match_basis) = text_field(details, &["matchBasis", "match_basis"]) {
        lines.push(format!("Review match: {match_basis}"));
    }
    if let Some(destination) = text_field(details, &["destination"]) {
        lines.push(format!("Review destination: {destination}"));
    }
    if let Some(file) = text_field(details, &["file", "sourceFile", "source_file"]) {
        lines.push(format!("Review source: {file}"));
    }
    if let Some(target_uid) = text_field(details, &["targetUid", "target_uid"]) {
        lines.push(format!("Review target UID: {target_uid}"));
    }
    if let Some(target_version) = i64_field(details, &["targetVersion", "target_version"]) {
        lines.push(format!("Review target version: {target_version}"));
    }
    if let Some(target_read_only) = bool_field(details, &["targetReadOnly", "target_read_only"]) {
        lines.push(format!("Review target: read-only={target_read_only}"));
    }
    if let Some(changed_fields) = text_list_field(details, &["changedFields", "changed_fields"]) {
        let changed_fields = changed_fields
            .into_iter()
            .filter(|field| is_safe_changed_field(field))
            .collect::<Vec<_>>();
        if !changed_fields.is_empty() {
            lines.push(format!(
                "Review changed fields: {}",
                changed_fields.join(", ")
            ));
        }
    }
    if bool_field(details, &["reviewRequired", "review_required"]) == Some(true) {
        lines.push("Review required: true".to_string());
    }
    if bool_field(details, &["requiresSecretValues", "requires_secret_values"]) == Some(true) {
        lines.push("Review requires secret values: true".to_string());
    }
    if let Some(review_hints) = text_list_field(details, &["reviewHints", "review_hints"]) {
        lines.push(format!("Review hints: {}", review_hints.join(", ")));
    }
    lines
}

fn text_field(details: &Map<String, Value>, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        details
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn bool_field(details: &Map<String, Value>, names: &[&str]) -> Option<bool> {
    names
        .iter()
        .find_map(|name| details.get(*name).and_then(Value::as_bool))
}

fn i64_field(details: &Map<String, Value>, names: &[&str]) -> Option<i64> {
    names
        .iter()
        .find_map(|name| details.get(*name).and_then(Value::as_i64))
}

fn text_list_field(details: &Map<String, Value>, names: &[&str]) -> Option<Vec<String>> {
    let mut values = names.iter().find_map(|name| {
        let value = details.get(*name)?;
        if let Some(items) = value.as_array() {
            let items = items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>();
            return (!items.is_empty()).then_some(items);
        }
        value
            .as_str()
            .map(|text| {
                text.split(',')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .filter(|items| !items.is_empty())
    })?;
    values.sort();
    Some(values)
}

pub(crate) fn is_safe_changed_field(field: &str) -> bool {
    is_safe_review_changed_field(field)
}

fn sorted_object_keys(object: &Map<String, Value>) -> Vec<String> {
    let mut keys = object.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}
