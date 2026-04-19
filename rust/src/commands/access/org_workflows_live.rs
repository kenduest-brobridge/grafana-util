use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, render_json_value, string_field, Result};

use super::super::super::pending_delete::{
    format_prompt_row, print_delete_confirmation_summary, prompt_confirm_delete,
    prompt_select_indexes, validate_delete_prompt,
};
use super::super::super::render::{
    access_delete_summary_line, build_access_delete_review_document, format_table, render_csv,
    render_objects_json, render_yaml, scalar_text,
};
use super::super::super::{OrgAddArgs, OrgDeleteArgs, OrgListArgs, OrgModifyArgs};
use super::super::org_import_export_diff::load_org_import_records;
use super::super::{
    create_organization_with_request, delete_organization_with_request,
    list_org_users_with_request, list_organizations_with_request, lookup_org_by_identity,
    normalize_org_row, normalize_org_user_row, org_csv_headers, org_matches, org_summary_line,
    org_table_headers, org_table_rows, update_organization_with_request, validate_basic_auth_only,
};

pub(crate) fn list_orgs_with_request<F>(mut request_json: F, args: &OrgListArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_basic_auth_only(&args.common)?;
    let mut rows = list_organizations_with_request(&mut request_json)?
        .into_iter()
        .map(|item| normalize_org_row(&item))
        .collect::<Vec<Map<String, Value>>>();
    rows.retain(|row| org_matches(row, args));
    if args.with_users {
        for row in rows.iter_mut() {
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
    render_org_list_rows(&rows, args)?;
    if args.table || (!args.json && !args.yaml && !args.csv) {
        println!();
        println!("Listed {} org(s) at {}", rows.len(), args.common.url);
    }
    Ok(rows.len())
}

pub(crate) fn list_orgs_from_input_dir(args: &OrgListArgs) -> Result<usize> {
    let input_dir = args
        .input_dir
        .as_ref()
        .ok_or_else(|| message("Organization list local mode requires --input-dir."))?;
    let mut rows = load_org_import_records(input_dir)?
        .into_iter()
        .map(|item| normalize_org_row(&item))
        .collect::<Vec<Map<String, Value>>>();
    rows.retain(|row| org_matches(row, args));
    if !args.with_users {
        for row in &mut rows {
            row.insert("users".to_string(), Value::Array(Vec::new()));
        }
    }
    render_org_list_rows(&rows, args)?;
    if args.table || (!args.json && !args.yaml && !args.csv) {
        println!();
        println!(
            "Listed {} org(s) from local bundle at {}",
            rows.len(),
            input_dir.display()
        );
    }
    Ok(rows.len())
}

fn render_org_list_rows(rows: &[Map<String, Value>], args: &OrgListArgs) -> Result<()> {
    if args.json {
        println!("{}", render_objects_json(rows)?);
    } else if args.yaml {
        println!("{}", render_yaml(&rows.to_vec())?);
    } else if args.table {
        for line in format_table(
            &org_table_headers(args.with_users),
            &org_table_rows(rows, args.with_users),
        ) {
            println!("{line}");
        }
    } else if args.csv {
        for line in render_csv(
            &org_csv_headers(args.with_users),
            &org_table_rows(rows, args.with_users),
        ) {
            println!("{line}");
        }
    } else {
        for row in rows {
            println!("{}", org_summary_line(row, args.with_users));
        }
    }
    Ok(())
}

pub(crate) fn add_org_with_request<F>(mut request_json: F, args: &OrgAddArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_basic_auth_only(&args.common)?;
    let payload = Value::Object(Map::from_iter(vec![(
        "name".to_string(),
        Value::String(args.name.clone()),
    )]));
    let created = create_organization_with_request(&mut request_json, &payload)?;
    let row = Map::from_iter(vec![
        (
            "id".to_string(),
            Value::String({
                let id = scalar_text(created.get("orgId"));
                if id.is_empty() {
                    scalar_text(created.get("id"))
                } else {
                    id
                }
            }),
        ),
        ("name".to_string(), Value::String(args.name.clone())),
        ("userCount".to_string(), Value::String("0".to_string())),
        ("users".to_string(), Value::Array(Vec::new())),
    ]);
    if args.json {
        println!("{}", render_objects_json(&[row])?);
    } else {
        println!(
            "Created org {} -> id={}",
            args.name,
            scalar_text(created.get("orgId"))
        );
    }
    Ok(0)
}

pub(crate) fn modify_org_with_request<F>(mut request_json: F, args: &OrgModifyArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_basic_auth_only(&args.common)?;
    let org = lookup_org_by_identity(&mut request_json, args.org_id, args.name.as_deref())?;
    let org_id = scalar_text(org.get("id"));
    let payload = Value::Object(Map::from_iter(vec![(
        "name".to_string(),
        Value::String(args.set_name.clone()),
    )]));
    let _ = update_organization_with_request(&mut request_json, &org_id, &payload)?;
    let row = Map::from_iter(vec![
        ("id".to_string(), Value::String(org_id.clone())),
        ("name".to_string(), Value::String(args.set_name.clone())),
        (
            "previousName".to_string(),
            Value::String(string_field(&org, "name", "")),
        ),
    ]);
    if args.json {
        println!("{}", render_objects_json(&[row])?);
    } else {
        println!(
            "Modified org {} -> id={} name={}",
            string_field(&org, "name", ""),
            org_id,
            args.set_name
        );
    }
    Ok(0)
}

pub(crate) fn delete_org_with_request<F>(mut request_json: F, args: &OrgDeleteArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_basic_auth_only(&args.common)?;
    validate_delete_prompt(args.prompt, args.json, "Org")?;
    if !args.prompt && !args.yes {
        return Err(message("Org delete requires --yes."));
    }
    let orgs = if args.prompt && args.org_id.is_none() && args.name.is_none() {
        let orgs = list_organizations_with_request(&mut request_json)?
            .into_iter()
            .map(|org| normalize_org_row(&org))
            .collect::<Vec<_>>();
        if orgs.is_empty() {
            return Err(message(
                "Org delete --prompt did not find any matching organizations.",
            ));
        }
        let labels = orgs.iter().map(org_delete_prompt_label).collect::<Vec<_>>();
        let Some(indexes) = prompt_select_indexes("Organizations To Delete", &labels)? else {
            println!("Cancelled org delete.");
            return Ok(0);
        };
        indexes
            .into_iter()
            .filter_map(|index| orgs.get(index).cloned())
            .collect::<Vec<Map<String, Value>>>()
    } else {
        vec![lookup_org_by_identity(
            &mut request_json,
            args.org_id,
            args.name.as_deref(),
        )?]
    };
    if args.prompt {
        let labels = orgs.iter().map(org_delete_prompt_label).collect::<Vec<_>>();
        print_delete_confirmation_summary("The following organizations will be deleted:", &labels);
    }
    if args.prompt && !prompt_confirm_delete(&format!("Delete {} organization(s)?", orgs.len()))? {
        println!("Cancelled org delete.");
        return Ok(0);
    }
    let mut rows = Vec::new();
    for org in &orgs {
        let org_id = scalar_text(org.get("id"));
        let delete_payload = delete_organization_with_request(&mut request_json, &org_id)?;
        rows.push(Map::from_iter(vec![
            ("id".to_string(), Value::String(org_id.clone())),
            (
                "name".to_string(),
                Value::String(string_field(org, "name", "")),
            ),
            (
                "userCount".to_string(),
                Value::String(string_field(org, "userCount", "")),
            ),
            (
                "message".to_string(),
                Value::String(string_field(&delete_payload, "message", "")),
            ),
        ]));
    }
    if args.json {
        println!(
            "{}",
            render_json_value(&build_access_delete_review_document(
                "org",
                "Grafana live orgs",
                &rows.iter().cloned().map(Value::Object).collect::<Vec<_>>(),
            ))?
        );
    } else {
        for row in &rows {
            println!(
                "{}",
                access_delete_summary_line(
                    "org",
                    &string_field(row, "name", ""),
                    &[
                        ("id", scalar_text(row.get("id"))),
                        ("userCount", string_field(row, "userCount", "")),
                        ("message", string_field(row, "message", "")),
                    ],
                )
            );
        }
        if rows.len() > 1 {
            println!("Deleted {} organization(s).", rows.len());
        }
    }
    Ok(rows.len())
}

fn org_delete_prompt_label(org: &Map<String, Value>) -> String {
    let name = string_field(org, "name", "-");
    let id = scalar_text(org.get("id"));
    let user_count = string_field(org, "userCount", "");
    let trailer = if user_count.is_empty() {
        format!("id={id}")
    } else {
        format!("id={id} users={user_count}")
    };
    format_prompt_row(&[(&name, 32)], &trailer)
}

#[cfg(test)]
mod org_delete_prompt_tests {
    use super::*;

    #[test]
    fn org_delete_prompt_label_includes_user_count_when_present() {
        let org = Map::from_iter(vec![
            ("id".to_string(), Value::String("4".to_string())),
            ("name".to_string(), Value::String("Main Org".to_string())),
            ("userCount".to_string(), Value::String("12".to_string())),
        ]);

        let label = org_delete_prompt_label(&org);

        assert!(label.contains("Main Org"));
        assert!(label.contains("id=4 users=12"));
    }

    #[test]
    fn org_delete_summary_line_includes_identity_and_context() {
        let line = access_delete_summary_line(
            "org",
            "Main Org",
            &[
                ("id", "4".to_string()),
                ("userCount", "12".to_string()),
                ("message", "Org deleted.".to_string()),
            ],
        );

        assert_eq!(
            line,
            "Deleted org Main Org id=4 userCount=12 message=Org deleted."
        );
    }
}
