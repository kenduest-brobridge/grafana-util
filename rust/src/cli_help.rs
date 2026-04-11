//! Unified CLI help examples and rendering helpers.
//!
//! The CLI help subsystem is split by responsibility:
//! grouped entrypoint specs, grouped rendering, schema-help routing,
//! contextual clap rendering, and legacy command hints.

mod grouped;
pub(crate) mod grouped_specs;
mod legacy;
mod routing;
mod schema;

pub use legacy::legacy_command_error_hint;
pub(crate) use routing::{
    canonicalize_inferred_subcommands, UNIFIED_ACCESS_HELP_TEXT, UNIFIED_ALERT_HELP_TEXT,
    UNIFIED_DATASOURCE_HELP_TEXT, UNIFIED_SYNC_HELP_TEXT,
};
pub use routing::{
    maybe_render_unified_help_from_os_args, render_unified_help_full_text,
    render_unified_help_text, render_unified_version_text,
};
