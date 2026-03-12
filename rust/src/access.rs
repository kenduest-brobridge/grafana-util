use clap::{Args, Command, CommandFactory, Parser, Subcommand, ValueEnum};
use reqwest::Method;
use serde_json::{Map, Value};
use std::fmt::Write as _;

use crate::common::{message, resolve_auth_headers, string_field, value_as_object, Result};
use crate::http::{JsonHttpClient, JsonHttpClientConfig};

pub const DEFAULT_URL: &str = "http://127.0.0.1:3000";
pub const DEFAULT_TIMEOUT: u64 = 30;
pub const DEFAULT_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone, Args)]
pub struct CommonCliArgs {
    #[arg(long, default_value = DEFAULT_URL, help = "Grafana base URL.")]
    pub url: String,
    #[arg(
        long = "token",
        visible_alias = "api-token",
        help = "Grafana API token. Preferred flag: --token. Falls back to GRAFANA_API_TOKEN."
    )]
    pub api_token: Option<String>,
    #[arg(
        long = "basic-user",
        visible_alias = "username",
        help = "Grafana Basic auth username. Preferred flag: --basic-user. Falls back to GRAFANA_USERNAME."
    )]
    pub username: Option<String>,
    #[arg(
        long = "basic-password",
        visible_alias = "password",
        help = "Grafana Basic auth password. Preferred flag: --basic-password. Falls back to GRAFANA_PASSWORD."
    )]
    pub password: Option<String>,
    #[arg(long, default_value_t = false, help = "Prompt for the Grafana Basic auth password.")]
    pub prompt_password: bool,
    #[arg(long, help = "Grafana organization id to send through X-Grafana-Org-Id.")]
    pub org_id: Option<i64>,
    #[arg(long, default_value_t = DEFAULT_TIMEOUT, help = "HTTP timeout in seconds.")]
    pub timeout: u64,
    #[arg(
        long,
        default_value_t = false,
        help = "Enable TLS certificate verification. Verification is disabled by default."
    )]
    pub verify_ssl: bool,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum Scope {
    Org,
    Global,
}

#[derive(Debug, Clone, Args)]
pub struct UserListArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long, value_enum, default_value_t = Scope::Org)]
    pub scope: Scope,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub login: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long)]
    pub org_role: Option<String>,
    #[arg(long, value_parser = parse_bool_text)]
    pub grafana_admin: Option<bool>,
    #[arg(long, default_value_t = false)]
    pub with_teams: bool,
    #[arg(long, default_value_t = 1)]
    pub page: usize,
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE)]
    pub per_page: usize,
    #[arg(long, default_value_t = false, conflicts_with_all = ["csv", "json"])]
    pub table: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "json"])]
    pub csv: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv"])]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UserAddArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long)]
    pub login: String,
    #[arg(long)]
    pub email: String,
    #[arg(long)]
    pub name: String,
    #[arg(long = "password")]
    pub new_user_password: String,
    #[arg(long = "org-role")]
    pub org_role: Option<String>,
    #[arg(long = "grafana-admin", value_parser = parse_bool_text)]
    pub grafana_admin: Option<bool>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UserModifyArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long, conflicts_with_all = ["login", "email"])]
    pub user_id: Option<String>,
    #[arg(long, conflicts_with_all = ["user_id", "email"])]
    pub login: Option<String>,
    #[arg(long, conflicts_with_all = ["user_id", "login"])]
    pub email: Option<String>,
    #[arg(long)]
    pub set_login: Option<String>,
    #[arg(long)]
    pub set_email: Option<String>,
    #[arg(long)]
    pub set_name: Option<String>,
    #[arg(long)]
    pub set_password: Option<String>,
    #[arg(long)]
    pub set_org_role: Option<String>,
    #[arg(long, value_parser = parse_bool_text)]
    pub set_grafana_admin: Option<bool>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UserDeleteArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long, conflicts_with_all = ["login", "email"])]
    pub user_id: Option<String>,
    #[arg(long, conflicts_with_all = ["user_id", "email"])]
    pub login: Option<String>,
    #[arg(long, conflicts_with_all = ["user_id", "login"])]
    pub email: Option<String>,
    #[arg(long, value_enum, default_value_t = Scope::Global)]
    pub scope: Scope,
    #[arg(long, default_value_t = false)]
    pub yes: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TeamListArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long, default_value_t = false)]
    pub with_members: bool,
    #[arg(long, default_value_t = 1)]
    pub page: usize,
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE)]
    pub per_page: usize,
    #[arg(long, default_value_t = false, conflicts_with_all = ["csv", "json"])]
    pub table: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "json"])]
    pub csv: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv"])]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TeamAddArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long = "member")]
    pub members: Vec<String>,
    #[arg(long = "admin")]
    pub admins: Vec<String>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TeamModifyArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long, conflicts_with = "name")]
    pub team_id: Option<String>,
    #[arg(long, conflicts_with = "team_id")]
    pub name: Option<String>,
    #[arg(long = "add-member")]
    pub add_member: Vec<String>,
    #[arg(long = "remove-member")]
    pub remove_member: Vec<String>,
    #[arg(long = "add-admin")]
    pub add_admin: Vec<String>,
    #[arg(long = "remove-admin")]
    pub remove_admin: Vec<String>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ServiceAccountListArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long, default_value_t = 1)]
    pub page: usize,
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE)]
    pub per_page: usize,
    #[arg(long, default_value_t = false, conflicts_with_all = ["csv", "json"])]
    pub table: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "json"])]
    pub csv: bool,
    #[arg(long, default_value_t = false, conflicts_with_all = ["table", "csv"])]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ServiceAccountAddArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "Viewer")]
    pub role: String,
    #[arg(long, value_parser = parse_bool_text, default_value = "false")]
    pub disabled: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ServiceAccountTokenAddArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(long, conflicts_with = "name")]
    pub service_account_id: Option<String>,
    #[arg(long, conflicts_with = "service_account_id")]
    pub name: Option<String>,
    #[arg(long)]
    pub token_name: String,
    #[arg(long)]
    pub seconds_to_live: Option<usize>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceAccountTokenCommand {
    Add(ServiceAccountTokenAddArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceAccountCommand {
    List(ServiceAccountListArgs),
    Add(ServiceAccountAddArgs),
    Token {
        #[command(subcommand)]
        command: ServiceAccountTokenCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum TeamCommand {
    List(TeamListArgs),
    Add(TeamAddArgs),
    Modify(TeamModifyArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum UserCommand {
    List(UserListArgs),
    Add(UserAddArgs),
    Modify(UserModifyArgs),
    Delete(UserDeleteArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum AccessCommand {
    User {
        #[command(subcommand)]
        command: UserCommand,
    },
    Team {
        #[command(subcommand)]
        command: TeamCommand,
    },
    #[command(name = "service-account")]
    ServiceAccount {
        #[command(subcommand)]
        command: ServiceAccountCommand,
    },
}

#[derive(Debug, Clone, Parser)]
#[command(name = "grafana-access-utils", about = "List and manage Grafana users, teams, and service accounts.")]
struct AccessCliRoot {
    #[command(flatten)]
    args: AccessCliArgs,
}

#[derive(Debug, Clone, Args)]
pub struct AccessCliArgs {
    #[command(subcommand)]
    pub command: AccessCommand,
}

pub fn parse_cli_from<I, T>(iter: I) -> AccessCliArgs
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    AccessCliRoot::parse_from(iter).args
}

pub fn root_command() -> Command {
    AccessCliRoot::command()
}

#[derive(Debug, Clone)]
pub struct AccessAuthContext {
    pub url: String,
    pub timeout: u64,
    pub verify_ssl: bool,
    pub auth_mode: String,
    pub headers: Vec<(String, String)>,
}

fn parse_bool_text(value: &str) -> std::result::Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("value must be true or false".to_string()),
    }
}

fn build_auth_context(common: &CommonCliArgs) -> Result<AccessAuthContext> {
    let mut headers = resolve_auth_headers(
        common.api_token.as_deref(),
        common.username.as_deref(),
        common.password.as_deref(),
        common.prompt_password,
    )?;
    if let Some(org_id) = common.org_id {
        headers.push(("X-Grafana-Org-Id".to_string(), org_id.to_string()));
    }
    let auth_mode = headers
        .iter()
        .find(|(name, _)| name == "Authorization")
        .map(|(_, value)| {
            if value.starts_with("Basic ") {
                "basic".to_string()
            } else {
                "token".to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    Ok(AccessAuthContext {
        url: common.url.clone(),
        timeout: common.timeout,
        verify_ssl: common.verify_ssl,
        auth_mode,
        headers,
    })
}

pub fn build_http_client(common: &CommonCliArgs) -> Result<JsonHttpClient> {
    let context = build_auth_context(common)?;
    JsonHttpClient::new(JsonHttpClientConfig {
        base_url: context.url,
        headers: context.headers,
        timeout_secs: context.timeout,
        verify_ssl: context.verify_ssl,
    })
}

fn request_object<F>(
    mut request_json: F,
    method: Method,
    path: &str,
    params: &[(String, String)],
    payload: Option<&Value>,
    error_message: &str,
) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let value = request_json(method, path, params, payload)?
        .ok_or_else(|| message(error_message.to_string()))?;
    Ok(value_as_object(&value, error_message)?.clone())
}

fn request_array<F>(
    mut request_json: F,
    method: Method,
    path: &str,
    params: &[(String, String)],
    payload: Option<&Value>,
    error_message: &str,
) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(method, path, params, payload)? {
        Some(Value::Array(items)) => items
            .into_iter()
            .map(|item| Ok(value_as_object(&item, error_message)?.clone()))
            .collect(),
        Some(_) => Err(message(error_message.to_string())),
        None => Ok(Vec::new()),
    }
}

fn bool_label(value: Option<bool>) -> String {
    match value {
        Some(true) => "true".to_string(),
        Some(false) => "false".to_string(),
        None => String::new(),
    }
}

fn scalar_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => String::new(),
    }
}

fn value_bool(value: Option<&Value>) -> Option<bool> {
    match value {
        Some(Value::Bool(v)) => Some(*v),
        Some(Value::String(text)) => match text.to_ascii_lowercase().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        Some(Value::Number(number)) => match number.as_i64() {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn normalize_org_role(value: Option<&Value>) -> String {
    let text = match value {
        Some(Value::String(text)) => text.trim(),
        _ => "",
    };
    match text.to_ascii_lowercase().as_str() {
        "" => String::new(),
        "nobasicrole" | "none" => "None".to_string(),
        lowered => {
            let mut chars = lowered.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        }
    }
}

fn service_account_role_to_api(role: &str) -> String {
    match role.trim().to_ascii_lowercase().as_str() {
        "none" => "NoBasicRole".to_string(),
        "viewer" => "Viewer".to_string(),
        "editor" => "Editor".to_string(),
        "admin" => "Admin".to_string(),
        other => other.to_string(),
    }
}

fn user_scope_text(scope: &Scope) -> &'static str {
    match scope {
        Scope::Org => "org",
        Scope::Global => "global",
    }
}

fn format_table(headers: &[&str], rows: &[Vec<String>]) -> Vec<String> {
    let mut widths: Vec<usize> = headers.iter().map(|header| header.len()).collect();
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            widths[index] = widths[index].max(value.len());
        }
    }
    let format_row = |values: &[String]| -> String {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{:<width$}", value, width = widths[index]))
            .collect::<Vec<String>>()
            .join("  ")
    };
    let header_row = headers.iter().map(|value| value.to_string()).collect::<Vec<String>>();
    let separator = widths.iter().map(|width| "-".repeat(*width)).collect::<Vec<String>>();
    let mut lines = vec![format_row(&header_row), format_row(&separator)];
    lines.extend(rows.iter().map(|row| format_row(row)));
    lines
}

fn csv_escape(value: String) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn render_csv(headers: &[&str], rows: &[Vec<String>]) -> Vec<String> {
    let mut lines = vec![headers.join(",")];
    lines.extend(rows.iter().map(|row| row.iter().cloned().map(csv_escape).collect::<Vec<String>>().join(",")));
    lines
}

fn list_org_users_with_request<F>(mut request_json: F) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_array(
        &mut request_json,
        Method::GET,
        "/api/org/users",
        &[],
        None,
        "Unexpected org user list response from Grafana.",
    )
}

fn iter_global_users_with_request<F>(mut request_json: F, page_size: usize) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut users = Vec::new();
    let mut page = 1usize;
    loop {
        let params = vec![
            ("page".to_string(), page.to_string()),
            ("perpage".to_string(), page_size.to_string()),
        ];
        let batch = request_array(
            &mut request_json,
            Method::GET,
            "/api/users",
            &params,
            None,
            "Unexpected global user list response from Grafana.",
        )?;
        if batch.is_empty() {
            break;
        }
        let batch_len = batch.len();
        users.extend(batch);
        if batch_len < page_size {
            break;
        }
        page += 1;
    }
    Ok(users)
}

fn list_user_teams_with_request<F>(mut request_json: F, user_id: &str) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_array(
        &mut request_json,
        Method::GET,
        &format!("/api/users/{user_id}/teams"),
        &[],
        None,
        &format!("Unexpected team list response for Grafana user {user_id}."),
    )
}

fn get_user_with_request<F>(mut request_json: F, user_id: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::GET,
        &format!("/api/users/{user_id}"),
        &[],
        None,
        &format!("Unexpected user lookup response for Grafana user {user_id}."),
    )
}

fn create_user_with_request<F>(mut request_json: F, payload: &Value) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::POST,
        "/api/admin/users",
        &[],
        Some(payload),
        "Unexpected user create response from Grafana.",
    )
}

fn update_user_with_request<F>(mut request_json: F, user_id: &str, payload: &Value) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::PUT,
        &format!("/api/users/{user_id}"),
        &[],
        Some(payload),
        &format!("Unexpected user update response for Grafana user {user_id}."),
    )
}

fn update_user_password_with_request<F>(mut request_json: F, user_id: &str, password: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::PUT,
        &format!("/api/admin/users/{user_id}/password"),
        &[],
        Some(&Value::Object(Map::from_iter(vec![("password".to_string(), Value::String(password.to_string()))]))),
        &format!("Unexpected password update response for Grafana user {user_id}."),
    )
}

fn update_user_org_role_with_request<F>(mut request_json: F, user_id: &str, role: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::PATCH,
        &format!("/api/org/users/{user_id}"),
        &[],
        Some(&Value::Object(Map::from_iter(vec![("role".to_string(), Value::String(role.to_string()))]))),
        &format!("Unexpected org-role update response for Grafana user {user_id}."),
    )
}

fn update_user_permissions_with_request<F>(mut request_json: F, user_id: &str, is_admin: bool) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::PUT,
        &format!("/api/admin/users/{user_id}/permissions"),
        &[],
        Some(&Value::Object(Map::from_iter(vec![("isGrafanaAdmin".to_string(), Value::Bool(is_admin))]))),
        &format!("Unexpected permission update response for Grafana user {user_id}."),
    )
}

fn delete_global_user_with_request<F>(mut request_json: F, user_id: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::DELETE,
        &format!("/api/admin/users/{user_id}"),
        &[],
        None,
        &format!("Unexpected global delete response for Grafana user {user_id}."),
    )
}

fn delete_org_user_with_request<F>(mut request_json: F, user_id: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::DELETE,
        &format!("/api/org/users/{user_id}"),
        &[],
        None,
        &format!("Unexpected org delete response for Grafana user {user_id}."),
    )
}

fn list_teams_with_request<F>(mut request_json: F, query: Option<&str>, page: usize, per_page: usize) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let params = vec![
        ("query".to_string(), query.unwrap_or("").to_string()),
        ("page".to_string(), page.to_string()),
        ("perpage".to_string(), per_page.to_string()),
    ];
    let object = request_object(
        &mut request_json,
        Method::GET,
        "/api/teams/search",
        &params,
        None,
        "Unexpected team list response from Grafana.",
    )?;
    match object.get("teams") {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| Ok(value_as_object(value, "Unexpected team list response from Grafana.")?.clone()))
            .collect(),
        _ => Err(message("Unexpected team list response from Grafana.")),
    }
}

fn list_team_members_with_request<F>(mut request_json: F, team_id: &str) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_array(
        &mut request_json,
        Method::GET,
        &format!("/api/teams/{team_id}/members"),
        &[],
        None,
        &format!("Unexpected member list response for Grafana team {team_id}."),
    )
}

fn get_team_with_request<F>(mut request_json: F, team_id: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::GET,
        &format!("/api/teams/{team_id}"),
        &[],
        None,
        &format!("Unexpected team lookup response for Grafana team {team_id}."),
    )
}

fn create_team_with_request<F>(mut request_json: F, payload: &Value) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::POST,
        "/api/teams",
        &[],
        Some(payload),
        "Unexpected team create response from Grafana.",
    )
}

fn add_team_member_with_request<F>(mut request_json: F, team_id: &str, user_id: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::POST,
        &format!("/api/teams/{team_id}/members"),
        &[],
        Some(&Value::Object(Map::from_iter(vec![("userId".to_string(), Value::String(user_id.to_string()))]))),
        &format!("Unexpected add-member response for Grafana team {team_id}."),
    )
}

fn remove_team_member_with_request<F>(mut request_json: F, team_id: &str, user_id: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::DELETE,
        &format!("/api/teams/{team_id}/members/{user_id}"),
        &[],
        None,
        &format!("Unexpected remove-member response for Grafana team {team_id}."),
    )
}

fn update_team_members_with_request<F>(mut request_json: F, team_id: &str, members: Vec<String>, admins: Vec<String>) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::PUT,
        &format!("/api/teams/{team_id}/members"),
        &[],
        Some(&Value::Object(Map::from_iter(vec![
            ("members".to_string(), Value::Array(members.into_iter().map(Value::String).collect())),
            ("admins".to_string(), Value::Array(admins.into_iter().map(Value::String).collect())),
        ]))),
        &format!("Unexpected team member update response for Grafana team {team_id}."),
    )
}

fn list_service_accounts_with_request<F>(mut request_json: F, query: Option<&str>, page: usize, per_page: usize) -> Result<Vec<Map<String, Value>>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let params = vec![
        ("query".to_string(), query.unwrap_or("").to_string()),
        ("page".to_string(), page.to_string()),
        ("perpage".to_string(), per_page.to_string()),
    ];
    let object = request_object(
        &mut request_json,
        Method::GET,
        "/api/serviceaccounts/search",
        &params,
        None,
        "Unexpected service-account list response from Grafana.",
    )?;
    match object.get("serviceAccounts") {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| Ok(value_as_object(value, "Unexpected service-account list response from Grafana.")?.clone()))
            .collect(),
        _ => Err(message("Unexpected service-account list response from Grafana.")),
    }
}

fn create_service_account_with_request<F>(mut request_json: F, payload: &Value) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::POST,
        "/api/serviceaccounts",
        &[],
        Some(payload),
        "Unexpected service-account create response from Grafana.",
    )
}

fn create_service_account_token_with_request<F>(mut request_json: F, service_account_id: &str, payload: &Value) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_object(
        &mut request_json,
        Method::POST,
        &format!("/api/serviceaccounts/{service_account_id}/tokens"),
        &[],
        Some(payload),
        "Unexpected service-account token create response from Grafana.",
    )
}

fn normalize_user_row(user: &Map<String, Value>, scope: &Scope) -> Map<String, Value> {
    Map::from_iter(vec![
        ("id".to_string(), Value::String({
            let user_id = scalar_text(user.get("userId"));
            if user_id.is_empty() {
                scalar_text(user.get("id"))
            } else {
                user_id
            }
        })),
        ("login".to_string(), Value::String(string_field(user, "login", ""))),
        ("email".to_string(), Value::String(string_field(user, "email", ""))),
        ("name".to_string(), Value::String(string_field(user, "name", ""))),
        ("orgRole".to_string(), Value::String(normalize_org_role(user.get("role")))),
        (
            "grafanaAdmin".to_string(),
            Value::String(bool_label(value_bool(user.get("isGrafanaAdmin")).or_else(|| value_bool(user.get("isAdmin"))))),
        ),
        ("scope".to_string(), Value::String(user_scope_text(scope).to_string())),
        ("teams".to_string(), Value::Array(Vec::new())),
    ])
}

fn normalize_team_row(team: &Map<String, Value>) -> Map<String, Value> {
    Map::from_iter(vec![
        ("id".to_string(), Value::String(scalar_text(team.get("id")))),
        ("name".to_string(), Value::String(string_field(team, "name", ""))),
        ("email".to_string(), Value::String(string_field(team, "email", ""))),
        ("memberCount".to_string(), Value::String({
            let value = scalar_text(team.get("memberCount"));
            if value.is_empty() { "0".to_string() } else { value }
        })),
        ("members".to_string(), Value::Array(Vec::new())),
    ])
}

fn normalize_service_account_row(team: &Map<String, Value>) -> Map<String, Value> {
    Map::from_iter(vec![
        ("id".to_string(), Value::String(scalar_text(team.get("id")))),
        ("name".to_string(), Value::String(string_field(team, "name", ""))),
        ("login".to_string(), Value::String(string_field(team, "login", ""))),
        ("role".to_string(), Value::String(normalize_org_role(team.get("role")))),
        ("disabled".to_string(), Value::String(bool_label(value_bool(team.get("isDisabled"))))),
        ("tokens".to_string(), Value::String({
            let value = scalar_text(team.get("tokens"));
            if value.is_empty() { "0".to_string() } else { value }
        })),
        ("orgId".to_string(), Value::String(scalar_text(team.get("orgId")))),
    ])
}

fn map_get_text(map: &Map<String, Value>, key: &str) -> String {
    match map.get(key) {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect::<Vec<&str>>().join(","),
        _ => String::new(),
    }
}

fn render_objects_json(rows: &[Map<String, Value>]) -> Result<String> {
    Ok(serde_json::to_string_pretty(&Value::Array(rows.iter().cloned().map(Value::Object).collect()))?)
}

fn user_table_rows(rows: &[Map<String, Value>]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|row| vec![
            map_get_text(row, "id"),
            map_get_text(row, "login"),
            map_get_text(row, "email"),
            map_get_text(row, "name"),
            map_get_text(row, "orgRole"),
            map_get_text(row, "grafanaAdmin"),
            map_get_text(row, "scope"),
            map_get_text(row, "teams"),
        ])
        .collect()
}

fn team_table_rows(rows: &[Map<String, Value>]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|row| vec![
            map_get_text(row, "id"),
            map_get_text(row, "name"),
            map_get_text(row, "email"),
            map_get_text(row, "memberCount"),
            map_get_text(row, "members"),
        ])
        .collect()
}

fn service_account_table_rows(rows: &[Map<String, Value>]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|row| vec![
            map_get_text(row, "id"),
            map_get_text(row, "name"),
            map_get_text(row, "login"),
            map_get_text(row, "role"),
            map_get_text(row, "disabled"),
            map_get_text(row, "tokens"),
            map_get_text(row, "orgId"),
        ])
        .collect()
}

fn user_summary_line(row: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("id={}", map_get_text(row, "id")),
        format!("login={}", map_get_text(row, "login")),
    ];
    let email = map_get_text(row, "email");
    if !email.is_empty() {
        parts.push(format!("email={email}"));
    }
    let name = map_get_text(row, "name");
    if !name.is_empty() {
        parts.push(format!("name={name}"));
    }
    let role = map_get_text(row, "orgRole");
    if !role.is_empty() {
        parts.push(format!("orgRole={role}"));
    }
    let admin = map_get_text(row, "grafanaAdmin");
    if !admin.is_empty() {
        parts.push(format!("grafanaAdmin={admin}"));
    }
    let teams = map_get_text(row, "teams");
    if !teams.is_empty() {
        parts.push(format!("teams={teams}"));
    }
    parts.push(format!("scope={}", map_get_text(row, "scope")));
    parts.join(" ")
}

fn team_summary_line(row: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("id={}", map_get_text(row, "id")),
        format!("name={}", map_get_text(row, "name")),
    ];
    let email = map_get_text(row, "email");
    if !email.is_empty() {
        parts.push(format!("email={email}"));
    }
    parts.push(format!("memberCount={}", map_get_text(row, "memberCount")));
    let members = map_get_text(row, "members");
    if !members.is_empty() {
        parts.push(format!("members={members}"));
    }
    parts.join(" ")
}

fn service_account_summary_line(row: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("id={}", map_get_text(row, "id")),
        format!("name={}", map_get_text(row, "name")),
    ];
    let login = map_get_text(row, "login");
    if !login.is_empty() {
        parts.push(format!("login={login}"));
    }
    parts.push(format!("role={}", map_get_text(row, "role")));
    parts.push(format!("disabled={}", map_get_text(row, "disabled")));
    parts.push(format!("tokens={}", map_get_text(row, "tokens")));
    let org_id = map_get_text(row, "orgId");
    if !org_id.is_empty() {
        parts.push(format!("orgId={org_id}"));
    }
    parts.join(" ")
}

fn exact_text_matches(text: &str, filter: &Option<String>) -> bool {
    match filter {
        Some(value) => text == value,
        None => true,
    }
}

fn user_matches(row: &Map<String, Value>, args: &UserListArgs) -> bool {
    let login = map_get_text(row, "login");
    let email = map_get_text(row, "email");
    let name = map_get_text(row, "name");
    if let Some(query) = &args.query {
        let query = query.to_ascii_lowercase();
        if !login.to_ascii_lowercase().contains(&query)
            && !email.to_ascii_lowercase().contains(&query)
            && !name.to_ascii_lowercase().contains(&query)
        {
            return false;
        }
    }
    if !exact_text_matches(&login, &args.login) {
        return false;
    }
    if !exact_text_matches(&email, &args.email) {
        return false;
    }
    if let Some(role) = &args.org_role {
        if map_get_text(row, "orgRole") != *role {
            return false;
        }
    }
    if let Some(admin) = args.grafana_admin {
        if map_get_text(row, "grafanaAdmin") != bool_label(Some(admin)) {
            return false;
        }
    }
    true
}

fn paginate_rows(rows: &[Map<String, Value>], page: usize, per_page: usize) -> Vec<Map<String, Value>> {
    let start = per_page.saturating_mul(page.saturating_sub(1));
    rows.iter().skip(start).take(per_page).cloned().collect()
}

fn lookup_global_user_by_identity<F>(mut request_json: F, login: Option<&str>, email: Option<&str>) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let users = iter_global_users_with_request(&mut request_json, DEFAULT_PAGE_SIZE)?;
    users
        .into_iter()
        .find(|user| {
            login.is_some_and(|value| string_field(user, "login", "") == value)
                || email.is_some_and(|value| string_field(user, "email", "") == value)
        })
        .ok_or_else(|| message("Grafana user lookup did not find a matching global user."))
}

fn lookup_org_user_by_identity<F>(mut request_json: F, identity: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let users = list_org_users_with_request(&mut request_json)?;
    users
        .into_iter()
        .find(|user| {
            string_field(user, "login", "") == identity
                || string_field(user, "email", "") == identity
                || scalar_text(user.get("userId")) == identity
                || scalar_text(user.get("id")) == identity
        })
        .ok_or_else(|| message(format!("Grafana org user lookup did not find {identity}.")))
}

fn lookup_team_by_name<F>(mut request_json: F, name: &str) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let teams = list_teams_with_request(&mut request_json, Some(name), 1, DEFAULT_PAGE_SIZE)?;
    teams
        .into_iter()
        .find(|team| string_field(team, "name", "") == name)
        .ok_or_else(|| message(format!("Grafana team lookup did not find {name}.")))
}

fn lookup_service_account_id_by_name<F>(mut request_json: F, name: &str) -> Result<String>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let accounts = list_service_accounts_with_request(&mut request_json, Some(name), 1, DEFAULT_PAGE_SIZE)?;
    let account = accounts
        .into_iter()
        .find(|item| string_field(item, "name", "") == name)
        .ok_or_else(|| message(format!("Grafana service-account lookup did not find {name}.")))?;
    Ok(scalar_text(account.get("id")))
}

fn validate_basic_auth_only(auth_mode: &str, operation: &str) -> Result<()> {
    if auth_mode != "basic" {
        Err(message(format!("{operation} requires Basic auth (--basic-user / --basic-password).")))
    } else {
        Ok(())
    }
}

fn validate_user_list_auth(args: &UserListArgs, auth_mode: &str) -> Result<()> {
    if args.scope == Scope::Global && auth_mode != "basic" {
        return Err(message(
            "User list with --scope global requires Basic auth (--basic-user / --basic-password).",
        ));
    }
    if args.with_teams && auth_mode != "basic" {
        return Err(message("--with-teams requires Basic auth."));
    }
    Ok(())
}

fn validate_user_modify_args(args: &UserModifyArgs) -> Result<()> {
    let has_identity = args.user_id.is_some() || args.login.is_some() || args.email.is_some();
    if !has_identity {
        return Err(message("User modify requires one of --user-id, --login, or --email."));
    }
    if args.set_login.is_none()
        && args.set_email.is_none()
        && args.set_name.is_none()
        && args.set_password.is_none()
        && args.set_org_role.is_none()
        && args.set_grafana_admin.is_none()
    {
        return Err(message(
            "User modify requires at least one of --set-login, --set-email, --set-name, --set-password, --set-org-role, or --set-grafana-admin.",
        ));
    }
    Ok(())
}

fn validate_user_delete_args(args: &UserDeleteArgs) -> Result<()> {
    if !args.yes {
        return Err(message("User delete requires --yes."));
    }
    if args.user_id.is_none() && args.login.is_none() && args.email.is_none() {
        return Err(message("User delete requires one of --user-id, --login, or --email."));
    }
    Ok(())
}

fn validate_team_modify_args(args: &TeamModifyArgs) -> Result<()> {
    if args.team_id.is_none() && args.name.is_none() {
        return Err(message("Team modify requires one of --team-id or --name."));
    }
    if args.add_member.is_empty()
        && args.remove_member.is_empty()
        && args.add_admin.is_empty()
        && args.remove_admin.is_empty()
    {
        return Err(message(
            "Team modify requires at least one of --add-member, --remove-member, --add-admin, or --remove-admin.",
        ));
    }
    Ok(())
}

fn list_users_with_request<F>(mut request_json: F, args: &UserListArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let auth_mode = build_auth_context(&args.common)?.auth_mode;
    validate_user_list_auth(args, &auth_mode)?;
    let mut rows = match args.scope {
        Scope::Org => list_org_users_with_request(&mut request_json)?
            .into_iter()
            .map(|item| normalize_user_row(&item, &Scope::Org))
            .collect::<Vec<Map<String, Value>>>(),
        Scope::Global => iter_global_users_with_request(&mut request_json, DEFAULT_PAGE_SIZE)?
            .into_iter()
            .map(|item| normalize_user_row(&item, &Scope::Global))
            .collect::<Vec<Map<String, Value>>>(),
    };
    if args.with_teams {
        for row in &mut rows {
            let user_id = map_get_text(row, "id");
            let teams = list_user_teams_with_request(&mut request_json, &user_id)?
                .into_iter()
                .map(|team| string_field(&team, "name", ""))
                .filter(|name| !name.is_empty())
                .map(Value::String)
                .collect::<Vec<Value>>();
            row.insert("teams".to_string(), Value::Array(teams));
        }
    }
    rows.retain(|row| user_matches(row, args));
    let rows = paginate_rows(&rows, args.page, args.per_page);
    if args.json {
        println!("{}", render_objects_json(&rows)?);
    } else if args.csv {
        for line in render_csv(&["id", "login", "email", "name", "orgRole", "grafanaAdmin", "scope", "teams"], &user_table_rows(&rows)) {
            println!("{line}");
        }
    } else if args.table {
        for line in format_table(&["ID", "LOGIN", "EMAIL", "NAME", "ORG_ROLE", "GRAFANA_ADMIN", "SCOPE", "TEAMS"], &user_table_rows(&rows)) {
            println!("{line}");
        }
        println!();
        println!("Listed {} user(s) from {} scope at {}", rows.len(), user_scope_text(&args.scope), args.common.url);
    } else {
        for row in &rows {
            println!("{}", user_summary_line(row));
        }
        println!();
        println!("Listed {} user(s) from {} scope at {}", rows.len(), user_scope_text(&args.scope), args.common.url);
    }
    Ok(rows.len())
}

fn add_user_with_request<F>(mut request_json: F, args: &UserAddArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let auth_mode = build_auth_context(&args.common)?.auth_mode;
    validate_basic_auth_only(&auth_mode, "User add")?;
    let mut payload = Map::from_iter(vec![
        ("login".to_string(), Value::String(args.login.clone())),
        ("email".to_string(), Value::String(args.email.clone())),
        ("name".to_string(), Value::String(args.name.clone())),
        ("password".to_string(), Value::String(args.new_user_password.clone())),
    ]);
    if let Some(org_id) = args.common.org_id {
        payload.insert("OrgId".to_string(), Value::Number(org_id.into()));
    }
    let created = create_user_with_request(&mut request_json, &Value::Object(payload))?;
    let user_id = scalar_text(created.get("id"));
    if user_id.is_empty() {
        return Err(message("Grafana user create response did not include an id."));
    }
    if let Some(role) = &args.org_role {
        let _ = update_user_org_role_with_request(&mut request_json, &user_id, role)?;
    }
    if let Some(is_admin) = args.grafana_admin {
        let _ = update_user_permissions_with_request(&mut request_json, &user_id, is_admin)?;
    }
    let row = Map::from_iter(vec![
        ("id".to_string(), Value::String(user_id.clone())),
        ("login".to_string(), Value::String(args.login.clone())),
        ("email".to_string(), Value::String(args.email.clone())),
        ("name".to_string(), Value::String(args.name.clone())),
        ("orgRole".to_string(), Value::String(args.org_role.clone().unwrap_or_default())),
        ("grafanaAdmin".to_string(), Value::String(bool_label(args.grafana_admin))),
        ("scope".to_string(), Value::String("global".to_string())),
        ("teams".to_string(), Value::Array(Vec::new())),
    ]);
    if args.json {
        println!("{}", render_objects_json(&[row])?);
    } else {
        println!(
            "Created user {} -> id={} orgRole={} grafanaAdmin={}",
            args.login,
            user_id,
            args.org_role.clone().unwrap_or_default(),
            bool_label(args.grafana_admin)
        );
    }
    Ok(0)
}

fn modify_user_with_request<F>(mut request_json: F, args: &UserModifyArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let auth_mode = build_auth_context(&args.common)?.auth_mode;
    validate_basic_auth_only(&auth_mode, "User modify")?;
    validate_user_modify_args(args)?;
    let base_user = if let Some(user_id) = &args.user_id {
        get_user_with_request(&mut request_json, user_id)?
    } else {
        lookup_global_user_by_identity(&mut request_json, args.login.as_deref(), args.email.as_deref())?
    };
    let user_id = string_field(&base_user, "id", "");
    let user_id = if user_id.is_empty() { scalar_text(base_user.get("id")) } else { user_id };
    let mut payload = Map::new();
    if let Some(value) = &args.set_login {
        payload.insert("login".to_string(), Value::String(value.clone()));
    }
    if let Some(value) = &args.set_email {
        payload.insert("email".to_string(), Value::String(value.clone()));
    }
    if let Some(value) = &args.set_name {
        payload.insert("name".to_string(), Value::String(value.clone()));
    }
    if !payload.is_empty() {
        let _ = update_user_with_request(&mut request_json, &user_id, &Value::Object(payload))?;
    }
    if let Some(password) = &args.set_password {
        let _ = update_user_password_with_request(&mut request_json, &user_id, password)?;
    }
    if let Some(role) = &args.set_org_role {
        let _ = update_user_org_role_with_request(&mut request_json, &user_id, role)?;
    }
    if let Some(is_admin) = args.set_grafana_admin {
        let _ = update_user_permissions_with_request(&mut request_json, &user_id, is_admin)?;
    }
    let login = args.set_login.clone().unwrap_or_else(|| string_field(&base_user, "login", ""));
    let row = Map::from_iter(vec![
        ("id".to_string(), Value::String(user_id.clone())),
        ("login".to_string(), Value::String(login.clone())),
        ("email".to_string(), Value::String(args.set_email.clone().unwrap_or_else(|| string_field(&base_user, "email", "")))),
        ("name".to_string(), Value::String(args.set_name.clone().unwrap_or_else(|| string_field(&base_user, "name", "")))),
        ("orgRole".to_string(), Value::String(args.set_org_role.clone().unwrap_or_else(|| normalize_org_role(base_user.get("role"))))),
        ("grafanaAdmin".to_string(), Value::String(bool_label(args.set_grafana_admin.or_else(|| value_bool(base_user.get("isGrafanaAdmin")))))),
        ("scope".to_string(), Value::String("global".to_string())),
        ("teams".to_string(), Value::Array(Vec::new())),
    ]);
    if args.json {
        println!("{}", render_objects_json(&[row])?);
    } else {
        println!("Modified user {} -> id={}", login, user_id);
    }
    Ok(0)
}

fn delete_user_with_request<F>(mut request_json: F, args: &UserDeleteArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let auth_mode = build_auth_context(&args.common)?.auth_mode;
    validate_user_delete_args(args)?;
    if args.scope == Scope::Global {
        validate_basic_auth_only(&auth_mode, "User delete with --scope global")?;
    }
    let base_user = match args.scope {
        Scope::Org => {
            if let Some(user_id) = &args.user_id {
                lookup_org_user_by_identity(&mut request_json, user_id)?
            } else {
                lookup_org_user_by_identity(&mut request_json, args.login.as_deref().or(args.email.as_deref()).unwrap_or(""))?
            }
        }
        Scope::Global => {
            if let Some(user_id) = &args.user_id {
                get_user_with_request(&mut request_json, user_id)?
            } else {
                lookup_global_user_by_identity(&mut request_json, args.login.as_deref(), args.email.as_deref())?
            }
        }
    };
    let user_id = {
        let user_id = scalar_text(base_user.get("userId"));
        if user_id.is_empty() {
            scalar_text(base_user.get("id"))
        } else {
            user_id
        }
    };
    match args.scope {
        Scope::Org => {
            let _ = delete_org_user_with_request(&mut request_json, &user_id)?;
        }
        Scope::Global => {
            let _ = delete_global_user_with_request(&mut request_json, &user_id)?;
        }
    }
    let row = Map::from_iter(vec![
        ("id".to_string(), Value::String(user_id.clone())),
        ("login".to_string(), Value::String(string_field(&base_user, "login", ""))),
        ("scope".to_string(), Value::String(user_scope_text(&args.scope).to_string())),
    ]);
    if args.json {
        println!("{}", render_objects_json(&[row])?);
    } else {
        println!("Deleted user {} -> id={} scope={}", map_get_text(&row, "login"), user_id, user_scope_text(&args.scope));
    }
    Ok(0)
}

fn team_member_identity(member: &Map<String, Value>) -> String {
    let email = string_field(member, "email", "");
    if !email.is_empty() {
        email
    } else {
        string_field(member, "login", "")
    }
}

fn team_member_is_admin(member: &Map<String, Value>) -> bool {
    value_bool(member.get("isAdmin")).unwrap_or_else(|| value_bool(member.get("admin")).unwrap_or(false))
}

fn add_or_remove_member<F>(request_json: &mut F, team_id: &str, identity: &str, add: bool) -> Result<String>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let user = lookup_org_user_by_identity(&mut *request_json, identity)?;
    let user_id = string_field(&user, "userId", &string_field(&user, "id", ""));
    if add {
        let _ = add_team_member_with_request(&mut *request_json, team_id, &user_id)?;
    } else {
        let _ = remove_team_member_with_request(&mut *request_json, team_id, &user_id)?;
    }
    Ok(string_field(&user, "email", &string_field(&user, "login", identity)))
}

fn list_teams_command_with_request<F>(mut request_json: F, args: &TeamListArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut rows = list_teams_with_request(&mut request_json, args.query.as_deref(), args.page, args.per_page)?
        .into_iter()
        .map(|team| normalize_team_row(&team))
        .collect::<Vec<Map<String, Value>>>();
    if let Some(name) = &args.name {
        rows.retain(|row| map_get_text(row, "name") == *name);
    }
    if args.with_members {
        for row in &mut rows {
            let team_id = map_get_text(row, "id");
            let members = list_team_members_with_request(&mut request_json, &team_id)?
                .into_iter()
                .map(|member| team_member_identity(&member))
                .filter(|identity| !identity.is_empty())
                .map(Value::String)
                .collect::<Vec<Value>>();
            row.insert("members".to_string(), Value::Array(members));
        }
    }
    if args.json {
        println!("{}", render_objects_json(&rows)?);
    } else if args.csv {
        for line in render_csv(&["id", "name", "email", "memberCount", "members"], &team_table_rows(&rows)) {
            println!("{line}");
        }
    } else if args.table {
        for line in format_table(&["ID", "NAME", "EMAIL", "MEMBER_COUNT", "MEMBERS"], &team_table_rows(&rows)) {
            println!("{line}");
        }
        println!();
        println!("Listed {} team(s) at {}", rows.len(), args.common.url);
    } else {
        for row in &rows {
            println!("{}", team_summary_line(row));
        }
        println!();
        println!("Listed {} team(s) at {}", rows.len(), args.common.url);
    }
    Ok(rows.len())
}

fn team_modify_result(team_id: &str, team_name: &str, added_members: Vec<String>, removed_members: Vec<String>, added_admins: Vec<String>, removed_admins: Vec<String>, email: String) -> Map<String, Value> {
    Map::from_iter(vec![
        ("teamId".to_string(), Value::String(team_id.to_string())),
        ("name".to_string(), Value::String(team_name.to_string())),
        ("email".to_string(), Value::String(email)),
        ("addedMembers".to_string(), Value::Array(added_members.into_iter().map(Value::String).collect())),
        ("removedMembers".to_string(), Value::Array(removed_members.into_iter().map(Value::String).collect())),
        ("addedAdmins".to_string(), Value::Array(added_admins.into_iter().map(Value::String).collect())),
        ("removedAdmins".to_string(), Value::Array(removed_admins.into_iter().map(Value::String).collect())),
    ])
}

fn team_modify_summary_line(result: &Map<String, Value>) -> String {
    let mut text = format!("teamId={} name={}", map_get_text(result, "teamId"), map_get_text(result, "name"));
    for key in ["addedMembers", "removedMembers", "addedAdmins", "removedAdmins"] {
        let value = map_get_text(result, key);
        if !value.is_empty() {
            let _ = write!(&mut text, " {}={}", key, value);
        }
    }
    text
}

fn modify_team_with_request<F>(mut request_json: F, args: &TeamModifyArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_team_modify_args(args)?;
    let team = if let Some(team_id) = &args.team_id {
        get_team_with_request(&mut request_json, team_id)?
    } else {
        lookup_team_by_name(&mut request_json, args.name.as_deref().unwrap_or(""))?
    };
    let team_id = scalar_text(team.get("id"));
    let team_name = string_field(&team, "name", "");
    let mut added_members = Vec::new();
    let mut removed_members = Vec::new();
    for identity in &args.add_member {
        added_members.push(add_or_remove_member(&mut request_json, &team_id, identity, true)?);
    }
    for identity in &args.remove_member {
        removed_members.push(add_or_remove_member(&mut request_json, &team_id, identity, false)?);
    }
    let existing_members = list_team_members_with_request(&mut request_json, &team_id)?;
    let mut member_identities = existing_members.iter().map(team_member_identity).collect::<Vec<String>>();
    let mut admin_identities = existing_members
        .iter()
        .filter(|member| team_member_is_admin(member))
        .map(team_member_identity)
        .collect::<Vec<String>>();
    let mut added_admins = Vec::new();
    let mut removed_admins = Vec::new();
    if !args.add_admin.is_empty() || !args.remove_admin.is_empty() {
        for identity in &args.add_admin {
            let user = lookup_org_user_by_identity(&mut request_json, identity)?;
            let resolved = string_field(&user, "email", &string_field(&user, "login", identity));
            if !member_identities.contains(&resolved) {
                member_identities.push(resolved.clone());
            }
            if !admin_identities.contains(&resolved) {
                admin_identities.push(resolved.clone());
                added_admins.push(resolved);
            }
        }
        for identity in &args.remove_admin {
            let user = lookup_org_user_by_identity(&mut request_json, identity)?;
            let resolved = string_field(&user, "email", &string_field(&user, "login", identity));
            if let Some(index) = admin_identities.iter().position(|value| value == &resolved) {
                admin_identities.remove(index);
                removed_admins.push(resolved);
            }
        }
        member_identities.sort();
        member_identities.dedup();
        admin_identities.sort();
        admin_identities.dedup();
        let _ = update_team_members_with_request(&mut request_json, &team_id, member_identities.clone(), admin_identities.clone())?;
    }
    let result = team_modify_result(
        &team_id,
        &team_name,
        added_members,
        removed_members,
        added_admins,
        removed_admins,
        string_field(&team, "email", ""),
    );
    if args.json {
        println!("{}", render_objects_json(&[result])?);
    } else {
        println!("{}", team_modify_summary_line(&result));
    }
    Ok(0)
}

fn add_team_with_request<F>(mut request_json: F, args: &TeamAddArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut payload = Map::from_iter(vec![("name".to_string(), Value::String(args.name.clone()))]);
    if let Some(email) = &args.email {
        payload.insert("email".to_string(), Value::String(email.clone()));
    }
    let created = create_team_with_request(&mut request_json, &Value::Object(payload))?;
    let team_id = {
        let team_id = scalar_text(created.get("teamId"));
        if team_id.is_empty() {
            scalar_text(created.get("id"))
        } else {
            team_id
        }
    };
    let team = get_team_with_request(&mut request_json, &team_id)?;
    let modify = TeamModifyArgs {
        common: args.common.clone(),
        team_id: Some(team_id.clone()),
        name: None,
        add_member: args.members.clone(),
        remove_member: Vec::new(),
        add_admin: args.admins.clone(),
        remove_admin: Vec::new(),
        json: true,
    };
    let _ = modify_team_with_request(&mut request_json, &modify)?;
    let result = team_modify_result(
        &team_id,
        &string_field(&team, "name", &args.name),
        args.members.clone(),
        Vec::new(),
        args.admins.clone(),
        Vec::new(),
        string_field(&team, "email", args.email.as_deref().unwrap_or("")),
    );
    if args.json {
        println!("{}", render_objects_json(&[result])?);
    } else {
        println!("{}", team_modify_summary_line(&result));
    }
    Ok(0)
}

fn list_service_accounts_command_with_request<F>(mut request_json: F, args: &ServiceAccountListArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut rows = list_service_accounts_with_request(&mut request_json, args.query.as_deref(), args.page, args.per_page)?
        .into_iter()
        .map(|item| normalize_service_account_row(&item))
        .collect::<Vec<Map<String, Value>>>();
    if let Some(query) = &args.query {
        let query = query.to_ascii_lowercase();
        rows.retain(|row| {
            map_get_text(row, "name").to_ascii_lowercase().contains(&query)
                || map_get_text(row, "login").to_ascii_lowercase().contains(&query)
        });
    }
    if args.json {
        println!("{}", render_objects_json(&rows)?);
    } else if args.csv {
        for line in render_csv(&["id", "name", "login", "role", "disabled", "tokens", "orgId"], &service_account_table_rows(&rows)) {
            println!("{line}");
        }
    } else if args.table {
        for line in format_table(&["ID", "NAME", "LOGIN", "ROLE", "DISABLED", "TOKENS", "ORG_ID"], &service_account_table_rows(&rows)) {
            println!("{line}");
        }
        println!();
        println!("Listed {} service account(s) at {}", rows.len(), args.common.url);
    } else {
        for row in &rows {
            println!("{}", service_account_summary_line(row));
        }
        println!();
        println!("Listed {} service account(s) at {}", rows.len(), args.common.url);
    }
    Ok(rows.len())
}

fn add_service_account_with_request<F>(mut request_json: F, args: &ServiceAccountAddArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let payload = Value::Object(Map::from_iter(vec![
        ("name".to_string(), Value::String(args.name.clone())),
        ("role".to_string(), Value::String(service_account_role_to_api(&args.role))),
        ("isDisabled".to_string(), Value::Bool(args.disabled)),
    ]));
    let created = normalize_service_account_row(&create_service_account_with_request(&mut request_json, &payload)?);
    if args.json {
        println!("{}", render_objects_json(&[created])?);
    } else {
        println!(
            "Created service-account {} -> id={} role={} disabled={}",
            args.name,
            map_get_text(&created, "id"),
            map_get_text(&created, "role"),
            map_get_text(&created, "disabled")
        );
    }
    Ok(0)
}

fn add_service_account_token_with_request<F>(mut request_json: F, args: &ServiceAccountTokenAddArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let service_account_id = match &args.service_account_id {
        Some(value) => value.clone(),
        None => lookup_service_account_id_by_name(&mut request_json, args.name.as_deref().unwrap_or(""))?,
    };
    let mut payload = Map::from_iter(vec![("name".to_string(), Value::String(args.token_name.clone()))]);
    if let Some(seconds) = args.seconds_to_live {
        payload.insert("secondsToLive".to_string(), Value::Number((seconds as i64).into()));
    }
    let mut token = create_service_account_token_with_request(&mut request_json, &service_account_id, &Value::Object(payload))?;
    token.insert("serviceAccountId".to_string(), Value::String(service_account_id.clone()));
    if args.json {
        println!("{}", render_objects_json(&[token])?);
    } else {
        println!(
            "Created service-account token {} -> serviceAccountId={}",
            args.token_name, service_account_id
        );
    }
    Ok(0)
}

pub fn run_access_cli_with_client(client: &JsonHttpClient, args: AccessCliArgs) -> Result<()> {
    run_access_cli_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

pub fn run_access_cli_with_request<F>(mut request_json: F, args: AccessCliArgs) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match args.command {
        AccessCommand::User { command } => match command {
            UserCommand::List(args) => {
                let _ = list_users_with_request(&mut request_json, &args)?;
            }
            UserCommand::Add(args) => {
                let _ = add_user_with_request(&mut request_json, &args)?;
            }
            UserCommand::Modify(args) => {
                let _ = modify_user_with_request(&mut request_json, &args)?;
            }
            UserCommand::Delete(args) => {
                let _ = delete_user_with_request(&mut request_json, &args)?;
            }
        },
        AccessCommand::Team { command } => match command {
            TeamCommand::List(args) => {
                let _ = list_teams_command_with_request(&mut request_json, &args)?;
            }
            TeamCommand::Add(args) => {
                let _ = add_team_with_request(&mut request_json, &args)?;
            }
            TeamCommand::Modify(args) => {
                let _ = modify_team_with_request(&mut request_json, &args)?;
            }
        },
        AccessCommand::ServiceAccount { command } => match command {
            ServiceAccountCommand::List(args) => {
                let _ = list_service_accounts_command_with_request(&mut request_json, &args)?;
            }
            ServiceAccountCommand::Add(args) => {
                let _ = add_service_account_with_request(&mut request_json, &args)?;
            }
            ServiceAccountCommand::Token { command } => match command {
                ServiceAccountTokenCommand::Add(args) => {
                    let _ = add_service_account_token_with_request(&mut request_json, &args)?;
                }
            },
        },
    }
    Ok(())
}

pub fn run_access_cli(args: AccessCliArgs) -> Result<()> {
    match &args.command {
        AccessCommand::User { command } => match command {
            UserCommand::List(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
            UserCommand::Add(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
            UserCommand::Modify(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
            UserCommand::Delete(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
        },
        AccessCommand::Team { command } => match command {
            TeamCommand::List(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
            TeamCommand::Add(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
            TeamCommand::Modify(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
        },
        AccessCommand::ServiceAccount { command } => match command {
            ServiceAccountCommand::List(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
            ServiceAccountCommand::Add(inner) => {
                let client = build_http_client(&inner.common)?;
                run_access_cli_with_client(&client, args)
            }
            ServiceAccountCommand::Token { command } => match command {
                ServiceAccountTokenCommand::Add(inner) => {
                    let client = build_http_client(&inner.common)?;
                    run_access_cli_with_client(&client, args)
                }
            },
        },
    }
}

#[cfg(test)]
#[path = "access_rust_tests.rs"]
mod access_rust_tests;
