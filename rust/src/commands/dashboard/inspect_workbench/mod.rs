//! Inspect workbench TUI module facade.
//!
//! The dashboard root keeps compatibility aliases for existing call sites, while
//! this directory owns the workbench runtime, state, rendering, and document
//! construction internals.
#![cfg(feature = "tui")]

mod content;
mod modal_state;
pub(crate) mod render;
mod render_helpers;
mod render_modal;
mod render_modal_sections;
mod runtime;
pub(crate) mod state;
pub(crate) mod support;

pub(crate) use runtime::run_inspect_workbench;
