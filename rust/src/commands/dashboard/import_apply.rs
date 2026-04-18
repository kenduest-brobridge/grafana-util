//! Import orchestration for Dashboard resources, including input normalization and apply contract handling.

use reqwest::Method;
use serde_json::Value;

use crate::common::Result;
#[cfg(feature = "tui")]
use crate::dashboard::import_interactive;
use crate::dashboard::{build_http_client_for_org, DiffArgs, ImportArgs};
use crate::http::{JsonHttpClient, JsonHttpClientConfig};

use super::super::import_compare::diff_dashboards_with_request;
use super::super::import_lookup::ImportLookupCache;
use super::import_dry_run::{
    collect_import_dry_run_report_with_client, collect_import_dry_run_report_with_request,
};

#[path = "import_apply_backend.rs"]
mod import_apply_backend;
#[path = "import_apply_live.rs"]
mod import_apply_live;
#[path = "import_apply_prepare.rs"]
mod import_apply_prepare;
#[path = "import_apply_render.rs"]
mod import_apply_render;

use import_apply_backend::{ClientImportBackend, LiveImportBackend, RequestImportBackend};
use import_apply_live::run_live_import;
use import_apply_prepare::{prepare_import_run, validate_import_args, PreparedImportRun};
use import_apply_render::render_dry_run_report;

/// Purpose: implementation note.
pub fn diff_dashboards_with_client(client: &JsonHttpClient, args: &DiffArgs) -> Result<usize> {
    diff_dashboards_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

/// Purpose: implementation note.
pub(crate) fn import_dashboards_with_request<F>(
    mut request_json: F,
    args: &ImportArgs,
) -> Result<usize>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if args.dry_run {
        validate_import_args(args)?;
        let report = collect_import_dry_run_report_with_request(&mut request_json, args)?;
        return render_dry_run_report(&report, args);
    }

    validate_import_args(args)?;
    let mut lookup_cache = ImportLookupCache::default();
    let prepared = prepare_request_import_run(&mut request_json, args, &mut lookup_cache)?;
    if prepared.dashboard_files.is_empty() {
        return Ok(0);
    }
    let mut backend = RequestImportBackend::new(&mut request_json);
    validate_live_import_backend(&mut backend, args, &prepared, &mut lookup_cache)?;
    run_live_import(&mut backend, args, &prepared, &mut lookup_cache)
}

fn prepare_request_import_run<F>(
    request_json: &mut F,
    args: &ImportArgs,
    lookup_cache: &mut ImportLookupCache,
) -> Result<PreparedImportRun>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    prepare_import_run(
        &mut |resolved_import, dashboard_files| {
            #[cfg(feature = "tui")]
            {
                super::selected_dashboard_files(
                    request_json,
                    lookup_cache,
                    args,
                    resolved_import,
                    resolved_import.dashboard_dir(),
                    dashboard_files,
                )
            }
            #[cfg(not(feature = "tui"))]
            {
                let _ = request_json;
                let _ = lookup_cache;
                super::selected_dashboard_files(
                    args,
                    resolved_import,
                    resolved_import.dashboard_dir(),
                    dashboard_files,
                )
            }
        },
        args,
    )
}

fn validate_live_import_backend<B: LiveImportBackend>(
    backend: &mut B,
    args: &ImportArgs,
    prepared: &PreparedImportRun,
    lookup_cache: &mut ImportLookupCache,
) -> Result<()> {
    backend.validate_export_org(
        lookup_cache,
        args,
        prepared.resolved_import.metadata_dir(),
        prepared.metadata.as_ref(),
    )?;
    backend.validate_dependencies(
        prepared.resolved_import.dashboard_dir(),
        args.strict_schema,
        args.target_schema_version,
    )
}

/// Purpose: implementation note.
pub fn import_dashboards_with_client(client: &JsonHttpClient, args: &ImportArgs) -> Result<usize> {
    if args.dry_run {
        let report = collect_import_dry_run_report_with_client(client, args)?;
        return render_dry_run_report(&report, args);
    }
    validate_import_args(args)?;
    let mut backend = ClientImportBackend::new(client);
    let mut lookup_cache = ImportLookupCache::default();
    let prepared = prepare_client_import_run(&mut backend, args, &mut lookup_cache)?;
    if prepared.dashboard_files.is_empty() {
        return Ok(0);
    }
    validate_live_import_backend(&mut backend, args, &prepared, &mut lookup_cache)?;
    run_live_import(&mut backend, args, &prepared, &mut lookup_cache)
}

fn prepare_client_import_run(
    backend: &mut ClientImportBackend<'_>,
    args: &ImportArgs,
    lookup_cache: &mut ImportLookupCache,
) -> Result<PreparedImportRun> {
    prepare_import_run(
        &mut |resolved_import, _dashboard_files| {
            #[cfg(feature = "tui")]
            {
                import_interactive::select_import_dashboard_files_with_client(
                    &backend.dashboard,
                    lookup_cache,
                    args,
                    resolved_import,
                    _dashboard_files.as_slice(),
                )
            }
            #[cfg(not(feature = "tui"))]
            {
                let _ = backend;
                let _ = lookup_cache;
                let _ = _dashboard_files;
                let _ = resolved_import;
                if args.interactive {
                    return super::super::tui_not_built("import --interactive");
                }
                Ok(None)
            }
        },
        args,
    )
}

/// Purpose: implementation note.
pub(crate) fn import_dashboards_with_org_clients(args: &ImportArgs) -> Result<usize> {
    let context = super::build_import_auth_context(args)?;
    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: context.url.clone(),
        headers: context.headers.clone(),
        timeout_secs: context.timeout,
        verify_ssl: context.verify_ssl,
    })?;
    if !args.use_export_org {
        return import_dashboards_with_request(
            |method, path, params, payload| client.request_json(method, path, params, payload),
            args,
        );
    }
    super::super::import_routed::import_dashboards_by_export_org_with_request(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        |target_org_id, scoped_args| {
            let scoped_client = build_http_client_for_org(&args.common, target_org_id)?;
            import_dashboards_with_client(&scoped_client, scoped_args)
        },
        |target_org_id, scoped_args| {
            let scoped_client = build_http_client_for_org(&args.common, target_org_id)?;
            collect_import_dry_run_report_with_request(
                |method, path, params, payload| {
                    scoped_client.request_json(method, path, params, payload)
                },
                scoped_args,
            )
        },
        args,
    )
}
