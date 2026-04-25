use serde_json::{Map, Value};

#[derive(Clone, Debug)]
struct YamlLine {
    line_number: usize,
    indent: usize,
    text: String,
}

pub(super) fn parse_governance_policy_yaml(raw: &str) -> std::result::Result<Value, String> {
    let lines = collect_yaml_lines(raw)?;
    let (value, next_index) = parse_yaml_block(&lines, 0)?;
    if next_index != lines.len() {
        let line_number = lines
            .get(next_index)
            .map(|line| line.line_number)
            .unwrap_or_else(|| lines.last().map(|line| line.line_number).unwrap_or(1));
        return Err(format!(
            "Unexpected trailing content in YAML governance policy on line {}.",
            line_number
        ));
    }
    Ok(value)
}

fn collect_yaml_lines(raw: &str) -> std::result::Result<Vec<YamlLine>, String> {
    let mut lines = Vec::new();
    for (index, raw_line) in raw.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed == "---" || trimmed == "..." || trimmed.starts_with('#') {
            continue;
        }
        if raw_line.contains('\t') {
            return Err(format!(
                "YAML governance policy cannot use tab indentation on line {}.",
                index + 1
            ));
        }
        let indent = raw_line.chars().take_while(|ch| *ch == ' ').count();
        lines.push(YamlLine {
            line_number: index + 1,
            indent,
            text: raw_line[indent..].trim_end().to_string(),
        });
    }
    Ok(lines)
}

fn split_yaml_mapping_pair(text: &str) -> Option<(&str, &str)> {
    let colon = text.find(':')?;
    let key = text[..colon].trim();
    let raw_value = &text[colon + 1..];
    if key.is_empty() {
        return None;
    }
    if raw_value.is_empty() || raw_value.starts_with(' ') {
        Some((key, raw_value.trim_start()))
    } else {
        None
    }
}

fn parse_yaml_scalar(text: &str) -> Value {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Value::Null;
    }
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return value;
    }
    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
        return Value::String(trimmed[1..trimmed.len() - 1].replace("''", "'"));
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "null" | "~" => Value::Null,
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => trimmed
            .parse::<i64>()
            .map(|value| Value::Number(value.into()))
            .unwrap_or_else(|_| Value::String(trimmed.to_string())),
    }
}

fn parse_yaml_block(
    lines: &[YamlLine],
    start_index: usize,
) -> std::result::Result<(Value, usize), String> {
    let Some(first_line) = lines.get(start_index) else {
        return Err("YAML governance policy document is empty.".to_string());
    };
    if first_line.text.trim_start().starts_with('-') {
        parse_yaml_sequence(lines, start_index, first_line.indent)
    } else {
        parse_yaml_mapping(lines, start_index, first_line.indent)
    }
}

fn parse_yaml_mapping(
    lines: &[YamlLine],
    mut index: usize,
    indent: usize,
) -> std::result::Result<(Value, usize), String> {
    let mut map = Map::new();
    while let Some(line) = lines.get(index) {
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(format!(
                "Unexpected indentation in YAML governance policy on line {}.",
                line.line_number
            ));
        }
        let Some((key, value_text)) = split_yaml_mapping_pair(&line.text) else {
            return Err(format!(
                "Expected a YAML mapping entry on line {}.",
                line.line_number
            ));
        };
        index += 1;
        let value = if value_text.is_empty() {
            if let Some(next_line) = lines.get(index) {
                if next_line.indent > indent {
                    let (child_value, next_index) = parse_yaml_block(lines, index)?;
                    index = next_index;
                    child_value
                } else {
                    Value::Null
                }
            } else {
                Value::Null
            }
        } else {
            if let Some(next_line) = lines.get(index) {
                if next_line.indent > indent {
                    return Err(format!(
                        "YAML mapping entry on line {} cannot also own an indented block.",
                        line.line_number
                    ));
                }
            }
            parse_yaml_scalar(value_text)
        };
        map.insert(key.to_string(), value);
    }
    Ok((Value::Object(map), index))
}

fn parse_yaml_sequence_item(
    after_dash: &str,
    child_value: Option<Value>,
    line_number: usize,
) -> std::result::Result<Value, String> {
    if after_dash.is_empty() {
        return Ok(child_value.unwrap_or(Value::Null));
    }

    if let Some((key, value_text)) = split_yaml_mapping_pair(after_dash) {
        let mut map = Map::new();
        if value_text.is_empty() {
            map.insert(key.to_string(), child_value.unwrap_or(Value::Null));
            return Ok(Value::Object(map));
        }
        map.insert(key.to_string(), parse_yaml_scalar(value_text));
        if let Some(Value::Object(child_map)) = child_value {
            for (child_key, child_value) in child_map {
                map.insert(child_key, child_value);
            }
            return Ok(Value::Object(map));
        }
        if child_value.is_some() {
            return Err(format!(
                "YAML sequence item on line {} cannot own a nested block after a scalar value.",
                line_number
            ));
        }
        return Ok(Value::Object(map));
    }

    if child_value.is_some() {
        return Err(format!(
            "YAML sequence item on line {} cannot own an indented block after a scalar value.",
            line_number
        ));
    }
    Ok(parse_yaml_scalar(after_dash))
}

fn parse_yaml_sequence(
    lines: &[YamlLine],
    mut index: usize,
    indent: usize,
) -> std::result::Result<(Value, usize), String> {
    let mut items = Vec::new();
    while let Some(line) = lines.get(index) {
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(format!(
                "Unexpected indentation in YAML governance policy on line {}.",
                line.line_number
            ));
        }
        let trimmed = line.text.trim_start();
        if !trimmed.starts_with('-') {
            break;
        }
        let after_dash = trimmed[1..].trim_start();
        index += 1;
        let child_value = if let Some(next_line) = lines.get(index) {
            if next_line.indent > indent {
                let (child_value, next_index) = parse_yaml_block(lines, index)?;
                index = next_index;
                Some(child_value)
            } else {
                None
            }
        } else {
            None
        };
        items.push(parse_yaml_sequence_item(
            after_dash,
            child_value,
            line.line_number,
        )?);
    }
    Ok((Value::Array(items), index))
}
