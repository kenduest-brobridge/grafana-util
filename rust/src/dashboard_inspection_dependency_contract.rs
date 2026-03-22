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
    let query = row.query.to_lowercase();
    let mut metrics = Vec::new();
    let mut measurements = Vec::new();
    let mut buckets = Vec::new();
    let mut labels = Vec::new();
    let mut functions = Vec::new();

    if ["prometheus", "graphite", "victoriametrics"].contains(&family.as_str()) {
        for capture in row.query.split(&[' ', '\n', '\t', ';'][..]) {
            let value = capture.trim();
            if value.is_empty() {
                continue;
            }
            if value.ends_with(")") && value.chars().all(|c| !c.is_ascii_digit()) {
                continue;
            }
            if value
                .chars()
                .next()
                .map(|value| value.is_ascii_alphabetic())
                .unwrap_or(false)
            {
                metrics.push(value.to_string());
            }
        }
        if query.contains("rate(") {
            metrics.push("rate".to_string());
            functions.push("rate".to_string());
        }
        if query.contains("sum(") {
            metrics.push("sum".to_string());
            functions.push("sum".to_string());
        }
    }

    if family == "loki" {
        let mut selectors = BTreeSet::new();
        for segment in query.split('|') {
            if let Some(begin) = segment.find('{') {
                if let Some(end) = segment.find('}') {
                    selectors.insert(segment[begin + 1..end].to_string());
                }
            }
        }
        labels = selectors.into_iter().collect();
        if query.contains("|") {
            buckets.push("pipeline".to_string());
        }
        if query.contains("|=") {
            buckets.push("line_filter_contains".to_string());
        }
        if query.contains("|~") {
            buckets.push("line_filter_regex".to_string());
        }
    }

    if family == "flux" || family == "influxdb" {
        if query.contains("from(") {
            measurements.push("from".to_string());
        }
        if query.contains("window(") || query.contains("range(") {
            buckets.push("window".to_string());
        }
    }

    if ["mysql", "postgresql", "postgres", "sql"].contains(&family.as_str()) {
        for keyword in ["from", "join", "where", "group by"] {
            if query.contains(keyword) {
                measurements.push(keyword.to_string());
            }
        }
    }

    for token in row.query.to_lowercase().split(|c: char| {
        c.is_ascii_whitespace()
            || c == '.'
            || c == ','
            || c == ';'
            || c == '{'
            || c == '}'
            || c == '['
            || c == ']'
            || c == '|'
            || c == '+'
            || c == '*'
            || c == '/'
            || c == '-'
    }) {
        if let Some(index) = token.find('(') {
            let candidate = &token[..index];
            if candidate
                .chars()
                .next()
                .map(|value| value.is_ascii_alphabetic())
                .unwrap_or(false)
            {
                functions.push(candidate.to_string());
            }
        }
    }

    QueryFeatureHints {
        functions: dedupe_strings(&functions),
        metrics: dedupe_strings(&metrics),
        measurements: dedupe_strings(&measurements),
        buckets: dedupe_strings(&buckets),
        labels: dedupe_strings(&labels),
    }
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
}
