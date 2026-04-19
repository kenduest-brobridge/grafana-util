//! Read-only interactive review surface for access plan documents.

use crate::common::Result;

#[cfg(not(feature = "tui"))]
use crate::common::tui;
#[cfg(any(feature = "tui", test))]
use crate::interactive_browser::BrowserItem;
#[cfg(feature = "tui")]
use crate::interactive_browser::run_interactive_browser;

use super::AccessPlanDocument;

#[cfg(any(feature = "tui", test))]
fn build_access_plan_summary_lines(document: &AccessPlanDocument) -> Vec<String> {
    vec![
        "Access plan review".to_string(),
        format!(
            "Resources: {}  checked: {}  same: {}",
            document.summary.resource_count, document.summary.checked, document.summary.same
        ),
        format!(
            "Create: {}  update: {}  extra remote: {}  delete: {}",
            document.summary.create,
            document.summary.update,
            document.summary.extra_remote,
            document.summary.delete
        ),
        format!(
            "Blocked: {}  warning: {}  prune: {}  actions: {}",
            document.summary.blocked,
            document.summary.warning,
            document.summary.prune,
            document.actions.len()
        ),
    ]
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn build_access_plan_browser_items(document: &AccessPlanDocument) -> Vec<BrowserItem> {
    let projection = document.build_review_projection();
    let mut items = Vec::with_capacity(document.resources.len() + projection.actions.len());

    for resource in &document.resources {
        items.push(BrowserItem {
            kind: "resource".to_string(),
            title: format!("{} bundle", resource.resource_kind),
            meta: format!(
                "{}  checked={} same={} create={} update={} extra={} delete={} blocked={} warning={}",
                if resource.bundle_present {
                    "present"
                } else {
                    "missing"
                },
                resource.checked,
                resource.same,
                resource.create,
                resource.update,
                resource.extra_remote,
                resource.delete,
                resource.blocked,
                resource.warning
            ),
            details: {
                let mut details = vec![
                    format!("Resource kind: {}", resource.resource_kind),
                    format!("Source path: {}", resource.source_path),
                    format!("Bundle present: {}", resource.bundle_present),
                    format!("Source count: {}", resource.source_count),
                    format!("Live count: {}", resource.live_count),
                ];
                if let Some(scope) = &resource.scope {
                    details.push(format!("Scope: {}", scope));
                }
                details.extend(resource.notes.iter().map(|note| format!("Note: {}", note)));
                details
            },
        });
    }

    for action in projection.actions {
        let source_path = action
            .raw
            .get("sourcePath")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let scope = action
            .raw
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let mut details = vec![
            format!("Action id: {}", action.action_id),
            format!("Domain: {}", action.domain),
            format!("Resource kind: {}", action.resource_kind),
            format!("Identity: {}", action.identity),
            format!("Action: {}", action.action),
            format!("Status: {}", action.status),
        ];
        if !scope.is_empty() {
            details.push(format!("Scope: {}", scope));
        }
        if let Some(detail) = &action.details {
            details.push(format!("Details: {}", detail));
        }
        if let Some(reason) = &action.blocked_reason {
            details.push(format!("Blocked reason: {}", reason));
        }
        if !source_path.is_empty() {
            details.push(format!("Source path: {}", source_path));
        }
        details.extend(
            action
                .review_hints
                .iter()
                .map(|hint| format!("Hint: {}", hint)),
        );
        items.push(BrowserItem {
            kind: action.resource_kind.clone(),
            title: action.identity,
            meta: format!("{}  {}  {}", action.status, action.action, action.order_group),
            details,
        });
    }

    items
}

#[cfg(feature = "tui")]
pub(crate) fn run_access_plan_interactive(document: &AccessPlanDocument) -> Result<()> {
    let summary_lines = build_access_plan_summary_lines(document);
    let items = build_access_plan_browser_items(document);
    run_interactive_browser("Access plan review", &summary_lines, &items)
}

#[cfg(not(feature = "tui"))]
pub(crate) fn run_access_plan_interactive(_document: &AccessPlanDocument) -> Result<()> {
    Err(tui("Access plan --interactive requires the `tui` feature."))
}
