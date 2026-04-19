use serde::Serialize;
use serde_json::{Map, Value};

use super::catalog::{
    describe_records, parse_selector, supported_kind_records, ResourceDescribeRecord,
};
use super::cli_defs::{
    ResourceDescribeArgs, ResourceGetArgs, ResourceKind, ResourceKindsArgs, ResourceListArgs,
    ResourceOutputFormat,
};
use super::runtime::{build_client, get_resource_item, list_resource_items};
use crate::common::{message, render_json_value, string_field, Result};
use crate::tabular_output::{print_lines, render_summary_table, render_table, render_yaml};

#[derive(Debug, Clone, Serialize)]
struct ResourceListDocument {
    kind: &'static str,
    count: usize,
    items: Vec<Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize)]
struct ResourceDescribeDocument {
    kind: Option<&'static str>,
    count: usize,
    items: Vec<ResourceDescribeRecord>,
}

fn list_row(kind: ResourceKind, item: &Map<String, Value>) -> Vec<String> {
    match kind {
        ResourceKind::Dashboards => vec![
            string_field(item, "uid", ""),
            string_field(item, "title", &string_field(item, "name", "")),
            string_field(item, "folderTitle", ""),
        ],
        ResourceKind::Folders => vec![
            string_field(item, "uid", ""),
            string_field(item, "title", ""),
            string_field(item, "parentUid", ""),
        ],
        ResourceKind::Datasources => vec![
            string_field(item, "uid", ""),
            string_field(item, "name", ""),
            string_field(item, "type", ""),
        ],
        ResourceKind::AlertRules => vec![
            string_field(item, "uid", ""),
            string_field(item, "title", ""),
            string_field(item, "folderUID", ""),
        ],
        ResourceKind::Orgs => vec![
            string_field(item, "id", ""),
            string_field(item, "name", ""),
            string_field(item, "address", ""),
        ],
    }
}

fn list_headers(kind: ResourceKind) -> [&'static str; 3] {
    match kind {
        ResourceKind::Dashboards => ["uid", "title", "folder"],
        ResourceKind::Folders => ["uid", "title", "parent_uid"],
        ResourceKind::Datasources => ["uid", "name", "type"],
        ResourceKind::AlertRules => ["uid", "title", "folder_uid"],
        ResourceKind::Orgs => ["id", "name", "address"],
    }
}

pub(crate) fn render_kind_catalog(args: &ResourceKindsArgs) -> Result<()> {
    let records = supported_kind_records();
    match args.output_format {
        ResourceOutputFormat::Text => {
            print_lines(
                &records
                    .iter()
                    .map(|record| {
                        format!(
                            "{} ({}) - {}",
                            record.kind, record.singular, record.description
                        )
                    })
                    .collect::<Vec<_>>(),
            );
        }
        ResourceOutputFormat::Table => {
            let rows = records
                .iter()
                .map(|record| {
                    vec![
                        record.kind.to_string(),
                        record.singular.to_string(),
                        record.description.to_string(),
                    ]
                })
                .collect::<Vec<_>>();
            print_lines(&render_table(&["kind", "singular", "description"], &rows));
        }
        ResourceOutputFormat::Json => {
            print!("{}", render_json_value(&records)?);
        }
        ResourceOutputFormat::Yaml => {
            print!("{}", render_yaml(&records)?);
        }
    }
    Ok(())
}

pub(crate) fn render_describe(args: &ResourceDescribeArgs) -> Result<()> {
    let items = describe_records(args.kind);
    let document = ResourceDescribeDocument {
        kind: args.kind.map(|kind| kind.as_str()),
        count: items.len(),
        items,
    };
    match args.output_format {
        ResourceOutputFormat::Text => {
            let mut lines = Vec::new();
            if let Some(kind) = document.kind {
                lines.push(format!("Resource kind: {kind}"));
            } else {
                lines.push("Resource kinds:".to_string());
            }
            for (index, record) in document.items.iter().enumerate() {
                if index > 0 {
                    lines.push(String::new());
                }
                lines.push(format!("Kind: {}", record.kind));
                lines.push(format!("Singular: {}", record.singular));
                lines.push(format!("Selector: {}", record.selector));
                lines.push(format!("List endpoint: {}", record.list_endpoint));
                lines.push(format!("Get endpoint: {}", record.get_endpoint));
                lines.push(format!("Description: {}", record.description));
            }
            print_lines(&lines);
        }
        ResourceOutputFormat::Table => {
            let rows = document
                .items
                .iter()
                .map(|record| {
                    vec![
                        record.kind.to_string(),
                        record.singular.to_string(),
                        record.selector.to_string(),
                        record.list_endpoint.to_string(),
                        record.get_endpoint.to_string(),
                        record.description.to_string(),
                    ]
                })
                .collect::<Vec<_>>();
            print_lines(&render_table(
                &[
                    "kind",
                    "singular",
                    "selector",
                    "list_endpoint",
                    "get_endpoint",
                    "description",
                ],
                &rows,
            ));
        }
        ResourceOutputFormat::Json => {
            print!("{}", render_json_value(&document)?);
        }
        ResourceOutputFormat::Yaml => {
            print!("{}", render_yaml(&document)?);
        }
    }
    Ok(())
}

pub(crate) fn render_list(args: &ResourceListArgs) -> Result<()> {
    let client = build_client(&args.common)?;
    let items = list_resource_items(&client, args.kind)?;
    let document = ResourceListDocument {
        kind: args.kind.as_str(),
        count: items.len(),
        items,
    };
    match args.output_format {
        ResourceOutputFormat::Text => {
            let lines = vec![
                format!("Resource list: {}", document.kind),
                format!("Count: {}", document.count),
            ];
            print_lines(&lines);
        }
        ResourceOutputFormat::Table => {
            let headers = list_headers(args.kind);
            let rows = document
                .items
                .iter()
                .map(|item| list_row(args.kind, item))
                .collect::<Vec<_>>();
            print_lines(&render_table(&headers, &rows));
        }
        ResourceOutputFormat::Json => {
            print!("{}", render_json_value(&document)?);
        }
        ResourceOutputFormat::Yaml => {
            print!("{}", render_yaml(&document)?);
        }
    }
    Ok(())
}

pub(crate) fn render_get(args: &ResourceGetArgs) -> Result<()> {
    let client = build_client(&args.common)?;
    let selector = parse_selector(&args.selector)?;
    let value = get_resource_item(&client, &selector)?;
    match args.output_format {
        ResourceOutputFormat::Text => {
            let object = value
                .as_object()
                .ok_or_else(|| message("Resource get text output requires a JSON object."))?;
            let summary = [
                ("kind", selector.kind.as_str().to_string()),
                ("identity", selector.identity.clone()),
                (
                    "title",
                    string_field(
                        object,
                        "title",
                        &string_field(object, "name", &string_field(object, "uid", "")),
                    ),
                ),
            ];
            print_lines(&render_summary_table(&summary));
        }
        ResourceOutputFormat::Table => {
            let object = value
                .as_object()
                .ok_or_else(|| message("Resource get table output requires a JSON object."))?;
            let rows = object
                .iter()
                .map(|(field, value)| {
                    (
                        field.as_str(),
                        match value {
                            Value::String(text) => text.clone(),
                            Value::Null => "null".to_string(),
                            _ => value.to_string(),
                        },
                    )
                })
                .collect::<Vec<_>>();
            print_lines(&render_summary_table(&rows));
        }
        ResourceOutputFormat::Json => {
            print!("{}", render_json_value(&value)?);
        }
        ResourceOutputFormat::Yaml => {
            print!("{}", render_yaml(&value)?);
        }
    }
    Ok(())
}
