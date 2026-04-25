//! Topology and impact document builders.
#![cfg_attr(not(any(feature = "tui", test)), allow(dead_code))]

use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{message, Result};

use super::{ParsedAlertResource, TopologyDocument, TopologyEdge, TopologyNode, TopologySummary};

#[path = "topology_build/impact.rs"]
mod impact;

pub(crate) use impact::build_impact_document;

pub(super) fn string_field(record: &Value, key: &str) -> String {
    record
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
        .to_string()
}

pub(super) fn string_list_field(record: &Value, key: &str) -> Vec<String> {
    record
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_alert_kind(kind: &str) -> &str {
    match kind {
        "grafana-alert-rule" => "alert-rule",
        "grafana-contact-point" => "contact-point",
        "grafana-mute-timing" => "mute-timing",
        "grafana-notification-policies" | "grafana-notification-policy" => "notification-policy",
        "grafana-notification-template" => "template",
        _ => "alert-resource",
    }
}

fn alert_resource_label(title: &str, identity: &str) -> String {
    if title.is_empty() {
        identity.to_string()
    } else {
        title.to_string()
    }
}

pub(super) fn collect_alert_resources(alert_contract: &Value) -> Result<Vec<ParsedAlertResource>> {
    let resources = alert_contract
        .get("resources")
        .and_then(Value::as_array)
        .ok_or_else(|| message("Alert contract JSON must contain a resources array."))?;
    let mut parsed_resources = Vec::new();
    for resource in resources {
        let kind = string_field(resource, "kind");
        let identity = string_field(resource, "identity");
        let title = string_field(resource, "title");
        if kind.is_empty() || identity.is_empty() {
            continue;
        }
        let references = resource
            .get("references")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        parsed_resources.push(ParsedAlertResource {
            node_id: format!("alert:{kind}:{identity}"),
            normalized_kind: normalize_alert_kind(&kind).to_string(),
            identity,
            title,
            source_path: string_field(resource, "sourcePath"),
            references,
        });
    }
    Ok(parsed_resources)
}

fn edge_relation_for_alert_reference(source_kind: &str, target_kind: &str) -> Option<&'static str> {
    match (source_kind, target_kind) {
        ("alert-rule", "contact-point") => Some("routes-to"),
        ("alert-rule", "notification-policy") => Some("routes-to"),
        ("alert-rule", "template") => Some("uses-template"),
        ("contact-point", "template") => Some("uses-template"),
        ("notification-policy", "template") => Some("uses-template"),
        _ => None,
    }
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

fn panel_node_id(dashboard_uid: &str, panel_id: &str) -> String {
    format!("panel:{dashboard_uid}:{panel_id}")
}

fn variable_node_id(dashboard_uid: &str, variable: &str) -> String {
    format!("variable:{dashboard_uid}:{variable}")
}

pub(crate) fn compare_topology_nodes(left: &TopologyNode, right: &TopologyNode) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.label.cmp(&right.label))
        .then_with(|| left.id.cmp(&right.id))
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
    let mut alert_identity_to_node = BTreeMap::<String, String>::new();
    let mut alert_identity_to_kind = BTreeMap::<String, String>::new();
    let mut datasource_names_to_uid = BTreeMap::<String, String>::new();

    for dashboard in dashboards {
        let dashboard_uid = string_field(dashboard, "dashboardUid");
        if dashboard_uid.is_empty() {
            continue;
        }
        let dashboard_title = string_field(dashboard, "dashboardTitle");
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
        for variable in string_list_field(edge, "queryVariables") {
            let variable_id = variable_node_id(&dashboard_uid, &variable);
            push_unique_node(
                &mut nodes,
                variable_id.clone(),
                "variable",
                variable.clone(),
            );
            push_unique_edge(
                &mut edges,
                format!("datasource:{datasource_uid}"),
                variable_id,
                "feeds-variable",
            );
        }
    }

    let empty: &[Value] = &[];
    let dashboard_dependencies = governance_document
        .get("dashboardDependencies")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(empty);
    for dependency in dashboard_dependencies {
        let dashboard_uid = string_field(dependency, "dashboardUid");
        if dashboard_uid.is_empty() {
            continue;
        }
        let panel_ids = string_list_field(dependency, "panelIds");
        if panel_ids.is_empty() {
            continue;
        }
        let mut variable_names = BTreeSet::<String>::new();
        for variable in string_list_field(dependency, "panelVariables") {
            variable_names.insert(variable);
        }
        for variable in string_list_field(dependency, "queryVariables") {
            variable_names.insert(variable);
        }
        let dashboard_node_id = format!("dashboard:{dashboard_uid}");
        for panel_id in panel_ids {
            let panel_id = panel_id.trim();
            if panel_id.is_empty() {
                continue;
            }
            let panel_id_string = panel_id.to_string();
            let panel_node = panel_node_id(&dashboard_uid, &panel_id_string);
            push_unique_node(
                &mut nodes,
                panel_node.clone(),
                "panel",
                format!("Panel {panel_id_string}"),
            );
            push_unique_edge(
                &mut edges,
                panel_node.clone(),
                dashboard_node_id.clone(),
                "belongs-to",
            );
            for variable in &variable_names {
                let variable_id = variable_node_id(&dashboard_uid, variable);
                push_unique_node(
                    &mut nodes,
                    variable_id.clone(),
                    "variable",
                    variable.clone(),
                );
                push_unique_edge(&mut edges, variable_id, panel_node.clone(), "used-by");
            }
        }
    }

    let mut alert_resource_count = 0usize;
    if let Some(alert_contract) = alert_contract_document {
        let parsed_alert_resources = collect_alert_resources(alert_contract)?;
        for resource in &parsed_alert_resources {
            alert_resource_count += 1;
            alert_identity_to_node.insert(resource.identity.clone(), resource.node_id.clone());
            alert_identity_to_kind
                .insert(resource.identity.clone(), resource.normalized_kind.clone());
            push_unique_node(
                &mut nodes,
                resource.node_id.clone(),
                &resource.normalized_kind,
                alert_resource_label(&resource.title, &resource.identity),
            );
        }
        for resource in &parsed_alert_resources {
            for reference in &resource.references {
                if let Some(target_node) = alert_identity_to_node.get(reference) {
                    if let Some(target_kind) = alert_identity_to_kind.get(reference) {
                        if let Some(relation) = edge_relation_for_alert_reference(
                            &resource.normalized_kind,
                            target_kind,
                        ) {
                            push_unique_edge(
                                &mut edges,
                                resource.node_id.clone(),
                                target_node.clone(),
                                relation,
                            );
                        }
                    }
                }
                let datasource_uid = datasource_names_to_uid
                    .get(reference)
                    .cloned()
                    .unwrap_or_else(|| reference.clone());
                if nodes.contains_key(&format!("datasource:{datasource_uid}"))
                    && resource.normalized_kind == "alert-rule"
                {
                    push_unique_edge(
                        &mut edges,
                        format!("datasource:{datasource_uid}"),
                        resource.node_id.clone(),
                        "alerts-on",
                    );
                }
                if nodes.contains_key(&format!("dashboard:{reference}"))
                    && resource.normalized_kind == "alert-rule"
                {
                    push_unique_edge(
                        &mut edges,
                        format!("dashboard:{reference}"),
                        resource.node_id.clone(),
                        "backs",
                    );
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
    let panel_count = nodes.iter().filter(|node| node.kind == "panel").count();
    let variable_count = nodes.iter().filter(|node| node.kind == "variable").count();
    let alert_rule_count = nodes
        .iter()
        .filter(|node| node.kind == "alert-rule")
        .count();
    let contact_point_count = nodes
        .iter()
        .filter(|node| node.kind == "contact-point")
        .count();
    let mute_timing_count = nodes
        .iter()
        .filter(|node| node.kind == "mute-timing")
        .count();
    let notification_policy_count = nodes
        .iter()
        .filter(|node| node.kind == "notification-policy")
        .count();
    let template_count = nodes.iter().filter(|node| node.kind == "template").count();

    Ok(TopologyDocument {
        summary: TopologySummary {
            node_count: nodes.len(),
            edge_count: edges.len(),
            datasource_count,
            dashboard_count,
            panel_count,
            variable_count,
            alert_resource_count,
            alert_rule_count,
            contact_point_count,
            mute_timing_count,
            notification_policy_count,
            template_count,
        },
        nodes,
        edges,
    })
}
