#[cfg(feature = "tui")]
use super::detail_title;
use super::{
    append_browser_detail_section, browser_detail_aligned_fact, browser_detail_fact,
    browser_detail_fallback_fact, build_search_state, find_match_in_visible,
    matching_visible_indexes, BrowserItem, BrowserSearchController, SearchDirection,
};

fn sample_items() -> Vec<BrowserItem> {
    vec![
        BrowserItem {
            kind: "dashboard".to_string(),
            title: "CPU Overview".to_string(),
            meta: "folder=ops".to_string(),
            details: vec!["Prometheus datasource".to_string()],
        },
        BrowserItem {
            kind: "alert".to_string(),
            title: "Disk alert".to_string(),
            meta: "sev=high".to_string(),
            details: vec!["filesystem saturation".to_string()],
        },
        BrowserItem {
            kind: "dashboard".to_string(),
            title: "Memory Board".to_string(),
            meta: "folder=infra".to_string(),
            details: vec!["CPU and memory detail".to_string()],
        },
    ]
}

#[test]
fn browser_item_search_matches_kind_title_meta_and_details() {
    let item = BrowserItem {
        kind: "dashboard".to_string(),
        title: "CPU Overview".to_string(),
        meta: "folder=ops".to_string(),
        details: vec!["prometheus datasource".to_string()],
    };

    assert!(item.matches_query("dash"));
    assert!(item.matches_query("cpu"));
    assert!(item.matches_query("ops"));
    assert!(item.matches_query("datasource"));
    assert!(!item.matches_query("loki"));
}

#[test]
fn append_browser_detail_section_formats_empty_and_populated_sections() {
    let mut details = vec!["Node ID: dashboard:db".to_string()];

    append_browser_detail_section(&mut details, "Inbound edge summary", Vec::new());
    append_browser_detail_section(
        &mut details,
        "Outbound edge summary",
        vec!["  uses -> Env [variable]".to_string()],
    );

    assert_eq!(
        details,
        vec![
            "Node ID: dashboard:db".to_string(),
            "Inbound edge summary: none".to_string(),
            "Outbound edge summary:".to_string(),
            "  uses -> Env [variable]".to_string(),
        ]
    );
}

#[test]
fn browser_detail_fact_formats_label_value_rows() {
    assert_eq!(
        browser_detail_fact("Dashboard UID", "cpu-main"),
        "Dashboard UID: cpu-main"
    );
    assert_eq!(browser_detail_fact("Query Count", 3), "Query Count: 3");
}

#[test]
fn browser_detail_fallback_fact_trims_or_uses_fallback() {
    assert_eq!(
        browser_detail_fallback_fact("Org", " Main Org. ", "-"),
        "Org: Main Org."
    );
    assert_eq!(browser_detail_fallback_fact("UID", "  ", "-"), "UID: -");
}

#[cfg(feature = "tui")]
#[test]
fn browser_detail_info_line_formats_label_value_with_fallback() {
    assert_eq!(
        super::browser_detail_info_line("Login", "alice", "-").to_string(),
        "Login             : alice"
    );
    assert_eq!(
        super::browser_detail_info_line("Email", "", "-").to_string(),
        "Email             : -"
    );
}

#[cfg(feature = "tui")]
#[test]
fn browser_review_empty_line_formats_review_prefixed_message() {
    assert_eq!(
        super::browser_review_empty_line("No review evidence.").to_string(),
        "REVIEW No review evidence."
    );
}

#[cfg(feature = "tui")]
#[test]
fn browser_wrapped_labeled_detail_lines_preserve_prefix_width() {
    let lines = super::browser_wrapped_labeled_detail_lines("Summary", "abcdef", 16, 22, true)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        lines,
        vec!["Summary         : abcd", "                  ef"]
    );
}

#[test]
fn browser_detail_aligned_fact_formats_full_detail_rows() {
    assert_eq!(
        browser_detail_aligned_fact("Kind", "dashboard-summary"),
        "Kind            : dashboard-summary"
    );
    assert_eq!(
        browser_detail_aligned_fact("Summary", "uid=cpu-main"),
        "Summary         : uid=cpu-main"
    );
}

#[test]
fn search_uses_only_active_filter_visible_indexes() {
    let items = sample_items();
    let visible_indexes = vec![0, 2];

    let matches = matching_visible_indexes(&items, &visible_indexes, "disk");
    let selected = find_match_in_visible(
        &items,
        &visible_indexes,
        "cpu",
        SearchDirection::Forward,
        Some(0),
    );

    assert!(matches.is_empty());
    assert_eq!(selected, Some(0));
}

#[cfg(feature = "tui")]
#[test]
fn detail_title_uses_filtered_visible_position_and_total() {
    let items = sample_items();
    let visible_indexes = [0, 2];
    let selected_visible = 1;
    let item = &items[visible_indexes[selected_visible]];

    assert_eq!(
        detail_title(
            item,
            selected_visible,
            visible_indexes.len(),
            0,
            item.details.len()
        ),
        "Detail 2/2 [dashboard]  line 1/1"
    );
}

#[test]
fn repeat_search_advances_from_current_selection() {
    let items = sample_items();
    let visible_indexes = vec![0, 2];
    let mut search = BrowserSearchController::default();

    search.start(SearchDirection::Forward);
    search.push_char('c');
    search.push_char('p');
    search.push_char('u');

    let first = search.apply(&items, &visible_indexes, Some(0), "dashboard");
    let repeated = search.repeat(&items, &visible_indexes, first, "dashboard");

    assert_eq!(first, Some(0));
    assert_eq!(repeated, Some(1));
}

#[test]
fn repeat_search_wraps_forward_to_first_visible_match() {
    let items = sample_items();
    let visible_indexes = vec![0, 2];
    let mut search = BrowserSearchController::default();

    search.start(SearchDirection::Forward);
    search.push_char('c');
    search.push_char('p');
    search.push_char('u');
    let first = search.apply(&items, &visible_indexes, Some(1), "dashboard");
    let repeated = search.repeat(&items, &visible_indexes, first, "dashboard");

    assert_eq!(first, Some(1));
    assert_eq!(repeated, Some(0));
    assert_eq!(
        search.summary_line("dashboard"),
        "Last search /\"cpu\" in filter dashboard matched 1/2 results. Press n for next match."
    );
}

#[test]
fn repeat_search_wraps_backward_to_last_visible_match() {
    let items = sample_items();
    let visible_indexes = vec![0, 2];
    let mut search = BrowserSearchController::default();

    search.start(SearchDirection::Backward);
    search.push_char('c');
    search.push_char('p');
    search.push_char('u');
    let first = search.apply(&items, &visible_indexes, Some(0), "dashboard");
    let repeated = search.repeat(&items, &visible_indexes, first, "dashboard");

    assert_eq!(first, Some(0));
    assert_eq!(repeated, Some(1));
    assert_eq!(
        search.summary_line("dashboard"),
        "Last search ?\"cpu\" in filter dashboard matched 2/2 results. Press n for next match."
    );
}

#[test]
fn cancel_search_prompt_preserves_last_search_state() {
    let items = sample_items();
    let visible_indexes = vec![0, 2];
    let mut search = BrowserSearchController::default();

    search.start(SearchDirection::Forward);
    search.push_char('c');
    search.push_char('p');
    search.push_char('u');
    let applied = search.apply(&items, &visible_indexes, Some(0), "dashboard");
    assert_eq!(applied, Some(0));

    search.start(SearchDirection::Backward);
    search.push_char('d');
    search.cancel();

    assert_eq!(
        search.summary_line("dashboard"),
        "Last search /\"cpu\" in filter dashboard matched 1/2 results. Press n for next match."
    );
}

#[test]
fn search_summary_reports_no_matches() {
    let state = build_search_state(
        SearchDirection::Backward,
        "missing".to_string(),
        "alert",
        &[],
        None,
    );
    let search = BrowserSearchController {
        pending: None,
        last: Some(state),
    };

    assert_eq!(
        search.summary_line("dashboard"),
        "Last search ?\"missing\" in filter alert matched 0 results. Press / or ? to try again."
    );
}

#[test]
fn pending_search_summary_uses_compact_prompt_hints_without_repeat_key() {
    let mut search = BrowserSearchController::default();

    search.start(SearchDirection::Backward);
    search.push_char('c');
    search.push_char('p');
    search.push_char('u');

    let summary = search.summary_line("dashboard");

    assert!(summary.contains("Search prompt ? in filter dashboard"));
    assert!(summary.contains("Enter search"));
    assert!(summary.contains("Esc cancel"));
    assert!(!summary.contains("Enter apply"));
    assert!(!summary.contains("n repeat"));
}
