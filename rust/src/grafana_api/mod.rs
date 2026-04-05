//! Shared internal Grafana connection and client layer.
//!
//! This module centralizes profile/auth/header resolution and exposes one
//! root client that domain runtimes can share without each rebuilding the
//! same `JsonHttpClient` wiring.

mod access;
mod alerting;
mod client;
mod connection;
mod dashboard;
mod datasource;

pub(crate) use access::AccessResourceClient;
pub(crate) use alerting::{
    expect_object, expect_object_list, parse_template_list_response, AlertingResourceClient,
};
pub(crate) use client::GrafanaApiClient;
pub(crate) use connection::{AuthInputs, GrafanaConnection};
pub(crate) use dashboard::DashboardResourceClient;
pub(crate) use datasource::DatasourceResourceClient;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
