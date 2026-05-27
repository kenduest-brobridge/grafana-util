//! Shared read-only browser projection for mutation review envelopes.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]

use crate::common::Result;
#[cfg(any(feature = "tui", test))]
use crate::interactive_browser::BrowserItem;
use crate::review_contract::ReviewMutationEnvelope;
#[cfg(any(feature = "tui", test))]
use crate::review_contract::{
    append_review_evidence_section, build_review_mutation_action_change_detail_lines,
    build_review_mutation_action_context_lines, build_review_mutation_action_detail_lines,
    build_review_mutation_action_diff_preview_lines, build_review_mutation_action_impact_line,
    build_review_mutation_action_narrative_line, build_review_mutation_action_next_check_lines,
    build_review_mutation_action_target_evidence_lines,
};

#[cfg(any(feature = "tui", test))]
pub(crate) fn build_review_mutation_browser_summary_lines(
    title: &str,
    envelope: &ReviewMutationEnvelope,
) -> Vec<String> {
    let mut lines = vec![
        title.to_string(),
        format!(
            "Actions: {}  domains: {}  same: {}",
            envelope.summary.action_count,
            envelope.summary.domain_count,
            envelope.summary.same_count
        ),
        format!(
            "Blocked: {}  warning: {}",
            envelope.summary.blocked_count, envelope.summary.warning_count
        ),
    ];
    if !envelope.blocked_reasons.is_empty() {
        lines.push(format!(
            "Blocked reasons: {}",
            envelope.blocked_reasons.join(" | ")
        ));
    }
    lines
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn build_review_mutation_browser_items(
    envelope: &ReviewMutationEnvelope,
) -> Vec<BrowserItem> {
    envelope
        .actions
        .iter()
        .map(|action| {
            let mut details = vec![build_review_mutation_action_narrative_line(action)];
            if let Some(impact) = build_review_mutation_action_impact_line(action) {
                details.push(impact);
            }
            append_review_evidence_section(
                &mut details,
                build_review_mutation_action_detail_lines(action),
            );
            details.extend(build_review_mutation_action_change_detail_lines(action));
            details.extend(build_review_mutation_action_diff_preview_lines(action));
            details.extend(build_review_mutation_action_target_evidence_lines(action));
            details.extend(build_review_mutation_action_context_lines(action));
            details.extend(
                action
                    .review_hints
                    .iter()
                    .map(|hint| format!("Hint: {}", hint)),
            );
            details.extend(build_review_mutation_action_next_check_lines(action));
            BrowserItem {
                kind: action.resource_kind.clone(),
                title: action.identity.clone(),
                meta: format!(
                    "{}  {}  {}",
                    action.status, action.action, action.order_group
                ),
                details,
            }
        })
        .collect()
}

#[cfg(feature = "tui")]
pub(crate) fn run_review_mutation_browser(
    title: &str,
    envelope: &ReviewMutationEnvelope,
) -> Result<()> {
    let summary_lines = build_review_mutation_browser_summary_lines(title, envelope);
    let items = build_review_mutation_browser_items(envelope);
    crate::interactive_browser::run_interactive_browser(title, &summary_lines, &items)
}

#[cfg(not(feature = "tui"))]
pub(crate) fn run_review_mutation_browser(
    title: &str,
    _envelope: &ReviewMutationEnvelope,
) -> Result<()> {
    Err(crate::common::tui_feature_required(title))
}
