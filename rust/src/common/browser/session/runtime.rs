use crate::common::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

use super::model::{BrowserItem, BrowserPane, SearchDirection};
use super::render::{
    collect_kind_filters, render_browser_frame, selected_detail_line_count, visible_item_indexes,
};
use super::search::BrowserSearchController;

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

pub(crate) fn run_interactive_browser(
    title: &str,
    summary_lines: &[String],
    items: &[BrowserItem],
) -> Result<()> {
    let mut session = TerminalSession::enter()?;
    let mut state = ListState::default();
    let kind_filters = collect_kind_filters(items);
    let mut active_filter = 0usize;
    let mut visible_indexes = visible_item_indexes(items, &kind_filters[active_filter]);
    state.select((!visible_indexes.is_empty()).then_some(0));
    let mut detail_scroll = 0u16;
    let mut pane_focus = BrowserPane::Items;
    let mut search = BrowserSearchController::default();

    loop {
        session.terminal.draw(|frame| {
            render_browser_frame(
                frame,
                title,
                summary_lines,
                items,
                &mut state,
                &kind_filters,
                active_filter,
                &visible_indexes,
                detail_scroll,
                pane_focus,
                &search,
            );
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if search.has_pending() {
                match key.code {
                    KeyCode::Esc => search.cancel(),
                    KeyCode::Enter => {
                        let next_selection = search.apply(
                            items,
                            &visible_indexes,
                            state.selected(),
                            &kind_filters[active_filter],
                        );
                        if let Some(next_selection) = next_selection {
                            state.select(Some(next_selection));
                            detail_scroll = 0;
                        }
                    }
                    KeyCode::Backspace => search.pop_char(),
                    KeyCode::Char(_ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {}
                    KeyCode::Char(ch) => search.push_char(ch),
                    _ => {}
                }
                continue;
            }
            let selected_visible = state.selected().unwrap_or(0);
            let selected_item = visible_indexes
                .get(selected_visible)
                .and_then(|index| items.get(*index));
            let total_detail_lines = selected_detail_line_count(selected_item);
            match key.code {
                KeyCode::BackTab => {
                    pane_focus = match pane_focus {
                        BrowserPane::Items => BrowserPane::Detail,
                        BrowserPane::Detail => BrowserPane::Items,
                    };
                }
                KeyCode::Tab => {
                    pane_focus = match pane_focus {
                        BrowserPane::Items => BrowserPane::Detail,
                        BrowserPane::Detail => BrowserPane::Items,
                    };
                }
                KeyCode::Up => match pane_focus {
                    BrowserPane::Items => {
                        let selected = state.selected().unwrap_or(0);
                        state.select(Some(selected.saturating_sub(1)));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => {
                        detail_scroll = detail_scroll.saturating_sub(1);
                    }
                },
                KeyCode::Down => match pane_focus {
                    BrowserPane::Items => {
                        let selected = state.selected().unwrap_or(0);
                        state.select(Some(
                            (selected + 1).min(visible_indexes.len().saturating_sub(1)),
                        ));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => {
                        detail_scroll = detail_scroll
                            .saturating_add(1)
                            .min(total_detail_lines.saturating_sub(1) as u16);
                    }
                },
                KeyCode::PageUp => {
                    detail_scroll = detail_scroll.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    detail_scroll = detail_scroll
                        .saturating_add(10)
                        .min(total_detail_lines.saturating_sub(1) as u16);
                }
                KeyCode::Home => match pane_focus {
                    BrowserPane::Items => {
                        state.select(Some(0));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => detail_scroll = 0,
                },
                KeyCode::End => match pane_focus {
                    BrowserPane::Items => {
                        state.select(Some(visible_indexes.len().saturating_sub(1)));
                        detail_scroll = 0;
                    }
                    BrowserPane::Detail => {
                        detail_scroll = total_detail_lines.saturating_sub(1) as u16;
                    }
                },
                KeyCode::Enter => detail_scroll = 0,
                KeyCode::Char('f') => {
                    active_filter = (active_filter + 1) % kind_filters.len();
                    visible_indexes = visible_item_indexes(items, &kind_filters[active_filter]);
                    state.select((!visible_indexes.is_empty()).then_some(0));
                    detail_scroll = 0;
                }
                KeyCode::Char('F') => {
                    active_filter = if active_filter == 0 {
                        kind_filters.len().saturating_sub(1)
                    } else {
                        active_filter - 1
                    };
                    visible_indexes = visible_item_indexes(items, &kind_filters[active_filter]);
                    state.select((!visible_indexes.is_empty()).then_some(0));
                    detail_scroll = 0;
                }
                KeyCode::Char('/') => search.start(SearchDirection::Forward),
                KeyCode::Char('?') => search.start(SearchDirection::Backward),
                KeyCode::Char('n') => {
                    let next_selection = search.repeat(
                        items,
                        &visible_indexes,
                        state.selected(),
                        &kind_filters[active_filter],
                    );
                    if next_selection.is_some() {
                        state.select(next_selection);
                        detail_scroll = 0;
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
