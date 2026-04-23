use regex::Regex;

/// Normalize user-provided strings into filesystem-safe path components.
pub fn sanitize_path_component(value: &str) -> String {
    let invalid = Regex::new(r"[^\w.\- ]+").expect("invalid hard-coded regex");
    let spaces = Regex::new(r"\s+").expect("invalid hard-coded regex");
    let duplicate_underscores = Regex::new(r"_+").expect("invalid hard-coded regex");

    let normalized = invalid.replace_all(value.trim(), "_");
    let normalized = spaces.replace_all(normalized.as_ref(), "_");
    let normalized = duplicate_underscores.replace_all(normalized.as_ref(), "_");
    let normalized = normalized.trim_matches(|character| character == '.' || character == '_');
    if normalized.is_empty() {
        "untitled".to_string()
    } else {
        normalized.to_string()
    }
}
