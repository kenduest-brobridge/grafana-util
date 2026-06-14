//! Access-wide interactive browse model and TUI shell.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::Result;
#[cfg(feature = "tui")]
use crate::tui_shell;

use super::render::{
    map_get_text, normalize_org_role, normalize_service_account_row, normalize_team_row,
    normalize_user_row, scalar_text,
};
use super::{request_array, AccessBrowseArgs, Scope};

#[cfg(feature = "tui")]
use std::time::Duration;

#[cfg(feature = "tui")]
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[cfg(feature = "tui")]
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

#[cfg(feature = "tui")]
use super::browse_terminal::TerminalSession;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccessBrowseItem {
    pub(crate) kind: String,
    pub(crate) identity: String,
    pub(crate) summary: String,
    pub(crate) review: String,
}

impl AccessBrowseItem {
    fn matches_query(&self, query: &str) -> bool {
        let query = query.to_ascii_lowercase();
        self.kind.to_ascii_lowercase().contains(&query)
            || self.identity.to_ascii_lowercase().contains(&query)
            || self.summary.to_ascii_lowercase().contains(&query)
            || self.review.to_ascii_lowercase().contains(&query)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccessBrowseState {
    items: Vec<AccessBrowseItem>,
    visible_indices: Vec<usize>,
    selected: usize,
    query: String,
}

impl AccessBrowseState {
    pub(crate) fn new(items: Vec<AccessBrowseItem>) -> Self {
        let visible_indices = (0..items.len()).collect();
        Self {
            items,
            visible_indices,
            selected: 0,
            query: String::new(),
        }
    }

    fn rebuild_visible_indices(&mut self) {
        let query = self.query.trim();
        self.visible_indices = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                if query.is_empty() || item.matches_query(query) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        self.clamp_selection();
    }

    fn clamp_selection(&mut self) {
        self.selected = self
            .selected
            .min(self.visible_indices.len().saturating_sub(1));
    }

    #[cfg(test)]
    fn apply_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.selected = 0;
        self.rebuild_visible_indices();
    }

    #[cfg(feature = "tui")]
    fn push_query_char(&mut self, value: char) {
        self.query.push(value);
        self.rebuild_visible_indices();
    }

    #[cfg(feature = "tui")]
    fn pop_query_char(&mut self) {
        self.query.pop();
        self.rebuild_visible_indices();
    }

    pub(crate) fn move_down(&mut self) {
        self.selected = (self.selected + 1).min(self.visible_indices.len().saturating_sub(1));
    }

    pub(crate) fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub(crate) fn selected_item(&self) -> Option<&AccessBrowseItem> {
        self.visible_indices
            .get(self.selected)
            .and_then(|index| self.items.get(*index))
    }

    pub(crate) fn visible_items(&self) -> Vec<&AccessBrowseItem> {
        self.visible_indices
            .iter()
            .filter_map(|index| self.items.get(*index))
            .collect()
    }

    pub(crate) fn summary_line(&self) -> String {
        let prefix = if self.query.trim().is_empty() {
            format!(
                "Showing {} of {} access rows.",
                self.visible_indices.len(),
                self.items.len()
            )
        } else {
            format!(
                "Filter \"{}\" matched {} of {} access rows.",
                self.query.trim(),
                self.visible_indices.len(),
                self.items.len()
            )
        };

        match self.selected_item() {
            Some(item) => format!(
                "{} Selected {}/{} {} {}.",
                prefix,
                self.selected + 1,
                self.visible_indices.len(),
                item.kind,
                item.identity
            ),
            None => format!("{prefix} No row selected."),
        }
    }

    #[cfg(feature = "tui")]
    fn detail_text(&self) -> String {
        self.selected_item()
            .map(|item| format!("{}: {}\n{}", item.kind, item.identity, item.review))
            .unwrap_or_else(|| "No access inventory rows matched.".to_string())
    }
}

#[cfg(feature = "tui")]
fn search_summary(state: &AccessBrowseState) -> String {
    let query = state.query.trim();
    if query.is_empty() {
        "off".to_string()
    } else {
        query.to_string()
    }
}

#[cfg(feature = "tui")]
fn build_header_lines(state: &AccessBrowseState, editing_query: bool) -> Vec<Line<'static>> {
    vec![
        tui_shell::summary_line(&[
            tui_shell::summary_cell(
                "Rows",
                state.visible_indices.len().to_string(),
                Color::White,
            ),
            tui_shell::summary_cell("Total", state.items.len().to_string(), Color::White),
            tui_shell::summary_cell(
                "Mode",
                if editing_query { "search" } else { "browse" },
                if editing_query {
                    Color::Yellow
                } else {
                    Color::Green
                },
            ),
        ]),
        Line::from(vec![
            tui_shell::label("Search "),
            tui_shell::accent(search_summary(state), Color::LightMagenta),
            Span::raw("  "),
            tui_shell::label("Selection "),
            tui_shell::accent(
                if state.visible_indices.is_empty() {
                    "0/0".to_string()
                } else {
                    format!("{}/{}", state.selected + 1, state.visible_indices.len())
                },
                Color::White,
            ),
        ]),
    ]
}

#[cfg(feature = "tui")]
fn build_footer_lines(editing_query: bool) -> Vec<Line<'static>> {
    if editing_query {
        tui_shell::control_grid(&[vec![
            ("Type", Color::LightBlue, "add filter text"),
            ("Backspace", Color::Yellow, "remove filter text"),
            ("Enter/Esc", Color::Green, "leave search"),
        ]])
    } else {
        tui_shell::control_grid(&[vec![
            ("Up/Down", Color::Blue, "move"),
            ("j/k", Color::Blue, "vim-style movement"),
            ("/", Color::Yellow, "search"),
            ("Esc/q", Color::DarkGray, "exit"),
        ]])
    }
}

#[cfg(feature = "tui")]
fn shell_status(state: &AccessBrowseState, editing_query: bool) -> String {
    let search = search_summary(state);
    if editing_query {
        format!("Mode=search   Search={search}   Type to filter. Enter/Esc returns to browse.")
    } else if search == "off" {
        "Mode=browse   Search=off   Press / to filter. Esc/q exits.".to_string()
    } else {
        format!("Mode=browse   Search={search}   Press / to refine. Esc/q exits.")
    }
}

#[cfg(feature = "tui")]
fn header_height(line_count: usize) -> u16 {
    line_count.saturating_add(2).max(3).min(u16::MAX as usize) as u16
}

fn enabled(args: &AccessBrowseArgs, flag: bool) -> bool {
    if args.include_users
        || args.include_teams
        || args.include_orgs
        || args.include_service_accounts
    {
        flag
    } else {
        true
    }
}

fn user_identity(row: &Map<String, Value>) -> String {
    let login = map_get_text(row, "login");
    if !login.is_empty() {
        return login;
    }
    let email = map_get_text(row, "email");
    if !email.is_empty() {
        return email;
    }
    scalar_text(row.get("id"))
}

fn shape_user_item(user: &Map<String, Value>, scope: &Scope) -> AccessBrowseItem {
    let row = normalize_user_row(user, scope);
    AccessBrowseItem {
        kind: "user".to_string(),
        identity: user_identity(&row),
        summary: format!(
            "email={} role={} admin={} scope={}",
            map_get_text(&row, "email"),
            map_get_text(&row, "orgRole"),
            map_get_text(&row, "grafanaAdmin"),
            map_get_text(&row, "scope")
        ),
        review: "review user org role, admin state, identity, and team membership".to_string(),
    }
}

fn shape_team_item(team: &Map<String, Value>) -> AccessBrowseItem {
    let row = normalize_team_row(team);
    AccessBrowseItem {
        kind: "team".to_string(),
        identity: map_get_text(&row, "name"),
        summary: format!(
            "id={} email={} members={}",
            map_get_text(&row, "id"),
            map_get_text(&row, "email"),
            map_get_text(&row, "memberCount")
        ),
        review: "review team contact and membership count before import or sync".to_string(),
    }
}

fn shape_org_item(org: &Map<String, Value>) -> AccessBrowseItem {
    let id = scalar_text(org.get("id").or_else(|| org.get("orgId")));
    let name = map_get_text(org, "name");
    let user_count = {
        let count = scalar_text(org.get("userCount"));
        if count.is_empty() {
            match org.get("users") {
                Some(Value::Array(users)) => users.len().to_string(),
                _ => "0".to_string(),
            }
        } else {
            count
        }
    };
    AccessBrowseItem {
        kind: "org".to_string(),
        identity: if name.is_empty() { id.clone() } else { name },
        summary: format!("id={id} users={user_count}"),
        review: "review org identity and membership count before global changes".to_string(),
    }
}

fn shape_service_account_item(account: &Map<String, Value>) -> AccessBrowseItem {
    let row = normalize_service_account_row(account);
    AccessBrowseItem {
        kind: "service-account".to_string(),
        identity: map_get_text(&row, "name"),
        summary: format!(
            "login={} role={} disabled={} tokens={} orgId={}",
            map_get_text(&row, "login"),
            normalize_org_role(row.get("role")),
            map_get_text(&row, "disabled"),
            map_get_text(&row, "tokens"),
            map_get_text(&row, "orgId")
        ),
        review: "review service-account role, disabled state, and token metadata only".to_string(),
    }
}

pub(crate) fn build_access_browse_items<F>(
    mut request_json: F,
    args: &AccessBrowseArgs,
) -> Result<Vec<AccessBrowseItem>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut items = Vec::new();
    if enabled(args, args.include_users) {
        let users = super::user::iter_global_users_with_request(&mut request_json, args.per_page)?;
        items.extend(
            users
                .iter()
                .map(|user| shape_user_item(user, &Scope::Global)),
        );
    }
    if enabled(args, args.include_teams) {
        let teams = super::team::iter_teams_with_request(&mut request_json, args.query.as_deref())?;
        items.extend(teams.iter().map(shape_team_item));
    }
    if enabled(args, args.include_orgs) {
        let orgs = request_array(
            &mut request_json,
            Method::GET,
            "/api/orgs",
            &[],
            None,
            "Unexpected organization list response from Grafana.",
        )?;
        items.extend(orgs.iter().map(shape_org_item));
    }
    if enabled(args, args.include_service_accounts) {
        let accounts =
            super::service_account::list_all_service_accounts_with_request(&mut request_json)?;
        items.extend(accounts.iter().map(shape_service_account_item));
    }
    if let Some(query) = args.query.as_deref() {
        let query = query.trim();
        if !query.is_empty() {
            items.retain(|item| item.matches_query(query));
        }
    }
    Ok(items)
}

#[cfg(feature = "tui")]
pub(crate) fn browse_access_with_request<F>(request_json: F, args: &AccessBrowseArgs) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let items = build_access_browse_items(request_json, args)?;
    let mut state = AccessBrowseState::new(items);
    let mut session = TerminalSession::enter()?;
    let mut editing_query = false;
    loop {
        let mut list_state = ListState::default();
        if !state.visible_indices.is_empty() {
            list_state.select(Some(state.selected));
        }
        session.terminal.draw(|frame| {
            let header_lines = build_header_lines(&state, editing_query);
            let footer_lines = build_footer_lines(editing_query);
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(header_height(header_lines.len())),
                    Constraint::Min(5),
                    Constraint::Length(tui_shell::footer_height(footer_lines.len())),
                ])
                .split(frame.area());
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
                .split(outer[1]);
            let rows = state
                .visible_items()
                .into_iter()
                .map(|item| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<15}", item.kind),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!(" {:<32} ", item.identity)),
                        Span::raw(&item.summary),
                    ]))
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                tui_shell::build_header("Access Browser", header_lines),
                outer[0],
            );

            let list = List::new(rows)
                .block(tui_shell::pane_block(
                    "Inventory",
                    true,
                    Color::LightBlue,
                    Color::Rgb(16, 20, 27),
                ))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            frame.render_stateful_widget(list, panes[0], &mut list_state);

            let detail = format!("{}\n{}", state.summary_line(), state.detail_text());
            frame.render_widget(
                Paragraph::new(detail).block(tui_shell::pane_block(
                    "Facts",
                    false,
                    Color::LightCyan,
                    Color::Rgb(16, 20, 27),
                )),
                panes[1],
            );

            frame.render_widget(
                tui_shell::build_footer(footer_lines, shell_status(&state, editing_query)),
                outer[2],
            );
        })?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if editing_query {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => editing_query = false,
                KeyCode::Backspace => state.pop_query_char(),
                KeyCode::Char(value) => state.push_query_char(value),
                _ => {}
            }
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Char('/') => editing_query = true,
            KeyCode::Down | KeyCode::Char('j') => state.move_down(),
            KeyCode::Up | KeyCode::Char('k') => state.move_up(),
            _ => {}
        }
    }
}

#[cfg(not(feature = "tui"))]
pub(crate) fn browse_access_with_request<F>(
    _request_json: F,
    _args: &AccessBrowseArgs,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    Err(crate::common::tui_feature_required("Access browse"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::message;
    use serde_json::json;

    #[test]
    fn service_account_item_shows_token_metadata_without_secrets() {
        let account = json!({
            "id": 7,
            "name": "deploy",
            "login": "sa-deploy",
            "role": "Editor",
            "isDisabled": false,
            "tokens": 2,
            "key": "secret-token-value"
        });
        let item = shape_service_account_item(account.as_object().unwrap());

        assert_eq!(item.kind, "service-account");
        assert_eq!(item.identity, "deploy");
        assert!(item.summary.contains("tokens=2"));
        assert!(item.review.contains("token metadata only"));
        assert!(!item.summary.contains("secret-token-value"));
        assert!(!item.review.contains("secret-token-value"));
    }

    #[test]
    fn access_browse_items_can_consolidate_all_live_resource_rows() {
        let args = AccessBrowseArgs {
            common: super::super::CommonCliArgs {
                profile: None,
                url: "http://127.0.0.1:3000".to_string(),
                api_token: Some("token".to_string()),
                username: None,
                password: None,
                prompt_password: false,
                prompt_token: false,
                org_id: None,
                timeout: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            query: None,
            include_users: true,
            include_teams: true,
            include_orgs: true,
            include_service_accounts: true,
            per_page: 100,
        };
        let mut calls = Vec::new();
        let items = build_access_browse_items(
            |method, path, _params, _payload| {
                calls.push((method, path.to_string()));
                match path {
                    "/api/users" => Ok(Some(json!([
                        {"id": 1, "login": "alice", "email": "alice@example.com", "isAdmin": true}
                    ]))),
                    "/api/teams/search" => Ok(Some(json!({
                        "teams": [{"id": 2, "name": "ops", "email": "ops@example.com", "memberCount": 3}]
                    }))),
                    "/api/orgs" => Ok(Some(json!([
                        {"id": 3, "name": "Main Org.", "userCount": 9}
                    ]))),
                    "/api/serviceaccounts/search" => Ok(Some(json!({
                        "serviceAccounts": [{"id": 4, "name": "deploy", "login": "sa-deploy", "role": "Viewer", "tokens": 1}]
                    }))),
                    _ => Err(message(format!("unexpected request {path}"))),
                }
            },
            &args,
        )
        .unwrap();

        assert_eq!(
            items
                .iter()
                .map(|item| item.kind.as_str())
                .collect::<Vec<_>>(),
            vec!["user", "team", "org", "service-account"]
        );
        assert!(calls.iter().any(|(_, path)| path == "/api/orgs"));
    }

    #[test]
    fn access_browse_state_filters_rows_and_reports_selection_summary() {
        let items = vec![
            AccessBrowseItem {
                kind: "user".to_string(),
                identity: "alice".to_string(),
                summary: "email=alice@example.com role=Admin".to_string(),
                review: "review user org role and admin state".to_string(),
            },
            AccessBrowseItem {
                kind: "team".to_string(),
                identity: "ops".to_string(),
                summary: "id=2 email=ops@example.com members=3".to_string(),
                review: "review team contact and membership count".to_string(),
            },
            AccessBrowseItem {
                kind: "service-account".to_string(),
                identity: "deploy".to_string(),
                summary: "login=sa-deploy role=Viewer disabled=false tokens=1 orgId=1".to_string(),
                review: "review service-account role and token metadata only".to_string(),
            },
        ];
        let mut state = AccessBrowseState::new(items);

        assert_eq!(
            state.summary_line(),
            "Showing 3 of 3 access rows. Selected 1/3 user alice."
        );
        state.move_down();
        state.apply_query("token");

        assert_eq!(state.visible_items().len(), 1);
        assert_eq!(state.selected_item().unwrap().identity, "deploy");
        assert_eq!(
            state.summary_line(),
            "Filter \"token\" matched 1 of 3 access rows. Selected 1/1 service-account deploy."
        );

        state.apply_query("missing");

        assert!(state.selected_item().is_none());
        assert_eq!(
            state.summary_line(),
            "Filter \"missing\" matched 0 of 3 access rows. No row selected."
        );
    }

    #[cfg(feature = "tui")]
    #[test]
    fn access_browse_header_surfaces_mode_and_search_summary() {
        let items = vec![AccessBrowseItem {
            kind: "user".to_string(),
            identity: "alice".to_string(),
            summary: "email=alice@example.com role=Admin".to_string(),
            review: "review user org role and admin state".to_string(),
        }];
        let mut state = AccessBrowseState::new(items);

        let browse_lines = build_header_lines(&state, false);
        let browse_text = browse_lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(browse_text.contains("Mode"));
        assert!(browse_text.contains("browse"));
        assert!(browse_text.contains("Search"));
        assert!(browse_text.contains("off"));
        assert!(browse_text.contains("Selection"));
        assert!(browse_text.contains("1/1"));

        state.apply_query("alice");
        let search_lines = build_header_lines(&state, true);
        let search_text = search_lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(search_text.contains("Mode"));
        assert!(search_text.contains("search"));
        assert!(search_text.contains("Search"));
        assert!(search_text.contains("alice"));
    }

    #[cfg(feature = "tui")]
    #[test]
    fn access_browse_footer_uses_shared_control_copy() {
        let lines = build_footer_lines(false);
        let text = lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(text.contains("Up/Down"));
        assert!(text.contains("move"));
        assert!(text.contains("j/k"));
        assert!(text.contains("vim-style movement"));
        assert!(text.contains("/"));
        assert!(text.contains("search"));
        assert!(text.contains("Esc/q"));
        assert!(text.contains("exit"));
    }

    #[cfg(feature = "tui")]
    #[test]
    fn access_browse_search_mode_footer_surfaces_prompt_controls() {
        let text = build_footer_lines(true)
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(text.contains("Type"));
        assert!(text.contains("add filter text"));
        assert!(text.contains("Backspace"));
        assert!(text.contains("remove filter text"));
        assert!(text.contains("Enter/Esc"));
        assert!(text.contains("leave search"));
    }
}
