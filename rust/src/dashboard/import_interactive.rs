#![cfg(feature = "tui")]
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph, Wrap};
use reqwest::Method;
use serde_json::Value;

use crate::common::{message, string_field, value_as_object, Result};
use crate::tui_shell;

use super::browse_terminal::TerminalSession;
use super::build_preserved_web_import_document;
use super::import_lookup::{
    apply_folder_path_guard_to_action, build_folder_path_match_result,
    determine_dashboard_import_action_with_request,
    determine_import_folder_uid_override_with_request, fetch_dashboard_if_exists_cached,
    resolve_dashboard_import_folder_path_with_request,
    resolve_existing_dashboard_folder_path_with_request, resolve_source_dashboard_folder_path,
    ImportLookupCache,
};
use super::import_render::describe_dashboard_import_mode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InteractiveImportItem {
    pub(crate) path: PathBuf,
    pub(crate) uid: String,
    pub(crate) title: String,
    pub(crate) folder_path: String,
    pub(crate) file_label: String,
    pub(crate) review: InteractiveImportReviewState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveImportAction {
    Continue,
    Confirm(Vec<PathBuf>),
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveImportGrouping {
    Folder,
    Action,
    Flat,
}

impl InteractiveImportGrouping {
    fn next(self) -> Self {
        match self {
            Self::Folder => Self::Action,
            Self::Action => Self::Flat,
            Self::Flat => Self::Folder,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::Action => "action",
            Self::Flat => "flat",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InteractiveImportReview {
    pub(crate) action: String,
    pub(crate) destination: String,
    pub(crate) action_label: String,
    pub(crate) folder_path: String,
    pub(crate) source_folder_path: String,
    pub(crate) destination_folder_path: String,
    pub(crate) reason: String,
    pub(crate) diff_status: String,
    pub(crate) diff_summary_lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveImportReviewState {
    Pending,
    Resolved(InteractiveImportReview),
    Failed(String),
}

pub(crate) struct InteractiveImportState {
    pub(crate) items: Vec<InteractiveImportItem>,
    pub(crate) selected_paths: BTreeSet<PathBuf>,
    pub(crate) list_state: ListState,
    pub(crate) grouping: InteractiveImportGrouping,
    pub(crate) import_mode: String,
    pub(crate) status: String,
    review_on_focus: bool,
}

#[derive(Default)]
pub(crate) struct InteractiveImportSummaryCounts {
    pub(crate) total: usize,
    pub(crate) selected: usize,
    pub(crate) pending: usize,
    pub(crate) reviewed: usize,
    pub(crate) blocked: usize,
    pub(crate) create: usize,
    pub(crate) update: usize,
    pub(crate) skip_missing: usize,
    pub(crate) skip_folder: usize,
}

impl InteractiveImportState {
    pub(crate) fn new(items: Vec<InteractiveImportItem>, import_mode: String) -> Self {
        let mut list_state = ListState::default();
        list_state.select((!items.is_empty()).then_some(0));
        Self {
            items,
            selected_paths: BTreeSet::new(),
            list_state,
            grouping: InteractiveImportGrouping::Folder,
            import_mode,
            status: "Loaded local dashboards. Review follows focus; Enter imports the selected dashboards.".to_string(),
            review_on_focus: true,
        }
    }

    fn ordered_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.items.len()).collect();
        indices.sort_by_key(|index| self.sort_key(*index));
        indices
    }

    fn sort_key(&self, index: usize) -> (String, String, String, String) {
        let item = &self.items[index];
        match self.grouping {
            InteractiveImportGrouping::Folder => (
                item.folder_path.clone(),
                item.title.clone(),
                item.uid.clone(),
                item.file_label.clone(),
            ),
            InteractiveImportGrouping::Action => (
                self.action_group_title(item),
                item.folder_path.clone(),
                item.title.clone(),
                item.uid.clone(),
            ),
            InteractiveImportGrouping::Flat => (
                String::new(),
                item.title.clone(),
                item.uid.clone(),
                item.file_label.clone(),
            ),
        }
    }

    fn action_group_title(&self, item: &InteractiveImportItem) -> String {
        match &item.review {
            InteractiveImportReviewState::Pending => "Pending Review".to_string(),
            InteractiveImportReviewState::Failed(_) => "Blocked Review".to_string(),
            InteractiveImportReviewState::Resolved(review) => match review.action_label.as_str() {
                "create" => "Create".to_string(),
                "update" => "Update".to_string(),
                "skip-missing" => "Skip Missing".to_string(),
                "skip-folder-mismatch" => "Skip Folder Mismatch".to_string(),
                "blocked-existing" => "Blocked Existing".to_string(),
                _ => "Other".to_string(),
            },
        }
    }

    fn visible_count(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn review_summary_counts(&self) -> InteractiveImportSummaryCounts {
        let mut counts = InteractiveImportSummaryCounts {
            total: self.items.len(),
            selected: self.selected_paths.len(),
            ..InteractiveImportSummaryCounts::default()
        };
        for item in &self.items {
            match &item.review {
                InteractiveImportReviewState::Pending => counts.pending += 1,
                InteractiveImportReviewState::Failed(_) => counts.blocked += 1,
                InteractiveImportReviewState::Resolved(review) => {
                    match review.action_label.as_str() {
                        "create" => counts.create += 1,
                        "update" => counts.update += 1,
                        "skip-missing" => counts.skip_missing += 1,
                        "skip-folder-mismatch" => counts.skip_folder += 1,
                        "blocked-existing" => counts.blocked += 1,
                        _ => {}
                    }
                }
            }
        }
        counts.reviewed = counts.total.saturating_sub(counts.pending);
        counts
    }

    fn focused_group_summary(&self) -> Option<String> {
        let focused = self.selected_item()?;
        let group_label = match self.grouping {
            InteractiveImportGrouping::Folder => {
                if focused.folder_path.is_empty() {
                    "General".to_string()
                } else {
                    focused.folder_path.clone()
                }
            }
            InteractiveImportGrouping::Action => self.action_group_title(focused),
            InteractiveImportGrouping::Flat => return None,
        };
        let mut item_count = 0usize;
        let mut selected_count = 0usize;
        let mut reviewed_count = 0usize;
        for item in &self.items {
            let same_group = match self.grouping {
                InteractiveImportGrouping::Folder => {
                    let label = if item.folder_path.is_empty() {
                        "General"
                    } else {
                        item.folder_path.as_str()
                    };
                    label == group_label
                }
                InteractiveImportGrouping::Action => self.action_group_title(item) == group_label,
                InteractiveImportGrouping::Flat => false,
            };
            if !same_group {
                continue;
            }
            item_count += 1;
            if self.selected_paths.contains(&item.path) {
                selected_count += 1;
            }
            if !matches!(item.review, InteractiveImportReviewState::Pending) {
                reviewed_count += 1;
            }
        }
        Some(format!(
            "Group={}   Items={}   Reviewed={}   Selected={}",
            group_label, item_count, reviewed_count, selected_count
        ))
    }

    pub(crate) fn selected_item(&self) -> Option<&InteractiveImportItem> {
        let visible_index = self.list_state.selected()?;
        let ordered = self.ordered_indices();
        let item_index = *ordered.get(visible_index)?;
        self.items.get(item_index)
    }

    fn selected_item_index(&self) -> Option<usize> {
        let visible_index = self.list_state.selected()?;
        let ordered = self.ordered_indices();
        ordered.get(visible_index).copied()
    }

    pub(crate) fn move_selection(&mut self, delta: isize) {
        let visible_count = self.visible_count();
        if visible_count == 0 {
            self.list_state.select(None);
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, visible_count.saturating_sub(1) as isize) as usize;
        self.list_state.select(Some(next));
        self.review_on_focus = true;
    }

    pub(crate) fn select_first(&mut self) {
        self.list_state
            .select((!self.items.is_empty()).then_some(0));
        self.review_on_focus = true;
    }

    pub(crate) fn select_last(&mut self) {
        self.list_state.select(self.items.len().checked_sub(1));
        self.review_on_focus = true;
    }

    pub(crate) fn toggle_selected(&mut self) {
        let Some(path) = self.selected_item().map(|item| item.path.clone()) else {
            return;
        };
        if !self.selected_paths.remove(&path) {
            self.selected_paths.insert(path);
        }
        self.status = format!("Selected {} dashboard(s).", self.selected_paths.len());
    }

    pub(crate) fn toggle_select_all(&mut self) {
        if self.selected_paths.len() == self.items.len() {
            self.selected_paths.clear();
            self.status = "Cleared dashboard selection.".to_string();
            return;
        }
        self.selected_paths = self.items.iter().map(|item| item.path.clone()).collect();
        self.status = format!("Selected all {} dashboard(s).", self.selected_paths.len());
    }

    pub(crate) fn cycle_grouping(&mut self) {
        let focused_path = self.selected_item().map(|item| item.path.clone());
        self.grouping = self.grouping.next();
        if let Some(path) = focused_path {
            self.select_path(&path);
        }
        self.status = format!(
            "Grouping is now {}. Review rows are still resolved on focus.",
            self.grouping.label()
        );
    }

    fn select_path(&mut self, path: &PathBuf) {
        let ordered = self.ordered_indices();
        let next_index = ordered
            .iter()
            .position(|item_index| self.items[*item_index].path == *path);
        self.list_state.select(next_index);
    }

    pub(crate) fn focus_needs_review(&self) -> bool {
        self.review_on_focus
    }

    pub(crate) fn mark_focus_reviewed(&mut self) {
        self.review_on_focus = false;
    }

    pub(crate) fn resolve_focused_review_with_request<F>(
        &mut self,
        request_json: &mut F,
        lookup_cache: &mut ImportLookupCache,
        args: &super::ImportArgs,
    ) where
        F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
    {
        let Some(item_index) = self.selected_item_index() else {
            self.mark_focus_reviewed();
            return;
        };
        if !matches!(
            self.items[item_index].review,
            InteractiveImportReviewState::Pending
        ) {
            self.mark_focus_reviewed();
            return;
        }
        let path = self.items[item_index].path.clone();
        let uid = self.items[item_index].uid.clone();
        let source_folder_path = self.items[item_index].folder_path.clone();
        let result = build_interactive_import_review_with_request(
            request_json,
            lookup_cache,
            args,
            &path,
            &uid,
            &source_folder_path,
        );
        let (status, review_state) = match result {
            Ok(review) => (
                format!(
                    "Reviewed {}: {} {}.",
                    uid, review.destination, review.action_label
                ),
                InteractiveImportReviewState::Resolved(review),
            ),
            Err(error) => (
                format!("Review blocked for {}: {}", uid, error),
                InteractiveImportReviewState::Failed(error.to_string()),
            ),
        };
        self.status = status;
        self.items[item_index].review = review_state;
        self.mark_focus_reviewed();
    }

    pub(crate) fn selected_files(&self) -> Vec<PathBuf> {
        self.items
            .iter()
            .filter(|item| self.selected_paths.contains(&item.path))
            .map(|item| item.path.clone())
            .collect()
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> InteractiveImportAction {
        match key.code {
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::Home => self.select_first(),
            KeyCode::End => self.select_last(),
            KeyCode::Char(' ') => self.toggle_selected(),
            KeyCode::Char('a') => self.toggle_select_all(),
            KeyCode::Char('g') => self.cycle_grouping(),
            KeyCode::Enter => {
                let files = self.selected_files();
                if files.is_empty() {
                    self.status = "Select at least one dashboard before importing.".to_string();
                } else {
                    return InteractiveImportAction::Confirm(files);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => return InteractiveImportAction::Cancel,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return InteractiveImportAction::Cancel
            }
            _ => {}
        }
        InteractiveImportAction::Continue
    }
}

pub(crate) fn select_import_dashboard_files<F>(
    request_json: &mut F,
    lookup_cache: &mut ImportLookupCache,
    args: &super::ImportArgs,
) -> Result<Option<Vec<PathBuf>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if !args.interactive {
        return Ok(None);
    }
    if args.use_export_org {
        return Err(message(
            "Dashboard import --interactive does not support --use-export-org yet.",
        ));
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(message("Dashboard import interactive mode requires a TTY."));
    }
    let (items, _folders_by_uid) = load_interactive_import_context(args)?;
    if items.is_empty() {
        return Err(message(format!(
            "No dashboard JSON files were found under {}.",
            args.import_dir.display()
        )));
    }
    run_import_selector(
        request_json,
        lookup_cache,
        args,
        args.import_dir.display().to_string(),
        items,
    )
}

fn run_import_selector<F>(
    request_json: &mut F,
    lookup_cache: &mut ImportLookupCache,
    args: &super::ImportArgs,
    import_dir_label: String,
    items: Vec<InteractiveImportItem>,
) -> Result<Option<Vec<PathBuf>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut session = TerminalSession::enter()?;
    session.terminal.hide_cursor()?;
    let mut state = InteractiveImportState::new(
        items,
        describe_dashboard_import_mode(args.replace_existing, args.update_existing_only)
            .to_string(),
    );
    loop {
        session.terminal.draw(|frame| {
            let size = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(size);
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(chunks[1]);
            let summary = state.review_summary_counts();

            let header = tui_shell::build_header(
                "Interactive Dashboard Import",
                vec![
                    Line::from(format!(
                        "Import dir={}   Mode={}   Grouping={}   Selected={}/{}",
                        import_dir_label,
                        state.import_mode,
                        state.grouping.label(),
                        state.selected_paths.len(),
                        state.items.len()
                    )),
                    Line::from(format!(
                        "Review={} pending={} create={} update={} skip-missing={} skip-folder={} blocked={} selected={}",
                        summary.reviewed,
                        summary.pending,
                        summary.create,
                        summary.update,
                        summary.skip_missing,
                        summary.skip_folder,
                        summary.blocked,
                        summary.selected
                    )),
                    Line::from(
                        state
                            .focused_group_summary()
                            .unwrap_or_else(|| "Flat grouping keeps one continuous dashboard list.".to_string()),
                    ),
                ],
            );
            frame.render_widget(header, chunks[0]);

            let ordered = state.ordered_indices();
            let list_items: Vec<ListItem> = {
                let grouping = state.grouping;
                let selected_paths = &state.selected_paths;
                let items = &state.items;
                ordered
                    .iter()
                    .enumerate()
                    .map(|(visible_index, item_index)| {
                        let item = &items[*item_index];
                        let marker = if selected_paths.contains(&item.path) {
                            "[x]"
                        } else {
                            "[ ]"
                        };
                        let folder = if item.folder_path.is_empty() {
                            "General"
                        } else {
                            item.folder_path.as_str()
                        };
                        let mut lines = Vec::new();
                        if grouping != InteractiveImportGrouping::Flat {
                            let current_group = match grouping {
                                InteractiveImportGrouping::Folder => {
                                    if item.folder_path.is_empty() {
                                        "General".to_string()
                                    } else {
                                        item.folder_path.clone()
                                    }
                                }
                                InteractiveImportGrouping::Action => state.action_group_title(item),
                                InteractiveImportGrouping::Flat => String::new(),
                            };
                            let previous_group = visible_index.checked_sub(1).map(|previous| {
                                let previous_item = &items[ordered[previous]];
                                match grouping {
                                    InteractiveImportGrouping::Folder => {
                                        if previous_item.folder_path.is_empty() {
                                            "General".to_string()
                                        } else {
                                            previous_item.folder_path.clone()
                                        }
                                    }
                                    InteractiveImportGrouping::Action => {
                                        state.action_group_title(previous_item)
                                    }
                                    InteractiveImportGrouping::Flat => String::new(),
                                }
                            });
                            if previous_group.as_deref() != Some(current_group.as_str()) {
                                lines.push(Line::from(Span::styled(
                                    format!(" {} ", current_group),
                                    Style::default()
                                        .fg(Color::Black)
                                        .bg(Color::Rgb(132, 146, 166))
                                        .add_modifier(Modifier::BOLD),
                                )));
                            }
                        }
                        lines.push(Line::from(vec![
                            Span::styled(review_badge(item), review_badge_style(item)),
                            Span::raw(" "),
                            Span::styled(marker, Style::default().fg(Color::Green)),
                            Span::raw(" "),
                            Span::styled(
                                item.title.as_str(),
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("uid ", Style::default().fg(Color::DarkGray)),
                            Span::raw(item.uid.as_str()),
                            Span::raw("  "),
                            Span::styled("folder ", Style::default().fg(Color::DarkGray)),
                            Span::raw(folder),
                        ]));
                        ListItem::new(lines)
                    })
                    .collect()
            };
            let list = List::new(list_items)
                .block(tui_shell::pane_block("Dashboards", true, Color::Cyan, Color::Reset))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightBlue)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            frame.render_stateful_widget(list, body[0], &mut state.list_state);

            let detail_lines = if let Some(item) = state.selected_item() {
                let mut lines = vec![
                    Line::from(Span::styled(
                        item.title.as_str(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("UID: ", Style::default().fg(Color::Yellow)),
                        Span::raw(item.uid.as_str()),
                    ]),
                    Line::from(vec![
                        Span::styled("File: ", Style::default().fg(Color::Yellow)),
                        Span::raw(item.file_label.as_str()),
                    ]),
                    Line::from(vec![
                        Span::styled("Source Folder: ", Style::default().fg(Color::Yellow)),
                        Span::raw(if item.folder_path.is_empty() {
                            "General"
                        } else {
                            item.folder_path.as_str()
                        }),
                    ]),
                    Line::from(vec![
                        Span::styled("Import Mode: ", Style::default().fg(Color::Yellow)),
                        Span::raw(state.import_mode.as_str()),
                    ]),
                    Line::from(""),
                ];
                match &item.review {
                    InteractiveImportReviewState::Pending => {
                        lines.push(Line::from(Span::styled(
                            "Review pending. Move focus here to resolve create/update/skip behavior.",
                            Style::default().fg(Color::Gray),
                        )));
                    }
                    InteractiveImportReviewState::Failed(error) => {
                        lines.push(Line::from(Span::styled(
                            "BLOCKED REVIEW",
                            Style::default()
                                .fg(Color::White)
                                .bg(Color::DarkGray)
                                .add_modifier(Modifier::BOLD),
                        )));
                        lines.push(Line::from(""));
                        lines.push(Line::from(error.as_str()));
                    }
                    InteractiveImportReviewState::Resolved(review) => {
                        lines.push(Line::from(vec![
                            Span::styled("Review: ", Style::default().fg(Color::Yellow)),
                            Span::raw(format!(
                                "{} {}",
                                review.destination, review.action_label
                            )),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("Target Folder: ", Style::default().fg(Color::Yellow)),
                            Span::raw(review.folder_path.as_str()),
                        ]));
                        if !review.destination_folder_path.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "Existing Folder: ",
                                    Style::default().fg(Color::Yellow),
                                ),
                                Span::raw(review.destination_folder_path.as_str()),
                            ]));
                        }
                        if !review.reason.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("Reason: ", Style::default().fg(Color::Yellow)),
                                Span::raw(review.reason.as_str()),
                            ]));
                        }
                        lines.push(Line::from(vec![
                            Span::styled("Live Diff: ", Style::default().fg(Color::Yellow)),
                            Span::raw(review.diff_status.as_str()),
                        ]));
                        for diff_line in &review.diff_summary_lines {
                            lines.push(Line::from(diff_line.as_str()));
                        }
                    }
                }
                lines
            } else {
                vec![Line::from("No dashboard selected.")]
            };
            let detail = Paragraph::new(detail_lines)
                .block(tui_shell::pane_block("Review", false, Color::Yellow, Color::Reset))
                .wrap(Wrap { trim: false });
            frame.render_widget(detail, body[1]);

            let footer = tui_shell::build_footer(
                vec![
                    tui_shell::control_line(&[
                        ("Up/Down", Color::Rgb(24, 78, 140), "move"),
                        ("Space", Color::Rgb(24, 106, 59), "toggle"),
                        ("a", Color::Rgb(24, 106, 59), "all/none"),
                        ("g", Color::Rgb(164, 116, 19), "grouping"),
                        ("Enter", Color::Rgb(24, 106, 59), "import selected"),
                        ("q", Color::Rgb(90, 98, 107), "cancel"),
                    ]),
                ],
                state.status.as_str(),
            );
            frame.render_widget(Clear, chunks[2]);
            frame.render_widget(footer, chunks[2]);
        })?;

        if state.focus_needs_review() {
            state.resolve_focused_review_with_request(request_json, lookup_cache, args);
            continue;
        }
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match state.handle_key(key) {
            InteractiveImportAction::Continue => {}
            InteractiveImportAction::Confirm(files) => return Ok(Some(files)),
            InteractiveImportAction::Cancel => return Ok(None),
        }
    }
}

fn review_badge(item: &InteractiveImportItem) -> &'static str {
    match &item.review {
        InteractiveImportReviewState::Pending => "PENDING",
        InteractiveImportReviewState::Failed(_) => "BLOCKED",
        InteractiveImportReviewState::Resolved(review) => match review.action_label.as_str() {
            "create" => "CREATE",
            "update" => "UPDATE",
            "skip-missing" => "SKIP-MISSING",
            "skip-folder-mismatch" => "SKIP-FOLDER",
            "blocked-existing" => "BLOCKED",
            _ => "REVIEWED",
        },
    }
}

fn review_badge_style(item: &InteractiveImportItem) -> Style {
    match &item.review {
        InteractiveImportReviewState::Pending => Style::default().fg(Color::Black).bg(Color::Gray),
        InteractiveImportReviewState::Failed(_) => {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        }
        InteractiveImportReviewState::Resolved(review) => match review.action_label.as_str() {
            "create" => Style::default().fg(Color::Black).bg(Color::Green),
            "update" => Style::default().fg(Color::Black).bg(Color::Yellow),
            "skip-missing" | "skip-folder-mismatch" => {
                Style::default().fg(Color::Black).bg(Color::LightBlue)
            }
            "blocked-existing" => Style::default().fg(Color::White).bg(Color::Red),
            _ => Style::default().fg(Color::White).bg(Color::DarkGray),
        },
    }
}

#[cfg(test)]
pub(crate) fn load_interactive_import_items(
    args: &super::ImportArgs,
) -> Result<Vec<InteractiveImportItem>> {
    Ok(load_interactive_import_context(args)?.0)
}

fn load_interactive_import_context(
    args: &super::ImportArgs,
) -> Result<(
    Vec<InteractiveImportItem>,
    BTreeMap<String, super::FolderInventoryItem>,
)> {
    let metadata = super::load_export_metadata(&args.import_dir, Some(super::RAW_EXPORT_SUBDIR))?;
    let folder_inventory = super::load_folder_inventory(&args.import_dir, metadata.as_ref())?;
    let folders_by_uid: BTreeMap<String, super::FolderInventoryItem> = folder_inventory
        .into_iter()
        .map(|item| (item.uid.clone(), item))
        .collect();
    let mut items = Vec::new();
    for path in super::import::dashboard_files_for_import(&args.import_dir)? {
        let document = super::load_json_file(&path)?;
        let document_object =
            value_as_object(&document, "Dashboard payload must be a JSON object.")?;
        let dashboard = super::extract_dashboard_object(document_object)?;
        let uid = string_field(dashboard, "uid", super::DEFAULT_UNKNOWN_UID).to_string();
        let title = string_field(dashboard, "title", super::DEFAULT_DASHBOARD_TITLE).to_string();
        let folder_path = resolve_source_dashboard_folder_path(
            &document,
            &path,
            &args.import_dir,
            &folders_by_uid,
        )
        .unwrap_or_default();
        let file_label = path
            .strip_prefix(&args.import_dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        items.push(InteractiveImportItem {
            path,
            uid,
            title,
            folder_path,
            file_label,
            review: InteractiveImportReviewState::Pending,
        });
    }
    items.sort_by(|left, right| {
        (
            left.folder_path.as_str(),
            left.title.as_str(),
            left.uid.as_str(),
        )
            .cmp(&(
                right.folder_path.as_str(),
                right.title.as_str(),
                right.uid.as_str(),
            ))
    });
    Ok((items, folders_by_uid))
}

fn build_interactive_import_review_with_request<F>(
    request_json: &mut F,
    lookup_cache: &mut ImportLookupCache,
    args: &super::ImportArgs,
    dashboard_file: &std::path::Path,
    uid: &str,
    source_folder_path: &str,
) -> Result<InteractiveImportReview>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let document = super::load_json_file(dashboard_file)?;
    if args.strict_schema {
        super::validate::validate_dashboard_import_document(
            &document,
            dashboard_file,
            true,
            args.target_schema_version,
        )?;
    }
    let metadata = super::load_export_metadata(&args.import_dir, Some(super::RAW_EXPORT_SUBDIR))?;
    let folder_inventory = super::load_folder_inventory(&args.import_dir, metadata.as_ref())?;
    let folders_by_uid: BTreeMap<String, super::FolderInventoryItem> = folder_inventory
        .into_iter()
        .map(|item| (item.uid.clone(), item))
        .collect();
    let effective_replace_existing = args.replace_existing || args.update_existing_only;
    let folder_uid_override = determine_import_folder_uid_override_with_request(
        request_json,
        lookup_cache,
        uid,
        args.import_folder_uid.as_deref(),
        effective_replace_existing,
    )?;
    let payload = super::build_import_payload(
        &document,
        folder_uid_override.as_deref(),
        effective_replace_existing,
        &args.import_message,
    )?;
    let action = determine_dashboard_import_action_with_request(
        request_json,
        lookup_cache,
        &payload,
        args.replace_existing,
        args.update_existing_only,
    )?;
    let destination_folder_path = if args.require_matching_folder_path {
        resolve_existing_dashboard_folder_path_with_request(request_json, lookup_cache, uid)?
    } else {
        None
    };
    let (
        folder_paths_match,
        reason,
        normalized_source_folder_path,
        normalized_destination_folder_path,
    ) = if args.require_matching_folder_path {
        build_folder_path_match_result(
            Some(source_folder_path),
            destination_folder_path.as_deref(),
            destination_folder_path.is_some(),
            true,
        )
    } else {
        (true, "", source_folder_path.to_string(), None::<String>)
    };
    let action = apply_folder_path_guard_to_action(action, folder_paths_match);
    let prefer_live_folder_path =
        folder_uid_override.is_some() && args.import_folder_uid.is_none() && !uid.is_empty();
    let folder_path = resolve_dashboard_import_folder_path_with_request(
        request_json,
        lookup_cache,
        &payload,
        &folders_by_uid,
        prefer_live_folder_path,
    )?;
    let (destination, action_label) = match action {
        "would-create" => ("missing", "create"),
        "would-update" => ("exists", "update"),
        "would-skip-missing" => ("missing", "skip-missing"),
        "would-skip-folder-mismatch" => ("exists", "skip-folder-mismatch"),
        "would-fail-existing" => ("exists", "blocked-existing"),
        _ => ("unknown", action),
    };
    let (diff_status, diff_summary_lines) = build_interactive_import_diff_summary_with_request(
        request_json,
        lookup_cache,
        &document,
        &payload,
        uid,
    )?;
    Ok(InteractiveImportReview {
        action: action.to_string(),
        destination: destination.to_string(),
        action_label: action_label.to_string(),
        folder_path,
        source_folder_path: normalized_source_folder_path,
        destination_folder_path: normalized_destination_folder_path.unwrap_or_default(),
        reason: reason.to_string(),
        diff_status,
        diff_summary_lines,
    })
}

fn build_interactive_import_diff_summary_with_request<F>(
    request_json: &mut F,
    lookup_cache: &mut ImportLookupCache,
    local_document: &Value,
    payload: &Value,
    uid: &str,
) -> Result<(String, Vec<String>)>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if uid.is_empty() {
        return Ok((
            "new dashboard".to_string(),
            vec!["No live dashboard exists yet; import would create a new item.".to_string()],
        ));
    }
    let Some(remote_payload) = fetch_dashboard_if_exists_cached(request_json, lookup_cache, uid)?
    else {
        return Ok((
            "new dashboard".to_string(),
            vec!["No live dashboard exists yet; import would create a new item.".to_string()],
        ));
    };
    let payload_object =
        value_as_object(payload, "Dashboard import payload must be a JSON object.")?;
    let local_dashboard = payload_object
        .get("dashboard")
        .and_then(Value::as_object)
        .ok_or_else(|| message("Dashboard import payload is missing dashboard."))?;
    let remote_dashboard_value = build_preserved_web_import_document(&remote_payload)?;
    let remote_dashboard = value_as_object(
        &remote_dashboard_value,
        "Unexpected dashboard payload from Grafana.",
    )?;
    let local_title = string_field(local_dashboard, "title", uid);
    let remote_title = string_field(remote_dashboard, "title", uid);
    let local_folder_uid = payload_object
        .get("folderUid")
        .and_then(Value::as_str)
        .unwrap_or("");
    let remote_folder_uid = value_as_object(
        &remote_payload,
        "Unexpected dashboard payload from Grafana.",
    )?
    .get("meta")
    .and_then(Value::as_object)
    .map(|meta| string_field(meta, "folderUid", ""))
    .unwrap_or_default();
    let local_tags = join_tags(local_dashboard.get("tags"));
    let remote_tags = join_tags(remote_dashboard.get("tags"));
    let local_panels = panel_count(local_document);
    let remote_panels = panel_count(&remote_dashboard_value);

    let mut lines = Vec::new();
    if local_title != remote_title {
        lines.push(format!(
            "Title: {} -> {}",
            display_text(&remote_title),
            display_text(&local_title)
        ));
    }
    if local_folder_uid != remote_folder_uid {
        lines.push(format!(
            "Folder UID: {} -> {}",
            display_text(&remote_folder_uid),
            display_text(local_folder_uid)
        ));
    }
    if local_tags != remote_tags {
        lines.push(format!(
            "Tags: {} -> {}",
            display_text(&remote_tags),
            display_text(&local_tags)
        ));
    }
    if local_panels != remote_panels {
        lines.push(format!("Panels: {} -> {}", remote_panels, local_panels));
    }
    if lines.is_empty() {
        Ok((
            "matches live".to_string(),
            vec!["Import payload already matches the live dashboard shape.".to_string()],
        ))
    } else {
        Ok(("changed".to_string(), lines))
    }
}

fn join_tags(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

fn display_text(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

fn panel_count(document: &Value) -> usize {
    let Ok(object) = value_as_object(document, "Dashboard payload must be a JSON object.") else {
        return 0;
    };
    let Ok(dashboard) = super::extract_dashboard_object(object) else {
        return 0;
    };
    dashboard
        .get("panels")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}
