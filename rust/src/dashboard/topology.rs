//! Artifact-driven topology and impact analysis for dashboards and alert contracts.
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::common::{message, Result};

use super::{
    write_json_document, ImpactArgs, ImpactOutputFormat, TopologyArgs, TopologyOutputFormat,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct TopologySummary {
    #[serde(rename = "nodeCount")]
    pub(crate) node_count: usize,
    #[serde(rename = "edgeCount")]
    pub(crate) edge_count: usize,
    #[serde(rename = "datasourceCount")]
    pub(crate) datasource_count: usize,
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "alertResourceCount")]
    pub(crate) alert_resource_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct TopologyNode {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct TopologyEdge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) relation: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct TopologyDocument {
    pub(crate) summary: TopologySummary,
    pub(crate) nodes: Vec<TopologyNode>,
    pub(crate) edges: Vec<TopologyEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ImpactSummary {
    #[serde(rename = "datasourceUid")]
    pub(crate) datasource_uid: String,
    #[serde(rename = "dashboardCount")]
    pub(crate) dashboard_count: usize,
    #[serde(rename = "alertResourceCount")]
    pub(crate) alert_resource_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ImpactDashboard {
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ImpactAlertResource {
    pub(crate) kind: String,
    pub(crate) identity: String,
    pub(crate) title: String,
    #[serde(rename = "sourcePath")]
    pub(crate) source_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ImpactDocument {
    pub(crate) summary: ImpactSummary,
    pub(crate) dashboards: Vec<ImpactDashboard>,
    #[serde(rename = "alertResources")]
    pub(crate) alert_resources: Vec<ImpactAlertResource>,
}

fn load_object(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&raw)?;
    if !value.is_object() {
        return Err(message(format!(
            "JSON document at {} must be an object.",
            path.display()
        )));
    }
    Ok(value)
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

fn push_unique_node(
    nodes: &mut BTreeMap<String, TopologyNode>,
    id: String,
    kind: &str,
    label: String,
) {
    nodes.entry(id.clone()).or_insert(TopologyNode {
        id,
        kind: kind.to_string(),
        label,
    });
}

fn push_unique_edge(
    edges: &mut BTreeSet<(String, String, String)>,
    from: String,
    to: String,
    relation: &str,
) {
    edges.insert((from, to, relation.to_string()));
}

fn slug_for_mermaid(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn build_topology_document(
    governance_document: &Value,
    alert_contract_document: Option<&Value>,
) -> Result<TopologyDocument> {
    let dashboard_edges = governance_document
        .get("dashboardDatasourceEdges")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            message("Dashboard governance JSON must contain a dashboardDatasourceEdges array.")
        })?;
    let dashboards = governance_document
        .get("dashboardGovernance")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            message("Dashboard governance JSON must contain a dashboardGovernance array.")
        })?;

    let mut nodes = BTreeMap::<String, TopologyNode>::new();
    let mut edges = BTreeSet::<(String, String, String)>::new();
    let mut dashboard_lookup = BTreeMap::<String, (String, String, usize, usize)>::new();
    let mut alert_identity_to_node = BTreeMap::<String, String>::new();
    let mut datasource_names_to_uid = BTreeMap::<String, String>::new();

    for dashboard in dashboards {
        let dashboard_uid = string_field(dashboard, "dashboardUid");
        if dashboard_uid.is_empty() {
            continue;
        }
        let dashboard_title = string_field(dashboard, "dashboardTitle");
        let folder_path = string_field(dashboard, "folderPath");
        let panel_count = dashboard
            .get("panelCount")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let query_count = dashboard
            .get("queryCount")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        dashboard_lookup.insert(
            dashboard_uid.clone(),
            (
                dashboard_title.clone(),
                folder_path,
                panel_count,
                query_count,
            ),
        );
        push_unique_node(
            &mut nodes,
            format!("dashboard:{dashboard_uid}"),
            "dashboard",
            if dashboard_title.is_empty() {
                dashboard_uid.clone()
            } else {
                dashboard_title
            },
        );
    }

    for edge in dashboard_edges {
        let datasource_uid = string_field(edge, "datasourceUid");
        let datasource_name = string_field(edge, "datasource");
        let dashboard_uid = string_field(edge, "dashboardUid");
        if datasource_uid.is_empty() || dashboard_uid.is_empty() {
            continue;
        }
        datasource_names_to_uid.insert(datasource_name.clone(), datasource_uid.clone());
        push_unique_node(
            &mut nodes,
            format!("datasource:{datasource_uid}"),
            "datasource",
            if datasource_name.is_empty() {
                datasource_uid.clone()
            } else {
                datasource_name
            },
        );
        push_unique_edge(
            &mut edges,
            format!("datasource:{datasource_uid}"),
            format!("dashboard:{dashboard_uid}"),
            "feeds",
        );
    }

    let mut alert_resource_count = 0usize;
    if let Some(alert_contract) = alert_contract_document {
        let resources = alert_contract
            .get("resources")
            .and_then(Value::as_array)
            .ok_or_else(|| message("Alert contract JSON must contain a resources array."))?;
        for resource in resources {
            let kind = string_field(resource, "kind");
            let identity = string_field(resource, "identity");
            let title = string_field(resource, "title");
            if kind.is_empty() || identity.is_empty() {
                continue;
            }
            alert_resource_count += 1;
            let node_id = format!("alert:{kind}:{identity}");
            alert_identity_to_node.insert(identity.clone(), node_id.clone());
            push_unique_node(
                &mut nodes,
                node_id.clone(),
                "alert-resource",
                if title.is_empty() {
                    identity.clone()
                } else {
                    format!("{kind}: {title}")
                },
            );
            if let Some(references) = resource.get("references").and_then(Value::as_array) {
                for reference in references.iter().filter_map(Value::as_str) {
                    let reference = reference.trim();
                    if reference.is_empty() {
                        continue;
                    }
                    if let Some(target_node) = alert_identity_to_node.get(reference) {
                        push_unique_edge(
                            &mut edges,
                            node_id.clone(),
                            target_node.clone(),
                            "routes",
                        );
                    }
                    let datasource_uid = datasource_names_to_uid
                        .get(reference)
                        .cloned()
                        .unwrap_or_else(|| reference.to_string());
                    if nodes.contains_key(&format!("datasource:{datasource_uid}")) {
                        push_unique_edge(
                            &mut edges,
                            format!("datasource:{datasource_uid}"),
                            node_id.clone(),
                            "alerts-on",
                        );
                    }
                    if nodes.contains_key(&format!("dashboard:{reference}")) {
                        push_unique_edge(
                            &mut edges,
                            format!("dashboard:{reference}"),
                            node_id.clone(),
                            "backs",
                        );
                    }
                }
            }
        }
    }

    let nodes = nodes.into_values().collect::<Vec<_>>();
    let edges = edges
        .into_iter()
        .map(|(from, to, relation)| TopologyEdge { from, to, relation })
        .collect::<Vec<_>>();
    let datasource_count = nodes
        .iter()
        .filter(|node| node.kind == "datasource")
        .count();
    let dashboard_count = nodes.iter().filter(|node| node.kind == "dashboard").count();

    Ok(TopologyDocument {
        summary: TopologySummary {
            node_count: nodes.len(),
            edge_count: edges.len(),
            datasource_count,
            dashboard_count,
            alert_resource_count,
        },
        nodes,
        edges,
    })
}

pub(crate) fn render_topology_text(document: &TopologyDocument) -> String {
    let mut lines = vec![format!(
        "Dashboard topology: nodes={} edges={} datasources={} dashboards={} alert-resources={}",
        document.summary.node_count,
        document.summary.edge_count,
        document.summary.datasource_count,
        document.summary.dashboard_count,
        document.summary.alert_resource_count
    )];
    for edge in &document.edges {
        lines.push(format!(
            "  {} --{}--> {}",
            edge.from, edge.relation, edge.to
        ));
    }
    lines.join("\n")
}

pub(crate) fn render_topology_mermaid(document: &TopologyDocument) -> String {
    let mut lines = vec!["graph TD".to_string()];
    for node in &document.nodes {
        lines.push(format!(
            "  {}[\"{}\"]",
            slug_for_mermaid(&node.id),
            node.label.replace('"', "\\\"")
        ));
    }
    for edge in &document.edges {
        lines.push(format!(
            "  {} -->|{}| {}",
            slug_for_mermaid(&edge.from),
            edge.relation,
            slug_for_mermaid(&edge.to)
        ));
    }
    lines.join("\n")
}

pub(crate) fn render_topology_dot(document: &TopologyDocument) -> String {
    let mut lines = vec!["digraph grafana_topology {".to_string()];
    for node in &document.nodes {
        lines.push(format!(
            "  \"{}\" [label=\"{}\\n{}\"] ;",
            node.id,
            node.label.replace('"', "\\\""),
            node.kind
        ));
    }
    for edge in &document.edges {
        lines.push(format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"] ;",
            edge.from, edge.to, edge.relation
        ));
    }
    lines.push("}".to_string());
    lines.join("\n")
}

pub(crate) fn build_impact_document(
    governance_document: &Value,
    alert_contract_document: Option<&Value>,
    datasource_uid: &str,
) -> Result<ImpactDocument> {
    let dashboard_edges = governance_document
        .get("dashboardDatasourceEdges")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            message("Dashboard governance JSON must contain a dashboardDatasourceEdges array.")
        })?;
    let dashboards = governance_document
        .get("dashboardGovernance")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            message("Dashboard governance JSON must contain a dashboardGovernance array.")
        })?;
    let mut dashboard_lookup = BTreeMap::<String, ImpactDashboard>::new();
    for dashboard in dashboards {
        let dashboard_uid = string_field(dashboard, "dashboardUid");
        if dashboard_uid.is_empty() {
            continue;
        }
        dashboard_lookup.insert(
            dashboard_uid.clone(),
            ImpactDashboard {
                dashboard_uid,
                dashboard_title: string_field(dashboard, "dashboardTitle"),
                folder_path: string_field(dashboard, "folderPath"),
                panel_count: dashboard
                    .get("panelCount")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize,
                query_count: dashboard
                    .get("queryCount")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize,
            },
        );
    }

    let mut affected_dashboards = BTreeMap::<String, ImpactDashboard>::new();
    let mut datasource_names = BTreeSet::<String>::new();
    for edge in dashboard_edges {
        if string_field(edge, "datasourceUid") != datasource_uid {
            continue;
        }
        datasource_names.insert(string_field(edge, "datasource"));
        let dashboard_uid = string_field(edge, "dashboardUid");
        if let Some(dashboard) = dashboard_lookup.get(&dashboard_uid) {
            affected_dashboards.insert(dashboard_uid, dashboard.clone());
        }
    }

    let mut alert_resources = BTreeMap::<(String, String), ImpactAlertResource>::new();
    if let Some(alert_contract) = alert_contract_document {
        let resources = alert_contract
            .get("resources")
            .and_then(Value::as_array)
            .ok_or_else(|| message("Alert contract JSON must contain a resources array."))?;
        for resource in resources {
            let refs = resource
                .get("references")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<&str>>()
                })
                .unwrap_or_default();
            let touches_datasource = refs.iter().any(|reference| {
                *reference == datasource_uid || datasource_names.contains(*reference)
            });
            let touches_dashboard = refs
                .iter()
                .any(|reference| affected_dashboards.contains_key(*reference));
            if !touches_datasource && !touches_dashboard {
                continue;
            }
            let kind = string_field(resource, "kind");
            let identity = string_field(resource, "identity");
            if kind.is_empty() || identity.is_empty() {
                continue;
            }
            alert_resources.insert(
                (kind.clone(), identity.clone()),
                ImpactAlertResource {
                    kind,
                    identity,
                    title: string_field(resource, "title"),
                    source_path: string_field(resource, "sourcePath"),
                },
            );
        }
    }

    Ok(ImpactDocument {
        summary: ImpactSummary {
            datasource_uid: datasource_uid.to_string(),
            dashboard_count: affected_dashboards.len(),
            alert_resource_count: alert_resources.len(),
        },
        dashboards: affected_dashboards.into_values().collect(),
        alert_resources: alert_resources.into_values().collect(),
    })
}

pub(crate) fn render_impact_text(document: &ImpactDocument) -> String {
    let mut lines = vec![format!(
        "Datasource impact: {} dashboards={} alert-resources={}",
        document.summary.datasource_uid,
        document.summary.dashboard_count,
        document.summary.alert_resource_count
    )];
    if !document.dashboards.is_empty() {
        lines.push("Dashboards:".to_string());
        for dashboard in &document.dashboards {
            lines.push(format!(
                "  {} ({}) panels={} queries={}",
                dashboard.dashboard_uid,
                dashboard.folder_path,
                dashboard.panel_count,
                dashboard.query_count
            ));
        }
    }
    if !document.alert_resources.is_empty() {
        lines.push("Alert resources:".to_string());
        for resource in &document.alert_resources {
            lines.push(format!(
                "  {}:{} {}",
                resource.kind, resource.identity, resource.title
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn run_dashboard_topology(args: &TopologyArgs) -> Result<()> {
    let governance = load_object(&args.governance)?;
    let alert_contract = match args.alert_contract.as_ref() {
        Some(path) => Some(load_object(path)?),
        None => None,
    };
    let document = build_topology_document(&governance, alert_contract.as_ref())?;
    let rendered = match args.output_format {
        TopologyOutputFormat::Text => render_topology_text(&document),
        TopologyOutputFormat::Json => serde_json::to_string_pretty(&document)?,
        TopologyOutputFormat::Mermaid => render_topology_mermaid(&document),
        TopologyOutputFormat::Dot => render_topology_dot(&document),
    };
    if let Some(output_file) = args.output_file.as_ref() {
        if matches!(args.output_format, TopologyOutputFormat::Json) {
            write_json_document(&document, output_file)?;
        } else {
            fs::write(output_file, &rendered)?;
        }
    }
    println!("{rendered}");
    Ok(())
}

pub(crate) fn run_dashboard_impact(args: &ImpactArgs) -> Result<()> {
    let governance = load_object(&args.governance)?;
    let alert_contract = match args.alert_contract.as_ref() {
        Some(path) => Some(load_object(path)?),
        None => None,
    };
    let document =
        build_impact_document(&governance, alert_contract.as_ref(), &args.datasource_uid)?;
    match args.output_format {
        ImpactOutputFormat::Text => println!("{}", render_impact_text(&document)),
        ImpactOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&document)?),
    }
    Ok(())
}
