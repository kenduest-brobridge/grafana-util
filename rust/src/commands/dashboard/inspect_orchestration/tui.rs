//! Terminal UI for dashboard inspect-export orchestration.

use std::io::{self, IsTerminal};
use std::path::Path;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::common::{message, Result};
use crate::tui_shell;

use super::super::super::browse_terminal::TerminalSession;
use super::super::super::cli_defs::InspectExportInputType;
use super::super::super::files::DashboardSourceKind;
use super::super::super::inspect_governance::build_export_inspection_governance_document;
use super::super::super::inspect_workbench::run_inspect_workbench;
use super::super::super::inspect_workbench_support::build_inspect_workbench_document;
use super::super::super::RAW_EXPORT_SUBDIR;
use super::super::inspect_query_report::build_export_inspection_query_report_for_variant;

fn centered_popup_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_width = area.width.saturating_sub(8).min(width).max(72);
    let popup_height = area.height.saturating_sub(4).min(height).max(12);
    Rect {
        x: area.x + area.width.saturating_sub(popup_width) / 2,
        y: area.y + area.height.saturating_sub(popup_height) / 2,
        width: popup_width,
        height: popup_height,
    }
}

fn render_interactive_loading_frame(
    frame: &mut ratatui::Frame<'_>,
    input_dir: &Path,
    expected_variant: &str,
    source_kind: Option<DashboardSourceKind>,
    active_step: usize,
) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    let popup = centered_popup_rect(area, 88, 16);
    let inner = popup.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(7),
            Constraint::Length(3),
        ])
        .split(inner);

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title("Inspect Export")
            .border_style(Style::default().fg(Color::LightBlue))
            .style(Style::default().bg(Color::Rgb(8, 12, 18))),
        popup,
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                tui_shell::label("Stage "),
                tui_shell::accent("Preparing interactive workbench", Color::Cyan),
            ]),
            Line::from(vec![
                tui_shell::label("Source "),
                tui_shell::plain(input_dir.display().to_string()),
            ]),
            Line::from(vec![
                tui_shell::label("Variant "),
                match source_kind {
                    Some(DashboardSourceKind::RawExport) => {
                        tui_shell::key_chip("RAW", Color::Rgb(78, 161, 255))
                    }
                    Some(DashboardSourceKind::ProvisioningExport) => {
                        tui_shell::key_chip("PROVISIONING", Color::Rgb(73, 182, 133))
                    }
                    _ if expected_variant == RAW_EXPORT_SUBDIR => {
                        tui_shell::key_chip("RAW", Color::Rgb(78, 161, 255))
                    }
                    _ => tui_shell::key_chip("SOURCE", Color::Rgb(73, 182, 133)),
                },
            ]),
            Line::from("Building inspection artifacts before opening the interactive browser."),
        ])
        .wrap(Wrap { trim: false }),
        chunks[0],
    );

    let steps = [
        "Build summary",
        "Build query report",
        "Build governance review",
        "Launch inspect workbench",
    ];
    let items = steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let (marker, style, text_color) = if index < active_step {
                (
                    " DONE ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                    Color::White,
                )
            } else if index == active_step {
                (
                    " NOW  ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                    Color::White,
                )
            } else {
                (
                    " WAIT ",
                    Style::default().fg(Color::Black).bg(Color::DarkGray),
                    Color::Gray,
                )
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {marker} "), style),
                Span::raw(" "),
                Span::styled(
                    (*step).to_string(),
                    Style::default()
                        .fg(text_color)
                        .add_modifier(if index == active_step {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
            ]))
        })
        .collect::<Vec<ListItem>>();
    frame.render_widget(List::new(items), chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            tui_shell::label("Status "),
            tui_shell::plain(
                "Loading is automatic. The workbench opens when preparation completes.",
            ),
        ])),
        chunks[2],
    );
}

fn draw_interactive_loading_step(
    session: &mut TerminalSession,
    input_dir: &Path,
    expected_variant: &str,
    source_kind: Option<DashboardSourceKind>,
    active_step: usize,
) -> Result<()> {
    session.terminal.draw(|frame| {
        render_interactive_loading_frame(
            frame,
            input_dir,
            expected_variant,
            source_kind,
            active_step,
        )
    })?;
    Ok(())
}

// Interactive selector for dual input variant (raw/source) before opening inspect workbench.
fn run_interactive_input_type_selector(input_dir: &Path) -> Result<InspectExportInputType> {
    let mut session = TerminalSession::enter()?;
    let options = [
        (
            InspectExportInputType::Raw,
            "raw",
            "Inspect API-safe raw export artifacts",
        ),
        (
            InspectExportInputType::Source,
            "source",
            "Inspect prompt/source export artifacts",
        ),
    ];
    let mut selected = 0usize;

    loop {
        session.terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(Clear, area);
            let popup = centered_popup_rect(area, 88, 17);
            let inner = popup.inner(Margin {
                vertical: 1,
                horizontal: 3,
            });
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7),
                    Constraint::Length(5),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(inner);

            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Inspect export input")
                    .border_style(Style::default().fg(Color::LightBlue))
                    .style(Style::default().bg(Color::Black)),
                popup,
            );

            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(vec![
                        tui_shell::label("Title "),
                        tui_shell::accent("Choose dashboard export variant", Color::Cyan),
                    ]),
                    Line::from(vec![
                        tui_shell::label("Import "),
                        tui_shell::plain(input_dir.display().to_string()),
                    ]),
                    Line::from(""),
                    Line::from(
                        "This dashboard export root contains both raw/ and prompt/ variants.",
                    ),
                    Line::from("Select one variant before continuing into the inspect workbench."),
                ])
                .wrap(Wrap { trim: false }),
                chunks[0],
            );

            let items = options
                .iter()
                .enumerate()
                .map(|(index, (_, label, detail))| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{}. {label}", index + 1),
                            Style::default()
                                .fg(Color::LightCyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(format!("({detail})"), Style::default().fg(Color::White)),
                    ]))
                })
                .collect::<Vec<ListItem>>();
            let mut state = ListState::default().with_selected(Some(selected));
            frame.render_stateful_widget(
                List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Options")
                            .border_style(Style::default().fg(Color::Gray)),
                    )
                    .highlight_symbol("   ")
                    .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
                chunks[1],
                &mut state,
            );

            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(vec![
                        tui_shell::label("Choice "),
                        tui_shell::plain(format!("{}. {}", selected + 1, options[selected].1)),
                    ]),
                    Line::from(vec![
                        tui_shell::key_chip("Up/Down", Color::Blue),
                        Span::raw(" move  "),
                        tui_shell::key_chip("Enter", Color::Green),
                        Span::raw(" confirm  "),
                        tui_shell::key_chip("Esc/q", Color::DarkGray),
                        Span::raw(" cancel"),
                    ]),
                ]),
                chunks[3],
            );
        })?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1).min(options.len().saturating_sub(1));
            }
            KeyCode::Enter => return Ok(options[selected].0),
            KeyCode::Esc | KeyCode::Char('q') => {
                return Err(message("Interactive inspect selection cancelled."));
            }
            _ => {}
        }
    }
}

pub(super) fn prompt_interactive_input_type(input_dir: &Path) -> Result<InspectExportInputType> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(message(format!(
            "Import path {} contains both raw/ and prompt/ dashboard variants. Re-run with --input-type raw or --input-type source.",
            input_dir.display()
        )));
    }
    run_interactive_input_type_selector(input_dir)
}

// Render export inspection in an interactive workbench; shared with non-interactive
// and local-mode call-sites via the same dashboard-count return contract.
pub(super) fn run_interactive_export_workbench(
    input_dir: &Path,
    expected_variant: &str,
    source_kind: Option<DashboardSourceKind>,
) -> Result<usize> {
    let mut session = TerminalSession::enter()?;
    draw_interactive_loading_step(&mut session, input_dir, expected_variant, source_kind, 0)?;
    let summary = super::super::super::build_export_inspection_summary_for_variant(
        input_dir,
        expected_variant,
    )?;
    draw_interactive_loading_step(&mut session, input_dir, expected_variant, source_kind, 1)?;
    let report = build_export_inspection_query_report_for_variant(input_dir, expected_variant)?;
    draw_interactive_loading_step(&mut session, input_dir, expected_variant, source_kind, 2)?;
    let governance = build_export_inspection_governance_document(&summary, &report);
    draw_interactive_loading_step(&mut session, input_dir, expected_variant, source_kind, 3)?;
    let document =
        build_inspect_workbench_document("export artifacts", &summary, &governance, &report);
    drop(session);
    run_inspect_workbench(document)?;
    Ok(summary.dashboard_count)
}
