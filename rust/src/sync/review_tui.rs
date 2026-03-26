//! Interactive sync review TUI.
//! Allows operators to keep or drop actionable sync operations before the plan is marked reviewed.
#[cfg(feature = "tui")]
use crate::common::message;
#[cfg(not(feature = "tui"))]
use crate::common::tui;
use crate::common::Result;

#[cfg(any(feature = "tui", test))]
use super::json::{require_json_array_field, require_json_object};
#[cfg(any(feature = "tui", test))]
use super::plan_builder::{build_sync_alert_assessment_document, build_sync_plan_summary_document};

#[cfg(feature = "tui")]
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
#[cfg(feature = "tui")]
use crossterm::execute;
#[cfg(feature = "tui")]
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
#[cfg(feature = "tui")]
use ratatui::backend::CrosstermBackend;
#[cfg(feature = "tui")]
use ratatui::layout::{Constraint, Direction, Layout};
#[cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Line, Span};
#[cfg(feature = "tui")]
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
#[cfg(feature = "tui")]
use ratatui::Terminal;
use serde_json::Value;
#[cfg(any(feature = "tui", test))]
use std::collections::BTreeSet;
#[cfg(feature = "tui")]
use std::io::{self, Stdout};
#[cfg(feature = "tui")]
use std::time::Duration;

#[cfg(any(feature = "tui", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewableOperation {
    key: String,
    label: String,
    operation: Value,
}

#[cfg(any(feature = "tui", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewDiffLine {
    pub changed: bool,
    pub marker: char,
    pub content: String,
    pub highlight_range: Option<(usize, usize)>,
}

#[cfg(any(feature = "tui", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewDiffModel {
    pub title: String,
    pub action: String,
    pub live_lines: Vec<ReviewDiffLine>,
    pub desired_lines: Vec<ReviewDiffLine>,
}

#[cfg(any(feature = "tui", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffPaneFocus {
    Live,
    Desired,
}

#[cfg(any(feature = "tui", test))]
type HighlightRange = Option<(usize, usize)>;

#[cfg(any(feature = "tui", test))]
pub(crate) struct DiffControlsState {
    pub selected: usize,
    pub total: usize,
    pub diff_focus: DiffPaneFocus,
    pub live_wrap_lines: bool,
    pub desired_wrap_lines: bool,
    pub live_diff_cursor: usize,
    pub live_horizontal_offset: usize,
    pub desired_diff_cursor: usize,
    pub desired_horizontal_offset: usize,
}

#[cfg(any(feature = "tui", test))]
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

#[cfg(any(feature = "tui", test))]
fn operation_label(operation: &serde_json::Map<String, Value>) -> String {
    format!(
        "{} {}",
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

#[cfg(any(feature = "tui", test))]
fn operation_badge(action: &str) -> &'static str {
    match action {
        "would-create" => "CREATE",
        "would-update" => "UPDATE",
        "would-delete" => "DELETE",
        _ => "UNKNOWN",
    }
}

#[cfg(feature = "tui")]
fn operation_badge_color(action: &str) -> Color {
    match action {
        "would-create" => Color::Green,
        "would-update" => Color::Yellow,
        "would-delete" => Color::Red,
        _ => Color::DarkGray,
    }
}

#[cfg(feature = "tui")]
fn operation_row_color(action: &str) -> Color {
    match action {
        "would-create" => Color::LightGreen,
        "would-update" => Color::LightYellow,
        "would-delete" => Color::LightRed,
        _ => Color::Gray,
    }
}

#[cfg(feature = "tui")]
fn selection_mark(selected: bool) -> &'static str {
    if selected {
        "✓"
    } else {
        "·"
    }
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn operation_preview(item: &ReviewableOperation) -> Vec<String> {
    let object = match item.operation.as_object() {
        Some(object) => object,
        None => return vec!["Invalid operation payload".to_string()],
    };
    let action = object
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let identity = object
        .get("identity")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let changed_fields = object
        .get("changedFields")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "none".to_string());
    vec![
        format!("Action: {}", operation_badge(action)),
        format!("Kind: {kind}"),
        format!("Identity: {identity}"),
        format!("Changed: {changed_fields}"),
    ]
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn operation_detail_line_count(item: &ReviewableOperation) -> usize {
    build_review_operation_diff_model(&item.operation)
        .map(|model| model.live_lines.len().max(model.desired_lines.len()))
        .unwrap_or(0)
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn operation_changed_count(item: &ReviewableOperation) -> usize {
    build_review_operation_diff_model(&item.operation)
        .map(|model| model.live_lines.iter().filter(|line| line.changed).count())
        .unwrap_or(0)
}

#[cfg(any(feature = "tui", test))]
fn truncate_text(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let kept = text.chars().take(max_chars - 1).collect::<String>();
    format!("{kept}…")
}

#[cfg(feature = "tui")]
pub(crate) fn build_checklist_line(
    item: &ReviewableOperation,
    index: usize,
    selected: bool,
    content_width: usize,
) -> Line<'static> {
    let action = item
        .operation
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let prefix = format!("{} {:>2}. ", selection_mark(selected), index + 1);
    let badge_text = format!("[{}]", operation_badge(action));
    let detail_rows = operation_detail_line_count(item);
    let changed = operation_changed_count(item);
    let row_label = if detail_rows == 1 { "row" } else { "rows" };
    let meta = format!("{detail_rows} {row_label} / {changed} changed");
    let reserved =
        prefix.chars().count() + 1 + badge_text.chars().count() + 1 + meta.chars().count();
    let label_width = content_width.saturating_sub(reserved).max(8);
    let label_text = truncate_text(&item.label, label_width);
    let current =
        prefix.chars().count() + badge_text.chars().count() + 1 + label_text.chars().count();
    let gap = content_width
        .saturating_sub(current + meta.chars().count())
        .max(1);

    Line::from(vec![
        Span::raw(prefix),
        Span::styled(
            badge_text,
            Style::default()
                .fg(operation_badge_color(action))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(label_text),
        Span::raw(" ".repeat(gap)),
        Span::styled(meta, Style::default().fg(Color::DarkGray)),
    ])
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn selection_title_with_position(
    item: Option<&ReviewableOperation>,
    position: Option<usize>,
    total: Option<usize>,
) -> String {
    let Some(item) = item else {
        return "Selection".to_string();
    };
    let action = item
        .operation
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let identity = item
        .operation
        .get("identity")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    match (position, total) {
        (Some(position), Some(total)) if total > 0 => format!(
            "Selection {}/{} [{}] {identity}",
            position + 1,
            total,
            operation_badge(action)
        ),
        _ => format!("Selection [{}] {identity}", operation_badge(action)),
    }
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn collect_reviewable_operations(plan: &Value) -> Result<Vec<ReviewableOperation>> {
    let plan = require_json_object(plan, "Sync plan document")?;
    let operations = require_json_array_field(plan, "operations", "Sync plan document")?;
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
            operation: Value::Object(operation.clone()),
        })
        .collect())
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn filter_review_plan_operations(
    plan: &Value,
    selected_keys: &BTreeSet<String>,
) -> Result<Value> {
    let plan_object = require_json_object(plan, "Sync plan document")?;
    let operations = require_json_array_field(plan_object, "operations", "Sync plan document")?;
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

#[cfg(feature = "tui")]
struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[cfg(feature = "tui")]
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

#[cfg(feature = "tui")]
impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[cfg(any(feature = "tui", test))]
fn pretty_inline_json(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "null".to_string(),
        Some(Value::String(text)) => format!("{text:?}"),
        Some(other) => serde_json::to_string(other).unwrap_or_else(|_| "null".to_string()),
    }
}

#[cfg(any(feature = "tui", test))]
fn numbered_line(index: usize, content: String) -> String {
    format!("{:>3} | {content}", index + 1)
}

#[cfg(any(feature = "tui", test))]
fn diff_highlight_ranges(left: &str, right: &str) -> (HighlightRange, HighlightRange) {
    if left == right {
        return (None, None);
    }
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    let mut prefix = 0usize;
    let min_len = left_bytes.len().min(right_bytes.len());
    while prefix < min_len && left_bytes[prefix] == right_bytes[prefix] {
        prefix += 1;
    }

    let mut left_suffix = left_bytes.len();
    let mut right_suffix = right_bytes.len();
    while left_suffix > prefix
        && right_suffix > prefix
        && left_bytes[left_suffix - 1] == right_bytes[right_suffix - 1]
    {
        left_suffix -= 1;
        right_suffix -= 1;
    }

    let left_range = if prefix == left_suffix {
        None
    } else {
        Some((prefix, left_suffix))
    };
    let right_range = if prefix == right_suffix {
        None
    } else {
        Some((prefix, right_suffix))
    };
    (left_range, right_range)
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn build_review_operation_diff_model(operation: &Value) -> Result<ReviewDiffModel> {
    let object = require_json_object(operation, "Sync review operation")?;
    let action = object
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let title = format!(
        "{} {}",
        object
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        object
            .get("identity")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    let desired = object.get("desired").and_then(Value::as_object);
    let live = object.get("live").and_then(Value::as_object);
    let changed_fields = object
        .get("changedFields")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let fields = if changed_fields.is_empty() {
        let mut combined = BTreeSet::new();
        if let Some(object) = live {
            combined.extend(object.keys().cloned());
        }
        if let Some(object) = desired {
            combined.extend(object.keys().cloned());
        }
        combined.into_iter().collect::<Vec<_>>()
    } else {
        changed_fields
    };
    if fields.is_empty() {
        return Ok(ReviewDiffModel {
            title,
            action,
            live_lines: vec![ReviewDiffLine {
                changed: false,
                marker: '=',
                content: numbered_line(0, "<no managed field changes>".to_string()),
                highlight_range: None,
            }],
            desired_lines: vec![ReviewDiffLine {
                changed: false,
                marker: '=',
                content: numbered_line(0, "<no managed field changes>".to_string()),
                highlight_range: None,
            }],
        });
    }
    let mut ordered_fields = fields
        .into_iter()
        .map(|field| {
            let live_value = live.and_then(|object| object.get(&field));
            let desired_value = desired.and_then(|object| object.get(&field));
            let changed = live_value != desired_value;
            (field, changed, live_value, desired_value)
        })
        .collect::<Vec<_>>();
    ordered_fields.sort_by_key(|(_, changed, _, _)| if *changed { 0 } else { 1 });

    let mut live_lines = Vec::new();
    let mut desired_lines = Vec::new();
    for (index, (field, changed, live_value, desired_value)) in
        ordered_fields.into_iter().enumerate()
    {
        let live_value_text = pretty_inline_json(live_value);
        let desired_value_text = pretty_inline_json(desired_value);
        let (live_range, desired_range) =
            diff_highlight_ranges(&live_value_text, &desired_value_text);
        let base_prefix = format!("{field}: ");
        let live_content = numbered_line(index, format!("{base_prefix}{live_value_text}"));
        let desired_content = numbered_line(index, format!("{base_prefix}{desired_value_text}"));
        let value_offset = numbered_line(index, base_prefix).len();

        live_lines.push(ReviewDiffLine {
            changed,
            marker: if changed { '-' } else { '=' },
            content: live_content,
            highlight_range: live_range
                .map(|(start, end)| (value_offset + start, value_offset + end)),
        });
        desired_lines.push(ReviewDiffLine {
            changed,
            marker: if changed { '+' } else { '=' },
            content: desired_content,
            highlight_range: desired_range
                .map(|(start, end)| (value_offset + start, value_offset + end)),
        });
    }
    Ok(ReviewDiffModel {
        title,
        action,
        live_lines,
        desired_lines,
    })
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn wrap_text_chunks(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    if text.is_empty() {
        return vec![String::new()];
    }
    let chars = text.chars().collect::<Vec<_>>();
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn clip_text_window(text: &str, offset: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    text.chars().skip(offset).take(width).collect::<String>()
}

#[cfg(feature = "tui")]
#[cfg(any(feature = "tui", test))]
fn render_diff_items(
    lines: &[ReviewDiffLine],
    color: Color,
    content_width: usize,
    wrap_lines: bool,
    horizontal_offset: usize,
) -> Vec<ListItem<'static>> {
    lines
        .iter()
        .map(|line| {
            let marker_style = if line.changed {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let content_style = if line.changed {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let wrapped = if wrap_lines {
                wrap_text_chunks(&line.content, content_width.max(1))
            } else {
                vec![clip_text_window(
                    &line.content,
                    horizontal_offset,
                    content_width.max(1),
                )]
            };
            let body = wrapped
                .into_iter()
                .enumerate()
                .map(|(index, chunk)| {
                    let marker = if index == 0 {
                        format!("{} ", line.marker)
                    } else {
                        "  ".to_string()
                    };
                    let marker_span = Span::styled(marker, marker_style);
                    let visible_highlight = if wrap_lines || index > 0 {
                        None
                    } else {
                        line.highlight_range.and_then(|(start, end)| {
                            let visible_start = start.max(horizontal_offset);
                            let visible_end = end.min(horizontal_offset + content_width.max(1));
                            if visible_start < visible_end {
                                Some((
                                    visible_start.saturating_sub(horizontal_offset),
                                    visible_end.saturating_sub(horizontal_offset),
                                ))
                            } else {
                                None
                            }
                        })
                    };
                    let content_span = match visible_highlight {
                        Some((start, end)) if start < end && end <= chunk.len() => {
                            let prefix = chunk[..start].to_string();
                            let middle = chunk[start..end].to_string();
                            let suffix = chunk[end..].to_string();
                            let mut spans = vec![marker_span, Span::raw(prefix)];
                            spans.push(Span::styled(
                                middle,
                                Style::default()
                                    .fg(color)
                                    .bg(Color::Black)
                                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                            ));
                            spans.push(Span::raw(suffix));
                            return Line::from(spans);
                        }
                        Some(_) if line.changed && index == 0 => Span::styled(
                            chunk,
                            Style::default()
                                .fg(color)
                                .bg(Color::Black)
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        ),
                        _ if line.changed => Span::styled(chunk, content_style),
                        _ => Span::raw(chunk),
                    };
                    Line::from(vec![marker_span, content_span])
                })
                .collect::<Vec<_>>();
            ListItem::new(body)
        })
        .collect()
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn diff_pane_title(
    pane: &str,
    action: &str,
    title: &str,
    position: usize,
    total: usize,
) -> String {
    format!("{pane} {}/{} [{}] {title}", position + 1, total, action)
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn diff_scroll_max(model: &ReviewDiffModel, focus: DiffPaneFocus) -> usize {
    match focus {
        DiffPaneFocus::Live => model.live_lines.len().saturating_sub(1),
        DiffPaneFocus::Desired => model.desired_lines.len().saturating_sub(1),
    }
}

#[cfg(feature = "tui")]
pub(crate) fn build_diff_controls_lines(state: &DiffControlsState) -> Vec<Line<'static>> {
    let focus = match state.diff_focus {
        DiffPaneFocus::Live => "LIVE",
        DiffPaneFocus::Desired => "DESIRED",
    };
    vec![
        Line::from(vec![
            Span::styled(
                format!("Item {}/{}", state.selected + 1, state.total),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(format!("Focus {focus}"), Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Live wrap {}",
                    if state.live_wrap_lines { "ON" } else { "OFF" }
                ),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Desired wrap {}",
                    if state.desired_wrap_lines {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled(
                "w active  W both".to_string(),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled("Left/Right pan", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "Live {} @{}",
                    state.live_diff_cursor + 1,
                    state.live_horizontal_offset
                ),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "Desired {} @{}",
                    state.desired_diff_cursor + 1,
                    state.desired_horizontal_offset
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from("Tab pane  Up/Down scroll  [/] item  PgUp/PgDn jump  Home/End bounds"),
        Line::from("Space toggle selection  c confirm  Esc/q back"),
    ]
}

#[cfg(feature = "tui")]
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
    let mut diff_mode = false;
    let mut diff_focus = DiffPaneFocus::Live;
    let mut live_diff_cursor = 0usize;
    let mut desired_diff_cursor = 0usize;
    let mut live_horizontal_offset = 0usize;
    let mut desired_horizontal_offset = 0usize;
    let mut live_wrap_lines = false;
    let mut desired_wrap_lines = false;

    loop {
        session.terminal.draw(|frame| {
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(5),
                    Constraint::Length(5),
                ])
                .split(frame.area());
            let selected = state.selected().unwrap_or(0);
            let selected_item = items.get(selected);
            let selected_count = selected_keys.len();
            let summary = Paragraph::new(format!(
                "Reviewable operations: {}  Selected: {}  Pending drop: {}",
                items.len(),
                selected_count,
                items.len().saturating_sub(selected_count),
            ))
            .block(Block::default().borders(Borders::ALL).title("Plan Status"));
            frame.render_widget(summary, outer[0]);
            if diff_mode {
                let model = selected_item.and_then(|item| build_review_operation_diff_model(&item.operation).ok());
                let panes = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(outer[1]);
                if let Some(model) = model {
                    let action_color = operation_badge_color(&model.action);
                    let mut live_state = ListState::default();
                    live_state.select(Some(live_diff_cursor.min(model.live_lines.len().saturating_sub(1))));
                    let live = List::new(render_diff_items(
                        &model.live_lines,
                        Color::Red,
                        panes[0].width.saturating_sub(5) as usize,
                        live_wrap_lines,
                        live_horizontal_offset,
                    ))
                        .block(
                            Block::default()
                                .title(diff_pane_title(
                                    "Live",
                                    &model.action,
                                    &model.title,
                                    selected,
                                    items.len(),
                                ))
                                .border_style(Style::default().fg(if diff_focus == DiffPaneFocus::Live {
                                    Color::Cyan
                                } else {
                                    action_color
                                }))
                                .borders(Borders::ALL),
                        )
                        .highlight_symbol("▌ ")
                        .repeat_highlight_symbol(true)
                        .highlight_style(
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                        );
                    let mut desired_state = ListState::default();
                    desired_state
                        .select(Some(desired_diff_cursor.min(model.desired_lines.len().saturating_sub(1))));
                    let desired = List::new(render_diff_items(
                        &model.desired_lines,
                        Color::Green,
                        panes[1].width.saturating_sub(5) as usize,
                        desired_wrap_lines,
                        desired_horizontal_offset,
                    ))
                        .block(
                            Block::default()
                                .title(diff_pane_title(
                                    "Desired",
                                    &model.action,
                                    &model.title,
                                    selected,
                                    items.len(),
                                ))
                                .border_style(Style::default().fg(if diff_focus == DiffPaneFocus::Desired {
                                    Color::Cyan
                                } else {
                                    action_color
                                }))
                                .borders(Borders::ALL),
                        )
                        .highlight_symbol("▌ ")
                        .repeat_highlight_symbol(true)
                        .highlight_style(
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                        );
                    frame.render_stateful_widget(live, panes[0], &mut live_state);
                    frame.render_stateful_widget(desired, panes[1], &mut desired_state);
                }
                let preview = Paragraph::new(
                    selected_item
                        .map(operation_preview)
                        .unwrap_or_else(|| vec!["No operation selected".to_string()])
                        .join("\n"),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(selection_title_with_position(
                            selected_item,
                            state.selected(),
                            Some(items.len()),
                        ))
                        .border_style(Style::default().fg(
                            selected_item
                                .and_then(|item| {
                                    item.operation
                                        .get("action")
                                        .and_then(Value::as_str)
                                        .map(operation_badge_color)
                                })
                                .unwrap_or(Color::Gray),
                        )),
                );
                frame.render_widget(preview, outer[2]);
                let help = Paragraph::new(build_diff_controls_lines(&DiffControlsState {
                    selected,
                    total: items.len(),
                    diff_focus,
                    live_wrap_lines,
                    desired_wrap_lines,
                    live_diff_cursor,
                    live_horizontal_offset,
                    desired_diff_cursor,
                    desired_horizontal_offset,
                }))
                .block(Block::default().borders(Borders::ALL).title("Diff Controls"));
                frame.render_widget(help, outer[3]);
            } else {
                let list_items = items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        let action = item
                            .operation
                            .get("action")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        ListItem::new(build_checklist_line(
                            item,
                            index,
                            selected_keys.contains(&item.key),
                            outer[1].width.saturating_sub(6) as usize,
                        ))
                            .style(Style::default().fg(operation_row_color(action)))
                    })
                    .collect::<Vec<_>>();
                let list = List::new(list_items)
                    .block(
                        Block::default()
                            .title("Sync Review Checklist")
                            .borders(Borders::ALL),
                    )
                    .highlight_symbol("▌ ")
                    .repeat_highlight_symbol(true)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    );
                frame.render_stateful_widget(list, outer[1], &mut state);
                let preview = Paragraph::new(
                    selected_item
                        .map(operation_preview)
                        .unwrap_or_else(|| vec!["No operation selected".to_string()])
                        .join("\n"),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(selection_title_with_position(
                            selected_item,
                            state.selected(),
                            Some(items.len()),
                        ))
                        .border_style(Style::default().fg(
                            selected_item
                                .and_then(|item| {
                                    item.operation
                                        .get("action")
                                        .and_then(Value::as_str)
                                        .map(operation_badge_color)
                                })
                                .unwrap_or(Color::Gray),
                        )),
                );
                frame.render_widget(preview, outer[2]);
                let help = Paragraph::new(
                    "Up/Down move  Space toggle  a select-all  n select-none  Enter diff  c confirm  q cancel",
                )
                .block(Block::default().borders(Borders::ALL).title("Controls"));
                frame.render_widget(help, outer[3]);
            }
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
                    if diff_mode {
                        match diff_focus {
                            DiffPaneFocus::Live => {
                                live_diff_cursor = live_diff_cursor.saturating_sub(1);
                            }
                            DiffPaneFocus::Desired => {
                                desired_diff_cursor = desired_diff_cursor.saturating_sub(1);
                            }
                        }
                    } else {
                        let next = selected.saturating_sub(1);
                        state.select(Some(next));
                    }
                }
                KeyCode::Down => {
                    if diff_mode {
                        if let Some(item) = items.get(selected) {
                            if let Ok(model) = build_review_operation_diff_model(&item.operation) {
                                match diff_focus {
                                    DiffPaneFocus::Live => {
                                        live_diff_cursor = (live_diff_cursor + 1)
                                            .min(diff_scroll_max(&model, DiffPaneFocus::Live));
                                    }
                                    DiffPaneFocus::Desired => {
                                        desired_diff_cursor = (desired_diff_cursor + 1)
                                            .min(diff_scroll_max(&model, DiffPaneFocus::Desired));
                                    }
                                }
                            }
                        }
                    } else {
                        let next = (selected + 1).min(items.len().saturating_sub(1));
                        state.select(Some(next));
                    }
                }
                KeyCode::Left => {
                    if diff_mode {
                        match diff_focus {
                            DiffPaneFocus::Live if !live_wrap_lines => {
                                live_horizontal_offset = live_horizontal_offset.saturating_sub(4);
                            }
                            DiffPaneFocus::Desired if !desired_wrap_lines => {
                                desired_horizontal_offset =
                                    desired_horizontal_offset.saturating_sub(4);
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Right => {
                    if diff_mode {
                        match diff_focus {
                            DiffPaneFocus::Live if !live_wrap_lines => live_horizontal_offset += 4,
                            DiffPaneFocus::Desired if !desired_wrap_lines => {
                                desired_horizontal_offset += 4
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Char('[') => {
                    if diff_mode {
                        let next = selected.saturating_sub(1);
                        state.select(Some(next));
                        live_diff_cursor = 0;
                        desired_diff_cursor = 0;
                        live_horizontal_offset = 0;
                        desired_horizontal_offset = 0;
                    }
                }
                KeyCode::Char(']') => {
                    if diff_mode {
                        let next = (selected + 1).min(items.len().saturating_sub(1));
                        state.select(Some(next));
                        live_diff_cursor = 0;
                        desired_diff_cursor = 0;
                        live_horizontal_offset = 0;
                        desired_horizontal_offset = 0;
                    }
                }
                KeyCode::Tab => {
                    if diff_mode {
                        diff_focus = match diff_focus {
                            DiffPaneFocus::Live => DiffPaneFocus::Desired,
                            DiffPaneFocus::Desired => DiffPaneFocus::Live,
                        };
                    }
                }
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    if diff_mode {
                        let apply_both = key.modifiers.contains(KeyModifiers::SHIFT)
                            || matches!(key.code, KeyCode::Char('W'));
                        if apply_both {
                            let next_wrap = !(live_wrap_lines && desired_wrap_lines);
                            live_wrap_lines = next_wrap;
                            desired_wrap_lines = next_wrap;
                            if next_wrap {
                                live_horizontal_offset = 0;
                                desired_horizontal_offset = 0;
                            }
                        } else {
                            match diff_focus {
                                DiffPaneFocus::Live => {
                                    live_wrap_lines = !live_wrap_lines;
                                    if live_wrap_lines {
                                        live_horizontal_offset = 0;
                                    }
                                }
                                DiffPaneFocus::Desired => {
                                    desired_wrap_lines = !desired_wrap_lines;
                                    if desired_wrap_lines {
                                        desired_horizontal_offset = 0;
                                    }
                                }
                            }
                        }
                    }
                }
                KeyCode::PageUp => {
                    if diff_mode {
                        match diff_focus {
                            DiffPaneFocus::Live => {
                                live_diff_cursor = live_diff_cursor.saturating_sub(10);
                            }
                            DiffPaneFocus::Desired => {
                                desired_diff_cursor = desired_diff_cursor.saturating_sub(10);
                            }
                        }
                    }
                }
                KeyCode::PageDown => {
                    if diff_mode {
                        if let Some(item) = items.get(selected) {
                            if let Ok(model) = build_review_operation_diff_model(&item.operation) {
                                match diff_focus {
                                    DiffPaneFocus::Live => {
                                        live_diff_cursor = (live_diff_cursor + 10)
                                            .min(diff_scroll_max(&model, DiffPaneFocus::Live));
                                    }
                                    DiffPaneFocus::Desired => {
                                        desired_diff_cursor = (desired_diff_cursor + 10)
                                            .min(diff_scroll_max(&model, DiffPaneFocus::Desired));
                                    }
                                }
                            }
                        }
                    }
                }
                KeyCode::Home => {
                    if diff_mode {
                        match diff_focus {
                            DiffPaneFocus::Live => live_diff_cursor = 0,
                            DiffPaneFocus::Desired => desired_diff_cursor = 0,
                        }
                    }
                }
                KeyCode::End => {
                    if diff_mode {
                        if let Some(item) = items.get(selected) {
                            if let Ok(model) = build_review_operation_diff_model(&item.operation) {
                                match diff_focus {
                                    DiffPaneFocus::Live => {
                                        live_diff_cursor =
                                            diff_scroll_max(&model, DiffPaneFocus::Live);
                                    }
                                    DiffPaneFocus::Desired => {
                                        desired_diff_cursor =
                                            diff_scroll_max(&model, DiffPaneFocus::Desired);
                                    }
                                }
                            }
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(item) = items.get(selected) {
                        if !selected_keys.insert(item.key.clone()) {
                            selected_keys.remove(&item.key);
                        }
                    }
                }
                KeyCode::Char('a') => {
                    if !diff_mode {
                        selected_keys = items.iter().map(|item| item.key.clone()).collect();
                    }
                }
                KeyCode::Char('n') => {
                    if !diff_mode {
                        selected_keys.clear();
                    }
                }
                KeyCode::Enter => {
                    if !diff_mode {
                        diff_mode = true;
                        diff_focus = DiffPaneFocus::Live;
                        live_diff_cursor = 0;
                        desired_diff_cursor = 0;
                        live_horizontal_offset = 0;
                        desired_horizontal_offset = 0;
                    }
                }
                KeyCode::Char('c') => return filter_review_plan_operations(plan, &selected_keys),
                KeyCode::Char('q') | KeyCode::Esc => {
                    if diff_mode {
                        diff_mode = false;
                        diff_focus = DiffPaneFocus::Live;
                        live_diff_cursor = 0;
                        desired_diff_cursor = 0;
                        live_horizontal_offset = 0;
                        desired_horizontal_offset = 0;
                        continue;
                    }
                    return Err(message("Interactive sync review cancelled."));
                }
                _ => {}
            }
        }
    }
}

#[cfg(not(feature = "tui"))]
pub(crate) fn run_sync_review_tui(plan: &Value) -> Result<Value> {
    let _ = plan;
    Err(tui(
        "Sync review interactive TUI requires the `tui` feature.",
    ))
}

#[cfg(not(feature = "tui"))]
#[test]
fn run_sync_review_tui_returns_tui_error_when_feature_disabled() {
    let plan = serde_json::json!({
        "kind": "grafana-utils-sync-plan",
        "operations": []
    });

    let error = run_sync_review_tui(&plan).expect_err("feature-disabled review should error");
    assert_eq!(
        error.to_string(),
        "Sync review interactive TUI requires the `tui` feature."
    );
}
