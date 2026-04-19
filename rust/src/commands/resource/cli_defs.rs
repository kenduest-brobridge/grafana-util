use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

use super::resource_help::{
    RESOURCE_DESCRIBE_AFTER_HELP, RESOURCE_GET_AFTER_HELP, RESOURCE_KINDS_AFTER_HELP,
    RESOURCE_LIST_AFTER_HELP,
};
use crate::common::CliColorChoice;
use crate::dashboard::CommonCliArgs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ResourceOutputFormat {
    Text,
    Table,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
pub enum ResourceKind {
    Dashboards,
    Folders,
    Datasources,
    #[value(name = "alert-rules")]
    AlertRules,
    Orgs,
}

impl ResourceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Dashboards => "dashboards",
            Self::Folders => "folders",
            Self::Datasources => "datasources",
            Self::AlertRules => "alert-rules",
            Self::Orgs => "orgs",
        }
    }

    pub(crate) fn singular_label(self) -> &'static str {
        match self {
            Self::Dashboards => "dashboard",
            Self::Folders => "folder",
            Self::Datasources => "datasource",
            Self::AlertRules => "alert-rule",
            Self::Orgs => "org",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Dashboards => "Grafana dashboards from /api/search and /api/dashboards/uid/{uid}.",
            Self::Folders => "Grafana folders from /api/folders and /api/folders/{uid}.",
            Self::Datasources => "Grafana datasources from /api/datasources and /api/datasources/uid/{uid}.",
            Self::AlertRules => {
                "Grafana alert rules from /api/v1/provisioning/alert-rules and /api/v1/provisioning/alert-rules/{uid}."
            }
            Self::Orgs => "Grafana org inventory from /api/orgs and /api/orgs/{id}.",
        }
    }

    pub(crate) fn selector_pattern(self) -> &'static str {
        match self {
            Self::Dashboards => "dashboards/<uid>",
            Self::Folders => "folders/<uid>",
            Self::Datasources => "datasources/<uid>",
            Self::AlertRules => "alert-rules/<uid>",
            Self::Orgs => "orgs/<id>",
        }
    }

    pub(crate) fn list_endpoint(self) -> &'static str {
        match self {
            Self::Dashboards => "GET /api/search",
            Self::Folders => "GET /api/folders",
            Self::Datasources => "GET /api/datasources",
            Self::AlertRules => "GET /api/v1/provisioning/alert-rules",
            Self::Orgs => "GET /api/orgs",
        }
    }

    pub(crate) fn get_endpoint(self) -> &'static str {
        match self {
            Self::Dashboards => "GET /api/dashboards/uid/{uid}",
            Self::Folders => "GET /api/folders/{uid}",
            Self::Datasources => "GET /api/datasources/uid/{uid}",
            Self::AlertRules => "GET /api/v1/provisioning/alert-rules/{uid}",
            Self::Orgs => "GET /api/orgs/{id}",
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct ResourceKindsArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Table,
        help = "Render supported resource kinds as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct ResourceDescribeArgs {
    #[arg(
        value_enum,
        help = "Optional resource kind to describe. Omit this to describe every supported kind."
    )]
    pub kind: Option<ResourceKind>,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Table,
        help = "Render resource descriptions as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct ResourceListArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        value_enum,
        help = "Grafana resource kind to list. Use grafana-util resource describe to see the current selector patterns and endpoints."
    )]
    pub kind: ResourceKind,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Table,
        help = "Render live resource inventory as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
}

#[derive(Debug, Clone, Args)]
pub struct ResourceGetArgs {
    #[command(flatten)]
    pub common: CommonCliArgs,
    #[arg(
        value_name = "SELECTOR",
        help = "Fetch one live resource by selector. Use <kind>/<identity>, for example dashboards/cpu-main or folders/infra. Run grafana-util resource describe first if you need the supported selector patterns."
    )]
    pub selector: String,
    #[arg(
        long,
        value_enum,
        default_value_t = ResourceOutputFormat::Json,
        help = "Render the fetched live resource as text, table, json, or yaml."
    )]
    pub output_format: ResourceOutputFormat,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ResourceCommand {
    #[command(
        name = "kinds",
        about = "List the resource kinds supported by the generic read-only resource query surface.",
        after_help = RESOURCE_KINDS_AFTER_HELP
    )]
    Kinds(ResourceKindsArgs),
    #[command(
        name = "describe",
        about = "Describe the supported live Grafana resource kinds and selector patterns.",
        after_help = RESOURCE_DESCRIBE_AFTER_HELP
    )]
    Describe(ResourceDescribeArgs),
    #[command(
        name = "list",
        about = "List one supported live Grafana resource kind.",
        after_help = RESOURCE_LIST_AFTER_HELP
    )]
    List(ResourceListArgs),
    #[command(
        name = "get",
        about = "Fetch one supported live Grafana resource by selector.",
        after_help = RESOURCE_GET_AFTER_HELP
    )]
    Get(ResourceGetArgs),
}

#[derive(Debug, Clone, Parser)]
pub struct ResourceCliArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = CliColorChoice::Auto,
        help = "Colorize JSON and YAML output. Use auto, always, or never."
    )]
    pub color: CliColorChoice,
    #[command(subcommand)]
    pub command: ResourceCommand,
}
