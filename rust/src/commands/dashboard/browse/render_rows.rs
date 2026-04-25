#![cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use super::super::browse_support::{DashboardBrowseNode, DashboardBrowseNodeKind};

pub(super) fn build_live_tree_items(nodes: &[DashboardBrowseNode]) -> Vec<ListItem<'_>> {
    let mut rendered = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        if node.kind == DashboardBrowseNodeKind::Org {
            rendered.push(render_live_org_row(node, index));
            continue;
        }

        rendered.push(render_tree_row(node));
    }
    rendered
}

pub(super) fn build_local_export_tree_items(nodes: &[DashboardBrowseNode]) -> Vec<ListItem<'_>> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| render_local_export_row(node, index))
        .collect()
}

fn render_live_org_row(node: &DashboardBrowseNode, index: usize) -> ListItem<'_> {
    let divider = Line::from(vec![
        Span::styled("──── ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            node.org_name.clone(),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " ─────────────────────",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
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
            format!("{} ", node.title),
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("│ id={} │ {}", node.org_id, node.meta),
            Style::default().fg(Color::Gray),
        ),
    ]);
    if index > 0 {
        ListItem::new(vec![Line::from(Span::raw(" ")), divider, line])
    } else {
        ListItem::new(vec![divider, line])
    }
}

fn render_local_export_row(node: &DashboardBrowseNode, index: usize) -> ListItem<'_> {
    match node.kind {
        DashboardBrowseNodeKind::Org => render_live_org_row(node, index),
        DashboardBrowseNodeKind::Folder | DashboardBrowseNodeKind::Dashboard => {
            render_tree_row(node)
        }
    }
}

fn render_tree_row(node: &DashboardBrowseNode) -> ListItem<'_> {
    let prefix = match node.kind {
        DashboardBrowseNodeKind::Folder => "+",
        DashboardBrowseNodeKind::Dashboard => "-",
        DashboardBrowseNodeKind::Org => "",
    };
    let line = Line::from(vec![
        Span::styled("     ", Style::default().fg(Color::DarkGray)),
        Span::raw(format!("{}{} ", "  ".repeat(node.depth), prefix)),
        Span::styled(
            node.title.clone(),
            Style::default()
                .fg(node_color(node))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  │  {}", node.meta),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    ListItem::new(line)
}

fn node_color(node: &DashboardBrowseNode) -> Color {
    match node.kind {
        DashboardBrowseNodeKind::Org => Color::LightCyan,
        DashboardBrowseNodeKind::Folder => Color::Cyan,
        DashboardBrowseNodeKind::Dashboard => Color::Yellow,
    }
}
