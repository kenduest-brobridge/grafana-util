//! CLI definitions for Core command surface and option compatibility behavior.

use clap::{Args, CommandFactory, Parser, Subcommand};

#[cfg(test)]
use crate::common::set_json_color_choice;
use crate::common::CliColorChoice;

#[path = "defs_mutation.rs"]
mod datasource_defs_mutation;
#[path = "defs_read.rs"]
mod datasource_defs_read;
#[path = "defs_sync.rs"]
mod datasource_defs_sync;
#[path = "formats.rs"]
mod datasource_formats;
#[path = "help_texts.rs"]
mod datasource_help_texts;

pub(crate) use self::datasource_formats::normalize_datasource_group_command;
#[cfg(test)]
use self::datasource_formats::normalize_output_formats;
use self::datasource_formats::{
    parse_bool_choice, parse_datasource_import_output_column, parse_datasource_list_output_column,
    parse_datasource_plan_output_column,
};
pub use self::datasource_formats::{
    DatasourceImportInputFormat, DatasourcePlanOutputFormat, DryRunOutputFormat, ListOutputFormat,
};
use self::datasource_help_texts::*;
pub use datasource_defs_mutation::{DatasourceAddArgs, DatasourceDeleteArgs, DatasourceModifyArgs};
pub use datasource_defs_read::{
    ArtifactRunMode, DatasourceBrowseArgs, DatasourceListArgs, DatasourceTypesArgs,
};
pub use datasource_defs_sync::{
    DatasourceDiffArgs, DatasourceExportArgs, DatasourceImportArgs, DatasourcePlanArgs,
};

#[derive(Debug, Clone, Subcommand)]
pub enum DatasourceGroupCommand {
    #[command(about = "Show the built-in supported datasource type catalog.", after_help = DATASOURCE_TYPES_HELP_TEXT)]
    Types(DatasourceTypesArgs),
    #[command(about = "List datasource inventory from live Grafana or a local export bundle.", after_help = DATASOURCE_LIST_HELP_TEXT)]
    List(DatasourceListArgs),
    #[command(
        about = "Open a live datasource browser against Grafana with in-place modify and delete actions.",
        after_help = DATASOURCE_BROWSE_HELP_TEXT
    )]
    Browse(DatasourceBrowseArgs),
    #[command(about = "Create one live Grafana datasource through the Grafana API.", after_help = DATASOURCE_ADD_HELP_TEXT)]
    Add(DatasourceAddArgs),
    #[command(about = "Modify one live Grafana datasource through the Grafana API.", after_help = DATASOURCE_MODIFY_HELP_TEXT)]
    Modify(DatasourceModifyArgs),
    #[command(about = "Delete one live Grafana datasource through the Grafana API.", after_help = DATASOURCE_DELETE_HELP_TEXT)]
    Delete(DatasourceDeleteArgs),
    #[command(about = "Export live Grafana datasource inventory as normalized JSON plus provisioning files.", after_help = DATASOURCE_EXPORT_HELP_TEXT)]
    Export(DatasourceExportArgs),
    #[command(about = "Import datasource inventory through the Grafana API.", after_help = DATASOURCE_IMPORT_HELP_TEXT)]
    Import(DatasourceImportArgs),
    #[command(about = "Compare local datasource export files against live Grafana datasources.", after_help = DATASOURCE_DIFF_HELP_TEXT)]
    Diff(DatasourceDiffArgs),
    #[command(about = "Build a review-first datasource reconcile plan against live Grafana.", after_help = DATASOURCE_PLAN_HELP_TEXT)]
    Plan(DatasourcePlanArgs),
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "grafana-util datasource",
    about = "List live or local datasource inventory, browse live, add, modify, delete, export, import, and diff Grafana datasources.",
    after_help = DATASOURCE_ROOT_HELP_TEXT,
    styles = crate::help_styles::CLI_HELP_STYLES
)]
pub(crate) struct DatasourceCliRoot {
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = CliColorChoice::Auto,
        help = "Colorize JSON output. Use auto, always, never, none, or off."
    )]
    color: CliColorChoice,
    #[command(flatten)]
    args: DatasourceCliArgs,
}

#[derive(Debug, Clone, Args)]
pub struct DatasourceCliArgs {
    #[command(subcommand)]
    pub command: DatasourceGroupCommand,
}

pub fn root_command() -> clap::Command {
    DatasourceCliRoot::command()
}

#[cfg(test)]
impl DatasourceCliArgs {
    pub(crate) fn command() -> clap::Command {
        root_command()
    }

    pub(crate) fn parse_from<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let root = DatasourceCliRoot::parse_from(iter);
        set_json_color_choice(root.color);
        root.args
    }

    pub(crate) fn try_parse_from<I, T>(iter: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let root = DatasourceCliRoot::try_parse_from(iter)?;
        set_json_color_choice(root.color);
        Ok(root.args)
    }
}

#[cfg(test)]
impl DatasourceCliArgs {
    pub(crate) fn parse_normalized_from<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let mut args = Self::parse_from(iter);
        normalize_output_formats(&mut args);
        args
    }
}
