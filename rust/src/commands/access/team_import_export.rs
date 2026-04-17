//! Team export/import workflow helpers.
//!
//! Maintainer notes:
//! - Team import is a two-phase sync: ensure the team and member identities
//!   exist first, then apply the admin/member role split with the bulk update.
//! - Membership removals are destructive and must remain behind `--yes`.

use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::common::{message, render_json_value, string_field, write_json_file, Result};

use crate::access::render::{
    access_export_summary_line, access_import_summary_line, format_table, map_get_text,
    normalize_team_row, scalar_text, value_bool,
};
use crate::access::team_import_export_diff::{
    assert_not_overwrite, build_team_access_export_metadata, build_team_import_dry_run_document,
    build_team_import_dry_run_row, build_team_import_dry_run_rows, load_team_import_records,
    parse_access_identity_list, validate_team_import_dry_run_output,
};
use crate::access::team_runtime::{
    add_team_member_with_request, create_team_with_request, iter_teams_with_request,
    list_team_members_with_request, lookup_team_by_name, normalize_access_identity,
    remove_team_member_with_request, team_member_identity, team_member_is_admin,
    update_team_members_with_request, user_id_from_record,
};
use crate::access::user::lookup_org_user_by_identity;
use crate::access::{
    TeamExportArgs, TeamImportArgs, ACCESS_EXPORT_KIND_TEAMS, ACCESS_EXPORT_METADATA_FILENAME,
    ACCESS_EXPORT_VERSION, ACCESS_TEAM_EXPORT_FILENAME,
};

struct ResolvedTeamMember {
    identity: String,
    email: String,
    user_id: String,
}

struct ResolvedTeamMemberships {
    members: Vec<String>,
    admins: Vec<String>,
    merged: Vec<ResolvedTeamMember>,
    target_keys: BTreeSet<String>,
}

fn mark_team_import_row_blocked(row: &mut Map<String, Value>, blocker: &str) {
    row.insert("status".to_string(), Value::String("blocked".to_string()));
    row.insert("blocked".to_string(), Value::Bool(true));
    row.insert(
        "blockers".to_string(),
        Value::Array(vec![Value::String(blocker.to_string())]),
    );
}

fn attach_team_target(row: &mut Map<String, Value>, team: &Map<String, Value>) {
    let mut target = Map::new();
    for (source, dest) in [
        ("id", "targetId"),
        ("teamId", "targetId"),
        ("uid", "targetUid"),
        ("memberCount", "memberCount"),
    ] {
        let value = scalar_text(team.get(source));
        if !value.is_empty() && !target.contains_key(dest) {
            target.insert(dest.to_string(), Value::String(value));
        }
    }
    if let Some(is_provisioned) = value_bool(team.get("isProvisioned")) {
        target.insert("isProvisioned".to_string(), Value::Bool(is_provisioned));
    }
    if !target.is_empty() {
        row.insert("target".to_string(), Value::Object(target));
    }
}

fn resolve_team_memberships_for_bulk<F>(
    request_json: &mut F,
    record_members: &[String],
    record_admins: &[String],
) -> Result<ResolvedTeamMemberships>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut seen_email_keys = BTreeSet::new();
    let mut members = Vec::new();
    let mut admins = Vec::new();
    let mut merged = Vec::new();

    for (identity, admin) in record_admins
        .iter()
        .map(|identity| (identity, true))
        .chain(record_members.iter().map(|identity| (identity, false)))
    {
        let user = lookup_org_user_by_identity(&mut *request_json, identity)?;
        let email = string_field(&user, "email", "");
        if email.is_empty() {
            return Err(message(format!(
                "Team member identity {identity} resolved without an email; Grafana bulk team membership updates require user emails."
            )));
        }
        let user_id = user_id_from_record(&user);
        if user_id.is_empty() {
            return Err(message(format!(
                "Team member lookup did not return an id: {}",
                identity
            )));
        }
        let email_key = normalize_access_identity(&email);
        if !seen_email_keys.insert(email_key) {
            continue;
        }
        if admin {
            admins.push(email.clone());
        } else {
            members.push(email.clone());
        }
        merged.push(ResolvedTeamMember {
            identity: identity.clone(),
            email,
            user_id,
        });
    }

    let target_keys = merged
        .iter()
        .map(|member| normalize_access_identity(&member.email))
        .collect::<BTreeSet<String>>();

    Ok(ResolvedTeamMemberships {
        members,
        admins,
        merged,
        target_keys,
    })
}

pub(crate) fn export_teams_with_request<F>(
    mut request_json: F,
    args: &TeamExportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let teams = iter_teams_with_request(&mut request_json, None)?;
    let mut records = teams
        .into_iter()
        .map(|team| normalize_team_row(&team))
        .collect::<Vec<Map<String, Value>>>();
    records.sort_by(|left, right| {
        let lhs = map_get_text(left, "name");
        let rhs = map_get_text(right, "name");
        lhs.cmp(&rhs)
            .then_with(|| map_get_text(left, "id").cmp(&map_get_text(right, "id")))
    });

    if args.with_members {
        for row in &mut records {
            let team_id = map_get_text(row, "id");
            let mut members: Vec<String> = Vec::new();
            let mut admins: Vec<String> = Vec::new();
            for member in list_team_members_with_request(&mut request_json, &team_id)? {
                let identity = team_member_identity(&member);
                if identity.is_empty() {
                    continue;
                }
                if team_member_is_admin(&member) {
                    admins.push(identity.clone());
                }
                members.push(identity);
            }
            members.sort();
            members.dedup();
            admins.sort();
            admins.dedup();
            // Export admins separately even though they are also members so
            // import can rebuild both presence and admin-state deterministically.
            row.insert(
                "members".to_string(),
                Value::Array(members.iter().cloned().map(Value::String).collect()),
            );
            row.insert(
                "admins".to_string(),
                Value::Array(admins.iter().cloned().map(Value::String).collect()),
            );
        }
    }

    let teams_path = args.output_dir.join(ACCESS_TEAM_EXPORT_FILENAME);
    let metadata_path = args.output_dir.join(ACCESS_EXPORT_METADATA_FILENAME);
    assert_not_overwrite(&teams_path, args.dry_run, args.overwrite)?;
    assert_not_overwrite(&metadata_path, args.dry_run, args.overwrite)?;

    if !args.dry_run {
        let payload = Value::Object(Map::from_iter(vec![
            (
                "kind".to_string(),
                Value::String(ACCESS_EXPORT_KIND_TEAMS.to_string()),
            ),
            (
                "version".to_string(),
                Value::Number((ACCESS_EXPORT_VERSION).into()),
            ),
            (
                "records".to_string(),
                Value::Array(records.iter().cloned().map(Value::Object).collect()),
            ),
        ]));
        write_json_file(&teams_path, &payload, args.overwrite)?;
        write_json_file(
            &metadata_path,
            &Value::Object(build_team_access_export_metadata(
                &args.common.url,
                args.common.profile.as_deref(),
                &args.output_dir,
                records.len(),
                args.with_members,
            )),
            args.overwrite,
        )?;
    }

    println!(
        "{}",
        access_export_summary_line(
            "team",
            records.len(),
            &args.common.url,
            &teams_path.to_string_lossy(),
            &metadata_path.to_string_lossy(),
            args.dry_run,
        )
    );

    Ok(records.len())
}

pub(crate) fn import_teams_with_request<F>(
    mut request_json: F,
    args: &TeamImportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_team_import_dry_run_output(args)?;
    let records = load_team_import_records(&args.input_dir, ACCESS_EXPORT_KIND_TEAMS)?;
    let mut created = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut processed = 0usize;
    let mut dry_run_rows: Vec<Map<String, Value>> = Vec::new();
    let is_dry_run_table_or_json = args.dry_run && (args.table || args.json);

    for (index, record) in records.iter().enumerate() {
        processed += 1;
        let team_name = string_field(record, "name", "");
        if team_name.is_empty() {
            return Err(message(format!(
                "Access team import record {} in {} is missing name.",
                index + 1,
                args.input_dir.display()
            )));
        }

        let record_members =
            parse_access_identity_list(record.get("members").unwrap_or(&Value::Null));
        let record_admins =
            parse_access_identity_list(record.get("admins").unwrap_or(&Value::Null));
        let has_membership_payload = !(record_members.is_empty() && record_admins.is_empty());
        let resolved_memberships = if has_membership_payload {
            match resolve_team_memberships_for_bulk(
                &mut request_json,
                &record_members,
                &record_admins,
            ) {
                Ok(resolved) => Some(resolved),
                Err(error) if args.dry_run => {
                    let detail = error.to_string();
                    if is_dry_run_table_or_json {
                        let mut row = build_team_import_dry_run_row(
                            index + 1,
                            &team_name,
                            "blocked",
                            &detail,
                        );
                        mark_team_import_row_blocked(&mut row, &detail);
                        dry_run_rows.push(row);
                    } else {
                        println!("Blocked team {} import: {}", team_name, detail);
                    }
                    updated += 1;
                    continue;
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        };
        let target_keys = resolved_memberships
            .as_ref()
            .map(|resolved| resolved.target_keys.clone())
            .unwrap_or_default();

        let existing = lookup_team_by_name(&mut request_json, &team_name).ok();

        let existing_team_id = existing.as_ref().and_then(|team| {
            let team_id = scalar_text(team.get("teamId"));
            if team_id.is_empty() {
                let id = scalar_text(team.get("id"));
                if id.is_empty() {
                    None
                } else {
                    Some(id)
                }
            } else {
                Some(team_id)
            }
        });

        if existing_team_id.is_none() {
            if args.dry_run {
                if is_dry_run_table_or_json {
                    let mut row = build_team_import_dry_run_row(
                        index + 1,
                        &team_name,
                        "create",
                        "would create team",
                    );
                    if let Some(resolved) = resolved_memberships.as_ref() {
                        row.insert(
                            "desired".to_string(),
                            Value::Object(Map::from_iter(vec![
                                (
                                    "members".to_string(),
                                    Value::Array(
                                        resolved
                                            .members
                                            .iter()
                                            .cloned()
                                            .map(Value::String)
                                            .collect(),
                                    ),
                                ),
                                (
                                    "admins".to_string(),
                                    Value::Array(
                                        resolved
                                            .admins
                                            .iter()
                                            .cloned()
                                            .map(Value::String)
                                            .collect(),
                                    ),
                                ),
                            ])),
                        );
                    }
                    dry_run_rows.push(row);
                } else {
                    println!("Would create team {}", team_name);
                }
                created += 1;
                continue;
            }

            let created_team = create_team_with_request(
                &mut request_json,
                &Value::Object(Map::from_iter([
                    ("name".to_string(), Value::String(team_name.clone())),
                    (
                        "email".to_string(),
                        Value::String(string_field(record, "email", "")),
                    ),
                ])),
            )?;
            let team_id = {
                let team_id = scalar_text(created_team.get("teamId"));
                if team_id.is_empty() {
                    scalar_text(created_team.get("id"))
                } else {
                    team_id
                }
            };
            if team_id.is_empty() {
                return Err(message(format!(
                    "Team import did not return team id for {}",
                    team_name
                )));
            }

            if let Some(resolved) = resolved_memberships {
                // Add every referenced user first so the later bulk membership
                // update only has to flip admin-state, not create missing edges.
                for member in resolved.merged.iter() {
                    add_team_member_with_request(&mut request_json, &team_id, &member.user_id)?;
                }

                if !resolved.members.is_empty() || !resolved.admins.is_empty() {
                    let _ = update_team_members_with_request(
                        &mut request_json,
                        &team_id,
                        resolved.members,
                        resolved.admins,
                    )?;
                }
            }

            println!("Created team {}", team_name);
            created += 1;
            continue;
        }

        let team_id = existing_team_id.unwrap();
        if !args.replace_existing {
            skipped += 1;
            if is_dry_run_table_or_json {
                let mut row = build_team_import_dry_run_row(
                    index + 1,
                    &team_name,
                    "skip",
                    "existing and --replace-existing was not set.",
                );
                if let Some(team) = existing.as_ref() {
                    attach_team_target(&mut row, team);
                }
                row.insert("status".to_string(), Value::String("skipped".to_string()));
                dry_run_rows.push(row);
            } else {
                println!("Skipped team {} ({})", team_name, index + 1);
            }
            continue;
        }

        let existing_is_provisioned = existing
            .as_ref()
            .and_then(|team| value_bool(team.get("isProvisioned")))
            .unwrap_or(false);

        let mut existing_members = BTreeMap::<String, (String, bool, String)>::new();
        for member in list_team_members_with_request(&mut request_json, &team_id)? {
            let identity = team_member_identity(&member);
            if identity.is_empty() {
                continue;
            }
            existing_members.insert(
                normalize_access_identity(&identity),
                (
                    identity,
                    team_member_is_admin(&member),
                    scalar_text(member.get("userId")),
                ),
            );
        }

        let remove_keys: Vec<String> = existing_members
            .keys()
            .filter(|identity| !target_keys.contains(*identity))
            .cloned()
            .collect();
        if !remove_keys.is_empty() && !args.yes {
            return Err(message(format!(
                "Team import would remove team memberships for {}. Add --yes to confirm.",
                team_name
            )));
        }

        if existing_is_provisioned && (has_membership_payload || !remove_keys.is_empty()) {
            let detail = "provisioned team memberships cannot be changed";
            if args.dry_run {
                if is_dry_run_table_or_json {
                    let mut row =
                        build_team_import_dry_run_row(index + 1, &team_name, "blocked", detail);
                    mark_team_import_row_blocked(&mut row, detail);
                    if let Some(team) = existing.as_ref() {
                        attach_team_target(&mut row, team);
                    }
                    dry_run_rows.push(row);
                } else {
                    println!("Blocked team {} import: {}", team_name, detail);
                }
                updated += 1;
                continue;
            }
            return Err(message(format!(
                "Team import blocked for {team_name}: {detail}"
            )));
        }

        if !args.dry_run {
            // For existing teams, converge to the exported membership set before
            // sending the admin/member payload split that finalizes role state.
            if let Some(resolved) = resolved_memberships.as_ref() {
                for member in resolved.merged.iter() {
                    let key = normalize_access_identity(&member.email);
                    if existing_members.contains_key(&key) {
                        continue;
                    }
                    add_team_member_with_request(&mut request_json, &team_id, &member.user_id)?;
                    existing_members
                        .insert(key, (member.email.clone(), false, member.user_id.clone()));
                }
            }

            if !remove_keys.is_empty() {
                for key in remove_keys.iter() {
                    if let Some((_, _, user_id)) = existing_members.remove(key.as_str()) {
                        if !user_id.is_empty() {
                            remove_team_member_with_request(&mut request_json, &team_id, &user_id)?;
                        }
                    }
                }
            }

            let _ = update_team_members_with_request(
                &mut request_json,
                &team_id,
                resolved_memberships
                    .as_ref()
                    .map(|resolved| resolved.members.clone())
                    .unwrap_or_default(),
                resolved_memberships
                    .as_ref()
                    .map(|resolved| resolved.admins.clone())
                    .unwrap_or_default(),
            )?;
        }

        if args.dry_run {
            if let Some(resolved) = resolved_memberships.as_ref() {
                for member in resolved.merged.iter() {
                    let key = normalize_access_identity(&member.email);
                    if !existing_members.contains_key(&key) {
                        if is_dry_run_table_or_json {
                            let mut row = build_team_import_dry_run_row(
                                index + 1,
                                &team_name,
                                "add-member",
                                &format!(
                                    "would add team member {} as {}",
                                    member.identity, member.email
                                ),
                            );
                            if let Some(team) = existing.as_ref() {
                                attach_team_target(&mut row, team);
                            }
                            dry_run_rows.push(row);
                        } else {
                            println!("Would add team {} member {}", team_name, member.email);
                        }
                    }
                }
            }
            for key in remove_keys.iter() {
                if let Some((identity, _, _)) = existing_members.get(key.as_str()) {
                    if is_dry_run_table_or_json {
                        let mut row = build_team_import_dry_run_row(
                            index + 1,
                            &team_name,
                            "remove-member",
                            &format!("would remove team member {identity}"),
                        );
                        if let Some(team) = existing.as_ref() {
                            attach_team_target(&mut row, team);
                        }
                        dry_run_rows.push(row);
                    } else {
                        println!("Would remove team {} member {}", team_name, identity);
                    }
                }
            }
        }

        updated += 1;
        if is_dry_run_table_or_json {
            let mut row = build_team_import_dry_run_row(
                index + 1,
                &team_name,
                "updated",
                "would update team",
            );
            if let Some(team) = existing.as_ref() {
                attach_team_target(&mut row, team);
            }
            dry_run_rows.push(row);
        } else {
            println!("Updated team {}", team_name);
        }
    }

    if args.dry_run && is_dry_run_table_or_json {
        if args.json {
            print!(
                "{}",
                render_json_value(&build_team_import_dry_run_document(
                    &dry_run_rows,
                    processed,
                    created,
                    updated,
                    skipped,
                    &args.input_dir,
                ))?
            );
            return Ok(0);
        }
        if args.table {
            for line in format_table(
                &["INDEX", "IDENTITY", "ACTION", "STATUS", "DETAIL"],
                &build_team_import_dry_run_rows(&dry_run_rows),
            ) {
                println!("{line}");
            }
        }
    }

    println!(
        "{}",
        access_import_summary_line(
            "team",
            processed,
            created,
            updated,
            skipped,
            &args.input_dir.to_string_lossy(),
        )
    );
    Ok(processed)
}
