use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::common::{message, string_field, Result};

use super::{build_auth_context, AlertCliArgs, AlertListKind, GrafanaAlertClient};

const ALERT_RULE_LIST_FIELDS: [&str; 4] = ["uid", "title", "folderUID", "ruleGroup"];
const CONTACT_POINT_LIST_FIELDS: [&str; 3] = ["uid", "name", "type"];
const MUTE_TIMING_LIST_FIELDS: [&str; 2] = ["name", "intervals"];
const TEMPLATE_LIST_FIELDS: [&str; 1] = ["name"];

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn render_alert_table(
    rows: &[BTreeMap<&str, String>],
    fields: &[&str],
    headers: &[(&str, &str)],
    include_header: bool,
) -> String {
    let mut widths = BTreeMap::new();
    for (field, header) in headers {
        let mut width = header.len();
        for row in rows {
            width = width.max(row.get(field).map(|item| item.len()).unwrap_or(0));
        }
        widths.insert(*field, width);
    }

    let build_row = |values: &BTreeMap<&str, String>| -> String {
        fields
            .iter()
            .map(|field| {
                let width = *widths.get(field).unwrap_or(&0);
                format!(
                    "{:width$}",
                    values.get(field).map(String::as_str).unwrap_or(""),
                    width = width
                )
            })
            .collect::<Vec<_>>()
            .join("  ")
    };

    let mut lines = Vec::new();
    if include_header {
        let header_map = headers
            .iter()
            .map(|(field, header)| (*field, header.to_string()))
            .collect::<BTreeMap<_, _>>();
        lines.push(build_row(&header_map));
        lines.push(
            fields
                .iter()
                .map(|field| "-".repeat(*widths.get(field).unwrap_or(&0)))
                .collect::<Vec<_>>()
                .join("  "),
        );
    }
    for row in rows {
        lines.push(build_row(row));
    }
    lines.join("\n")
}

fn render_alert_csv(rows: &[BTreeMap<&str, String>], fields: &[&str]) -> String {
    let mut output = String::new();
    let _ = writeln!(&mut output, "{}", fields.join(","));
    for row in rows {
        let line = fields
            .iter()
            .map(|field| csv_escape(row.get(field).map(String::as_str).unwrap_or("")))
            .collect::<Vec<_>>()
            .join(",");
        let _ = writeln!(&mut output, "{line}");
    }
    output
}

fn serialize_rule_list_rows(items: &[Map<String, Value>]) -> Vec<BTreeMap<&'static str, String>> {
    items
        .iter()
        .map(|item| {
            BTreeMap::from([
                ("uid", string_field(item, "uid", "")),
                ("title", string_field(item, "title", "")),
                ("folderUID", string_field(item, "folderUID", "")),
                ("ruleGroup", string_field(item, "ruleGroup", "")),
            ])
        })
        .collect()
}

fn serialize_contact_point_list_rows(
    items: &[Map<String, Value>],
) -> Vec<BTreeMap<&'static str, String>> {
    items
        .iter()
        .map(|item| {
            BTreeMap::from([
                ("uid", string_field(item, "uid", "")),
                ("name", string_field(item, "name", "")),
                ("type", string_field(item, "type", "")),
            ])
        })
        .collect()
}

fn serialize_mute_timing_list_rows(
    items: &[Map<String, Value>],
) -> Vec<BTreeMap<&'static str, String>> {
    items
        .iter()
        .map(|item| {
            let intervals = item
                .get("time_intervals")
                .and_then(Value::as_array)
                .map(|value| value.len())
                .unwrap_or(0);
            BTreeMap::from([
                ("name", string_field(item, "name", "")),
                ("intervals", intervals.to_string()),
            ])
        })
        .collect()
}

fn serialize_template_list_rows(
    items: &[Map<String, Value>],
) -> Vec<BTreeMap<&'static str, String>> {
    items
        .iter()
        .map(|item| BTreeMap::from([("name", string_field(item, "name", ""))]))
        .collect()
}

pub fn list_alert_resources(args: &AlertCliArgs) -> Result<()> {
    let client = GrafanaAlertClient::new(&build_auth_context(args)?)?;
    match args.list_kind {
        Some(AlertListKind::Rules) => {
            let rows = serialize_rule_list_rows(&client.list_alert_rules()?);
            if args.json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if args.csv {
                print!("{}", render_alert_csv(&rows, &ALERT_RULE_LIST_FIELDS));
            } else {
                println!(
                    "{}",
                    render_alert_table(
                        &rows,
                        &ALERT_RULE_LIST_FIELDS,
                        &[
                            ("uid", "UID"),
                            ("title", "TITLE"),
                            ("folderUID", "FOLDER_UID"),
                            ("ruleGroup", "RULE_GROUP")
                        ],
                        !args.no_header,
                    )
                );
            }
        }
        Some(AlertListKind::ContactPoints) => {
            let rows = serialize_contact_point_list_rows(&client.list_contact_points()?);
            if args.json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if args.csv {
                print!("{}", render_alert_csv(&rows, &CONTACT_POINT_LIST_FIELDS));
            } else {
                println!(
                    "{}",
                    render_alert_table(
                        &rows,
                        &CONTACT_POINT_LIST_FIELDS,
                        &[("uid", "UID"), ("name", "NAME"), ("type", "TYPE")],
                        !args.no_header,
                    )
                );
            }
        }
        Some(AlertListKind::MuteTimings) => {
            let rows = serialize_mute_timing_list_rows(&client.list_mute_timings()?);
            if args.json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if args.csv {
                print!("{}", render_alert_csv(&rows, &MUTE_TIMING_LIST_FIELDS));
            } else {
                println!(
                    "{}",
                    render_alert_table(
                        &rows,
                        &MUTE_TIMING_LIST_FIELDS,
                        &[("name", "NAME"), ("intervals", "INTERVALS")],
                        !args.no_header,
                    )
                );
            }
        }
        Some(AlertListKind::Templates) => {
            let rows = serialize_template_list_rows(&client.list_templates()?);
            if args.json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if args.csv {
                print!("{}", render_alert_csv(&rows, &TEMPLATE_LIST_FIELDS));
            } else {
                println!(
                    "{}",
                    render_alert_table(
                        &rows,
                        &TEMPLATE_LIST_FIELDS,
                        &[("name", "NAME")],
                        !args.no_header,
                    )
                );
            }
        }
        None => return Err(message("Alert list command is required.")),
    }
    Ok(())
}
