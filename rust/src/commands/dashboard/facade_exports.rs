pub(crate) mod pub_exports {
    pub use super::super::cli_defs::{
        build_auth_context, build_http_client, build_http_client_for_org,
        normalize_dashboard_cli_args, parse_cli_from, BrowseArgs, CloneLiveArgs, CommonCliArgs,
        DashboardAuthContext, DashboardCliArgs, DashboardCommand, DashboardHistoryArgs,
        DashboardHistorySubcommand, DashboardImportInputFormat, DashboardPlanOutputFormat,
        DashboardServeScriptFormat, DeleteArgs, DiffArgs, EditLiveArgs, ExportArgs,
        ExportLayoutArgs, ExportLayoutOutputFormat, ExportLayoutVariant, FolderPermissionMatchMode,
        GetArgs, GovernanceGateArgs, GovernanceGateOutputFormat, GovernancePolicySource,
        HistoryDiffArgs, HistoryExportArgs, HistoryListArgs, HistoryOutputFormat,
        HistoryRestoreArgs, ImpactArgs, ImpactOutputFormat, ImportArgs, InspectExportArgs,
        InspectExportInputType, InspectExportReportFormat, InspectLiveArgs, InspectOutputFormat,
        InspectVarsArgs, ListArgs, PatchFileArgs, PlanArgs, PublishArgs, RawToPromptArgs,
        RawToPromptLogFormat, RawToPromptOutputFormat, RawToPromptResolution, ReviewArgs,
        ScreenshotArgs, ScreenshotFullPageOutput, ScreenshotOutputFormat, ScreenshotTheme,
        ServeArgs, SimpleOutputFormat, SummaryArgs, TopologyArgs, TopologyOutputFormat,
        ValidateExportArgs, ValidationOutputFormat,
    };
    pub use super::super::command_runner::{
        execute_dashboard_inspect_export, execute_dashboard_inspect_live,
        execute_dashboard_inspect_vars, execute_dashboard_list,
    };
    pub use super::super::export::{
        build_export_variant_dirs, build_output_path, export_dashboards_with_client,
    };
    pub use super::super::help::{
        maybe_render_dashboard_help_full_from_os_args,
        maybe_render_dashboard_subcommand_help_from_os_args, render_summary_export_help_full,
        render_summary_live_help_full,
    };
    pub use super::super::import::{diff_dashboards_with_client, import_dashboards_with_client};
    pub use super::super::list::list_dashboards_with_client;
    pub use super::super::live::{
        delete_dashboard_request, delete_folder_request, fetch_dashboard, import_dashboard_request,
        list_dashboard_summaries, list_datasources,
    };
    pub use super::super::prompt::build_external_export_document;
    pub use super::super::screenshot::capture_dashboard_screenshot;
}

pub(crate) mod crate_exports {
    pub(crate) use super::super::authoring::{
        clone_live_dashboard_to_file_with_client, get_live_dashboard_to_file_with_client,
        patch_dashboard_file, publish_dashboard_with_client, render_dashboard_review_csv,
        render_dashboard_review_json, render_dashboard_review_table, render_dashboard_review_text,
        render_dashboard_review_yaml, review_dashboard_file as build_dashboard_review,
    };
    pub(crate) use super::super::cli_defs::materialize_dashboard_common_auth;
    pub(crate) use super::super::cli_defs::{build_api_client, build_http_client_for_org_from_api};
    pub(crate) use super::super::export_layout::run_export_layout_repair;
    pub(crate) use super::super::plan::run_dashboard_plan;
    pub(crate) use super::super::raw_to_prompt::run_raw_to_prompt;
    pub(crate) use super::super::source_loader::{
        infer_dashboard_workspace_root, load_dashboard_source,
        resolve_dashboard_workspace_variant_dir, LoadedDashboardSource,
    };

    #[allow(unused_imports)]
    pub(crate) use super::super::facade_support::{
        build_datasource_inventory_record, build_folder_path, build_live_dashboard_domain_status,
        build_live_dashboard_domain_status_from_inputs, collect_folder_inventory_with_request,
        collect_live_dashboard_project_status_inputs_with_request,
        fetch_dashboard_if_exists_with_request, fetch_dashboard_permissions_with_request,
        fetch_dashboard_with_request, fetch_folder_if_exists_with_request,
        fetch_folder_permissions_with_request, format_folder_inventory_status_line,
        import_dashboard_request_with_request, list_dashboard_summaries_with_request,
        list_datasources_with_request, load_builtin_governance_policy, load_governance_policy,
        load_governance_policy_file, load_governance_policy_source,
        LiveDashboardProjectStatusInputs,
    };
    #[allow(unused_imports)]
    pub(crate) use super::super::files::{
        build_dashboard_index_item, build_export_metadata, build_import_payload,
        build_preserved_web_import_document, build_root_export_index, build_variant_index,
        discover_dashboard_files, extract_dashboard_object, load_datasource_inventory,
        load_export_metadata, load_folder_inventory, load_json_file,
        resolve_dashboard_import_source, write_dashboard, write_json_document,
        DashboardRepoLayoutKind, DashboardSourceKind, ResolvedDashboardImportSource,
    };
    pub(crate) use super::super::inspect::build_export_inspection_summary_for_variant;
    pub(crate) use super::super::inspect_live::TempInspectDir;
    pub(crate) use super::super::inspect_report::ExportInspectionQueryRow;
    pub(crate) use super::super::inspect_summary::{
        build_export_inspection_summary_document, ExportInspectionSummary,
    };
    pub(crate) use super::super::models::{
        DashboardExportRootManifest, DashboardExportRootScopeKind, DashboardIndexItem,
        DatasourceInventoryItem, ExportDatasourceUsageSummary, ExportMetadata, ExportOrgSummary,
        FolderInventoryItem, RootExportIndex, RootExportVariants, VariantIndexEntry,
    };
    pub(crate) use super::super::project_status::build_dashboard_domain_status;
    pub(crate) use super::super::prompt::{
        build_datasource_catalog, collect_datasource_refs, datasource_type_alias,
        is_builtin_datasource_ref, is_placeholder_string, lookup_datasource,
        resolve_datasource_type_alias,
    };
}
