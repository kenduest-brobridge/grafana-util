use serde_json::{json, Value};

use super::contracts::{
    ActionDescriptor, ActionFieldDescriptor, CapabilityDescriptor, FieldOptionDescriptor,
    WorkspaceDescriptor,
};
use crate::dashboard::DEFAULT_PAGE_SIZE;

fn text_field(
    id: &'static str,
    label: &'static str,
    kind: &'static str,
    help: &'static str,
    placeholder: &'static str,
    default_value: Value,
    required: bool,
) -> ActionFieldDescriptor {
    ActionFieldDescriptor {
        id,
        label,
        kind,
        required,
        help,
        placeholder,
        default_value,
        options: Vec::new(),
    }
}

fn select_field(
    id: &'static str,
    label: &'static str,
    help: &'static str,
    default_value: &'static str,
    options: Vec<FieldOptionDescriptor>,
) -> ActionFieldDescriptor {
    ActionFieldDescriptor {
        id,
        label,
        kind: "select",
        required: false,
        help,
        placeholder: "",
        default_value: Value::String(default_value.to_string()),
        options,
    }
}

fn checkbox_field(
    id: &'static str,
    label: &'static str,
    help: &'static str,
    default_value: bool,
) -> ActionFieldDescriptor {
    ActionFieldDescriptor {
        id,
        label,
        kind: "checkbox",
        required: false,
        help,
        placeholder: "",
        default_value: Value::Bool(default_value),
        options: Vec::new(),
    }
}

fn dashboard_fields_for_browse() -> Vec<ActionFieldDescriptor> {
    vec![
        text_field(
            "pageSize",
            "Page Size",
            "number",
            "Dashboard search page size for the live inventory pull.",
            "500",
            json!(DEFAULT_PAGE_SIZE),
            false,
        ),
        text_field(
            "orgId",
            "Org ID",
            "number",
            "Optional explicit Grafana org scope.",
            "2",
            Value::Null,
            false,
        ),
        checkbox_field(
            "allOrgs",
            "All Orgs",
            "Aggregate across all visible orgs. Prefer Basic auth for this.",
            false,
        ),
        checkbox_field(
            "withSources",
            "With Sources",
            "Resolve datasource usage for each dashboard row.",
            true,
        ),
        text_field(
            "query",
            "Search",
            "text",
            "Optional client-side browse filter across uid, title, folder, org, and sources.",
            "cpu main",
            Value::Null,
            false,
        ),
        text_field(
            "path",
            "Folder Path",
            "text",
            "Optional folder subtree to focus, for example Platform / Infra.",
            "Platform / Infra",
            Value::Null,
            false,
        ),
    ]
}

fn dashboard_fields_for_list() -> Vec<ActionFieldDescriptor> {
    let mut fields = dashboard_fields_for_browse();
    fields.push(text_field(
        "outputColumns",
        "Output Columns",
        "text",
        "Optional comma-separated subset such as uid,name,path,org,sources.",
        "uid,name,path,org,sources",
        Value::Null,
        false,
    ));
    fields.push(select_field(
        "outputMode",
        "Output Mode",
        "Choose the list preview format.",
        "table",
        vec![
            FieldOptionDescriptor {
                value: "table",
                label: "Table",
            },
            FieldOptionDescriptor {
                value: "csv",
                label: "CSV",
            },
            FieldOptionDescriptor {
                value: "json",
                label: "JSON",
            },
        ],
    ));
    fields
}

fn dashboard_fields_for_inspect() -> Vec<ActionFieldDescriptor> {
    vec![
        text_field("pageSize", "Page Size", "number", "Dashboard search page size.", "500", json!(DEFAULT_PAGE_SIZE), false),
        text_field("concurrency", "Concurrency", "number", "Live inspect worker pool size.", "8", json!(8), false),
        text_field("orgId", "Org ID", "number", "Optional explicit Grafana org scope.", "2", Value::Null, false),
        checkbox_field("allOrgs", "All Orgs", "Inspect across all visible orgs.", false),
        select_field(
            "outputMode",
            "Output Mode",
            "Choose summary, query report, dependency, or governance output.",
            "summary",
            vec![
                FieldOptionDescriptor { value: "summary", label: "Summary" },
                FieldOptionDescriptor { value: "table", label: "Table" },
                FieldOptionDescriptor { value: "json", label: "JSON" },
                FieldOptionDescriptor { value: "report-table", label: "Report Table" },
                FieldOptionDescriptor { value: "report-csv", label: "Report CSV" },
                FieldOptionDescriptor { value: "report-json", label: "Report JSON" },
                FieldOptionDescriptor { value: "report-tree", label: "Report Tree" },
                FieldOptionDescriptor { value: "report-tree-table", label: "Report Tree Table" },
                FieldOptionDescriptor { value: "dependency", label: "Dependency" },
                FieldOptionDescriptor { value: "dependency-json", label: "Dependency JSON" },
                FieldOptionDescriptor { value: "governance", label: "Governance" },
                FieldOptionDescriptor { value: "governance-json", label: "Governance JSON" },
            ],
        ),
        text_field("reportColumns", "Report Columns", "text", "Optional comma-separated report columns for report-table/report-csv/report-tree-table.", "dashboard_uid,dashboard_title,panel_id,datasource,datasource_family,query", Value::Null, false),
        text_field("reportFilterDatasource", "Datasource Filter", "text", "Optional exact datasource label, uid, type, or family filter.", "prometheus", Value::Null, false),
        text_field("reportFilterPanelId", "Panel Filter", "text", "Optional exact panel id filter.", "7", Value::Null, false),
        checkbox_field("progress", "Progress", "Show live inspect progress while staging dashboards.", false),
    ]
}

pub(crate) fn workspaces() -> Vec<WorkspaceDescriptor> {
    vec![
        WorkspaceDescriptor {
            id: "dashboard",
            title: "Dashboard",
            description: "Browse dashboards first, then switch into list or inspect tasks when you need more precision.",
            actions: vec![
                ActionDescriptor {
                    id: "dashboard-browse",
                    title: "Browse",
                    description: "Folder-aware dashboard browser for org, folder, and datasource-oriented exploration.",
                    ui_mode: "browse",
                    method: "POST",
                    path: "/api/1.0/dashboard/browse",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"pageSize": 500, "withSources": true}),
                    fields: dashboard_fields_for_browse(),
                },
                ActionDescriptor {
                    id: "dashboard-list",
                    title: "List",
                    description: "Read-only dashboard inventory with table, CSV, or JSON projection.",
                    ui_mode: "table",
                    method: "POST",
                    path: "/api/1.0/dashboard/list",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"pageSize": 500, "withSources": true, "outputMode": "table"}),
                    fields: dashboard_fields_for_list(),
                },
                ActionDescriptor {
                    id: "dashboard-inspect-live",
                    title: "Inspect Live",
                    description: "Stage a live dashboard export, then reuse the offline inspect and governance builders.",
                    ui_mode: "analysis",
                    method: "POST",
                    path: "/api/1.0/dashboard/inspect-live",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"pageSize": 500, "concurrency": 8, "outputMode": "governance-json"}),
                    fields: dashboard_fields_for_inspect(),
                },
                ActionDescriptor {
                    id: "dashboard-inspect-export",
                    title: "Inspect Export",
                    description: "Inspect one local raw export directory with the same summary, report, dependency, and governance builders.",
                    ui_mode: "analysis",
                    method: "POST",
                    path: "/api/1.0/dashboard/inspect-export",
                    read_only: true,
                    requires_connection: false,
                    example_request: json!({"importDir": "./dashboards/raw", "outputMode": "dependency-json"}),
                    fields: {
                        let mut fields = dashboard_fields_for_inspect();
                        fields.insert(0, text_field("importDir", "Import Dir", "path", "Raw export directory to analyze.", "./dashboards/raw", json!("./dashboards/raw"), true));
                        fields
                    },
                },
                ActionDescriptor {
                    id: "dashboard-inspect-vars",
                    title: "Inspect Vars",
                    description: "Resolve dashboard templating variables from one UID or full dashboard URL.",
                    ui_mode: "table",
                    method: "POST",
                    path: "/api/1.0/dashboard/inspect-vars",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"dashboardUid": "cpu-main", "outputMode": "table"}),
                    fields: vec![
                        text_field("dashboardUid", "Dashboard UID", "text", "UID to inspect. Leave blank when using Dashboard URL instead.", "cpu-main", Value::Null, false),
                        text_field("dashboardUrl", "Dashboard URL", "text", "Full Grafana dashboard URL as an alternative to UID.", "http://localhost:3000/d/cpu-main/cpu-main", Value::Null, false),
                        text_field("varsQuery", "Vars Query", "text", "Optional var- query string overlay.", "var-env=prod&var-cluster=core", Value::Null, false),
                        text_field("orgId", "Org ID", "number", "Optional explicit org scope.", "2", Value::Null, false),
                        select_field(
                            "outputMode",
                            "Output Mode",
                            "Choose table, CSV, or JSON variable output.",
                            "table",
                            vec![
                                FieldOptionDescriptor { value: "table", label: "Table" },
                                FieldOptionDescriptor { value: "csv", label: "CSV" },
                                FieldOptionDescriptor { value: "json", label: "JSON" },
                            ],
                        ),
                    ],
                },
            ],
        },
        WorkspaceDescriptor {
            id: "project",
            title: "Project",
            description: "Project-wide staged and live status views over overview and project-status builders.",
            actions: vec![
                ActionDescriptor {
                    id: "overview-staged",
                    title: "Overview Staged",
                    description: "Summarize staged exports and handoff artifacts in one overview document.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/overview/staged",
                    read_only: true,
                    requires_connection: false,
                    example_request: json!({"dashboardExportDir": "./dashboards/raw", "datasourceExportDir": "./datasources", "desiredFile": "./desired.json"}),
                    fields: vec![
                        text_field("dashboardExportDir", "Dashboard Export Dir", "path", "Optional dashboard raw export dir.", "./dashboards/raw", Value::Null, false),
                        text_field("datasourceExportDir", "Datasource Export Dir", "path", "Optional datasource export dir.", "./datasources", Value::Null, false),
                        text_field("alertExportDir", "Alert Export Dir", "path", "Optional alert export dir.", "./alerts", Value::Null, false),
                        text_field("desiredFile", "Desired File", "path", "Optional sync desired file.", "./desired.json", Value::Null, false),
                        text_field("sourceBundle", "Source Bundle", "path", "Optional sync source bundle.", "./sync-source-bundle.json", Value::Null, false),
                        text_field("targetInventory", "Target Inventory", "path", "Optional target inventory file.", "./target-inventory.json", Value::Null, false),
                        text_field("availabilityFile", "Availability File", "path", "Optional availability mapping.", "./availability.json", Value::Null, false),
                        text_field("mappingFile", "Mapping File", "path", "Optional promotion mapping file.", "./mapping.json", Value::Null, false),
                    ],
                },
                ActionDescriptor {
                    id: "project-status-staged",
                    title: "Project Status Staged",
                    description: "Build the staged cross-domain readiness document.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/project-status/staged",
                    read_only: true,
                    requires_connection: false,
                    example_request: json!({"dashboardExportDir": "./dashboards/raw", "desiredFile": "./desired.json"}),
                    fields: vec![
                        text_field("dashboardExportDir", "Dashboard Export Dir", "path", "Optional dashboard raw export dir.", "./dashboards/raw", Value::Null, false),
                        text_field("datasourceExportDir", "Datasource Export Dir", "path", "Optional datasource export dir.", "./datasources", Value::Null, false),
                        text_field("alertExportDir", "Alert Export Dir", "path", "Optional alert export dir.", "./alerts", Value::Null, false),
                        text_field("desiredFile", "Desired File", "path", "Optional sync desired file.", "./desired.json", Value::Null, false),
                        text_field("sourceBundle", "Source Bundle", "path", "Optional sync source bundle.", "./sync-source-bundle.json", Value::Null, false),
                        text_field("targetInventory", "Target Inventory", "path", "Optional target inventory file.", "./target-inventory.json", Value::Null, false),
                        text_field("availabilityFile", "Availability File", "path", "Optional availability mapping.", "./availability.json", Value::Null, false),
                        text_field("mappingFile", "Mapping File", "path", "Optional promotion mapping file.", "./mapping.json", Value::Null, false),
                    ],
                },
                ActionDescriptor {
                    id: "project-status-live",
                    title: "Project Status Live",
                    description: "Read current Grafana status and optionally merge staged sync evidence.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/project-status/live",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"allOrgs": false}),
                    fields: vec![
                        text_field("orgId", "Org ID", "number", "Optional explicit org scope.", "2", Value::Null, false),
                        checkbox_field("allOrgs", "All Orgs", "Aggregate across all visible orgs.", false),
                        text_field("syncSummaryFile", "Sync Summary File", "path", "Optional sync summary file.", "./sync-summary.json", Value::Null, false),
                        text_field("bundlePreflightFile", "Bundle Preflight File", "path", "Optional bundle preflight file.", "./bundle-preflight.json", Value::Null, false),
                        text_field("promotionSummaryFile", "Promotion Summary File", "path", "Optional promotion summary file.", "./promotion-summary.json", Value::Null, false),
                        text_field("mappingFile", "Mapping File", "path", "Optional promotion mapping file.", "./mapping.json", Value::Null, false),
                        text_field("availabilityFile", "Availability File", "path", "Optional availability file.", "./availability.json", Value::Null, false),
                    ],
                },
            ],
        },
        WorkspaceDescriptor {
            id: "sync",
            title: "Sync",
            description: "Read-only sync review and preflight flows backed by the shared sync builders.",
            actions: vec![
                ActionDescriptor {
                    id: "sync-summary",
                    title: "Summary",
                    description: "Summarize desired sync resources from one local desired file.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/sync/summary",
                    read_only: true,
                    requires_connection: false,
                    example_request: json!({"desiredFile": "./desired.json"}),
                    fields: vec![text_field("desiredFile", "Desired File", "path", "Desired sync input file.", "./desired.json", json!("./desired.json"), true)],
                },
                ActionDescriptor {
                    id: "sync-plan",
                    title: "Plan",
                    description: "Compare desired state against live or staged live input.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/sync/plan",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"desiredFile": "./desired.json", "liveFile": "./live.json"}),
                    fields: vec![
                        text_field("desiredFile", "Desired File", "path", "Desired sync input file.", "./desired.json", json!("./desired.json"), true),
                        text_field("liveFile", "Live File", "path", "Optional staged live inventory file.", "./live.json", Value::Null, false),
                        checkbox_field("fetchLive", "Fetch Live", "Fetch live inventory instead of using a staged live file.", false),
                        text_field("orgId", "Org ID", "number", "Optional explicit org scope.", "2", Value::Null, false),
                        text_field("pageSize", "Page Size", "number", "Live fetch page size.", "500", json!(DEFAULT_PAGE_SIZE), false),
                        checkbox_field("allowPrune", "Allow Prune", "Allow prune actions in the generated plan.", false),
                        text_field("traceId", "Trace ID", "text", "Optional operator trace id.", "change-123", Value::Null, false),
                    ],
                },
                ActionDescriptor {
                    id: "sync-preflight",
                    title: "Preflight",
                    description: "Evaluate availability and placeholder readiness before mutation.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/sync/preflight",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"desiredFile": "./desired.json", "availabilityFile": "./availability.json"}),
                    fields: vec![
                        text_field("desiredFile", "Desired File", "path", "Desired sync input file.", "./desired.json", json!("./desired.json"), true),
                        text_field("availabilityFile", "Availability File", "path", "Optional availability file.", "./availability.json", Value::Null, false),
                        checkbox_field("fetchLive", "Fetch Live", "Fetch live inventory during preflight.", false),
                        text_field("orgId", "Org ID", "number", "Optional explicit org scope.", "2", Value::Null, false),
                    ],
                },
                ActionDescriptor {
                    id: "sync-bundle-preflight",
                    title: "Bundle Preflight",
                    description: "Review bundle-level blockers against one target inventory.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/sync/bundle-preflight",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"sourceBundle": "./sync-source-bundle.json", "targetInventory": "./target-inventory.json"}),
                    fields: vec![
                        text_field("sourceBundle", "Source Bundle", "path", "Source bundle file.", "./sync-source-bundle.json", json!("./sync-source-bundle.json"), true),
                        text_field("targetInventory", "Target Inventory", "path", "Target inventory file.", "./target-inventory.json", json!("./target-inventory.json"), true),
                        text_field("availabilityFile", "Availability File", "path", "Optional availability file.", "./availability.json", Value::Null, false),
                        checkbox_field("fetchLive", "Fetch Live", "Fetch target live inventory instead of using only staged input.", false),
                        text_field("orgId", "Org ID", "number", "Optional explicit org scope.", "2", Value::Null, false),
                    ],
                },
                ActionDescriptor {
                    id: "sync-promotion-preflight",
                    title: "Promotion Preflight",
                    description: "Review promotion mapping and readiness without applying live change.",
                    ui_mode: "document",
                    method: "POST",
                    path: "/api/1.0/sync/promotion-preflight",
                    read_only: true,
                    requires_connection: true,
                    example_request: json!({"sourceBundle": "./sync-source-bundle.json", "targetInventory": "./target-inventory.json", "mappingFile": "./mapping.json"}),
                    fields: vec![
                        text_field("sourceBundle", "Source Bundle", "path", "Source bundle file.", "./sync-source-bundle.json", json!("./sync-source-bundle.json"), true),
                        text_field("targetInventory", "Target Inventory", "path", "Target inventory file.", "./target-inventory.json", json!("./target-inventory.json"), true),
                        text_field("mappingFile", "Mapping File", "path", "Optional promotion mapping file.", "./mapping.json", Value::Null, false),
                        text_field("availabilityFile", "Availability File", "path", "Optional availability file.", "./availability.json", Value::Null, false),
                        checkbox_field("fetchLive", "Fetch Live", "Fetch target live inventory instead of using only staged input.", false),
                        text_field("orgId", "Org ID", "number", "Optional explicit org scope.", "2", Value::Null, false),
                    ],
                },
            ],
        },
    ]
}

pub(crate) fn capabilities() -> Vec<CapabilityDescriptor> {
    workspaces()
        .into_iter()
        .flat_map(|workspace| workspace.actions.into_iter())
        .map(|action| CapabilityDescriptor {
            id: action.id,
            title: action.title,
            description: action.description,
            method: action.method,
            path: action.path,
            read_only: action.read_only,
            dry_run_supported: true,
            live_apply_supported: false,
            example_request: action.example_request,
        })
        .collect()
}

pub(crate) fn action(id: &'static str) -> ActionDescriptor {
    workspaces()
        .into_iter()
        .flat_map(|workspace| workspace.actions.into_iter())
        .find(|item| item.id == id)
        .expect("action descriptor should exist")
}
