use crate::tui_shell;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use super::model::{BrowserItem, BrowserPane};
use super::search::BrowserSearchController;

pub(super) fn item_color(kind: &str) -> Color {
    match kind {
        "dashboard" => Color::Yellow,
        "alert" | "alert-rule" => Color::Red,
        "datasource" => Color::Cyan,
        "user" => Color::Green,
        "team" => Color::LightMagenta,
        "warning" => Color::Yellow,
        "violation" => Color::LightRed,
        "drift" => Color::LightRed,
        "policy" => Color::Magenta,
        _ => Color::Gray,
    }
}

pub(super) fn collect_kind_filters(items: &[BrowserItem]) -> Vec<String> {
    let mut filters = vec!["all".to_string()];
    for item in items {
        if !filters.iter().any(|kind| kind == &item.kind) {
            filters.push(item.kind.clone());
        }
    }
    filters
}

pub(super) fn visible_item_indexes(items: &[BrowserItem], filter_kind: &str) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if filter_kind == "all" || item.kind == filter_kind {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn selected_detail_line_count(item: Option<&BrowserItem>) -> usize {
    item.map(|candidate| candidate.details.len().max(1))
        .unwrap_or(1)
}

pub(crate) fn detail_title(
    item: &BrowserItem,
    selected_visible: usize,
    visible_total: usize,
    detail_scroll: u16,
    total_detail_lines: usize,
) -> String {
    format!(
        "Detail {}/{} [{}]  line {}/{}",
        selected_visible + 1,
        visible_total,
        item.kind,
        (detail_scroll as usize + 1).min(total_detail_lines),
        total_detail_lines
    )
}

fn pane_block(title: &str, focused: bool, accent: Color, bg: Color) -> Block<'static> {
    let title_bg = if focused { accent } else { bg };
    let title_fg = if focused { Color::Black } else { Color::White };
    Block::default()
        .borders(Borders::ALL)
        .title(if focused {
            format!("{title} [Focused]")
        } else {
            title.to_string()
        })
        .style(Style::default().bg(bg))
        .border_style(Style::default().fg(if focused { accent } else { Color::Gray }))
        .title_style(
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(Modifier::BOLD),
        )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_browser_frame(
    frame: &mut ratatui::Frame,
    title: &str,
    summary_lines: &[String],
    items: &[BrowserItem],
    state: &mut ListState,
    kind_filters: &[String],
    active_filter: usize,
    visible_indexes: &[usize],
    detail_scroll: u16,
    pane_focus: BrowserPane,
    search: &BrowserSearchController,
) {
    let mut runtime_summary_lines = summary_lines.to_vec();
    runtime_summary_lines.push(search.summary_line(&kind_filters[active_filter]));
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((runtime_summary_lines.len().max(1) + 2) as u16),
            Constraint::Min(1),
            Constraint::Length(4),
        ])
        .split(frame.area());
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(outer[1]);
    let selected_visible = state.selected().unwrap_or(0);
    let selected_item = visible_indexes
        .get(selected_visible)
        .and_then(|index| items.get(*index));
    let total_detail_lines = selected_detail_line_count(selected_item);
    let detail_lines = selected_item
        .map(|item| {
            if item.details.is_empty() {
                vec!["No detail lines.".to_string()]
            } else {
                item.details.clone()
            }
        })
        .unwrap_or_else(|| vec!["No item selected".to_string()]);
    let detail_selected = (detail_scroll as usize).min(detail_lines.len().saturating_sub(1));

    let summary = Paragraph::new(runtime_summary_lines.join("\n"))
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(summary, outer[0]);

    let list = List::new(
        visible_indexes
            .iter()
            .enumerate()
            .map(|(visible_index, item_index)| {
                let item = &items[*item_index];
                let line = Line::from(vec![
                    Span::styled(
                        format!("{:>2}. ", visible_index + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("[{}]", item.kind.to_uppercase()),
                        Style::default()
                            .fg(item_color(&item.kind))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" {}", item.title)),
                    Span::styled(
                        format!("  {}", item.meta),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect::<Vec<_>>(),
    )
    .block(
        pane_block(
            "Items",
            pane_focus == BrowserPane::Items,
            Color::Cyan,
            Color::Black,
        )
        .title(format!(
            "Items {}/{}  filter:{}",
            visible_indexes.len(),
            items.len(),
            kind_filters[active_filter]
        )),
    )
    .highlight_symbol("▌ ")
    .repeat_highlight_symbol(true)
    .highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
    );
    frame.render_stateful_widget(list, panes[0], state);

    let detail_title = selected_item
        .map(|item| {
            detail_title(
                item,
                selected_visible,
                visible_indexes.len(),
                detail_scroll,
                total_detail_lines,
            )
        })
        .unwrap_or_else(|| "Detail".to_string());
    let detail_items = detail_lines
        .iter()
        .map(|line| {
            ListItem::new(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(Color::White),
            )))
        })
        .collect::<Vec<_>>();
    if pane_focus == BrowserPane::Detail {
        let mut detail_state = ListState::default();
        detail_state.select(Some(detail_selected));
        let detail = List::new(detail_items)
            .block(pane_block("Detail", true, Color::LightBlue, Color::Black).title(detail_title))
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(detail, panes[1], &mut detail_state);
    } else {
        let detail = List::new(detail_items)
            .block(pane_block("Detail", false, Color::LightBlue, Color::Black).title(detail_title));
        frame.render_widget(detail, panes[1]);
    }

    frame.render_widget(
        tui_shell::build_footer_controls(vec![
            Line::from(vec![
                tui_shell::label("Selection "),
                tui_shell::accent(
                    format!(
                        "{}/{}",
                        state.selected().map(|index| index + 1).unwrap_or(0),
                        visible_indexes.len()
                    ),
                    Color::White,
                ),
                Span::raw("  "),
                tui_shell::label("Filter "),
                tui_shell::accent(kind_filters[active_filter].to_string(), Color::Yellow),
                Span::raw("  "),
                tui_shell::focus_label("Focus "),
                tui_shell::key_chip(
                    match pane_focus {
                        BrowserPane::Items => "Items",
                        BrowserPane::Detail => "Detail",
                    },
                    Color::Blue,
                ),
                Span::raw("  "),
                tui_shell::label("Search "),
                tui_shell::accent(search.footer_label(), Color::LightGreen),
            ]),
            tui_shell::control_line(&[
                ("Tab", Color::Blue, "next pane"),
                ("Shift+Tab", Color::Blue, "previous pane"),
                ("Up/Down", Color::Blue, "move"),
                ("PgUp/PgDn", Color::Blue, "scroll detail"),
            ]),
            if search.has_pending() {
                tui_shell::control_line(&[
                    ("Backspace", Color::Blue, "edit"),
                    ("Enter", Color::LightGreen, "search"),
                    ("Esc", Color::Yellow, "cancel"),
                    ("q", Color::LightGreen, "search text"),
                ])
            } else {
                tui_shell::control_line(&[
                    ("f/F", Color::Yellow, "change filter"),
                    ("Home/End", Color::Blue, "jump"),
                    ("/ ?", Color::LightGreen, "search"),
                    ("n", Color::LightGreen, "repeat"),
                    ("Esc/q", Color::Gray, "exit"),
                ])
            },
        ]),
        outer[2],
    );
}
