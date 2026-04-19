use std::path::Path;

pub(crate) fn format_export_progress_line(
    current: usize,
    total: usize,
    uid: &str,
    dry_run: bool,
) -> String {
    format!(
        "{} dashboard {current}/{total}: {uid}",
        if dry_run { "Would export" } else { "Exporting" }
    )
}

pub(crate) fn format_export_verbose_line(
    kind: &str,
    uid: &str,
    path: &Path,
    dry_run: bool,
) -> String {
    format!(
        "{} {kind:<6} {uid} -> {}",
        if dry_run { "Would export" } else { "Exported" },
        path.display()
    )
}
