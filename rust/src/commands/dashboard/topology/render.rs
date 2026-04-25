use super::{ImpactDocument, TopologyDocument};

fn slug_for_mermaid(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            slug.push(character);
        } else {
            slug.push('_');
        }
    }
    if slug
        .chars()
        .next()
        .map(|character| character.is_ascii_digit())
        .unwrap_or(true)
    {
        slug.insert(0, 'n');
    }
    slug
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(crate) fn render_topology_text(document: &TopologyDocument) -> String {
    let mut lines = vec![format!(
        "Dashboard topology: nodes={} edges={} datasources={} dashboards={} panels={} variables={} alert-resources={} alert-rules={} contact-points={} mute-timings={} notification-policies={} templates={}",
        document.summary.node_count,
        document.summary.edge_count,
        document.summary.datasource_count,
        document.summary.dashboard_count,
        document.summary.panel_count,
        document.summary.variable_count,
        document.summary.alert_resource_count,
        document.summary.alert_rule_count,
        document.summary.contact_point_count,
        document.summary.mute_timing_count,
        document.summary.notification_policy_count,
        document.summary.template_count
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
            escape_label(&node.label)
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
            escape_label(&node.label),
            node.kind
        ));
    }
    for edge in &document.edges {
        lines.push(format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"] ;",
            escape_label(&edge.from),
            escape_label(&edge.to),
            escape_label(&edge.relation)
        ));
    }
    lines.push("}".to_string());
    lines.join("\n")
}

pub(crate) fn render_impact_text(document: &ImpactDocument) -> String {
    let mut lines = vec![format!(
        "Datasource impact: {} dashboards={} alert-resources={} alert-rules={} contact-points={} mute-timings={} notification-policies={} templates={}",
        document.summary.datasource_uid,
        document.summary.dashboard_count,
        document.summary.alert_resource_count,
        document.summary.alert_rule_count,
        document.summary.contact_point_count,
        document.summary.mute_timing_count,
        document.summary.notification_policy_count,
        document.summary.template_count
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
    if !document.affected_contact_points.is_empty() {
        lines.push("Affected contact points:".to_string());
        for resource in &document.affected_contact_points {
            lines.push(format!(
                "  {}:{} {}",
                resource.kind, resource.identity, resource.title
            ));
        }
    }
    if !document.affected_policies.is_empty() {
        lines.push("Affected policies:".to_string());
        for resource in &document.affected_policies {
            lines.push(format!(
                "  {}:{} {}",
                resource.kind, resource.identity, resource.title
            ));
        }
    }
    if !document.affected_templates.is_empty() {
        lines.push("Affected templates:".to_string());
        for resource in &document.affected_templates {
            lines.push(format!(
                "  {}:{} {}",
                resource.kind, resource.identity, resource.title
            ));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
pub(crate) fn build_impact_summary_lines(document: &ImpactDocument) -> Vec<String> {
    vec![
        format!(
            "Datasource {} impact: dashboards={}; alert assets={} (alert rules={}, contact points={}, policies={}, templates={}, mute timings={}).",
            document.summary.datasource_uid,
            document.summary.dashboard_count,
            document.summary.alert_resource_count,
            document.summary.alert_rule_count,
            document.summary.contact_point_count,
            document.summary.notification_policy_count,
            document.summary.template_count,
            document.summary.mute_timing_count
        ),
        "Blast radius first: inspect dashboards, then follow alert rules into routing assets."
            .to_string(),
    ]
}
