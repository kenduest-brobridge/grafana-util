//! Risk and audit row builders for dashboard inspect governance output.
//! Keeps scoring, deduping, and risk metadata out of the facade module.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use super::{
    build_dashboard_dependency_rows, build_inventory_lookup, normalize_family_name,
    resolve_datasource_identity, DashboardAuditRow, GovernanceRiskRow, QueryAuditRow,
};
use crate::dashboard::inspect_report::{ExportInspectionQueryReport, ExportInspectionQueryRow};
use crate::dashboard::ExportInspectionSummary;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GovernanceRiskSpec {
    category: &'static str,
    severity: &'static str,
    recommendation: &'static str,
}

const GOVERNANCE_RISK_KIND_MIXED_DASHBOARD: &str = "mixed-datasource-dashboard";
const GOVERNANCE_RISK_KIND_ORPHANED_DATASOURCE: &str = "orphaned-datasource";
const GOVERNANCE_RISK_KIND_UNKNOWN_DATASOURCE_FAMILY: &str = "unknown-datasource-family";
const GOVERNANCE_RISK_KIND_EMPTY_QUERY_ANALYSIS: &str = "empty-query-analysis";
const GOVERNANCE_RISK_KIND_BROAD_LOKI_SELECTOR: &str = "broad-loki-selector";
const GOVERNANCE_RISK_KIND_BROAD_PROMETHEUS_SELECTOR: &str = "broad-prometheus-selector";
const GOVERNANCE_RISK_KIND_PROMETHEUS_REGEX_HEAVY: &str = "prometheus-regex-heavy";
const GOVERNANCE_RISK_KIND_PROMETHEUS_HIGH_CARDINALITY_REGEX: &str =
    "prometheus-high-cardinality-regex";
const GOVERNANCE_RISK_KIND_PROMETHEUS_DEEP_AGGREGATION: &str = "prometheus-deep-aggregation";
const GOVERNANCE_RISK_KIND_LARGE_PROMETHEUS_RANGE: &str = "large-prometheus-range";
const GOVERNANCE_RISK_KIND_UNSCOPED_LOKI_SEARCH: &str = "unscoped-loki-search";
const GOVERNANCE_RISK_KIND_DASHBOARD_PANEL_PRESSURE: &str = "dashboard-panel-pressure";
const GOVERNANCE_RISK_KIND_DASHBOARD_REFRESH_PRESSURE: &str = "dashboard-refresh-pressure";

const GOVERNANCE_RISK_DEFAULT_SPEC: GovernanceRiskSpec = GovernanceRiskSpec {
    category: "other",
    severity: "low",
    recommendation:
        "Review this governance finding and assign a follow-up owner if action is needed.",
};

const GOVERNANCE_RISK_SPECS: [(&str, GovernanceRiskSpec); 13] = [
    (
        GOVERNANCE_RISK_KIND_MIXED_DASHBOARD,
        GovernanceRiskSpec {
            category: "topology",
            severity: "medium",
            recommendation:
                "Split panel queries by datasource or document why mixed datasource composition is required.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_ORPHANED_DATASOURCE,
        GovernanceRiskSpec {
            category: "inventory",
            severity: "low",
            recommendation:
                "Remove the unused datasource or reattach it to retained dashboards before the next cleanup cycle.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_UNKNOWN_DATASOURCE_FAMILY,
        GovernanceRiskSpec {
            category: "coverage",
            severity: "medium",
            recommendation:
                "Map this datasource plugin type to a known governance family or extend analyzer support for it.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_EMPTY_QUERY_ANALYSIS,
        GovernanceRiskSpec {
            category: "coverage",
            severity: "low",
            recommendation:
                "Review the query text and extend analyzer coverage if this datasource family should emit governance signals.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_BROAD_LOKI_SELECTOR,
        GovernanceRiskSpec {
            category: "cost",
            severity: "medium",
            recommendation:
                "Narrow the Loki stream selector before running expensive line filters or aggregations.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_BROAD_PROMETHEUS_SELECTOR,
        GovernanceRiskSpec {
            category: "cost",
            severity: "medium",
            recommendation:
                "Add label filters to the Prometheus selector before promoting this dashboard to shared or high-refresh use.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_PROMETHEUS_REGEX_HEAVY,
        GovernanceRiskSpec {
            category: "cost",
            severity: "medium",
            recommendation:
                "Reduce Prometheus regex matcher scope or replace it with exact labels where possible.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_PROMETHEUS_HIGH_CARDINALITY_REGEX,
        GovernanceRiskSpec {
            category: "cost",
            severity: "high",
            recommendation:
                "Avoid regex matchers on high-cardinality Prometheus labels such as instance, pod, or container unless the scope is already tightly bounded.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_PROMETHEUS_DEEP_AGGREGATION,
        GovernanceRiskSpec {
            category: "cost",
            severity: "medium",
            recommendation:
                "Reduce nested Prometheus aggregation layers or pre-aggregate upstream before adding more dashboard fanout.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_LARGE_PROMETHEUS_RANGE,
        GovernanceRiskSpec {
            category: "cost",
            severity: "medium",
            recommendation:
                "Shorten the Prometheus range window or pre-aggregate the series before using long lookback queries in dashboards.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_UNSCOPED_LOKI_SEARCH,
        GovernanceRiskSpec {
            category: "cost",
            severity: "high",
            recommendation:
                "Add at least one concrete Loki label matcher before running full-text or regex log search.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_DASHBOARD_PANEL_PRESSURE,
        GovernanceRiskSpec {
            category: "dashboard-load",
            severity: "medium",
            recommendation:
                "Split the dashboard into smaller views or collapse low-value panels before broad rollout.",
        },
    ),
    (
        GOVERNANCE_RISK_KIND_DASHBOARD_REFRESH_PRESSURE,
        GovernanceRiskSpec {
            category: "dashboard-load",
            severity: "medium",
            recommendation:
                "Increase the dashboard refresh interval to reduce repeated load on Grafana and backing datasources.",
        },
    ),
];

fn severity_for_score(score: usize) -> String {
    match score {
        0..=1 => "low".to_string(),
        2..=3 => "medium".to_string(),
        _ => "high".to_string(),
    }
}

fn ordered_push(values: &mut Vec<String>, candidate: &str) {
    let candidate = candidate.trim();
    if candidate.is_empty() {
        return;
    }
    if !values.iter().any(|value| value == candidate) {
        values.push(candidate.to_string());
    }
}

fn parse_duration_seconds(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("off") {
        return None;
    }
    let mut digits = String::new();
    let mut suffix = String::new();
    for character in trimmed.chars() {
        if character.is_ascii_digit() && suffix.is_empty() {
            digits.push(character);
        } else if !character.is_ascii_whitespace() {
            suffix.push(character);
        }
    }
    let number = digits.parse::<u64>().ok()?;
    let multiplier = match suffix.to_ascii_lowercase().as_str() {
        "ms" => 0,
        "s" | "" => 1,
        "m" => 60,
        "h" => 60 * 60,
        "d" => 60 * 60 * 24,
        "w" => 60 * 60 * 24 * 7,
        _ => return None,
    };
    Some(number.saturating_mul(multiplier))
}

fn query_uses_broad_prometheus_selector(row: &ExportInspectionQueryRow) -> bool {
    if normalize_family_name(&row.datasource_type) != "prometheus" {
        return false;
    }
    let query_text = row.query_text.trim();
    if query_text.is_empty() || !row.measurements.is_empty() || query_text.contains('{') {
        return false;
    }
    if row.metrics.len() != 1
        || query_text.contains(' ')
        || query_text.contains('(')
        || query_text.contains('[')
    {
        return false;
    }
    row.metrics[0].trim() == query_text
}

fn query_uses_prometheus_regex(row: &ExportInspectionQueryRow) -> bool {
    normalize_family_name(&row.datasource_type) == "prometheus"
        && (row.query_text.contains("=~") || row.query_text.contains("!~"))
}

fn prometheus_regex_matcher_count(row: &ExportInspectionQueryRow) -> usize {
    if normalize_family_name(&row.datasource_type) != "prometheus" {
        return 0;
    }
    row.query_text.matches("=~").count() + row.query_text.matches("!~").count()
}

fn query_uses_high_cardinality_prometheus_regex(row: &ExportInspectionQueryRow) -> bool {
    if normalize_family_name(&row.datasource_type) != "prometheus" {
        return false;
    }
    const HIGH_CARDINALITY_LABELS: [&str; 8] = [
        "instance",
        "pod",
        "container",
        "endpoint",
        "path",
        "uri",
        "name",
        "id",
    ];
    HIGH_CARDINALITY_LABELS.iter().any(|label| {
        row.query_text.contains(&format!("{label}=~"))
            || row.query_text.contains(&format!("{label}!~"))
    })
}

fn prometheus_aggregation_depth(row: &ExportInspectionQueryRow) -> usize {
    if normalize_family_name(&row.datasource_type) != "prometheus" {
        return 0;
    }
    const AGGREGATORS: [&str; 11] = [
        "sum",
        "avg",
        "min",
        "max",
        "count",
        "group",
        "count_values",
        "quantile",
        "topk",
        "bottomk",
        "stddev",
    ];
    row.functions
        .iter()
        .filter(|function: &&String| AGGREGATORS.contains(&function.as_str()))
        .count()
}

fn prometheus_estimated_series_risk(
    broad_selector: bool,
    regex_matcher_count: usize,
    high_cardinality_regex: bool,
    aggregation_depth: usize,
    largest_bucket_seconds: Option<u64>,
) -> String {
    let mut score = 0usize;
    if broad_selector {
        score += 2;
    }
    if regex_matcher_count != 0 {
        score += 1;
    }
    if high_cardinality_regex {
        score += 2;
    }
    if aggregation_depth >= 2 {
        score += 1;
    }
    if largest_bucket_seconds.unwrap_or(0) >= 60 * 60 {
        score += 1;
    }
    match score {
        0..=1 => "low".to_string(),
        2..=3 => "medium".to_string(),
        _ => "high".to_string(),
    }
}

fn largest_bucket_seconds(row: &ExportInspectionQueryRow) -> Option<u64> {
    row.buckets
        .iter()
        .filter_map(|value| parse_duration_seconds(value))
        .max()
}

fn loki_selector_has_concrete_matcher(selector: &str) -> bool {
    let inner = selector
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if inner.is_empty() {
        return false;
    }
    inner.split(',').any(|matcher| {
        let matcher = matcher.trim();
        if matcher.is_empty() {
            return false;
        }
        let Some((_, value)) = matcher.split_once('=') else {
            return false;
        };
        let value = value.trim().trim_matches('"');
        !(value.is_empty() || value == ".*" || value == ".+" || value == "*")
    })
}

fn extract_loki_stream_selectors(query_text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut capture_start: Option<usize> = None;
    for (index, character) in query_text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' if in_quotes => {
                escaped = true;
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            '{' if !in_quotes => {
                capture_start = Some(index);
            }
            '}' if !in_quotes => {
                if let Some(start) = capture_start.take() {
                    let selector = &query_text[start..index + character.len_utf8()];
                    if !values.iter().any(|value| value == selector) {
                        values.push(selector.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    values
}

fn split_loki_selector_matchers(selector: &str) -> Vec<String> {
    let mut matchers = Vec::new();
    let mut start = 0usize;
    let mut in_quotes = false;
    let mut escaped = false;
    for (index, character) in selector.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' if in_quotes => {
                escaped = true;
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                matchers.push(selector[start..index].trim().to_string());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    matchers.push(selector[start..].trim().to_string());
    matchers
}

fn loki_regex_is_wildcard(value: &str) -> bool {
    let trimmed = value.trim();
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed);
    matches!(unquoted.trim(), ".*" | "^.*$" | ".+" | "^.+$")
}

fn loki_selector_is_broad(selector: &str) -> bool {
    let trimmed = selector.trim();
    let Some(inner) = trimmed
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
    else {
        return false;
    };
    if inner.trim().is_empty() {
        return true;
    }
    let mut saw_matcher = false;
    for matcher in split_loki_selector_matchers(inner) {
        let matcher = matcher.trim();
        if matcher.is_empty() {
            continue;
        }
        saw_matcher = true;
        if let Some((_, value)) = matcher.split_once("=~") {
            if !loki_regex_is_wildcard(value) {
                return false;
            }
            continue;
        }
        if matcher.contains("!~") || matcher.contains("!=") || matcher.contains('=') {
            return false;
        }
        return false;
    }
    saw_matcher
}

pub(crate) fn find_broad_loki_selector(query_text: &str) -> Option<String> {
    extract_loki_stream_selectors(query_text)
        .into_iter()
        .find(|selector| loki_selector_is_broad(selector))
}

fn build_query_audit_row(
    row: &ExportInspectionQueryRow,
    datasource_name: String,
    datasource_uid: String,
    datasource_family: String,
) -> QueryAuditRow {
    let mut score = 0usize;
    let mut reasons = Vec::new();
    let mut recommendations = Vec::new();
    let broad_prometheus_selector = query_uses_broad_prometheus_selector(row);
    let prometheus_regex = query_uses_prometheus_regex(row);
    let regex_matcher_count = prometheus_regex_matcher_count(row);
    let high_cardinality_regex = query_uses_high_cardinality_prometheus_regex(row);
    let aggregation_depth = prometheus_aggregation_depth(row);
    let max_bucket_seconds = largest_bucket_seconds(row);
    if broad_prometheus_selector {
        score += 2;
        ordered_push(&mut reasons, GOVERNANCE_RISK_KIND_BROAD_PROMETHEUS_SELECTOR);
        ordered_push(
            &mut recommendations,
            lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_BROAD_PROMETHEUS_SELECTOR)
                .recommendation,
        );
    }
    if prometheus_regex {
        score += 1;
        ordered_push(&mut reasons, GOVERNANCE_RISK_KIND_PROMETHEUS_REGEX_HEAVY);
        ordered_push(
            &mut recommendations,
            lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_PROMETHEUS_REGEX_HEAVY).recommendation,
        );
    }
    if high_cardinality_regex {
        score += 2;
        ordered_push(
            &mut reasons,
            GOVERNANCE_RISK_KIND_PROMETHEUS_HIGH_CARDINALITY_REGEX,
        );
        ordered_push(
            &mut recommendations,
            lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_PROMETHEUS_HIGH_CARDINALITY_REGEX)
                .recommendation,
        );
    }
    if aggregation_depth >= 2 {
        score += 1;
        ordered_push(
            &mut reasons,
            GOVERNANCE_RISK_KIND_PROMETHEUS_DEEP_AGGREGATION,
        );
        ordered_push(
            &mut recommendations,
            lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_PROMETHEUS_DEEP_AGGREGATION)
                .recommendation,
        );
    }
    if max_bucket_seconds.unwrap_or(0) >= 60 * 60
        && normalize_family_name(&row.datasource_type) == "prometheus"
    {
        score += 2;
        ordered_push(&mut reasons, GOVERNANCE_RISK_KIND_LARGE_PROMETHEUS_RANGE);
        ordered_push(
            &mut recommendations,
            lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_LARGE_PROMETHEUS_RANGE).recommendation,
        );
    }
    if query_uses_unscoped_loki_search(row) {
        score += 3;
        ordered_push(&mut reasons, GOVERNANCE_RISK_KIND_UNSCOPED_LOKI_SEARCH);
        ordered_push(
            &mut recommendations,
            lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_UNSCOPED_LOKI_SEARCH).recommendation,
        );
    }
    let query_cost_score = score;
    let estimated_series_risk = prometheus_estimated_series_risk(
        broad_prometheus_selector,
        regex_matcher_count,
        high_cardinality_regex,
        aggregation_depth,
        max_bucket_seconds,
    );
    QueryAuditRow {
        dashboard_uid: row.dashboard_uid.clone(),
        dashboard_title: row.dashboard_title.clone(),
        folder_path: row.folder_path.clone(),
        panel_id: row.panel_id.clone(),
        panel_title: row.panel_title.clone(),
        ref_id: row.ref_id.clone(),
        datasource: datasource_name,
        datasource_uid,
        datasource_family,
        aggregation_depth,
        regex_matcher_count,
        estimated_series_risk,
        query_cost_score,
        score,
        severity: severity_for_score(score),
        reasons,
        recommendations,
    }
}

pub(crate) fn build_dashboard_audit_rows(
    report: &ExportInspectionQueryReport,
) -> Vec<DashboardAuditRow> {
    let refresh_by_dashboard = load_dashboard_refresh_by_uid(report);
    build_dashboard_dependency_rows(report)
        .into_iter()
        .map(|dashboard| {
            let mut score = 0usize;
            let mut reasons = Vec::new();
            let mut recommendations = Vec::new();
            if dashboard.panel_count > 30 {
                score += 2;
                ordered_push(&mut reasons, GOVERNANCE_RISK_KIND_DASHBOARD_PANEL_PRESSURE);
                ordered_push(
                    &mut recommendations,
                    lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_DASHBOARD_PANEL_PRESSURE)
                        .recommendation,
                );
            }
            let refresh_interval_seconds = refresh_by_dashboard
                .get(&dashboard.dashboard_uid)
                .and_then(|value| parse_duration_seconds(value));
            if refresh_interval_seconds.unwrap_or(0) != 0
                && refresh_interval_seconds.unwrap_or(0) < 30
            {
                score += if refresh_interval_seconds.unwrap_or(0) < 10 {
                    3
                } else {
                    2
                };
                ordered_push(
                    &mut reasons,
                    GOVERNANCE_RISK_KIND_DASHBOARD_REFRESH_PRESSURE,
                );
                ordered_push(
                    &mut recommendations,
                    lookup_governance_risk_spec(GOVERNANCE_RISK_KIND_DASHBOARD_REFRESH_PRESSURE)
                        .recommendation,
                );
            }
            DashboardAuditRow {
                dashboard_uid: dashboard.dashboard_uid,
                dashboard_title: dashboard.dashboard_title,
                folder_path: dashboard.folder_path,
                panel_count: dashboard.panel_count,
                query_count: dashboard.query_count,
                refresh_interval_seconds,
                score,
                severity: severity_for_score(score),
                reasons,
                recommendations,
            }
        })
        .collect()
}

fn query_uses_unscoped_loki_search(row: &ExportInspectionQueryRow) -> bool {
    if normalize_family_name(&row.datasource_type) != "loki" {
        return false;
    }
    let has_line_filter = row.functions.iter().any(|function: &String| {
        function.starts_with("line_filter_")
            || function.contains("pattern")
            || function.contains("regexp")
    });
    if !has_line_filter {
        return false;
    }
    let selectors = extract_loki_stream_selectors(&row.query_text);
    !selectors.is_empty()
        && selectors
            .iter()
            .all(|selector| !loki_selector_has_concrete_matcher(selector))
}

fn load_dashboard_refresh_by_uid(report: &ExportInspectionQueryReport) -> BTreeMap<String, String> {
    let mut refresh_by_uid = BTreeMap::new();
    let mut file_by_uid = BTreeMap::new();
    for row in &report.queries {
        if !row.dashboard_uid.trim().is_empty() && !row.file_path.trim().is_empty() {
            file_by_uid
                .entry(row.dashboard_uid.clone())
                .or_insert(row.file_path.clone());
        }
    }
    for (dashboard_uid, file_path) in file_by_uid {
        let Ok(raw) = fs::read_to_string(&file_path) else {
            continue;
        };
        let Ok(document) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let refresh = document
            .get("dashboard")
            .and_then(|dashboard| dashboard.get("refresh"))
            .or_else(|| document.get("refresh"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        if let Some(refresh) = refresh {
            refresh_by_uid.insert(dashboard_uid, refresh);
        }
    }
    refresh_by_uid
}

fn lookup_governance_risk_spec(kind: &str) -> GovernanceRiskSpec {
    GOVERNANCE_RISK_SPECS
        .iter()
        .copied()
        .find(|(registered_kind, _)| *registered_kind == kind)
        .map(|(_, spec)| spec)
        .unwrap_or(GOVERNANCE_RISK_DEFAULT_SPEC)
}

#[cfg(test)]
pub(crate) fn governance_risk_spec(kind: &str) -> (&'static str, &'static str, &'static str) {
    let spec = lookup_governance_risk_spec(kind);
    (spec.category, spec.severity, spec.recommendation)
}

fn build_governance_risk_row(
    kind: &str,
    dashboard_uid: String,
    panel_id: String,
    datasource: String,
    detail: String,
) -> GovernanceRiskRow {
    let spec = lookup_governance_risk_spec(kind);
    GovernanceRiskRow {
        kind: kind.to_string(),
        severity: spec.severity.to_string(),
        category: spec.category.to_string(),
        dashboard_uid,
        panel_id,
        datasource,
        detail,
        recommendation: spec.recommendation.to_string(),
    }
}

pub(crate) fn build_governance_risk_rows(
    summary: &ExportInspectionSummary,
    report: &ExportInspectionQueryReport,
) -> Vec<GovernanceRiskRow> {
    let (inventory_by_uid, inventory_by_name) = build_inventory_lookup(summary);
    let refresh_by_dashboard = load_dashboard_refresh_by_uid(report);
    let mut seen = BTreeSet::new();
    let mut risks = Vec::new();

    for dashboard in &summary.mixed_dashboards {
        let risk = build_governance_risk_row(
            GOVERNANCE_RISK_KIND_MIXED_DASHBOARD,
            dashboard.uid.clone(),
            String::new(),
            dashboard.datasources.join(","),
            dashboard.title.clone(),
        );
        if seen.insert(risk.clone()) {
            risks.push(risk);
        }
    }
    for datasource in &summary.datasource_inventory {
        if datasource.reference_count != 0 || datasource.dashboard_count != 0 {
            continue;
        }
        let risk = build_governance_risk_row(
            GOVERNANCE_RISK_KIND_ORPHANED_DATASOURCE,
            String::new(),
            String::new(),
            if datasource.uid.trim().is_empty() {
                datasource.name.clone()
            } else {
                datasource.uid.clone()
            },
            datasource.datasource_type.clone(),
        );
        if seen.insert(risk.clone()) {
            risks.push(risk);
        }
    }
    for row in &report.queries {
        let identity = resolve_datasource_identity(row, &inventory_by_uid, &inventory_by_name);
        if normalize_family_name(&identity.datasource_type) == "unknown" {
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_UNKNOWN_DATASOURCE_FAMILY,
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                identity.name.clone(),
                row.query_field.clone(),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
        if normalize_family_name(&identity.datasource_type) == "loki" {
            if let Some(selector) = find_broad_loki_selector(&row.query_text) {
                let risk = build_governance_risk_row(
                    GOVERNANCE_RISK_KIND_BROAD_LOKI_SELECTOR,
                    row.dashboard_uid.clone(),
                    row.panel_id.clone(),
                    identity.name.clone(),
                    selector,
                );
                if seen.insert(risk.clone()) {
                    risks.push(risk);
                }
            }
        }
        if query_uses_broad_prometheus_selector(row) {
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_BROAD_PROMETHEUS_SELECTOR,
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                identity.name.clone(),
                row.query_text.clone(),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
        if query_uses_prometheus_regex(row) {
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_PROMETHEUS_REGEX_HEAVY,
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                identity.name.clone(),
                row.query_text.clone(),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
        if query_uses_high_cardinality_prometheus_regex(row) {
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_PROMETHEUS_HIGH_CARDINALITY_REGEX,
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                identity.name.clone(),
                row.query_text.clone(),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
        if prometheus_aggregation_depth(row) >= 2 {
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_PROMETHEUS_DEEP_AGGREGATION,
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                identity.name.clone(),
                row.query_text.clone(),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
        if let Some(bucket_seconds) = largest_bucket_seconds(row) {
            if normalize_family_name(&identity.datasource_type) == "prometheus"
                && bucket_seconds >= 60 * 60
            {
                let risk = build_governance_risk_row(
                    GOVERNANCE_RISK_KIND_LARGE_PROMETHEUS_RANGE,
                    row.dashboard_uid.clone(),
                    row.panel_id.clone(),
                    identity.name.clone(),
                    row.buckets.join(","),
                );
                if seen.insert(risk.clone()) {
                    risks.push(risk);
                }
            }
        }
        if query_uses_unscoped_loki_search(row) {
            let selectors = extract_loki_stream_selectors(&row.query_text);
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_UNSCOPED_LOKI_SEARCH,
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                identity.name.clone(),
                selectors.join(","),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
        if row.metrics.is_empty()
            && row.functions.is_empty()
            && row.measurements.is_empty()
            && row.buckets.is_empty()
        {
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_EMPTY_QUERY_ANALYSIS,
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                identity.name,
                row.query_field.clone(),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
    }
    for dashboard in build_dashboard_dependency_rows(report) {
        if dashboard.panel_count > 30 {
            let risk = build_governance_risk_row(
                GOVERNANCE_RISK_KIND_DASHBOARD_PANEL_PRESSURE,
                dashboard.dashboard_uid.clone(),
                String::new(),
                dashboard.dashboard_title.clone(),
                dashboard.panel_count.to_string(),
            );
            if seen.insert(risk.clone()) {
                risks.push(risk);
            }
        }
        if let Some(refresh) = refresh_by_dashboard.get(&dashboard.dashboard_uid) {
            if let Some(refresh_seconds) = parse_duration_seconds(refresh) {
                if refresh_seconds != 0 && refresh_seconds < 30 {
                    let risk = build_governance_risk_row(
                        GOVERNANCE_RISK_KIND_DASHBOARD_REFRESH_PRESSURE,
                        dashboard.dashboard_uid.clone(),
                        String::new(),
                        dashboard.dashboard_title.clone(),
                        refresh.clone(),
                    );
                    if seen.insert(risk.clone()) {
                        risks.push(risk);
                    }
                }
            }
        }
    }
    risks.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then(left.kind.cmp(&right.kind))
            .then(left.dashboard_uid.cmp(&right.dashboard_uid))
            .then(left.panel_id.cmp(&right.panel_id))
            .then(left.datasource.cmp(&right.datasource))
    });
    risks
}

pub(crate) fn build_query_audit_rows(
    summary: &ExportInspectionSummary,
    report: &ExportInspectionQueryReport,
) -> Vec<QueryAuditRow> {
    let (inventory_by_uid, inventory_by_name) = build_inventory_lookup(summary);
    let mut rows = Vec::new();
    for row in &report.queries {
        let identity = resolve_datasource_identity(row, &inventory_by_uid, &inventory_by_name);
        let datasource_family = normalize_family_name(&identity.datasource_type);
        let audit = build_query_audit_row(row, identity.name, identity.uid, datasource_family);
        if audit.score != 0 {
            rows.push(audit);
        }
    }
    rows.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then(left.dashboard_uid.cmp(&right.dashboard_uid))
            .then(left.panel_id.cmp(&right.panel_id))
            .then(left.ref_id.cmp(&right.ref_id))
    });
    rows
}
