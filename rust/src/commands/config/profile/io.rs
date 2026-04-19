use std::fs;
use std::path::Path;

use crate::common::{validation, Result};
use crate::profile_secret_store::ensure_owner_only_permissions;

use super::ProfileConfigFile;

pub fn load_profile_config_file(path: &Path) -> Result<ProfileConfigFile> {
    let raw = fs::read_to_string(path)?;
    serde_yaml::from_str::<ProfileConfigFile>(&raw).map_err(|error| {
        validation(format!(
            "Failed to parse grafana-util profile config {}: {error}",
            path.display()
        ))
    })
}

pub fn save_profile_config_file(path: &Path, config: &ProfileConfigFile) -> Result<()> {
    let rendered = serde_yaml::to_string(config).map_err(|error| {
        validation(format!(
            "Failed to render grafana-util profile config {}: {error}",
            path.display()
        ))
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            validation(format!(
                "Failed to create grafana-util profile config directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    fs::write(path, rendered).map_err(|error| {
        validation(format!(
            "Failed to write grafana-util profile config {}: {error}",
            path.display()
        ))
    })?;
    ensure_owner_only_permissions(path)?;
    Ok(())
}

pub fn render_profile_init_template() -> String {
    r#"default_profile: dev
profiles:
  dev:
    url: http://127.0.0.1:3000
    token_env: GRAFANA_API_TOKEN
    timeout: 30
    verify_ssl: false

  prod:
    url: https://grafana.example.com
    basic_user: admin
    password_env: GRAFANA_PROD_PASSWORD
    verify_ssl: true
"#
    .replace("basic_user", "username")
}
