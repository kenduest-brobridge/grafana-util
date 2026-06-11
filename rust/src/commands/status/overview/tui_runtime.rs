#![cfg(feature = "tui")]

use super::{
    overview_tui_render, OverviewDocument, OverviewPane, OverviewWorkbenchState, SearchDirection,
};
use crate::common::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

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

pub(crate) fn run_overview_interactive(document: OverviewDocument) -> Result<()> {
    let mut session = TerminalSession::enter()?;
    let mut state = OverviewWorkbenchState::new(document);

    loop {
        session
            .terminal
            .draw(|frame| overview_tui_render::render_overview_frame(frame, &mut state))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if state.pending_search.is_some() {
            state.handle_search_key(key.code);
            continue;
        }

        let detail_lines_len = state.current_detail_lines().len();
        match key.code {
            KeyCode::Char('/') => state.start_search(SearchDirection::Forward),
            KeyCode::Char('?') => state.start_search(SearchDirection::Backward),
            KeyCode::Char('n') => state.repeat_search(),
            KeyCode::Tab => state.focus_next(),
            KeyCode::BackTab => state.focus_previous(),
            KeyCode::Char('h') => state.focus_project_home(),
            KeyCode::Up => match state.focus {
                OverviewPane::ProjectHome => {}
                OverviewPane::Sections => state.move_section_selection(-1),
                OverviewPane::Views => state.move_view_selection(-1),
                OverviewPane::Items => state.move_item_selection(-1),
                OverviewPane::Details => state.move_detail_scroll(-1, detail_lines_len),
            },
            KeyCode::Down => match state.focus {
                OverviewPane::ProjectHome => {}
                OverviewPane::Sections => state.move_section_selection(1),
                OverviewPane::Views => state.move_view_selection(1),
                OverviewPane::Items => state.move_item_selection(1),
                OverviewPane::Details => state.move_detail_scroll(1, detail_lines_len),
            },
            KeyCode::PageUp if state.focus == OverviewPane::Details => {
                state.move_detail_scroll(-10, detail_lines_len);
            }
            KeyCode::PageDown if state.focus == OverviewPane::Details => {
                state.move_detail_scroll(10, detail_lines_len);
            }
            KeyCode::Home => match state.focus {
                OverviewPane::ProjectHome => {}
                OverviewPane::Sections => {
                    state.move_section_selection(
                        -(state.section_state.selected().unwrap_or(0) as isize),
                    );
                }
                OverviewPane::Views => {
                    state.move_view_selection(-(state.view_state.selected().unwrap_or(0) as isize));
                }
                OverviewPane::Items => {
                    state.move_item_selection(-(state.item_state.selected().unwrap_or(0) as isize));
                }
                OverviewPane::Details => state.detail_scroll = 0,
            },
            KeyCode::End => match state.focus {
                OverviewPane::ProjectHome => {}
                OverviewPane::Sections => state.move_section_selection(
                    state.document.sections.len().saturating_sub(1) as isize,
                ),
                OverviewPane::Views => {
                    let last = state
                        .current_section()
                        .map(|section| section.views.len())
                        .unwrap_or(0);
                    if last > 0 {
                        state.move_view_selection(last.saturating_sub(1) as isize);
                    }
                }
                OverviewPane::Items => {
                    let last = state.current_items().len();
                    if last > 0 {
                        state.move_item_selection(last.saturating_sub(1) as isize);
                    }
                }
                OverviewPane::Details => {
                    state.detail_scroll = detail_lines_len.saturating_sub(1) as u16;
                }
            },
            KeyCode::Enter => match state.focus {
                OverviewPane::ProjectHome => state.handoff_from_home(),
                _ => state.detail_scroll = 0,
            },
            KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
            _ => {}
        }
    }
}
