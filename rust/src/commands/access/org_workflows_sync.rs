use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, string_field, value_as_object, write_json_file, Result};

use super::super::super::render::{
    access_export_summary_line, access_import_summary_line, scalar_text, value_bool,
};
use super::super::super::{
    OrgExportArgs, OrgImportArgs, ACCESS_EXPORT_KIND_ORGS, ACCESS_EXPORT_METADATA_FILENAME,
    ACCESS_EXPORT_VERSION, ACCESS_ORG_EXPORT_FILENAME,
};
use super::super::org_import_export_diff::{
    assert_not_overwrite, build_org_export_metadata, load_org_import_records,
};
use super::super::{
    add_user_to_org_with_request, create_organization_with_request, list_org_users_with_request,
    list_organizations_with_request, normalize_org_row, normalize_org_user_row,
    update_org_user_role_with_request, update_organization_with_request, validate_basic_auth_only,
};

fn org_user_role_change_blocker(user: &Map<String, Value>) -> Option<String> {
    if value_bool(user.get("isExternallySynced")).unwrap_or(false) {
        Some(
            "externally synced user orgRole cannot be updated through Grafana org user API"
                .to_string(),
        )
    } else {
        None
    }
}

fn org_user_role_target_evidence(user: &Map<String, Value>) -> String {
    let mut details = Vec::new();
    for (label, value) in [
        ("userId", scalar_text(user.get("userId"))),
        ("id", scalar_text(user.get("id"))),
        ("login", string_field(user, "login", "")),
        ("email", string_field(user, "email", "")),
        ("currentRole", string_field(user, "role", "")),
    ] {
        if !value.is_empty() {
            details.push(format!("{label}={value}"));
        }
    }
    if let Some(is_externally_synced) = value_bool(user.get("isExternallySynced")) {
        details.push(format!("isExternallySynced={is_externally_synced}"));
    }
    if let Some(is_provisioned) = value_bool(user.get("isProvisioned")) {
        details.push(format!("isProvisioned={is_provisioned}"));
    }
    details.join(" ")
}

pub(crate) fn export_orgs_with_request<F>(
    mut request_json: F,
    args: &OrgExportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_basic_auth_only(&args.common)?;
    let payload_path = args.output_dir.join(ACCESS_ORG_EXPORT_FILENAME);
    let metadata_path = args.output_dir.join(ACCESS_EXPORT_METADATA_FILENAME);
    assert_not_overwrite(&payload_path, args.dry_run, args.overwrite)?;
    assert_not_overwrite(&metadata_path, args.dry_run, args.overwrite)?;
    let mut records = list_organizations_with_request(&mut request_json)?
        .into_iter()
        .map(|item| normalize_org_row(&item))
        .collect::<Vec<Map<String, Value>>>();
    records.retain(|row| {
        if let Some(org_id) = args.org_id {
            if scalar_text(row.get("id")) != org_id.to_string() {
                return false;
            }
        }
        if let Some(name) = &args.name {
            return string_field(row, "name", "") == *name;
        }
        true
    });
    if args.with_users {
        for row in records.iter_mut() {
            let org_id = scalar_text(row.get("id"));
            let users = list_org_users_with_request(&mut request_json, &org_id)?
                .into_iter()
                .map(|user| normalize_org_user_row(&user))
                .map(Value::Object)
                .collect::<Vec<Value>>();
            row.insert(
                "userCount".to_string(),
                Value::String(users.len().to_string()),
            );
            row.insert("users".to_string(), Value::Array(users));
        }
    }
    if !args.dry_run {
        write_json_file(
            &payload_path,
            &Value::Object(Map::from_iter(vec![
                (
                    "kind".to_string(),
                    Value::String(ACCESS_EXPORT_KIND_ORGS.to_string()),
                ),
                (
                    "version".to_string(),
                    Value::Number(ACCESS_EXPORT_VERSION.into()),
                ),
                (
                    "records".to_string(),
                    Value::Array(records.iter().cloned().map(Value::Object).collect()),
                ),
            ])),
            args.overwrite,
        )?;
        write_json_file(
            &metadata_path,
            &Value::Object(build_org_export_metadata(
                &args.common.url,
                args.common.profile.as_deref(),
                &args.output_dir,
                records.len(),
                args.with_users,
                args.org_id,
                args.name.as_deref(),
            )),
            args.overwrite,
        )?;
    }
    println!(
        "{}",
        access_export_summary_line(
            "org",
            records.len(),
            &args.common.url,
            &payload_path.to_string_lossy(),
            &metadata_path.to_string_lossy(),
            args.dry_run,
        )
    );
    Ok(records.len())
}

pub(crate) fn import_orgs_with_request<F>(
    mut request_json: F,
    args: &OrgImportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_basic_auth_only(&args.common)?;
    let records = load_org_import_records(&args.input_dir)?
        .into_iter()
        .map(|record| normalize_org_row(&record))
        .collect::<Vec<Map<String, Value>>>();
    let mut processed = 0usize;
    let mut created = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut live_orgs = list_organizations_with_request(&mut request_json)?;

    for record in records {
        processed += 1;
        let desired_name = string_field(&record, "name", "");
        if desired_name.is_empty() {
            return Err(message("Organization import record is missing name."));
        }
        let exported_id = scalar_text(record.get("id"));
        let existing = live_orgs
            .iter()
            .find(|org| {
                // Match by stable exported id when present, but still allow
                // name-based reconciliation for bundles that only carry names.
                string_field(org, "name", "") == desired_name
                    || (!exported_id.is_empty() && scalar_text(org.get("id")) == exported_id)
            })
            .cloned();
        let existing_found = existing.is_some();

        let org_id = if let Some(existing) = existing.as_ref() {
            let existing_id = scalar_text(existing.get("id"));
            if !args.replace_existing {
                skipped += 1;
                println!("Skipped existing org {}", desired_name);
                continue;
            }
            if !exported_id.is_empty()
                && exported_id == existing_id
                && string_field(existing, "name", "") != desired_name
            {
                // Same exported org id but different current name means rename,
                // not create. This preserves downstream user-role sync in-place.
                if args.dry_run {
                    println!(
                        "Would rename org {} -> {}",
                        string_field(existing, "name", ""),
                        desired_name
                    );
                } else {
                    let payload = Value::Object(Map::from_iter(vec![(
                        "name".to_string(),
                        Value::String(desired_name.clone()),
                    )]));
                    let _ = update_organization_with_request(
                        &mut request_json,
                        &existing_id,
                        &payload,
                    )?;
                }
            }
            existing_id
        } else {
            if !args.replace_existing {
                skipped += 1;
                println!(
                    "Skipped org {}: missing and --replace-existing was not set.",
                    desired_name
                );
                continue;
            }
            if args.dry_run {
                created += 1;
                println!("Would create org {}", desired_name);
                exported_id.clone()
            } else {
                let payload = Value::Object(Map::from_iter(vec![(
                    "name".to_string(),
                    Value::String(desired_name.clone()),
                )]));
                let created_payload =
                    create_organization_with_request(&mut request_json, &payload)?;
                created += 1;
                let created_id = scalar_text(created_payload.get("orgId"));
                live_orgs.push(Map::from_iter(vec![
                    ("id".to_string(), Value::String(created_id.clone())),
                    ("name".to_string(), Value::String(desired_name.clone())),
                ]));
                created_id
            }
        };

        if !org_id.is_empty() {
            let desired_users = match record.get("users") {
                Some(Value::Array(values)) => values
                    .iter()
                    .filter_map(|item| value_as_object(item, "Unexpected org user record.").ok())
                    .map(normalize_org_user_row)
                    .collect::<Vec<Map<String, Value>>>(),
                _ => Vec::new(),
            };
            let live_users = if desired_users.is_empty() {
                Vec::new()
            } else {
                list_org_users_with_request(&mut request_json, &org_id)?
            };
            for desired_user in desired_users {
                let login = string_field(&desired_user, "login", "");
                let email = string_field(&desired_user, "email", "");
                let identity = if !login.is_empty() { login } else { email };
                if identity.is_empty() {
                    continue;
                }
                let desired_role = {
                    let role = string_field(&desired_user, "orgRole", "");
                    if role.is_empty() {
                        "Viewer".to_string()
                    } else {
                        role
                    }
                };
                let existing_user = live_users.iter().find(|user| {
                    string_field(user, "login", "") == identity
                        || string_field(user, "email", "") == identity
                });
                match existing_user {
                    Some(user) => {
                        let current_role = string_field(user, "role", "");
                        if current_role != desired_role {
                            let blocker = org_user_role_change_blocker(user);
                            if args.dry_run {
                                if let Some(blocker) = blocker {
                                    println!(
                                        "Blocked org user role update {} -> {} in org {} ({})",
                                        identity,
                                        desired_role,
                                        desired_name,
                                        org_user_role_target_evidence(user)
                                    );
                                    println!("  blocker: {blocker}");
                                } else {
                                    println!(
                                        "Would update org user role {} -> {} in org {} ({})",
                                        identity,
                                        desired_role,
                                        desired_name,
                                        org_user_role_target_evidence(user)
                                    );
                                }
                            } else {
                                if let Some(blocker) = blocker {
                                    return Err(message(format!(
                                        "Org import blocked for {} user {}: {} ({})",
                                        desired_name,
                                        identity,
                                        blocker,
                                        org_user_role_target_evidence(user)
                                    )));
                                }
                                let user_id = {
                                    let user_id = scalar_text(user.get("userId"));
                                    if user_id.is_empty() {
                                        scalar_text(user.get("id"))
                                    } else {
                                        user_id
                                    }
                                };
                                let _ = update_org_user_role_with_request(
                                    &mut request_json,
                                    &org_id,
                                    &user_id,
                                    &desired_role,
                                )?;
                            }
                        }
                    }
                    None => {
                        // Import adds missing listed users, but intentionally does
                        // not remove extra live users absent from the bundle.
                        if args.dry_run {
                            println!(
                                "Would add org user {} -> {} in org {}",
                                identity, desired_role, desired_name
                            );
                        } else {
                            let _ = add_user_to_org_with_request(
                                &mut request_json,
                                &org_id,
                                &identity,
                                &desired_role,
                            )?;
                        }
                    }
                }
            }
        }
        if existing_found {
            updated += 1;
            println!("Updated org {}", desired_name);
        }
    }

    println!(
        "{}",
        access_import_summary_line(
            "org",
            processed,
            created,
            updated,
            skipped,
            &args.input_dir.to_string_lossy(),
        )
    );
    Ok(0)
}
