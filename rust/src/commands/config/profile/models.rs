use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::profile_secret_store::StoredSecretRef;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileConfigFile {
    #[serde(default)]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub artifact_root: Option<PathBuf>,
    #[serde(default)]
    pub profiles: BTreeMap<String, ConnectionProfile>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionProfile {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub token_env: Option<String>,
    #[serde(default)]
    pub token_store: Option<StoredSecretRef>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub username_env: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub password_env: Option<String>,
    #[serde(default)]
    pub password_store: Option<StoredSecretRef>,
    #[serde(default)]
    pub org_id: Option<i64>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub verify_ssl: Option<bool>,
    #[serde(default)]
    pub insecure: Option<bool>,
    #[serde(default)]
    pub ca_cert: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedProfile {
    pub name: String,
    pub source_path: PathBuf,
    pub profile: ConnectionProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConnectionSettings {
    pub url: String,
    pub api_token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub org_id: Option<i64>,
    pub timeout: u64,
    pub verify_ssl: bool,
    pub ca_cert: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub struct ConnectionMergeInput<'a> {
    pub url: &'a str,
    pub url_default: &'a str,
    pub api_token: Option<&'a str>,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub org_id: Option<i64>,
    pub timeout: u64,
    pub timeout_default: u64,
    pub verify_ssl: bool,
    pub insecure: bool,
    pub ca_cert: Option<&'a Path>,
}
