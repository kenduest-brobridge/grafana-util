use crate::common::Result;
use crate::dashboard::ImportArgs;

use super::super::super::import_render::{
    render_import_dry_run_json, render_import_dry_run_table, target_review_counts,
    ImportDryRunReport,
};
use super::super::import_dry_run::folder_inventory_status_output_lines;

pub(super) fn render_dry_run_report(
    report: &ImportDryRunReport,
    args: &ImportArgs,
) -> Result<usize> {
    let (target_warning_count, target_blocked_count) =
        target_review_counts(&report.dashboard_records);
    if args.json {
        print!(
            "{}",
            render_import_dry_run_json(
                &report.mode,
                &report.folder_statuses,
                &report.dashboard_records,
                &report.input_dir,
                report.skipped_missing_count,
                report.skipped_folder_mismatch_count,
            )?
        );
    } else {
        folder_inventory_status_output_lines(
            &report.folder_statuses,
            args.no_header,
            args.json,
            args.table,
        );
        if args.table {
            for line in render_import_dry_run_table(
                &report.dashboard_records,
                !args.no_header,
                if args.output_columns.is_empty() {
                    None
                } else {
                    Some(args.output_columns.as_slice())
                },
            ) {
                println!("{line}");
            }
        } else if args.verbose {
            render_verbose_lines(report);
        } else if args.progress {
            render_progress_lines(report);
        }
        render_summary_line(report, args, target_warning_count, target_blocked_count);
    }
    Ok(report.dashboard_records.len())
}

fn render_verbose_lines(report: &ImportDryRunReport) {
    for record in &report.dashboard_records {
        if record[6].is_empty() {
            if record[3].is_empty() {
                println!(
                    "Dry-run import uid={} dest={} action={} file={}",
                    record[0], record[1], record[2], record[7]
                );
            } else {
                println!(
                    "Dry-run import uid={} dest={} action={} folderPath={} file={}",
                    record[0], record[1], record[2], record[3], record[7]
                );
            }
        } else if record[3].is_empty() {
            println!(
                "Dry-run import uid={} dest={} action={} reason={} file={}",
                record[0], record[1], record[2], record[6], record[7]
            );
        } else {
            println!(
                "Dry-run import uid={} dest={} action={} folderPath={} reason={} file={}",
                record[0], record[1], record[2], record[3], record[6], record[7]
            );
        }
    }
}

fn render_progress_lines(report: &ImportDryRunReport) {
    for (index, record) in report.dashboard_records.iter().enumerate() {
        if record[3].is_empty() {
            println!(
                "Dry-run dashboard {}/{}: {} dest={} action={}",
                index + 1,
                report.dashboard_records.len(),
                record[0],
                record[1],
                record[2]
            );
        } else {
            println!(
                "Dry-run dashboard {}/{}: {} dest={} action={} folderPath={}",
                index + 1,
                report.dashboard_records.len(),
                record[0],
                record[1],
                record[2],
                record[3]
            );
        }
    }
}

fn render_summary_line(
    report: &ImportDryRunReport,
    args: &ImportArgs,
    target_warning_count: usize,
    target_blocked_count: usize,
) {
    if args.update_existing_only
        && report.skipped_missing_count > 0
        && report.skipped_folder_mismatch_count > 0
    {
        println!(
            "Dry-run checked {} dashboard(s) from {}; would skip {} missing dashboards and {} folder-mismatched dashboards",
            report.dashboard_records.len(),
            report.input_dir.display(),
            report.skipped_missing_count,
            report.skipped_folder_mismatch_count
        );
    } else if args.update_existing_only && report.skipped_missing_count > 0 {
        println!(
            "Dry-run checked {} dashboard(s) from {}; would skip {} missing dashboards",
            report.dashboard_records.len(),
            report.input_dir.display(),
            report.skipped_missing_count
        );
    } else if report.skipped_folder_mismatch_count > 0 {
        println!(
            "Dry-run checked {} dashboard(s) from {}; would skip {} folder-mismatched dashboards",
            report.dashboard_records.len(),
            report.input_dir.display(),
            report.skipped_folder_mismatch_count
        );
    } else {
        println!(
            "Dry-run checked {} dashboard(s) from {}{}{}",
            report.dashboard_records.len(),
            report.input_dir.display(),
            if target_warning_count > 0 {
                format!("; target warnings={target_warning_count}")
            } else {
                String::new()
            },
            if target_blocked_count > 0 {
                format!("; target blocked={target_blocked_count}")
            } else {
                String::new()
            }
        );
    }
}
