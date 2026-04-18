use crate::overview::OverviewOutputFormat;
use crate::snapshot::{build_snapshot_overview_args, SnapshotCliArgs, SnapshotReviewArgs};
use clap::Parser;
use serde_json::json;

#[test]
fn snapshot_review_builds_overview_args_for_interactive_output() {
    let review_args = SnapshotReviewArgs {
        input_dir: std::path::PathBuf::from("./snapshot"),
        interactive: false,
        output_format: OverviewOutputFormat::Interactive,
    };

    let overview_args = build_snapshot_overview_args(&review_args);

    assert_eq!(
        overview_args.dashboard_export_dir,
        Some(std::path::PathBuf::from("./snapshot/dashboards"))
    );
    assert_eq!(
        overview_args.datasource_export_dir,
        Some(std::path::PathBuf::from("./snapshot/datasources"))
    );
    assert_eq!(
        overview_args.access_user_export_dir,
        Some(std::path::PathBuf::from("./snapshot/access/users"))
    );
    assert_eq!(
        overview_args.access_team_export_dir,
        Some(std::path::PathBuf::from("./snapshot/access/teams"))
    );
    assert_eq!(
        overview_args.access_org_export_dir,
        Some(std::path::PathBuf::from("./snapshot/access/orgs"))
    );
    assert_eq!(
        overview_args.access_service_account_export_dir,
        Some(std::path::PathBuf::from(
            "./snapshot/access/service-accounts"
        ))
    );
    assert_eq!(
        overview_args.output_format,
        OverviewOutputFormat::Interactive
    );

    let document = json!({
        "kind": "grafana-utils-snapshot-review",
        "schemaVersion": 1,
        "summary": {
            "orgCount": 2,
            "dashboardOrgCount": 2,
            "datasourceOrgCount": 1,
            "dashboardCount": 3,
            "datasourceCount": 4
        },
        "orgs": [
            {
                "org": "Main Org.",
                "orgId": "1",
                "dashboardCount": 2,
                "datasourceCount": 3
            }
        ],
        "warnings": [
            {
                "code": "org-partial-coverage",
                "message": "Org Main Org. (orgId=1) has 2 dashboard(s) and 3 datasource(s)."
            }
        ]
    });

    let summary_lines = crate::snapshot::build_snapshot_review_summary_lines(&document).unwrap();
    assert!(summary_lines.iter().any(|line| line
        .contains("Org coverage: 2 combined org(s), 2 dashboard org(s), 1 datasource org(s)")));
    assert!(summary_lines
        .iter()
        .any(|line| line.contains("Warnings: 1")));
}

#[test]
fn snapshot_review_parses_all_supported_output_modes() {
    let cases = [
        ("table", OverviewOutputFormat::Table),
        ("csv", OverviewOutputFormat::Csv),
        ("text", OverviewOutputFormat::Text),
        ("json", OverviewOutputFormat::Json),
        ("yaml", OverviewOutputFormat::Yaml),
    ];

    for (output, expected) in cases {
        let review_args = SnapshotReviewArgs {
            input_dir: std::path::PathBuf::from("./snapshot"),
            interactive: false,
            output_format: expected,
        };
        let overview_args = build_snapshot_overview_args(&review_args);

        assert_eq!(overview_args.output_format, expected);
        assert_eq!(
            match SnapshotCliArgs::parse_from([
                "grafana-util",
                "review",
                "--input-dir",
                "./snapshot",
                "--output-format",
                output,
            ])
            .command
            {
                crate::snapshot::SnapshotCommand::Review(review) => review.output_format,
                other => panic!("expected snapshot review, got {:?}", other),
            },
            expected
        );
    }
}
