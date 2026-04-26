use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DashboardWorkspaceLayoutKind {
    Export,
    GitSync,
}

impl DashboardWorkspaceLayoutKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Export => "export",
            Self::GitSync => "git-sync",
        }
    }
}

/// Resolve a dashboard workspace root from a local path.
pub(crate) fn infer_dashboard_workspace_root(input_dir: &Path) -> PathBuf {
    if let Some(workspace_root) = infer_workspace_root_from_layout_ancestors(input_dir) {
        return workspace_root;
    }
    infer_dashboard_workspace_root_fallback(input_dir)
}

fn infer_workspace_root_from_layout_ancestors(input_dir: &Path) -> Option<PathBuf> {
    if input_dir.is_file()
        && input_dir.file_name().and_then(|name| name.to_str()) == Some("datasources.yaml")
    {
        let parent = input_dir.parent();
        let grandparent = parent.and_then(Path::parent);
        let great_grandparent = grandparent.and_then(Path::parent);
        if parent.and_then(Path::file_name).and_then(|v| v.to_str()) == Some("provisioning")
            && grandparent
                .and_then(Path::file_name)
                .and_then(|v| v.to_str())
                == Some("datasources")
        {
            return Some(great_grandparent.unwrap_or(input_dir).to_path_buf());
        }
    }

    for ancestor in input_dir.ancestors() {
        let Some(name) = ancestor.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        match name {
            "dashboards" | "alerts" | "datasources" => {
                if ancestor
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|v| v.to_str())
                    == Some("provisioning")
                {
                    continue;
                }
                return Some(ancestor.parent().unwrap_or(ancestor).to_path_buf());
            }
            "git-sync" => {
                let parent = ancestor.parent();
                let grandparent = parent.and_then(Path::parent);
                if parent.and_then(Path::file_name).and_then(|v| v.to_str()) == Some("dashboards") {
                    return Some(grandparent.unwrap_or(ancestor).to_path_buf());
                }
            }
            _ => {}
        }
    }

    None
}

fn infer_dashboard_workspace_root_fallback(input_dir: &Path) -> PathBuf {
    let Some(name) = input_dir.file_name().and_then(|name| name.to_str()) else {
        return input_dir.to_path_buf();
    };
    if input_dir.is_file() {
        if name == "datasources.yaml" {
            let parent = input_dir.parent();
            let grandparent = parent.and_then(Path::parent);
            let great_grandparent = grandparent.and_then(Path::parent);
            if parent.and_then(Path::file_name).and_then(|v| v.to_str()) == Some("provisioning")
                && grandparent
                    .and_then(Path::file_name)
                    .and_then(|v| v.to_str())
                    == Some("datasources")
            {
                return great_grandparent.unwrap_or(input_dir).to_path_buf();
            }
        }
        return input_dir.parent().unwrap_or(input_dir).to_path_buf();
    }
    let parent = input_dir.parent();
    let grandparent = parent.and_then(Path::parent);
    let great_grandparent = grandparent.and_then(Path::parent);
    match name {
        "dashboards" | "alerts" | "datasources" => parent.unwrap_or(input_dir).to_path_buf(),
        "git-sync" => {
            if parent.and_then(Path::file_name).and_then(|v| v.to_str()) == Some("dashboards") {
                grandparent.unwrap_or(input_dir).to_path_buf()
            } else {
                input_dir.to_path_buf()
            }
        }
        "raw" | "provisioning" => {
            if parent.and_then(Path::file_name).and_then(|v| v.to_str()) == Some("git-sync")
                && grandparent
                    .and_then(Path::file_name)
                    .and_then(|v| v.to_str())
                    == Some("dashboards")
            {
                great_grandparent.unwrap_or(input_dir).to_path_buf()
            } else {
                grandparent.unwrap_or(input_dir).to_path_buf()
            }
        }
        _ if matches!(
            parent.and_then(Path::file_name).and_then(|v| v.to_str()),
            Some("git-sync")
        ) && matches!(
            grandparent
                .and_then(Path::file_name)
                .and_then(|v| v.to_str()),
            Some("dashboards")
        ) =>
        {
            great_grandparent.unwrap_or(input_dir).to_path_buf()
        }
        _ => input_dir.to_path_buf(),
    }
}

fn path_from_components(components: &[Component<'_>]) -> PathBuf {
    let mut path = PathBuf::new();
    for component in components {
        path.push(component.as_os_str());
    }
    path
}

fn canonical_dashboard_variant_root(input_dir: &Path, variant_dir_name: &str) -> Option<PathBuf> {
    let components: Vec<Component<'_>> = input_dir.components().collect();
    let dashboards_index = components
        .iter()
        .position(|component| component.as_os_str() == "dashboards")?;
    if dashboards_index + 2 < components.len()
        && components[dashboards_index + 1].as_os_str() == "git-sync"
        && components[dashboards_index + 2].as_os_str() == variant_dir_name
    {
        return Some(path_from_components(&components[..=dashboards_index + 2]));
    }
    if dashboards_index + 1 < components.len()
        && components[dashboards_index + 1].as_os_str() == variant_dir_name
    {
        return Some(path_from_components(&components[..=dashboards_index + 1]));
    }
    None
}

fn variant_root_uses_git_sync_wrapper(path: &Path, variant_dir_name: &str) -> bool {
    let components: Vec<Component<'_>> = path.components().collect();
    let dashboards_index = components
        .iter()
        .position(|component| component.as_os_str() == "dashboards");
    matches!(
        dashboards_index,
        Some(index)
            if index + 2 < components.len()
                && components[index + 1].as_os_str() == "git-sync"
                && components[index + 2].as_os_str() == variant_dir_name
    )
}

/// Classify whether a dashboard review path resolves to a plain export tree or a
/// Git Sync-wrapped repo layout for the requested variant.
pub(crate) fn classify_dashboard_workspace_layout(
    input_dir: &Path,
    variant_dir_name: &str,
) -> DashboardWorkspaceLayoutKind {
    let candidate = canonical_dashboard_variant_root(input_dir, variant_dir_name)
        .or_else(|| {
            if input_dir.is_dir()
                && input_dir.file_name().and_then(|name| name.to_str()) == Some(variant_dir_name)
            {
                Some(input_dir.to_path_buf())
            } else {
                None
            }
        })
        .or_else(|| resolve_dashboard_workspace_variant_dir(input_dir, variant_dir_name));
    if candidate
        .as_deref()
        .map(|path| variant_root_uses_git_sync_wrapper(path, variant_dir_name))
        .unwrap_or(false)
    {
        DashboardWorkspaceLayoutKind::GitSync
    } else {
        DashboardWorkspaceLayoutKind::Export
    }
}

/// Resolve a dashboard variant root from a workspace, dashboards root, or repo root.
pub(crate) fn resolve_dashboard_workspace_variant_dir(
    input_dir: &Path,
    variant_dir_name: &str,
) -> Option<PathBuf> {
    if let Some(canonical_root) = canonical_dashboard_variant_root(input_dir, variant_dir_name) {
        return Some(canonical_root);
    }
    if input_dir.file_name().and_then(|name| name.to_str()) == Some(variant_dir_name)
        && input_dir.is_dir()
    {
        return Some(input_dir.to_path_buf());
    }

    let direct_candidate = input_dir.join(variant_dir_name);
    if direct_candidate.is_dir() {
        return Some(direct_candidate);
    }

    let dashboards_dir =
        if input_dir.file_name().and_then(|name| name.to_str()) == Some("dashboards") {
            input_dir.to_path_buf()
        } else {
            input_dir.join("dashboards")
        };
    let direct_dashboards_candidate = dashboards_dir.join(variant_dir_name);
    if direct_dashboards_candidate.is_dir() {
        return Some(direct_dashboards_candidate);
    }

    let git_sync_dir = if input_dir.file_name().and_then(|name| name.to_str()) == Some("git-sync") {
        input_dir.to_path_buf()
    } else {
        dashboards_dir.join("git-sync")
    };
    let wrapped_candidate = git_sync_dir.join(variant_dir_name);
    wrapped_candidate.is_dir().then_some(wrapped_candidate)
}
