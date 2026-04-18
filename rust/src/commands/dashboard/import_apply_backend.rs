use reqwest::Method;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

use crate::common::Result;
use crate::dashboard::{
    import_dashboard_request_with_request, ExportMetadata, FolderInventoryItem, ImportArgs,
};
use crate::grafana_api::DashboardResourceClient;
use crate::http::JsonHttpClient;

use super::super::super::import_lookup::{
    determine_dashboard_import_action_with_client, determine_dashboard_import_action_with_request,
    determine_import_folder_uid_override_with_client,
    determine_import_folder_uid_override_with_request, ensure_folder_inventory_entry_cached,
    ensure_folder_inventory_entry_with_client, resolve_existing_dashboard_folder_path_with_client,
    resolve_existing_dashboard_folder_path_with_request, ImportLookupCache,
};
use super::super::super::import_validation::{
    validate_dashboard_import_dependencies_with_request, validate_matching_export_org_with_request,
};

pub(super) trait LiveImportBackend {
    fn validate_export_org(
        &mut self,
        cache: &mut ImportLookupCache,
        args: &ImportArgs,
        input_dir: &Path,
        metadata: Option<&ExportMetadata>,
    ) -> Result<()>;

    fn validate_dependencies(
        &mut self,
        input_dir: &Path,
        strict_schema: bool,
        target_schema_version: Option<i64>,
    ) -> Result<()>;

    fn determine_import_folder_uid_override(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
        folder_uid_override: Option<&str>,
        preserve_existing_folder: bool,
    ) -> Result<Option<String>>;

    fn determine_dashboard_import_action(
        &mut self,
        cache: &mut ImportLookupCache,
        payload: &Value,
        replace_existing: bool,
        update_existing_only: bool,
    ) -> Result<&'static str>;

    fn resolve_existing_dashboard_folder_path(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
    ) -> Result<Option<String>>;

    fn fetch_existing_dashboard(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
    ) -> Result<Option<Value>>;

    fn ensure_folder_inventory_entry(
        &mut self,
        cache: &mut ImportLookupCache,
        folders_by_uid: &BTreeMap<String, FolderInventoryItem>,
        folder_uid: &str,
    ) -> Result<()>;

    fn import_dashboard(&mut self, payload: &Value) -> Result<()>;
}

pub(super) struct RequestImportBackend<'a, F>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    request_json: &'a mut F,
}

impl<'a, F> RequestImportBackend<'a, F>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    pub(super) fn new(request_json: &'a mut F) -> Self {
        Self { request_json }
    }
}

impl<F> LiveImportBackend for RequestImportBackend<'_, F>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    fn validate_export_org(
        &mut self,
        cache: &mut ImportLookupCache,
        args: &ImportArgs,
        input_dir: &Path,
        metadata: Option<&ExportMetadata>,
    ) -> Result<()> {
        validate_matching_export_org_with_request(
            &mut *self.request_json,
            cache,
            args,
            input_dir,
            metadata,
            None,
        )
    }

    fn validate_dependencies(
        &mut self,
        input_dir: &Path,
        strict_schema: bool,
        target_schema_version: Option<i64>,
    ) -> Result<()> {
        validate_dashboard_import_dependencies_with_request(
            &mut *self.request_json,
            input_dir,
            strict_schema,
            target_schema_version,
        )
    }

    fn determine_import_folder_uid_override(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
        folder_uid_override: Option<&str>,
        preserve_existing_folder: bool,
    ) -> Result<Option<String>> {
        determine_import_folder_uid_override_with_request(
            &mut *self.request_json,
            cache,
            uid,
            folder_uid_override,
            preserve_existing_folder,
        )
    }

    fn determine_dashboard_import_action(
        &mut self,
        cache: &mut ImportLookupCache,
        payload: &Value,
        replace_existing: bool,
        update_existing_only: bool,
    ) -> Result<&'static str> {
        determine_dashboard_import_action_with_request(
            &mut *self.request_json,
            cache,
            payload,
            replace_existing,
            update_existing_only,
        )
    }

    fn resolve_existing_dashboard_folder_path(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
    ) -> Result<Option<String>> {
        resolve_existing_dashboard_folder_path_with_request(&mut *self.request_json, cache, uid)
    }

    fn fetch_existing_dashboard(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
    ) -> Result<Option<Value>> {
        super::super::super::import_lookup::fetch_dashboard_if_exists_cached(
            &mut *self.request_json,
            cache,
            uid,
        )
    }

    fn ensure_folder_inventory_entry(
        &mut self,
        cache: &mut ImportLookupCache,
        folders_by_uid: &BTreeMap<String, FolderInventoryItem>,
        folder_uid: &str,
    ) -> Result<()> {
        ensure_folder_inventory_entry_cached(
            &mut *self.request_json,
            cache,
            folders_by_uid,
            folder_uid,
        )
    }

    fn import_dashboard(&mut self, payload: &Value) -> Result<()> {
        let _ = import_dashboard_request_with_request(&mut *self.request_json, payload)?;
        Ok(())
    }
}

pub(super) struct ClientImportBackend<'a> {
    pub(super) dashboard: DashboardResourceClient<'a>,
}

impl<'a> ClientImportBackend<'a> {
    pub(super) fn new(client: &'a JsonHttpClient) -> Self {
        Self {
            dashboard: DashboardResourceClient::new(client),
        }
    }
}

impl LiveImportBackend for ClientImportBackend<'_> {
    fn validate_export_org(
        &mut self,
        _cache: &mut ImportLookupCache,
        args: &ImportArgs,
        input_dir: &Path,
        metadata: Option<&ExportMetadata>,
    ) -> Result<()> {
        super::super::super::import_validation::validate_matching_export_org_with_client(
            &self.dashboard,
            args,
            input_dir,
            metadata,
            None,
        )
    }

    fn validate_dependencies(
        &mut self,
        input_dir: &Path,
        strict_schema: bool,
        target_schema_version: Option<i64>,
    ) -> Result<()> {
        super::super::super::import_validation::validate_dashboard_import_dependencies_with_client(
            &self.dashboard,
            input_dir,
            strict_schema,
            target_schema_version,
        )
    }

    fn determine_import_folder_uid_override(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
        folder_uid_override: Option<&str>,
        preserve_existing_folder: bool,
    ) -> Result<Option<String>> {
        determine_import_folder_uid_override_with_client(
            &self.dashboard,
            cache,
            uid,
            folder_uid_override,
            preserve_existing_folder,
        )
    }

    fn determine_dashboard_import_action(
        &mut self,
        cache: &mut ImportLookupCache,
        payload: &Value,
        replace_existing: bool,
        update_existing_only: bool,
    ) -> Result<&'static str> {
        determine_dashboard_import_action_with_client(
            &self.dashboard,
            cache,
            payload,
            replace_existing,
            update_existing_only,
        )
    }

    fn resolve_existing_dashboard_folder_path(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
    ) -> Result<Option<String>> {
        resolve_existing_dashboard_folder_path_with_client(&self.dashboard, cache, uid)
    }

    fn fetch_existing_dashboard(
        &mut self,
        cache: &mut ImportLookupCache,
        uid: &str,
    ) -> Result<Option<Value>> {
        super::super::super::import_lookup::fetch_dashboard_if_exists_cached_with_client(
            &self.dashboard,
            cache,
            uid,
        )
    }

    fn ensure_folder_inventory_entry(
        &mut self,
        cache: &mut ImportLookupCache,
        folders_by_uid: &BTreeMap<String, FolderInventoryItem>,
        folder_uid: &str,
    ) -> Result<()> {
        ensure_folder_inventory_entry_with_client(
            &self.dashboard,
            cache,
            folders_by_uid,
            folder_uid,
        )
    }

    fn import_dashboard(&mut self, payload: &Value) -> Result<()> {
        let _ = self.dashboard.import_dashboard_request(payload)?;
        Ok(())
    }
}
