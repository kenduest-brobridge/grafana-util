//! Overview artifact summary projection helpers.

use super::{
    overview_kind::{parse_overview_artifact_kind, OverviewArtifactKind},
    OverviewArtifact, OverviewProjectBlocker, OverviewProjectStatus, OverviewSectionFact,
    OverviewSummary,
};
use crate::common::{message, Result};
use serde_json::Value;

fn summary_value<'a>(
    document: &'a Value,
    label: &str,
) -> Result<&'a serde_json::Map<String, Value>> {
    document
        .get("summary")
        .and_then(Value::as_object)
        .ok_or_else(|| message(format!("{label} is missing summary.")))
}

fn summary_number(summary: &serde_json::Map<String, Value>, key: &str) -> usize {
    summary.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

fn fact(label: &str, value: usize) -> OverviewSectionFact {
    OverviewSectionFact {
        label: label.to_string(),
        value: value.to_string(),
    }
}

pub(crate) fn render_overview_finding_summary(
    findings: &[OverviewProjectBlocker],
) -> Option<String> {
    if findings.is_empty() {
        return None;
    }
    Some(
        findings
            .iter()
            .map(|finding| format!("{}:{}", finding.kind, finding.count))
            .collect::<Vec<_>>()
            .join(","),
    )
}

pub(crate) fn render_overview_signal_summary(status: &OverviewProjectStatus) -> Option<String> {
    let mut fragments = Vec::new();
    for domain in &status.domains {
        let has_sync_signals = domain.source_kinds.iter().any(|kind| {
            matches!(
                kind.as_str(),
                "sync-summary" | "bundle-preflight" | "promotion-preflight"
            )
        });
        let should_render = has_sync_signals || domain.source_kinds.len() > 1;
        if !should_render || domain.source_kinds.is_empty() {
            continue;
        }
        fragments.push(format!(
            "{} sources={} signalKeys={} blockers={} warnings={}",
            domain.id,
            domain.source_kinds.join(","),
            domain.signal_keys.len(),
            domain.blocker_count,
            domain.warning_count
        ));
    }
    if fragments.is_empty() {
        None
    } else {
        Some(format!("Signals: {}", fragments.join("; ")))
    }
}

pub(crate) fn render_overview_decision_order(
    status: &OverviewProjectStatus,
) -> Option<Vec<String>> {
    let mut ordered_domains = Vec::new();
    for blocker in &status.top_blockers {
        if !ordered_domains.contains(&blocker.domain) {
            ordered_domains.push(blocker.domain.clone());
        }
    }
    for action in &status.next_actions {
        if !ordered_domains.contains(&action.domain) {
            ordered_domains.push(action.domain.clone());
        }
    }
    if ordered_domains.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for (index, domain) in ordered_domains.iter().enumerate() {
        let blockers = status
            .top_blockers
            .iter()
            .filter(|finding| &finding.domain == domain)
            .map(|finding| format!("{}:{}", finding.kind, finding.count))
            .collect::<Vec<_>>();
        let actions = status
            .next_actions
            .iter()
            .filter(|action| &action.domain == domain)
            .map(|action| action.action.clone())
            .collect::<Vec<_>>();
        let mut line = format!("{}. {}", index + 1, domain);
        if !blockers.is_empty() {
            line.push_str(&format!(" blockers={}", blockers.join(", ")));
        }
        if !actions.is_empty() {
            line.push_str(&format!(" next={}", actions.join(" / ")));
        }
        lines.push(line);
    }
    Some(lines)
}

pub(crate) fn section_summary_facts(
    artifact: &OverviewArtifact,
) -> Result<Vec<OverviewSectionFact>> {
    let summary = summary_value(&artifact.document, "Overview document")?;
    let facts = match parse_overview_artifact_kind(&artifact.kind)? {
        OverviewArtifactKind::DashboardExport => vec![
            fact("dashboards", summary_number(summary, "dashboardCount")),
            fact("folders", summary_number(summary, "folderCount")),
            fact("panels", summary_number(summary, "panelCount")),
            fact("queries", summary_number(summary, "queryCount")),
            fact(
                "datasource-inventory",
                summary_number(summary, "datasourceInventoryCount"),
            ),
            fact(
                "orphaned-datasources",
                summary_number(summary, "orphanedDatasourceCount"),
            ),
            fact(
                "mixed-dashboards",
                summary_number(summary, "mixedDatasourceDashboardCount"),
            ),
        ],
        OverviewArtifactKind::DatasourceExport => vec![
            fact("datasources", summary_number(summary, "datasourceCount")),
            fact("orgs", summary_number(summary, "orgCount")),
            fact("defaults", summary_number(summary, "defaultCount")),
            fact("types", summary_number(summary, "typeCount")),
        ],
        OverviewArtifactKind::AlertExport => vec![
            fact("rules", summary_number(summary, "ruleCount")),
            fact(
                "contact-points",
                summary_number(summary, "contactPointCount"),
            ),
            fact("mute-timings", summary_number(summary, "muteTimingCount")),
            fact("policies", summary_number(summary, "policyCount")),
            fact("templates", summary_number(summary, "templateCount")),
        ],
        OverviewArtifactKind::AccessUserExport => {
            vec![fact("users", summary_number(summary, "recordCount"))]
        }
        OverviewArtifactKind::AccessTeamExport => {
            vec![fact("teams", summary_number(summary, "recordCount"))]
        }
        OverviewArtifactKind::AccessOrgExport => {
            vec![fact("orgs", summary_number(summary, "recordCount"))]
        }
        OverviewArtifactKind::AccessServiceAccountExport => {
            vec![fact(
                "service-accounts",
                summary_number(summary, "recordCount"),
            )]
        }
        OverviewArtifactKind::SyncSummary => vec![
            fact("resources", summary_number(summary, "resourceCount")),
            fact("dashboards", summary_number(summary, "dashboardCount")),
            fact("datasources", summary_number(summary, "datasourceCount")),
            fact("folders", summary_number(summary, "folderCount")),
            fact("alerts", summary_number(summary, "alertCount")),
        ],
        OverviewArtifactKind::BundlePreflight => vec![
            fact("resources", summary_number(summary, "resourceCount")),
            fact(
                "sync-blocking",
                summary_number(summary, "syncBlockingCount"),
            ),
            fact(
                "provider-blocking",
                summary_number(summary, "providerBlockingCount"),
            ),
            fact(
                "secret-placeholder-blocking",
                summary_number(summary, "secretPlaceholderBlockingCount"),
            ),
            fact(
                "alert-artifacts",
                summary_number(summary, "alertArtifactCount"),
            ),
        ],
        OverviewArtifactKind::PromotionPreflight => vec![
            fact("resources", summary_number(summary, "resourceCount")),
            fact("direct", summary_number(summary, "directMatchCount")),
            fact("mapped", summary_number(summary, "mappedCount")),
            fact(
                "missing-mappings",
                summary_number(summary, "missingMappingCount"),
            ),
            fact(
                "bundle-blocking",
                summary_number(summary, "bundleBlockingCount"),
            ),
            fact("blocking", summary_number(summary, "blockingCount")),
        ],
    };
    Ok(facts)
}

pub(crate) fn build_overview_summary(artifacts: &[OverviewArtifact]) -> Result<OverviewSummary> {
    let mut summary = OverviewSummary {
        artifact_count: artifacts.len(),
        ..OverviewSummary::default()
    };
    for artifact in artifacts {
        parse_overview_artifact_kind(&artifact.kind)?.count_summary(&mut summary);
    }
    Ok(summary)
}
