//! Dashboard import target review helpers.

use serde_json::{Map, Value};

use crate::common::{object_field, string_field, value_as_object, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DashboardTargetReviewKind {
    Ready,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardTargetReview {
    pub(crate) kind: DashboardTargetReviewKind,
    pub(crate) evidence: Vec<String>,
}

impl DashboardTargetReview {
    fn kind_label(&self) -> &'static str {
        match self.kind {
            DashboardTargetReviewKind::Ready => "ready",
            DashboardTargetReviewKind::Warning => "warn",
            DashboardTargetReviewKind::Blocked => "blocked",
        }
    }
}

pub(crate) fn dashboard_target_review_status_label(review: &DashboardTargetReview) -> &'static str {
    review.kind_label()
}

pub(crate) fn dashboard_target_review_is_warning(review: &DashboardTargetReview) -> bool {
    matches!(review.kind, DashboardTargetReviewKind::Warning)
}

pub(crate) fn dashboard_target_review_is_blocked(review: &DashboardTargetReview) -> bool {
    matches!(review.kind, DashboardTargetReviewKind::Blocked)
}

fn summarize_scalar_value(key: &str, value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Bool(flag) => Some(format!("{key}={flag}")),
        Value::Number(number) => Some(format!("{key}={number}")),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(format!("{key}={trimmed}"))
            }
        }
        Value::Array(items) => {
            let values = items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .collect::<Vec<&str>>();
            if values.is_empty() {
                None
            } else {
                Some(format!("{key}=[{}]", values.join(",")))
            }
        }
        Value::Object(object) => {
            let mut fields = Vec::new();
            for nested_key in ["kind", "id", "name", "uid", "title", "repository", "url"] {
                let nested_value = string_field(object, nested_key, "");
                if !nested_value.trim().is_empty() {
                    fields.push(format!("{nested_key}={}", nested_value.trim()));
                }
            }
            if fields.is_empty() {
                Some(format!("{key}={value}"))
            } else {
                Some(format!("{key}{{{}}}", fields.join(",")))
            }
        }
    }
}

fn summarize_meta_field(meta: &Map<String, Value>, key: &str) -> Option<String> {
    meta.get(key)
        .and_then(|value| summarize_scalar_value(key, value))
}

pub(crate) fn build_dashboard_target_review(
    existing_payload: &Value,
) -> Result<DashboardTargetReview> {
    let object = value_as_object(
        existing_payload,
        "Unexpected dashboard payload from Grafana.",
    )?;
    let Some(meta): Option<&Map<String, Value>> = object_field(object, "meta") else {
        return Ok(DashboardTargetReview {
            kind: DashboardTargetReviewKind::Ready,
            evidence: Vec::new(),
        });
    };

    let mut evidence = Vec::new();
    let provisioned = meta
        .get("provisioned")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if provisioned {
        evidence.push("provisioned=true".to_string());
    }
    if let Some(value) = meta.get("provisionedExternalId").and_then(Value::as_str) {
        let value = value.trim();
        if !value.is_empty() {
            evidence.push(format!("provisionedExternalId={value}"));
        }
    }
    for key in ["managedBy", "managedRepository", "repository"] {
        if let Some(summary) = summarize_meta_field(meta, key) {
            evidence.push(summary);
        }
    }

    let kind = if provisioned {
        DashboardTargetReviewKind::Blocked
    } else if evidence.is_empty() {
        DashboardTargetReviewKind::Ready
    } else {
        DashboardTargetReviewKind::Warning
    };
    Ok(DashboardTargetReview { kind, evidence })
}

pub(crate) fn build_dashboard_target_review_reason(review: &DashboardTargetReview) -> String {
    if review.evidence.is_empty() {
        return dashboard_target_review_status_label(review).to_string();
    }
    format!(
        "target={} {}",
        dashboard_target_review_status_label(review),
        review.evidence.join(" ")
    )
}
