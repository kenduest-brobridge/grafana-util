use std::path::Path;

use reqwest::Url;

use crate::common::{env_value, validation, Result};

use super::{
    secrets::resolve_credential_value, ConnectionMergeInput, ConnectionProfile,
    ResolvedConnectionSettings, SelectedProfile, DEFAULT_PROFILE_CONFIG_FILENAME,
};

pub fn resolve_connection_settings(
    input: ConnectionMergeInput<'_>,
    selected_profile: Option<&SelectedProfile>,
) -> Result<ResolvedConnectionSettings> {
    let profile = selected_profile.map(|selected| &selected.profile);
    let cli_or_env_url = if input.url != input.url_default && !input.url.trim().is_empty() {
        Some(input.url.to_string())
    } else {
        env_value("GRAFANA_URL")
    };
    let url = if let Some(url) = cli_or_env_url {
        url
    } else if let Some(url) = profile.and_then(|item| item.url.clone()) {
        url
    } else if !input.url_default.trim().is_empty() {
        input.url_default.to_string()
    } else {
        return Err(validation(
            "Grafana base URL is required. Pass --url, set GRAFANA_URL, or configure a profile with url.",
        ));
    };
    let url = strip_url_credentials_with_warning(&url);
    let api_token = resolve_credential_value(
        input.api_token,
        profile.and_then(|item| item.token.as_deref()),
        profile.and_then(|item| item.token_store.as_ref()),
        profile.and_then(|item| item.token_env.as_deref()),
        selected_profile,
        "token",
    )?;
    let username = resolve_credential_value(
        input.username,
        profile.and_then(|item| item.username.as_deref()),
        None,
        profile.and_then(|item| item.username_env.as_deref()),
        selected_profile,
        "username",
    )?;
    let password = resolve_credential_value(
        input.password,
        profile.and_then(|item| item.password.as_deref()),
        profile.and_then(|item| item.password_store.as_ref()),
        profile.and_then(|item| item.password_env.as_deref()),
        selected_profile,
        "password",
    )?;
    let timeout = if input.timeout != input.timeout_default {
        input.timeout
    } else {
        profile
            .and_then(|item| item.timeout)
            .unwrap_or(input.timeout_default)
    };
    let org_id = input
        .org_id
        .or_else(|| profile.and_then(|item| item.org_id));
    let ca_cert = input
        .ca_cert
        .map(Path::to_path_buf)
        .or_else(|| profile.and_then(|item| item.ca_cert.clone()));
    let verify_ssl = resolve_verify_ssl(input, selected_profile, profile, ca_cert.is_some())?;

    Ok(ResolvedConnectionSettings {
        url,
        api_token,
        username,
        password,
        org_id,
        timeout,
        verify_ssl,
        ca_cert,
    })
}

fn strip_url_credentials_with_warning(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return url.to_string();
    };
    if !parsed.username().is_empty() || parsed.password().is_some() {
        eprintln!(
            "Warning: Grafana base URL includes username or password; URL credentials are ignored. Use --basic-user with --basic-password or --prompt-password, set GRAFANA_USERNAME and GRAFANA_PASSWORD, or store credentials in a profile instead."
        );
        let _ = parsed.set_username("");
        let _ = parsed.set_password(None);
        return parsed.to_string();
    }
    url.to_string()
}

fn resolve_verify_ssl(
    input: ConnectionMergeInput<'_>,
    selected_profile: Option<&SelectedProfile>,
    profile: Option<&ConnectionProfile>,
    ca_cert_present: bool,
) -> Result<bool> {
    if input.insecure && input.verify_ssl {
        return Err(validation(
            "Choose either --insecure or --verify-ssl, not both.",
        ));
    }
    if let Some(profile) = profile {
        if profile.insecure == Some(true) && profile.verify_ssl == Some(true) {
            let profile_name = selected_profile
                .map(|item| item.name.as_str())
                .unwrap_or("default");
            let source_path = selected_profile
                .map(|item| item.source_path.display().to_string())
                .unwrap_or_else(|| DEFAULT_PROFILE_CONFIG_FILENAME.to_string());
            return Err(validation(format!(
                "Profile `{}` in {} cannot set both verify_ssl: true and insecure: true.",
                profile_name, source_path
            )));
        }
    }
    if input.insecure {
        return Ok(false);
    }
    if input.verify_ssl || input.ca_cert.is_some() {
        return Ok(true);
    }
    if let Some(profile) = profile {
        if profile.insecure == Some(true) {
            return Ok(false);
        }
        if profile.verify_ssl == Some(true) || profile.ca_cert.is_some() {
            return Ok(true);
        }
        if let Some(value) = profile.verify_ssl {
            return Ok(value);
        }
    }
    Ok(ca_cert_present)
}
