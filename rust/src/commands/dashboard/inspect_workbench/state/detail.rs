#![cfg(feature = "tui")]

use crate::interactive_browser::BrowserItem;

pub(super) fn current_detail_lines(selected_item: Option<&BrowserItem>) -> Vec<String> {
    selected_item
        .map(|item| {
            if item.details.is_empty() {
                vec!["No facts available.".to_string()]
            } else {
                item.details.clone()
            }
        })
        .unwrap_or_else(|| vec!["No item selected.".to_string()])
}

pub(super) fn current_full_detail_lines(selected_item: Option<&BrowserItem>) -> Vec<String> {
    selected_item
        .map(|item| {
            let mut lines = vec![
                fact_line("Kind", &item.kind),
                fact_line("Title", &item.title),
            ];
            if !item.meta.is_empty() {
                lines.push(fact_line("Summary", &item.meta));
            }
            if !item.details.is_empty() {
                lines.push(String::new());
                lines.extend(item.details.clone());
            }
            lines
        })
        .unwrap_or_else(|| vec!["No item selected.".to_string()])
}

fn fact_line(label: &str, value: &str) -> String {
    format!("{label:<16}: {value}")
}
