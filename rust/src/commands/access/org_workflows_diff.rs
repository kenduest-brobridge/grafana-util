use reqwest::Method;
use serde_json::Value;

use crate::common::Result;

use super::super::super::render::{access_diff_review_line, access_diff_summary_line};
use super::super::super::OrgDiffArgs;
use super::super::org_import_export_diff::{
    build_org_diff_map, build_org_live_records_for_diff, build_record_diff_fields,
    load_org_import_records,
};
use super::super::validate_basic_auth_only;

pub(crate) fn diff_orgs_with_request<F>(mut request_json: F, args: &OrgDiffArgs) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    validate_basic_auth_only(&args.common)?;
    let local_records = load_org_import_records(&args.diff_dir)?;
    let include_users = local_records
        .iter()
        .any(|record| record.contains_key("users"));
    let local_map = build_org_diff_map(
        &local_records,
        &args.diff_dir.to_string_lossy(),
        include_users,
    )?;
    let live_records = build_org_live_records_for_diff(&mut request_json, include_users)?;
    let live_map = build_org_diff_map(&live_records, "Grafana live orgs", include_users)?;

    let mut differences = 0usize;
    let mut checked = 0usize;
    for key in local_map.keys() {
        checked += 1;
        let (local_identity, local_payload) = &local_map[key];
        match live_map.get(key) {
            None => {
                println!("Diff missing-live org {}", local_identity);
                differences += 1;
            }
            Some((_live_identity, live_payload)) => {
                let changed = build_record_diff_fields(local_payload, live_payload);
                if changed.is_empty() {
                    println!("Diff same org {}", local_identity);
                } else {
                    differences += 1;
                    println!(
                        "Diff different org {} fields={}",
                        local_identity,
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
        println!("Diff extra-live org {}", identity);
    }
    println!(
        "{}",
        org_diff_review_line(checked, differences, &args.diff_dir.to_string_lossy())
    );
    println!(
        "{}",
        access_diff_summary_line(
            "org",
            checked,
            differences,
            &args.diff_dir.to_string_lossy(),
            "Grafana live orgs",
        )
    );
    Ok(differences)
}

fn org_diff_review_line(checked: usize, differences: usize, local_source: &str) -> String {
    access_diff_review_line(
        "org",
        checked,
        differences,
        local_source,
        "Grafana live orgs",
    )
}

#[cfg(test)]
mod org_diff_review_tests {
    use super::*;

    #[test]
    fn org_diff_review_line_uses_shared_review_contract() {
        let line = org_diff_review_line(3, 1, "./access-orgs");

        assert_eq!(
            line,
            "Review: required=true reviewed=false kind=org checked=3 same=2 different=1 source=./access-orgs live=Grafana live orgs"
        );
    }
}
