//! Generic read-only resource queries for Grafana objects.
//!
//! This surface intentionally stays narrower than the higher-level workflow
//! namespaces. It exists so operators can inspect a few live Grafana resource
//! kinds before richer domain-specific flows exist.

mod catalog;
mod cli_defs;
mod render;
mod runtime;

#[path = "help.rs"]
mod resource_help;

pub use cli_defs::{
    ResourceCliArgs, ResourceCommand, ResourceDescribeArgs, ResourceGetArgs, ResourceKind,
    ResourceKindsArgs, ResourceListArgs, ResourceOutputFormat,
};
pub use runtime::run_resource_cli;
