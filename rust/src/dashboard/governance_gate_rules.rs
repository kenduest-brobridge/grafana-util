use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{message, Result};

use super::governance_gate::DashboardGovernanceGateFinding;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct QueryThresholdPolicy {
    allowed_families: BTreeSet<String>,
    allowed_uids: BTreeSet<String>,
    allowed_folder_prefixes: Vec<String>,
    forbid_unknown: bool,
    forbid_mixed_families: bool,
    forbid_select_star: bool,
    require_sql_time_filter: bool,
    forbid_broad_loki_regex: bool,
    forbid_broad_prometheus_selectors: bool,
    forbid_regex_heavy_prometheus: bool,
    forbid_high_cardinality_regex: bool,
    max_prometheus_range_window_seconds: Option<usize>,
    max_prometheus_aggregation_depth: Option<usize>,
    max_prometheus_cost_score: Option<usize>,
    forbid_unscoped_loki_search: bool,
    max_panels_per_dashboard: Option<usize>,
    min_refresh_interval_seconds: Option<usize>,
    max_audit_score: Option<usize>,
    max_reason_count: Option<usize>,
    block_reasons: BTreeSet<String>,
    max_dashboard_load_score: Option<usize>,
    max_query_complexity_score: Option<usize>,
    max_dashboard_complexity_score: Option<usize>,
    max_queries_per_dashboard: Option<usize>,
    max_queries_per_panel: Option<usize>,
    pub(crate) fail_on_warnings: bool,
}

fn value_to_usize(value: Option<&Value>) -> Result<Option<usize>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number
            .as_u64()
            .map(|value| Some(value as usize))
            .ok_or_else(|| message("Expected a non-negative integer in governance policy.")),
        Some(other) => Err(message(format!(
            "Expected a non-negative integer in governance policy, got {other}."
        ))),
    }
}

fn value_to_bool(value: Option<&Value>, default: bool) -> Result<bool> {
    match value {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(other) => Err(message(format!(
            "Expected a boolean in governance policy, got {other}."
        ))),
    }
}

fn value_to_string_set(value: Option<&Value>) -> Result<BTreeSet<String>> {
    match value {
        None | Some(Value::Null) => Ok(BTreeSet::new()),
        Some(Value::Array(values)) => Ok(values
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect()),
        Some(other) => Err(message(format!(
            "Expected an array of strings in governance policy, got {other}."
        ))),
    }
}

pub(crate) fn parse_query_threshold_policy(policy: &Value) -> Result<QueryThresholdPolicy> {
    let Some(policy_object) = policy.as_object() else {
        return Err(message("Governance policy JSON must be an object."));
    };
    if let Some(version) = policy_object.get("version") {
        match version {
            Value::Number(number) if number.as_i64() == Some(1) => {}
            _ => {
                return Err(message(
                    "Governance policy version is not supported. Expected version 1.",
                ))
            }
        }
    }
    let queries = policy_object
        .get("queries")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let datasources = policy_object
        .get("datasources")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let enforcement = policy_object
        .get("enforcement")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let routing = policy_object
        .get("routing")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let dashboards = policy_object
        .get("dashboards")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    Ok(QueryThresholdPolicy {
        allowed_families: value_to_string_set(datasources.get("allowedFamilies"))?,
        allowed_uids: value_to_string_set(datasources.get("allowedUids"))?,
        allowed_folder_prefixes: value_to_string_set(routing.get("allowedFolderPrefixes"))?
            .into_iter()
            .collect(),
        forbid_unknown: value_to_bool(datasources.get("forbidUnknown"), false)?,
        forbid_mixed_families: value_to_bool(datasources.get("forbidMixedFamilies"), false)?,
        forbid_select_star: value_to_bool(queries.get("forbidSelectStar"), false)?,
        require_sql_time_filter: value_to_bool(queries.get("requireSqlTimeFilter"), false)?,
        forbid_broad_loki_regex: value_to_bool(queries.get("forbidBroadLokiRegex"), false)?,
        forbid_broad_prometheus_selectors: value_to_bool(
            queries.get("forbidBroadPrometheusSelectors"),
            false,
        )?,
        forbid_regex_heavy_prometheus: value_to_bool(
            queries.get("forbidRegexHeavyPrometheus"),
            false,
        )?,
        forbid_high_cardinality_regex: value_to_bool(
            queries.get("forbidHighCardinalityRegex"),
            false,
        )?,
        max_prometheus_range_window_seconds: value_to_usize(
            queries.get("maxPrometheusRangeWindowSeconds"),
        )?,
        max_prometheus_aggregation_depth: value_to_usize(
            queries.get("maxPrometheusAggregationDepth"),
        )?,
        max_prometheus_cost_score: value_to_usize(queries.get("maxPrometheusCostScore"))?,
        forbid_unscoped_loki_search: value_to_bool(queries.get("forbidUnscopedLokiSearch"), false)?,
        max_panels_per_dashboard: value_to_usize(dashboards.get("maxPanelsPerDashboard"))?,
        min_refresh_interval_seconds: value_to_usize(dashboards.get("minRefreshIntervalSeconds"))?,
        max_audit_score: value_to_usize(queries.get("maxAuditScore"))?,
        max_reason_count: value_to_usize(queries.get("maxReasonCount"))?,
        block_reasons: value_to_string_set(queries.get("blockReasons"))?,
        max_dashboard_load_score: value_to_usize(dashboards.get("maxDashboardLoadScore"))?,
        max_query_complexity_score: value_to_usize(queries.get("maxQueryComplexityScore"))?,
        max_dashboard_complexity_score: value_to_usize(queries.get("maxDashboardComplexityScore"))?,
        max_queries_per_dashboard: value_to_usize(queries.get("maxQueriesPerDashboard"))?,
        max_queries_per_panel: value_to_usize(queries.get("maxQueriesPerPanel"))?,
        fail_on_warnings: value_to_bool(enforcement.get("failOnWarnings"), false)?,
    })
}

pub(crate) fn build_checked_rules(policy: &QueryThresholdPolicy) -> Value {
    serde_json::json!({
        "datasourceAllowedFamilies": policy.allowed_families,
        "datasourceAllowedUids": policy.allowed_uids,
        "allowedFolderPrefixes": policy.allowed_folder_prefixes,
        "forbidUnknown": policy.forbid_unknown,
        "forbidMixedFamilies": policy.forbid_mixed_families,
        "forbidSelectStar": policy.forbid_select_star,
        "requireSqlTimeFilter": policy.require_sql_time_filter,
        "forbidBroadLokiRegex": policy.forbid_broad_loki_regex,
        "forbidBroadPrometheusSelectors": policy.forbid_broad_prometheus_selectors,
        "forbidRegexHeavyPrometheus": policy.forbid_regex_heavy_prometheus,
        "forbidHighCardinalityRegex": policy.forbid_high_cardinality_regex,
        "maxPrometheusRangeWindowSeconds": policy.max_prometheus_range_window_seconds,
        "maxPrometheusAggregationDepth": policy.max_prometheus_aggregation_depth,
        "maxPrometheusCostScore": policy.max_prometheus_cost_score,
        "forbidUnscopedLokiSearch": policy.forbid_unscoped_loki_search,
        "maxPanelsPerDashboard": policy.max_panels_per_dashboard,
        "minRefreshIntervalSeconds": policy.min_refresh_interval_seconds,
        "maxAuditScore": policy.max_audit_score,
        "maxReasonCount": policy.max_reason_count,
        "blockReasons": policy.block_reasons,
        "maxDashboardLoadScore": policy.max_dashboard_load_score,
        "maxQueryComplexityScore": policy.max_query_complexity_score,
        "maxDashboardComplexityScore": policy.max_dashboard_complexity_score,
        "maxQueriesPerDashboard": policy.max_queries_per_dashboard,
        "maxQueriesPerPanel": policy.max_queries_per_panel,
        "failOnWarnings": policy.fail_on_warnings,
    })
}

fn string_field(record: &Value, key: &str) -> String {
    record
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .to_string()
}

fn is_sql_family(family: &str) -> bool {
    matches!(
        family.trim().to_ascii_lowercase().as_str(),
        "mysql" | "postgres" | "mssql" | "sql"
    )
}

fn query_uses_time_filter(query_text: &str) -> bool {
    let lowered = query_text.trim().to_ascii_lowercase();
    lowered.contains("$__timefilter(")
        || lowered.contains("$__unixepochfilter(")
        || lowered.contains("$timefilter")
}

fn parse_duration_seconds(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("off") {
        return None;
    }
    let mut digits = String::new();
    let mut suffix = String::new();
    for character in trimmed.chars() {
        if character.is_ascii_digit() && suffix.is_empty() {
            digits.push(character);
        } else if !character.is_whitespace() {
            suffix.push(character);
        }
    }
    let number = digits.parse::<usize>().ok()?;
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

fn prometheus_query_is_broad(query: &Value) -> bool {
    let query_text = string_field(query, "query");
    let family = string_field(query, "datasourceFamily");
    if !family.eq_ignore_ascii_case("prometheus")
        || query_text.is_empty()
        || query_text.contains('{')
        || query_text.contains(' ')
        || query_text.contains('(')
        || query_text.contains('[')
    {
        return false;
    }
    let metrics = query
        .get("metrics")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<&str>>()
        })
        .unwrap_or_default();
    metrics.len() == 1 && metrics[0] == query_text
}

fn query_uses_regex_matchers(query_text: &str) -> bool {
    query_text.contains("=~") || query_text.contains("!~")
}

fn query_uses_unscoped_loki_search(query: &Value) -> bool {
    if !string_field(query, "datasourceFamily").eq_ignore_ascii_case("loki") {
        return false;
    }
    let functions = query
        .get("functions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<&str>>()
        })
        .unwrap_or_default();
    let has_line_filter = functions
        .iter()
        .any(|value| value.starts_with("line_filter_"));
    if !has_line_filter {
        return false;
    }
    let measurements = query
        .get("measurements")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<&str>>()
        })
        .unwrap_or_default();
    !measurements.is_empty()
        && measurements
            .iter()
            .all(|value| *value == "{}" || !value.contains('=') || value.contains(".*"))
}

fn query_dashboard_refresh_seconds(query: &Value) -> Option<usize> {
    for key in [
        "dashboardRefreshSeconds",
        "refreshIntervalSeconds",
        "refreshSeconds",
    ] {
        if let Some(seconds) = query.get(key).and_then(Value::as_u64) {
            return Some(seconds as usize);
        }
    }
    match query.get("refresh") {
        Some(Value::Number(number)) => number.as_u64().map(|value| value as usize),
        Some(Value::String(value)) => parse_duration_seconds(value),
        _ => None,
    }
}

fn loki_query_is_broad(query_text: &str) -> bool {
    let lowered = query_text.trim().to_ascii_lowercase();
    lowered.contains("=~\".*\"")
        || lowered.contains("=~\".+\"")
        || lowered.contains("|~\".*\"")
        || lowered.contains("|~\".+\"")
        || lowered.contains("{}")
}

fn value_array_len(record: &Value, key: &str) -> usize {
    record
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.len())
        .unwrap_or(0)
}

fn score_query_complexity(query: &Value) -> usize {
    let query_text = string_field(query, "query");
    if query_text.is_empty() {
        return 0;
    }
    let token_pattern = Regex::new(
        r"\b(sum|avg|min|max|count|rate|increase|histogram_quantile|label_replace|topk|bottomk|join|union|group by|order by)\b",
    )
    .unwrap();
    let lowered = query_text.to_ascii_lowercase();
    let mut score = 1usize;
    if query_text.len() > 80 {
        score += 1;
    }
    if query_text.len() > 160 {
        score += 1;
    }
    score += std::cmp::min(3, token_pattern.find_iter(&query_text).count());
    score += std::cmp::min(2, lowered.matches('|').count());
    if query_text.contains("=~") || query_text.contains("!~") {
        score += 1;
    }
    if query_text.contains('(') && query_text.contains(')') {
        score += std::cmp::min(2, query_text.matches('(').count() / 2);
    }
    score += std::cmp::min(2, value_array_len(query, "metrics"));
    score += std::cmp::min(1, value_array_len(query, "measurements"));
    score += std::cmp::min(1, value_array_len(query, "buckets"));
    score
}

fn build_query_violation(
    code: &str,
    message_text: String,
    query: &Value,
) -> DashboardGovernanceGateFinding {
    DashboardGovernanceGateFinding {
        severity: "error".to_string(),
        code: code.to_string(),
        message: message_text,
        dashboard_uid: string_field(query, "dashboardUid"),
        dashboard_title: string_field(query, "dashboardTitle"),
        panel_id: string_field(query, "panelId"),
        panel_title: string_field(query, "panelTitle"),
        ref_id: string_field(query, "refId"),
        datasource: string_field(query, "datasource"),
        datasource_uid: string_field(query, "datasourceUid"),
        datasource_family: string_field(query, "datasourceFamily"),
        risk_kind: String::new(),
    }
}

fn build_dashboard_violation(
    code: &str,
    message_text: String,
    dashboard: &Value,
) -> DashboardGovernanceGateFinding {
    DashboardGovernanceGateFinding {
        severity: "error".to_string(),
        code: code.to_string(),
        message: message_text,
        dashboard_uid: string_field(dashboard, "dashboardUid"),
        dashboard_title: string_field(dashboard, "dashboardTitle"),
        panel_id: String::new(),
        panel_title: String::new(),
        ref_id: String::new(),
        datasource: String::new(),
        datasource_uid: String::new(),
        datasource_family: String::new(),
        risk_kind: String::new(),
    }
}

fn build_dashboard_violation_from_fields(
    code: &str,
    message_text: String,
    dashboard_uid: String,
    dashboard_title: String,
) -> DashboardGovernanceGateFinding {
    DashboardGovernanceGateFinding {
        severity: "error".to_string(),
        code: code.to_string(),
        message: message_text,
        dashboard_uid,
        dashboard_title,
        panel_id: String::new(),
        panel_title: String::new(),
        ref_id: String::new(),
        datasource: String::new(),
        datasource_uid: String::new(),
        datasource_family: String::new(),
        risk_kind: String::new(),
    }
}

fn array_of_objects<'a>(document: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    document.get(key).and_then(Value::as_array).ok_or_else(|| {
        message(format!(
            "Dashboard governance JSON must contain a {key} array."
        ))
    })
}

pub(crate) fn evaluate_dashboard_governance_gate_violations(
    policy: &QueryThresholdPolicy,
    governance_document: &Value,
    queries: &[Value],
) -> Result<Vec<DashboardGovernanceGateFinding>> {
    let mut dashboard_counts = BTreeMap::<String, (String, usize)>::new();
    let mut dashboard_refresh_seconds = BTreeMap::<String, (String, usize)>::new();
    let mut dashboard_complexity_scores = BTreeMap::<(String, String), usize>::new();
    let mut panel_counts = BTreeMap::<(String, String), (String, String, usize)>::new();
    let mut violations = Vec::new();
    for query in queries {
        let dashboard_uid = string_field(query, "dashboardUid");
        let dashboard_title = string_field(query, "dashboardTitle");
        let panel_id = string_field(query, "panelId");
        let panel_title = string_field(query, "panelTitle");
        let datasource = string_field(query, "datasource");
        let datasource_uid = string_field(query, "datasourceUid");
        let datasource_family = string_field(query, "datasourceFamily");
        let folder_path = string_field(query, "folderPath");
        let query_text = string_field(query, "query");
        let complexity_score = score_query_complexity(query);
        let dashboard_entry = dashboard_counts
            .entry(dashboard_uid.clone())
            .or_insert((dashboard_title.clone(), 0usize));
        dashboard_entry.1 += 1;
        *dashboard_complexity_scores
            .entry((dashboard_uid.clone(), dashboard_title.clone()))
            .or_insert(0usize) += complexity_score;
        if let Some(refresh_seconds) = query_dashboard_refresh_seconds(query) {
            let entry = dashboard_refresh_seconds
                .entry(dashboard_uid.clone())
                .or_insert((dashboard_title.clone(), refresh_seconds));
            if refresh_seconds != 0 {
                entry.1 = entry.1.min(refresh_seconds);
            }
        }
        let panel_entry = panel_counts.entry((dashboard_uid, panel_id)).or_insert((
            dashboard_title,
            panel_title,
            0usize,
        ));
        panel_entry.2 += 1;

        if policy.forbid_unknown
            && (datasource_family.is_empty()
                || datasource_family.eq_ignore_ascii_case("unknown")
                || datasource.is_empty())
        {
            violations.push(build_query_violation(
                "datasource-unknown",
                "Datasource identity could not be resolved for this query row.".to_string(),
                query,
            ));
        }
        if !policy.allowed_families.is_empty()
            && !policy.allowed_families.contains(&datasource_family)
        {
            let family = if datasource_family.is_empty() {
                "unknown".to_string()
            } else {
                datasource_family.clone()
            };
            violations.push(build_query_violation(
                "datasource-family-not-allowed",
                format!("Datasource family {family} is not allowed by policy."),
                query,
            ));
        }
        if !policy.allowed_uids.is_empty()
            && !datasource_uid.is_empty()
            && !policy.allowed_uids.contains(&datasource_uid)
        {
            violations.push(build_query_violation(
                "datasource-uid-not-allowed",
                format!("Datasource uid {datasource_uid} is not allowed by policy."),
                query,
            ));
        }
        if !policy.allowed_folder_prefixes.is_empty()
            && !policy.allowed_folder_prefixes.iter().any(|prefix| {
                folder_path == *prefix || folder_path.starts_with(&format!("{prefix} /"))
            })
        {
            violations.push(build_query_violation(
                "routing-folder-not-allowed",
                format!(
                    "Dashboard folderPath {} is not allowed by policy.",
                    if folder_path.is_empty() {
                        "unknown".to_string()
                    } else {
                        folder_path.clone()
                    }
                ),
                query,
            ));
        }
        if policy.forbid_select_star
            && is_sql_family(&datasource_family)
            && query_text.to_ascii_lowercase().contains("select *")
        {
            violations.push(build_query_violation(
                "sql-select-star",
                "SQL query uses SELECT * and violates the policy.".to_string(),
                query,
            ));
        }
        if policy.require_sql_time_filter
            && is_sql_family(&datasource_family)
            && !query_uses_time_filter(&query_text)
        {
            violations.push(build_query_violation(
                "sql-missing-time-filter",
                "SQL query does not include a Grafana time filter macro.".to_string(),
                query,
            ));
        }
        if policy.forbid_broad_loki_regex
            && datasource_family.eq_ignore_ascii_case("loki")
            && loki_query_is_broad(&query_text)
        {
            violations.push(build_query_violation(
                "loki-broad-regex",
                "Loki query contains a broad match or empty selector.".to_string(),
                query,
            ));
        }
        if policy.forbid_broad_prometheus_selectors && prometheus_query_is_broad(query) {
            violations.push(build_query_violation(
                "prometheus-broad-selector",
                "Prometheus query uses a broad selector without label filters.".to_string(),
                query,
            ));
        }
        if policy.forbid_regex_heavy_prometheus
            && datasource_family.eq_ignore_ascii_case("prometheus")
            && query_uses_regex_matchers(&query_text)
        {
            violations.push(build_query_violation(
                "prometheus-regex-heavy",
                "Prometheus query uses regex label matchers and violates the policy.".to_string(),
                query,
            ));
        }
        if let Some(limit) = policy.max_prometheus_range_window_seconds {
            if datasource_family.eq_ignore_ascii_case("prometheus") {
                let max_bucket = query
                    .get("buckets")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .filter_map(parse_duration_seconds)
                            .max()
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                if max_bucket > limit {
                    violations.push(build_query_violation(
                        "prometheus-range-window-too-large",
                        format!(
                            "Prometheus range window {max_bucket}s exceeds policy maxPrometheusRangeWindowSeconds={limit}."
                        ),
                        query,
                    ));
                }
            }
        }
        if policy.forbid_unscoped_loki_search && query_uses_unscoped_loki_search(query) {
            violations.push(build_query_violation(
                "loki-unscoped-search",
                "Loki query performs line filtering without concrete label scoping.".to_string(),
                query,
            ));
        }
        if let Some(limit) = policy.max_query_complexity_score {
            if complexity_score > limit {
                violations.push(build_query_violation(
                    "query-complexity-too-high",
                    format!(
                        "Query complexity score {complexity_score} exceeds policy maxQueryComplexityScore={limit}."
                    ),
                    query,
                ));
            }
        }
    }

    if let Some(limit) = policy.max_queries_per_dashboard {
        for (dashboard_uid, (dashboard_title, query_count)) in &dashboard_counts {
            if *query_count > limit {
                violations.push(DashboardGovernanceGateFinding {
                    severity: "error".to_string(),
                    code: "max-queries-per-dashboard".to_string(),
                    message: format!(
                        "Dashboard query count {query_count} exceeds policy maxQueriesPerDashboard={limit}."
                    ),
                    dashboard_uid: dashboard_uid.clone(),
                    dashboard_title: dashboard_title.clone(),
                    panel_id: String::new(),
                    panel_title: String::new(),
                    ref_id: String::new(),
                    datasource: String::new(),
                    datasource_uid: String::new(),
                    datasource_family: String::new(),
                    risk_kind: String::new(),
                });
            }
        }
    }
    if let Some(limit) = policy.max_queries_per_panel {
        for ((dashboard_uid, panel_id), (dashboard_title, panel_title, query_count)) in
            &panel_counts
        {
            if *query_count > limit {
                violations.push(DashboardGovernanceGateFinding {
                    severity: "error".to_string(),
                    code: "max-queries-per-panel".to_string(),
                    message: format!(
                        "Panel query count {query_count} exceeds policy maxQueriesPerPanel={limit}."
                    ),
                    dashboard_uid: dashboard_uid.clone(),
                    dashboard_title: dashboard_title.clone(),
                    panel_id: panel_id.clone(),
                    panel_title: panel_title.clone(),
                    ref_id: String::new(),
                    datasource: String::new(),
                    datasource_uid: String::new(),
                    datasource_family: String::new(),
                    risk_kind: String::new(),
                });
            }
        }
    }
    if let Some(limit) = policy.max_dashboard_complexity_score {
        for ((dashboard_uid, dashboard_title), complexity_score) in &dashboard_complexity_scores {
            if *complexity_score > limit {
                violations.push(DashboardGovernanceGateFinding {
                    severity: "error".to_string(),
                    code: "dashboard-complexity-too-high".to_string(),
                    message: format!(
                        "Dashboard complexity score {complexity_score} exceeds policy maxDashboardComplexityScore={limit}."
                    ),
                    dashboard_uid: dashboard_uid.clone(),
                    dashboard_title: dashboard_title.clone(),
                    panel_id: String::new(),
                    panel_title: String::new(),
                    ref_id: String::new(),
                    datasource: String::new(),
                    datasource_uid: String::new(),
                    datasource_family: String::new(),
                    risk_kind: String::new(),
                });
            }
        }
    }

    if policy.forbid_mixed_families {
        let dashboards = governance_document
            .get("dashboardGovernance")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                message("Dashboard governance JSON must contain a dashboardGovernance array.")
            })?;
        for dashboard in dashboards {
            let mixed = dashboard
                .get("mixedDatasource")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if mixed {
                let families = dashboard
                    .get("datasourceFamilies")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<&str>>()
                            .join(",")
                    })
                    .unwrap_or_default();
                violations.push(build_dashboard_violation(
                    "mixed-datasource-families-not-allowed",
                    format!(
                        "Dashboard uses mixed datasource families{}{}.",
                        if families.is_empty() { "" } else { ": " },
                        families
                    ),
                    dashboard,
                ));
            }
        }
    }
    if policy.max_panels_per_dashboard.is_some() || policy.min_refresh_interval_seconds.is_some() {
        let dashboards = array_of_objects(governance_document, "dashboardGovernance")?;
        if let Some(limit) = policy.max_panels_per_dashboard {
            for dashboard in dashboards {
                let panel_count = dashboard
                    .get("panelCount")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize;
                if panel_count > limit {
                    violations.push(build_dashboard_violation(
                        "max-panels-per-dashboard",
                        format!(
                            "Dashboard panel count {panel_count} exceeds policy maxPanelsPerDashboard={limit}."
                        ),
                        dashboard,
                    ));
                }
            }
        }
        if let Some(limit) = policy.min_refresh_interval_seconds {
            let mut refresh_by_dashboard = BTreeMap::<String, (String, usize)>::new();
            for query in queries {
                let Some(refresh_seconds) = query_dashboard_refresh_seconds(query) else {
                    continue;
                };
                let dashboard_uid = string_field(query, "dashboardUid");
                if dashboard_uid.is_empty() {
                    continue;
                }
                let dashboard_title = string_field(query, "dashboardTitle");
                let entry = refresh_by_dashboard
                    .entry(dashboard_uid)
                    .or_insert((dashboard_title, refresh_seconds));
                if refresh_seconds != 0 {
                    entry.1 = entry.1.min(refresh_seconds);
                }
            }
            for (dashboard_uid, (dashboard_title, refresh_seconds)) in refresh_by_dashboard {
                if refresh_seconds != 0 && refresh_seconds < limit {
                    violations.push(build_dashboard_violation_from_fields(
                        "min-refresh-interval-seconds",
                        format!(
                            "Dashboard refresh interval {refresh_seconds}s is below policy minRefreshIntervalSeconds={limit}."
                        ),
                        dashboard_uid,
                        dashboard_title,
                    ));
                }
            }
        }
    }

    if policy.max_audit_score.is_some()
        || policy.max_reason_count.is_some()
        || !policy.block_reasons.is_empty()
        || policy.max_dashboard_load_score.is_some()
        || policy.forbid_high_cardinality_regex
        || policy.max_prometheus_aggregation_depth.is_some()
        || policy.max_prometheus_cost_score.is_some()
    {
        let query_audits = array_of_objects(governance_document, "queryAudits")?;
        for audit in query_audits {
            let score = audit.get("score").and_then(Value::as_u64).unwrap_or(0) as usize;
            let query_cost_score = audit
                .get("queryCostScore")
                .and_then(Value::as_u64)
                .unwrap_or(score as u64) as usize;
            let aggregation_depth = audit
                .get("aggregationDepth")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            let reasons = audit
                .get("reasons")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            if let Some(limit) = policy.max_audit_score {
                if score > limit {
                    violations.push(build_query_violation(
                        "query-audit-score-too-high",
                        format!(
                            "Query audit score {score} exceeds policy maxAuditScore={limit}. reasons={}",
                            reasons.join(",")
                        ),
                        audit,
                    ));
                }
            }
            if let Some(limit) = policy.max_reason_count {
                if reasons.len() > limit {
                    violations.push(build_query_violation(
                        "query-audit-reason-count-too-high",
                        format!(
                            "Query audit reason count {} exceeds policy maxReasonCount={limit}. reasons={}",
                            reasons.len(),
                            reasons.join(",")
                        ),
                        audit,
                    ));
                }
            }
            if !policy.block_reasons.is_empty()
                && reasons
                    .iter()
                    .any(|reason| policy.block_reasons.contains(reason))
            {
                violations.push(build_query_violation(
                    "query-audit-blocked-reason",
                    format!(
                        "Query audit contains blocked reasons: {}",
                        reasons.join(",")
                    ),
                    audit,
                ));
            }
            if policy.forbid_high_cardinality_regex
                && reasons
                    .iter()
                    .any(|reason| reason == "prometheus-high-cardinality-regex")
            {
                violations.push(build_query_violation(
                    "prometheus-high-cardinality-regex",
                    "Prometheus query uses regex matchers on likely high-cardinality labels."
                        .to_string(),
                    audit,
                ));
            }
            if let Some(limit) = policy.max_prometheus_aggregation_depth {
                if aggregation_depth > limit {
                    violations.push(build_query_violation(
                        "prometheus-aggregation-depth-too-high",
                        format!(
                            "Prometheus aggregation depth {aggregation_depth} exceeds policy maxPrometheusAggregationDepth={limit}."
                        ),
                        audit,
                    ));
                }
            }
            if let Some(limit) = policy.max_prometheus_cost_score {
                if query_cost_score > limit {
                    violations.push(build_query_violation(
                        "prometheus-cost-score-too-high",
                        format!(
                            "Prometheus query cost score {query_cost_score} exceeds policy maxPrometheusCostScore={limit}."
                        ),
                        audit,
                    ));
                }
            }
        }
        let dashboard_audits = array_of_objects(governance_document, "dashboardAudits")?;
        if let Some(limit) = policy.max_dashboard_load_score {
            for audit in dashboard_audits {
                let score = audit.get("score").and_then(Value::as_u64).unwrap_or(0) as usize;
                if score > limit {
                    violations.push(build_dashboard_violation(
                        "dashboard-load-score-too-high",
                        format!(
                            "Dashboard load score {score} exceeds policy maxDashboardLoadScore={limit}."
                        ),
                        audit,
                    ));
                }
            }
        }
    }

    Ok(violations)
}

pub(crate) fn build_governance_warning_findings(
    governance_document: &Value,
) -> Result<Vec<DashboardGovernanceGateFinding>> {
    let risk_records = governance_document
        .get("riskRecords")
        .and_then(Value::as_array)
        .ok_or_else(|| message("Dashboard governance JSON must contain a riskRecords array."))?;
    Ok(risk_records
        .iter()
        .map(|record| DashboardGovernanceGateFinding {
            severity: "warning".to_string(),
            code: string_field(record, "kind"),
            message: record
                .get("recommendation")
                .and_then(Value::as_str)
                .map(str::to_string)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| {
                    let detail = string_field(record, "detail");
                    if detail.is_empty() {
                        "Governance warning surfaced from inspect report.".to_string()
                    } else {
                        detail
                    }
                }),
            dashboard_uid: string_field(record, "dashboardUid"),
            dashboard_title: String::new(),
            panel_id: string_field(record, "panelId"),
            panel_title: String::new(),
            ref_id: String::new(),
            datasource: string_field(record, "datasource"),
            datasource_uid: String::new(),
            datasource_family: String::new(),
            risk_kind: string_field(record, "kind"),
        })
        .collect::<Vec<DashboardGovernanceGateFinding>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_policy() -> QueryThresholdPolicy {
        QueryThresholdPolicy {
            max_queries_per_dashboard: Some(1),
            max_queries_per_panel: Some(1),
            min_refresh_interval_seconds: Some(30),
            forbid_mixed_families: true,
            ..QueryThresholdPolicy::default()
        }
    }

    #[test]
    fn evaluate_dashboard_governance_gate_violations_keeps_aggregate_rules_intact() {
        let policy = sample_policy();
        let governance_document = json!({
            "dashboardGovernance": [
                {
                    "dashboardUid": "cpu-main",
                    "dashboardTitle": "CPU Main",
                    "panelCount": 2,
                    "mixedDatasource": true,
                    "datasourceFamilies": ["prometheus", "loki"]
                }
            ],
            "queryAudits": [],
            "dashboardAudits": [],
            "riskRecords": []
        });
        let queries = vec![
            json!({
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelId": "7",
                "panelTitle": "CPU",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "query": "up",
                "metrics": ["up"],
                "functions": [],
                "measurements": [],
                "buckets": [],
                "refresh": "5s"
            }),
            json!({
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "panelId": "7",
                "panelTitle": "CPU",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceFamily": "prometheus",
                "query": "sum(rate(http_requests_total[5m]))",
                "metrics": ["http_requests_total"],
                "functions": ["sum", "rate"],
                "measurements": [],
                "buckets": ["5m"],
                "refresh": "5s"
            }),
        ];

        let violations =
            evaluate_dashboard_governance_gate_violations(&policy, &governance_document, &queries)
                .unwrap();
        let codes = violations
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<&str>>();

        assert!(codes.contains(&"max-queries-per-dashboard"));
        assert!(codes.contains(&"max-queries-per-panel"));
        assert!(codes.contains(&"min-refresh-interval-seconds"));
        assert!(codes.contains(&"mixed-datasource-families-not-allowed"));
    }

    #[test]
    fn build_governance_warning_findings_uses_recommendation_then_detail_fallback() {
        let governance_document = json!({
            "riskRecords": [
                {
                    "kind": "dashboard-refresh-pressure",
                    "dashboardUid": "cpu-main",
                    "panelId": "7",
                    "datasource": "Prometheus Main",
                    "detail": "5s",
                    "recommendation": "Increase the refresh interval."
                },
                {
                    "kind": "custom-risk",
                    "dashboardUid": "cpu-main",
                    "panelId": "8",
                    "datasource": "Logs Main",
                    "detail": "Review this dashboard"
                }
            ]
        });

        let warnings = build_governance_warning_findings(&governance_document).unwrap();

        assert_eq!(warnings[0].message, "Increase the refresh interval.");
        assert_eq!(warnings[0].risk_kind, "dashboard-refresh-pressure");
        assert_eq!(warnings[1].message, "Review this dashboard");
        assert_eq!(warnings[1].risk_kind, "custom-risk");
    }
}
