//! Unified CLI help examples and rendering helpers.
//!
//! The CLI help subsystem is split by responsibility:
//! grouped entrypoint specs, grouped rendering, schema-help routing,
//! and contextual clap rendering.

mod grouped;
pub(crate) mod grouped_specs;
mod routing;
mod schema;

pub(crate) use routing::{
    canonicalize_inferred_subcommands, UNIFIED_ACCESS_HELP_TEXT, UNIFIED_ALERT_HELP_TEXT,
    UNIFIED_DATASOURCE_HELP_TEXT, UNIFIED_SYNC_HELP_TEXT,
};
pub use routing::{
    maybe_render_unified_help_from_os_args, render_unified_help_full_text,
    render_unified_help_text, render_unified_version_text,
};
