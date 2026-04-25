use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

use crate::common::{string_field, value_as_object, Result};
use crate::dashboard::extract_dashboard_object;

const CORE_PANEL_TYPES: &[&str] = &[
    "alertlist",
    "bargauge",
    "candlestick",
    "dashlist",
    "gauge",
    "geomap",
    "graph",
    "heatmap",
    "histogram",
    "logs",
    "news",
    "nodeGraph",
    "piechart",
    "row",
    "state-timeline",
    "stat",
    "status-history",
    "table",
    "text",
    "timeseries",
    "traces",
    "trend",
    "xychart",
];

const CORE_DATASOURCE_TYPES: &[&str] = &[
    "alertmanager",
    "cloud-monitoring",
    "elasticsearch",
    "grafana",
    "graphite",
    "influxdb",
    "jaeger",
    "loki",
    "mysql",
    "opensearch",
    "postgres",
    "prometheus",
    "tempo",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardValidationIssue {
    pub(crate) severity: &'static str,
    pub(crate) code: &'static str,
    pub(crate) file: String,
    pub(crate) dashboard_uid: String,
    pub(crate) dashboard_title: String,
    pub(crate) message: String,
}

fn push_issue(
    issues: &mut Vec<DashboardValidationIssue>,
    severity: &'static str,
    code: &'static str,
    file: &Path,
    dashboard_uid: &str,
    dashboard_title: &str,
    message_text: impl Into<String>,
) {
    issues.push(DashboardValidationIssue {
        severity,
        code,
        file: file.display().to_string(),
        dashboard_uid: dashboard_uid.to_string(),
        dashboard_title: dashboard_title.to_string(),
        message: message_text.into(),
    });
}

fn walk_panels<'a>(panels: &'a [Value], collected: &mut Vec<&'a Map<String, Value>>) {
    for panel in panels {
        let Some(panel_object) = panel.as_object() else {
            continue;
        };
        collected.push(panel_object);
        if let Some(children) = panel_object.get("panels").and_then(Value::as_array) {
            walk_panels(children, collected);
        }
    }
}

fn collect_dashboard_panels(dashboard: &Map<String, Value>) -> Vec<&Map<String, Value>> {
    let mut panels = Vec::new();
    if let Some(items) = dashboard.get("panels").and_then(Value::as_array) {
        walk_panels(items, &mut panels);
    }
    panels
}

fn collect_dashboard_datasource_types(
    dashboard: &Map<String, Value>,
    panels: &[&Map<String, Value>],
) -> BTreeSet<String> {
    let mut types = BTreeSet::new();
    let collect_from_value = |value: Option<&Value>, collected: &mut BTreeSet<String>| {
        let Some(value) = value else {
            return;
        };
        match value {
            Value::Object(object) => {
                let datasource_type = string_field(object, "type", "");
                if !datasource_type.is_empty() {
                    collected.insert(datasource_type);
                }
            }
            Value::String(text) if !text.is_empty() => {
                collected.insert(text.clone());
            }
            _ => {}
        }
    };

    collect_from_value(dashboard.get("datasource"), &mut types);
    for panel in panels {
        collect_from_value(panel.get("datasource"), &mut types);
        if let Some(targets) = panel.get("targets").and_then(Value::as_array) {
            for target in targets {
                collect_from_value(target.get("datasource"), &mut types);
            }
        }
    }
    types
}

pub(super) fn validate_dashboard_document(
    document: &Value,
    file: &Path,
    reject_custom_plugins: bool,
    reject_legacy_properties: bool,
    target_schema_version: Option<i64>,
) -> Result<Vec<DashboardValidationIssue>> {
    let document_object = value_as_object(document, "Dashboard payload must be a JSON object.")?;
    if is_dashboard_v2_document(document_object) {
        let title = document_object
            .get("spec")
            .and_then(Value::as_object)
            .map(|spec| string_field(spec, "title", ""))
            .unwrap_or_default();
        let uid = document_object
            .get("metadata")
            .and_then(Value::as_object)
            .map(|metadata| string_field(metadata, "name", ""))
            .unwrap_or_default();
        let mut issues = Vec::new();
        push_issue(
            &mut issues,
            "warning",
            "unsupported-dashboard-v2-resource",
            file,
            &uid,
            &title,
            "Dashboard v2 resources are detected but not fully supported by classic dashboard export validation yet.",
        );
        return Ok(issues);
    }
    let dashboard = extract_dashboard_object(document_object)?;
    let dashboard_uid = string_field(dashboard, "uid", "");
    let dashboard_title = string_field(dashboard, "title", "");
    let dashboard_label = if dashboard_uid.is_empty() {
        dashboard_title.as_str()
    } else {
        dashboard_uid.as_str()
    };
    let mut issues = Vec::new();

    let schema_version = dashboard.get("schemaVersion").and_then(Value::as_i64);
    match schema_version {
        Some(version) => {
            if let Some(target_version) = target_schema_version {
                if version < target_version {
                    push_issue(
                        &mut issues,
                        "error",
                        "schema-migration-required",
                        file,
                        &dashboard_uid,
                        &dashboard_title,
                        format!(
                            "Dashboard {dashboard_label} schemaVersion {version} is below required target {target_version}."
                        ),
                    );
                }
            }
        }
        None => push_issue(
            &mut issues,
            "error",
            "missing-schema-version",
            file,
            &dashboard_uid,
            &dashboard_title,
            format!("Dashboard {dashboard_label} is missing dashboard.schemaVersion."),
        ),
    }

    if document_object.contains_key("__inputs") {
        push_issue(
            &mut issues,
            "error",
            "web-import-placeholders",
            file,
            &dashboard_uid,
            &dashboard_title,
            format!("Dashboard {dashboard_label} still contains Grafana web-import placeholders (__inputs)."),
        );
    }

    if reject_legacy_properties {
        if document_object.contains_key("__requires") {
            push_issue(
                &mut issues,
                "error",
                "legacy-web-import-requires",
                file,
                &dashboard_uid,
                &dashboard_title,
                format!("Dashboard {dashboard_label} still contains top-level __requires web-import scaffolding."),
            );
        }
        if dashboard.contains_key("rows") {
            push_issue(
                &mut issues,
                "error",
                "legacy-row-layout",
                file,
                &dashboard_uid,
                &dashboard_title,
                format!("Dashboard {dashboard_label} uses legacy rows layout. Re-save or migrate before sync."),
            );
        }
    }

    if reject_custom_plugins {
        let panels = collect_dashboard_panels(dashboard);
        for panel in &panels {
            let panel_type = string_field(panel, "type", "");
            if !panel_type.is_empty() && !CORE_PANEL_TYPES.contains(&panel_type.as_str()) {
                push_issue(
                    &mut issues,
                    "error",
                    "custom-panel-plugin",
                    file,
                    &dashboard_uid,
                    &dashboard_title,
                    format!("Dashboard {dashboard_label} uses unsupported custom panel plugin type {panel_type}."),
                );
            }
        }
        for datasource_type in collect_dashboard_datasource_types(dashboard, &panels) {
            if datasource_type.starts_with("-- ") || datasource_type == "__expr__" {
                continue;
            }
            if !CORE_DATASOURCE_TYPES.contains(&datasource_type.as_str()) {
                push_issue(
                    &mut issues,
                    "error",
                    "custom-datasource-plugin",
                    file,
                    &dashboard_uid,
                    &dashboard_title,
                    format!("Dashboard {dashboard_label} references unsupported datasource plugin type {datasource_type}."),
                );
            }
        }
    }

    Ok(issues)
}

fn is_dashboard_v2_document(document: &Map<String, Value>) -> bool {
    let api_version = document
        .get("apiVersion")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if api_version.starts_with("dashboard.grafana.app/") {
        return true;
    }
    let kind = document
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if kind != "Dashboard" {
        return false;
    }
    document
        .get("spec")
        .and_then(Value::as_object)
        .is_some_and(|spec| spec.contains_key("elements") || spec.contains_key("variables"))
}
