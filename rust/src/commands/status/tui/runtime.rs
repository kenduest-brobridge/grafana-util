#![cfg(feature = "tui")]

use super::{
    render_project_status_frame, ProjectStatusPane, ProjectStatusTuiState, SearchDirection,
};
use crate::common::Result;
use crate::project_status::ProjectStatus;
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

pub(crate) fn run_project_status_interactive(document: ProjectStatus) -> Result<()> {
    let mut session = TerminalSession::enter()?;
    let mut state = ProjectStatusTuiState::new(document);

    loop {
        session
            .terminal
            .draw(|frame| render_project_status_frame(frame, &mut state))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if state.pending_search().is_some() {
            state.handle_search_key(key.code);
            continue;
        }

        let detail_lines_len = state.current_domain_lines().len();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Tab => state.focus_next(),
            KeyCode::BackTab => state.focus_previous(),
            KeyCode::Char('h') => state.focus_home(),
            KeyCode::Char('/') => state.start_search(SearchDirection::Forward),
            KeyCode::Char('?') => state.start_search(SearchDirection::Backward),
            KeyCode::Char('n') => state.repeat_search(),
            KeyCode::Enter if state.focus() == ProjectStatusPane::Home => {
                state.handoff_from_home();
            }
            KeyCode::Up => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => state.move_domain_selection(-1),
                ProjectStatusPane::Details => state.move_detail_scroll(-1),
                ProjectStatusPane::Actions => state.move_action_selection(-1),
            },
            KeyCode::Down => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => state.move_domain_selection(1),
                ProjectStatusPane::Details => state.move_detail_scroll(1),
                ProjectStatusPane::Actions => state.move_action_selection(1),
            },
            KeyCode::PageUp if state.focus() == ProjectStatusPane::Details => {
                state.move_detail_scroll(-10);
            }
            KeyCode::PageDown if state.focus() == ProjectStatusPane::Details => {
                state.move_detail_scroll(10);
            }
            KeyCode::Home => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => {
                    let current = state.current_domain_index().unwrap_or(0) as isize;
                    state.move_domain_selection(-current);
                }
                ProjectStatusPane::Details => state.detail_scroll = 0,
                ProjectStatusPane::Actions => {
                    let current = state.current_action_index().unwrap_or(0) as isize;
                    state.move_action_selection(-current);
                }
            },
            KeyCode::End => match state.focus() {
                ProjectStatusPane::Home => {}
                ProjectStatusPane::Domains => {
                    let len = state.document().domains.len();
                    if len > 0 {
                        let current = state.current_domain_index().unwrap_or(0) as isize;
                        state.move_domain_selection(len.saturating_sub(1) as isize - current);
                    }
                }
                ProjectStatusPane::Details => {
                    state.detail_scroll = detail_lines_len.saturating_sub(1) as u16;
                }
                ProjectStatusPane::Actions => {
                    let len = state.document().next_actions.len();
                    if len > 0 {
                        let current = state.current_action_index().unwrap_or(0) as isize;
                        state.move_action_selection(len.saturating_sub(1) as isize - current);
                    }
                }
            },
            _ => {}
        }
    }
}
