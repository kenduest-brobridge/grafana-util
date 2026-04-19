//! Profile configuration loading, merge, and persistence contracts.
//!
//! Responsibilities:
//! - Resolve active profile selection from CLI/profile/env inputs.
//! - Merge profile data with inline arguments and defaults.
//! - Read/write profile configuration files used by Rust/CLI entrypoints.

#[path = "connection.rs"]
mod connection;
#[path = "io.rs"]
mod io;
#[path = "models.rs"]
mod models;
#[path = "paths.rs"]
mod paths;
#[path = "secrets.rs"]
mod secrets;
#[path = "selection.rs"]
mod selection;

pub use connection::resolve_connection_settings;
pub use io::{load_profile_config_file, render_profile_init_template, save_profile_config_file};
pub use models::{
    ConnectionMergeInput, ConnectionProfile, ProfileConfigFile, ResolvedConnectionSettings,
    SelectedProfile,
};
pub use paths::{
    default_artifact_root_path_for_config_path, default_profile_config_path,
    resolve_artifact_root_path, resolve_profile_config_path, set_profile_config_path_override,
    DEFAULT_PROFILE_CONFIG_FILENAME, PROFILE_CONFIG_ENV_VAR,
};
pub use selection::{load_selected_profile, select_profile};

#[cfg(test)]
pub(crate) use secrets::resolve_stored_profile_secret_with_store;

#[cfg(test)]
mod tests {
    use super::{
        default_profile_config_path, load_profile_config_file, render_profile_init_template,
        resolve_artifact_root_path, resolve_connection_settings,
        resolve_stored_profile_secret_with_store, save_profile_config_file, select_profile,
        ConnectionMergeInput, ConnectionProfile, ProfileConfigFile, SelectedProfile,
    };
    use crate::common::{validation, Result};
    use crate::profile_secret_store::{
        write_secret_to_encrypted_file, EncryptedSecretKeySource, OsSecretStore, StoredSecretRef,
    };
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static TEST_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> MutexGuard<'static, ()> {
        TEST_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("test env lock poisoned")
    }
    use tempfile::tempdir;

    #[derive(Default)]
    struct MemoryOsSecretStore {
        values: RefCell<BTreeMap<String, String>>,
    }

    impl OsSecretStore for MemoryOsSecretStore {
        fn set_secret(&self, key: &str, value: &str) -> Result<()> {
            self.values
                .borrow_mut()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn get_secret(&self, key: &str) -> Result<String> {
            self.values
                .borrow()
                .get(key)
                .cloned()
                .ok_or_else(|| validation(format!("missing key {key}")))
        }
    }

    #[test]
    fn default_profile_config_path_uses_repo_local_filename() {
        assert_eq!(
            default_profile_config_path().to_string_lossy(),
            "grafana-util.yaml"
        );
    }

    #[test]
    fn select_profile_prefers_requested_name_then_default_then_single_profile() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "dev".to_string(),
            ConnectionProfile {
                url: Some("http://dev".to_string()),
                ..ConnectionProfile::default()
            },
        );
        profiles.insert(
            "prod".to_string(),
            ConnectionProfile {
                url: Some("http://prod".to_string()),
                ..ConnectionProfile::default()
            },
        );
        let config = ProfileConfigFile {
            default_profile: Some("prod".to_string()),
            profiles,
            ..ProfileConfigFile::default()
        };

        let selected = select_profile(&config, Some("dev"), Path::new("./grafana-util.yaml"))
            .unwrap()
            .unwrap();
        assert_eq!(selected.name, "dev");

        let selected = select_profile(&config, None, Path::new("./grafana-util.yaml"))
            .unwrap()
            .unwrap();
        assert_eq!(selected.name, "prod");
    }

    #[test]
    fn resolve_connection_settings_prefers_cli_and_falls_back_to_profile() {
        let selected_profile = super::SelectedProfile {
            name: "prod".to_string(),
            source_path: PathBuf::from("./grafana-util.yaml"),
            profile: ConnectionProfile {
                url: Some("https://grafana.example.com".to_string()),
                token: Some("profile-token".to_string()),
                org_id: Some(9),
                timeout: Some(45),
                verify_ssl: Some(true),
                ..ConnectionProfile::default()
            },
        };
        let resolved = resolve_connection_settings(
            ConnectionMergeInput {
                url: "http://127.0.0.1:3000",
                url_default: "http://127.0.0.1:3000",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            Some(&selected_profile),
        )
        .unwrap();

        assert_eq!(resolved.url, "https://grafana.example.com");
        assert_eq!(resolved.api_token.as_deref(), Some("profile-token"));
        assert_eq!(resolved.org_id, Some(9));
        assert_eq!(resolved.timeout, 45);
        assert!(resolved.verify_ssl);
    }

    #[test]
    fn resolve_connection_settings_supports_profile_env_credentials() {
        let selected_profile = super::SelectedProfile {
            name: "prod".to_string(),
            source_path: PathBuf::from("./grafana-util.yaml"),
            profile: ConnectionProfile {
                token_env: Some("TEST_GRAFANA_PROFILE_TOKEN".to_string()),
                ..ConnectionProfile::default()
            },
        };
        env::set_var("TEST_GRAFANA_PROFILE_TOKEN", "token-from-env");
        let resolved = resolve_connection_settings(
            ConnectionMergeInput {
                url: "http://127.0.0.1:3000",
                url_default: "http://127.0.0.1:3000",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            Some(&selected_profile),
        )
        .unwrap();
        env::remove_var("TEST_GRAFANA_PROFILE_TOKEN");

        assert_eq!(resolved.api_token.as_deref(), Some("token-from-env"));
    }

    #[test]
    fn resolve_connection_settings_supports_grafana_url_env() {
        let _env_guard = env_lock();
        env::set_var("GRAFANA_URL", "https://env-grafana.example.com");
        let resolved = resolve_connection_settings(
            ConnectionMergeInput {
                url: "",
                url_default: "",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            None,
        )
        .unwrap();
        env::remove_var("GRAFANA_URL");

        assert_eq!(resolved.url, "https://env-grafana.example.com");
    }

    #[test]
    fn resolve_connection_settings_ignores_credentials_in_grafana_url_env() {
        let _env_guard = env_lock();
        env::set_var("GRAFANA_URL", "https://admin:secret@grafana.example.com");
        let resolved = resolve_connection_settings(
            ConnectionMergeInput {
                url: "",
                url_default: "",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            None,
        )
        .unwrap();
        env::remove_var("GRAFANA_URL");

        assert_eq!(resolved.url, "https://grafana.example.com/");
        assert_eq!(resolved.username, None);
        assert_eq!(resolved.password, None);
    }

    #[test]
    fn resolve_connection_settings_ignores_credentials_in_profile_url() {
        let _env_guard = env_lock();
        env::remove_var("GRAFANA_URL");
        let selected_profile = super::SelectedProfile {
            name: "prod".to_string(),
            source_path: PathBuf::from("./grafana-util.yaml"),
            profile: ConnectionProfile {
                url: Some("https://admin:secret@grafana.example.com".to_string()),
                ..ConnectionProfile::default()
            },
        };
        let resolved = resolve_connection_settings(
            ConnectionMergeInput {
                url: "",
                url_default: "",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            Some(&selected_profile),
        )
        .unwrap();

        assert_eq!(resolved.url, "https://grafana.example.com/");
        assert_eq!(resolved.username, None);
        assert_eq!(resolved.password, None);
    }

    #[test]
    fn resolve_connection_settings_requires_url_when_cli_env_and_profile_are_missing() {
        let _env_guard = env_lock();
        env::remove_var("GRAFANA_URL");
        let error = resolve_connection_settings(
            ConnectionMergeInput {
                url: "",
                url_default: "",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            None,
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("Grafana base URL is required. Pass --url, set GRAFANA_URL, or configure a profile with url."));
    }

    #[test]
    fn load_profile_config_file_reads_yaml_document() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("grafana-util.yaml");
        fs::write(
            &config_path,
            r#"default_profile: dev
profiles:
  dev:
    url: http://localhost:3000
    token_env: TEST_PROFILE_TOKEN
"#,
        )
        .unwrap();

        let config = load_profile_config_file(&config_path).unwrap();

        assert_eq!(config.default_profile.as_deref(), Some("dev"));
        assert_eq!(
            config.profiles["dev"].url.as_deref(),
            Some("http://localhost:3000")
        );
    }

    #[test]
    fn render_profile_init_template_contains_default_profiles() {
        let rendered = render_profile_init_template();

        assert!(rendered.contains("default_profile: dev"));
        assert!(rendered.contains("profiles:"));
        assert!(rendered.contains("token_env: GRAFANA_API_TOKEN"));
        assert!(rendered.contains("username: admin"));
    }

    #[test]
    fn save_profile_config_file_creates_parent_directories() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("nested/work/grafana-util.yaml");
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "dev".to_string(),
            ConnectionProfile {
                url: Some("http://127.0.0.1:3000".to_string()),
                ..ConnectionProfile::default()
            },
        );

        save_profile_config_file(
            &config_path,
            &ProfileConfigFile {
                default_profile: Some("dev".to_string()),
                profiles,
                ..ProfileConfigFile::default()
            },
        )
        .unwrap();

        assert!(config_path.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&config_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn resolve_artifact_root_path_defaults_under_config_dir() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("grafana-util.yaml");

        let resolved = resolve_artifact_root_path(None, &config_path);

        assert_eq!(resolved, dir.path().join(".grafana-util").join("artifacts"));
    }

    #[test]
    fn resolve_artifact_root_path_resolves_relative_to_config_dir() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("grafana-util.yaml");
        let config = ProfileConfigFile {
            artifact_root: Some(PathBuf::from("my-artifacts")),
            ..ProfileConfigFile::default()
        };

        let resolved = resolve_artifact_root_path(Some(&config), &config_path);

        assert_eq!(resolved, dir.path().join("my-artifacts"));
    }

    #[test]
    fn resolve_artifact_root_path_keeps_absolute_root() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("grafana-util.yaml");
        let absolute_root = dir.path().join("abs-root");
        let config = ProfileConfigFile {
            artifact_root: Some(absolute_root.clone()),
            ..ProfileConfigFile::default()
        };

        let resolved = resolve_artifact_root_path(Some(&config), &config_path);

        assert_eq!(resolved, absolute_root);
    }

    #[test]
    fn resolve_stored_profile_secret_supports_os_store_refs() {
        let store = MemoryOsSecretStore::default();
        store
            .set_secret("grafana-util/profile/dev/token", "token-from-store")
            .unwrap();
        let selected = SelectedProfile {
            name: "dev".to_string(),
            source_path: PathBuf::from("./grafana-util.yaml"),
            profile: ConnectionProfile::default(),
        };

        let value = resolve_stored_profile_secret_with_store(
            &StoredSecretRef {
                provider: "os".to_string(),
                key: "grafana-util/profile/dev/token".to_string(),
                ..StoredSecretRef::default()
            },
            Some(&selected),
            "token",
            &store,
        )
        .unwrap();

        assert_eq!(value, "token-from-store");
    }

    #[test]
    fn resolve_connection_settings_reads_encrypted_file_with_passphrase_env() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("configs/grafana-util.yaml");
        let secret_path = dir.path().join("configs/.grafana-util.secrets.yaml");
        write_secret_to_encrypted_file(
            &secret_path,
            &EncryptedSecretKeySource::Passphrase("hunter2".to_string()),
            "grafana-util/profile/prod/password",
            "secret-password",
        )
        .unwrap();
        env::set_var("PROFILE_SECRET_PASSPHRASE", "hunter2");
        let selected = SelectedProfile {
            name: "prod".to_string(),
            source_path: config_path,
            profile: ConnectionProfile {
                url: Some("https://grafana.example.com".to_string()),
                username: Some("admin".to_string()),
                password_store: Some(StoredSecretRef {
                    provider: "encrypted-file".to_string(),
                    key: "grafana-util/profile/prod/password".to_string(),
                    path: Some(PathBuf::from(".grafana-util.secrets.yaml")),
                    passphrase_env: Some("PROFILE_SECRET_PASSPHRASE".to_string()),
                }),
                ..ConnectionProfile::default()
            },
        };

        let resolved = resolve_connection_settings(
            ConnectionMergeInput {
                url: "http://127.0.0.1:3000",
                url_default: "http://127.0.0.1:3000",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            Some(&selected),
        )
        .unwrap();
        env::remove_var("PROFILE_SECRET_PASSPHRASE");

        assert_eq!(resolved.password.as_deref(), Some("secret-password"));
    }

    #[test]
    fn resolve_connection_settings_reads_encrypted_file_with_local_key_default_path() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("envs/dev/grafana-util.yaml");
        let secret_path = dir.path().join("envs/dev/.grafana-util.secrets.yaml");
        let key_path = dir.path().join("envs/dev/.grafana-util.secrets.key");
        write_secret_to_encrypted_file(
            &secret_path,
            &EncryptedSecretKeySource::LocalKeyFile(key_path),
            "grafana-util/profile/dev/token",
            "local-key-token",
        )
        .unwrap();
        let selected = SelectedProfile {
            name: "dev".to_string(),
            source_path: config_path,
            profile: ConnectionProfile {
                url: Some("http://127.0.0.1:3000".to_string()),
                token_store: Some(StoredSecretRef {
                    provider: "encrypted-file".to_string(),
                    key: "grafana-util/profile/dev/token".to_string(),
                    path: Some(PathBuf::from(".grafana-util.secrets.yaml")),
                    ..StoredSecretRef::default()
                }),
                ..ConnectionProfile::default()
            },
        };

        let resolved = resolve_connection_settings(
            ConnectionMergeInput {
                url: "http://127.0.0.1:3000",
                url_default: "http://127.0.0.1:3000",
                api_token: None,
                username: None,
                password: None,
                org_id: None,
                timeout: 30,
                timeout_default: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            Some(&selected),
        )
        .unwrap();

        assert_eq!(resolved.api_token.as_deref(), Some("local-key-token"));
    }
}
