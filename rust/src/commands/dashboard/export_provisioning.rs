use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::common::{message, Result};

#[derive(Serialize)]
pub(super) struct ProvisioningOptions {
    path: String,
    #[serde(rename = "foldersFromFilesStructure")]
    folders_from_files_structure: bool,
}

#[derive(Serialize)]
pub(super) struct ProvisioningProvider {
    name: String,
    #[serde(rename = "orgId")]
    org_id: i64,
    #[serde(rename = "type")]
    provider_type: String,
    #[serde(rename = "disableDeletion")]
    disable_deletion: bool,
    #[serde(rename = "allowUiUpdates")]
    allow_ui_updates: bool,
    #[serde(rename = "updateIntervalSeconds")]
    update_interval_seconds: i64,
    options: ProvisioningOptions,
}

#[derive(Serialize)]
pub(super) struct ProvisioningConfig {
    #[serde(rename = "apiVersion")]
    api_version: i64,
    providers: Vec<ProvisioningProvider>,
}

pub(super) fn build_provisioning_config(
    org_id: i64,
    provider_name: &str,
    dashboard_path: &Path,
    disable_deletion: bool,
    allow_ui_updates: bool,
    update_interval_seconds: i64,
) -> ProvisioningConfig {
    let path = dashboard_path
        .canonicalize()
        .unwrap_or_else(|_| dashboard_path.to_path_buf())
        .display()
        .to_string();
    ProvisioningConfig {
        api_version: 1,
        providers: vec![ProvisioningProvider {
            name: provider_name.to_string(),
            org_id,
            provider_type: "file".to_string(),
            disable_deletion,
            allow_ui_updates,
            update_interval_seconds,
            options: ProvisioningOptions {
                path,
                folders_from_files_structure: true,
            },
        }],
    }
}

pub(super) fn write_yaml_document<T: Serialize>(
    payload: &T,
    output_path: &Path,
    overwrite: bool,
) -> Result<()> {
    if output_path.exists() && !overwrite {
        return Err(message(format!(
            "Refusing to overwrite existing file: {}. Use --overwrite.",
            output_path.display()
        )));
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let rendered = serde_yaml::to_string(payload).map_err(|error| {
        message(format!(
            "Failed to serialize YAML document for {}: {error}",
            output_path.display()
        ))
    })?;
    fs::write(output_path, rendered)?;
    Ok(())
}
