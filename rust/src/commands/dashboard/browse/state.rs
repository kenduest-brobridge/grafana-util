//! State container for the dashboard browse TUI.
//!
//! `document` owns the current rendered tree snapshot while this module owns ephemeral UI state:
//! current row focus, fact scrolling, cached live detail fetches, and any pending modal dialog.
//! Selection is preserved across refreshes through a lightweight anchor so reloaded trees keep the
//! operator near the same org/folder/dashboard when possible.
#![cfg(feature = "tui")]
use std::collections::BTreeMap;

use ratatui::widgets::ListState;

use super::super::delete_support::DeletePlan;
use super::browse_edit_dialog::EditDialogState;
use super::browse_external_edit_dialog::{ExternalEditDialogState, ExternalEditErrorState};
use super::browse_history_dialog::HistoryDialogState;
use super::browse_support::{
    DashboardBrowseDocument, DashboardBrowseNode, DashboardBrowseNodeKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaneFocus {
    Tree,
    Facts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SearchPromptState {
    pub(crate) direction: SearchDirection,
    pub(crate) query: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SearchState {
    pub(crate) direction: SearchDirection,
    pub(crate) query: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SelectionAnchor {
    kind: DashboardBrowseNodeKind,
    uid: Option<String>,
    path: String,
    org_id: String,
}

pub(crate) struct CompletionNotice {
    pub(crate) title: String,
    pub(crate) body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BrowseSelectionTarget {
    pub(crate) kind: DashboardBrowseNodeKind,
    pub(crate) uid: Option<String>,
    pub(crate) title: String,
    pub(crate) path: String,
    pub(crate) org_id: String,
    pub(crate) org_name: String,
}

impl BrowseSelectionTarget {
    pub(crate) fn from_node(node: &DashboardBrowseNode) -> Option<Self> {
        if node.kind != DashboardBrowseNodeKind::Dashboard {
            return None;
        }
        node.uid.as_ref()?;
        Some(Self {
            kind: node.kind.clone(),
            uid: node.uid.clone(),
            title: node.title.clone(),
            path: node.path.clone(),
            org_id: node.org_id.clone(),
            org_name: node.org_name.clone(),
        })
    }

    pub(crate) fn key(&self) -> String {
        selection_key(
            self.kind.clone(),
            self.uid.as_deref(),
            &self.path,
            &self.org_id,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeleteReview {
    pub(crate) plan: DeletePlan,
    pub(crate) scope_node: Option<DashboardBrowseNode>,
    pub(crate) targets: Vec<BrowseSelectionTarget>,
}

impl DeleteReview {
    pub(crate) fn single(
        plan: DeletePlan,
        scope_node: Option<DashboardBrowseNode>,
        targets: Vec<BrowseSelectionTarget>,
    ) -> Self {
        Self {
            plan,
            scope_node,
            targets,
        }
    }

    pub(crate) fn is_batch(&self) -> bool {
        self.targets.len() > 1
    }
}

pub(crate) struct BrowserState {
    pub(crate) document: DashboardBrowseDocument,
    pub(crate) local_mode: bool,
    pub(crate) list_state: ListState,
    pub(crate) detail_scroll: u16,
    pub(crate) live_view_cache: BTreeMap<String, Vec<String>>,
    pub(crate) pending_delete: Option<DeleteReview>,
    selected_targets: BTreeMap<String, BrowseSelectionTarget>,
    pub(crate) pending_edit: Option<EditDialogState>,
    pub(crate) pending_external_edit: Option<ExternalEditDialogState>,
    pub(crate) pending_external_edit_error: Option<ExternalEditErrorState>,
    pub(crate) pending_history: Option<HistoryDialogState>,
    pub(crate) pending_search: Option<SearchPromptState>,
    pub(crate) last_search: Option<SearchState>,
    pub(crate) completion_notice: Option<CompletionNotice>,
    pub(crate) focus: PaneFocus,
    pub(crate) status: String,
}

impl BrowserState {
    #[cfg(test)]
    pub(crate) fn new(document: DashboardBrowseDocument) -> Self {
        Self::new_with_mode(document, false)
    }

    pub(crate) fn new_with_mode(document: DashboardBrowseDocument, local_mode: bool) -> Self {
        let mut list_state = ListState::default();
        list_state.select((!document.nodes.is_empty()).then_some(0));
        let status = if document.nodes.is_empty() {
            "No dashboards matched the current tree.".to_string()
        } else if local_mode {
            "Loaded local dashboard tree. Live actions are unavailable in browse mode.".to_string()
        } else {
            "Loaded dashboard tree. Use e for metadata edit, E for raw JSON review/apply, h for history, v for live details, and d/D for delete.".to_string()
        };
        Self {
            document,
            local_mode,
            list_state,
            detail_scroll: 0,
            live_view_cache: BTreeMap::new(),
            pending_delete: None,
            selected_targets: BTreeMap::new(),
            pending_edit: None,
            pending_external_edit: None,
            pending_external_edit_error: None,
            pending_history: None,
            pending_search: None,
            last_search: None,
            completion_notice: None,
            focus: PaneFocus::Tree,
            status,
        }
    }

    pub(crate) fn selected_node(&self) -> Option<&DashboardBrowseNode> {
        if self.document.nodes.is_empty() {
            None
        } else {
            let index = self
                .list_state
                .selected()
                .unwrap_or(0)
                .min(self.document.nodes.len().saturating_sub(1));
            self.document.nodes.get(index)
        }
    }

    pub(crate) fn selected_targets(&self) -> Vec<BrowseSelectionTarget> {
        self.selected_targets.values().cloned().collect()
    }

    pub(crate) fn selected_dashboard_targets(&self) -> Vec<BrowseSelectionTarget> {
        self.selected_targets
            .values()
            .filter(|target| target.kind == DashboardBrowseNodeKind::Dashboard)
            .cloned()
            .collect()
    }

    pub(crate) fn has_selected_targets(&self) -> bool {
        !self.selected_targets.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn is_node_selected(&self, node: &DashboardBrowseNode) -> bool {
        selection_key_for_node(node)
            .map(|key| self.selected_targets.contains_key(&key))
            .unwrap_or(false)
    }

    pub(crate) fn toggle_selected_node(&mut self) -> bool {
        let Some(node) = self.selected_node() else {
            return false;
        };
        let Some(target) = BrowseSelectionTarget::from_node(node) else {
            return false;
        };
        let key = target.key();
        self.pending_delete = None;
        if self.selected_targets.remove(&key).is_some() {
            false
        } else {
            self.selected_targets.insert(key, target);
            true
        }
    }

    pub(crate) fn replace_document(&mut self, document: DashboardBrowseDocument) {
        let anchor = self.selection_anchor();
        self.document = document;
        // Cached live details belong to the old tree snapshot and may be stale after a refresh.
        self.live_view_cache.clear();
        self.pending_delete = None;
        self.pending_history = None;
        self.pending_external_edit = None;
        self.pending_external_edit_error = None;
        self.pending_search = None;
        self.completion_notice = None;
        self.prune_selected_targets();
        // Restore the operator's position by identity first, then degrade to the containing folder.
        self.restore_selection(anchor.as_ref());
        self.detail_scroll = 0;
    }

    fn prune_selected_targets(&mut self) {
        let valid_keys = self
            .document
            .nodes
            .iter()
            .filter_map(selection_key_for_node)
            .collect::<std::collections::BTreeSet<_>>();
        self.selected_targets
            .retain(|key, _target| valid_keys.contains(key));
    }

    pub(crate) fn move_selection(&mut self, delta: isize) {
        if self.document.nodes.is_empty() {
            self.list_state.select(None);
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, self.document.nodes.len().saturating_sub(1) as isize)
            as usize;
        self.list_state.select(Some(next));
    }

    pub(crate) fn select_first(&mut self) {
        self.list_state
            .select((!self.document.nodes.is_empty()).then_some(0));
    }

    pub(crate) fn select_last(&mut self) {
        self.list_state
            .select(self.document.nodes.len().checked_sub(1));
    }

    pub(crate) fn focus_next_pane(&mut self) {
        self.focus = match self.focus {
            PaneFocus::Tree => PaneFocus::Facts,
            PaneFocus::Facts => PaneFocus::Tree,
        };
    }

    pub(crate) fn focus_previous_pane(&mut self) {
        self.focus = match self.focus {
            PaneFocus::Tree => PaneFocus::Facts,
            PaneFocus::Facts => PaneFocus::Tree,
        };
    }

    pub(crate) fn focus_label(&self) -> &'static str {
        match self.focus {
            PaneFocus::Tree => "tree",
            PaneFocus::Facts => "facts",
        }
    }

    #[cfg(test)]
    pub(crate) fn detail_line_count(&self) -> usize {
        self.selected_detail_line_count().max(1)
    }

    pub(crate) fn move_detail_scroll(&mut self, delta: isize) {
        if !matches!(self.focus, PaneFocus::Facts) {
            return;
        }
        let max_scroll = self.max_detail_scroll();
        let next = (self.detail_scroll as isize + delta).clamp(0, max_scroll as isize);
        self.detail_scroll = next as u16;
    }

    pub(crate) fn scroll_detail_to_end(&mut self) {
        if !matches!(self.focus, PaneFocus::Facts) {
            self.detail_scroll = 0;
            return;
        }
        self.detail_scroll = self.max_detail_scroll().min(u16::MAX as usize) as u16;
    }

    pub(crate) fn selected_position_summary(&self) -> String {
        match (self.list_state.selected(), self.document.nodes.len()) {
            (Some(index), total) if total > 0 => {
                format!("{}/{}", index.saturating_add(1).min(total), total)
            }
            _ => "0/0".to_string(),
        }
    }

    pub(crate) fn selected_kind_summary(&self) -> &'static str {
        self.selected_node()
            .map(|node| match node.kind {
                DashboardBrowseNodeKind::Org => "org",
                DashboardBrowseNodeKind::Folder => "folder",
                DashboardBrowseNodeKind::Dashboard => "dashboard",
            })
            .unwrap_or("none")
    }

    pub(crate) fn search_summary(&self) -> String {
        if let Some(search) = self.pending_search.as_ref() {
            let prefix = match search.direction {
                SearchDirection::Forward => "/",
                SearchDirection::Backward => "?",
            };
            format!("prompt {prefix}{}", search.query)
        } else if let Some(search) = self.last_search.as_ref() {
            let prefix = match search.direction {
                SearchDirection::Forward => "/",
                SearchDirection::Backward => "?",
            };
            format!("last {prefix}{}", search.query)
        } else {
            "idle".to_string()
        }
    }

    pub(crate) fn mode_summary(&self) -> &'static str {
        if self.pending_delete.is_some() {
            "confirm-delete"
        } else if self.pending_edit.is_some() {
            "edit"
        } else if self.pending_external_edit.is_some() {
            "raw-edit"
        } else if self.pending_external_edit_error.is_some() {
            "raw-edit-error"
        } else if self.pending_history.is_some() {
            "history"
        } else if self.pending_search.is_some() {
            "search"
        } else if self.local_mode {
            "local-browse"
        } else {
            "browse"
        }
    }

    pub(crate) fn start_search(&mut self, direction: SearchDirection) {
        // Search is a transient modal layered over the current tree selection.
        self.pending_search = Some(SearchPromptState {
            direction,
            query: String::new(),
        });
        self.status = match direction {
            SearchDirection::Forward => "Search forward by org, folder, or dashboard.".to_string(),
            SearchDirection::Backward => {
                "Search backward by org, folder, or dashboard.".to_string()
            }
        };
    }

    pub(crate) fn select_index(&mut self, index: usize) {
        if index < self.document.nodes.len() {
            self.list_state.select(Some(index));
            self.detail_scroll = 0;
        }
    }

    pub(crate) fn find_match(&self, query: &str, direction: SearchDirection) -> Option<usize> {
        self.find_match_from(query, direction, self.list_state.selected())
    }

    pub(crate) fn repeat_last_search(&self) -> Option<usize> {
        let search = self.last_search.as_ref()?;
        match search.direction {
            SearchDirection::Forward => {
                let next_start = self
                    .list_state
                    .selected()
                    .map(|index| index.saturating_add(1));
                self.find_match_from(&search.query, search.direction, next_start)
                    .or_else(|| self.find_match_from(&search.query, search.direction, Some(0)))
            }
            SearchDirection::Backward => {
                if self.document.nodes.is_empty() {
                    return None;
                }
                let next_start = self
                    .list_state
                    .selected()
                    .and_then(|index| index.checked_sub(1))
                    .or_else(|| self.document.nodes.len().checked_sub(1));
                if let Some(index) =
                    self.find_match_from(&search.query, search.direction, next_start)
                {
                    return Some(index);
                }
                let wrapped_start = self.document.nodes.len().checked_sub(1)?;
                self.find_match_from(&search.query, search.direction, Some(wrapped_start))
            }
        }
    }

    fn selection_anchor(&self) -> Option<SelectionAnchor> {
        self.selected_node().map(|node| SelectionAnchor {
            kind: node.kind.clone(),
            uid: node.uid.clone(),
            path: node.path.clone(),
            org_id: node.org_id.clone(),
        })
    }

    fn restore_selection(&mut self, anchor: Option<&SelectionAnchor>) {
        let selected_index = anchor
            .and_then(|item| {
                self.document.nodes.iter().position(|node| {
                    node.kind == item.kind
                        && node.org_id == item.org_id
                        && match item.kind {
                            DashboardBrowseNodeKind::Org => node.title == item.path,
                            DashboardBrowseNodeKind::Dashboard => node.uid == item.uid,
                            DashboardBrowseNodeKind::Folder => node.path == item.path,
                        }
                })
            })
            .or_else(|| {
                // Dashboard rows can disappear across refreshes; fall back to the enclosing folder
                // before giving up and jumping to the top of the tree.
                anchor.and_then(|item| {
                    self.document.nodes.iter().position(|node| {
                        node.kind == DashboardBrowseNodeKind::Folder
                            && node.org_id == item.org_id
                            && node.path == item.path
                    })
                })
            })
            .or((!self.document.nodes.is_empty()).then_some(0));
        self.list_state.select(selected_index);
    }

    fn find_match_from(
        &self,
        query: &str,
        direction: SearchDirection,
        start: Option<usize>,
    ) -> Option<usize> {
        let needle = query.trim().to_ascii_lowercase();
        if needle.is_empty() || self.document.nodes.is_empty() {
            return None;
        }
        match direction {
            SearchDirection::Forward => {
                let start_index = start
                    .unwrap_or(0)
                    .min(self.document.nodes.len().saturating_sub(1));
                (start_index..self.document.nodes.len())
                    .find(|&index| node_matches(&self.document.nodes[index], &needle))
            }
            SearchDirection::Backward => {
                let start_index = start.unwrap_or(self.document.nodes.len().saturating_sub(1));
                (0..=start_index.min(self.document.nodes.len().saturating_sub(1)))
                    .rev()
                    .find(|&index| node_matches(&self.document.nodes[index], &needle))
            }
        }
    }

    fn selected_detail_line_count(&self) -> usize {
        self.selected_node()
            .map(|node| {
                if let Some(uid) = node.uid.as_ref() {
                    if let Some(lines) =
                        self.live_view_cache.get(&format!("{}::{uid}", node.org_id))
                    {
                        return lines.len();
                    }
                }
                node.details.len()
            })
            .unwrap_or(0)
    }

    fn max_detail_scroll(&self) -> usize {
        self.selected_detail_line_count().saturating_sub(1)
    }
}

fn selection_key_for_node(node: &DashboardBrowseNode) -> Option<String> {
    Some(selection_key(
        node.kind.clone(),
        node.uid.as_deref(),
        &node.path,
        &node.org_id,
    ))
    .filter(|_| BrowseSelectionTarget::from_node(node).is_some())
}

fn selection_key(
    kind: DashboardBrowseNodeKind,
    uid: Option<&str>,
    path: &str,
    org_id: &str,
) -> String {
    match kind {
        DashboardBrowseNodeKind::Dashboard => {
            format!("dashboard:{org_id}:{}", uid.unwrap_or_default())
        }
        DashboardBrowseNodeKind::Folder => format!("folder:{org_id}:{path}"),
        DashboardBrowseNodeKind::Org => format!("org:{org_id}:{path}"),
    }
}

fn node_matches(node: &DashboardBrowseNode, needle: &str) -> bool {
    // Search intentionally stays on stable identity/location fields so refreshed live metadata
    // does not change basic match semantics.
    [
        node.org_name.as_str(),
        node.title.as_str(),
        node.path.as_str(),
        node.uid.as_deref().unwrap_or(""),
    ]
    .iter()
    .any(|value| value.to_ascii_lowercase().contains(needle))
}

#[cfg(test)]
mod tests {
    use super::super::browse_support::DashboardBrowseSummary;
    use super::*;

    fn dashboard_node(uid: &str, title: &str, org_id: &str) -> DashboardBrowseNode {
        dashboard_node_with_details(uid, title, org_id, Vec::new())
    }

    fn dashboard_node_with_details(
        uid: &str,
        title: &str,
        org_id: &str,
        details: Vec<String>,
    ) -> DashboardBrowseNode {
        DashboardBrowseNode {
            kind: DashboardBrowseNodeKind::Dashboard,
            title: title.to_string(),
            path: "Platform".to_string(),
            uid: Some(uid.to_string()),
            depth: 1,
            meta: format!("uid={uid}"),
            details,
            url: None,
            org_name: format!("Org {org_id}"),
            org_id: org_id.to_string(),
            child_count: 0,
        }
    }

    fn folder_node() -> DashboardBrowseNode {
        DashboardBrowseNode {
            kind: DashboardBrowseNodeKind::Folder,
            title: "Platform".to_string(),
            path: "Platform".to_string(),
            uid: Some("platform".to_string()),
            depth: 0,
            meta: "0 folder(s) | 1 dashboard(s)".to_string(),
            details: Vec::new(),
            url: None,
            org_name: "Org 1".to_string(),
            org_id: "1".to_string(),
            child_count: 1,
        }
    }

    fn document(nodes: Vec<DashboardBrowseNode>) -> DashboardBrowseDocument {
        DashboardBrowseDocument {
            summary: DashboardBrowseSummary {
                root_path: None,
                dashboard_count: nodes
                    .iter()
                    .filter(|node| node.kind == DashboardBrowseNodeKind::Dashboard)
                    .count(),
                folder_count: nodes
                    .iter()
                    .filter(|node| node.kind == DashboardBrowseNodeKind::Folder)
                    .count(),
                org_count: 1,
                scope_label: "current-org".to_string(),
            },
            nodes,
        }
    }

    #[test]
    fn toggled_dashboard_selection_tracks_stable_target_identity() {
        let mut state = BrowserState::new(document(vec![
            folder_node(),
            dashboard_node("cpu-main", "CPU Main", "1"),
        ]));
        state.select_index(1);

        assert!(state.toggle_selected_node());

        let selected = state.selected_targets();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].uid.as_deref(), Some("cpu-main"));
        assert_eq!(selected[0].title, "CPU Main");
        assert_eq!(selected[0].path, "Platform");
        assert_eq!(selected[0].org_id, "1");
        assert!(state.is_node_selected(&state.document.nodes[1]));

        assert!(!state.toggle_selected_node());
        assert!(state.selected_targets().is_empty());
    }

    #[test]
    fn replace_document_prunes_selected_targets_missing_from_new_tree() {
        let mut state = BrowserState::new(document(vec![
            dashboard_node("cpu-main", "CPU Main", "1"),
            dashboard_node("memory-main", "Memory Main", "1"),
        ]));
        state.select_index(0);
        state.toggle_selected_node();
        state.select_index(1);
        state.toggle_selected_node();

        state.replace_document(document(vec![dashboard_node("cpu-main", "CPU Main", "1")]));

        let selected = state.selected_targets();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].uid.as_deref(), Some("cpu-main"));
    }

    #[test]
    fn repeat_last_search_wraps_backward_without_reselecting_boundary_match() {
        let mut state = BrowserState::new(document(vec![
            dashboard_node("cpu-main", "CPU Main", "1"),
            dashboard_node("disk-main", "Disk Main", "1"),
            dashboard_node("cpu-detail", "CPU Detail", "1"),
        ]));
        state.select_index(0);
        state.last_search = Some(SearchState {
            direction: SearchDirection::Backward,
            query: "cpu".to_string(),
        });

        assert_eq!(state.repeat_last_search(), Some(2));
    }

    #[test]
    fn detail_scroll_moves_and_clamps_within_detail_lines() {
        let mut state = BrowserState::new(document(vec![dashboard_node_with_details(
            "cpu-main",
            "CPU Main",
            "1",
            vec![
                "overview".to_string(),
                "metrics".to_string(),
                "alerts".to_string(),
            ],
        )]));
        state.focus_next_pane();
        state.select_index(0);

        state.move_detail_scroll(10);
        assert_eq!(state.detail_scroll, (state.detail_line_count() - 1) as u16);

        state.move_detail_scroll(-10);
        assert_eq!(state.detail_scroll, 0);
    }

    #[test]
    fn detail_scroll_end_targets_last_visible_line() {
        let mut state = BrowserState::new(document(vec![dashboard_node_with_details(
            "cpu-main",
            "CPU Main",
            "1",
            vec![
                "overview".to_string(),
                "metrics".to_string(),
                "alerts".to_string(),
            ],
        )]));
        state.focus_next_pane();
        state.select_index(0);
        state.scroll_detail_to_end();

        assert_eq!(state.detail_scroll, (state.detail_line_count() - 1) as u16);
    }

    #[test]
    fn selection_and_search_summaries_describe_current_tree_context() {
        let mut state = BrowserState::new(document(vec![
            folder_node(),
            dashboard_node("cpu-main", "CPU Main", "1"),
        ]));

        assert_eq!(state.selected_position_summary(), "1/2");
        assert_eq!(state.selected_kind_summary(), "folder");
        assert_eq!(state.search_summary(), "idle");

        state.select_last();
        state.last_search = Some(SearchState {
            direction: SearchDirection::Backward,
            query: "cpu".to_string(),
        });
        assert_eq!(state.selected_position_summary(), "2/2");
        assert_eq!(state.selected_kind_summary(), "dashboard");
        assert_eq!(state.search_summary(), "last ?cpu");

        state.start_search(SearchDirection::Forward);
        state.pending_search.as_mut().unwrap().query = "mem".to_string();
        assert_eq!(state.search_summary(), "prompt /mem");
    }

    #[test]
    fn selection_summaries_handle_empty_tree() {
        let state = BrowserState::new(document(Vec::new()));

        assert_eq!(state.selected_position_summary(), "0/0");
        assert_eq!(state.selected_kind_summary(), "none");
    }

    #[test]
    fn mode_summary_reports_search_and_local_browse_context() {
        let mut state = BrowserState::new_with_mode(document(vec![folder_node()]), true);

        assert_eq!(state.mode_summary(), "local-browse");

        state.start_search(SearchDirection::Forward);
        assert_eq!(state.mode_summary(), "search");
    }
}
