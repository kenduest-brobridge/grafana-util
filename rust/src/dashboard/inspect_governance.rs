//! Governance report builder for inspect mode.
//! Computes datasource-family coverage and risk summaries from the shared query inspection data.
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use super::inspect_render::render_simple_table;
use super::inspect_report::{ExportInspectionQueryReport, ExportInspectionQueryRow};
use super::ExportInspectionSummary;

/// Struct definition for GovernanceSummary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct GovernanceSummary {
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "queryRecordCount")]
    pub(crate) query_record_count: usize,
    #[serde(rename = "datasourceInventoryCount")]
    pub(crate) datasource_inventory_count: usize,
    #[serde(rename = "datasourceFamilyCount")]
    pub(crate) datasource_family_count: usize,
    #[serde(rename = "datasourceCoverageCount")]
    pub(crate) datasource_coverage_count: usize,
    #[serde(rename = "dashboardDatasourceEdgeCount")]
    pub(crate) dashboard_datasource_edge_count: usize,
    #[serde(rename = "datasourceRiskCoverageCount")]
    pub(crate) datasource_risk_coverage_count: usize,
    #[serde(rename = "dashboardRiskCoverageCount")]
    pub(crate) dashboard_risk_coverage_count: usize,
    #[serde(rename = "mixedDatasourceDashboardCount")]
    pub(crate) mixed_datasource_dashboard_count: usize,
    #[serde(rename = "orphanedDatasourceCount")]
    pub(crate) orphaned_datasource_count: usize,
    #[serde(rename = "riskRecordCount")]
    pub(crate) risk_record_count: usize,
    #[serde(rename = "queryAuditCount")]
    pub(crate) query_audit_count: usize,
    #[serde(rename = "dashboardAuditCount")]
    pub(crate) dashboard_audit_count: usize,
}

/// Struct definition for DatasourceFamilyCoverageRow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DatasourceFamilyCoverageRow {
    pub(crate) family: String,
    #[serde(rename = "datasourceTypes")]
    pub(crate) datasource_types: Vec<String>,
    #[serde(rename = "datasourceCount")]
    pub(crate) datasource_count: usize,
    #[serde(rename = "orphanedDatasourceCount")]
    pub(crate) orphaned_datasource_count: usize,
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
}

/// Struct definition for DatasourceCoverageRow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DatasourceCoverageRow {
    #[serde(rename = "datasourceUid")]
    pub(crate) datasource_uid: String,
    pub(crate) datasource: String,
    pub(crate) family: String,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "dashboardUids")]
    pub(crate) dashboard_uids: Vec<String>,
    #[serde(rename = "queryFields")]
    pub(crate) query_fields: Vec<String>,
    pub(crate) orphaned: bool,
}

/// Struct definition for DatasourceGovernanceRow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DatasourceGovernanceRow {
    #[serde(rename = "datasourceUid")]
    pub(crate) datasource_uid: String,
    pub(crate) datasource: String,
    pub(crate) family: String,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "mixedDashboardCount")]
    pub(crate) mixed_dashboard_count: usize,
    #[serde(rename = "riskCount")]
    pub(crate) risk_count: usize,
    #[serde(rename = "riskKinds")]
    pub(crate) risk_kinds: Vec<String>,
    #[serde(rename = "dashboardUids")]
    pub(crate) dashboard_uids: Vec<String>,
    pub(crate) orphaned: bool,
}

/// Struct definition for DashboardDependencyRow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DashboardDependencyRow {
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "dashboardTitle")]
    pub(crate) dashboard_title: String,
    #[serde(rename = "folderPath")]
    pub(crate) folder_path: String,
    #[serde(rename = "file")]
    pub(crate) file_path: String,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
    #[serde(rename = "datasourceCount")]
    pub(crate) datasource_count: usize,
    #[serde(rename = "datasourceFamilyCount")]
    pub(crate) datasource_family_count: usize,
    #[serde(rename = "panelIds")]
    pub(crate) panel_ids: Vec<String>,
    pub(crate) datasources: Vec<String>,
    #[serde(rename = "datasourceFamilies")]
    pub(crate) datasource_families: Vec<String>,
    #[serde(rename = "queryFields")]
    pub(crate) query_fields: Vec<String>,
    #[serde(rename = "panelVariables")]
    pub(crate) panel_variables: Vec<String>,
    #[serde(rename = "queryVariables")]
    pub(crate) query_variables: Vec<String>,
    pub(crate) metrics: Vec<String>,
    pub(crate) functions: Vec<String>,
    pub(crate) measurements: Vec<String>,
    pub(crate) buckets: Vec<String>,
}

/// Struct definition for DashboardDatasourceEdgeRow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DashboardDatasourceEdgeRow {
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "dashboardTitle")]
    pub(crate) dashboard_title: String,
    #[serde(rename = "folderPath")]
    pub(crate) folder_path: String,
    #[serde(rename = "datasourceUid")]
    pub(crate) datasource_uid: String,
    pub(crate) datasource: String,
    #[serde(rename = "datasourceType")]
    pub(crate) datasource_type: String,
    pub(crate) family: String,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
    #[serde(rename = "queryFields")]
    pub(crate) query_fields: Vec<String>,
    #[serde(rename = "queryVariables")]
    pub(crate) query_variables: Vec<String>,
    pub(crate) metrics: Vec<String>,
    pub(crate) functions: Vec<String>,
    pub(crate) measurements: Vec<String>,
    pub(crate) buckets: Vec<String>,
}

/// Struct definition for DashboardGovernanceRow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DashboardGovernanceRow {
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "dashboardTitle")]
    pub(crate) dashboard_title: String,
    #[serde(rename = "folderPath")]
    pub(crate) folder_path: String,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
    #[serde(rename = "datasourceCount")]
    pub(crate) datasource_count: usize,
    #[serde(rename = "datasourceFamilyCount")]
    pub(crate) datasource_family_count: usize,
    pub(crate) datasources: Vec<String>,
    #[serde(rename = "datasourceFamilies")]
    pub(crate) datasource_families: Vec<String>,
    #[serde(rename = "mixedDatasource")]
    pub(crate) mixed_datasource: bool,
    #[serde(rename = "riskCount")]
    pub(crate) risk_count: usize,
    #[serde(rename = "riskKinds")]
    pub(crate) risk_kinds: Vec<String>,
}

/// Struct definition for GovernanceRiskRow.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct GovernanceRiskRow {
    pub(crate) kind: String,
    pub(crate) severity: String,
    pub(crate) category: String,
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "panelId")]
    pub(crate) panel_id: String,
    pub(crate) datasource: String,
    pub(crate) detail: String,
    pub(crate) recommendation: String,
}

/// Struct definition for QueryAuditRow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct QueryAuditRow {
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "dashboardTitle")]
    pub(crate) dashboard_title: String,
    #[serde(rename = "folderPath")]
    pub(crate) folder_path: String,
    #[serde(rename = "panelId")]
    pub(crate) panel_id: String,
    #[serde(rename = "panelTitle")]
    pub(crate) panel_title: String,
    #[serde(rename = "refId")]
    pub(crate) ref_id: String,
    pub(crate) datasource: String,
    #[serde(rename = "datasourceUid")]
    pub(crate) datasource_uid: String,
    #[serde(rename = "datasourceFamily")]
    pub(crate) datasource_family: String,
    #[serde(rename = "aggregationDepth")]
    pub(crate) aggregation_depth: usize,
    #[serde(rename = "regexMatcherCount")]
    pub(crate) regex_matcher_count: usize,
    #[serde(rename = "estimatedSeriesRisk")]
    pub(crate) estimated_series_risk: String,
    #[serde(rename = "queryCostScore")]
    pub(crate) query_cost_score: usize,
    pub(crate) score: usize,
    pub(crate) severity: String,
    pub(crate) reasons: Vec<String>,
    pub(crate) recommendations: Vec<String>,
}

/// Struct definition for DashboardAuditRow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct DashboardAuditRow {
    #[serde(rename = "dashboardUid")]
    pub(crate) dashboard_uid: String,
    #[serde(rename = "dashboardTitle")]
    pub(crate) dashboard_title: String,
    #[serde(rename = "folderPath")]
    pub(crate) folder_path: String,
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "queryCount")]
    pub(crate) query_count: usize,
    #[serde(rename = "refreshIntervalSeconds")]
    pub(crate) refresh_interval_seconds: Option<u64>,
    pub(crate) score: usize,
    pub(crate) severity: String,
    pub(crate) reasons: Vec<String>,
    pub(crate) recommendations: Vec<String>,
}

const GOVERNANCE_RISK_KIND_MIXED_DASHBOARD: &str = "mixed-datasource-dashboard";
const GOVERNANCE_RISK_KIND_ORPHANED_DATASOURCE: &str = "orphaned-datasource";
const GOVERNANCE_RISK_KIND_UNKNOWN_DATASOURCE_FAMILY: &str = "unknown-datasource-family";
const GOVERNANCE_RISK_KIND_EMPTY_QUERY_ANALYSIS: &str = "empty-query-analysis";
const GOVERNANCE_RISK_KIND_BROAD_LOKI_SELECTOR: &str = "broad-loki-selector";
const GOVERNANCE_RISK_KIND_DASHBOARD_PANEL_PRESSURE: &str = "dashboard-panel-pressure";

#[path = "inspect_governance_risk.rs"]
mod inspect_governance_risk;
#[path = "inspect_governance_coverage.rs"]
mod inspect_governance_coverage;

pub(crate) use inspect_governance_risk::{
    build_dashboard_audit_rows, build_governance_risk_rows, build_query_audit_rows,
    find_broad_loki_selector,
};
pub(crate) use inspect_governance_coverage::{
    build_datasource_coverage_rows, build_datasource_family_coverage_rows,
    build_datasource_governance_rows, dashboard_dependency_normalize_family_list,
    dashboard_dependency_unique_strings, normalize_family_name, build_inventory_lookup,
};
#[cfg(test)]
pub(crate) use inspect_governance_risk::governance_risk_spec;

/// Struct definition for ExportInspectionGovernanceDocument.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExportInspectionGovernanceDocument {
    pub(crate) summary: GovernanceSummary,
    #[serde(rename = "datasourceFamilies")]
    pub(crate) datasource_families: Vec<DatasourceFamilyCoverageRow>,
    #[serde(rename = "dashboardDependencies")]
    pub(crate) dashboard_dependencies: Vec<DashboardDependencyRow>,
    #[serde(rename = "dashboardGovernance")]
    pub(crate) dashboard_governance: Vec<DashboardGovernanceRow>,
    #[serde(rename = "dashboardDatasourceEdges")]
    pub(crate) dashboard_datasource_edges: Vec<DashboardDatasourceEdgeRow>,
    #[serde(rename = "datasourceGovernance")]
    pub(crate) datasource_governance: Vec<DatasourceGovernanceRow>,
    pub(crate) datasources: Vec<DatasourceCoverageRow>,
    #[serde(rename = "riskRecords")]
    pub(crate) risk_records: Vec<GovernanceRiskRow>,
    #[serde(rename = "queryAudits")]
    pub(crate) query_audits: Vec<QueryAuditRow>,
    #[serde(rename = "dashboardAudits")]
    pub(crate) dashboard_audits: Vec<DashboardAuditRow>,
}

#[derive(Clone, Debug)]
struct ResolvedDatasourceIdentity {
    uid: String,
    name: String,
    datasource_type: String,
}

fn resolve_datasource_identity(
    row: &ExportInspectionQueryRow,
    inventory_by_uid: &BTreeMap<String, (String, String, String)>,
    inventory_by_name: &BTreeMap<String, (String, String, String)>,
) -> ResolvedDatasourceIdentity {
    let normalized_family = normalize_family_name(&row.datasource_type);
    let datasource_type = if matches!(normalized_family.as_str(), "search" | "tracing") {
        row.datasource_type.clone()
    } else {
        "unknown".to_string()
    };
    if !row.datasource_uid.trim().is_empty() {
        if let Some((uid, name, datasource_type)) = inventory_by_uid.get(&row.datasource_uid) {
            return ResolvedDatasourceIdentity {
                uid: uid.clone(),
                name: name.clone(),
                datasource_type: datasource_type.clone(),
            };
        }
    }
    if !row.datasource.trim().is_empty() {
        if let Some((uid, name, datasource_type)) = inventory_by_uid
            .get(&row.datasource)
            .or_else(|| inventory_by_name.get(&row.datasource))
        {
            return ResolvedDatasourceIdentity {
                uid: uid.clone(),
                name: name.clone(),
                datasource_type: datasource_type.clone(),
            };
        }
    }
    if !row.datasource_uid.trim().is_empty() {
        return ResolvedDatasourceIdentity {
            uid: row.datasource_uid.clone(),
            name: if row.datasource.trim().is_empty() {
                row.datasource_uid.clone()
            } else {
                row.datasource.clone()
            },
            datasource_type,
        };
    }
    if !row.datasource.trim().is_empty() {
        return ResolvedDatasourceIdentity {
            uid: row.datasource.clone(),
            name: row.datasource.clone(),
            datasource_type,
        };
    }
    ResolvedDatasourceIdentity {
        uid: "unknown".to_string(),
        name: "unknown".to_string(),
        datasource_type,
    }
}



/// Fold grouped query rows into dashboard dependencies without changing the
/// per-target row contract that the query report already established.
pub(crate) fn build_dashboard_dependency_rows(
    report: &ExportInspectionQueryReport,
) -> Vec<DashboardDependencyRow> {
    let normalized = super::inspect_report::normalize_query_report(report);
    normalized
        .dashboards
        .into_iter()
        .map(|dashboard| {
            let dashboard_uid = dashboard.dashboard_uid;
            let dashboard_title = dashboard.dashboard_title;
            let folder_path = dashboard.folder_path;
            let file_path = dashboard.file_path;
            let panel_count = dashboard.panels.len();
            let query_count = dashboard
                .panels
                .iter()
                .map(|panel| panel.queries.len())
                .sum::<usize>();
            let datasources = dashboard.datasources;
            let datasource_families =
                dashboard_dependency_normalize_family_list(&dashboard.datasource_families);
            let panel_ids = dashboard_dependency_unique_strings(
                dashboard.panels.iter().map(|panel| panel.panel_id.clone()),
            );
            let query_fields = dashboard_dependency_unique_strings(
                dashboard
                    .panels
                    .iter()
                    .flat_map(|panel| panel.query_fields.iter().cloned()),
            );
            let panel_variables = dashboard_dependency_unique_strings(
                dashboard.panels.iter().flat_map(|panel| {
                    panel
                        .queries
                        .iter()
                        .flat_map(|row| row.panel_variables.iter().cloned())
                }),
            );
            let query_variables = dashboard_dependency_unique_strings(
                dashboard.panels.iter().flat_map(|panel| {
                    panel
                        .queries
                        .iter()
                        .flat_map(|row| row.query_variables.iter().cloned())
                }),
            );
            let metrics = dashboard_dependency_unique_strings(dashboard.panels.iter().flat_map(|panel| {
                panel
                    .queries
                    .iter()
                    .flat_map(|row| row.metrics.iter().cloned())
            }));
            let functions = dashboard_dependency_unique_strings(dashboard.panels.iter().flat_map(|panel| {
                panel
                    .queries
                    .iter()
                    .flat_map(|row| row.functions.iter().cloned())
            }));
            let measurements = dashboard_dependency_unique_strings(dashboard.panels.iter().flat_map(|panel| {
                panel
                    .queries
                    .iter()
                    .flat_map(|row| row.measurements.iter().cloned())
            }));
            let buckets = dashboard_dependency_unique_strings(dashboard.panels.iter().flat_map(|panel| {
                panel
                    .queries
                    .iter()
                    .flat_map(|row| row.buckets.iter().cloned())
            }));

            DashboardDependencyRow {
                dashboard_uid,
                dashboard_title,
                folder_path,
                file_path,
                panel_count,
                query_count,
                datasource_count: datasources.len(),
                datasource_family_count: datasource_families.len(),
                panel_ids,
                datasources,
                datasource_families,
                query_fields,
                panel_variables,
                query_variables,
                metrics,
                functions,
                measurements,
                buckets,
            }
        })
        .collect()
}

/// Derive dashboard governance from the dependency rows plus risk records so mixed
/// datasource and pressure signals share one source of truth.
pub(crate) fn build_dashboard_governance_rows(
    report: &ExportInspectionQueryReport,
    risk_records: &[GovernanceRiskRow],
) -> Vec<DashboardGovernanceRow> {
    let mut risk_by_dashboard = BTreeMap::<String, (BTreeSet<String>, usize)>::new();
    for risk in risk_records {
        if risk.dashboard_uid.trim().is_empty() {
            continue;
        }
        let entry = risk_by_dashboard
            .entry(risk.dashboard_uid.clone())
            .or_insert_with(|| (BTreeSet::new(), 0usize));
        entry.0.insert(risk.kind.clone());
        entry.1 += 1;
    }

    let mut rows = build_dashboard_dependency_rows(report)
        .into_iter()
        .map(|row| {
            let (risk_kinds, risk_count) = risk_by_dashboard
                .remove(&row.dashboard_uid)
                .map(|(kinds, count)| (kinds.into_iter().collect::<Vec<String>>(), count))
                .unwrap_or_else(|| (Vec::new(), 0usize));
            DashboardGovernanceRow {
                dashboard_uid: row.dashboard_uid,
                dashboard_title: row.dashboard_title,
                folder_path: row.folder_path,
                panel_count: row.panel_count,
                query_count: row.query_count,
                datasource_count: row.datasource_count,
                datasource_family_count: row.datasource_family_count,
                datasources: row.datasources,
                datasource_families: row.datasource_families,
                mixed_datasource: risk_kinds
                    .iter()
                    .any(|kind| kind == GOVERNANCE_RISK_KIND_MIXED_DASHBOARD),
                risk_count,
                risk_kinds,
            }
        })
        .collect::<Vec<DashboardGovernanceRow>>();

    rows.sort_by(|left, right| {
        right
            .risk_count
            .cmp(&left.risk_count)
            .then(right.mixed_datasource.cmp(&left.mixed_datasource))
            .then(right.query_count.cmp(&left.query_count))
            .then(left.dashboard_uid.cmp(&right.dashboard_uid))
    });
    rows
}

/// Produce dashboard-to-datasource edge rows for governance and topology consumers from
/// the normalized report.
pub(crate) fn build_dashboard_datasource_edge_rows(
    summary: &ExportInspectionSummary,
    report: &ExportInspectionQueryReport,
) -> Vec<DashboardDatasourceEdgeRow> {
    let (inventory_by_uid, inventory_by_name) = build_inventory_lookup(summary);
    let mut edges = BTreeMap::<
        (String, String),
        (
            String,
            String,
            String,
            String,
            String,
            BTreeSet<String>,
            BTreeSet<String>,
            BTreeSet<String>,
            BTreeSet<String>,
            BTreeSet<String>,
            BTreeSet<String>,
            BTreeSet<String>,
            usize,
        ),
    >::new();
    for row in &report.queries {
        let identity = resolve_datasource_identity(row, &inventory_by_uid, &inventory_by_name);
        let edge = edges
            .entry((row.dashboard_uid.clone(), identity.uid.clone()))
            .or_insert_with(|| {
                (
                    row.dashboard_title.clone(),
                    row.folder_path.clone(),
                    identity.name.clone(),
                    identity.datasource_type.clone(),
                    normalize_family_name(&identity.datasource_type),
                    BTreeSet::new(),
                    BTreeSet::new(),
                    BTreeSet::new(),
                    BTreeSet::new(),
                    BTreeSet::new(),
                    BTreeSet::new(),
                    BTreeSet::new(),
                    0usize,
                )
            });
        edge.5
            .insert(format!("{}:{}", row.dashboard_uid, row.panel_id));
        if !row.query_field.trim().is_empty() {
            edge.6.insert(row.query_field.clone());
        }
        edge.7.extend(row.query_variables.iter().cloned());
        edge.8.extend(row.metrics.iter().cloned());
        edge.9.extend(row.functions.iter().cloned());
        edge.10.extend(row.measurements.iter().cloned());
        edge.11.extend(row.buckets.iter().cloned());
        edge.12 += 1;
    }
    edges
        .into_iter()
        .map(
            |(
                (dashboard_uid, datasource_uid),
                (
                    dashboard_title,
                    folder_path,
                    datasource,
                    datasource_type,
                    family,
                    panel_keys,
                    query_fields,
                    query_variables,
                    metrics,
                    functions,
                    measurements,
                    buckets,
                    query_count,
                ),
            )| DashboardDatasourceEdgeRow {
                dashboard_uid,
                dashboard_title,
                folder_path,
                datasource_uid,
                datasource,
                datasource_type,
                family,
                panel_count: panel_keys.len(),
                query_count,
                query_fields: query_fields.into_iter().collect(),
                query_variables: query_variables.into_iter().collect(),
                metrics: metrics.into_iter().collect(),
                functions: functions.into_iter().collect(),
                measurements: measurements.into_iter().collect(),
                buckets: buckets.into_iter().collect(),
            },
        )
        .collect()
}

/// Project the normalized summary/report pair into the stable governance JSON document
/// used by both live and export inspect flows.
pub(crate) fn build_export_inspection_governance_document(
    summary: &ExportInspectionSummary,
    report: &ExportInspectionQueryReport,
) -> ExportInspectionGovernanceDocument {
    let datasource_families = build_datasource_family_coverage_rows(summary, report);
    let dashboard_dependencies = build_dashboard_dependency_rows(report);
    let query_audits = build_query_audit_rows(summary, report);
    let dashboard_audits = build_dashboard_audit_rows(report);
    let risk_records = build_governance_risk_rows(summary, report);
    let dashboard_governance = build_dashboard_governance_rows(report, &risk_records);
    let dashboard_datasource_edges = build_dashboard_datasource_edge_rows(summary, report);
    let datasource_governance = build_datasource_governance_rows(summary, report);
    let datasources = build_datasource_coverage_rows(summary, report);
    ExportInspectionGovernanceDocument {
        summary: GovernanceSummary {
            dashboard_count: summary.dashboard_count,
            query_record_count: report.summary.report_row_count,
            datasource_inventory_count: summary.datasource_inventory_count,
            datasource_family_count: datasource_families.len(),
            datasource_coverage_count: datasources.len(),
            dashboard_datasource_edge_count: dashboard_datasource_edges.len(),
            datasource_risk_coverage_count: datasource_governance
                .iter()
                .filter(|row| row.risk_count != 0)
                .count(),
            dashboard_risk_coverage_count: dashboard_governance
                .iter()
                .filter(|row| row.risk_count != 0)
                .count(),
            mixed_datasource_dashboard_count: summary.mixed_dashboard_count,
            orphaned_datasource_count: summary
                .datasource_inventory
                .iter()
                .filter(|item| item.reference_count == 0 && item.dashboard_count == 0)
                .count(),
            risk_record_count: risk_records.len(),
            query_audit_count: query_audits.len(),
            dashboard_audit_count: dashboard_audits.iter().filter(|row| row.score != 0).count(),
        },
        datasource_families,
        dashboard_dependencies,
        dashboard_governance,
        dashboard_datasource_edges,
        datasource_governance,
        datasources,
        risk_records,
        query_audits,
        dashboard_audits,
    }
}

/// Render the already-normalized governance document into text rows without recomputing
/// risk logic or re-reading files.
pub(crate) fn render_governance_table_report(
    import_dir: &str,
    document: &ExportInspectionGovernanceDocument,
) -> Vec<String> {
    let mut lines = vec![
        format!("Export inspection governance: {import_dir}"),
        String::new(),
    ];

    lines.push("# Summary".to_string());
    lines.extend(render_simple_table(
        &[
            "DASHBOARDS",
            "QUERIES",
            "FAMILIES",
            "DATASOURCES",
            "DASHBOARD_DATASOURCE_EDGES",
            "DATASOURCES_WITH_RISKS",
            "DASHBOARDS_WITH_RISKS",
            "MIXED_DASHBOARDS",
            "ORPHANED_DATASOURCES",
            "RISKS",
        ],
        &[vec![
            document.summary.dashboard_count.to_string(),
            document.summary.query_record_count.to_string(),
            document.summary.datasource_family_count.to_string(),
            document.summary.datasource_coverage_count.to_string(),
            document.summary.dashboard_datasource_edge_count.to_string(),
            document.summary.datasource_risk_coverage_count.to_string(),
            document.summary.dashboard_risk_coverage_count.to_string(),
            document
                .summary
                .mixed_datasource_dashboard_count
                .to_string(),
            document.summary.orphaned_datasource_count.to_string(),
            document.summary.risk_record_count.to_string(),
        ]],
        true,
    ));

    lines.push(String::new());
    lines.push("# Datasource Families".to_string());
    let family_rows = document
        .datasource_families
        .iter()
        .map(|row| {
            vec![
                row.family.clone(),
                row.datasource_types.join(","),
                row.datasource_count.to_string(),
                row.orphaned_datasource_count.to_string(),
                row.dashboard_count.to_string(),
                row.panel_count.to_string(),
                row.query_count.to_string(),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    if family_rows.is_empty() {
        lines.push("(none)".to_string());
    } else {
        lines.extend(render_simple_table(
            &[
                "FAMILY",
                "TYPES",
                "DATASOURCES",
                "ORPHANED_DATASOURCES",
                "DASHBOARDS",
                "PANELS",
                "QUERIES",
            ],
            &family_rows,
            true,
        ));
    }

    lines.push(String::new());
    lines.push("# Dashboard Dependencies".to_string());
    let dashboard_rows = document
        .dashboard_dependencies
        .iter()
        .map(|row| {
            vec![
                row.dashboard_uid.clone(),
                row.dashboard_title.clone(),
                row.folder_path.clone(),
                row.panel_count.to_string(),
                row.query_count.to_string(),
                row.datasource_count.to_string(),
                row.datasource_family_count.to_string(),
                row.datasources.join(","),
                row.datasource_families.join(","),
                row.query_fields.join(","),
                row.metrics.join(","),
                row.functions.join(","),
                row.measurements.join(","),
                row.buckets.join(","),
                row.file_path.clone(),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    if dashboard_rows.is_empty() {
        lines.push("(none)".to_string());
    } else {
        lines.extend(render_simple_table(
            &[
                "DASHBOARD_UID",
                "TITLE",
                "FOLDER_PATH",
                "PANELS",
                "QUERIES",
                "DATASOURCE_COUNT",
                "DATASOURCE_FAMILY_COUNT",
                "DATASOURCES",
                "FAMILIES",
                "QUERY_FIELDS",
                "METRICS",
                "FUNCTIONS",
                "MEASUREMENTS",
                "BUCKETS",
                "FILE",
            ],
            &dashboard_rows,
            true,
        ));
    }

    lines.push(String::new());
    lines.push("# Dashboard Governance".to_string());
    let dashboard_governance_rows = document
        .dashboard_governance
        .iter()
        .map(|row| {
            let datasources = if row.datasources.is_empty() {
                "(none)".to_string()
            } else {
                row.datasources.join(",")
            };
            let datasource_families = if row.datasource_families.is_empty() {
                "(none)".to_string()
            } else {
                row.datasource_families.join(",")
            };
            let risk_kinds = if row.risk_kinds.is_empty() {
                "(none)".to_string()
            } else {
                row.risk_kinds.join(",")
            };
            vec![
                row.dashboard_uid.clone(),
                row.dashboard_title.clone(),
                row.folder_path.clone(),
                row.panel_count.to_string(),
                row.query_count.to_string(),
                row.datasource_count.to_string(),
                row.datasource_family_count.to_string(),
                datasources,
                datasource_families,
                if row.mixed_datasource {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
                row.risk_count.to_string(),
                risk_kinds,
            ]
        })
        .collect::<Vec<Vec<String>>>();
    if dashboard_governance_rows.is_empty() {
        lines.push("(none)".to_string());
    } else {
        lines.extend(render_simple_table(
            &[
                "DASHBOARD_UID",
                "TITLE",
                "FOLDER_PATH",
                "PANELS",
                "QUERIES",
                "DATASOURCE_COUNT",
                "DATASOURCE_FAMILY_COUNT",
                "DATASOURCES",
                "FAMILIES",
                "MIXED_DATASOURCE",
                "RISKS",
                "RISK_KINDS",
            ],
            &dashboard_governance_rows,
            true,
        ));
    }

    lines.push(String::new());
    lines.push("# Dashboard Datasource Edges".to_string());
    let edge_rows = document
        .dashboard_datasource_edges
        .iter()
        .map(|row| {
            vec![
                row.dashboard_uid.clone(),
                row.dashboard_title.clone(),
                row.folder_path.clone(),
                row.datasource_uid.clone(),
                row.datasource.clone(),
                row.datasource_type.clone(),
                row.family.clone(),
                row.panel_count.to_string(),
                row.query_count.to_string(),
                row.query_fields.join(","),
                row.metrics.join(","),
                row.functions.join(","),
                row.measurements.join(","),
                row.buckets.join(","),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    if edge_rows.is_empty() {
        lines.push("(none)".to_string());
    } else {
        lines.extend(render_simple_table(
            &[
                "DASHBOARD_UID",
                "TITLE",
                "FOLDER_PATH",
                "DATASOURCE_UID",
                "DATASOURCE",
                "DATASOURCE_TYPE",
                "FAMILY",
                "PANELS",
                "QUERIES",
                "QUERY_FIELDS",
                "METRICS",
                "FUNCTIONS",
                "MEASUREMENTS",
                "BUCKETS",
            ],
            &edge_rows,
            true,
        ));
    }

    lines.push(String::new());
    lines.push("# Datasource Governance".to_string());
    let datasource_governance_rows = document
        .datasource_governance
        .iter()
        .map(|row| {
            let dashboard_uids = if row.dashboard_uids.is_empty() {
                "(none)".to_string()
            } else {
                row.dashboard_uids.join(",")
            };
            let risk_kinds = if row.risk_kinds.is_empty() {
                "(none)".to_string()
            } else {
                row.risk_kinds.join(",")
            };
            vec![
                row.datasource_uid.clone(),
                row.datasource.clone(),
                row.family.clone(),
                row.query_count.to_string(),
                row.dashboard_count.to_string(),
                row.panel_count.to_string(),
                row.mixed_dashboard_count.to_string(),
                row.risk_count.to_string(),
                risk_kinds,
                dashboard_uids,
                if row.orphaned {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
            ]
        })
        .collect::<Vec<Vec<String>>>();
    if datasource_governance_rows.is_empty() {
        lines.push("(none)".to_string());
    } else {
        lines.extend(render_simple_table(
            &[
                "UID",
                "DATASOURCE",
                "FAMILY",
                "QUERIES",
                "DASHBOARDS",
                "PANELS",
                "MIXED_DASHBOARDS",
                "RISKS",
                "RISK_KINDS",
                "DASHBOARD_UIDS",
                "ORPHANED",
            ],
            &datasource_governance_rows,
            true,
        ));
    }

    lines.push(String::new());
    lines.push("# Datasources".to_string());
    let datasource_rows = document
        .datasources
        .iter()
        .map(|row| {
            let dashboard_uids = if row.dashboard_uids.is_empty() {
                "(none)".to_string()
            } else {
                row.dashboard_uids.join(",")
            };
            let query_fields = if row.query_fields.is_empty() {
                "(none)".to_string()
            } else {
                row.query_fields.join(",")
            };
            vec![
                row.datasource_uid.clone(),
                row.datasource.clone(),
                row.family.clone(),
                row.query_count.to_string(),
                row.dashboard_count.to_string(),
                row.panel_count.to_string(),
                dashboard_uids,
                query_fields,
                if row.orphaned {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
            ]
        })
        .collect::<Vec<Vec<String>>>();
    if datasource_rows.is_empty() {
        lines.push("(none)".to_string());
    } else {
        lines.extend(render_simple_table(
            &[
                "UID",
                "DATASOURCE",
                "FAMILY",
                "QUERIES",
                "DASHBOARDS",
                "PANELS",
                "DASHBOARD_UIDS",
                "QUERY_FIELDS",
                "ORPHANED",
            ],
            &datasource_rows,
            true,
        ));
    }

    lines.push(String::new());
    lines.push("# Risks".to_string());
    let risk_rows = document
        .risk_records
        .iter()
        .map(|row| {
            vec![
                row.severity.clone(),
                row.category.clone(),
                row.kind.clone(),
                row.dashboard_uid.clone(),
                row.panel_id.clone(),
                row.datasource.clone(),
                row.detail.clone(),
                row.recommendation.clone(),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    if risk_rows.is_empty() {
        lines.push("(none)".to_string());
    } else {
        lines.extend(render_simple_table(
            &[
                "SEVERITY",
                "CATEGORY",
                "KIND",
                "DASHBOARD_UID",
                "PANEL_ID",
                "DATASOURCE",
                "DETAIL",
                "RECOMMENDATION",
            ],
            &risk_rows,
            true,
        ));
    }
    lines
}
