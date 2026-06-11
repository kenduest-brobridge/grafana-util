//! Shared browser detail projection helpers.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]

#[cfg(feature = "tui")]
use crate::tui_shell;

#[cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Line, Span};

#[cfg(any(feature = "tui", test))]
pub(crate) fn append_browser_detail_section(
    details: &mut Vec<String>,
    heading: &str,
    lines: Vec<String>,
) {
    if lines.is_empty() {
        details.push(format!("{heading}: none"));
        return;
    }
    details.push(format!("{heading}:"));
    details.extend(lines);
}

pub(crate) fn browser_detail_fact(label: &str, value: impl std::fmt::Display) -> String {
    format!("{label}: {value}")
}

pub(crate) fn browser_detail_fallback_fact(label: &str, value: &str, fallback: &str) -> String {
    let value = value.trim();
    let value = if value.is_empty() { fallback } else { value };
    format!("{label}: {value}")
}

#[cfg(any(feature = "tui", test))]
pub(crate) fn browser_detail_aligned_fact(label: &str, value: impl std::fmt::Display) -> String {
    format!("{label:<16}: {value}")
}

#[cfg(feature = "tui")]
pub(crate) fn browser_detail_info_lines(lines: &[String]) -> Vec<Line<'static>> {
    browser_detail_info_lines_with(lines, |_| true, |_| None)
}

#[cfg(feature = "tui")]
pub(crate) fn browser_detail_info_line(label: &str, value: &str, fallback: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:<18}: "),
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if value.trim().is_empty() {
                fallback.to_string()
            } else {
                value.trim().to_string()
            },
            Style::default().fg(Color::White),
        ),
    ])
}

#[cfg(feature = "tui")]
pub(crate) fn browser_detail_info_lines_with(
    lines: &[String],
    include_line: impl Fn(&str) -> bool,
    special_line: impl Fn(&str) -> Option<Line<'static>>,
) -> Vec<Line<'static>> {
    lines
        .iter()
        .filter(|line| !line.is_empty() && include_line(line))
        .map(|line| {
            if let Some(line) = special_line(line) {
                line
            } else if let Some((label, value)) = line.split_once(':') {
                Line::from(vec![
                    Span::styled(
                        format!("{label:<18}: "),
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(value.trim().to_string(), Style::default().fg(Color::White)),
                ])
            } else {
                Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect()
}

#[cfg(feature = "tui")]
pub(crate) fn browser_review_empty_line(message: &str) -> Line<'static> {
    Line::from(vec![
        tui_shell::muted("REVIEW "),
        tui_shell::plain(message.to_string()),
    ])
}

#[cfg(feature = "tui")]
pub(crate) fn browser_review_info_lines(lines: &[String]) -> Vec<Line<'static>> {
    lines
        .iter()
        .map(|line| {
            if let Some((label, value)) = line.split_once(':') {
                let color = if label.contains("blocker") || label.contains("required") {
                    Color::Yellow
                } else {
                    Color::LightCyan
                };
                Line::from(vec![
                    Span::styled(
                        format!("{label:<24}: "),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(value.trim().to_string(), Style::default().fg(Color::White)),
                ])
            } else {
                Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect()
}

#[cfg(feature = "tui")]
pub(crate) fn browser_wrapped_labeled_detail_lines(
    label: &str,
    value: &str,
    label_width: usize,
    width: usize,
    wrapped: bool,
) -> Vec<Line<'static>> {
    let prefix = format!("{label:<label_width$}: ");
    if !wrapped || width <= prefix.len().saturating_add(1) {
        return vec![Line::from(vec![
            Span::styled(
                prefix,
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(value.to_string(), Style::default().fg(Color::White)),
        ])];
    }
    let first_width = width.saturating_sub(prefix.len()).max(1);
    let continuation_prefix = " ".repeat(prefix.len());
    let chunks = wrap_text_chunks(value, first_width);
    chunks
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            if index == 0 {
                Line::from(vec![
                    Span::styled(
                        prefix.clone(),
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(chunk, Style::default().fg(Color::White)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        continuation_prefix.clone(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(chunk, Style::default().fg(Color::White)),
                ])
            }
        })
        .collect()
}

#[cfg(feature = "tui")]
pub(crate) fn wrap_text_chunks(value: &str, width: usize) -> Vec<String> {
    if width == 0 || value.is_empty() {
        return vec![value.to_string()];
    }
    let chars = value.chars().collect::<Vec<_>>();
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
}
