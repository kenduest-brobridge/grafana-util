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
    ownership: DashboardTargetOwnership,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DashboardTargetOwnership {
    ApiManaged,
    FileProvisioned,
    GitSyncManaged,
    ManagedUnknown,
}

impl DashboardTargetOwnership {
    fn label(self) -> &'static str {
        match self {
            DashboardTargetOwnership::ApiManaged => "api-managed",
            DashboardTargetOwnership::FileProvisioned => "file-provisioned",
            DashboardTargetOwnership::GitSyncManaged => "git-sync-managed",
            DashboardTargetOwnership::ManagedUnknown => "managed-unknown",
        }
    }

    fn blocks_direct_write(self) -> bool {
        matches!(
            self,
            DashboardTargetOwnership::FileProvisioned | DashboardTargetOwnership::GitSyncManaged
        )
    }

    fn warns_direct_write(self) -> bool {
        matches!(self, DashboardTargetOwnership::ManagedUnknown)
    }
}

pub(crate) fn dashboard_target_review_status_label(review: &DashboardTargetReview) -> &'static str {
    review.kind_label()
}

pub(crate) fn dashboard_target_review_ownership_label(
    review: &DashboardTargetReview,
) -> &'static str {
    review.ownership.label()
}

pub(crate) fn dashboard_target_review_is_warning(review: &DashboardTargetReview) -> bool {
    dashboard_target_review_warns_direct_write(review)
}

pub(crate) fn dashboard_target_review_is_blocked(review: &DashboardTargetReview) -> bool {
    dashboard_target_review_blocks_direct_write(review)
}

pub(crate) fn dashboard_target_review_blocks_direct_write(review: &DashboardTargetReview) -> bool {
    review.ownership.blocks_direct_write()
}

pub(crate) fn dashboard_target_review_warns_direct_write(review: &DashboardTargetReview) -> bool {
    review.ownership.warns_direct_write()
}

pub(crate) fn dashboard_target_evidence_blocks_direct_write(evidence: &[String]) -> bool {
    evidence_has_ownership(evidence, DashboardTargetOwnership::FileProvisioned)
        || evidence_has_ownership(evidence, DashboardTargetOwnership::GitSyncManaged)
}

pub(crate) fn dashboard_target_evidence_warns_direct_write(evidence: &[String]) -> bool {
    evidence_has_ownership(evidence, DashboardTargetOwnership::ManagedUnknown)
}

fn evidence_has_ownership(evidence: &[String], ownership: DashboardTargetOwnership) -> bool {
    let expected = format!("ownership={}", ownership.label());
    evidence.iter().any(|value| value == &expected)
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

fn is_non_empty_managed_value(value: Option<&Value>) -> bool {
    value
        .and_then(|value| summarize_scalar_value("managed", value))
        .is_some()
}

fn value_has_git_sync_signal(value: &Value) -> bool {
    match value {
        Value::String(text) => text_contains_git_sync_signal(text),
        Value::Array(items) => items.iter().any(value_has_git_sync_signal),
        Value::Object(object) => object.iter().any(|(key, value)| {
            text_contains_git_sync_signal(key) || value_has_git_sync_signal(value)
        }),
        _ => false,
    }
}

fn text_contains_git_sync_signal(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    !normalized.is_empty()
        && (normalized.contains("git")
            || normalized.contains("repo")
            || normalized.contains("repository"))
}

fn classify_dashboard_target_ownership(meta: &Map<String, Value>) -> DashboardTargetOwnership {
    let provisioned = meta
        .get("provisioned")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let provisioned_external_id = meta
        .get("provisionedExternalId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some();
    let has_managed_repository = is_non_empty_managed_value(meta.get("managedRepository"));
    let has_repository = is_non_empty_managed_value(meta.get("repository"));
    let managed_by = meta.get("managedBy");
    let managed_by_is_git_sync = managed_by.map(value_has_git_sync_signal).unwrap_or(false);
    let has_managed_by = is_non_empty_managed_value(managed_by);

    if has_managed_repository || has_repository || managed_by_is_git_sync {
        DashboardTargetOwnership::GitSyncManaged
    } else if provisioned || provisioned_external_id {
        DashboardTargetOwnership::FileProvisioned
    } else if has_managed_by {
        DashboardTargetOwnership::ManagedUnknown
    } else {
        DashboardTargetOwnership::ApiManaged
    }
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
            ownership: DashboardTargetOwnership::ApiManaged,
            evidence: Vec::new(),
        });
    };

    let mut evidence = Vec::new();
    let ownership = classify_dashboard_target_ownership(meta);
    if ownership != DashboardTargetOwnership::ApiManaged {
        evidence.push(format!("ownership={}", ownership.label()));
    }
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

    let kind = if ownership.blocks_direct_write() {
        DashboardTargetReviewKind::Blocked
    } else if evidence.is_empty() {
        DashboardTargetReviewKind::Ready
    } else {
        DashboardTargetReviewKind::Warning
    };
    Ok(DashboardTargetReview {
        kind,
        ownership,
        evidence,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn target_review_classifies_api_managed_as_ready() {
        let review = build_dashboard_target_review(&json!({
            "dashboard": {"uid": "api"},
            "meta": {"folderUid": "general"}
        }))
        .unwrap();

        assert_eq!(review.kind, DashboardTargetReviewKind::Ready);
        assert_eq!(review.ownership, DashboardTargetOwnership::ApiManaged);
        assert!(review.evidence.is_empty());
    }

    #[test]
    fn target_review_classifies_file_provisioned_as_blocked() {
        let review = build_dashboard_target_review(&json!({
            "dashboard": {"uid": "file"},
            "meta": {
                "provisioned": true,
                "provisionedExternalId": "dashboards/file.json"
            }
        }))
        .unwrap();

        assert_eq!(review.kind, DashboardTargetReviewKind::Blocked);
        assert_eq!(review.ownership, DashboardTargetOwnership::FileProvisioned);
        assert!(dashboard_target_review_blocks_direct_write(&review));
        assert!(review
            .evidence
            .contains(&"ownership=file-provisioned".to_string()));
    }

    #[test]
    fn target_review_classifies_git_sync_repository_as_blocked() {
        let review = build_dashboard_target_review(&json!({
            "dashboard": {"uid": "git"},
            "meta": {
                "managedBy": {"kind": "repo", "id": "grafana-dashboard-repo"},
                "managedRepository": {"name": "platform-dashboards"},
                "repository": "platform-dashboards"
            }
        }))
        .unwrap();

        assert_eq!(review.kind, DashboardTargetReviewKind::Blocked);
        assert_eq!(review.ownership, DashboardTargetOwnership::GitSyncManaged);
        assert!(dashboard_target_review_blocks_direct_write(&review));
        assert!(review
            .evidence
            .contains(&"ownership=git-sync-managed".to_string()));
    }

    #[test]
    fn target_review_classifies_unknown_managed_metadata_as_warning() {
        let review = build_dashboard_target_review(&json!({
            "dashboard": {"uid": "managed"},
            "meta": {"managedBy": {"kind": "plugin", "id": "unknown"}}
        }))
        .unwrap();

        assert_eq!(review.kind, DashboardTargetReviewKind::Warning);
        assert_eq!(review.ownership, DashboardTargetOwnership::ManagedUnknown);
        assert!(dashboard_target_review_warns_direct_write(&review));
        assert!(!dashboard_target_review_blocks_direct_write(&review));
        assert!(review
            .evidence
            .contains(&"ownership=managed-unknown".to_string()));
    }
}
