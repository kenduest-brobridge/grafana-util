use clap::Args;
use std::path::PathBuf;

/// Arguments for initializing a staged alert desired-state layout.
#[derive(Debug, Clone, Args)]
pub struct AlertInitArgs {
    #[arg(
        long,
        help = "Directory to initialize with staged alert desired-state scaffolding."
    )]
    pub desired_dir: PathBuf,
}

/// Shared arguments for new staged alert resource scaffolds.
#[derive(Debug, Clone, Args)]
pub struct AlertNewResourceArgs {
    #[arg(
        long,
        help = "Directory containing the staged alert desired-state layout."
    )]
    pub desired_dir: PathBuf,
    #[arg(long, help = "Resource name to seed into the new scaffold.")]
    pub name: String,
}

/// Shared authoring inputs for desired-state alert resource commands.
#[derive(Debug, Clone, Args)]
pub struct AlertAuthoringBaseArgs {
    #[arg(
        long,
        help = "Directory containing the staged alert desired-state layout."
    )]
    pub desired_dir: PathBuf,
}

/// Authoring inputs for high-level add-rule.
#[derive(Debug, Clone, Args)]
pub struct AlertAddRuleArgs {
    #[command(flatten)]
    pub base: AlertAuthoringBaseArgs,
    #[arg(long, help = "Rule name to author.")]
    pub name: String,
    #[arg(long, help = "Folder that will own the authored rule.")]
    pub folder: String,
    #[arg(long = "rule-group", help = "Rule group name inside the folder.")]
    pub rule_group: String,
    #[arg(
        long,
        required_unless_present = "no_route",
        help = "Receiver name to route the authored rule to."
    )]
    pub receiver: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "receiver",
        help = "Skip route authoring for this rule."
    )]
    pub no_route: bool,
    #[arg(
        long = "label",
        help = "Rule label in key=value form. Repeat for more labels."
    )]
    pub labels: Vec<String>,
    #[arg(
        long = "annotation",
        help = "Rule annotation in key=value form. Repeat for more annotations."
    )]
    pub annotations: Vec<String>,
    #[arg(long, help = "Convenience severity label value for the authored rule.")]
    pub severity: Option<String>,
    #[arg(long = "for", help = "Pending duration before the rule starts firing.")]
    pub for_duration: Option<String>,
    #[arg(long, help = "Simple-rule expression reference or expression text.")]
    pub expr: Option<String>,
    #[arg(long, help = "Simple-rule threshold value.")]
    pub threshold: Option<f64>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "below",
        help = "Fire when the evaluated value is above the threshold."
    )]
    pub above: bool,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "above",
        help = "Fire when the evaluated value is below the threshold."
    )]
    pub below: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the planned authored rule output without writing files."
    )]
    pub dry_run: bool,
}

/// Authoring inputs for rule cloning.
#[derive(Debug, Clone, Args)]
pub struct AlertCloneRuleArgs {
    #[command(flatten)]
    pub base: AlertAuthoringBaseArgs,
    #[arg(
        long,
        help = "Existing rule identity to clone from the desired-state tree."
    )]
    pub source: String,
    #[arg(long, help = "New rule name for the cloned rule.")]
    pub name: String,
    #[arg(long, help = "Optional replacement folder for the cloned rule.")]
    pub folder: Option<String>,
    #[arg(
        long = "rule-group",
        help = "Optional replacement rule group for the cloned rule."
    )]
    pub rule_group: Option<String>,
    #[arg(
        long,
        help = "Optional replacement receiver for the cloned rule route."
    )]
    pub receiver: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        conflicts_with = "receiver",
        help = "Clear route authoring while cloning."
    )]
    pub no_route: bool,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the planned cloned rule output without writing files."
    )]
    pub dry_run: bool,
}

/// Authoring inputs for contact point creation.
#[derive(Debug, Clone, Args)]
pub struct AlertAddContactPointArgs {
    #[command(flatten)]
    pub base: AlertAuthoringBaseArgs,
    #[arg(long, help = "Contact point name to author.")]
    pub name: String,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the planned authored contact point output without writing files."
    )]
    pub dry_run: bool,
}

/// Authoring inputs for notification route updates.
#[derive(Debug, Clone, Args)]
pub struct AlertSetRouteArgs {
    #[command(flatten)]
    pub base: AlertAuthoringBaseArgs,
    #[arg(long, help = "Receiver name for the route.")]
    pub receiver: String,
    #[arg(
        long = "label",
        help = "Route matcher in key=value form. Repeat for more matchers."
    )]
    pub labels: Vec<String>,
    #[arg(long, help = "Convenience severity matcher value for the route.")]
    pub severity: Option<String>,
    #[arg(
        long,
        default_value_t = false,
        help = "Render the managed route document that would replace the tool-owned route without writing files."
    )]
    pub dry_run: bool,
}

/// Authoring inputs for route previews.
#[derive(Debug, Clone, Args)]
pub struct AlertPreviewRouteArgs {
    #[command(flatten)]
    pub base: AlertAuthoringBaseArgs,
    #[arg(
        long = "label",
        help = "Preview label in key=value form. Repeat for more labels."
    )]
    pub labels: Vec<String>,
    #[arg(long, help = "Convenience severity label value for route preview.")]
    pub severity: Option<String>,
}
