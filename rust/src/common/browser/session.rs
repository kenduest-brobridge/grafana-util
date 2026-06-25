//! Shared read-only TUI browser for list/detail artifact inspection.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]
#[cfg(any(feature = "tui", test))]
pub(crate) use crate::interactive_browser_detail::{
    append_browser_detail_section, browser_detail_aligned_fact,
};
pub(crate) use crate::interactive_browser_detail::{
    browser_detail_fact, browser_detail_fallback_fact,
};
#[cfg(feature = "tui")]
pub(crate) use crate::interactive_browser_detail::{
    browser_detail_info_line, browser_detail_info_lines, browser_detail_info_lines_with,
    browser_review_empty_line, browser_review_info_lines, browser_wrapped_labeled_detail_lines,
    wrap_text_chunks,
};

#[path = "session/model.rs"]
mod model;
#[cfg(feature = "tui")]
#[path = "session/render.rs"]
mod render;
#[cfg(feature = "tui")]
#[path = "session/runtime.rs"]
mod runtime;
#[cfg(any(feature = "tui", test))]
#[path = "session/search.rs"]
mod search;

pub(crate) use model::BrowserItem;
#[cfg(test)]
pub(crate) use model::SearchDirection;
#[cfg(all(test, feature = "tui"))]
pub(crate) use render::detail_title;
#[cfg(feature = "tui")]
pub(crate) use runtime::run_interactive_browser;
#[cfg(test)]
pub(crate) use search::{
    build_search_state, find_match_in_visible, matching_visible_indexes, BrowserSearchController,
};

#[cfg(not(feature = "tui"))]
pub(crate) fn run_interactive_browser(
    _title: &str,
    _summary_lines: &[String],
    _items: &[BrowserItem],
) -> crate::common::Result<()> {
    Err(crate::common::tui_feature_required(
        "Shared interactive browser",
    ))
}

#[cfg(all(test, not(feature = "tui")))]
#[test]
fn run_interactive_browser_returns_tui_error_when_feature_disabled() {
    let error = run_interactive_browser(
        "Test",
        &[],
        &[BrowserItem {
            kind: "dashboard".to_string(),
            title: "Example".to_string(),
            meta: "meta".to_string(),
            details: vec!["detail".to_string()],
        }],
    )
    .expect_err("feature-disabled browser should return an error");

    assert_eq!(
        error.to_string(),
        "Shared interactive browser requires the `tui` feature."
    );
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
