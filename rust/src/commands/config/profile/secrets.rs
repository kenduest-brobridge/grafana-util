use crate::common::{env_value, validation, Result};
use crate::profile_secret_store::{
    read_secret_from_encrypted_file, read_secret_from_os_store, resolve_secret_file_path,
    resolve_secret_key_path, EncryptedSecretKeySource, OsSecretStore, StoredSecretRef,
    SystemOsSecretStore,
};

use super::SelectedProfile;

pub(crate) fn resolve_credential_value(
    cli_value: Option<&str>,
    profile_literal: Option<&str>,
    profile_store: Option<&StoredSecretRef>,
    profile_env: Option<&str>,
    selected_profile: Option<&SelectedProfile>,
    field_name: &str,
) -> Result<Option<String>> {
    if let Some(value) = cli_value.filter(|value| !value.is_empty()) {
        return Ok(Some(value.to_string()));
    }
    if let Some(value) = profile_literal.filter(|value| !value.is_empty()) {
        return Ok(Some(value.to_string()));
    }
    if let Some(store_ref) = profile_store {
        return Ok(Some(resolve_stored_profile_secret_with_store(
            store_ref,
            selected_profile,
            field_name,
            &SystemOsSecretStore,
        )?));
    }
    let Some(env_name) = profile_env.filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let value = env_value(env_name).ok_or_else(|| {
        let profile_name = selected_profile
            .map(|profile| profile.name.as_str())
            .unwrap_or("default");
        validation(format!(
            "Profile `{profile_name}` expected env var `{env_name}` for {field_name}, but it is not set."
        ))
    })?;
    Ok(Some(value))
}

pub(crate) fn resolve_stored_profile_secret_with_store<S: OsSecretStore>(
    store_ref: &StoredSecretRef,
    selected_profile: Option<&SelectedProfile>,
    field_name: &str,
    os_store: &S,
) -> Result<String> {
    match store_ref.provider.as_str() {
        "os" => read_secret_from_os_store(os_store, &store_ref.key),
        "encrypted-file" => {
            let selected = selected_profile.ok_or_else(|| {
                validation(format!(
                    "Profile secret store reference for {field_name} requires a selected profile context."
                ))
            })?;
            let secret_file_path =
                resolve_secret_file_path(&selected.source_path, store_ref.path.as_deref());
            let key_source = if let Some(env_name) = store_ref.passphrase_env.as_deref() {
                let value = env_value(env_name).ok_or_else(|| {
                    validation(format!(
                        "Profile `{}` expected passphrase env var `{env_name}` for {field_name}, but it is not set.",
                        selected.name
                    ))
                })?;
                EncryptedSecretKeySource::Passphrase(value)
            } else {
                let key_path = resolve_secret_key_path(&secret_file_path);
                EncryptedSecretKeySource::LocalKeyFile(key_path)
            };
            read_secret_from_encrypted_file(&secret_file_path, &key_source, &store_ref.key)
        }
        other => Err(validation(format!(
            "Unsupported profile secret provider `{other}` for {field_name}."
        ))),
    }
}
