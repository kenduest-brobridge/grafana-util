use std::path::Path;

use crate::common::{validation, Result};

use super::{
    load_profile_config_file, resolve_profile_config_path, ProfileConfigFile, SelectedProfile,
};

pub fn load_selected_profile(profile_name: Option<&str>) -> Result<Option<SelectedProfile>> {
    let path = resolve_profile_config_path();
    if !path.exists() {
        if let Some(name) = profile_name {
            return Err(validation(format!(
                "Requested profile `{name}` but config file {} does not exist.",
                path.display()
            )));
        }
        return Ok(None);
    }
    let config = load_profile_config_file(&path)?;
    select_profile(&config, profile_name, &path)
}

pub fn select_profile(
    config: &ProfileConfigFile,
    requested_profile: Option<&str>,
    source_path: &Path,
) -> Result<Option<SelectedProfile>> {
    let chosen_name = if let Some(name) = requested_profile {
        Some(name.to_string())
    } else if let Some(default_name) = config.default_profile.as_ref() {
        Some(default_name.clone())
    } else if config.profiles.len() == 1 {
        config.profiles.keys().next().cloned()
    } else {
        None
    };
    let Some(name) = chosen_name else {
        return Ok(None);
    };
    let Some(profile) = config.profiles.get(&name) else {
        return Err(validation(format!(
            "Profile `{name}` was not found in {}.",
            source_path.display()
        )));
    };
    Ok(Some(SelectedProfile {
        name,
        source_path: source_path.to_path_buf(),
        profile: profile.clone(),
    }))
}
