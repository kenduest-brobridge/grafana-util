use std::path::Path;

use serde::Serialize;

use crate::common::{message, Result};

use super::super::super::write_json_document;

pub(super) fn write_history_document<T: Serialize>(
    payload: &T,
    output_path: &Path,
    overwrite: bool,
) -> Result<()> {
    if output_path.exists() && !overwrite {
        return Err(message(format!(
            "Refusing to overwrite existing file: {}. Use --overwrite.",
            output_path.display()
        )));
    }
    write_json_document(payload, output_path)
}
