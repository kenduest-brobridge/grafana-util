//! Shared datasource staged domain-status producer.
//!
//! Maintainer note:
//! - This module derives one datasource-owned domain-status reading from the staged
//!   datasource export document.
//! - Keep it document-driven and conservative so the overview layer can reuse
//!   the same contract without duplicating staged-summary logic.

use serde_json::Value;

use crate::project_status::{
    ProjectDomainStatusReading, PROJECT_STATUS_BLOCKED, PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
};
use crate::project_status_model::{StatusReading, StatusRecordCount};

const DATASOURCE_DOMAIN_ID: &str = "datasource";
const DATASOURCE_SCOPE: &str = "staged";
const DATASOURCE_MODE: &str = "artifact-summary";
const DATASOURCE_REASON_READY: &str = PROJECT_STATUS_READY;
const DATASOURCE_REASON_PARTIAL_NO_DATA: &str = "partial-no-data";
const DATASOURCE_REASON_BLOCKED_BY_BLOCKERS: &str = "blocked-by-blockers";

const DATASOURCE_SOURCE_KINDS: &[&str] = &["datasource-export"];
const DATASOURCE_SIGNAL_KEYS: &[&str] = &[
    "summary.datasourceCount",
    "summary.orgCount",
    "summary.defaultCount",
    "summary.typeCount",
    "summary.wouldCreate",
    "summary.wouldUpdate",
    "summary.wouldSkip",
    "summary.wouldBlock",
    "summary.wouldCreateOrgCount",
];

const DATASOURCE_WARNING_MISSING_DEFAULT: &str = "missing-default";
const DATASOURCE_WARNING_MULTIPLE_DEFAULTS: &str = "multiple-defaults";
const DATASOURCE_WARNING_DIFF_DRIFT_CHANGED_FIELDS: &str = "diff-drift-changed-fields";
const DATASOURCE_WARNING_DIFF_DRIFT_MISSING_LIVE: &str = "diff-drift-missing-live";
const DATASOURCE_WARNING_DIFF_DRIFT_MISSING_EXPORT: &str = "diff-drift-missing-export";
const DATASOURCE_WARNING_DIFF_DRIFT_AMBIGUOUS: &str = "diff-drift-ambiguous";
const DATASOURCE_WARNING_DIFF_DRIFT_SUMMARY: &str = "diff-drift-summary";
const DATASOURCE_WARNING_SECRET_REFERENCE_READY: &str = "secret-reference-ready";
const DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_CREATE: &str = "import-preview-would-create";
const DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_UPDATE: &str = "import-preview-would-update";
const DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_SKIP: &str = "import-preview-would-skip";
const DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_CREATE_ORG: &str = "import-preview-would-create-org";
const DATASOURCE_WARNING_IMPORT_PREVIEW_ROUTED_SOURCE_ORGS: &str =
    "import-preview-routed-source-orgs";
const DATASOURCE_BLOCKER_IMPORT_PREVIEW_WOULD_BLOCK: &str = "import-preview-would-block";

const DATASOURCE_EXPORT_AT_LEAST_ONE_ACTIONS: &[&str] = &["export at least one datasource"];
const DATASOURCE_MARK_DEFAULT_ACTIONS: &[&str] = &["mark a default datasource if none is set"];
const DATASOURCE_KEEP_SINGLE_DEFAULT_ACTIONS: &[&str] =
    &["keep exactly one datasource marked as the default"];
const DATASOURCE_REVIEW_DIFF_DRIFT_ACTIONS: &[&str] =
    &["review datasource diff drift before import or sync"];
const DATASOURCE_REVIEW_SECRET_REFERENCE_ACTIONS: &[&str] =
    &["review datasource secret references before import or sync"];
const DATASOURCE_REVIEW_IMPORT_PREVIEW_ACTIONS: &[&str] =
    &["review datasource import preview before import or sync"];
const DATASOURCE_REVIEW_IMPORT_ORG_CREATION_ACTIONS: &[&str] =
    &["review datasource org creation before import or sync"];
const DATASOURCE_REVIEW_IMPORT_ROUTED_SOURCE_ORGS_ACTIONS: &[&str] =
    &["review datasource org routing before import or sync"];
const DATASOURCE_REVIEW_IMPORT_ROUTING_AND_ORG_CREATION_ACTIONS: &[&str] =
    &["review datasource org routing and org creation before import or sync"];
const DATASOURCE_RESOLVE_IMPORT_BLOCKERS_ACTIONS: &[&str] =
    &["resolve datasource import preview blockers before import or sync"];

fn summary_number(document: &Value, key: &str) -> usize {
    document
        .get("summary")
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

fn summary_number_first(document: &Value, keys: &[&'static str]) -> (usize, Option<&'static str>) {
    for key in keys {
        let count = summary_number(document, key);
        if count > 0 {
            return (count, Some(*key));
        }
    }
    (0, None)
}

fn summary_string_list_count(document: &Value, key: &str) -> usize {
    document
        .get("summary")
        .and_then(|value| value.get(key))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn append_signal_key(signal_keys: &mut Vec<String>, source: &str) {
    if !signal_keys.iter().any(|item| item == source) {
        signal_keys.push(source.to_string());
    }
}

fn summary_source_label(source: Option<&str>, fallback: &str) -> String {
    format!("summary.{}", source.unwrap_or(fallback))
}

fn push_warning(
    warnings: &mut Vec<StatusRecordCount>,
    signal_keys: &mut Vec<String>,
    kind: &str,
    count: usize,
    source: &str,
) {
    if count == 0 {
        return;
    }
    warnings.push(StatusRecordCount::new(kind, count, source));
    append_signal_key(signal_keys, source);
}

pub(crate) fn build_datasource_domain_status(
    summary_document: Option<&Value>,
) -> Option<ProjectDomainStatusReading> {
    let document = summary_document?;
    let datasources = summary_number(document, "datasourceCount");
    let _orgs = summary_number(document, "orgCount");
    let defaults = summary_number(document, "defaultCount");
    let _types = summary_number(document, "typeCount");

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut signal_keys = DATASOURCE_SIGNAL_KEYS
        .iter()
        .map(|item| (*item).to_string())
        .collect::<Vec<String>>();

    if datasources == 0 {
        return Some(
            StatusReading {
                id: DATASOURCE_DOMAIN_ID.to_string(),
                scope: DATASOURCE_SCOPE.to_string(),
                mode: DATASOURCE_MODE.to_string(),
                status: PROJECT_STATUS_PARTIAL.to_string(),
                reason_code: DATASOURCE_REASON_PARTIAL_NO_DATA.to_string(),
                primary_count: datasources,
                source_kinds: DATASOURCE_SOURCE_KINDS
                    .iter()
                    .map(|item| (*item).to_string())
                    .collect(),
                signal_keys,
                blockers: Vec::new(),
                warnings,
                next_actions: DATASOURCE_EXPORT_AT_LEAST_ONE_ACTIONS
                    .iter()
                    .map(|item| (*item).to_string())
                    .collect(),
                freshness: Default::default(),
            }
            .into_project_domain_status_reading(),
        );
    }

    let mut next_actions = if defaults == 0 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_MISSING_DEFAULT,
            1,
            "summary.defaultCount",
        );
        DATASOURCE_MARK_DEFAULT_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect()
    } else if defaults > 1 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_MULTIPLE_DEFAULTS,
            defaults - 1,
            "summary.defaultCount",
        );
        DATASOURCE_KEEP_SINGLE_DEFAULT_ACTIONS
            .iter()
            .map(|item| (*item).to_string())
            .collect()
    } else {
        Vec::new()
    };

    let (changed_fields, changed_fields_source) =
        summary_number_first(document, &["differentCount", "different_count"]);
    let (missing_live, missing_live_source) =
        summary_number_first(document, &["missingLiveCount", "missing_in_live_count"]);
    let (missing_export, missing_export_source) = summary_number_first(
        document,
        &[
            "extraLiveCount",
            "missingInExportCount",
            "missing_in_export_count",
        ],
    );
    let (ambiguous, ambiguous_source) = summary_number_first(
        document,
        &[
            "ambiguousCount",
            "ambiguousLiveMatchCount",
            "ambiguous_live_match_count",
        ],
    );
    let (summary_diff_drift, summary_diff_drift_source) =
        summary_number_first(document, &["diffCount", "diff_count"]);
    let (secret_reference_ready, secret_reference_source) =
        summary_number_first(document, &["secretVisibilityCount", "secretReferenceCount"]);
    let (would_create, would_create_source) =
        summary_number_first(document, &["wouldCreate", "would_create"]);
    let (would_update, would_update_source) =
        summary_number_first(document, &["wouldUpdate", "would_update"]);
    let (would_block, would_block_source) =
        summary_number_first(document, &["wouldBlock", "would_block"]);
    let (would_skip, would_skip_source) =
        summary_number_first(document, &["wouldSkip", "would_skip"]);
    let (would_create_org, would_create_org_source) =
        summary_number_first(document, &["wouldCreateOrgCount", "would_create_org_count"]);
    let routed_source_orgs = summary_string_list_count(document, "sourceOrgLabels");

    let mut diff_drift_review_required = false;
    let mut granular_diff_drift_found = false;
    if changed_fields > 0 {
        diff_drift_review_required = true;
        granular_diff_drift_found = true;
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_DIFF_DRIFT_CHANGED_FIELDS,
            changed_fields,
            &summary_source_label(changed_fields_source, "differentCount"),
        );
    }
    if missing_live > 0 {
        diff_drift_review_required = true;
        granular_diff_drift_found = true;
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_DIFF_DRIFT_MISSING_LIVE,
            missing_live,
            &summary_source_label(missing_live_source, "missingLiveCount"),
        );
    }
    if missing_export > 0 {
        diff_drift_review_required = true;
        granular_diff_drift_found = true;
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_DIFF_DRIFT_MISSING_EXPORT,
            missing_export,
            &summary_source_label(missing_export_source, "missingInExportCount"),
        );
    }
    if ambiguous > 0 {
        diff_drift_review_required = true;
        granular_diff_drift_found = true;
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_DIFF_DRIFT_AMBIGUOUS,
            ambiguous,
            &summary_source_label(ambiguous_source, "ambiguousCount"),
        );
    }
    if !granular_diff_drift_found && summary_diff_drift > 0 {
        diff_drift_review_required = true;
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_DIFF_DRIFT_SUMMARY,
            summary_diff_drift,
            &summary_source_label(summary_diff_drift_source, "diffCount"),
        );
    }
    if diff_drift_review_required {
        next_actions.extend(
            DATASOURCE_REVIEW_DIFF_DRIFT_ACTIONS
                .iter()
                .map(|item| (*item).to_string()),
        );
    }

    if secret_reference_ready > 0 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_SECRET_REFERENCE_READY,
            secret_reference_ready,
            &summary_source_label(secret_reference_source, "secretVisibilityCount"),
        );
        next_actions.extend(
            DATASOURCE_REVIEW_SECRET_REFERENCE_ACTIONS
                .iter()
                .map(|item| (*item).to_string()),
        );
    }
    if would_create > 0 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_CREATE,
            would_create,
            &summary_source_label(would_create_source, "wouldCreate"),
        );
    }
    if would_update > 0 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_UPDATE,
            would_update,
            &summary_source_label(would_update_source, "wouldUpdate"),
        );
    }
    if would_block > 0 {
        let source = summary_source_label(would_block_source, "wouldBlock");
        blockers.push(StatusRecordCount::new(
            DATASOURCE_BLOCKER_IMPORT_PREVIEW_WOULD_BLOCK,
            would_block,
            &source,
        ));
        append_signal_key(&mut signal_keys, &source);
    }
    if would_skip > 0 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_SKIP,
            would_skip,
            &summary_source_label(would_skip_source, "wouldSkip"),
        );
    }
    if would_create_org > 0 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_IMPORT_PREVIEW_WOULD_CREATE_ORG,
            would_create_org,
            &summary_source_label(would_create_org_source, "wouldCreateOrgCount"),
        );
    }
    if routed_source_orgs > 0 {
        push_warning(
            &mut warnings,
            &mut signal_keys,
            DATASOURCE_WARNING_IMPORT_PREVIEW_ROUTED_SOURCE_ORGS,
            routed_source_orgs,
            "summary.sourceOrgLabels",
        );
    }
    if would_create_org > 0 && routed_source_orgs > 0 {
        next_actions.extend(
            DATASOURCE_REVIEW_IMPORT_ROUTING_AND_ORG_CREATION_ACTIONS
                .iter()
                .map(|item| (*item).to_string()),
        );
    } else {
        if would_create_org > 0 {
            next_actions.extend(
                DATASOURCE_REVIEW_IMPORT_ORG_CREATION_ACTIONS
                    .iter()
                    .map(|item| (*item).to_string()),
            );
        }
        if routed_source_orgs > 0 {
            next_actions.extend(
                DATASOURCE_REVIEW_IMPORT_ROUTED_SOURCE_ORGS_ACTIONS
                    .iter()
                    .map(|item| (*item).to_string()),
            );
        }
    }
    if would_create > 0 || would_update > 0 || would_block > 0 || would_skip > 0 {
        next_actions.extend(
            DATASOURCE_REVIEW_IMPORT_PREVIEW_ACTIONS
                .iter()
                .map(|item| (*item).to_string()),
        );
    }

    if !blockers.is_empty() {
        next_actions.splice(
            0..0,
            DATASOURCE_RESOLVE_IMPORT_BLOCKERS_ACTIONS
                .iter()
                .map(|item| (*item).to_string()),
        );
    }

    let (status, reason_code): (&str, &str) = if !blockers.is_empty() {
        (
            PROJECT_STATUS_BLOCKED,
            DATASOURCE_REASON_BLOCKED_BY_BLOCKERS,
        )
    } else {
        (PROJECT_STATUS_READY, DATASOURCE_REASON_READY)
    };

    Some(
        StatusReading {
            id: DATASOURCE_DOMAIN_ID.to_string(),
            scope: DATASOURCE_SCOPE.to_string(),
            mode: DATASOURCE_MODE.to_string(),
            status: status.to_string(),
            reason_code: reason_code.to_string(),
            primary_count: datasources,
            source_kinds: DATASOURCE_SOURCE_KINDS
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            signal_keys,
            blockers,
            warnings,
            next_actions,
            freshness: Default::default(),
        }
        .into_project_domain_status_reading(),
    )
}

#[cfg(test)]
#[cfg(test)]
#[path = "staged_reading_rust_tests.rs"]
mod tests;
