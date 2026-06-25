use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use super::super::datasource_browse_support::DatasourceBrowseItem;
use super::render_summary::blank_dash;

pub(super) fn build_list_items(items: &[DatasourceBrowseItem]) -> Vec<ListItem<'_>> {
    let mut rendered = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if item.is_org_row() {
            let line = Line::from(vec![
                Span::styled(
                    " ORG ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(46, 66, 98))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{} ", item.org),
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "│ id={} │ {} datasource(s)",
                        item.org_id, item.datasource_count
                    ),
                    Style::default().fg(Color::Gray),
                ),
            ]);
            if index > 0 {
                rendered.push(ListItem::new(vec![Line::from(Span::raw(" ")), line]));
            } else {
                rendered.push(ListItem::new(line));
            }
            continue;
        }
        let badge_color = if item.is_default {
            Color::Green
        } else {
            Color::DarkGray
        };
        let branch = datasource_tree_branch(items, index);
        let line = Line::from(vec![
            Span::styled("     ", Style::default().fg(Color::DarkGray)),
            Span::styled(branch, Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(
                if item.is_default { "[*]" } else { "[ ]" },
                Style::default().fg(Color::White).bg(badge_color),
            ),
            Span::raw(" "),
            Span::styled(
                item.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  │  {}  │  {}",
                    blank_dash(&item.datasource_type),
                    blank_dash(&item.uid)
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        rendered.push(ListItem::new(line));
    }
    rendered
}

fn datasource_tree_branch(items: &[DatasourceBrowseItem], index: usize) -> &'static str {
    let has_next_datasource_in_same_group = items
        .get(index + 1)
        .map(|next| !next.is_org_row())
        .unwrap_or(false);
    if has_next_datasource_in_same_group {
        "├─"
    } else {
        "└─"
    }
}
