use clap::{ColorChoice, ValueEnum};
use regex::Regex;
use serde::Serialize;
use std::cell::Cell;
use std::io::IsTerminal;
use std::sync::OnceLock;

use super::Result;

thread_local! {
    static JSON_COLOR_CHOICE: Cell<CliColorChoice> = const { Cell::new(CliColorChoice::Auto) };
}

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_JSON_KEY: &str = "\x1b[1;36m";
const ANSI_JSON_STRING: &str = "\x1b[32m";
const ANSI_JSON_NUMBER: &str = "\x1b[33m";
const ANSI_JSON_BOOL: &str = "\x1b[35m";
const ANSI_JSON_NULL: &str = "\x1b[2;90m";
static ANSI_ESCAPE_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JsonContext {
    ObjectExpectKey,
    ObjectExpectValue,
    Array,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CliColorChoice {
    Auto,
    Always,
    #[value(alias = "none", alias = "off")]
    Never,
}

impl From<CliColorChoice> for ColorChoice {
    fn from(value: CliColorChoice) -> Self {
        match value {
            CliColorChoice::Auto => ColorChoice::Auto,
            CliColorChoice::Always => ColorChoice::Always,
            CliColorChoice::Never => ColorChoice::Never,
        }
    }
}

/// Set the thread-local JSON color policy used by shared render helpers.
pub fn set_json_color_choice(choice: CliColorChoice) {
    JSON_COLOR_CHOICE.with(|cell| cell.set(choice));
}

/// Read the current thread-local JSON color policy.
pub fn json_color_choice() -> CliColorChoice {
    JSON_COLOR_CHOICE.with(Cell::get)
}

/// Return whether JSON output should be colorized for the given policy and terminal state.
pub fn json_color_enabled(choice: CliColorChoice, stdout_is_terminal: bool) -> bool {
    match choice {
        CliColorChoice::Always => true,
        CliColorChoice::Never => false,
        CliColorChoice::Auto => stdout_is_terminal,
    }
}

/// Render pretty JSON with the active thread-local color policy.
pub fn render_json_value<T>(payload: &T) -> Result<String>
where
    T: Serialize + ?Sized,
{
    render_json_value_with_choice(
        payload,
        json_color_choice(),
        std::io::stdout().is_terminal(),
    )
}

/// Render pretty JSON with an explicit color policy.
pub fn render_json_value_with_choice<T>(
    payload: &T,
    choice: CliColorChoice,
    stdout_is_terminal: bool,
) -> Result<String>
where
    T: Serialize + ?Sized,
{
    let rendered = serde_json::to_string_pretty(payload)?;
    if json_color_enabled(choice, stdout_is_terminal) {
        Ok(format!("{}\n", colorize_json_pretty(&rendered)))
    } else {
        Ok(format!("{rendered}\n"))
    }
}

/// Remove ANSI escape sequences so persisted output files remain plain text.
pub fn strip_ansi_codes(text: &str) -> String {
    let ansi_re = ANSI_ESCAPE_RE
        .get_or_init(|| Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").expect("valid ANSI regex"));
    ansi_re.replace_all(text, "").into_owned()
}

fn colorize_json_pretty(rendered: &str) -> String {
    let mut colored = String::with_capacity(rendered.len() + 32);
    let mut chars = rendered.chars().peekable();
    let mut contexts: Vec<JsonContext> = Vec::new();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                let mut token = String::from("\"");
                let mut escaped = false;
                for next in chars.by_ref() {
                    token.push(next);
                    if escaped {
                        escaped = false;
                        continue;
                    }
                    if next == '\\' {
                        escaped = true;
                    } else if next == '"' {
                        break;
                    }
                }
                let is_key = matches!(contexts.last(), Some(JsonContext::ObjectExpectKey));
                push_ansi_colored(
                    &mut colored,
                    if is_key {
                        ANSI_JSON_KEY
                    } else {
                        ANSI_JSON_STRING
                    },
                    &token,
                );
            }
            '{' => {
                colored.push(ch);
                contexts.push(JsonContext::ObjectExpectKey);
            }
            '}' => {
                colored.push(ch);
                contexts.pop();
            }
            '[' => {
                colored.push(ch);
                contexts.push(JsonContext::Array);
            }
            ']' => {
                colored.push(ch);
                contexts.pop();
            }
            ':' => {
                colored.push(ch);
                if let Some(context) = contexts.last_mut() {
                    if matches!(context, JsonContext::ObjectExpectKey) {
                        *context = JsonContext::ObjectExpectValue;
                    }
                }
            }
            ',' => {
                colored.push(ch);
                if let Some(context) = contexts.last_mut() {
                    if matches!(context, JsonContext::ObjectExpectValue) {
                        *context = JsonContext::ObjectExpectKey;
                    }
                }
            }
            ch if ch.is_whitespace() => colored.push(ch),
            ch => {
                let mut token = String::from(ch);
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace()
                        || matches!(next, '{' | '}' | '[' | ']' | ':' | ',' | '"')
                    {
                        break;
                    }
                    token.push(chars.next().expect("peeked character should exist"));
                }
                let color = match token.as_str() {
                    "true" | "false" => ANSI_JSON_BOOL,
                    "null" => ANSI_JSON_NULL,
                    _ if token
                        .chars()
                        .next()
                        .is_some_and(|first| first == '-' || first.is_ascii_digit()) =>
                    {
                        ANSI_JSON_NUMBER
                    }
                    _ => "",
                };
                if color.is_empty() {
                    colored.push_str(&token);
                } else {
                    push_ansi_colored(&mut colored, color, &token);
                }
            }
        }
    }

    colored
}

fn push_ansi_colored(output: &mut String, color: &str, token: &str) {
    output.push_str(color);
    output.push_str(token);
    output.push_str(ANSI_RESET);
}
