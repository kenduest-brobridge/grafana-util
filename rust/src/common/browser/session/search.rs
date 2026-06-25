use super::model::{BrowserItem, SearchDirection, SearchState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SearchPromptState {
    pub(crate) direction: SearchDirection,
    pub(crate) query: String,
}

#[derive(Default)]
pub(crate) struct BrowserSearchController {
    pub(crate) pending: Option<SearchPromptState>,
    pub(crate) last: Option<SearchState>,
}

impl BrowserSearchController {
    pub(crate) fn start(&mut self, direction: SearchDirection) {
        self.pending = Some(SearchPromptState {
            direction,
            query: String::new(),
        });
    }

    pub(crate) fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(crate) fn push_char(&mut self, value: char) {
        if let Some(prompt) = self.pending.as_mut() {
            prompt.query.push(value);
        }
    }

    pub(crate) fn pop_char(&mut self) {
        if let Some(prompt) = self.pending.as_mut() {
            prompt.query.pop();
        }
    }

    pub(crate) fn cancel(&mut self) {
        self.pending = None;
    }

    pub(crate) fn apply(
        &mut self,
        items: &[BrowserItem],
        visible_indexes: &[usize],
        selected_visible: Option<usize>,
        filter_kind: &str,
    ) -> Option<usize> {
        let prompt = self.pending.take()?;
        let query = prompt.query.trim().to_string();
        if query.is_empty() {
            return None;
        }

        let matches = matching_visible_indexes(items, visible_indexes, &query);
        let selected = find_match_in_visible(
            items,
            visible_indexes,
            &query,
            prompt.direction,
            selected_visible,
        );
        self.last = Some(build_search_state(
            prompt.direction,
            query,
            filter_kind,
            &matches,
            selected,
        ));
        selected
    }

    pub(crate) fn repeat(
        &mut self,
        items: &[BrowserItem],
        visible_indexes: &[usize],
        selected_visible: Option<usize>,
        filter_kind: &str,
    ) -> Option<usize> {
        let last = self.last.as_ref()?.clone();
        let matches = matching_visible_indexes(items, visible_indexes, &last.query);
        let selected = repeat_match_in_visible(
            items,
            visible_indexes,
            &last.query,
            last.direction,
            selected_visible,
        );
        self.last = Some(build_search_state(
            last.direction,
            last.query,
            filter_kind,
            &matches,
            selected,
        ));
        selected
    }

    pub(crate) fn footer_label(&self) -> String {
        if let Some(prompt) = self.pending.as_ref() {
            format!(
                "Search {}{}",
                search_direction_symbol(prompt.direction),
                prompt.query
            )
        } else if let Some(last) = self.last.as_ref() {
            format!(
                "Last {}{}",
                search_direction_symbol(last.direction),
                last.query
            )
        } else {
            "Search idle".to_string()
        }
    }

    pub(crate) fn summary_line(&self, active_filter: &str) -> String {
        if let Some(prompt) = self.pending.as_ref() {
            return format!(
                "Search prompt {} in filter {}: \"{}\" (Enter search, Esc cancel).",
                search_direction_symbol(prompt.direction),
                active_filter,
                prompt.query
            );
        }
        if let Some(last) = self.last.as_ref() {
            return match last.match_ordinal {
                Some(ordinal) => format!(
                    "Last search {}\"{}\" in filter {} matched {}/{} results. Press n for next match.",
                    search_direction_symbol(last.direction),
                    last.query,
                    last.filter_kind,
                    ordinal,
                    last.match_count
                ),
                None => format!(
                    "Last search {}\"{}\" in filter {} matched 0 results. Press / or ? to try again.",
                    search_direction_symbol(last.direction),
                    last.query,
                    last.filter_kind
                ),
            };
        }
        "Search: / forward, ? backward, n repeat within the active filter.".to_string()
    }
}

pub(crate) fn repeat_match_in_visible(
    items: &[BrowserItem],
    visible_indexes: &[usize],
    query: &str,
    direction: SearchDirection,
    selected_visible: Option<usize>,
) -> Option<usize> {
    if visible_indexes.is_empty() || query.trim().is_empty() {
        return None;
    }

    match direction {
        SearchDirection::Forward => {
            let start = selected_visible.map(|index| index + 1).unwrap_or(0);
            (start..visible_indexes.len())
                .find(|visible_index| {
                    items
                        .get(visible_indexes[*visible_index])
                        .is_some_and(|item| item.matches_query(query))
                })
                .or_else(|| {
                    let wrap_end = selected_visible.unwrap_or(visible_indexes.len());
                    (0..wrap_end).find(|visible_index| {
                        items
                            .get(visible_indexes[*visible_index])
                            .is_some_and(|item| item.matches_query(query))
                    })
                })
        }
        SearchDirection::Backward => {
            let start = selected_visible
                .and_then(|index| index.checked_sub(1))
                .or_else(|| visible_indexes.len().checked_sub(1))?;
            (0..=start).rev().find(|visible_index| {
                items
                    .get(visible_indexes[*visible_index])
                    .is_some_and(|item| item.matches_query(query))
            })
        }
    }
}

pub(crate) fn search_direction_symbol(direction: SearchDirection) -> &'static str {
    match direction {
        SearchDirection::Forward => "/",
        SearchDirection::Backward => "?",
    }
}

pub(crate) fn matching_visible_indexes(
    items: &[BrowserItem],
    visible_indexes: &[usize],
    query: &str,
) -> Vec<usize> {
    visible_indexes
        .iter()
        .enumerate()
        .filter_map(|(visible_index, item_index)| {
            items
                .get(*item_index)
                .filter(|item| item.matches_query(query))
                .map(|_| visible_index)
        })
        .collect()
}

pub(crate) fn build_search_state(
    direction: SearchDirection,
    query: String,
    filter_kind: &str,
    matches: &[usize],
    selected: Option<usize>,
) -> SearchState {
    SearchState {
        direction,
        query,
        filter_kind: filter_kind.to_string(),
        match_ordinal: selected.and_then(|visible_index| {
            matches
                .iter()
                .position(|candidate| *candidate == visible_index)
                .map(|index| index + 1)
        }),
        match_count: matches.len(),
    }
}

pub(crate) fn find_match_in_visible(
    items: &[BrowserItem],
    visible_indexes: &[usize],
    query: &str,
    direction: SearchDirection,
    start: Option<usize>,
) -> Option<usize> {
    if visible_indexes.is_empty() || query.trim().is_empty() {
        return None;
    }

    match direction {
        SearchDirection::Forward => {
            let start = start
                .unwrap_or(0)
                .min(visible_indexes.len().saturating_sub(1));
            (start..visible_indexes.len()).find(|visible_index| {
                items
                    .get(visible_indexes[*visible_index])
                    .is_some_and(|item| item.matches_query(query))
            })
        }
        SearchDirection::Backward => {
            let start = start.unwrap_or(visible_indexes.len().saturating_sub(1));
            (0..=start.min(visible_indexes.len().saturating_sub(1)))
                .rev()
                .find(|visible_index| {
                    items
                        .get(visible_indexes[*visible_index])
                        .is_some_and(|item| item.matches_query(query))
                })
        }
    }
}
