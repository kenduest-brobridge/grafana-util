use std::path::Path;

use serde_json::{json, Value};

use crate::access::{
    self, AccessCliArgs, AccessCommand, CommonCliArgs as AccessCommonCliArgs,
    CommonCliArgsNoOrgId as AccessCommonCliArgsNoOrgId, OrgCommand, OrgExportArgs, Scope,
    ServiceAccountCommand, ServiceAccountExportArgs, TeamCommand, TeamExportArgs, UserCommand,
    UserExportArgs,
};
use crate::common::Result;
use crate::dashboard::CommonCliArgs;

use super::snapshot_artifacts::build_snapshot_paths;
use super::snapshot_metadata::load_snapshot_lane_metadata_summary;

#[derive(Debug, Clone, Default)]
pub(crate) struct SnapshotAccessReviewCounts {
    pub(crate) user_count: usize,
    pub(crate) team_count: usize,
    pub(crate) org_count: usize,
    pub(crate) service_account_count: usize,
}

fn access_common_from_snapshot(common: &CommonCliArgs) -> AccessCommonCliArgs {
    AccessCommonCliArgs {
        profile: common.profile.clone(),
        url: common.url.clone(),
        api_token: common.api_token.clone(),
        username: common.username.clone(),
        password: common.password.clone(),
        prompt_password: common.prompt_password,
        prompt_token: common.prompt_token,
        org_id: None,
        timeout: common.timeout,
        verify_ssl: common.verify_ssl,
        insecure: false,
        ca_cert: None,
    }
}

fn access_common_no_org_id_from_snapshot(common: &CommonCliArgs) -> AccessCommonCliArgsNoOrgId {
    AccessCommonCliArgsNoOrgId {
        profile: common.profile.clone(),
        url: common.url.clone(),
        api_token: common.api_token.clone(),
        username: common.username.clone(),
        password: common.password.clone(),
        prompt_password: common.prompt_password,
        prompt_token: common.prompt_token,
        timeout: common.timeout,
        verify_ssl: common.verify_ssl,
        insecure: false,
        ca_cert: None,
    }
}

fn build_snapshot_access_user_export_args(
    args: &super::super::SnapshotExportArgs,
) -> UserExportArgs {
    UserExportArgs {
        common: access_common_from_snapshot(&args.common),
        output_dir: build_snapshot_paths(&args.output_dir)
            .access
            .join(super::super::SNAPSHOT_ACCESS_USERS_DIR),
        overwrite: args.overwrite,
        dry_run: false,
        scope: Scope::Org,
        with_teams: true,
        run: None,
        run_id: None,
    }
}

fn build_snapshot_access_team_export_args(
    args: &super::super::SnapshotExportArgs,
) -> TeamExportArgs {
    TeamExportArgs {
        common: access_common_from_snapshot(&args.common),
        output_dir: build_snapshot_paths(&args.output_dir)
            .access
            .join(super::super::SNAPSHOT_ACCESS_TEAMS_DIR),
        overwrite: args.overwrite,
        dry_run: false,
        with_members: true,
        run: None,
        run_id: None,
    }
}

fn build_snapshot_access_org_export_args(args: &super::super::SnapshotExportArgs) -> OrgExportArgs {
    OrgExportArgs {
        common: access_common_no_org_id_from_snapshot(&args.common),
        org_id: None,
        output_dir: build_snapshot_paths(&args.output_dir)
            .access
            .join(super::super::SNAPSHOT_ACCESS_ORGS_DIR),
        overwrite: args.overwrite,
        dry_run: false,
        name: None,
        with_users: true,
        run: None,
        run_id: None,
    }
}

fn build_snapshot_access_service_account_export_args(
    args: &super::super::SnapshotExportArgs,
) -> ServiceAccountExportArgs {
    ServiceAccountExportArgs {
        common: access_common_from_snapshot(&args.common),
        output_dir: build_snapshot_paths(&args.output_dir)
            .access
            .join(super::super::SNAPSHOT_ACCESS_SERVICE_ACCOUNTS_DIR),
        overwrite: args.overwrite,
        dry_run: false,
        run: None,
        run_id: None,
    }
}

pub(crate) fn build_snapshot_access_lane_summaries(
    output_dir: &Path,
) -> Result<(Value, SnapshotAccessReviewCounts, Vec<Value>)> {
    let access_root = output_dir.join(super::super::SNAPSHOT_ACCESS_DIR);
    if !access_root.exists() {
        return Ok((
            json!({
                "present": false
            }),
            SnapshotAccessReviewCounts::default(),
            Vec::new(),
        ));
    }

    let users = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_USERS_DIR),
        "users.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_USERS,
        "users",
    )?;
    let teams = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_TEAMS_DIR),
        "teams.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_TEAMS,
        "teams",
    )?;
    let orgs = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_ORGS_DIR),
        "orgs.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_ORGS,
        "orgs",
    )?;
    let service_accounts = load_snapshot_lane_metadata_summary(
        &access_root.join(super::super::SNAPSHOT_ACCESS_SERVICE_ACCOUNTS_DIR),
        "service-accounts.json",
        &["recordCount"],
        access::ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS,
        "service-accounts",
    )?;

    let mut warnings = Vec::new();
    for (code, lane, label, payload_name) in [
        ("access-users-lane-missing", &users, "users", "users.json"),
        ("access-teams-lane-missing", &teams, "teams", "teams.json"),
        ("access-orgs-lane-missing", &orgs, "orgs", "orgs.json"),
        (
            "access-service-accounts-lane-missing",
            &service_accounts,
            "service accounts",
            "service-accounts.json",
        ),
    ] {
        let present = lane
            .get("present")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !present {
            warnings.push(json!({
                "code": code,
                "message": format!("At least one access export scope is missing {}.", payload_name)
            }));
            continue;
        }
        let metadata_present = lane
            .get("metadataPresent")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let payload_present = lane
            .get("payloadPresent")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !metadata_present || !payload_present {
            warnings.push(json!({
                "code": format!("{}-partial", code),
                "message": format!(
                    "Access lane {} is incomplete (metadata={}, payload={}).",
                    label,
                    metadata_present,
                    payload_present
                )
            }));
        }
    }

    let counts = SnapshotAccessReviewCounts {
        user_count: users
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize,
        team_count: teams
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize,
        org_count: orgs.get("recordCount").and_then(Value::as_u64).unwrap_or(0) as usize,
        service_account_count: service_accounts
            .get("recordCount")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize,
    };

    Ok((
        json!({
            "present": true,
            "users": users,
            "teams": teams,
            "orgs": orgs,
            "serviceAccounts": service_accounts,
        }),
        counts,
        warnings,
    ))
}

// Build and execute Access CLI commands for each selected Access snapshot lane.
pub(super) fn run_snapshot_access_exports_with_handler<FA>(
    args: &super::super::SnapshotExportArgs,
    selection: &super::super::SnapshotExportSelection,
    mut run_access: FA,
) -> Result<()>
where
    FA: FnMut(AccessCliArgs) -> Result<()>,
{
    if selection.contains(super::super::SnapshotExportLane::AccessUsers) {
        run_access(AccessCliArgs {
            command: AccessCommand::User {
                command: UserCommand::Export(build_snapshot_access_user_export_args(args)),
            },
        })?;
    }
    if selection.contains(super::super::SnapshotExportLane::AccessTeams) {
        run_access(AccessCliArgs {
            command: AccessCommand::Team {
                command: TeamCommand::Export(build_snapshot_access_team_export_args(args)),
            },
        })?;
    }
    if selection.contains(super::super::SnapshotExportLane::AccessOrgs) {
        run_access(AccessCliArgs {
            command: AccessCommand::Org {
                command: OrgCommand::Export(build_snapshot_access_org_export_args(args)),
            },
        })?;
    }
    if selection.contains(super::super::SnapshotExportLane::AccessServiceAccounts) {
        run_access(AccessCliArgs {
            command: AccessCommand::ServiceAccount {
                command: ServiceAccountCommand::Export(
                    build_snapshot_access_service_account_export_args(args),
                ),
            },
        })?;
    }
    Ok(())
}
