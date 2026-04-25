//! Stable topology and impact data models.

use serde::Serialize;

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
    #[serde(rename = "panelCount")]
    pub(crate) panel_count: usize,
    #[serde(rename = "variableCount")]
    pub(crate) variable_count: usize,
    #[serde(rename = "alertResourceCount")]
    pub(crate) alert_resource_count: usize,
    #[serde(rename = "alertRuleCount")]
    pub(crate) alert_rule_count: usize,
    #[serde(rename = "contactPointCount")]
    pub(crate) contact_point_count: usize,
    #[serde(rename = "muteTimingCount")]
    pub(crate) mute_timing_count: usize,
    #[serde(rename = "notificationPolicyCount")]
    pub(crate) notification_policy_count: usize,
    #[serde(rename = "templateCount")]
    pub(crate) template_count: usize,
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
    #[serde(rename = "alertRuleCount")]
    pub(crate) alert_rule_count: usize,
    #[serde(rename = "contactPointCount")]
    pub(crate) contact_point_count: usize,
    #[serde(rename = "muteTimingCount")]
    pub(crate) mute_timing_count: usize,
    #[serde(rename = "notificationPolicyCount")]
    pub(crate) notification_policy_count: usize,
    #[serde(rename = "templateCount")]
    pub(crate) template_count: usize,
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
    #[serde(rename = "affectedContactPoints")]
    pub(crate) affected_contact_points: Vec<ImpactAlertResource>,
    #[serde(rename = "affectedPolicies")]
    pub(crate) affected_policies: Vec<ImpactAlertResource>,
    #[serde(rename = "affectedTemplates")]
    pub(crate) affected_templates: Vec<ImpactAlertResource>,
}

#[derive(Clone, Debug)]
pub(super) struct ParsedAlertResource {
    pub(super) normalized_kind: String,
    pub(super) identity: String,
    pub(super) title: String,
    pub(super) source_path: String,
    pub(super) references: Vec<String>,
    pub(super) node_id: String,
}
