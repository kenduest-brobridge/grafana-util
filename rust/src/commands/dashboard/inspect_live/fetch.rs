use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::common::{message, string_field, GrafanaCliError, Result};

use super::super::export::build_output_path;
use super::super::files::write_dashboard;

const MAX_LIVE_INSPECT_CONCURRENCY: usize = 16;
const LIVE_INSPECT_RETRY_LIMIT: usize = 3;
const LIVE_INSPECT_RETRY_BACKOFF_MS: u64 = 200;

fn build_live_scan_progress_bar(total: usize) -> ProgressBar {
    let bar = ProgressBar::new(total as u64);
    let style = ProgressStyle::with_template(
        "{spinner:.green} dashboards {pos}/{len} [{bar:40.cyan/blue}] {msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar());
    bar.set_style(style.progress_chars("##-"));
    bar
}

fn bounded_live_inspect_concurrency(requested: usize) -> usize {
    requested.clamp(1, MAX_LIVE_INSPECT_CONCURRENCY)
}

fn is_retryable_live_fetch_error(error: &GrafanaCliError) -> bool {
    matches!(error.status_code(), Some(429 | 502 | 503 | 504))
}

fn fetch_live_dashboard_with_retries<F>(uid: &str, fetch_dashboard: &F) -> Result<Value>
where
    F: Fn(&str) -> Result<Value> + Sync,
{
    let mut attempt = 0usize;
    loop {
        attempt += 1;
        match fetch_dashboard(uid) {
            Ok(payload) => return Ok(payload),
            Err(error)
                if is_retryable_live_fetch_error(&error) && attempt < LIVE_INSPECT_RETRY_LIMIT =>
            {
                thread::sleep(Duration::from_millis(
                    LIVE_INSPECT_RETRY_BACKOFF_MS * attempt as u64,
                ));
            }
            Err(error) => {
                return Err(error.with_context(format!(
                    "Failed to fetch live dashboard uid={uid} during summary-live"
                )))
            }
        }
    }
}

pub(crate) fn snapshot_live_dashboard_export_with_fetcher<F>(
    raw_dir: &Path,
    summaries: &[Map<String, Value>],
    concurrency: usize,
    progress: bool,
    fetch_dashboard: F,
) -> Result<usize>
where
    F: Fn(&str) -> Result<Value> + Sync,
{
    // Only dashboard fetches fan out; the file layout and progress semantics stay
    // deterministic so the staged export remains comparable to offline export roots.
    fs::create_dir_all(raw_dir)?;
    let progress_bar = if progress {
        Some(build_live_scan_progress_bar(summaries.len()))
    } else {
        None
    };
    let thread_count = bounded_live_inspect_concurrency(concurrency);
    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .map_err(|error| {
            message(format!(
                "Failed to build dashboard scan worker pool: {error}"
            ))
        })?;
    let results = pool.install(|| {
        summaries
            .par_iter()
            .map(|summary| {
                let uid = string_field(summary, "uid", "");
                let output_path = build_output_path(raw_dir, summary, false);
                let payload = fetch_live_dashboard_with_retries(&uid, &fetch_dashboard)?;
                write_dashboard(&payload, &output_path, false).map_err(|error| {
                    error.with_context(format!(
                        "Failed to stage live dashboard uid={uid} into {}",
                        output_path.display()
                    ))
                })?;
                if let Some(bar) = progress_bar.as_ref() {
                    bar.inc(1);
                    bar.set_message(uid);
                }
                Ok::<(), GrafanaCliError>(())
            })
            .collect::<Vec<_>>()
    });
    if let Some(bar) = progress_bar.as_ref() {
        bar.finish_and_clear();
    }
    for result in results {
        result?;
    }
    Ok(summaries.len())
}
