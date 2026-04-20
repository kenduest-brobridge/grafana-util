use reqwest::Method;
use serde_json::{Map, Value};

use crate::common::{message, Result};

use super::SyncLiveClient;

#[path = "sync_live_read/alert.rs"]
mod alert;
#[path = "sync_live_read/availability.rs"]
mod availability;
#[path = "sync_live_read/dashboard.rs"]
mod dashboard;
#[path = "sync_live_read/datasource.rs"]
mod datasource;

impl<'a> SyncLiveClient<'a> {
    pub(crate) fn list_folders(&self) -> Result<Vec<Map<String, Value>>> {
        self.api.dashboard().list_folders()
    }

    pub(crate) fn list_dashboard_summaries(
        &self,
        page_size: usize,
    ) -> Result<Vec<Map<String, Value>>> {
        self.api.dashboard().list_dashboard_summaries(page_size)
    }

    pub(crate) fn fetch_dashboard(&self, uid: &str) -> Result<Value> {
        self.api.dashboard().fetch_dashboard(uid)
    }

    pub(crate) fn list_datasources(&self) -> Result<Vec<Map<String, Value>>> {
        self.api.datasource().list_datasources()
    }

    pub(crate) fn list_plugins(&self) -> Result<Vec<Map<String, Value>>> {
        match self.request_json(Method::GET, "/api/plugins", &[], None)? {
            Some(Value::Array(items)) => items
                .into_iter()
                .map(|item| match item {
                    Value::Object(object) => Ok(object),
                    _ => Err(message("Unexpected plugin list response from Grafana.")),
                })
                .collect(),
            Some(_) => Err(message("Unexpected plugin list response from Grafana.")),
            None => Ok(Vec::new()),
        }
    }

    pub(crate) fn list_alert_rules(&self) -> Result<Vec<Map<String, Value>>> {
        self.api.alerting().list_alert_rules()
    }

    pub(crate) fn list_contact_points(&self) -> Result<Vec<Map<String, Value>>> {
        self.api.alerting().list_contact_points()
    }

    pub(crate) fn list_mute_timings(&self) -> Result<Vec<Map<String, Value>>> {
        self.api.alerting().list_mute_timings()
    }

    pub(crate) fn get_notification_policies(&self) -> Result<Map<String, Value>> {
        self.api.alerting().get_notification_policies()
    }

    pub(crate) fn list_templates(&self) -> Result<Vec<Map<String, Value>>> {
        self.api.alerting().list_templates()
    }

    pub(crate) fn get_template(&self, name: &str) -> Result<Map<String, Value>> {
        self.api.alerting().get_template(name)
    }

    pub(crate) fn fetch_live_resource_specs(&self, page_size: usize) -> Result<Vec<Value>> {
        let mut specs = Vec::new();

        dashboard::append_folder_specs(self, &mut specs)?;
        dashboard::append_dashboard_specs(self, &mut specs, page_size)?;
        datasource::append_datasource_resource_specs_from_client(self, &mut specs)?;
        alert::append_alert_resource_specs_from_client(self, &mut specs)?;

        Ok(specs)
    }

    pub(crate) fn fetch_live_availability(&self) -> Result<Value> {
        let mut availability = availability::empty_document();

        datasource::append_datasource_availability_from_client(self, &mut availability)?;
        availability::append_plugin_ids_from_slice(&self.list_plugins()?, &mut availability);
        availability::append_contact_point_identifiers_from_slice(
            &self.list_contact_points()?,
            &mut availability,
        );

        Ok(Value::Object(availability))
    }
}

pub(crate) fn fetch_live_resource_specs_with_client(
    client: &SyncLiveClient<'_>,
    page_size: usize,
) -> Result<Vec<Value>> {
    client.fetch_live_resource_specs(page_size)
}

pub(crate) fn fetch_live_availability_with_client(client: &SyncLiveClient<'_>) -> Result<Value> {
    client.fetch_live_availability()
}

#[cfg(test)]
pub(crate) fn fetch_live_resource_specs_with_request<F>(
    mut request_json: F,
    page_size: usize,
) -> Result<Vec<Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut specs = Vec::new();
    dashboard::append_folder_specs_with_request(&mut request_json, &mut specs)?;
    dashboard::append_dashboard_specs_with_request(&mut request_json, &mut specs, page_size)?;
    datasource::append_datasource_resource_specs_with_request(&mut request_json, &mut specs)?;
    alert::append_alert_resource_specs_with_request(&mut request_json, &mut specs)?;

    Ok(specs)
}

#[cfg(test)]
pub(crate) fn fetch_live_availability_with_request<F>(mut request_json: F) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut availability = availability::empty_document();

    datasource::append_datasource_availability_with_request(&mut request_json, &mut availability)?;

    match request_json(Method::GET, "/api/plugins", &[], None)? {
        Some(Value::Array(plugins)) => {
            availability::append_plugin_ids_from_values(&plugins, &mut availability);
        }
        Some(_) => return Err(message("Unexpected plugin list response from Grafana.")),
        None => {}
    }

    match request_json(
        Method::GET,
        "/api/v1/provisioning/contact-points",
        &[],
        None,
    )? {
        Some(Value::Array(contact_points)) => {
            availability::append_contact_point_identifiers_from_values(
                &contact_points,
                &mut availability,
            )?;
        }
        Some(_) => {
            return Err(message(
                "Unexpected contact-point list response from Grafana.",
            ))
        }
        None => {}
    }

    Ok(Value::Object(availability))
}
