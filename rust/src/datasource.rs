//! Datasource domain orchestrator.
//!
//! Purpose:
//! - Own datasource command flows (`list`, `add`, `delete`, `export`, `import`, `diff`).
//! - Normalize datasource contract shape across live API payloads and exported metadata.
//! - Keep output serialization (`table`/`csv`/`json`/`yaml`) selection centralized.
//!
//! Flow:
//! - Parse args from `dashboard`-shared auth/common CLI types where possible.
//! - Normalize command variants before branching by subcommand.
//! - Build client and route execution to list/export/import/diff helpers.
//!
//! Caveats:
//! - Keep API-field compatibility logic in `datasource_diff.rs` and import/export helpers.
//! - Avoid side effects in normalization helpers; keep them as pure value transforms.
use reqwest::Method;
use serde_json::{Map, Value};
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use crate::common::{
    load_json_object_file, message, render_json_value, string_field, write_json_file, Result,
};
use crate::dashboard::{
    build_auth_context, build_http_client, build_http_client_for_org, list_datasources,
    CommonCliArgs, SimpleOutputFormat,
};
use crate::datasource::datasource_diff::{
    build_datasource_diff_report, normalize_export_records, normalize_live_records,
    DatasourceDiffEntry, DatasourceDiffReport, DatasourceDiffStatus,
};
use crate::datasource_catalog::{
    render_supported_datasource_catalog_csv, render_supported_datasource_catalog_json,
    render_supported_datasource_catalog_table, render_supported_datasource_catalog_text,
    render_supported_datasource_catalog_yaml,
};
#[cfg(any(feature = "tui", test))]
use crate::interactive_browser::{run_interactive_browser, BrowserItem};
use crate::tabular_output::render_yaml;
#[cfg(feature = "tui")]
use crate::tui_shell;

#[path = "datasource_browse.rs"]
mod datasource_browse;
#[cfg(feature = "tui")]
#[path = "datasource_browse_edit_dialog.rs"]
mod datasource_browse_edit_dialog;
#[cfg(feature = "tui")]
#[path = "datasource_browse_input.rs"]
mod datasource_browse_input;
#[cfg(feature = "tui")]
#[path = "datasource_browse_render.rs"]
mod datasource_browse_render;
#[cfg(feature = "tui")]
#[path = "datasource_browse_state.rs"]
mod datasource_browse_state;
#[path = "datasource_browse_support.rs"]
mod datasource_browse_support;
#[cfg(feature = "tui")]
#[path = "datasource_browse_terminal.rs"]
mod datasource_browse_terminal;
#[cfg(feature = "tui")]
#[path = "datasource_browse_tui.rs"]
mod datasource_browse_tui;
#[path = "datasource_cli_defs.rs"]
mod datasource_cli_defs;
#[path = "datasource_diff.rs"]
mod datasource_diff;
#[path = "datasource_import_export.rs"]
mod datasource_import_export;
#[path = "datasource_mutation_support.rs"]
mod datasource_mutation_support;

#[cfg(feature = "tui")]
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
#[cfg(feature = "tui")]
use datasource_browse_terminal::TerminalSession;
pub(crate) use datasource_cli_defs::{normalize_datasource_group_command, root_command};
pub use datasource_cli_defs::{
    DatasourceAddArgs, DatasourceBrowseArgs, DatasourceCliArgs, DatasourceDeleteArgs,
    DatasourceDiffArgs, DatasourceExportArgs, DatasourceGroupCommand, DatasourceImportArgs,
    DatasourceImportInputFormat, DatasourceInspectExportArgs, DatasourceInspectExportOutputFormat,
    DatasourceListArgs, DatasourceModifyArgs, DatasourceTypesArgs, DryRunOutputFormat,
    ListOutputFormat,
};
pub(crate) use datasource_import_export::{
    build_all_orgs_export_index, build_all_orgs_export_metadata, build_all_orgs_output_dir,
    build_datasource_export_metadata, build_datasource_provisioning_document, build_export_index,
    build_export_records, build_list_records, export_datasource_scope, fetch_current_org,
    import_datasources_by_export_org, import_datasources_with_client, list_orgs,
    load_datasource_export_root_manifest, load_datasource_inventory_records_from_export_root,
    load_diff_record_values, load_import_records, render_data_source_csv, render_data_source_json,
    render_data_source_table, resolve_datasource_export_root_dir, resolve_target_client,
    validate_import_org_auth, write_yaml_file, DatasourceExportRootScopeKind,
    DatasourceImportRecord, DATASOURCE_EXPORT_FILENAME, DATASOURCE_PROVISIONING_FILENAME,
    DATASOURCE_PROVISIONING_SUBDIR, EXPORT_METADATA_FILENAME,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use datasource_import_export::{
    build_import_payload, build_import_payload_with_secret_values,
    classify_datasource_export_root_scope_kind, collect_datasource_import_dry_run_report,
    discover_export_org_import_scopes, format_routed_datasource_scope_summary_fields,
    format_routed_datasource_target_org_label, render_routed_datasource_import_org_table,
    resolve_export_org_target_plan, DatasourceExportOrgScope, DatasourceExportOrgTargetPlan,
    DatasourceImportDryRunReport,
};
#[cfg(test)]
pub(crate) use datasource_mutation_support::parse_json_object_argument;
pub(crate) use datasource_mutation_support::resolve_match;
use datasource_mutation_support::{
    build_add_payload, build_modify_payload, build_modify_updates,
    fetch_datasource_by_uid_if_exists, render_import_table, render_live_mutation_json,
    render_live_mutation_table, resolve_delete_match, resolve_live_mutation_match,
    validate_live_mutation_dry_run_args,
};
#[cfg(feature = "tui")]
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
#[cfg(feature = "tui")]
use ratatui::style::{Color, Modifier, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Line, Span};
#[cfg(feature = "tui")]
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DatasourceInspectExportRenderFormat {
    Text,
    Table,
    Csv,
    Json,
    Yaml,
}

pub(crate) struct DatasourceInspectExportSource {
    input_mode: &'static str,
    input_path: String,
    metadata: Option<Value>,
    records: Vec<Map<String, Value>>,
}

fn build_datasource_inspect_export_metadata(mut metadata: Map<String, Value>) -> Value {
    metadata.insert(
        "bundleKind".to_string(),
        Value::String("masked-recovery".to_string()),
    );
    metadata.insert("masked".to_string(), Value::Bool(true));
    metadata.insert("recoveryCapable".to_string(), Value::Bool(true));
    Value::Object(metadata)
}

fn datasource_inspect_provisioning_candidates(input_dir: &Path) -> [std::path::PathBuf; 4] {
    [
        input_dir.join(DATASOURCE_PROVISIONING_FILENAME),
        input_dir.join("datasources.yml"),
        input_dir
            .join(DATASOURCE_PROVISIONING_SUBDIR)
            .join(DATASOURCE_PROVISIONING_FILENAME),
        input_dir
            .join(DATASOURCE_PROVISIONING_SUBDIR)
            .join("datasources.yml"),
    ]
}

fn has_datasource_inventory_export(input_dir: &Path) -> bool {
    input_dir.join(EXPORT_METADATA_FILENAME).is_file()
}

fn has_datasource_provisioning_export(input_dir: &Path) -> bool {
    datasource_inspect_provisioning_candidates(input_dir)
        .iter()
        .any(|candidate| candidate.is_file())
}

fn datasource_inspect_uses_tty() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn resolve_datasource_workspace_input_dir(input_dir: &Path) -> Result<PathBuf> {
    resolve_datasource_export_root_dir(input_dir).map_err(|error| {
        message(
            error
                .to_string()
                .replace("--import-dir", "--input-dir")
                .replace("Datasource import", "Datasource inspect-export"),
        )
    })
}

pub(crate) fn resolve_datasource_inspect_export_input_format(
    input_dir: &Path,
    input_type: Option<DatasourceImportInputFormat>,
) -> Result<Option<DatasourceImportInputFormat>> {
    let input_dir = resolve_datasource_workspace_input_dir(input_dir)?;
    if let Some(input_type) = input_type {
        return Ok(Some(input_type));
    }
    if input_dir.is_file() {
        return Ok(Some(DatasourceImportInputFormat::Provisioning));
    }
    let has_inventory = has_datasource_inventory_export(&input_dir);
    let has_provisioning = has_datasource_provisioning_export(&input_dir);
    match (has_inventory, has_provisioning) {
        (true, true) => Ok(Some(prompt_datasource_inspect_export_input_format(
            &input_dir,
        )?)),
        (true, false) => Ok(Some(DatasourceImportInputFormat::Inventory)),
        (false, true) => Ok(Some(DatasourceImportInputFormat::Provisioning)),
        (false, false) => Ok(None),
    }
}

fn render_datasource_text(records: &[Map<String, Value>]) -> Vec<String> {
    let mut lines = Vec::new();
    for record in records {
        let mut line = format!(
            "- name={} type={} uid={}",
            string_field(record, "name", ""),
            string_field(record, "type", ""),
            string_field(record, "uid", "")
        );
        let url = string_field(record, "url", "");
        if !url.is_empty() {
            line.push_str(&format!(" url={url}"));
        }
        let is_default = string_field(record, "isDefault", "");
        if !is_default.is_empty() {
            line.push_str(&format!(" default={is_default}"));
        }
        let org = string_field(record, "org", "");
        let org_id = string_field(record, "orgId", "");
        if !org.is_empty() || !org_id.is_empty() {
            line.push_str(&format!(" org={} ({})", org, org_id));
        }
        lines.push(line);
    }
    lines
}

#[cfg(any(feature = "tui", test))]
fn build_datasource_inspect_export_browser_items(
    source: &DatasourceInspectExportSource,
) -> Vec<BrowserItem> {
    source
        .records
        .iter()
        .map(|record| {
            let name = string_field(record, "name", "");
            let datasource_type = string_field(record, "type", "");
            let uid = string_field(record, "uid", "");
            let url = string_field(record, "url", "");
            let org = string_field(record, "org", "");
            let org_id = string_field(record, "orgId", "");
            let is_default = string_field(record, "isDefault", "");
            BrowserItem {
                kind: "datasource".to_string(),
                title: name.clone(),
                meta: format!(
                    "type={} uid={} org={} ({}) default={}",
                    datasource_type, uid, org, org_id, is_default
                ),
                details: vec![
                    format!("Name: {name}"),
                    format!("Type: {datasource_type}"),
                    format!("UID: {uid}"),
                    format!("URL: {url}"),
                    format!("Default: {is_default}"),
                    format!("Org: {org} ({org_id})"),
                    format!("Input mode: {}", source.input_mode),
                    format!("Input path: {}", source.input_path),
                ],
            }
        })
        .collect()
}

fn datasource_inspect_export_record(record: &DatasourceImportRecord) -> Map<String, Value> {
    record.to_inventory_record()
}

pub(crate) fn resolve_datasource_inspect_export_format(
    args: &DatasourceInspectExportArgs,
) -> DatasourceInspectExportRenderFormat {
    if args.table {
        DatasourceInspectExportRenderFormat::Table
    } else if args.csv {
        DatasourceInspectExportRenderFormat::Csv
    } else if args.json {
        DatasourceInspectExportRenderFormat::Json
    } else if args.yaml {
        DatasourceInspectExportRenderFormat::Yaml
    } else {
        DatasourceInspectExportRenderFormat::Text
    }
}

pub(crate) fn load_datasource_inspect_export_source(
    input_dir: &Path,
    input_format: DatasourceImportInputFormat,
) -> Result<DatasourceInspectExportSource> {
    let input_dir = resolve_datasource_workspace_input_dir(input_dir)?;
    if input_format == DatasourceImportInputFormat::Provisioning && input_dir.is_file() {
        let extension = input_dir
            .as_path()
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !matches!(extension, "yaml" | "yml") {
            return Err(message(format!(
                "Datasource inspect-export input file must be YAML (.yaml or .yml): {}",
                input_dir.display()
            )));
        }
        let (metadata, records) =
            load_import_records(&input_dir, DatasourceImportInputFormat::Provisioning)?;
        return Ok(DatasourceInspectExportSource {
            input_mode: "provisioning",
            input_path: input_dir.display().to_string(),
            metadata: Some(build_datasource_inspect_export_metadata(Map::from_iter(
                vec![
                    (
                        "inputFile".to_string(),
                        Value::String(input_dir.display().to_string()),
                    ),
                    (
                        "datasourcesFile".to_string(),
                        Value::String(input_dir.display().to_string()),
                    ),
                    (
                        "schemaVersion".to_string(),
                        Value::Number(metadata.schema_version.into()),
                    ),
                    ("kind".to_string(), Value::String(metadata.kind)),
                    ("variant".to_string(), Value::String(metadata.variant)),
                    ("resource".to_string(), Value::String(metadata.resource)),
                ],
            ))),
            records: records
                .into_iter()
                .map(|record| datasource_inspect_export_record(&record))
                .collect(),
        });
    }

    let metadata_path = input_dir.join(EXPORT_METADATA_FILENAME);
    if input_format == DatasourceImportInputFormat::Inventory && metadata_path.is_file() {
        let metadata = load_json_object_file(&metadata_path, "Datasource export metadata")?;
        let (_, records) = load_datasource_inventory_records_from_export_root(&input_dir)?;
        return Ok(DatasourceInspectExportSource {
            input_mode: "inventory",
            input_path: input_dir.display().to_string(),
            metadata: Some(build_datasource_inspect_export_metadata(
                metadata.as_object().cloned().ok_or_else(|| {
                    message(format!(
                        "Datasource export metadata must be a JSON object: {}",
                        metadata_path.display()
                    ))
                })?,
            )),
            records: records
                .into_iter()
                .map(|record| datasource_inspect_export_record(&record))
                .collect(),
        });
    }

    let provisioning_candidates = datasource_inspect_provisioning_candidates(&input_dir);
    if provisioning_candidates
        .iter()
        .any(|candidate| candidate.is_file())
    {
        let (metadata, records) =
            load_import_records(&input_dir, DatasourceImportInputFormat::Provisioning)?;
        return Ok(DatasourceInspectExportSource {
            input_mode: "provisioning",
            input_path: input_dir.display().to_string(),
            metadata: Some(build_datasource_inspect_export_metadata(Map::from_iter(
                vec![
                    (
                        "inputDir".to_string(),
                        Value::String(input_dir.display().to_string()),
                    ),
                    (
                        "datasourcesFile".to_string(),
                        Value::String(metadata.datasources_file),
                    ),
                    (
                        "schemaVersion".to_string(),
                        Value::Number(metadata.schema_version.into()),
                    ),
                    ("kind".to_string(), Value::String(metadata.kind)),
                    ("variant".to_string(), Value::String(metadata.variant)),
                    ("resource".to_string(), Value::String(metadata.resource)),
                ],
            ))),
            records: records
                .into_iter()
                .map(|record| datasource_inspect_export_record(&record))
                .collect(),
        });
    }

    Err(message(format!(
        "Datasource inspect-export could not find export-metadata.json or provisioning/datasources.yaml under {}.",
        input_dir.display()
    )))
}

#[cfg(feature = "tui")]
fn datasource_centered_popup_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_width = area.width.saturating_sub(8).min(width).max(72);
    let popup_height = area.height.saturating_sub(4).min(height).max(12);
    Rect {
        x: area.x + area.width.saturating_sub(popup_width) / 2,
        y: area.y + area.height.saturating_sub(popup_height) / 2,
        width: popup_width,
        height: popup_height,
    }
}

#[cfg(feature = "tui")]
fn run_datasource_inspect_input_selector(input_dir: &Path) -> Result<DatasourceImportInputFormat> {
    let mut session = TerminalSession::enter()?;
    let options = [
        (
            DatasourceImportInputFormat::Inventory,
            "inventory",
            "Inspect datasource inventory export records",
        ),
        (
            DatasourceImportInputFormat::Provisioning,
            "provisioning",
            "Inspect provisioning datasource YAML",
        ),
    ];
    let mut selected = 0usize;
    loop {
        session.terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(Clear, area);
            let popup = datasource_centered_popup_rect(area, 88, 17);
            let inner = popup.inner(Margin {
                vertical: 1,
                horizontal: 3,
            });
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7),
                    Constraint::Length(5),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(inner);
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Datasource inspect input")
                    .border_style(Style::default().fg(Color::LightBlue))
                    .style(Style::default().bg(Color::Black)),
                popup,
            );
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(vec![
                        tui_shell::label("Title "),
                        tui_shell::accent("Choose datasource inspect mode", Color::Cyan),
                    ]),
                    Line::from(vec![
                        tui_shell::label("Input "),
                        tui_shell::plain(input_dir.display().to_string()),
                    ]),
                    Line::from(""),
                    Line::from(
                        "This path contains both datasource inventory and provisioning artifacts.",
                    ),
                    Line::from("Select one input mode before continuing into the browser."),
                ])
                .wrap(Wrap { trim: false }),
                chunks[0],
            );
            let items = options
                .iter()
                .enumerate()
                .map(|(index, (_, label, detail))| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{}. {label}", index + 1),
                            Style::default()
                                .fg(Color::LightCyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(format!("({detail})"), Style::default().fg(Color::White)),
                    ]))
                })
                .collect::<Vec<ListItem>>();
            let mut state = ListState::default().with_selected(Some(selected));
            frame.render_stateful_widget(
                List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Options")
                            .border_style(Style::default().fg(Color::Gray)),
                    )
                    .highlight_symbol("   ")
                    .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black)),
                chunks[1],
                &mut state,
            );
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(vec![
                        tui_shell::label("Choice "),
                        tui_shell::plain(format!("{}. {}", selected + 1, options[selected].1)),
                    ]),
                    Line::from(vec![
                        tui_shell::key_chip("Up/Down", Color::Blue),
                        Span::raw(" move  "),
                        tui_shell::key_chip("Enter", Color::Green),
                        Span::raw(" confirm  "),
                        tui_shell::key_chip("Esc/q", Color::DarkGray),
                        Span::raw(" cancel"),
                    ]),
                ]),
                chunks[3],
            );
        })?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1).min(options.len().saturating_sub(1));
            }
            KeyCode::Enter => return Ok(options[selected].0),
            KeyCode::Esc | KeyCode::Char('q') => {
                return Err(message("Datasource inspect-export selection cancelled."));
            }
            _ => {}
        }
    }
}

#[cfg(feature = "tui")]
fn prompt_datasource_inspect_export_input_format(
    input_dir: &Path,
) -> Result<DatasourceImportInputFormat> {
    if !datasource_inspect_uses_tty() {
        return Err(message(format!(
            "Datasource inspect-export found both inventory and provisioning artifacts under {}. Re-run with --input-type inventory or --input-type provisioning.",
            input_dir.display()
        )));
    }
    run_datasource_inspect_input_selector(input_dir)
}

#[cfg(not(feature = "tui"))]
fn prompt_datasource_inspect_export_input_format(
    input_dir: &Path,
) -> Result<DatasourceImportInputFormat> {
    if !datasource_inspect_uses_tty() {
        return Err(message(format!(
            "Datasource inspect-export found both inventory and provisioning artifacts under {}. Re-run with --input-type inventory or --input-type provisioning.",
            input_dir.display()
        )));
    }
    loop {
        println!("Title: Choose datasource inspect mode");
        println!("Input: {}", input_dir.display());
        println!();
        println!("1. inventory (Inspect datasource inventory export records)");
        println!("2. provisioning (Inspect provisioning datasource YAML)");
        print!("Choice [1-2/inventory/provisioning]: ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let input_type = match line.trim().to_ascii_lowercase().as_str() {
            "1" | "inventory" | "i" => Some(DatasourceImportInputFormat::Inventory),
            "2" | "provisioning" | "p" | "yaml" | "yml" => {
                Some(DatasourceImportInputFormat::Provisioning)
            }
            _ => None,
        };
        if let Some(input_type) = input_type {
            return Ok(input_type);
        }
        eprintln!("Enter 1, 2, inventory, or provisioning.");
    }
}

fn build_datasource_inspect_export_document(source: &DatasourceInspectExportSource) -> Value {
    let mut document = Map::from_iter(vec![
        (
            "inputPath".to_string(),
            Value::String(source.input_path.clone()),
        ),
        (
            "inputMode".to_string(),
            Value::String(source.input_mode.to_string()),
        ),
        (
            "datasourceCount".to_string(),
            Value::Number((source.records.len() as i64).into()),
        ),
        (
            "datasources".to_string(),
            Value::Array(source.records.iter().cloned().map(Value::Object).collect()),
        ),
    ]);
    if let Some(metadata) = &source.metadata {
        document.insert("metadata".to_string(), metadata.clone());
    }
    Value::Object(document)
}

pub(crate) fn render_datasource_inspect_export_output(
    source: &DatasourceInspectExportSource,
    format: DatasourceInspectExportRenderFormat,
) -> Result<String> {
    let mut output = String::new();
    let document = build_datasource_inspect_export_document(source);
    // JSON and YAML keep the full bundle contract; human-readable views stay summary-oriented.
    match format {
        DatasourceInspectExportRenderFormat::Text => {
            output.push_str(&format!(
                "Datasource inspect-export: {}\n",
                source.input_path
            ));
            output.push_str(&format!(
                "Layer: {}\n",
                datasource_inspect_export_output_layer(format)
            ));
            output.push_str(&format!("Mode: {}\n", source.input_mode));
            if let Some(metadata) = source.metadata.as_ref().and_then(Value::as_object) {
                if metadata
                    .get("masked")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    output.push_str("Bundle: recovery-capable masked export\n");
                }
                if let Some(kind) = metadata.get("kind").and_then(Value::as_str) {
                    output.push_str(&format!("Kind: {kind}\n"));
                }
                if let Some(variant) = metadata.get("variant").and_then(Value::as_str) {
                    output.push_str(&format!("Variant: {variant}\n"));
                }
                if let Some(resource) = metadata.get("resource").and_then(Value::as_str) {
                    output.push_str(&format!("Resource: {resource}\n"));
                }
                if let Some(datasources_file) =
                    metadata.get("datasourcesFile").and_then(Value::as_str)
                {
                    output.push_str(&format!("Datasources file: {datasources_file}\n"));
                }
            }
            output.push_str(&format!("Datasource count: {}\n", source.records.len()));
            output.push('\n');
            for line in render_data_source_table(&source.records, true) {
                output.push_str(&line);
                output.push('\n');
            }
        }
        DatasourceInspectExportRenderFormat::Table => {
            output.push_str(&format!(
                "Datasource inspect-export: {}\n",
                source.input_path
            ));
            output.push_str(&format!(
                "Layer: {}\n",
                datasource_inspect_export_output_layer(format)
            ));
            output.push_str(&format!("Mode: {}\n\n", source.input_mode));
            for line in render_data_source_table(&source.records, true) {
                output.push_str(&line);
                output.push('\n');
            }
        }
        DatasourceInspectExportRenderFormat::Csv => {
            for line in render_data_source_csv(&source.records) {
                output.push_str(&line);
                output.push('\n');
            }
        }
        DatasourceInspectExportRenderFormat::Json => {
            output.push_str(&render_json_value(&document)?);
        }
        DatasourceInspectExportRenderFormat::Yaml => {
            output.push_str(&render_yaml(&document)?);
        }
    }
    Ok(output)
}

fn datasource_inspect_export_output_layer(
    format: DatasourceInspectExportRenderFormat,
) -> &'static str {
    match format {
        DatasourceInspectExportRenderFormat::Text
        | DatasourceInspectExportRenderFormat::Table
        | DatasourceInspectExportRenderFormat::Csv => "operator-summary",
        DatasourceInspectExportRenderFormat::Json | DatasourceInspectExportRenderFormat::Yaml => {
            "full-contract"
        }
    }
}

fn render_diff_identity(entry: &DatasourceDiffEntry) -> String {
    if let Some(record) = &entry.export_record {
        if !record.uid.is_empty() {
            return format!("uid={} name={}", record.uid, record.name);
        }
        return format!("name={}", record.name);
    }
    if let Some(record) = &entry.live_record {
        if !record.uid.is_empty() {
            return format!("uid={} name={}", record.uid, record.name);
        }
        return format!("name={}", record.name);
    }
    entry.key.clone()
}

// Render the diff as an operator-summary report rather than a machine contract.
fn print_datasource_diff_summary_report(report: &DatasourceDiffReport) {
    for entry in &report.entries {
        let identity = render_diff_identity(entry);
        match entry.status {
            DatasourceDiffStatus::Matches => {
                println!("Diff same datasource {identity}");
            }
            DatasourceDiffStatus::Different => {
                let changed_fields = entry
                    .differences
                    .iter()
                    .map(|item| item.field)
                    .collect::<Vec<&str>>()
                    .join(",");
                println!("Diff different datasource {identity} fields={changed_fields}");
            }
            DatasourceDiffStatus::MissingInLive => {
                println!("Diff missing-live datasource {identity}");
            }
            DatasourceDiffStatus::MissingInExport => {
                println!("Diff extra-live datasource {identity}");
            }
            DatasourceDiffStatus::AmbiguousLiveMatch => {
                println!("Diff ambiguous-live datasource {identity}");
            }
        }
    }
}

/// Purpose: implementation note.
pub(crate) fn diff_datasources_with_live(
    diff_dir: &Path,
    input_format: DatasourceImportInputFormat,
    live: &[Map<String, Value>],
) -> Result<(usize, usize)> {
    let export_values = load_diff_record_values(diff_dir, input_format)?;
    let live_values = live
        .iter()
        .cloned()
        .map(Value::Object)
        .collect::<Vec<Value>>();
    let report = build_datasource_diff_report(
        &normalize_export_records(&export_values),
        &normalize_live_records(&live_values),
    );
    print_datasource_diff_summary_report(&report);
    let difference_count = report.summary.compared_count - report.summary.matches_count;
    println!(
        "Diff checked {} datasource(s); {} difference(s) found.",
        report.summary.compared_count, difference_count
    );
    Ok((report.summary.compared_count, difference_count))
}

/// Datasource runtime entrypoint.
///
/// After command normalization, this function builds required clients, validates constraints
/// for output mode flags, and delegates execution to list/export/import/diff handlers.
pub fn run_datasource_cli(command: DatasourceGroupCommand) -> Result<()> {
    // Call graph (hierarchy): this function is used in related modules.
    // Upstream callers: datasource_rust_tests.rs:datasource_import_rejects_output_columns_without_table_output, datasource_rust_tests.rs:datasource_import_with_use_export_org_requires_basic_auth
    // Downstream callees: common.rs:message, common.rs:write_json_file, dashboard_cli_defs.rs:build_http_client_for_org, dashboard_live.rs:list_datasources, datasource.rs:build_add_payload, datasource.rs:build_all_orgs_export_index, datasource.rs:build_all_orgs_export_metadata, datasource.rs:build_all_orgs_output_dir, datasource.rs:build_datasource_export_metadata, datasource.rs:build_export_index, datasource.rs:build_export_records, datasource.rs:build_list_records ...

    let command = normalize_datasource_group_command(command);
    match command {
        DatasourceGroupCommand::Types(args) => {
            match args.output_format {
                SimpleOutputFormat::Text => {
                    for line in render_supported_datasource_catalog_text() {
                        println!("{line}");
                    }
                }
                SimpleOutputFormat::Table => {
                    for line in render_supported_datasource_catalog_table() {
                        println!("{line}");
                    }
                }
                SimpleOutputFormat::Csv => {
                    for line in render_supported_datasource_catalog_csv() {
                        println!("{line}");
                    }
                }
                SimpleOutputFormat::Json => {
                    print!(
                        "{}",
                        render_json_value(&render_supported_datasource_catalog_json())?
                    );
                }
                SimpleOutputFormat::Yaml => {
                    print!("{}", render_supported_datasource_catalog_yaml()?);
                }
            }
            Ok(())
        }
        DatasourceGroupCommand::List(args) => {
            let datasources = if args.all_orgs {
                let context = build_auth_context(&args.common)?;
                if context.auth_mode != "basic" {
                    return Err(message(
                        "Datasource list with --all-orgs requires Basic auth (--basic-user / --basic-password).",
                    ));
                }
                let admin_client = build_http_client(&args.common)?;
                let mut rows = Vec::new();
                for org in list_orgs(&admin_client)? {
                    let org_id = org
                        .get("id")
                        .and_then(Value::as_i64)
                        .ok_or_else(|| message("Grafana org list entry is missing numeric id."))?;
                    let org_client = build_http_client_for_org(&args.common, org_id)?;
                    rows.extend(build_list_records(&org_client)?);
                }
                rows.sort_by(|left, right| {
                    let left_org_id = string_field(left, "orgId", "");
                    let right_org_id = string_field(right, "orgId", "");
                    left_org_id
                        .cmp(&right_org_id)
                        .then_with(|| {
                            string_field(left, "name", "").cmp(&string_field(right, "name", ""))
                        })
                        .then_with(|| {
                            string_field(left, "uid", "").cmp(&string_field(right, "uid", ""))
                        })
                });
                rows
            } else if args.org_id.is_some() {
                let client = resolve_target_client(&args.common, args.org_id)?;
                build_list_records(&client)?
            } else {
                let client = build_http_client(&args.common)?;
                list_datasources(&client)?
            };
            if args.json {
                print!(
                    "{}",
                    render_json_value(&render_data_source_json(&datasources))?
                );
            } else if args.yaml {
                print!("{}", render_yaml(&render_data_source_json(&datasources))?);
            } else if args.csv {
                for line in render_data_source_csv(&datasources) {
                    println!("{line}");
                }
            } else if args.text {
                for line in render_datasource_text(&datasources) {
                    println!("{line}");
                }
                println!();
                println!("Listed {} data source(s).", datasources.len());
            } else {
                for line in render_data_source_table(&datasources, !args.no_header) {
                    println!("{line}");
                }
                println!();
                println!("Listed {} data source(s).", datasources.len());
            }
            Ok(())
        }
        DatasourceGroupCommand::Browse(args) => {
            let _ = datasource_browse::browse_datasources(&args)?;
            Ok(())
        }
        DatasourceGroupCommand::InspectExport(args) => {
            let input_format = resolve_datasource_inspect_export_input_format(
                &args.input_dir,
                args.input_type,
            )?
            .ok_or_else(|| {
                message(format!(
                    "Datasource inspect-export could not find export-metadata.json or provisioning/datasources.yaml under {}.",
                    args.input_dir.display()
                ))
            })?;
            if args.interactive {
                #[cfg(feature = "tui")]
                {
                    let source =
                        load_datasource_inspect_export_source(&args.input_dir, input_format)?;
                    let summary_lines = vec![
                        "Datasource inspect-export".to_string(),
                        format!("Input: {}", source.input_path),
                        format!("Mode: {}", source.input_mode),
                        format!("Datasources: {}", source.records.len()),
                    ];
                    let items = build_datasource_inspect_export_browser_items(&source);
                    return run_interactive_browser(
                        "Datasource inspect-export",
                        &summary_lines,
                        &items,
                    );
                }
                #[cfg(not(feature = "tui"))]
                {
                    return Err(crate::common::tui(
                        "Datasource inspect-export interactive mode requires the `tui` feature.",
                    ));
                }
            }
            let source = load_datasource_inspect_export_source(&args.input_dir, input_format)?;
            let format = resolve_datasource_inspect_export_format(&args);
            let rendered = render_datasource_inspect_export_output(&source, format)?;
            print!("{rendered}");
            Ok(())
        }
        DatasourceGroupCommand::Add(args) => {
            validate_live_mutation_dry_run_args(
                args.table,
                args.json,
                args.dry_run,
                args.no_header,
                "add",
            )?;
            let payload = build_add_payload(&args)?;
            let client = build_http_client(&args.common)?;
            let live = list_datasources(&client)?;
            let matching =
                resolve_live_mutation_match(args.uid.as_deref(), Some(&args.name), &live);
            let row = vec![
                "add".to_string(),
                args.uid.clone().unwrap_or_default(),
                args.name.clone(),
                args.datasource_type.clone(),
                matching.destination.to_string(),
                matching.action.to_string(),
                matching
                    .target_id
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            ];
            if args.dry_run {
                if args.json {
                    print!("{}", render_json_value(&render_live_mutation_json(&[row]))?);
                } else if args.table {
                    for line in render_live_mutation_table(&[row], !args.no_header) {
                        println!("{line}");
                    }
                    println!("Dry-run checked 1 datasource add request");
                } else {
                    println!(
                        "Dry-run datasource add uid={} name={} match={} action={}",
                        args.uid.clone().unwrap_or_default(),
                        args.name,
                        matching.destination,
                        matching.action
                    );
                    println!("Dry-run checked 1 datasource add request");
                }
                return Ok(());
            }
            if matching.action != "would-create" {
                return Err(message(format!(
                    "Datasource add blocked for name={} uid={}: destination={} action={}.",
                    args.name,
                    args.uid.clone().unwrap_or_default(),
                    matching.destination,
                    matching.action
                )));
            }
            client.request_json(Method::POST, "/api/datasources", &[], Some(&payload))?;
            println!(
                "Created datasource uid={} name={}",
                args.uid.unwrap_or_default(),
                args.name
            );
            Ok(())
        }
        DatasourceGroupCommand::Modify(args) => {
            validate_live_mutation_dry_run_args(
                args.table,
                args.json,
                args.dry_run,
                args.no_header,
                "modify",
            )?;
            let updates = build_modify_updates(&args)?;
            let client = build_http_client(&args.common)?;
            let existing = fetch_datasource_by_uid_if_exists(&client, &args.uid)?;
            let (action, destination, payload, name, datasource_type, target_id) =
                if let Some(existing) = existing {
                    let payload = build_modify_payload(&existing, &updates)?;
                    (
                        "would-update",
                        "exists-uid",
                        Some(payload),
                        string_field(&existing, "name", ""),
                        string_field(&existing, "type", ""),
                        existing.get("id").and_then(Value::as_i64),
                    )
                } else {
                    (
                        "would-fail-missing",
                        "missing",
                        None,
                        String::new(),
                        String::new(),
                        None,
                    )
                };
            let row = vec![
                "modify".to_string(),
                args.uid.clone(),
                name.clone(),
                datasource_type.clone(),
                destination.to_string(),
                action.to_string(),
                target_id.map(|id| id.to_string()).unwrap_or_default(),
            ];
            if args.dry_run {
                if args.json {
                    print!("{}", render_json_value(&render_live_mutation_json(&[row]))?);
                } else if args.table {
                    for line in render_live_mutation_table(&[row], !args.no_header) {
                        println!("{line}");
                    }
                    println!("Dry-run checked 1 datasource modify request");
                } else {
                    println!(
                        "Dry-run datasource modify uid={} name={} match={} action={}",
                        args.uid, name, destination, action
                    );
                    println!("Dry-run checked 1 datasource modify request");
                }
                return Ok(());
            }
            if action != "would-update" {
                return Err(message(format!(
                    "Datasource modify blocked for uid={}: destination={} action={}.",
                    args.uid, destination, action
                )));
            }
            let payload =
                payload.ok_or_else(|| message("Datasource modify did not build a payload."))?;
            let target_id = target_id
                .ok_or_else(|| message("Datasource modify requires a live datasource id."))?;
            client.request_json(
                Method::PUT,
                &format!("/api/datasources/{target_id}"),
                &[],
                Some(&payload),
            )?;
            println!(
                "Modified datasource uid={} name={} id={}",
                args.uid, name, target_id
            );
            Ok(())
        }
        DatasourceGroupCommand::Delete(args) => {
            validate_live_mutation_dry_run_args(
                args.table,
                args.json,
                args.dry_run,
                args.no_header,
                "delete",
            )?;
            let client = build_http_client(&args.common)?;
            let live = list_datasources(&client)?;
            let matching = resolve_delete_match(args.uid.as_deref(), args.name.as_deref(), &live);
            let row = vec![
                "delete".to_string(),
                args.uid
                    .clone()
                    .or_else(|| {
                        if matching.target_uid.is_empty() {
                            None
                        } else {
                            Some(matching.target_uid.clone())
                        }
                    })
                    .unwrap_or_default(),
                args.name
                    .clone()
                    .unwrap_or_else(|| matching.target_name.clone()),
                String::new(),
                matching.destination.to_string(),
                matching.action.to_string(),
                matching
                    .target_id
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            ];
            if args.dry_run {
                if args.json {
                    print!("{}", render_json_value(&render_live_mutation_json(&[row]))?);
                } else if args.table {
                    for line in render_live_mutation_table(&[row], !args.no_header) {
                        println!("{line}");
                    }
                    println!("Dry-run checked 1 datasource delete request");
                } else {
                    println!(
                        "Dry-run datasource delete uid={} name={} match={} action={}",
                        args.uid.clone().unwrap_or_default(),
                        args.name.clone().unwrap_or_default(),
                        matching.destination,
                        matching.action
                    );
                    println!("Dry-run checked 1 datasource delete request");
                }
                return Ok(());
            }
            if !args.yes {
                return Err(message(
                    "Datasource delete requires --yes unless --dry-run is set.",
                ));
            }
            if matching.action != "would-delete" {
                return Err(message(format!(
                    "Datasource delete blocked for uid={} name={}: destination={} action={}.",
                    args.uid.clone().unwrap_or_default(),
                    args.name.clone().unwrap_or_default(),
                    matching.destination,
                    matching.action
                )));
            }
            let target_id = matching
                .target_id
                .ok_or_else(|| message("Datasource delete requires a live datasource id."))?;
            client.request_json(
                Method::DELETE,
                &format!("/api/datasources/{target_id}"),
                &[],
                None,
            )?;
            println!(
                "Deleted datasource uid={} name={} id={}",
                if matching.target_uid.is_empty() {
                    args.uid.unwrap_or_default()
                } else {
                    matching.target_uid
                },
                if matching.target_name.is_empty() {
                    args.name.unwrap_or_default()
                } else {
                    matching.target_name
                },
                target_id
            );
            Ok(())
        }
        DatasourceGroupCommand::Export(args) => {
            if args.all_orgs {
                let context = build_auth_context(&args.common)?;
                if context.auth_mode != "basic" {
                    return Err(message(
                        "Datasource export with --all-orgs requires Basic auth (--basic-user / --basic-password).",
                    ));
                }
                let admin_client = build_http_client(&args.common)?;
                let mut total = 0usize;
                let mut org_count = 0usize;
                let mut root_items = Vec::new();
                let mut root_records = Vec::new();
                for org in list_orgs(&admin_client)? {
                    let org_id = org
                        .get("id")
                        .and_then(Value::as_i64)
                        .ok_or_else(|| message("Grafana org list entry is missing numeric id."))?;
                    let org_client = build_http_client_for_org(&args.common, org_id)?;
                    let records = build_export_records(&org_client)?;
                    let scoped_output_dir = build_all_orgs_output_dir(&args.export_dir, &org);
                    let datasources_path = scoped_output_dir.join(DATASOURCE_EXPORT_FILENAME);
                    let index_path = scoped_output_dir.join("index.json");
                    let metadata_path = scoped_output_dir.join(EXPORT_METADATA_FILENAME);
                    let provisioning_path = scoped_output_dir
                        .join(DATASOURCE_PROVISIONING_SUBDIR)
                        .join(DATASOURCE_PROVISIONING_FILENAME);
                    if !args.dry_run {
                        write_json_file(
                            &datasources_path,
                            &Value::Array(records.clone().into_iter().map(Value::Object).collect()),
                            args.overwrite,
                        )?;
                        write_json_file(
                            &index_path,
                            &build_export_index(&records),
                            args.overwrite,
                        )?;
                        write_json_file(
                            &metadata_path,
                            &build_datasource_export_metadata(records.len()),
                            args.overwrite,
                        )?;
                        if !args.without_datasource_provisioning {
                            write_yaml_file(
                                &provisioning_path,
                                &build_datasource_provisioning_document(&records),
                                args.overwrite,
                            )?;
                        }
                    }
                    let summary_verb = if args.dry_run {
                        "Would export"
                    } else {
                        "Exported"
                    };
                    println!(
                        "{summary_verb} {} datasource(s). Datasources: {} Index: {} Manifest: {}{}",
                        records.len(),
                        datasources_path.display(),
                        index_path.display(),
                        metadata_path.display(),
                        if args.without_datasource_provisioning {
                            String::new()
                        } else {
                            format!(" Provisioning: {}", provisioning_path.display())
                        }
                    );
                    for item in build_export_index(&records)
                        .get("items")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                    {
                        if let Some(object) = item.as_object() {
                            let mut entry = object.clone();
                            entry.insert(
                                "exportDir".to_string(),
                                Value::String(scoped_output_dir.display().to_string()),
                            );
                            root_items.push(entry);
                        }
                    }
                    root_records.extend(records.iter().cloned());
                    total += records.len();
                    org_count += 1;
                }
                if !args.dry_run {
                    write_json_file(
                        &args.export_dir.join("index.json"),
                        &build_all_orgs_export_index(&root_items),
                        args.overwrite,
                    )?;
                    write_json_file(
                        &args.export_dir.join(EXPORT_METADATA_FILENAME),
                        &build_all_orgs_export_metadata(org_count, total),
                        args.overwrite,
                    )?;
                    if !args.without_datasource_provisioning {
                        write_yaml_file(
                            &args
                                .export_dir
                                .join(DATASOURCE_PROVISIONING_SUBDIR)
                                .join(DATASOURCE_PROVISIONING_FILENAME),
                            &build_datasource_provisioning_document(&root_records),
                            args.overwrite,
                        )?;
                    }
                }
                println!(
                    "{} datasource(s) across {} exported org(s) under {}",
                    total,
                    org_count,
                    args.export_dir.display()
                );
                return Ok(());
            }
            let client = resolve_target_client(&args.common, args.org_id)?;
            export_datasource_scope(
                &client,
                &args.export_dir,
                args.overwrite,
                args.dry_run,
                !args.without_datasource_provisioning,
            )?;
            Ok(())
        }
        DatasourceGroupCommand::Import(args) => {
            validate_import_org_auth(&args.common, &args)?;
            if args.table && !args.dry_run {
                return Err(message(
                    "--table is only supported with --dry-run for datasource import.",
                ));
            }
            if args.json && !args.dry_run {
                return Err(message(
                    "--json is only supported with --dry-run for datasource import.",
                ));
            }
            if args.table && args.json {
                return Err(message(
                    "--table and --json are mutually exclusive for datasource import.",
                ));
            }
            if args.no_header && !args.table {
                return Err(message(
                    "--no-header is only supported with --dry-run --table for datasource import.",
                ));
            }
            if !args.output_columns.is_empty() && !args.table {
                return Err(message(
                    "--output-columns is only supported with --dry-run --table or table-like --output-format for datasource import.",
                ));
            }
            if args.use_export_org {
                if !args.output_columns.is_empty() {
                    return Err(message(
                        "--output-columns is not supported with --use-export-org for datasource import.",
                    ));
                }
                import_datasources_by_export_org(&args)?;
                return Ok(());
            }
            let client = resolve_target_client(&args.common, args.org_id)?;
            import_datasources_with_client(&client, &args)?;
            Ok(())
        }
        DatasourceGroupCommand::Diff(args) => {
            let client = build_http_client(&args.common)?;
            let live = list_datasources(&client)?;
            let (compared_count, differences) =
                diff_datasources_with_live(&args.diff_dir, args.input_format, &live)?;
            if differences > 0 {
                return Err(message(format!(
                    "Found {} datasource difference(s) across {} exported datasource(s).",
                    differences, compared_count
                )));
            }
            println!(
                "No datasource differences across {} exported datasource(s).",
                compared_count
            );
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "datasource_rust_tests.rs"]
mod datasource_rust_tests;

#[cfg(test)]
#[path = "datasource_diff_rust_tests.rs"]
mod datasource_diff_rust_tests;
