use serde::Serialize;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::common::Result;
use crate::dashboard::{ROOT_INDEX_KIND, TOOL_SCHEMA_VERSION};

use super::super::{
    build_export_metadata, DashboardIndexItem, ExportOrgSummary, FolderInventoryItem,
    EXPORT_METADATA_FILENAME,
};

#[derive(Serialize)]
struct RootExportIndexView<'a> {
    #[serde(rename = "schemaVersion")]
    schema_version: i64,
    #[serde(rename = "toolVersion")]
    tool_version: Option<String>,
    kind: &'a str,
    items: &'a [DashboardIndexItem],
    variants: RootExportVariantsView<'a>,
    #[serde(default)]
    folders: &'a [FolderInventoryItem],
}

#[derive(Serialize)]
struct RootExportVariantsView<'a> {
    raw: Option<&'a str>,
    prompt: Option<&'a str>,
    provisioning: Option<&'a str>,
}

fn write_json_document_streaming<T: Serialize>(payload: &T, output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, payload)?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub(super) fn write_all_orgs_root_export_bundle(
    output_dir: &Path,
    root_items: &[DashboardIndexItem],
    root_folders: &[FolderInventoryItem],
    org_summaries: Vec<ExportOrgSummary>,
    source_url: &str,
    source_profile: Option<&str>,
) -> Result<()> {
    let root_index = RootExportIndexView {
        schema_version: TOOL_SCHEMA_VERSION,
        tool_version: Some(crate::common::tool_version().to_string()),
        kind: ROOT_INDEX_KIND,
        items: root_items,
        variants: RootExportVariantsView {
            raw: None,
            prompt: None,
            provisioning: None,
        },
        folders: root_folders,
    };
    write_json_document_streaming(&root_index, &output_dir.join("index.json"))?;
    write_json_document_streaming(
        &build_export_metadata(
            "root",
            root_items.len(),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(org_summaries),
            "live",
            Some(source_url),
            None,
            source_profile,
            output_dir,
            &output_dir.join(EXPORT_METADATA_FILENAME),
        ),
        &output_dir.join(EXPORT_METADATA_FILENAME),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use tempfile::tempdir;

    #[test]
    fn write_all_orgs_root_export_bundle_streams_root_index_without_changing_shape() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path();
        let root_items = vec![DashboardIndexItem {
            uid: "cpu-main".to_string(),
            title: "CPU Main".to_string(),
            folder_title: "General".to_string(),
            folder_uid: "general".to_string(),
            folder_path: "General".to_string(),
            org: "Main Org".to_string(),
            org_id: "1".to_string(),
            ownership: "api-managed".to_string(),
            provenance: vec!["ownership=api-managed".to_string()],
            raw_path: Some("/tmp/export/raw/CPU__cpu-main.json".to_string()),
            prompt_path: None,
            provisioning_path: None,
        }];
        let root_folders = vec![FolderInventoryItem {
            uid: "general".to_string(),
            title: "General".to_string(),
            path: "General".to_string(),
            parent_uid: None,
            org: "Main Org".to_string(),
            org_id: "1".to_string(),
        }];

        write_all_orgs_root_export_bundle(
            output_dir,
            &root_items,
            &root_folders,
            vec![ExportOrgSummary {
                org: "Main Org".to_string(),
                org_id: "1".to_string(),
                dashboard_count: 1,
                datasource_count: Some(0),
                used_datasource_count: Some(0),
                used_datasources: Some(Vec::new()),
                output_dir: Some(output_dir.display().to_string()),
            }],
            "http://127.0.0.1:3000",
            Some("prod"),
        )
        .unwrap();

        let index: Value =
            serde_json::from_str(&fs::read_to_string(output_dir.join("index.json")).unwrap())
                .unwrap();
        let metadata: Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join(EXPORT_METADATA_FILENAME)).unwrap(),
        )
        .unwrap();

        assert_eq!(index["kind"], json!(crate::dashboard::ROOT_INDEX_KIND));
        assert_eq!(index["items"].as_array().unwrap().len(), 1);
        assert_eq!(index["folders"].as_array().unwrap().len(), 1);
        assert_eq!(metadata["variant"], json!("root"));
        assert_eq!(metadata["orgCount"], json!(1));
        assert_eq!(metadata["orgs"].as_array().unwrap().len(), 1);
    }
}
