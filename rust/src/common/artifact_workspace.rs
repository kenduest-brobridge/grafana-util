//! Shared artifact workspace primitives.
//!
//! This module is intentionally domain-agnostic: it provides a stable filesystem layout
//! and pointer-file conventions that other domains (dashboard/datasource/access/snapshot)
//! can build on without duplicating path logic.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{validation, Result};

pub const DEFAULT_PROFILE_SCOPE_NAME: &str = "default";
pub const LATEST_RUN_POINTER_FILENAME: &str = "latest-run.json";
pub const LATEST_RUN_POINTER_SCHEMA_VERSION: i64 = 1;
pub const LATEST_RUN_POINTER_KIND: &str = "grafana-util.latestRunPointer";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunSelector {
    /// Use the last recorded run id from `<scope>/latest-run.json`.
    Latest,
    /// Generate a new run id using local time.
    Timestamp,
    /// Use an explicit run id string.
    RunId(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactLane {
    Dashboards,
    Datasources,
    AccessUsers,
    AccessTeams,
    AccessOrgs,
    AccessServiceAccounts,
    Snapshots,
}

impl ArtifactLane {
    pub fn relative_path(self) -> &'static str {
        match self {
            ArtifactLane::Dashboards => "dashboards",
            ArtifactLane::Datasources => "datasources",
            ArtifactLane::AccessUsers => "access/users",
            ArtifactLane::AccessTeams => "access/teams",
            ArtifactLane::AccessOrgs => "access/orgs",
            ArtifactLane::AccessServiceAccounts => "access/service-accounts",
            ArtifactLane::Snapshots => "snapshots",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestRunPointer {
    pub schema_version: i64,
    pub kind: String,
    pub profile: String,
    pub run_id: String,
    pub run_path: String,
    pub updated_at: String,
}

pub fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty() {
        return Err(validation("Run id must not be empty."));
    }
    if run_id == "." || run_id == ".." {
        return Err(validation("Run id must not be '.' or '..'."));
    }
    if run_id.contains('/') || run_id.contains('\\') {
        return Err(validation("Run id must not contain path separators."));
    }
    Ok(())
}

pub fn profile_scope_name(profile: Option<&str>) -> &str {
    profile.unwrap_or(DEFAULT_PROFILE_SCOPE_NAME)
}

pub fn profile_scope_path(artifact_root: &Path, profile: Option<&str>) -> PathBuf {
    artifact_root.join(profile_scope_name(profile))
}

pub fn latest_run_pointer_path(scope_root: &Path) -> PathBuf {
    scope_root.join(LATEST_RUN_POINTER_FILENAME)
}

pub fn run_path_for_id(run_id: &str) -> PathBuf {
    PathBuf::from("runs").join(run_id)
}

pub fn run_root_path(scope_root: &Path, run_id: &str) -> PathBuf {
    scope_root.join(run_path_for_id(run_id))
}

pub fn lane_root_path(run_root: &Path, lane: ArtifactLane) -> PathBuf {
    run_root.join(lane.relative_path())
}

pub fn resolve_input_lane(
    profile: Option<&str>,
    lane_path: &str,
    run: Option<&str>,
    run_id: Option<&str>,
) -> Result<Option<PathBuf>> {
    let selector = if let Some(run_id) = run_id {
        RunSelector::RunId(run_id.to_string())
    } else {
        match run.unwrap_or("latest") {
            "latest" => RunSelector::Latest,
            "timestamp" => RunSelector::Timestamp,
            other => {
                return Err(validation(format!(
                    "Unsupported artifact run selector '{other}'. Use latest or timestamp."
                )));
            }
        }
    };
    let config_path = crate::profile_config::resolve_profile_config_path();
    let config = if config_path.is_file() {
        Some(crate::profile_config::load_profile_config_file(
            &config_path,
        )?)
    } else {
        None
    };
    let artifact_root =
        crate::profile_config::resolve_artifact_root_path(config.as_ref(), &config_path);
    let scope_root = profile_scope_path(&artifact_root, profile);
    let run_id = resolve_run_id(&scope_root, &selector)?;
    Ok(Some(run_root_path(&scope_root, &run_id).join(lane_path)))
}

/// Format a filesystem-safe timestamp run id in local time:
/// `YYYY-MM-DDTHHmmssZZZZ` (example: `2026-04-19T091500+0800`).
pub fn format_timestamp_run_id(now: DateTime<Local>) -> String {
    now.format("%Y-%m-%dT%H%M%S%z").to_string()
}

pub fn generate_timestamp_run_id() -> String {
    format_timestamp_run_id(Local::now())
}

pub fn build_latest_run_pointer(
    profile: &str,
    run_id: &str,
    updated_at: DateTime<Local>,
) -> LatestRunPointer {
    LatestRunPointer {
        schema_version: LATEST_RUN_POINTER_SCHEMA_VERSION,
        kind: LATEST_RUN_POINTER_KIND.to_string(),
        profile: profile.to_string(),
        run_id: run_id.to_string(),
        run_path: format!("runs/{run_id}"),
        updated_at: updated_at.to_rfc3339(),
    }
}

pub fn read_latest_run_pointer(scope_root: &Path) -> Result<Option<LatestRunPointer>> {
    let path = latest_run_pointer_path(scope_root);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    serde_json::from_str::<LatestRunPointer>(&raw)
        .map(Some)
        .map_err(|error| {
            validation(format!(
                "Failed to parse latest run pointer {}: {error}",
                path.display()
            ))
        })
}

pub fn write_latest_run_pointer(scope_root: &Path, pointer: &LatestRunPointer) -> Result<()> {
    let path = latest_run_pointer_path(scope_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            validation(format!(
                "Failed to create latest run pointer directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    let rendered = serde_json::to_string_pretty(pointer).map_err(|error| {
        validation(format!(
            "Failed to render latest run pointer {}: {error}",
            path.display()
        ))
    })?;
    fs::write(&path, format!("{rendered}\n")).map_err(|error| {
        validation(format!(
            "Failed to write latest run pointer {}: {error}",
            path.display()
        ))
    })?;
    Ok(())
}

pub fn record_latest_run(
    scope_root: &Path,
    profile: &str,
    run_id: &str,
) -> Result<LatestRunPointer> {
    validate_run_id(run_id)?;
    let pointer = build_latest_run_pointer(profile, run_id, Local::now());
    write_latest_run_pointer(scope_root, &pointer)?;
    Ok(pointer)
}

pub fn resolve_run_id(scope_root: &Path, selector: &RunSelector) -> Result<String> {
    match selector {
        RunSelector::Latest => {
            let pointer = read_latest_run_pointer(scope_root)?.ok_or_else(|| {
                validation(format!(
                    "No latest run recorded at {}.",
                    latest_run_pointer_path(scope_root).display()
                ))
            })?;
            validate_run_id(&pointer.run_id)?;
            Ok(pointer.run_id)
        }
        RunSelector::Timestamp => Ok(generate_timestamp_run_id()),
        RunSelector::RunId(run_id) => {
            validate_run_id(run_id)?;
            Ok(run_id.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validate_run_id_rejects_empty_dot_dotdot_and_separators() {
        assert!(validate_run_id("").is_err());
        assert!(validate_run_id(".").is_err());
        assert!(validate_run_id("..").is_err());
        assert!(validate_run_id("a/b").is_err());
        assert!(validate_run_id("a\\b").is_err());
        assert!(validate_run_id("ok-1_2.3").is_ok());
    }

    #[test]
    fn format_timestamp_run_id_is_filesystem_safe_and_stable_length() {
        let now = Local::now();
        let run_id = format_timestamp_run_id(now);
        assert_eq!(run_id.len(), 22);
        assert_eq!(&run_id[4..5], "-");
        assert_eq!(&run_id[7..8], "-");
        assert_eq!(&run_id[10..11], "T");
        assert!(!run_id.contains('/'));
        assert!(!run_id.contains('\\'));
    }

    #[test]
    fn latest_run_pointer_round_trips() {
        let temp = tempdir().unwrap();
        let scope_root = temp.path().join("scope");

        let pointer = record_latest_run(&scope_root, "dev", "run-1").unwrap();
        assert_eq!(pointer.profile, "dev");
        assert_eq!(pointer.run_id, "run-1");
        assert_eq!(pointer.run_path, "runs/run-1");
        assert!(!pointer.updated_at.is_empty());

        let loaded = read_latest_run_pointer(&scope_root).unwrap().unwrap();
        assert_eq!(loaded, pointer);
    }

    #[test]
    fn resolve_run_id_latest_errors_when_missing() {
        let temp = tempdir().unwrap();
        let scope_root = temp.path().join("scope");

        let err = resolve_run_id(&scope_root, &RunSelector::Latest).unwrap_err();
        assert!(err.to_string().contains("No latest run recorded"));
    }
}
