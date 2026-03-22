//! Interactive sync review TUI.
//! Allows operators to keep or drop actionable sync operations before the plan is marked reviewed.
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::{self, Stdout};
use std::time::Duration;

use crate::common::{message, Result};

use super::{build_sync_alert_assessment_document, build_sync_plan_summary_document};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReviewableOperation {
    key: String,
    label: String,
}

fn operation_key(operation: &serde_json::Map<String, Value>) -> String {
    format!(
        "{}::{}",
        operation
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        operation
            .get("identity")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )
}

fn operation_label(operation: &serde_json::Map<String, Value>) -> String {
    format!(
        "[{}] {} {}",
        operation
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        operation
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        operation
            .get("identity")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )
}

fn collect_reviewable_operations(plan: &Value) -> Result<Vec<ReviewableOperation>> {
    let operations = plan
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| message("Sync plan document is missing operations."))?;
    Ok(operations
        .iter()
        .filter_map(Value::as_object)
        .filter(|operation| {
            matches!(
                operation.get("action").and_then(Value::as_str),
                Some("would-create" | "would-update" | "would-delete")
            )
        })
        .map(|operation| ReviewableOperation {
            key: operation_key(operation),
            label: operation_label(operation),
        })
        .collect())
}

pub(crate) fn filter_review_plan_operations(
    plan: &Value,
    selected_keys: &BTreeSet<String>,
) -> Result<Value> {
    let plan_object = plan
        .as_object()
        .ok_or_else(|| message("Sync plan document must be a JSON object."))?;
    let operations = plan_object
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| message("Sync plan document is missing operations."))?;
    let filtered_operations = operations
        .iter()
        .filter(|item| {
            let Some(object) = item.as_object() else {
                return false;
            };
            match object.get("action").and_then(Value::as_str) {
                Some("would-create" | "would-update" | "would-delete") => {
                    selected_keys.contains(&operation_key(object))
                }
                _ => true,
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut filtered = plan_object.clone();
    filtered.insert(
        "summary".to_string(),
        build_sync_plan_summary_document(&filtered_operations),
    );
    filtered.insert(
        "alertAssessment".to_string(),
        build_sync_alert_assessment_document(&filtered_operations),
    );
    filtered.insert("operations".to_string(), Value::Array(filtered_operations));
    Ok(Value::Object(filtered))
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

pub(crate) fn run_sync_review_tui(plan: &Value) -> Result<Value> {
    let items = collect_reviewable_operations(plan)?;
    if items.is_empty() {
        return Ok(plan.clone());
    }
    let mut session = TerminalSession::enter()?;
    let mut selected_keys = items
        .iter()
        .map(|item| item.key.clone())
        .collect::<BTreeSet<_>>();
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        session.terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(frame.area());
            let list_items = items
                .iter()
                .map(|item| {
                    let checked = if selected_keys.contains(&item.key) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    ListItem::new(format!("{checked} {}", item.label))
                })
                .collect::<Vec<_>>();
            let list = List::new(list_items)
                .block(Block::default().title("Sync Review").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_stateful_widget(list, chunks[0], &mut state);
            let help = Paragraph::new(
                "Up/Down move  Space toggle  a select-all  n select-none  Enter confirm  q cancel",
            )
            .block(Block::default().borders(Borders::ALL).title("Controls"));
            frame.render_widget(help, chunks[1]);
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let selected = state.selected().unwrap_or(0);
            match key.code {
                KeyCode::Up => {
                    let next = selected.saturating_sub(1);
                    state.select(Some(next));
                }
                KeyCode::Down => {
                    let next = (selected + 1).min(items.len().saturating_sub(1));
                    state.select(Some(next));
                }
                KeyCode::Char(' ') => {
                    if let Some(item) = items.get(selected) {
                        if !selected_keys.insert(item.key.clone()) {
                            selected_keys.remove(&item.key);
                        }
                    }
                }
                KeyCode::Char('a') => {
                    selected_keys = items.iter().map(|item| item.key.clone()).collect();
                }
                KeyCode::Char('n') => {
                    selected_keys.clear();
                }
                KeyCode::Enter => return filter_review_plan_operations(plan, &selected_keys),
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Err(message("Interactive sync review cancelled."));
                }
                _ => {}
            }
        }
    }
}
