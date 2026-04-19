use serde::Serialize;

use super::cli_defs::ResourceKind;
use crate::common::{message, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceSelector {
    pub(crate) kind: ResourceKind,
    pub(crate) identity: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ResourceKindRecord {
    pub(crate) kind: &'static str,
    pub(crate) singular: &'static str,
    pub(crate) description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ResourceDescribeRecord {
    pub(crate) kind: &'static str,
    pub(crate) singular: &'static str,
    pub(crate) selector: &'static str,
    pub(crate) list_endpoint: &'static str,
    pub(crate) get_endpoint: &'static str,
    pub(crate) description: &'static str,
}

pub(crate) fn supported_kinds() -> [ResourceKind; 5] {
    [
        ResourceKind::Dashboards,
        ResourceKind::Folders,
        ResourceKind::Datasources,
        ResourceKind::AlertRules,
        ResourceKind::Orgs,
    ]
}

fn supported_kind_names() -> String {
    supported_kinds()
        .into_iter()
        .map(|kind| kind.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn supported_kind_records() -> Vec<ResourceKindRecord> {
    supported_kinds()
        .into_iter()
        .map(|kind| ResourceKindRecord {
            kind: kind.as_str(),
            singular: kind.singular_label(),
            description: kind.description(),
        })
        .collect()
}

pub(crate) fn describe_records(kind: Option<ResourceKind>) -> Vec<ResourceDescribeRecord> {
    let kinds = kind
        .map(|item| vec![item])
        .unwrap_or_else(|| supported_kinds().to_vec());
    kinds
        .into_iter()
        .map(|kind| ResourceDescribeRecord {
            kind: kind.as_str(),
            singular: kind.singular_label(),
            selector: kind.selector_pattern(),
            list_endpoint: kind.list_endpoint(),
            get_endpoint: kind.get_endpoint(),
            description: kind.description(),
        })
        .collect()
}

pub(crate) fn parse_selector(input: &str) -> Result<ResourceSelector> {
    let (kind, identity) = input
        .split_once('/')
        .ok_or_else(|| {
            message(
                "Resource selector must use <kind>/<identity>. Use grafana-util resource describe to see selector patterns and grafana-util resource kinds to list supported kinds.",
            )
        })?;
    let kind = kind.trim();
    let identity = identity.trim();
    if kind.is_empty() || identity.is_empty() {
        return Err(message(
            "Resource selector kind and identity cannot be empty.",
        ));
    }
    let kind = match kind {
        "dashboards" => ResourceKind::Dashboards,
        "folders" => ResourceKind::Folders,
        "datasources" => ResourceKind::Datasources,
        "alert-rules" => ResourceKind::AlertRules,
        "orgs" => ResourceKind::Orgs,
        _ => {
            return Err(message(format!(
                "Unsupported resource selector kind '{kind}'. Use grafana-util resource describe to see selector patterns. Supported kinds: {}.",
                supported_kind_names()
            )))
        }
    };
    Ok(ResourceSelector {
        kind,
        identity: identity.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_selector_requires_kind_and_identity() {
        let error = parse_selector("dashboards").unwrap_err().to_string();
        assert!(error.contains("<kind>/<identity>"));
        assert!(error.contains("resource describe"));
        assert!(error.contains("resource kinds"));
    }

    #[test]
    fn parse_selector_trims_whitespace() {
        let selector = parse_selector(" dashboards / cpu-main ").unwrap();
        assert_eq!(selector.kind, ResourceKind::Dashboards);
        assert_eq!(selector.identity, "cpu-main");
    }

    #[test]
    fn parse_selector_accepts_supported_kinds() {
        let selector = parse_selector("datasources/prom-main").unwrap();
        assert_eq!(selector.kind, ResourceKind::Datasources);
        assert_eq!(selector.identity, "prom-main");
    }

    #[test]
    fn parse_selector_rejects_unsupported_kind_with_help() {
        let error = parse_selector("widgets/demo").unwrap_err().to_string();
        assert!(error.contains("Unsupported resource selector kind 'widgets'."));
        assert!(error.contains("resource describe"));
        assert!(error.contains("dashboards, folders, datasources, alert-rules, orgs"));
    }

    #[test]
    fn describe_records_include_selector_and_endpoints() {
        let records = describe_records(Some(ResourceKind::Dashboards));
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.kind, "dashboards");
        assert_eq!(record.selector, "dashboards/<uid>");
        assert_eq!(record.list_endpoint, "GET /api/search");
        assert_eq!(record.get_endpoint, "GET /api/dashboards/uid/{uid}");
    }
}
