use base64::{engine::general_purpose::STANDARD, Engine as _};
use rpassword::prompt_password;
use std::env;

use super::{validation, GrafanaCliError, Result};

/// Read an environment variable and treat blank values as unset.
pub fn env_value(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

/// Resolve Grafana authentication headers from CLI args, prompts, and environment.
///
/// Resolution order is intentional:
/// - explicit token or prompted token
/// - explicit/basic credentials
/// - environment fallbacks
///
/// The function rejects mixed auth modes so downstream HTTP code never has to
/// guess which credential source should win.
pub fn resolve_auth_headers(
    api_token: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    prompt_for_password: bool,
    prompt_for_token: bool,
) -> Result<Vec<(String, String)>> {
    resolve_auth_headers_with_prompt(
        api_token,
        username,
        password,
        prompt_for_password,
        prompt_for_token,
        || prompt_password("Grafana Basic auth password: ").map_err(GrafanaCliError::from),
        || prompt_password("Grafana API token: ").map_err(GrafanaCliError::from),
    )
}

pub(super) fn resolve_auth_headers_with_prompt<F, G>(
    api_token: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    prompt_for_password: bool,
    prompt_for_token: bool,
    prompt_password_reader: F,
    prompt_token_reader: G,
) -> Result<Vec<(String, String)>>
where
    F: FnOnce() -> Result<String>,
    G: FnOnce() -> Result<String>,
{
    let cli_token = api_token
        .map(str::to_owned)
        .filter(|value| !value.is_empty());
    let cli_username = username
        .map(str::to_owned)
        .filter(|value| !value.is_empty());
    let mut cli_password = password
        .map(str::to_owned)
        .filter(|value| !value.is_empty());

    if cli_token.is_some() && prompt_for_token {
        return Err(validation(
            "Choose either --token / --api-token or --prompt-token, not both.",
        ));
    }
    if (cli_token.is_some() || prompt_for_token)
        && (cli_username.is_some() || cli_password.is_some() || prompt_for_password)
    {
        return Err(validation(
            "Choose either token auth (--token / --api-token) or Basic auth \
(--basic-user with --basic-password / --prompt-password), not both.",
        ));
    }
    if prompt_for_password && cli_password.is_some() {
        return Err(validation(
            "Choose either --basic-password or --prompt-password, not both.",
        ));
    }
    if cli_username.is_some() && cli_password.is_none() && !prompt_for_password {
        return Err(validation(
            "Basic auth requires both --basic-user and \
--basic-password or --prompt-password.",
        ));
    }
    if cli_password.is_some() && cli_username.is_none() {
        return Err(validation(
            "Basic auth requires both --basic-user and \
--basic-password or --prompt-password.",
        ));
    }
    if prompt_for_password && cli_username.is_none() {
        return Err(validation("--prompt-password requires --basic-user."));
    }

    if prompt_for_token {
        let token = prompt_token_reader()?;
        return Ok(vec![(
            "Authorization".to_string(),
            format!("Bearer {token}"),
        )]);
    }

    let token = cli_token.or_else(|| env_value("GRAFANA_API_TOKEN"));
    if let Some(token) = token {
        return Ok(vec![(
            "Authorization".to_string(),
            format!("Bearer {token}"),
        )]);
    }

    if prompt_for_password && cli_username.is_some() {
        cli_password = Some(prompt_password_reader()?);
    }

    let username = cli_username.or_else(|| env_value("GRAFANA_USERNAME"));
    let password = cli_password.or_else(|| env_value("GRAFANA_PASSWORD"));
    if let (Some(username), Some(password)) = (username.as_ref(), password.as_ref()) {
        let encoded = STANDARD.encode(format!("{username}:{password}"));
        return Ok(vec![(
            "Authorization".to_string(),
            format!("Basic {encoded}"),
        )]);
    }
    if username.is_some() || password.is_some() {
        return Err(validation(
            "Basic auth requires both --basic-user and \
--basic-password or --prompt-password.",
        ));
    }

    Err(validation(
        "Authentication required. Set --token / --api-token / GRAFANA_API_TOKEN \
or --prompt-token / --basic-user and --basic-password / --prompt-password / \
GRAFANA_USERNAME and GRAFANA_PASSWORD.",
    ))
}
