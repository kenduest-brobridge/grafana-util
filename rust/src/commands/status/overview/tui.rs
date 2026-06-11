#![cfg(feature = "tui")]
#![cfg_attr(test, allow(dead_code))]

#[path = "tui_render.rs"]
mod overview_tui_render;
#[cfg(feature = "tui")]
#[path = "tui_runtime.rs"]
mod overview_tui_runtime;

use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

use super::{OverviewDocument, OverviewSection, OverviewSectionItem, OverviewSectionView};

#[cfg(feature = "tui")]
pub(crate) use overview_tui_runtime::run_overview_interactive;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverviewPane {
    ProjectHome,
    Sections,
    Views,
    Items,
    Details,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchPromptState {
    direction: SearchDirection,
    query: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchState {
    direction: SearchDirection,
    query: String,
}

struct OverviewWorkbenchState {
    document: OverviewDocument,
    section_state: ListState,
    view_state: ListState,
    item_state: ListState,
    focus: OverviewPane,
    detail_scroll: u16,
    section_view_indexes: Vec<usize>,
    pending_search: Option<SearchPromptState>,
    last_search: Option<SearchState>,
    search_status: String,
}

impl OverviewWorkbenchState {
    fn new(document: OverviewDocument) -> Self {
        let mut section_state = ListState::default();
        let selected_section = document
            .selected_section_index
            .min(document.sections.len().saturating_sub(1));
        section_state.select((!document.sections.is_empty()).then_some(selected_section));
        let mut state = Self {
            section_view_indexes: vec![0; document.sections.len()],
            document,
            section_state,
            view_state: ListState::default(),
            item_state: ListState::default(),
            focus: OverviewPane::Items,
            detail_scroll: 0,
            pending_search: None,
            last_search: None,
            search_status: "Search idle. Use / or ? within the current view items.".to_string(),
        };
        state.sync_view_selection();
        state.reset_items();
        state
    }

    fn current_section_index(&self) -> Option<usize> {
        self.section_state.selected()
    }

    fn current_section(&self) -> Option<&OverviewSection> {
        self.current_section_index()
            .and_then(|index| self.document.sections.get(index))
    }

    fn current_view_index(&self) -> Option<usize> {
        self.view_state.selected()
    }

    fn current_view(&self) -> Option<&OverviewSectionView> {
        self.current_section().and_then(|section| {
            self.current_view_index()
                .and_then(|index| section.views.get(index))
        })
    }

    fn current_view_label(&self) -> String {
        self.current_view()
            .map(|view| view.label.clone())
            .unwrap_or_else(|| "No view selected".to_string())
    }

    fn section_index_for_domain(&self, domain_id: &str) -> Option<usize> {
        let matches_kind = |predicate: fn(&str) -> bool| {
            self.document
                .sections
                .iter()
                .position(|section| predicate(section.kind.as_str()))
        };
        match domain_id {
            "dashboard" => matches_kind(|kind| kind == "dashboard-export"),
            "datasource" => matches_kind(|kind| kind == "datasource-export"),
            "alert" => matches_kind(|kind| kind == "alert-export"),
            "access" => matches_kind(|kind| kind.starts_with("grafana-utils-access-")),
            "sync" => matches_kind(|kind| kind == "sync-summary" || kind == "bundle-preflight"),
            "promotion" => matches_kind(|kind| kind == "promotion-preflight"),
            _ => None,
        }
    }

    fn project_home_target_section_index(&self) -> Option<usize> {
        self.document
            .project_status
            .domains
            .iter()
            .find_map(|domain| {
                let actionable = domain.blocker_count > 0 || domain.status == "blocked";
                if actionable {
                    self.section_index_for_domain(&domain.id)
                } else {
                    None
                }
            })
            .or_else(|| {
                self.document
                    .project_status
                    .domains
                    .iter()
                    .find_map(|domain| {
                        if !domain.next_actions.is_empty() {
                            self.section_index_for_domain(&domain.id)
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| {
                if self.document.sections.is_empty() {
                    None
                } else {
                    Some(
                        self.section_state.selected().unwrap_or(
                            self.document
                                .selected_section_index
                                .min(self.document.sections.len().saturating_sub(1)),
                        ),
                    )
                }
            })
    }

    fn project_home_target_label(&self) -> Option<String> {
        self.project_home_target_section_index().and_then(|index| {
            self.document
                .sections
                .get(index)
                .map(|section| section.label.clone())
        })
    }

    fn project_home_domain(&self) -> Option<&crate::project_status::ProjectDomainStatus> {
        self.document.project_status.domains.iter().find(|domain| {
            domain.blocker_count > 0
                || domain.status == "blocked"
                || !domain.next_actions.is_empty()
        })
    }

    fn project_home_lines(&self) -> Vec<String> {
        let overall = &self.document.project_status.overall;
        let mut lines = vec![
            format!(
                "Overall: status={} scope={} domains={} present={} blocked={} blockers={} warnings={}",
                overall.status,
                self.document.project_status.scope,
                overall.domain_count,
                overall.present_count,
                overall.blocked_count,
                overall.blocker_count,
                overall.warning_count
            ),
            match self.project_home_target_label() {
                Some(label) => format!(
                    "Recommended handoff section: {label} | Project Home -> Sections -> Views -> Items -> Details"
                ),
                None => {
                    "Recommended handoff section: none | Project Home -> Sections -> Views -> Items -> Details"
                        .to_string()
                }
            },
        ];

        if let Some(domain) = self.project_home_domain() {
            let mut line = format!(
                "Top action: {} status={} reason={} primary={} blockers={} warnings={}",
                domain.id,
                domain.status,
                domain.reason_code,
                domain.primary_count,
                domain.blocker_count,
                domain.warning_count
            );
            if let Some(action) = domain.next_actions.first() {
                line.push_str(&format!(" next={action}"));
            }
            lines.push(line);
        } else {
            lines.push("Top action: no blocked or actionable domains".to_string());
        }

        lines.push(format!(
            "Domains: {}",
            self.document
                .project_status
                .domains
                .iter()
                .map(|domain| format!("{}={}", domain.id, domain.status))
                .collect::<Vec<String>>()
                .join(" | ")
        ));
        lines
    }

    fn status_focus_label(&self) -> &'static str {
        match self.focus {
            OverviewPane::ProjectHome => "Home",
            OverviewPane::Sections => "Sections",
            OverviewPane::Views => "Views",
            OverviewPane::Items => "Items",
            OverviewPane::Details => "Details",
        }
    }

    fn current_items(&self) -> &[OverviewSectionItem] {
        self.current_view()
            .map(|view| view.items.as_slice())
            .unwrap_or(&[])
    }

    fn search_summary(&self) -> String {
        if let Some(search) = &self.pending_search {
            let prefix = match search.direction {
                SearchDirection::Forward => "/",
                SearchDirection::Backward => "?",
            };
            return format!("prompt {prefix}{}", search.query);
        }
        match &self.last_search {
            Some(search) => {
                let prefix = match search.direction {
                    SearchDirection::Forward => "/",
                    SearchDirection::Backward => "?",
                };
                format!("last {prefix}{}", search.query)
            }
            None => "idle".to_string(),
        }
    }

    fn selected_item(&self) -> Option<&OverviewSectionItem> {
        self.item_state
            .selected()
            .and_then(|index| self.current_items().get(index))
    }

    fn current_detail_lines(&self) -> Vec<String> {
        self.selected_item()
            .map(|item| {
                let mut lines = vec![
                    format!("Kind: {}", item.kind),
                    format!("Title: {}", item.title),
                ];
                if !item.meta.is_empty() {
                    lines.push(format!("Summary: {}", item.meta));
                }
                if !item.facts.is_empty() {
                    lines.push(String::new());
                    lines.extend(
                        item.facts
                            .iter()
                            .map(|fact| format!("{}: {}", fact.label, fact.value)),
                    );
                }
                if !item.details.is_empty() {
                    lines.push(String::new());
                    let summary_line = format!("Summary: {}", item.meta);
                    lines.extend(
                        item.details
                            .iter()
                            .filter(|line| line.as_str() != summary_line)
                            .cloned(),
                    );
                }
                if lines.len() == 2 {
                    lines.push("No detail lines available.".to_string());
                }
                lines
            })
            .unwrap_or_else(|| vec!["No item selected.".to_string()])
    }

    fn sync_view_selection(&mut self) {
        let Some(section_index) = self.current_section_index() else {
            self.view_state.select(None);
            return;
        };
        let Some(section) = self.document.sections.get(section_index) else {
            self.view_state.select(None);
            return;
        };
        if section.views.is_empty() {
            self.view_state.select(None);
            return;
        }
        let view_index = self
            .section_view_indexes
            .get(section_index)
            .copied()
            .unwrap_or(0)
            .min(section.views.len().saturating_sub(1));
        self.view_state.select(Some(view_index));
        if let Some(slot) = self.section_view_indexes.get_mut(section_index) {
            *slot = view_index;
        }
    }

    fn reset_items(&mut self) {
        self.item_state
            .select((!self.current_items().is_empty()).then_some(0));
        self.detail_scroll = 0;
    }

    fn focus_next(&mut self) {
        self.focus = match self.focus {
            OverviewPane::ProjectHome => OverviewPane::Sections,
            OverviewPane::Sections => OverviewPane::Views,
            OverviewPane::Views => OverviewPane::Items,
            OverviewPane::Items => OverviewPane::Details,
            OverviewPane::Details => OverviewPane::ProjectHome,
        };
    }

    fn focus_previous(&mut self) {
        self.focus = match self.focus {
            OverviewPane::ProjectHome => OverviewPane::Details,
            OverviewPane::Sections => OverviewPane::ProjectHome,
            OverviewPane::Views => OverviewPane::Sections,
            OverviewPane::Items => OverviewPane::Views,
            OverviewPane::Details => OverviewPane::Items,
        };
    }

    fn focus_project_home(&mut self) {
        self.focus = OverviewPane::ProjectHome;
    }

    fn handoff_from_home(&mut self) {
        let Some(section_index) = self.project_home_target_section_index() else {
            return;
        };
        self.section_state.select(Some(section_index));
        self.sync_view_selection();
        self.reset_items();
        self.focus = OverviewPane::Sections;
    }

    fn move_section_selection(&mut self, delta: isize) {
        let count = self.document.sections.len();
        if count == 0 {
            self.section_state.select(None);
            self.view_state.select(None);
            self.item_state.select(None);
            self.detail_scroll = 0;
            return;
        }
        let current = self.section_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, count.saturating_sub(1) as isize) as usize;
        self.section_state.select(Some(next));
        self.sync_view_selection();
        self.reset_items();
    }

    fn move_view_selection(&mut self, delta: isize) {
        let Some(section_index) = self.current_section_index() else {
            self.view_state.select(None);
            self.item_state.select(None);
            self.detail_scroll = 0;
            return;
        };
        let Some(section) = self.document.sections.get(section_index) else {
            self.view_state.select(None);
            self.item_state.select(None);
            self.detail_scroll = 0;
            return;
        };
        if section.views.is_empty() {
            self.view_state.select(None);
            self.item_state.select(None);
            self.detail_scroll = 0;
            return;
        }
        let current = self.view_state.selected().unwrap_or(0) as isize;
        let next =
            (current + delta).clamp(0, section.views.len().saturating_sub(1) as isize) as usize;
        self.view_state.select(Some(next));
        if let Some(slot) = self.section_view_indexes.get_mut(section_index) {
            *slot = next;
        }
        self.reset_items();
    }

    fn move_item_selection(&mut self, delta: isize) {
        let count = self.current_items().len();
        if count == 0 {
            self.item_state.select(None);
            self.detail_scroll = 0;
            return;
        }
        let current = self.item_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, count.saturating_sub(1) as isize) as usize;
        self.item_state.select(Some(next));
        self.detail_scroll = 0;
    }

    fn move_detail_scroll(&mut self, delta: isize, total_lines: usize) {
        let max_scroll = total_lines.saturating_sub(1) as u16;
        if delta.is_negative() {
            self.detail_scroll = self
                .detail_scroll
                .saturating_sub(delta.unsigned_abs() as u16);
        } else {
            self.detail_scroll = self.detail_scroll.saturating_add(delta as u16);
        }
        self.detail_scroll = self.detail_scroll.min(max_scroll);
    }

    fn start_search(&mut self, direction: SearchDirection) {
        self.pending_search = Some(SearchPromptState {
            direction,
            query: String::new(),
        });
        self.search_status = match direction {
            SearchDirection::Forward => "Search forward within the current view items.".to_string(),
            SearchDirection::Backward => {
                "Search backward within the current view items.".to_string()
            }
        };
    }

    fn cancel_search(&mut self) {
        self.pending_search = None;
        self.search_status = "Cancelled status overview search.".to_string();
    }

    fn handle_search_key(&mut self, key: KeyCode) {
        let Some(mut search) = self.pending_search.take() else {
            return;
        };
        match key {
            KeyCode::Esc => self.cancel_search(),
            KeyCode::Enter => {
                let query = search.query.trim().to_string();
                if query.is_empty() {
                    self.search_status = "Search query is empty.".to_string();
                } else if let Some(index) = self.find_match(&query, search.direction) {
                    self.item_state.select(Some(index));
                    self.detail_scroll = 0;
                    self.last_search = Some(SearchState {
                        direction: search.direction,
                        query: query.clone(),
                    });
                    self.search_status = format!(
                        "Matched '{query}' at item {} of {} in the current view.",
                        index + 1,
                        self.current_items().len()
                    );
                } else {
                    self.last_search = Some(SearchState {
                        direction: search.direction,
                        query: query.clone(),
                    });
                    self.search_status = format!("No current-view items matched '{query}'.");
                }
            }
            KeyCode::Backspace => {
                search.query.pop();
                self.pending_search = Some(search);
            }
            KeyCode::Char(ch) => {
                search.query.push(ch);
                self.pending_search = Some(search);
            }
            _ => {
                self.pending_search = Some(search);
            }
        }
    }

    fn repeat_search(&mut self) {
        let Some(search) = self.last_search.clone() else {
            self.search_status =
                "No previous status overview search. Use / or ? first.".to_string();
            return;
        };
        if let Some(index) = self.repeat_last_search() {
            self.item_state.select(Some(index));
            self.detail_scroll = 0;
            self.search_status = format!(
                "Next match for '{}' at item {} of {} in the current view.",
                search.query,
                index + 1,
                self.current_items().len()
            );
        } else {
            self.search_status = format!("No more current-view matches for '{}'.", search.query);
        }
    }

    fn find_match(&self, query: &str, direction: SearchDirection) -> Option<usize> {
        let anchor = match direction {
            SearchDirection::Forward => self.item_state.selected().unwrap_or(0),
            SearchDirection::Backward => self
                .item_state
                .selected()
                .unwrap_or_else(|| self.current_items().len().saturating_sub(1)),
        };
        self.find_match_from(query, direction, anchor, true)
    }

    fn repeat_last_search(&self) -> Option<usize> {
        let search = self.last_search.as_ref()?;
        let anchor = match search.direction {
            SearchDirection::Forward => self.item_state.selected().unwrap_or(0),
            SearchDirection::Backward => self
                .item_state
                .selected()
                .unwrap_or_else(|| self.current_items().len().saturating_sub(1)),
        };
        self.find_match_from(&search.query, search.direction, anchor, false)
    }

    fn find_match_from(
        &self,
        query: &str,
        direction: SearchDirection,
        anchor: usize,
        include_anchor: bool,
    ) -> Option<usize> {
        let needle = query.trim().to_ascii_lowercase();
        let item_count = self.current_items().len();
        if needle.is_empty() || item_count == 0 {
            return None;
        }
        let normalized_anchor = anchor.min(item_count.saturating_sub(1));
        let start_offset = usize::from(!include_anchor);

        (start_offset..item_count).find_map(|offset| {
            let index = match direction {
                SearchDirection::Forward => (normalized_anchor + offset) % item_count,
                SearchDirection::Backward => {
                    (normalized_anchor + item_count - (offset % item_count)) % item_count
                }
            };
            item_matches(&self.current_items()[index], &needle).then_some(index)
        })
    }
}

fn item_matches(item: &OverviewSectionItem, needle: &str) -> bool {
    item.kind.to_ascii_lowercase().contains(needle)
        || item.title.to_ascii_lowercase().contains(needle)
        || item.meta.to_ascii_lowercase().contains(needle)
        || item.facts.iter().any(|fact| {
            fact.label.to_ascii_lowercase().contains(needle)
                || fact.value.to_ascii_lowercase().contains(needle)
        })
        || item
            .details
            .iter()
            .any(|line| line.to_ascii_lowercase().contains(needle))
}

#[cfg(test)]
#[path = "tui_tests.rs"]
mod tests;
