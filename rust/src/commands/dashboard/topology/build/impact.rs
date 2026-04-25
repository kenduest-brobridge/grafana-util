use serde_json::Value;
use std::collections::BTreeMap;

use crate::common::{message, Result};
use crate::reference_graph::{ReferenceGraph, ReferenceNode, ReferenceNodeKind, ReferenceRelation};

use super::super::{ImpactAlertResource, ImpactDashboard, ImpactDocument, ImpactSummary};
use super::{build_topology_document, collect_alert_resources, string_field};

fn sort_impact_resources(resources: &mut Vec<&ImpactAlertResource>) {
    resources.sort_by(|left, right| {
        left.title
            .cmp(&right.title)
            .then_with(|| left.identity.cmp(&right.identity))
            .then_with(|| left.source_path.cmp(&right.source_path))
    });
}

pub(crate) fn build_impact_document(
    governance_document: &Value,
    alert_contract_document: Option<&Value>,
    datasource_uid: &str,
) -> Result<ImpactDocument> {
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

    let topology = build_topology_document(governance_document, alert_contract_document)?;
    let mut graph = ReferenceGraph::new();
    for node in &topology.nodes {
        graph.insert_node(ReferenceNode {
            id: node.id.clone(),
            kind: ReferenceNodeKind::from_topology_kind(&node.kind),
            label: node.label.clone(),
            source_path: None,
        });
    }
    for edge in &topology.edges {
        graph.add_edge(
            edge.from.clone(),
            edge.to.clone(),
            ReferenceRelation::from_topology_relation(&edge.relation),
        );
    }

    let reachable = graph.reachable_from(&format!("datasource:{datasource_uid}"));

    let mut affected_dashboards = BTreeMap::<String, ImpactDashboard>::new();
    for node in &topology.nodes {
        if node.kind != "dashboard" || !reachable.contains(&node.id) {
            continue;
        }
        let dashboard_uid = node.id.strip_prefix("dashboard:").unwrap_or(&node.id);
        if let Some(dashboard) = dashboard_lookup.get(dashboard_uid) {
            affected_dashboards.insert(dashboard_uid.to_string(), dashboard.clone());
        }
    }

    let mut alert_resources = BTreeMap::<String, ImpactAlertResource>::new();
    if let Some(alert_contract) = alert_contract_document {
        for resource in collect_alert_resources(alert_contract)? {
            if !reachable.contains(&resource.node_id) {
                continue;
            }
            alert_resources.insert(
                resource.node_id.clone(),
                ImpactAlertResource {
                    kind: resource.normalized_kind,
                    identity: resource.identity,
                    title: resource.title,
                    source_path: resource.source_path,
                },
            );
        }
    }

    let mut affected_contact_points = Vec::<ImpactAlertResource>::new();
    let mut affected_policies = Vec::<ImpactAlertResource>::new();
    let mut affected_templates = Vec::<ImpactAlertResource>::new();
    let mut alert_rule_count = 0usize;
    let mut contact_point_count = 0usize;
    let mut mute_timing_count = 0usize;
    let mut notification_policy_count = 0usize;
    let mut template_count = 0usize;
    for resource in alert_resources.values() {
        match resource.kind.as_str() {
            "alert-rule" => alert_rule_count += 1,
            "contact-point" => {
                contact_point_count += 1;
                affected_contact_points.push(resource.clone());
            }
            "mute-timing" => mute_timing_count += 1,
            "notification-policy" => {
                notification_policy_count += 1;
                affected_policies.push(resource.clone());
            }
            "template" => {
                template_count += 1;
                affected_templates.push(resource.clone());
            }
            _ => {}
        }
    }

    let mut dashboards = affected_dashboards.values().collect::<Vec<_>>();
    dashboards.sort_by(|left, right| {
        left.dashboard_title
            .cmp(&right.dashboard_title)
            .then_with(|| left.dashboard_uid.cmp(&right.dashboard_uid))
    });
    let mut alert_resources = alert_resources.values().collect::<Vec<_>>();
    sort_impact_resources(&mut alert_resources);
    let mut affected_contact_point_refs = affected_contact_points.iter().collect::<Vec<_>>();
    let mut affected_policy_refs = affected_policies.iter().collect::<Vec<_>>();
    let mut affected_template_refs = affected_templates.iter().collect::<Vec<_>>();
    sort_impact_resources(&mut affected_contact_point_refs);
    sort_impact_resources(&mut affected_policy_refs);
    sort_impact_resources(&mut affected_template_refs);

    Ok(ImpactDocument {
        summary: ImpactSummary {
            datasource_uid: datasource_uid.to_string(),
            dashboard_count: dashboards.len(),
            alert_resource_count: alert_resources.len(),
            alert_rule_count,
            contact_point_count,
            mute_timing_count,
            notification_policy_count,
            template_count,
        },
        dashboards: dashboards.into_iter().cloned().collect(),
        alert_resources: alert_resources.into_iter().cloned().collect(),
        affected_contact_points: affected_contact_point_refs.into_iter().cloned().collect(),
        affected_policies: affected_policy_refs.into_iter().cloned().collect(),
        affected_templates: affected_template_refs.into_iter().cloned().collect(),
    })
}
