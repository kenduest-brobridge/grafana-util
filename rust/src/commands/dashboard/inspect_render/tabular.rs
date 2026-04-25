/// Purpose: implementation note.
pub(crate) fn render_csv(headers: &[&str], rows: &[Vec<String>]) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(
        headers
            .iter()
            .map(|value| escape_csv(value))
            .collect::<Vec<String>>()
            .join(","),
    );
    for row in rows {
        lines.push(
            row.iter()
                .map(|value| escape_csv(value))
                .collect::<Vec<String>>()
                .join(","),
        );
    }
    lines
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Purpose: implementation note.
pub(crate) fn render_simple_table(
    headers: &[&str],
    rows: &[Vec<String>],
    include_header: bool,
) -> Vec<String> {
    let mut widths = headers
        .iter()
        .map(|value| value.len())
        .collect::<Vec<usize>>();
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            if index >= widths.len() {
                widths.push(value.len());
            } else {
                widths[index] = widths[index].max(value.len());
            }
        }
    }
    let format_row = |values: &[String]| -> String {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{value:<width$}", width = widths[index]))
            .collect::<Vec<String>>()
            .join("  ")
    };
    let mut lines = Vec::new();
    if include_header {
        let header_row = headers
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>();
        let divider_row = widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<String>>();
        lines.push(format_row(&header_row));
        lines.push(format_row(&divider_row));
    }
    for row in rows {
        lines.push(format_row(row));
    }
    lines
}
