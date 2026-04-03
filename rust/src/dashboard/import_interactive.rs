#![cfg(feature = "tui")]
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::common::{message, string_field, value_as_object, Result};

use super::browse_terminal::TerminalSession;
use super::import_lookup::resolve_source_dashboard_folder_path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InteractiveImportItem {
    pub(crate) path: PathBuf,
    pub(crate) uid: String,
    pub(crate) title: String,
    pub(crate) folder_path: String,
    pub(crate) file_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InteractiveImportAction {
    Continue,
    Confirm(Vec<PathBuf>),
    Cancel,
}

pub(crate) struct InteractiveImportState {
    pub(crate) items: Vec<InteractiveImportItem>,
    pub(crate) selected_paths: BTreeSet<PathBuf>,
    pub(crate) list_state: ListState,
    pub(crate) status: String,
}

impl InteractiveImportState {
    pub(crate) fn new(items: Vec<InteractiveImportItem>) -> Self {
        let mut list_state = ListState::default();
        list_state.select((!items.is_empty()).then_some(0));
        Self {
            items,
            selected_paths: BTreeSet::new(),
            list_state,
            status: "Space toggles a dashboard. Enter imports the selected dashboards.".to_string(),
        }
    }

    pub(crate) fn selected_item(&self) -> Option<&InteractiveImportItem> {
        let index = self.list_state.selected()?;
        self.items.get(index)
    }

    pub(crate) fn move_selection(&mut self, delta: isize) {
        if self.items.is_empty() {
            self.list_state.select(None);
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, self.items.len().saturating_sub(1) as isize) as usize;
        self.list_state.select(Some(next));
    }

    pub(crate) fn select_first(&mut self) {
        self.list_state
            .select((!self.items.is_empty()).then_some(0));
    }

    pub(crate) fn select_last(&mut self) {
        self.list_state.select(self.items.len().checked_sub(1));
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

pub(crate) fn select_import_dashboard_files(
    args: &super::ImportArgs,
) -> Result<Option<Vec<PathBuf>>> {
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
    let items = load_interactive_import_items(args)?;
    if items.is_empty() {
        return Err(message(format!(
            "No dashboard JSON files were found under {}.",
            args.import_dir.display()
        )));
    }
    run_import_selector(args.import_dir.display().to_string(), items)
}

fn run_import_selector(
    import_dir_label: String,
    items: Vec<InteractiveImportItem>,
) -> Result<Option<Vec<PathBuf>>> {
    let mut session = TerminalSession::enter()?;
    session.terminal.hide_cursor()?;
    let mut state = InteractiveImportState::new(items);
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

            let header = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Interactive Dashboard Import",
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!(
                    "Import dir: {}   Selected: {}/{}",
                    import_dir_label,
                    state.selected_paths.len(),
                    state.items.len()
                )),
            ])
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Black)));
            frame.render_widget(header, chunks[0]);

            let list_items: Vec<ListItem> = state
                .items
                .iter()
                .map(|item| {
                    let marker = if state.selected_paths.contains(&item.path) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    let folder = if item.folder_path.is_empty() {
                        "General"
                    } else {
                        item.folder_path.as_str()
                    };
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(marker, Style::default().fg(Color::Green)),
                            Span::raw(" "),
                            Span::styled(
                                item.title.as_str(),
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(vec![
                            Span::styled("uid ", Style::default().fg(Color::DarkGray)),
                            Span::raw(item.uid.as_str()),
                            Span::raw("  "),
                            Span::styled("folder ", Style::default().fg(Color::DarkGray)),
                            Span::raw(folder),
                        ]),
                    ])
                })
                .collect();
            let list = List::new(list_items)
                .block(Block::default().title("Dashboards").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightBlue)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            frame.render_stateful_widget(list, body[0], &mut state.list_state);

            let detail_lines = if let Some(item) = state.selected_item() {
                vec![
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
                        Span::styled("Folder: ", Style::default().fg(Color::Yellow)),
                        Span::raw(if item.folder_path.is_empty() {
                            "General"
                        } else {
                            item.folder_path.as_str()
                        }),
                    ]),
                    Line::from(vec![
                        Span::styled("File: ", Style::default().fg(Color::Yellow)),
                        Span::raw(item.file_label.as_str()),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Restore/import applies the existing import flags to the selected dashboards only.",
                        Style::default().fg(Color::Gray),
                    )),
                ]
            } else {
                vec![Line::from("No dashboard selected.")]
            };
            let detail = Paragraph::new(detail_lines)
                .block(Block::default().title("Details").borders(Borders::ALL))
                .wrap(Wrap { trim: false });
            frame.render_widget(detail, body[1]);

            let footer = Paragraph::new(vec![
                Line::from(vec![
                    hotkey("Up/Down"),
                    Span::raw(" move   "),
                    hotkey("Space"),
                    Span::raw(" toggle   "),
                    hotkey("a"),
                    Span::raw(" all/none   "),
                    hotkey("Enter"),
                    Span::raw(" import selected   "),
                    hotkey("q"),
                    Span::raw(" cancel"),
                ]),
                Line::from(Span::raw(state.status.as_str())),
            ])
            .block(Block::default().title("Hotkeys").borders(Borders::ALL));
            frame.render_widget(Clear, chunks[2]);
            frame.render_widget(footer, chunks[2]);
        })?;

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

fn hotkey(label: &str) -> Span<'static> {
    Span::styled(
        format!(" {label} "),
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
}

pub(crate) fn load_interactive_import_items(
    args: &super::ImportArgs,
) -> Result<Vec<InteractiveImportItem>> {
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
    Ok(items)
}
