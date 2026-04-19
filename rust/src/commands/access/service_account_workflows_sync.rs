//! Sync service-account export, import, and diff workflows against Grafana.
//! This module turns live Grafana service-account state into export bundles, loads
//! saved bundle records for import or diff, and validates dry-run output before any
//! write path proceeds. It is the coordination layer between HTTP fetches, local
//! bundle files, and the operator-facing CLI commands.

use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, render_json_value, string_field, write_json_file, Result};

use super::super::super::render::{
    access_diff_summary_line, access_export_summary_line, access_import_summary_line, format_table,
    normalize_service_account_row, scalar_text, service_account_role_to_api, value_bool,
};
use super::super::super::{
    ServiceAccountDiffArgs, ServiceAccountExportArgs, ServiceAccountImportArgs,
    ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS, ACCESS_EXPORT_METADATA_FILENAME, ACCESS_EXPORT_VERSION,
    ACCESS_SERVICE_ACCOUNT_EXPORT_FILENAME, DEFAULT_PAGE_SIZE,
};
use super::super::{
    create_service_account_with_request, list_service_accounts_with_request,
    update_service_account_with_request,
};
use super::service_account_workflows_support::{
    assert_not_overwrite, attach_service_account_target, build_record_diff_fields,
    build_service_account_diff_map, build_service_account_diff_review_line,
    build_service_account_export_metadata, build_service_account_import_dry_run_document,
    build_service_account_import_dry_run_row, build_service_account_import_dry_run_rows,
    list_all_service_accounts_with_request, load_service_account_import_records,
    mark_service_account_import_row_blocked, normalize_service_account_import_role,
    validate_service_account_import_dry_run_output,
};

fn service_account_privilege_warning(role: &str) -> Option<&'static str> {
    if role == "Admin" {
        Some(
            "Grafana may reject Admin role changes if the caller lacks sufficient service-account privileges.",
        )
    } else {
        None
    }
}

fn attach_service_account_warnings(row: &mut Map<String, Value>, warning: Option<&str>) {
    if let Some(warning) = warning {
        row.insert(
            "warnings".to_string(),
            Value::Array(vec![Value::String(warning.to_string())]),
        );
    }
}

/// Purpose: implementation note.
pub(crate) fn export_service_accounts_with_request<F>(
    mut request_json: F,
    args: &ServiceAccountExportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let records = list_all_service_accounts_with_request(&mut request_json)?
        .into_iter()
        .map(|item| normalize_service_account_row(&item))
        .collect::<Vec<Map<String, Value>>>();
    let bundle_path = args.output_dir.join(ACCESS_SERVICE_ACCOUNT_EXPORT_FILENAME);
    let metadata_path = args.output_dir.join(ACCESS_EXPORT_METADATA_FILENAME);
    assert_not_overwrite(&bundle_path, args.dry_run, args.overwrite)?;
    assert_not_overwrite(&metadata_path, args.dry_run, args.overwrite)?;
    if !args.dry_run {
        let payload = Value::Object(Map::from_iter(vec![
            (
                "kind".to_string(),
                Value::String(ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS.to_string()),
            ),
            (
                "version".to_string(),
                Value::Number(ACCESS_EXPORT_VERSION.into()),
            ),
            (
                "records".to_string(),
                Value::Array(records.iter().cloned().map(Value::Object).collect()),
            ),
        ]));
        write_json_file(&bundle_path, &payload, args.overwrite)?;
        write_json_file(
            &metadata_path,
            &Value::Object(build_service_account_export_metadata(
                &args.common.url,
                args.common.profile.as_deref(),
                &args.output_dir,
                records.len(),
            )),
            args.overwrite,
        )?;
    }
    println!(
        "{}",
        access_export_summary_line(
            "service-account",
            records.len(),
            &args.common.url,
            &bundle_path.to_string_lossy(),
            &metadata_path.to_string_lossy(),
            args.dry_run,
        )
    );
    Ok(records.len())
}

/// Purpose: implementation note.
pub(crate) fn import_service_accounts_with_request<F>(
    mut request_json: F,
    args: &ServiceAccountImportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_service_account_import_dry_run_output(args)?;
    let records =
        load_service_account_import_records(&args.input_dir, ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS)?;
    let mut created = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut processed = 0usize;
    let mut dry_run_rows = Vec::new();
    let structured_output = args.dry_run && (args.table || args.json);

    for (index, record) in records.iter().enumerate() {
        processed += 1;
        let identity = string_field(record, "name", "");
        if identity.is_empty() {
            return Err(message(format!(
                "Access service-account import record {} in {} lacks name.",
                index + 1,
                args.input_dir.display()
            )));
        }
        let existing = list_service_accounts_with_request(
            &mut request_json,
            Some(&identity),
            1,
            DEFAULT_PAGE_SIZE,
        )?
        .into_iter()
        .find(|item| string_field(item, "name", "") == identity);
        let desired_role_raw = string_field(record, "role", "");

        if existing.is_none() {
            if !args.replace_existing {
                skipped += 1;
                let detail = "missing and --replace-existing was not set.";
                if structured_output {
                    dry_run_rows.push(build_service_account_import_dry_run_row(
                        index + 1,
                        &identity,
                        "skip",
                        detail,
                    ));
                } else {
                    println!(
                        "Skipped service-account {} ({}): {}",
                        identity,
                        index + 1,
                        detail
                    );
                }
                continue;
            }
            if args.dry_run {
                let desired_role = match normalize_service_account_import_role(&desired_role_raw) {
                    Ok(role) => role.unwrap_or_else(|| "Viewer".to_string()),
                    Err(error) => {
                        let detail = error.to_string();
                        if structured_output {
                            let mut row = build_service_account_import_dry_run_row(
                                index + 1,
                                &identity,
                                "blocked",
                                &detail,
                            );
                            mark_service_account_import_row_blocked(&mut row, &detail);
                            dry_run_rows.push(row);
                        } else {
                            println!("Blocked service-account {} import: {}", identity, detail);
                        }
                        continue;
                    }
                };
                let role_warning = service_account_privilege_warning(&desired_role);
                created += 1;
                if structured_output {
                    let detail = if let Some(warning) = role_warning {
                        format!("would create service account role={desired_role}; {warning}")
                    } else {
                        format!("would create service account role={desired_role}")
                    };
                    let mut row = build_service_account_import_dry_run_row(
                        index + 1,
                        &identity,
                        "create",
                        &detail,
                    );
                    attach_service_account_warnings(&mut row, role_warning);
                    dry_run_rows.push(row);
                } else if let Some(warning) = role_warning {
                    println!(
                        "Would create service-account {} role={} ({})",
                        identity, desired_role, warning
                    );
                } else {
                    println!(
                        "Would create service-account {} role={}",
                        identity, desired_role
                    );
                }
                continue;
            }
            let desired_role = match normalize_service_account_import_role(&desired_role_raw) {
                Ok(role) => role.unwrap_or_else(|| "Viewer".to_string()),
                Err(error) => return Err(error),
            };
            let payload = Value::Object(Map::from_iter(vec![
                ("name".to_string(), Value::String(identity.clone())),
                (
                    "role".to_string(),
                    Value::String(service_account_role_to_api(&desired_role)),
                ),
                (
                    "isDisabled".to_string(),
                    Value::Bool(
                        value_bool(record.get("disabled"))
                            .or_else(|| value_bool(record.get("isDisabled")))
                            .unwrap_or(false),
                    ),
                ),
            ]));
            let _ = create_service_account_with_request(&mut request_json, &payload)?;
            created += 1;
            println!("Created service-account {}", identity);
            continue;
        }

        let existing = existing.unwrap();
        if !args.replace_existing {
            skipped += 1;
            let detail = "existing and --replace-existing was not set.";
            if structured_output {
                dry_run_rows.push(build_service_account_import_dry_run_row(
                    index + 1,
                    &identity,
                    "skip",
                    detail,
                ));
            } else {
                println!(
                    "Skipped existing service-account {} ({})",
                    identity,
                    index + 1
                );
            }
            continue;
        }

        let desired_role = match normalize_service_account_import_role(&desired_role_raw) {
            Ok(role) => role.unwrap_or_else(|| "Viewer".to_string()),
            Err(error) => {
                let detail = error.to_string();
                if args.dry_run {
                    if structured_output {
                        let mut row = build_service_account_import_dry_run_row(
                            index + 1,
                            &identity,
                            "blocked",
                            &detail,
                        );
                        mark_service_account_import_row_blocked(&mut row, &detail);
                        attach_service_account_target(&mut row, &existing);
                        dry_run_rows.push(row);
                    } else {
                        println!("Blocked service-account {} import: {}", identity, detail);
                    }
                    continue;
                }
                return Err(error);
            }
        };
        let role_warning = service_account_privilege_warning(&desired_role);
        let existing_role = string_field(&existing, "role", "");
        let desired_disabled =
            value_bool(record.get("disabled")).or_else(|| value_bool(record.get("isDisabled")));
        let existing_disabled =
            value_bool(existing.get("disabled")).or_else(|| value_bool(existing.get("isDisabled")));
        let mut update_payload =
            Map::from_iter(vec![("name".to_string(), Value::String(identity.clone()))]);
        let mut changed = Vec::new();
        if desired_role != existing_role {
            update_payload.insert(
                "role".to_string(),
                Value::String(service_account_role_to_api(&desired_role)),
            );
            changed.push("role".to_string());
        }
        if desired_disabled.is_some() && desired_disabled != existing_disabled {
            update_payload.insert(
                "isDisabled".to_string(),
                Value::Bool(desired_disabled.unwrap_or(false)),
            );
            changed.push("disabled".to_string());
        }
        if changed.is_empty() {
            skipped += 1;
            let detail = "already matched live state.";
            if structured_output {
                dry_run_rows.push(build_service_account_import_dry_run_row(
                    index + 1,
                    &identity,
                    "skip",
                    detail,
                ));
            } else {
                println!(
                    "Skipped service-account {} ({}): {}",
                    identity,
                    index + 1,
                    detail
                );
            }
            continue;
        }
        if args.dry_run {
            updated += 1;
            let role_changed = changed.iter().any(|field| field == "role");
            let detail = if role_changed {
                if let Some(warning) = role_warning {
                    format!(
                        "would update fields={} role={desired_role}; {warning}",
                        changed.join(",")
                    )
                } else {
                    format!(
                        "would update fields={} role={desired_role}",
                        changed.join(",")
                    )
                }
            } else {
                format!("would update fields={}", changed.join(","))
            };
            if structured_output {
                let mut row = build_service_account_import_dry_run_row(
                    index + 1,
                    &identity,
                    "update",
                    &detail,
                );
                attach_service_account_target(&mut row, &existing);
                attach_service_account_warnings(
                    &mut row,
                    if role_changed { role_warning } else { None },
                );
                dry_run_rows.push(row);
            } else {
                println!("Would update service-account {} {}", identity, detail);
            }
            continue;
        }
        let service_account_id = scalar_text(existing.get("id"));
        if service_account_id.is_empty() {
            return Err(message(format!(
                "Resolved service-account did not include an id: {}",
                identity
            )));
        }
        let _ = update_service_account_with_request(
            &mut request_json,
            &service_account_id,
            &Value::Object(update_payload),
        )?;
        updated += 1;
        println!("Updated service-account {}", identity);
    }

    if structured_output {
        if args.json {
            print!(
                "{}",
                render_json_value(&build_service_account_import_dry_run_document(
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
                &build_service_account_import_dry_run_rows(&dry_run_rows),
            ) {
                println!("{line}");
            }
            println!();
        }
    }

    println!(
        "{}",
        access_import_summary_line(
            "service-account",
            processed,
            created,
            updated,
            skipped,
            &args.input_dir.to_string_lossy(),
        )
    );
    Ok(0)
}

/// Purpose: implementation note.
pub(crate) fn diff_service_accounts_with_request<F>(
    mut request_json: F,
    args: &ServiceAccountDiffArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let local_records =
        load_service_account_import_records(&args.diff_dir, ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS)?;
    let local_map =
        build_service_account_diff_map(&local_records, &args.diff_dir.to_string_lossy())?;
    let live_records = list_all_service_accounts_with_request(&mut request_json)?
        .into_iter()
        .map(|item| normalize_service_account_row(&item))
        .collect::<Vec<Map<String, Value>>>();
    let live_map = build_service_account_diff_map(&live_records, "Grafana live service accounts")?;

    let mut differences = 0usize;
    let mut checked = 0usize;
    for key in local_map.keys() {
        checked += 1;
        let (identity, local_payload) = &local_map[key];
        match live_map.get(key) {
            None => {
                println!("Diff missing-live service-account {}", identity);
                differences += 1;
            }
            Some((_live_identity, live_payload)) => {
                let changed = build_record_diff_fields(local_payload, live_payload);
                if changed.is_empty() {
                    println!("Diff same service-account {}", identity);
                } else {
                    differences += 1;
                    println!(
                        "Diff different service-account {} fields={}",
                        identity,
                        changed.join(",")
                    );
                }
            }
        }
    }
    for key in live_map.keys() {
        if local_map.contains_key(key) {
            continue;
        }
        checked += 1;
        differences += 1;
        let (identity, _) = &live_map[key];
        println!("Diff extra-live service-account {}", identity);
    }
    println!(
        "{}",
        build_service_account_diff_review_line(
            checked,
            differences,
            &args.diff_dir.to_string_lossy(),
            "Grafana live service accounts",
        )
    );
    println!(
        "{}",
        access_diff_summary_line(
            "service-account",
            checked,
            differences,
            &args.diff_dir.to_string_lossy(),
            "Grafana live service accounts",
        )
    );
    Ok(differences)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access::{CommonCliArgs, DryRunOutputFormat};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn service_account_import_dry_run_rows_include_status_and_blocked_summary() {
        let planned = build_service_account_import_dry_run_row(
            1,
            "svc",
            "update",
            "would update fields=role",
        );
        let mut blocked =
            build_service_account_import_dry_run_row(2, "svc-blocked", "blocked", "invalid role");
        mark_service_account_import_row_blocked(&mut blocked, "invalid role");
        let rows = vec![planned, blocked];

        assert_eq!(
            build_service_account_import_dry_run_rows(&rows),
            vec![
                vec![
                    "1".to_string(),
                    "svc".to_string(),
                    "update".to_string(),
                    "planned".to_string(),
                    "would update fields=role".to_string(),
                ],
                vec![
                    "2".to_string(),
                    "svc-blocked".to_string(),
                    "blocked".to_string(),
                    "blocked".to_string(),
                    "invalid role".to_string(),
                ],
            ]
        );

        let document = build_service_account_import_dry_run_document(
            &rows,
            2,
            0,
            1,
            0,
            std::path::Path::new("./access-service-accounts"),
        );
        assert_eq!(document["summary"]["blocked"], json!(1));
        assert_eq!(document["rows"][0]["status"], json!("planned"));
        assert_eq!(document["rows"][1]["status"], json!("blocked"));
    }

    #[test]
    fn service_account_import_dry_run_update_rows_include_target_evidence() {
        let mut row = build_service_account_import_dry_run_row(
            2,
            "svc",
            "update",
            "would update fields=role,disabled",
        );
        let target = Map::from_iter(vec![
            ("id".to_string(), json!("4")),
            ("name".to_string(), json!("svc")),
            ("login".to_string(), json!("sa-svc")),
            ("role".to_string(), json!("Viewer")),
            ("isDisabled".to_string(), json!(false)),
            ("tokens".to_string(), json!("2")),
            ("orgId".to_string(), json!("1")),
        ]);

        attach_service_account_target(&mut row, &target);

        assert_eq!(row["status"], json!("planned"));
        assert_eq!(row["blocked"], json!(false));
        assert_eq!(row["target"]["targetId"], json!("4"));
        assert_eq!(row["target"]["name"], json!("svc"));
        assert_eq!(row["target"]["login"], json!("sa-svc"));
        assert_eq!(row["target"]["role"], json!("Viewer"));
        assert_eq!(row["target"]["isDisabled"], json!(false));
    }

    #[test]
    fn service_account_import_rejects_invalid_role_before_create() {
        let temp_dir = tempdir().unwrap();
        fs::write(
            temp_dir.path().join("service-accounts.json"),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-access-service-account-export-index",
                "version": 1,
                "records": [
                    {"name": "svc-invalid", "role": "SuperAdmin", "disabled": false}
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let args = ServiceAccountImportArgs {
            common: CommonCliArgs {
                profile: None,
                url: "http://127.0.0.1:3000".to_string(),
                api_token: Some("token".to_string()),
                username: None,
                password: None,
                prompt_password: false,
                prompt_token: false,
                org_id: None,
                timeout: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            input_dir: temp_dir.path().to_path_buf(),
            local: false,
            run: None,
            run_id: None,
            replace_existing: true,
            dry_run: false,
            table: false,
            json: false,
            output_format: DryRunOutputFormat::Text,
            yes: false,
        };

        let mut calls = Vec::new();
        let error = import_service_accounts_with_request(
            |method, path, params, payload| {
                calls.push((
                    method.to_string(),
                    path.to_string(),
                    params.to_vec(),
                    payload.cloned(),
                ));
                match path {
                    "/api/serviceaccounts/search" => Ok(Some(json!({"serviceAccounts": []}))),
                    _ => panic!("unexpected path {path}"),
                }
            },
            &args,
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("Service account import role must be one of None, Viewer, Editor, Admin"));
        assert!(calls
            .iter()
            .all(|(_, path, _, _)| path == "/api/serviceaccounts/search"));
    }
}
