//! Browser-driven dashboard screenshot helpers.
//!
//! Purpose:
//! - Build Grafana dashboard URLs for browser capture.
//! - Validate screenshot CLI arguments before browser launch.
//! - Reuse dashboard auth headers for a headless Chromium session.
//! - Capture PNG, JPEG, or PDF output through a browser-rendered page.
#![cfg_attr(not(feature = "browser"), allow(dead_code))]

#[cfg(feature = "browser")]
#[path = "screenshot_full_page.rs"]
mod screenshot_full_page;
#[path = "screenshot_header.rs"]
mod screenshot_header;
#[path = "screenshot_runtime.rs"]
mod screenshot_runtime;
mod url;

use std::path::Path;

use crate::common::{message, Result};

#[cfg(feature = "browser")]
use super::ScreenshotFullPageOutput;
use super::{ScreenshotArgs, ScreenshotOutputFormat};
#[allow(unused_imports)]
pub(crate) use screenshot_header::resolve_manifest_title;
use screenshot_runtime::parse_dashboard_url_state;
pub(crate) use screenshot_runtime::{parse_vars_query, DashboardUrlState};

#[cfg(feature = "browser")]
use headless_chrome::protocol::cdp::Page;
#[cfg(feature = "browser")]
use headless_chrome::types::PrintToPdfOptions;
#[cfg(feature = "browser")]
use image::ImageFormat;
#[cfg(feature = "browser")]
use std::fs;

#[cfg(feature = "browser")]
use super::build_auth_context;
#[cfg(feature = "browser")]
use screenshot_full_page::{
    build_screenshot_clip, capture_full_page_segments, warm_full_page_render,
    write_full_page_output,
};
#[cfg(feature = "browser")]
use screenshot_header::apply_header_if_requested;
#[cfg(feature = "browser")]
use screenshot_header::{build_header_spec, resolve_dashboard_metadata};
#[cfg(feature = "browser")]
use screenshot_runtime::{
    build_browser, build_browser_headers, collapse_sidebar_if_present, configure_capture_viewport,
    prepare_dashboard_capture_dom, read_numeric_expression, wait_for_dashboard_ready,
};

/// validate screenshot args.
pub fn validate_screenshot_args(args: &ScreenshotArgs) -> Result<()> {
    url::validate_screenshot_args(args)
}

/// infer screenshot output format.
pub fn infer_screenshot_output_format(
    output: &Path,
    explicit: Option<ScreenshotOutputFormat>,
) -> Result<ScreenshotOutputFormat> {
    url::infer_screenshot_output_format(output, explicit)
}

/// Purpose: implementation note.
///
/// Args: see function signature.
/// Returns: see implementation.
pub fn build_dashboard_capture_url(args: &ScreenshotArgs) -> Result<String> {
    url::build_dashboard_capture_url(args)
}

/// capture dashboard screenshot.
pub fn capture_dashboard_screenshot(args: &ScreenshotArgs) -> Result<()> {
    #[cfg(feature = "browser")]
    {
        // Call graph (hierarchy): this function is used in related modules.
        // Upstream callers: dashboard.rs:run_dashboard_cli, dashboard.rs:run_dashboard_cli_with_client
        // Downstream callees: common.rs:message, dashboard_screenshot.rs:build_browser, dashboard_screenshot.rs:build_browser_headers, dashboard_screenshot.rs:build_dashboard_capture_url, dashboard_screenshot.rs:build_screenshot_clip, dashboard_screenshot.rs:capture_full_page_segments, dashboard_screenshot.rs:collapse_sidebar_if_present, dashboard_screenshot.rs:configure_capture_viewport, dashboard_screenshot.rs:infer_screenshot_output_format, dashboard_screenshot.rs:prepare_dashboard_capture_dom, screenshot_header.rs:apply_header_if_requested, screenshot_header.rs:build_header_spec, screenshot_header.rs:resolve_dashboard_metadata ...

        let mut resolved_args = args.clone();
        validate_screenshot_args(&resolved_args)?;
        let output_format =
            infer_screenshot_output_format(&resolved_args.output, resolved_args.output_format)?;
        let capture_metadata = resolve_dashboard_metadata(&mut resolved_args)?;
        let url = build_dashboard_capture_url(&resolved_args)?;
        if resolved_args.print_capture_url {
            eprintln!("Capture URL: {url}");
        }
        let header_spec = build_header_spec(&resolved_args, &url, &capture_metadata);
        let auth = build_auth_context(&args.common)?;

        if let Some(parent) = resolved_args.output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let browser = build_browser(&resolved_args)?;
        let tab = browser
            .new_tab()
            .map_err(|error| message(format!("Failed to create Chromium tab: {error}")))?;
        configure_capture_viewport(&tab, &resolved_args)?;

        tab.set_extra_http_headers(build_browser_headers(&auth.headers))
            .map_err(|error| message(format!("Failed to set Chromium request headers: {error}")))?;

        tab.navigate_to(&url)
            .map_err(|error| message(format!("Failed to open dashboard URL: {error}")))?;
        wait_for_dashboard_ready(&tab, resolved_args.wait_ms)?;

        collapse_sidebar_if_present(&tab)?;
        let capture_offsets = prepare_dashboard_capture_dom(&tab)?;
        warm_full_page_render(&tab, &resolved_args)?;
        let screenshot_clip = build_screenshot_clip(&tab, &resolved_args)?;

        match output_format {
            ScreenshotOutputFormat::Png => {
                if resolved_args.full_page {
                    let segments = capture_full_page_segments(
                        &tab,
                        &resolved_args,
                        &capture_offsets,
                        Page::CaptureScreenshotFormatOption::Png,
                        None,
                    )?;
                    write_full_page_output(
                        &resolved_args,
                        &header_spec,
                        &capture_metadata,
                        output_format,
                        segments,
                        ImageFormat::Png,
                    )?;
                } else {
                    let bytes = tab
                        .capture_screenshot(
                            Page::CaptureScreenshotFormatOption::Png,
                            None,
                            screenshot_clip.clone(),
                            true,
                        )
                        .map_err(|error| {
                            message(format!("Failed to capture PNG screenshot: {error}"))
                        })?;
                    let bytes = apply_header_if_requested(
                        bytes,
                        &resolved_args,
                        &header_spec,
                        ImageFormat::Png,
                    )?;
                    fs::write(&resolved_args.output, bytes)?;
                }
            }
            ScreenshotOutputFormat::Jpeg => {
                if resolved_args.full_page {
                    let segments = capture_full_page_segments(
                        &tab,
                        &resolved_args,
                        &capture_offsets,
                        Page::CaptureScreenshotFormatOption::Jpeg,
                        Some(90),
                    )?;
                    write_full_page_output(
                        &resolved_args,
                        &header_spec,
                        &capture_metadata,
                        output_format,
                        segments,
                        ImageFormat::Jpeg,
                    )?;
                } else {
                    let bytes = tab
                        .capture_screenshot(
                            Page::CaptureScreenshotFormatOption::Jpeg,
                            Some(90),
                            screenshot_clip,
                            true,
                        )
                        .map_err(|error| {
                            message(format!("Failed to capture JPEG screenshot: {error}"))
                        })?;
                    let bytes = apply_header_if_requested(
                        bytes,
                        &resolved_args,
                        &header_spec,
                        ImageFormat::Jpeg,
                    )?;
                    fs::write(&resolved_args.output, bytes)?;
                }
            }
            ScreenshotOutputFormat::Pdf => {
                if resolved_args.full_page_output != ScreenshotFullPageOutput::Single {
                    return Err(message(
                        "PDF output does not support --full-page-output tiles or manifest.",
                    ));
                }
                let pdf = tab
                    .print_to_pdf(Some(PrintToPdfOptions {
                        landscape: Some(false),
                        display_header_footer: Some(false),
                        print_background: Some(true),
                        scale: None,
                        paper_width: None,
                        paper_height: None,
                        margin_top: None,
                        margin_bottom: None,
                        margin_left: None,
                        margin_right: None,
                        page_ranges: None,
                        ignore_invalid_page_ranges: None,
                        header_template: None,
                        footer_template: None,
                        prefer_css_page_size: Some(true),
                        transfer_mode: None,
                        generate_tagged_pdf: None,
                        generate_document_outline: None,
                    }))
                    .map_err(|error| message(format!("Failed to render PDF output: {error}")))?;
                fs::write(&resolved_args.output, pdf)?;
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "browser"))]
    {
        let _ = args;
        Err(message(
            "Dashboard screenshot support was not built in. Rebuild with the `browser` feature enabled.",
        ))
    }
}
