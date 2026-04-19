#[path = "snapshot_access.rs"]
mod snapshot_access;
#[path = "snapshot_artifacts.rs"]
mod snapshot_artifacts;
#[path = "snapshot_export.rs"]
mod snapshot_export;
#[path = "snapshot_metadata.rs"]
mod snapshot_metadata;
#[path = "snapshot_review.rs"]
mod snapshot_review;

use clap::CommandFactory;

pub(crate) use snapshot_access::build_snapshot_access_lane_summaries;
#[cfg(test)]
pub(crate) use snapshot_export::run_snapshot_export_with_handlers;
#[cfg(test)]
pub(crate) use snapshot_export::{
    materialize_snapshot_common_auth_with_prompt, run_snapshot_export_selected_with_handlers,
};
#[cfg(test)]
pub(crate) use snapshot_metadata::build_snapshot_root_metadata;
pub(crate) use snapshot_metadata::export_scope_kind_from_metadata_value;
#[cfg(test)]
pub(crate) use snapshot_review::run_snapshot_review_document_with_handler;

#[cfg(test)]
pub use snapshot_artifacts::build_snapshot_paths;
pub use snapshot_export::run_snapshot_export;
#[cfg(test)]
pub use snapshot_review::build_snapshot_overview_args;
pub use snapshot_review::run_snapshot_review;

pub fn root_command() -> clap::Command {
    super::SnapshotCliArgs::command()
}

pub fn run_snapshot_cli(command: super::SnapshotCommand) -> crate::common::Result<()> {
    // Snapshot namespace boundary keeps only two concrete commands and delegates each to
    // its dedicated orchestration path.
    match command {
        super::SnapshotCommand::Export(args) => run_snapshot_export(args),
        super::SnapshotCommand::Review(args) => run_snapshot_review(args),
    }
}

#[cfg(test)]
mod tests {
    use super::materialize_snapshot_common_auth_with_prompt;
    use crate::dashboard::CommonCliArgs;

    fn sample_common_args() -> CommonCliArgs {
        CommonCliArgs {
            color: crate::common::CliColorChoice::Auto,
            profile: Some("prod".to_string()),
            url: "http://grafana.example.com".to_string(),
            api_token: None,
            username: Some("admin".to_string()),
            password: None,
            prompt_password: true,
            prompt_token: false,
            timeout: 30,
            verify_ssl: false,
        }
    }

    #[test]
    fn materialize_snapshot_common_auth_prompts_password_once_and_clears_prompt_flags() {
        let mut password_prompts = 0usize;
        let mut token_prompts = 0usize;
        let mut prompt_password = || {
            password_prompts += 1;
            Ok("prompted-password".to_string())
        };
        let mut prompt_token = || {
            token_prompts += 1;
            Ok("prompted-token".to_string())
        };

        let common = materialize_snapshot_common_auth_with_prompt(
            sample_common_args(),
            &mut prompt_password,
            &mut prompt_token,
        )
        .unwrap();

        assert_eq!(common.password.as_deref(), Some("prompted-password"));
        assert_eq!(common.api_token, None);
        assert!(!common.prompt_password);
        assert!(!common.prompt_token);
        assert_eq!(password_prompts, 1);
        assert_eq!(token_prompts, 0);
    }
}
