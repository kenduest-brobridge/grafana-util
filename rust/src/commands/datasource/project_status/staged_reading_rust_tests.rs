use super::build_datasource_domain_status;
use serde_json::json;

#[test]
fn build_datasource_domain_status_tracks_staged_summary_fields() {
    let document = json!({
        "summary": {
            "datasourceCount": 2,
            "orgCount": 2,
            "defaultCount": 0,
            "typeCount": 2,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["id"], json!("datasource"));
    assert_eq!(domain["scope"], json!("staged"));
    assert_eq!(domain["mode"], json!("artifact-summary"));
    assert_eq!(domain["status"], json!("ready"));
    assert_eq!(domain["reasonCode"], json!("ready"));
    assert_eq!(domain["primaryCount"], json!(2));
    assert_eq!(domain["blockerCount"], json!(0));
    assert_eq!(domain["warningCount"], json!(1));
    assert_eq!(domain["sourceKinds"], json!(["datasource-export"]));
    assert_eq!(
        domain["signalKeys"],
        json!([
            "summary.datasourceCount",
            "summary.orgCount",
            "summary.defaultCount",
            "summary.typeCount",
            "summary.wouldCreate",
            "summary.wouldUpdate",
            "summary.wouldSkip",
            "summary.wouldBlock",
            "summary.wouldCreateOrgCount",
        ])
    );
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "missing-default",
                "count": 1,
                "source": "summary.defaultCount",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!(["mark a default datasource if none is set"])
    );
}

#[test]
fn build_datasource_domain_status_surfaces_diff_drift_and_secret_readiness() {
    let document = json!({
        "summary": {
            "datasourceCount": 3,
            "orgCount": 2,
            "defaultCount": 1,
            "typeCount": 2,
            "differentCount": 2,
            "missingLiveCount": 1,
            "extraLiveCount": 1,
            "ambiguousCount": 1,
            "secretVisibilityCount": 4,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["status"], json!("ready"));
    assert_eq!(domain["reasonCode"], json!("ready"));
    assert_eq!(domain["warningCount"], json!(9));
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "diff-drift-changed-fields",
                "count": 2,
                "source": "summary.differentCount",
            },
            {
                "kind": "diff-drift-missing-live",
                "count": 1,
                "source": "summary.missingLiveCount",
            },
            {
                "kind": "diff-drift-missing-export",
                "count": 1,
                "source": "summary.extraLiveCount",
            },
            {
                "kind": "diff-drift-ambiguous",
                "count": 1,
                "source": "summary.ambiguousCount",
            },
            {
                "kind": "secret-reference-ready",
                "count": 4,
                "source": "summary.secretVisibilityCount",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!([
            "review datasource diff drift before import or sync",
            "review datasource secret references before import or sync",
        ])
    );
    assert_eq!(
        domain["signalKeys"],
        json!([
            "summary.datasourceCount",
            "summary.orgCount",
            "summary.defaultCount",
            "summary.typeCount",
            "summary.wouldCreate",
            "summary.wouldUpdate",
            "summary.wouldSkip",
            "summary.wouldBlock",
            "summary.wouldCreateOrgCount",
            "summary.differentCount",
            "summary.missingLiveCount",
            "summary.extraLiveCount",
            "summary.ambiguousCount",
            "summary.secretVisibilityCount",
        ])
    );
}

#[test]
fn build_datasource_domain_status_surfaces_import_preview_mutation_counts() {
    let document = json!({
        "summary": {
            "datasourceCount": 4,
            "orgCount": 2,
            "defaultCount": 1,
            "typeCount": 3,
            "would_create": 2,
            "would_update": 1,
            "would_skip": 1,
            "would_block": 3,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["status"], json!("blocked"));
    assert_eq!(domain["reasonCode"], json!("blocked-by-blockers"));
    assert_eq!(domain["blockerCount"], json!(3));
    assert_eq!(domain["warningCount"], json!(4));
    assert_eq!(
        domain["blockers"],
        json!([
            {
                "kind": "import-preview-would-block",
                "count": 3,
                "source": "summary.would_block",
            }
        ])
    );
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "import-preview-would-create",
                "count": 2,
                "source": "summary.would_create",
            },
            {
                "kind": "import-preview-would-update",
                "count": 1,
                "source": "summary.would_update",
            },
            {
                "kind": "import-preview-would-skip",
                "count": 1,
                "source": "summary.would_skip",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!([
            "resolve datasource import preview blockers before import or sync",
            "review datasource import preview before import or sync"
        ])
    );
    assert_eq!(
        domain["signalKeys"],
        json!([
            "summary.datasourceCount",
            "summary.orgCount",
            "summary.defaultCount",
            "summary.typeCount",
            "summary.wouldCreate",
            "summary.wouldUpdate",
            "summary.wouldSkip",
            "summary.wouldBlock",
            "summary.wouldCreateOrgCount",
            "summary.would_create",
            "summary.would_update",
            "summary.would_block",
            "summary.would_skip",
        ])
    );
}

#[test]
fn build_datasource_domain_status_surfaces_import_preview_skip_only() {
    let document = json!({
        "summary": {
            "datasourceCount": 2,
            "orgCount": 1,
            "defaultCount": 1,
            "typeCount": 1,
            "would_skip": 2,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["status"], json!("ready"));
    assert_eq!(domain["reasonCode"], json!("ready"));
    assert_eq!(domain["warningCount"], json!(2));
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "import-preview-would-skip",
                "count": 2,
                "source": "summary.would_skip",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!(["review datasource import preview before import or sync",])
    );
    assert_eq!(
        domain["signalKeys"],
        json!([
            "summary.datasourceCount",
            "summary.orgCount",
            "summary.defaultCount",
            "summary.typeCount",
            "summary.wouldCreate",
            "summary.wouldUpdate",
            "summary.wouldSkip",
            "summary.wouldBlock",
            "summary.wouldCreateOrgCount",
            "summary.would_skip",
        ])
    );
}

#[test]
fn build_datasource_domain_status_surfaces_import_org_creation_readiness() {
    let document = json!({
        "summary": {
            "datasourceCount": 4,
            "orgCount": 3,
            "defaultCount": 1,
            "typeCount": 2,
            "wouldCreateOrgCount": 2,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["status"], json!("ready"));
    assert_eq!(domain["reasonCode"], json!("ready"));
    assert_eq!(domain["warningCount"], json!(2));
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "import-preview-would-create-org",
                "count": 2,
                "source": "summary.wouldCreateOrgCount",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!(["review datasource org creation before import or sync"])
    );
    assert_eq!(
        domain["signalKeys"],
        json!([
            "summary.datasourceCount",
            "summary.orgCount",
            "summary.defaultCount",
            "summary.typeCount",
            "summary.wouldCreate",
            "summary.wouldUpdate",
            "summary.wouldSkip",
            "summary.wouldBlock",
            "summary.wouldCreateOrgCount",
        ])
    );
}

#[test]
fn build_datasource_domain_status_surfaces_routed_source_org_labels() {
    let document = json!({
        "summary": {
            "datasourceCount": 4,
            "orgCount": 3,
            "defaultCount": 1,
            "typeCount": 2,
            "sourceOrgLabels": ["1:Main Org.", "2:Ops Org"],
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["warningCount"], json!(2));
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "import-preview-routed-source-orgs",
                "count": 2,
                "source": "summary.sourceOrgLabels",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!(["review datasource org routing before import or sync"])
    );
    assert_eq!(
        domain["signalKeys"],
        json!([
            "summary.datasourceCount",
            "summary.orgCount",
            "summary.defaultCount",
            "summary.typeCount",
            "summary.wouldCreate",
            "summary.wouldUpdate",
            "summary.wouldSkip",
            "summary.wouldBlock",
            "summary.wouldCreateOrgCount",
            "summary.sourceOrgLabels",
        ])
    );
}

#[test]
fn build_datasource_domain_status_combines_routing_and_org_creation_guidance() {
    let document = json!({
        "summary": {
            "datasourceCount": 4,
            "orgCount": 3,
            "defaultCount": 1,
            "typeCount": 2,
            "wouldCreateOrgCount": 2,
            "sourceOrgLabels": ["1:Main Org.", "2:Ops Org"],
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["warningCount"], json!(4));
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "import-preview-would-create-org",
                "count": 2,
                "source": "summary.wouldCreateOrgCount",
            },
            {
                "kind": "import-preview-routed-source-orgs",
                "count": 2,
                "source": "summary.sourceOrgLabels",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!(["review datasource org routing and org creation before import or sync"])
    );
}

#[test]
fn build_datasource_domain_status_falls_back_to_snake_case_import_org_creation_count() {
    let document = json!({
        "summary": {
            "datasourceCount": 4,
            "orgCount": 3,
            "defaultCount": 1,
            "typeCount": 2,
            "would_create_org_count": 1,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "import-preview-would-create-org",
                "count": 1,
                "source": "summary.would_create_org_count",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!(["review datasource org creation before import or sync"])
    );
}

#[test]
fn build_datasource_domain_status_keeps_diff_and_secret_signals_with_import_preview() {
    let document = json!({
        "summary": {
            "datasourceCount": 5,
            "orgCount": 2,
            "defaultCount": 1,
            "typeCount": 3,
            "differentCount": 2,
            "secretVisibilityCount": 4,
            "wouldCreate": 1,
            "wouldSkip": 1,
            "wouldBlock": 2,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["status"], json!("blocked"));
    assert_eq!(domain["reasonCode"], json!("blocked-by-blockers"));
    assert_eq!(domain["blockerCount"], json!(2));
    assert_eq!(domain["warningCount"], json!(8));
    assert_eq!(
        domain["blockers"],
        json!([
            {
                "kind": "import-preview-would-block",
                "count": 2,
                "source": "summary.wouldBlock",
            }
        ])
    );
    assert_eq!(
        domain["warnings"],
        json!([
            {
                "kind": "diff-drift-changed-fields",
                "count": 2,
                "source": "summary.differentCount",
            },
            {
                "kind": "secret-reference-ready",
                "count": 4,
                "source": "summary.secretVisibilityCount",
            },
            {
                "kind": "import-preview-would-create",
                "count": 1,
                "source": "summary.wouldCreate",
            },
            {
                "kind": "import-preview-would-skip",
                "count": 1,
                "source": "summary.wouldSkip",
            }
        ])
    );
    assert_eq!(
        domain["nextActions"],
        json!([
            "resolve datasource import preview blockers before import or sync",
            "review datasource diff drift before import or sync",
            "review datasource secret references before import or sync",
            "review datasource import preview before import or sync",
        ])
    );
}

#[test]
fn build_datasource_domain_status_is_partial_without_datasources() {
    let document = json!({
        "summary": {
            "datasourceCount": 0,
            "orgCount": 0,
            "defaultCount": 0,
            "typeCount": 0,
        }
    });

    let domain = build_datasource_domain_status(Some(&document))
        .unwrap()
        .into_domain_status();
    let domain = serde_json::to_value(domain).unwrap();

    assert_eq!(domain["status"], json!("partial"));
    assert_eq!(domain["reasonCode"], json!("partial-no-data"));
    assert_eq!(
        domain["nextActions"],
        json!(["export at least one datasource"])
    );
}
