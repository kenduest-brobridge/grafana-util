use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::common::env_value;

use super::ProfileConfigFile;

pub const DEFAULT_PROFILE_CONFIG_FILENAME: &str = "grafana-util.yaml";
pub const PROFILE_CONFIG_ENV_VAR: &str = "GRAFANA_UTIL_CONFIG";

static PROFILE_CONFIG_PATH_OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

pub fn default_profile_config_path() -> PathBuf {
    PathBuf::from(DEFAULT_PROFILE_CONFIG_FILENAME)
}

pub fn resolve_profile_config_path() -> PathBuf {
    if let Some(path) = PROFILE_CONFIG_PATH_OVERRIDE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("profile config override lock poisoned")
        .clone()
    {
        return path;
    }
    env_value(PROFILE_CONFIG_ENV_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(default_profile_config_path)
}

pub fn set_profile_config_path_override(path: Option<PathBuf>) {
    *PROFILE_CONFIG_PATH_OVERRIDE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("profile config override lock poisoned") = path;
}

pub fn default_artifact_root_path_for_config_path(config_path: &Path) -> PathBuf {
    let config_dir = config_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    config_dir.join(".grafana-util").join("artifacts")
}

pub fn resolve_artifact_root_path(
    config: Option<&ProfileConfigFile>,
    config_path: &Path,
) -> PathBuf {
    let default_root = default_artifact_root_path_for_config_path(config_path);
    let Some(config) = config else {
        return default_root;
    };
    let Some(root) = config.artifact_root.as_ref() else {
        return default_root;
    };
    if root.is_absolute() {
        return root.clone();
    }
    let config_dir = config_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    config_dir.join(root)
}
