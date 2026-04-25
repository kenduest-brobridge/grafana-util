use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub(super) struct ViewerRenderRow {
    pub(super) logical_index: usize,
    pub(super) line: Line<'static>,
}

pub(super) fn viewer_rows(lines: Vec<String>, width: usize, wrapped: bool) -> Vec<ViewerRenderRow> {
    lines
        .into_iter()
        .enumerate()
        .flat_map(|(logical_index, line)| {
            if line.trim().is_empty() {
                return vec![ViewerRenderRow {
                    logical_index,
                    line: Line::from(""),
                }];
            }
            if let Some((label, value)) = line.split_once(':') {
                let prefix = format!("{label:<16}: ");
                return wrap_labeled_viewer_line(&prefix, value.trim(), width, wrapped)
                    .into_iter()
                    .map(|line| ViewerRenderRow {
                        logical_index,
                        line,
                    })
                    .collect::<Vec<_>>();
            }
            wrap_plain_viewer_line(&line, width, wrapped)
                .into_iter()
                .map(|line| ViewerRenderRow {
                    logical_index,
                    line,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn wrap_labeled_viewer_line(
    prefix: &str,
    value: &str,
    width: usize,
    wrapped: bool,
) -> Vec<Line<'static>> {
    if !wrapped || width <= prefix.len().saturating_add(1) {
        return vec![Line::from(vec![
            Span::styled(
                prefix.to_string(),
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
                        prefix.to_string(),
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

fn wrap_plain_viewer_line(line: &str, width: usize, wrapped: bool) -> Vec<Line<'static>> {
    if !wrapped || width == 0 {
        return vec![Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::White),
        ))];
    }
    wrap_text_chunks(line, width.max(1))
        .into_iter()
        .map(|chunk| Line::from(Span::styled(chunk, Style::default().fg(Color::White))))
        .collect()
}

fn wrap_text_chunks(value: &str, width: usize) -> Vec<String> {
    if width == 0 || value.is_empty() {
        return vec![value.to_string()];
    }
    let chars = value.chars().collect::<Vec<_>>();
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
}
