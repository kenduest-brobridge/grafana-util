//! Long-form resource CLI help text constants.

pub(crate) const RESOURCE_KINDS_AFTER_HELP: &str = r#"Examples:

  Show supported resource kinds as a table:
    grafana-util status resource kinds

  Render the same kind catalog as JSON:
    grafana-util status resource kinds --output-format json"#;

pub(crate) const RESOURCE_DESCRIBE_AFTER_HELP: &str = r#"Examples:

  Describe every supported kind as a table:
    grafana-util status resource describe

  Describe one supported kind as JSON:
    grafana-util status resource describe dashboards --output-format json"#;

pub(crate) const RESOURCE_LIST_AFTER_HELP: &str = r#"Examples:

  List dashboards as a table:
    grafana-util status resource list dashboards --url http://localhost:3000 --basic-user admin --basic-password admin

  List folders as YAML:
    grafana-util status resource list folders --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output-format yaml

  List alert rules as JSON:
    grafana-util status resource list alert-rules --profile prod --output-format json"#;

pub(crate) const RESOURCE_GET_AFTER_HELP: &str = r#"Examples:

  Fetch one dashboard by UID:
    grafana-util status resource get dashboards/cpu-main --url http://localhost:3000 --basic-user admin --basic-password admin

  Fetch one datasource by UID as YAML:
    grafana-util status resource get datasources/prom-main --profile prod --output-format yaml

  Fetch one org by numeric ID:
    grafana-util status resource get orgs/1 --profile prod --output-format json

  Prefer the legacy datasource path when UID lookup is not available:
    grafana-util status resource get datasources/10 --api-mode legacy --url http://localhost:3000"#;
