//! Offline dependency report models for future richer inspection modes.
//!
//! This module is a standalone contract builder for dependency-oriented
//! inspection artifacts and intentionally keeps runtime behavior local to staged
//! document shapes.

use crate::dashboard::DatasourceInventoryItem;
use crate::dashboard_reference_models::{
    build_query_reference_payload, dedupe_strings, normalize_family_name, DashboardQueryReference,
    QueryFeatureSet,
};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Struct definition for DependencyUsageSummary.
#[derive(Debug, Clone)]
pub struct DependencyUsageSummary {
    pub datasource_identity: String,
    pub family: String,
    pub query_count: usize,
    pub dashboard_count: usize,
    pub panel_count: usize,
    pub reference_count: usize,
    pub query_fields: Vec<String>,
}

impl DependencyUsageSummary {
    /// as json.
    pub fn as_json(&self) -> Value {
        // Call graph (hierarchy): this function is used in related modules.
        // Upstream callers: 無
        // Downstream callees: 無

        json!({
            "datasource": self.datasource_identity,
            "family": self.family,
            "queryCount": self.query_count,
            "dashboardCount": self.dashboard_count,
            "panelCount": self.panel_count,
            "referenceCount": self.reference_count,
            "queryFields": self.query_fields,
        })
    }
}

/// Struct definition for OfflineDependencyReportDocument.
#[derive(Debug, Clone)]
pub struct OfflineDependencyReportDocument {
    pub summary: BTreeMap<String, Value>,
    pub queries: Vec<DashboardQueryReference>,
    pub query_features: BTreeMap<String, QueryFeatureSet>,
    pub(crate) dashboard_dependencies: Vec<DashboardDependencySummary>,
    pub usage: Vec<DependencyUsageSummary>,
    pub orphaned: Vec<String>,
}

impl OfflineDependencyReportDocument {
    /// as json.
    pub fn as_json(&self) -> Value {
        // Call graph (hierarchy): this function is used in related modules.
        // Upstream callers: 無
        // Downstream callees: dashboard_inspection_dependency_contract.rs:query_signature_key

        let queries: Vec<Value> = self
            .queries
            .iter()
            .map(|query| {
                let feature = self
                    .query_features
                    .get(&query_signature_key(query))
                    .cloned()
                    .unwrap_or_else(QueryFeatureSet::blank);
                let mut record = Map::new();
                record.insert(
                    "dashboardUid".to_string(),
                    Value::String(query.dashboard_uid.clone()),
                );
                record.insert(
                    "dashboardTitle".to_string(),
                    Value::String(query.dashboard_title.clone()),
                );
                record.insert("panelId".to_string(), Value::String(query.panel_id.clone()));
                record.insert(
                    "panelTitle".to_string(),
                    Value::String(query.panel_title.clone()),
                );
                record.insert(
                    "panelType".to_string(),
                    Value::String(query.panel_type.clone()),
                );
                record.insert("refId".to_string(), Value::String(query.ref_id.clone()));
                record.insert(
                    "datasource".to_string(),
                    Value::String(query.datasource_name.clone()),
                );
                record.insert(
                    "datasourceUid".to_string(),
                    Value::String(query.datasource_uid.clone()),
                );
                record.insert(
                    "datasourceType".to_string(),
                    Value::String(query.datasource_type.clone()),
                );
                record.insert(
                    "datasourceFamily".to_string(),
                    Value::String(query.datasource_family.clone()),
                );
                record.insert("file".to_string(), Value::String(query.file.clone()));
                record.insert(
                    "queryField".to_string(),
                    Value::String(query.query_field.clone()),
                );
                record.insert("query".to_string(), Value::String(query.query.clone()));
                record.insert(
                    "analysis".to_string(),
                    json!({
                        "metrics": feature.metrics,
                        "functions": feature.functions,
                        "measurements": feature.measurements,
                        "buckets": feature.buckets,
                        "labels": feature.labels,
                    }),
                );
                Value::Object(record)
            })
            .collect();

        json!({
            "kind": "grafana-utils-dashboard-dependency-contract",
            "summary": serde_json::to_value(&self.summary).unwrap_or_else(|_| json!({})),
            "queryCount": self.queries.len(),
            "datasourceCount": self.usage.len(),
            "orphanedDatasourceCount": self.orphaned.len(),
            "queries": queries,
            "datasourceUsage": self.usage.iter().map(|item| item.as_json()).collect::<Vec<_>>(),
            "dashboardDependencies": self
                .dashboard_dependencies
                .iter()
                .map(|item| item.as_json())
                .collect::<Vec<_>>(),
            "orphanedDatasources": self
                .orphaned
                .iter()
                .map(|value| Value::String(value.clone()))
                .collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone)]
struct QueryFeatureHints {
    metrics: Vec<String>,
    functions: Vec<String>,
    measurements: Vec<String>,
    buckets: Vec<String>,
    labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DashboardDependencySummary {
    dashboard_uid: String,
    dashboard_title: String,
    query_count: usize,
    panel_count: usize,
    datasource_count: usize,
    datasource_family_count: usize,
    query_fields: Vec<String>,
    metrics: Vec<String>,
    functions: Vec<String>,
    measurements: Vec<String>,
    buckets: Vec<String>,
}

impl DashboardDependencySummary {
    fn as_json(&self) -> Value {
        json!({
            "dashboardUid": self.dashboard_uid.clone(),
            "dashboardTitle": self.dashboard_title.clone(),
            "queryCount": self.query_count,
            "panelCount": self.panel_count,
            "datasourceCount": self.datasource_count,
            "datasourceFamilyCount": self.datasource_family_count,
            "queryFields": self.query_fields,
            "metrics": self.metrics,
            "functions": self.functions,
            "measurements": self.measurements,
            "buckets": self.buckets,
        })
    }
}

#[derive(Debug, Clone)]
struct DashboardDependencyAccumulator {
    dashboard_title: String,
    query_count: usize,
    panel_keys: BTreeSet<String>,
    datasource_identities: BTreeSet<String>,
    datasource_families: BTreeSet<String>,
    query_fields: BTreeSet<String>,
    metrics: BTreeSet<String>,
    functions: BTreeSet<String>,
    measurements: BTreeSet<String>,
    buckets: BTreeSet<String>,
}

fn query_signature_key(row: &DashboardQueryReference) -> String {
    format!("{}|{}|{}", row.dashboard_uid, row.panel_id, row.ref_id)
}

fn parse_query_text_families(row: &DashboardQueryReference) -> QueryFeatureHints {
    // Call graph (hierarchy): this function is used in related modules.
    // Upstream callers: 無
    // Downstream callees: dashboard_reference_models.rs:dedupe_strings, dashboard_reference_models.rs:normalize_family_name

    let family = normalize_family_name(&row.datasource_type);
    let query = &row.query;
    match family.as_str() {
        "prometheus" | "graphite" | "victoriametrics" => parse_prometheus_query_features(query),
        "loki" => parse_loki_query_features(query),
        "flux" | "influxdb" => parse_flux_query_features(query),
        "mysql" | "postgres" | "postgresql" | "sql" => parse_sql_query_features(query),
        _ => parse_unknown_query_features(query),
    }
}

fn ordered_unique_push(values: &mut Vec<String>, candidate: &str) {
    let value = candidate.trim();
    if value.is_empty() {
        return;
    }
    if !values.iter().any(|item| item == value) {
        values.push(value.to_string());
    }
}

fn quoted_captures(query_text: &str, pattern: &str) -> Vec<String> {
    let regex = Regex::new(pattern).expect("invalid hard-coded regex");
    let mut values = Vec::new();
    for captures in regex.captures_iter(query_text) {
        if let Some(value) = captures.get(1) {
            values.push(value.as_str().to_string());
        }
    }
    values
}

fn strip_quoted_text(query_text: &str) -> String {
    let mut out = String::with_capacity(query_text.len());
    let mut in_quotes = false;
    let mut escaped = false;

    for c in query_text.chars() {
        if escaped {
            escaped = false;
            out.push(' ');
            continue;
        }
        if c == '\\' && in_quotes {
            escaped = true;
            out.push(' ');
            continue;
        }
        if c == '"' {
            in_quotes = !in_quotes;
            out.push(' ');
            continue;
        }
        if in_quotes {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

fn extract_quoted_aware_enclosed(query_text: &str, open: char, close: char) -> Vec<String> {
    let mut values = Vec::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut begin = None;

    for (index, value) in query_text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match value {
            '\\' if in_quotes => {
                escaped = true;
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c == open && !in_quotes => {
                if depth == 0 {
                    begin = Some(index + c.len_utf8());
                }
                depth += 1;
            }
            c if c == close && !in_quotes => {
                if depth > 0 {
                    depth -= 1;
                }
                if depth == 0 {
                    if let Some(start) = begin.take() {
                        ordered_unique_push(&mut values, &query_text[start..index]);
                    }
                }
            }
            _ => {}
        }
    }
    values
}

fn parse_prometheus_query_features(query_text: &str) -> QueryFeatureHints {
    let mut functions = Vec::new();
    let mut metrics = Vec::new();
    let mut buckets = Vec::new();

    for value in extract_prometheus_metric_names(query_text) {
        ordered_unique_push(&mut metrics, &value);
    }
    for value in extract_prometheus_functions(query_text) {
        ordered_unique_push(&mut functions, &value);
    }
    for value in extract_prometheus_range_windows(query_text) {
        ordered_unique_push(&mut buckets, &value);
    }

    QueryFeatureHints {
        metrics: dedupe_strings(&metrics),
        functions: dedupe_strings(&functions),
        measurements: Vec::new(),
        buckets: dedupe_strings(&buckets),
        labels: Vec::new(),
    }
}

fn parse_loki_query_features(query_text: &str) -> QueryFeatureHints {
    let mut functions = Vec::new();
    let mut measurements = Vec::new();
    let mut buckets = Vec::new();
    let mut labels = Vec::new();

    for value in extract_loki_selectors(query_text) {
        ordered_unique_push(&mut measurements, &format!("{{{}}}", value));
        for matcher in extract_loki_label_matchers(query_text, &value) {
            ordered_unique_push(&mut measurements, &matcher);
            ordered_unique_push(&mut labels, &matcher);
        }
    }
    for value in extract_loki_pipeline_functions(query_text) {
        ordered_unique_push(&mut functions, &value);
    }
    for hint in extract_loki_filter_hints(query_text) {
        ordered_unique_push(&mut functions, &hint);
    }
    for value in extract_loki_range_windows(query_text) {
        ordered_unique_push(&mut buckets, &value);
    }

    QueryFeatureHints {
        metrics: Vec::new(),
        functions: dedupe_strings(&functions),
        measurements: dedupe_strings(&measurements),
        buckets: dedupe_strings(&buckets),
        labels: dedupe_strings(&labels),
    }
}

fn parse_flux_query_features(query_text: &str) -> QueryFeatureHints {
    let mut functions = Vec::new();
    let mut measurements = Vec::new();
    let mut buckets = Vec::new();

    if query_text.to_lowercase().trim_start().contains("from(") || query_text.contains("|>") {
        for value in extract_flux_pipeline_functions(query_text) {
            ordered_unique_push(&mut functions, &value);
        }
        for value in extract_flux_buckets(query_text) {
            ordered_unique_push(&mut buckets, &value);
        }
        for value in extract_flux_source_references(query_text) {
            ordered_unique_push(&mut measurements, &value);
        }
    }

    QueryFeatureHints {
        metrics: Vec::new(),
        functions: dedupe_strings(&functions),
        measurements: dedupe_strings(&measurements),
        buckets: dedupe_strings(&buckets),
        labels: Vec::new(),
    }
}

fn parse_sql_query_features(query_text: &str) -> QueryFeatureHints {
    QueryFeatureHints {
        metrics: Vec::new(),
        functions: extract_sql_query_shape_hints(query_text),
        measurements: extract_sql_source_references(query_text),
        buckets: Vec::new(),
        labels: Vec::new(),
    }
}

fn parse_unknown_query_features(query_text: &str) -> QueryFeatureHints {
    let sanitized = strip_quoted_text(&query_text.to_lowercase());
    let mut functions = Vec::new();
    for capture in quoted_captures(&sanitized, r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(") {
        ordered_unique_push(&mut functions, &capture);
    }
    QueryFeatureHints {
        metrics: Vec::new(),
        functions: dedupe_strings(&functions),
        measurements: Vec::new(),
        buckets: Vec::new(),
        labels: Vec::new(),
    }
}

fn extract_prometheus_metric_names(query_text: &str) -> Vec<String> {
    let query_text = strip_quoted_text(query_text);
    let token_regex =
        Regex::new(r"[A-Za-z_:][A-Za-z0-9_:]*").expect("invalid hard-coded metric regex");
    let quoted_name_regex =
        Regex::new(r#"\"(?:\\.|[^\"\\])*\""#).expect("invalid hard-coded quoted regex");
    let matcher_regex = Regex::new(r"\{[^{}]*\}").expect("invalid hard-coded matcher regex");
    let vector_matching_regex = Regex::new(r"\b(?:by|without|on|ignoring)\s*\(\s*[^)]*\)")
        .expect("invalid hard-coded vector matching regex");
    let group_modifier_regex = Regex::new(r"\b(?:group_left|group_right)\s*(?:\(\s*[^)]*\))?")
        .expect("invalid hard-coded group modifier regex");
    let mut values = Vec::new();
    let reserved_words = [
        "and",
        "bool",
        "by",
        "group_left",
        "group_right",
        "ignoring",
        "offset",
        "on",
        "or",
        "unless",
        "without",
        "sum",
        "min",
        "max",
        "avg",
        "count",
        "stddev",
        "stdvar",
        "bottomk",
        "topk",
        "quantile",
        "count_values",
        "rate",
        "irate",
        "increase",
        "delta",
        "idelta",
        "deriv",
        "predict_linear",
        "holt_winters",
        "sort",
        "sort_desc",
        "label_replace",
        "label_join",
        "histogram_quantile",
        "clamp_max",
        "clamp_min",
        "abs",
        "absent",
        "ceil",
        "floor",
        "ln",
        "log2",
        "log10",
        "round",
        "scalar",
        "vector",
        "year",
        "month",
        "day_of_month",
        "day_of_week",
        "hour",
        "minute",
        "time",
    ];

    let query = quoted_name_regex.replace_all(&query_text, "\"\"");
    for capture in quoted_captures(&query, r#"__name__\s*=\s*\"([A-Za-z_:][A-Za-z0-9_:]*)\""#) {
        ordered_unique_push(&mut values, &capture);
    }
    let sanitized_query = vector_matching_regex.replace_all(&query, " ").into_owned();
    let sanitized_query = group_modifier_regex
        .replace_all(&sanitized_query, " ")
        .into_owned();
    let sanitized_query = matcher_regex
        .replace_all(&sanitized_query, "{}")
        .into_owned();

    for token in token_regex.find_iter(&sanitized_query) {
        let start = token.start();
        let end = token.end();
        let previous = sanitized_query[..start]
            .chars()
            .next_back()
            .map(|value| value.is_ascii_alphanumeric() || value == '_' || value == ':')
            .unwrap_or(false);
        if previous {
            continue;
        }
        let next = sanitized_query[end..]
            .chars()
            .next()
            .map(|value| value.is_ascii_alphanumeric() || value == '_' || value == ':')
            .unwrap_or(false);
        if next {
            continue;
        }
        let token = token.as_str();
        if reserved_words.contains(&token) {
            continue;
        }
        let trailing = sanitized_query[end..].trim_start();
        if trailing.starts_with('(')
            || ["=", "!=", "=~", "!~"]
                .iter()
                .any(|operator| trailing.starts_with(operator))
        {
            continue;
        }
        ordered_unique_push(&mut values, token);
    }
    values
}

fn extract_prometheus_functions(query_text: &str) -> Vec<String> {
    let function_regex = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(")
        .expect("invalid hard-coded promql function regex");
    let mut values = Vec::new();
    for captures in function_regex.captures_iter(&strip_quoted_text(query_text)) {
        if let Some(value) = captures.get(1) {
            let name = value.as_str();
            if ![
                "by",
                "without",
                "on",
                "ignoring",
                "group_left",
                "group_right",
            ]
            .contains(&name)
            {
                ordered_unique_push(&mut values, name);
            }
        }
    }
    values
}

fn extract_prometheus_range_windows(query_text: &str) -> Vec<String> {
    let mut values = Vec::new();
    for value in extract_quoted_aware_enclosed(query_text, '[', ']') {
        ordered_unique_push(&mut values, value.trim());
    }
    values
}

fn extract_loki_selectors(query_text: &str) -> Vec<String> {
    let mut values = Vec::new();
    for value in extract_quoted_aware_enclosed(query_text, '{', '}') {
        let candidate = value.trim();
        if candidate.is_empty() {
            continue;
        }
        if !candidate.contains('=') && !candidate.contains('!') {
            continue;
        }
        ordered_unique_push(&mut values, candidate);
    }
    values
}

fn extract_loki_label_matchers(_query_text: &str, selector: &str) -> Vec<String> {
    let mut values = Vec::new();
    for value in quoted_captures(
        selector,
        r#"([A-Za-z_][A-Za-z0-9_]*\s*(?:=|!=|=~|!~)\s*(?:\"(?:\\.|[^\"\\])*\"|'[^']*'|[0-9A-Za-z_.-]+))"#,
    ) {
        ordered_unique_push(&mut values, &value);
    }
    values
}

fn extract_loki_pipeline_functions(query_text: &str) -> Vec<String> {
    let sanitized_query = strip_quoted_text(query_text);
    let function_regex = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(")
        .expect("invalid hard-coded loki function regex");
    let stage_regex = Regex::new(r"\|\s*([A-Za-z_][A-Za-z0-9_]*)(?:\s|\(|$)")
        .expect("invalid hard-coded loki stage regex");
    let mut functions = Vec::new();
    for captures in function_regex.captures_iter(&sanitized_query) {
        if let Some(value) = captures.get(1) {
            let name = value.as_str();
            if matches!(
                name,
                "by" | "without" | "on" | "ignoring" | "group_left" | "group_right"
            ) {
                continue;
            }
            ordered_unique_push(&mut functions, name);
        }
    }
    for captures in stage_regex.captures_iter(&sanitized_query) {
        if let Some(value) = captures.get(1) {
            ordered_unique_push(&mut functions, value.as_str());
        }
    }
    functions
}

fn extract_loki_filter_hints(query_text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let bytes = query_text.as_bytes();
    let mut index = 0;
    let mut in_quotes = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
            index += 1;
            continue;
        }
        match byte {
            b'\\' if in_quotes => {
                escaped = true;
                index += 1;
            }
            b'"' => {
                in_quotes = !in_quotes;
                index += 1;
            }
            b'|' if !in_quotes => {
                let mut cursor = index + 1;
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                let Some(operator) = bytes.get(cursor).copied() else {
                    index += 1;
                    continue;
                };
                let (token, separator_len) = match operator {
                    b'=' => ("line_filter_contains", 1),
                    b'~' => ("line_filter_regex", 1),
                    _ => {
                        index += 1;
                        continue;
                    }
                };
                cursor += separator_len;
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if bytes.get(cursor) != Some(&b'"') {
                    index += 1;
                    continue;
                }
                cursor += 1;
                let literal_start = cursor;
                let mut escaped_literal = false;
                while cursor < bytes.len() {
                    if escaped_literal {
                        escaped_literal = false;
                        cursor += 1;
                        continue;
                    }
                    match bytes[cursor] {
                        b'\\' => {
                            escaped_literal = true;
                            cursor += 1;
                        }
                        b'"' => {
                            ordered_unique_push(&mut values, token);
                            let literal = &query_text[literal_start..cursor];
                            ordered_unique_push(&mut values, &format!("{token}:{literal}"));
                            index = cursor + 1;
                            break;
                        }
                        _ => {
                            cursor += 1;
                        }
                    }
                }
                if cursor >= bytes.len() {
                    break;
                }
            }
            _ => {
                index += 1;
            }
        }
    }
    values
}

fn extract_loki_range_windows(query_text: &str) -> Vec<String> {
    extract_quoted_aware_enclosed(query_text, '[', ']')
}

fn extract_flux_pipeline_functions(query_text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut has_flux = false;
    let trimmed = query_text.trim_start().to_lowercase();
    if trimmed.starts_with("from(") || query_text.contains("|>") {
        has_flux = true;
    }
    if has_flux {
        let regex = Regex::new(r#"(?i)\b([A-Za-z_][A-Za-z0-9_]*)\s*\("#)
            .expect("invalid hard-coded flux function regex");
        let first_stage = Regex::new(r#"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*\("#)
            .expect("invalid hard-coded flux first-stage regex");
        if let Some(matches) = first_stage.captures(&trimmed) {
            if let Some(value) = matches.get(1) {
                ordered_unique_push(&mut values, value.as_str());
            }
        }
        for captured in regex.captures_iter(query_text) {
            if let Some(value) = captured.get(1) {
                ordered_unique_push(&mut values, value.as_str());
            }
        }
        let pipeline_regex = Regex::new(r#"\|>\s*([A-Za-z_][A-Za-z0-9_]*)(?:\(|\s|$)"#)
            .expect("invalid hard-coded flux pipeline regex");
        for captured in pipeline_regex.captures_iter(query_text) {
            if let Some(value) = captured.get(1) {
                ordered_unique_push(&mut values, value.as_str());
            }
        }
    }
    values
}

fn extract_flux_buckets(query_text: &str) -> Vec<String> {
    let mut values = Vec::new();
    for value in quoted_captures(
        &Regex::new(r#""(?:\\.|[^"\\])*""#)
            .expect("invalid hard-coded quoted regex")
            .replace_all(query_text, "\"\""),
        r#"from\(\s*bucket\s*:\s*\"([^\"]+)\"\)"#,
    ) {
        ordered_unique_push(&mut values, &value);
    }
    for value in quoted_captures(
        &strip_quoted_text(query_text),
        r"(?i)\bevery\s*:\s*([0-9]+(?:ns|us|µs|ms|s|m|h|d|w|mo|y))",
    ) {
        ordered_unique_push(&mut values, &value);
    }
    for value in quoted_captures(
        &strip_quoted_text(query_text),
        r"(?i)\btime\s*\(\s*([^)]+?)\s*\)",
    ) {
        if !value.trim().is_empty() {
            ordered_unique_push(&mut values, &value);
        }
    }
    values
}

fn extract_flux_source_references(query_text: &str) -> Vec<String> {
    extract_influxql_select_metrics(query_text)
}

fn extract_sql_source_references(query_text: &str) -> Vec<String> {
    let query_text = strip_sql_comments(query_text);
    if query_text.trim().is_empty() {
        return Vec::new();
    }
    let cte_names = quoted_captures(
        &query_text,
        r#"(?i)\bwith\s+([A-Za-z_][A-Za-z0-9_$]*)\s+as\s*\("#,
    )
    .into_iter()
    .map(|value| value.to_ascii_lowercase())
    .collect::<std::collections::BTreeSet<String>>();
    let mut values = Vec::new();
    for value in quoted_captures(
        &query_text,
        r#"(?i)\b(?:from|join|update|into|delete\s+from)\s+((?:[A-Za-z_][A-Za-z0-9_$]*|\"[^\"]+\"|`[^`]+`|\[[^\]]+\])(?:\s*\.\s*(?:[A-Za-z_][A-Za-z0-9_$]*|\"[^\"]+\"|`[^`]+`|\[[^\]]+\])){0,2})"#,
    ) {
        let normalized = normalize_sql_identifier(&value);
        if !normalized.is_empty() && !cte_names.contains(&normalized.to_ascii_lowercase()) {
            ordered_unique_push(&mut values, &normalized);
        }
    }
    values
}

fn extract_sql_query_shape_hints(query_text: &str) -> Vec<String> {
    let lowered = strip_sql_comments(query_text).to_ascii_lowercase();
    let patterns = [
        ("with", r"\bwith\b"),
        ("select", r"\bselect\b"),
        ("insert", r"\binsert\s+into\b"),
        ("update", r"\bupdate\b"),
        ("delete", r"\bdelete\s+from\b"),
        ("distinct", r"\bdistinct\b"),
        ("join", r"\bjoin\b"),
        ("where", r"\bwhere\b"),
        ("group_by", r"\bgroup\s+by\b"),
        ("having", r"\bhaving\b"),
        ("order_by", r"\border\s+by\b"),
        ("limit", r"\blimit\b"),
        ("top", r"\btop\s+\d+\b"),
        ("union", r"\bunion(?:\s+all)?\b"),
        ("window", r"\bover\s*\("),
        ("subquery", r"\b(?:from|join)\s*\("),
    ];
    let mut values = Vec::new();
    for (name, pattern) in patterns {
        let regex = Regex::new(pattern).expect("invalid hard-coded shape regex");
        if regex.is_match(&lowered) {
            ordered_unique_push(&mut values, name);
        }
    }
    values
}

fn normalize_sql_identifier(value: &str) -> String {
    value
        .split('.')
        .filter_map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                return None;
            }
            let normalized = if trimmed.len() >= 2
                && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
                    || (trimmed.starts_with('`') && trimmed.ends_with('`'))
                    || (trimmed.starts_with('[') && trimmed.ends_with(']')))
            {
                &trimmed[1..trimmed.len() - 1]
            } else {
                trimmed
            };
            let normalized = normalized.trim();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized.to_string())
            }
        })
        .collect::<Vec<String>>()
        .join(".")
}

fn strip_sql_comments(query_text: &str) -> String {
    let block_regex = Regex::new(r"(?s)/\*.*?\*/").expect("invalid hard-coded sql comment regex");
    let line_regex = Regex::new(r"--[^\n]*").expect("invalid hard-coded sql line comment regex");
    let without_blocks = block_regex.replace_all(query_text, " ");
    line_regex.replace_all(&without_blocks, " ").into_owned()
}

fn extract_influxql_select_metrics(query_text: &str) -> Vec<String> {
    let query_text = strip_sql_comments(query_text);
    let query_text = Regex::new(r#"(?is)\s*select\s+(.*?)\s+\bfrom\b"#)
        .expect("invalid hard-coded influxql select regex")
        .captures(&query_text)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().trim().to_string())
        .unwrap_or_default();
    if query_text.is_empty() {
        return Vec::new();
    }
    let mut values = Vec::new();
    for value in quoted_captures(&query_text, r#""([^"]+)""#) {
        ordered_unique_push(&mut values, &value);
    }
    values
}

fn parse_features_from_object(row: &Value) -> QueryFeatureHints {
    let mut metrics = Vec::new();
    let mut functions = Vec::new();
    let mut measurements = Vec::new();
    let mut buckets = Vec::new();
    let mut labels = Vec::new();

    if let Some(analysis) = row.get("analysis").and_then(Value::as_object) {
        let collect = |key: &str, target: &mut Vec<String>| {
            if let Some(items) = analysis.get(key).and_then(Value::as_array) {
                for item in items {
                    if let Some(text) = item.as_str() {
                        target.push(text.to_string());
                    }
                }
            }
        };
        collect("metrics", &mut metrics);
        collect("functions", &mut functions);
        collect("measurements", &mut measurements);
        collect("buckets", &mut buckets);
        collect("labels", &mut labels);
    }

    if metrics.is_empty() {
        if let Some(items) = row.get("metrics").and_then(Value::as_array) {
            for item in items {
                if let Some(text) = item.as_str() {
                    metrics.push(text.to_string());
                }
            }
        }
    }
    if functions.is_empty() {
        if let Some(items) = row.get("functions").and_then(Value::as_array) {
            for item in items {
                if let Some(text) = item.as_str() {
                    functions.push(text.to_string());
                }
            }
        }
    }
    if measurements.is_empty() {
        if let Some(items) = row.get("measurements").and_then(Value::as_array) {
            for item in items {
                if let Some(text) = item.as_str() {
                    measurements.push(text.to_string());
                }
            }
        }
    }
    if buckets.is_empty() {
        if let Some(items) = row.get("buckets").and_then(Value::as_array) {
            for item in items {
                if let Some(text) = item.as_str() {
                    buckets.push(text.to_string());
                }
            }
        }
    }

    QueryFeatureHints {
        metrics: dedupe_strings(&metrics),
        functions: dedupe_strings(&functions),
        measurements: dedupe_strings(&measurements),
        buckets: dedupe_strings(&buckets),
        labels: dedupe_strings(&labels),
    }
}

fn build_query_features(row: &Value, reference: &DashboardQueryReference) -> QueryFeatureHints {
    let mut hints = parse_query_text_families(reference);
    let analysis_hints = parse_features_from_object(row);

    for value in analysis_hints.metrics {
        if !hints.metrics.iter().any(|item| item == &value) {
            hints.metrics.push(value);
        }
    }
    for value in analysis_hints.functions {
        if !hints.functions.iter().any(|item| item == &value) {
            hints.functions.push(value);
        }
    }
    for value in analysis_hints.measurements {
        if !hints.measurements.iter().any(|item| item == &value) {
            hints.measurements.push(value);
        }
    }
    for value in analysis_hints.buckets {
        if !hints.buckets.iter().any(|item| item == &value) {
            hints.buckets.push(value);
        }
    }
    for value in analysis_hints.labels {
        if !hints.labels.iter().any(|item| item == &value) {
            hints.labels.push(value);
        }
    }
    QueryFeatureHints {
        metrics: dedupe_strings(&hints.metrics),
        functions: dedupe_strings(&hints.functions),
        measurements: dedupe_strings(&hints.measurements),
        buckets: dedupe_strings(&hints.buckets),
        labels: dedupe_strings(&hints.labels),
    }
}

/// Purpose: implementation note.
pub(crate) fn build_offline_dependency_contract(
    query_report_rows: &[Value],
    datasource_inventory: &[DatasourceInventoryItem],
) -> Value {
    let mut queries = Vec::new();
    let mut query_features = BTreeMap::new();
    let mut dashboard_dependencies = BTreeMap::<String, DashboardDependencyAccumulator>::new();
    let mut usage =
        BTreeMap::<String, (DependencyUsageSummary, BTreeSet<String>, BTreeSet<String>)>::new();
    let mut query_fields = BTreeMap::<String, BTreeSet<String>>::new();
    let mut dashboard_uids = BTreeSet::new();
    let mut panel_keys = BTreeSet::new();

    for row in query_report_rows {
        let Some(reference) = build_query_reference_payload(row) else {
            continue;
        };
        let key = query_signature_key(&reference);
        let hint = build_query_features(row, &reference);
        let QueryFeatureHints {
            mut metrics,
            mut functions,
            mut measurements,
            mut buckets,
            labels,
        } = hint;
        dashboard_uids.insert(reference.dashboard_uid.clone());
        panel_keys.insert(format!(
            "{}:{}",
            reference.dashboard_uid, reference.panel_id
        ));
        query_features.insert(
            key,
            QueryFeatureSet {
                metrics: metrics.clone(),
                functions: functions.clone(),
                measurements: measurements.clone(),
                buckets: buckets.clone(),
                labels,
            },
        );
        let dashboard_entry = dashboard_dependencies
            .entry(reference.dashboard_uid.clone())
            .or_insert(DashboardDependencyAccumulator {
                dashboard_title: reference.dashboard_title.clone(),
                query_count: 0,
                panel_keys: BTreeSet::new(),
                datasource_identities: BTreeSet::new(),
                datasource_families: BTreeSet::new(),
                query_fields: BTreeSet::new(),
                metrics: BTreeSet::new(),
                functions: BTreeSet::new(),
                measurements: BTreeSet::new(),
                buckets: BTreeSet::new(),
            });
        dashboard_entry.query_count += 1;
        dashboard_entry.panel_keys.insert(format!(
            "{}:{}",
            reference.dashboard_uid, reference.panel_id
        ));
        dashboard_entry
            .datasource_identities
            .insert(reference.datasource_name.clone());
        dashboard_entry
            .datasource_families
            .insert(reference.datasource_family.clone());
        dashboard_entry
            .query_fields
            .insert(reference.query_field.clone());
        dashboard_entry.metrics.extend(metrics.drain(..));
        dashboard_entry.functions.extend(functions.drain(..));
        dashboard_entry.measurements.extend(measurements.drain(..));
        dashboard_entry.buckets.extend(buckets.drain(..));
        let fields = query_fields
            .entry(reference.datasource_name.clone())
            .or_default();
        fields.insert(reference.query_field.clone());

        let summary_entry = usage.entry(reference.datasource_name.clone()).or_insert((
            DependencyUsageSummary {
                datasource_identity: reference.datasource_name.clone(),
                family: reference.datasource_family.clone(),
                query_count: 0,
                dashboard_count: 0,
                panel_count: 0,
                reference_count: 0,
                query_fields: Vec::new(),
            },
            BTreeSet::new(),
            BTreeSet::new(),
        ));
        summary_entry.0.query_count += 1;
        summary_entry.0.reference_count += 1;
        summary_entry.0.query_fields = fields.iter().cloned().collect();
        summary_entry.1.insert(reference.dashboard_uid.clone());
        summary_entry.2.insert(format!(
            "{}:{}",
            reference.dashboard_uid, reference.panel_id
        ));
        summary_entry.0.dashboard_count = summary_entry.1.len();
        summary_entry.0.panel_count = summary_entry.2.len();
        queries.push(reference);
    }

    let mut used = BTreeSet::new();
    for key in usage.keys() {
        used.insert(key.clone());
    }

    let mut orphaned = Vec::new();
    for item in datasource_inventory {
        let uid = item.uid.trim().to_string();
        let name = item.name.trim().to_string();
        if !uid.is_empty() && used.contains(&uid) {
            continue;
        }
        if !name.is_empty() && used.contains(&name) {
            continue;
        }
        if !uid.is_empty() {
            orphaned.push(uid);
            continue;
        }
        if !name.is_empty() {
            orphaned.push(name);
        }
    }

    let dashboard_dependencies = dashboard_dependencies
        .into_iter()
        .map(|(dashboard_uid, summary)| DashboardDependencySummary {
            dashboard_uid,
            dashboard_title: summary.dashboard_title,
            query_count: summary.query_count,
            panel_count: summary.panel_keys.len(),
            datasource_count: summary.datasource_identities.len(),
            datasource_family_count: summary.datasource_families.len(),
            query_fields: summary.query_fields.into_iter().collect(),
            metrics: summary.metrics.into_iter().collect(),
            functions: summary.functions.into_iter().collect(),
            measurements: summary.measurements.into_iter().collect(),
            buckets: summary.buckets.into_iter().collect(),
        })
        .collect::<Vec<_>>();

    let mut usage_rows = usage
        .into_values()
        .map(|(summary, _, _)| summary)
        .collect::<Vec<_>>();
    usage_rows.sort_by(|left, right| left.datasource_identity.cmp(&right.datasource_identity));

    let mut summary = BTreeMap::new();
    summary.insert("queryCount".to_string(), Value::from(queries.len()));
    summary.insert(
        "dashboardCount".to_string(),
        Value::from(dashboard_uids.len() as u64),
    );
    summary.insert(
        "panelCount".to_string(),
        Value::from(panel_keys.len() as u64),
    );
    summary.insert(
        "datasourceCount".to_string(),
        Value::from(usage_rows.len() as u64),
    );
    summary.insert(
        "orphanedDatasourceCount".to_string(),
        Value::from(orphaned.len() as u64),
    );

    OfflineDependencyReportDocument {
        summary,
        queries,
        query_features,
        dashboard_dependencies,
        usage: usage_rows,
        orphaned,
    }
    .as_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard::DatasourceInventoryItem;
    use serde_json::json;

    #[test]
    fn build_offline_dependency_contract_reports_unique_dashboard_and_panel_counts() {
        let document = build_offline_dependency_contract(
            &[
                json!({
                    "dashboardUid": "dash-a",
                    "dashboardTitle": "Dash A",
                    "panelId": "1",
                    "panelTitle": "Panel One",
                    "panelType": "timeseries",
                    "refId": "A",
                    "datasource": "Prometheus Main",
                    "datasourceUid": "prom-main",
                    "datasourceType": "prometheus",
                    "file": "dash-a.json",
                    "queryField": "expr",
                    "query": "up"
                }),
                json!({
                    "dashboardUid": "dash-a",
                    "dashboardTitle": "Dash A",
                    "panelId": "1",
                    "panelTitle": "Panel One",
                    "panelType": "timeseries",
                    "refId": "B",
                    "datasource": "Prometheus Main",
                    "datasourceUid": "prom-main",
                    "datasourceType": "prometheus",
                    "file": "dash-a.json",
                    "queryField": "expr",
                    "query": "sum(rate(up[5m]))"
                }),
                json!({
                    "dashboardUid": "dash-b",
                    "dashboardTitle": "Dash B",
                    "panelId": "2",
                    "panelTitle": "Panel Two",
                    "panelType": "timeseries",
                    "refId": "A",
                    "datasource": "Prometheus Main",
                    "datasourceUid": "prom-main",
                    "datasourceType": "prometheus",
                    "file": "dash-b.json",
                    "queryField": "expr",
                    "query": "rate(http_requests_total[5m])"
                }),
            ],
            &[DatasourceInventoryItem {
                uid: "prom-main".to_string(),
                name: "Prometheus Main".to_string(),
                datasource_type: "prometheus".to_string(),
                access: String::new(),
                url: String::new(),
                database: String::new(),
                default_bucket: String::new(),
                organization: String::new(),
                index_pattern: String::new(),
                is_default: String::new(),
                org: String::new(),
                org_id: String::new(),
            }],
        );

        assert_eq!(document["summary"]["queryCount"], json!(3));
        assert_eq!(document["summary"]["dashboardCount"], json!(2));
        assert_eq!(document["summary"]["panelCount"], json!(2));
        assert_eq!(document["summary"]["datasourceCount"], json!(1));
        assert_eq!(document["datasourceUsage"][0]["queryCount"], json!(3));
        assert_eq!(document["datasourceUsage"][0]["dashboardCount"], json!(2));
        assert_eq!(document["datasourceUsage"][0]["panelCount"], json!(2));
        assert_eq!(
            document["datasourceUsage"][0]["queryFields"],
            json!(["expr"])
        );
    }

    #[test]
    fn build_offline_dependency_contract_parses_loki_family_features() {
        let document = build_offline_dependency_contract(
            &[json!({
                "dashboardUid": "dash-loki",
                "dashboardTitle": "Loki Dash",
                "panelId": "7",
                "panelTitle": "Logs",
                "panelType": "logs",
                "refId": "A",
                "datasource": "Loki Main",
                "datasourceUid": "loki-main",
                "datasourceType": "loki",
                "file": "dash-loki.json",
                "queryField": "expr",
                "query": "{job=\"api\",namespace=~\"prod-.*\"} |= \"a|~b\" | line_format \"{{.message}}\" |~ \"timeout\" | json"
            })],
            &[DatasourceInventoryItem {
                uid: "loki-main".to_string(),
                name: "Loki Main".to_string(),
                datasource_type: "loki".to_string(),
                access: String::new(),
                url: String::new(),
                database: String::new(),
                default_bucket: String::new(),
                organization: String::new(),
                index_pattern: String::new(),
                is_default: String::new(),
                org: String::new(),
                org_id: String::new(),
            }],
        );

        let analysis = &document["queries"][0]["analysis"];
        let mut measurements = analysis["measurements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        measurements.sort();
        let mut functions = analysis["functions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        functions.sort();
        let mut labels = analysis["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        labels.sort();

        assert_eq!(
            measurements,
            vec![
                "job=\"api\"",
                "namespace=~\"prod-.*\"",
                "{job=\"api\",namespace=~\"prod-.*\"}"
            ]
        );
        assert_eq!(
            functions,
            vec![
                "json",
                "line_filter_contains",
                "line_filter_contains:a|~b",
                "line_filter_regex",
                "line_filter_regex:timeout",
                "line_format"
            ]
        );
        assert_eq!(labels, vec!["job=\"api\"", "namespace=~\"prod-.*\""]);
    }

    #[test]
    fn parse_sql_features_extracts_shape_and_source() {
        let document = build_offline_dependency_contract(
            &[json!({
                "dashboardUid": "dash-sql",
                "dashboardTitle": "SQL Dash",
                "panelId": "3",
                "panelTitle": "SQL",
                "panelType": "table",
                "refId": "A",
                "datasource": "PG Main",
                "datasourceUid": "pg-main",
                "datasourceType": "postgres",
                "file": "dash-sql.json",
                "queryField": "rawQuery",
                "query": "WITH recent AS (SELECT id FROM users WHERE active = true) SELECT p.id, u.name FROM posts p JOIN users u ON p.user_id = u.id WHERE p.created_at > now() - INTERVAL '1 day' LIMIT 10"
            })],
            &[DatasourceInventoryItem {
                uid: "pg-main".to_string(),
                name: "PG Main".to_string(),
                datasource_type: "postgres".to_string(),
                access: String::new(),
                url: String::new(),
                database: String::new(),
                default_bucket: String::new(),
                organization: String::new(),
                index_pattern: String::new(),
                is_default: String::new(),
                org: String::new(),
                org_id: String::new(),
            }],
        );

        let analysis = &document["queries"][0]["analysis"];
        let mut functions = analysis["functions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        functions.sort();
        let mut measurements = analysis["measurements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        measurements.sort();

        assert!(functions.binary_search(&"with".to_string()).is_ok());
        assert!(functions.binary_search(&"select".to_string()).is_ok());
        assert!(functions.binary_search(&"join".to_string()).is_ok());
        assert!(functions.binary_search(&"limit".to_string()).is_ok());
        assert!(measurements.contains(&"posts".to_string()));
        assert!(measurements.contains(&"users".to_string()));
        assert!(!measurements.contains(&"recent".to_string()));
    }
}
