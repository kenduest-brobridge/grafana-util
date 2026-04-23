use serde_json::{Map, Value};

use crate::common::Result;

use super::sync_live_apply_datasource::resolve_live_datasource_target;
use super::SyncLiveClient;

impl<'a> SyncLiveClient<'a> {
    pub(crate) fn create_folder(
        &self,
        title: &str,
        uid: &str,
        parent_uid: Option<&str>,
    ) -> Result<Map<String, Value>> {
        self.api
            .dashboard()
            .create_folder_entry(title, uid, parent_uid)
    }

    pub(crate) fn update_folder(
        &self,
        uid: &str,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.dashboard().update_folder_request(uid, payload)
    }

    pub(crate) fn delete_folder(&self, uid: &str) -> Result<Value> {
        Ok(Value::Object(
            self.api
                .dashboard()
                .delete_folder_request(uid)?
                .into_iter()
                .collect(),
        ))
    }

    pub(crate) fn upsert_dashboard(
        &self,
        payload: &Map<String, Value>,
        overwrite: bool,
        folder_uid: Option<&str>,
    ) -> Result<Value> {
        let mut body = Map::new();
        body.insert("dashboard".to_string(), Value::Object(payload.clone()));
        body.insert("overwrite".to_string(), Value::Bool(overwrite));
        if let Some(folder_uid) = folder_uid.filter(|value: &&str| !value.is_empty()) {
            body.insert(
                "folderUid".to_string(),
                Value::String(folder_uid.to_string()),
            );
        }
        self.api
            .dashboard()
            .import_dashboard_request(&Value::Object(body))
    }

    pub(crate) fn delete_dashboard(&self, uid: &str) -> Result<Value> {
        Ok(Value::Object(
            self.api
                .dashboard()
                .delete_dashboard_request(uid)?
                .into_iter()
                .collect(),
        ))
    }

    pub(crate) fn resolve_datasource_target(
        &self,
        identity: &str,
    ) -> Result<Option<Map<String, Value>>> {
        resolve_live_datasource_target(&self.list_datasources()?, identity)
    }

    pub(crate) fn create_datasource(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.datasource().create_datasource(payload)
    }

    pub(crate) fn update_datasource(
        &self,
        datasource_uid: &str,
        fallback_datasource_id: Option<&str>,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.datasource().update_datasource_by_uid(
            datasource_uid,
            fallback_datasource_id,
            payload,
        )
    }

    pub(crate) fn delete_datasource(
        &self,
        datasource_uid: &str,
        fallback_datasource_id: Option<&str>,
    ) -> Result<Value> {
        self.api
            .datasource()
            .delete_datasource_by_uid(datasource_uid, fallback_datasource_id)
    }

    pub(crate) fn create_alert_rule(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().create_alert_rule(payload)
    }

    pub(crate) fn update_alert_rule(
        &self,
        uid: &str,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().update_alert_rule(uid, payload)
    }

    pub(crate) fn delete_alert_rule(&self, uid: &str) -> Result<Value> {
        self.api.alerting().delete_alert_rule(uid)
    }

    pub(crate) fn create_contact_point(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().create_contact_point(payload)
    }

    pub(crate) fn update_contact_point(
        &self,
        uid: &str,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().update_contact_point(uid, payload)
    }

    pub(crate) fn delete_contact_point(&self, uid: &str) -> Result<Value> {
        self.api.alerting().delete_contact_point(uid)
    }

    pub(crate) fn create_mute_timing(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().create_mute_timing(payload)
    }

    pub(crate) fn update_mute_timing(
        &self,
        name: &str,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().update_mute_timing(name, payload)
    }

    pub(crate) fn delete_mute_timing(&self, name: &str) -> Result<Value> {
        self.api.alerting().delete_mute_timing(name)
    }

    pub(crate) fn update_notification_policies(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().update_notification_policies(payload)
    }

    pub(crate) fn delete_notification_policies(&self) -> Result<Value> {
        self.api.alerting().delete_notification_policies()
    }

    pub(crate) fn update_template(
        &self,
        name: &str,
        payload: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        self.api.alerting().update_template(name, payload)
    }

    pub(crate) fn delete_template(&self, name: &str) -> Result<Value> {
        self.api.alerting().delete_template(name)
    }
}
