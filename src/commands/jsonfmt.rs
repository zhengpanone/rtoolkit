use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::Value;

const MAX_INDENT: usize = 16;

#[derive(clap::Args)]
pub struct JsonFmtOpts {
    /// Input JSON file path for backward compatibility
    #[arg(value_name = "INPUT")]
    positional_input: Option<String>,

    /// Input JSON text or JSON file path (omit to read from stdin)
    #[arg(
        short = 'i',
        long,
        value_name = "INPUT",
        help = "输入 JSON 文本或 JSON 文件路径，省略时从 stdin 读取"
    )]
    input: Option<String>,

    /// Output JSON file (omit to write to stdout)
    #[arg(
        short='o',
        long,
        value_name = "output.json",
        help = "输出 JSON 文件，省略时输出到 stdout"
    )]
    output: Option<PathBuf>,

    /// Indent width
    #[arg(long, default_value_t = 2, value_parser = parse_indent, help = "缩进宽度，范围 0-16")]
    indent: usize,

    /// Sort object keys
    #[arg(short, long, help = "是否根据对象键排序")]
    sort: bool,

    /// Output compact JSON
    #[arg(long, help = "压缩JSON")]
    compact: bool,
}

pub fn run_json_fmt(opts: JsonFmtOpts) -> Result<(), JsonFmtError> {
    let input_source = match (opts.input.as_deref(), opts.positional_input.as_deref()) {
        (Some(_), Some(_)) => return Err(JsonFmtError::InputConflict),
        (Some(input), None) | (None, Some(input)) => Some(input),
        (None, None) => None,
    };
    let input = read_json_input(input_source)?;
    let mut value = parse_json_with_fallback(&input).map_err(|e| {
        JsonFmtError::InvalidJson(format!("{} at line {}, column {}", e, e.line(), e.column()))
    })?;

    if opts.sort {
        sort_object_keys(&mut value);
    }

    let output = format_json(&value, opts.indent, opts.compact)?;
    write_json_output(opts.output.as_ref(), &output)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum JsonFmtError {
    #[error("invalid json: {0}")]
    InvalidJson(String),
    #[error("invalid file: {0}")]
    InvalidFile(String),
    #[error("cannot use both positional INPUT and -i/--input")]
    InputConflict,
    #[error("json serialize failed: {0}")]
    Serialize(String),
}

fn read_json_input(input: Option<&str>) -> Result<String, JsonFmtError> {
    match input {
        Some(source) => read_input_source(source),
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| JsonFmtError::InvalidFile(format!("stdin: {}", e)))?;
            Ok(buf)
        }
    }
}

fn parse_indent(value: &str) -> Result<usize, String> {
    let indent = value
        .parse::<usize>()
        .map_err(|_| "indent must be a number".to_string())?;
    if indent <= MAX_INDENT {
        Ok(indent)
    } else {
        Err(format!("indent must be between 0 and {}", MAX_INDENT))
    }
}

fn read_input_source(source: &str) -> Result<String, JsonFmtError> {
    if looks_like_inline_json(source) {
        return Ok(source.to_string());
    }

    let path = Path::new(source);
    fs::read_to_string(path)
        .map_err(|e| JsonFmtError::InvalidFile(format!("{}: {}", path.display(), e)))
}

fn looks_like_inline_json(source: &str) -> bool {
    let trimmed = source.trim_start_matches('\u{feff}').trim_start();
    matches!(
        trimmed.as_bytes().first(),
        Some(b'{')
            | Some(b'[')
            | Some(b'"')
            | Some(b'-')
            | Some(b'0'..=b'9')
            | Some(b't')
            | Some(b'f')
            | Some(b'n')
    )
}

fn write_json_output(output: Option<&PathBuf>, content: &str) -> Result<(), JsonFmtError> {
    match output {
        Some(path) => fs::write(path, content)
            .map_err(|e| JsonFmtError::InvalidFile(format!("{}: {}", path.display(), e))),
        None => {
            print!("{}", content);
            Ok(())
        }
    }
}

fn format_json(value: &Value, indent: usize, compact: bool) -> Result<String, JsonFmtError> {
    let mut output = if compact {
        json_compact(value)?
    } else {
        json_pretty(value, indent)?
    };
    output.push('\n');
    Ok(output)
}

fn parse_json(input: &str) -> serde_json::Result<Value> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    serde_json::from_str(input)
}

fn parse_json_with_fallback(input: &str) -> serde_json::Result<Value> {
    parse_json(input).or_else(|_| {
        let input = input.strip_prefix('\u{feff}').unwrap_or(input);
        serde_json::from_str(&quote_unquoted_object_keys(input))
    })
}

fn quote_unquoted_object_keys(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escaped = false;

    while i < chars.len() {
        let current = chars[i];
        output.push(current);

        if in_string {
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if current == '"' {
            in_string = true;
            i += 1;
            continue;
        }

        if current == '{' || current == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                output.push(chars[j]);
                j += 1;
            }

            if j < chars.len() && is_key_start(chars[j]) {
                let key_start = j;
                j += 1;
                while j < chars.len() && is_key_char(chars[j]) {
                    j += 1;
                }

                let mut colon = j;
                while colon < chars.len() && chars[colon].is_whitespace() {
                    colon += 1;
                }

                if colon < chars.len() && chars[colon] == ':' {
                    output.push('"');
                    for key in &chars[key_start..j] {
                        output.push(*key);
                    }
                    output.push('"');
                    i = j;
                    continue;
                }
            }
        }

        i += 1;
    }

    output
}

fn is_key_start(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_'
}

fn is_key_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_' || value == '-'
}

fn json_pretty(value: &Value, indent: usize) -> Result<String, JsonFmtError> {
    let mut writer = Vec::new();
    let indent = " ".repeat(indent);
    let formatter = PrettyFormatter::with_indent(indent.as_bytes());
    let mut serializer = Serializer::with_formatter(&mut writer, formatter);
    value
        .serialize(&mut serializer)
        .map_err(|e| JsonFmtError::Serialize(e.to_string()))?;
    String::from_utf8(writer).map_err(|e| JsonFmtError::Serialize(e.to_string()))
}

fn json_compact(value: &Value) -> Result<String, JsonFmtError> {
    serde_json::to_string(value).map_err(|e| JsonFmtError::Serialize(e.to_string()))
}

fn sort_object_keys(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for child in map.values_mut() {
                sort_object_keys(child);
            }
            let mut entries: Vec<_> = std::mem::take(map).into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            map.extend(entries);
        }

        Value::Array(items) => {
            for item in items {
                sort_object_keys(item);
            }
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_pretty_with_custom_indent() {
        let value: Value = serde_json::from_str(r#"{"name":"rtoolkit","items":[1,2]}"#).unwrap();
        let output = format_json(&value, 4, false).unwrap();

        assert!(output.contains("\n    \"name\""));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn formats_compact_json() {
        let value: Value = serde_json::from_str(r#"{"name":"rtoolkit","items":[1,2]}"#).unwrap();
        let output = format_json(&value, 2, true).unwrap();

        assert_eq!(serde_json::from_str::<Value>(&output).unwrap(), value);
        assert_eq!(output.matches('\n').count(), 1);
    }

    #[test]
    fn parses_utf8_bom_input() {
        let value = parse_json_with_fallback("\u{feff}{\"ok\":true}").unwrap();

        assert_eq!(value["ok"], true);
    }

    #[test]
    fn treats_inline_json_as_input_source() {
        let input = read_input_source(r#"{"b":2,"a":1}"#).unwrap();

        assert_eq!(input, r#"{"b":2,"a":1}"#);
    }

    #[test]
    fn invalid_inline_json_is_reported_as_json_error() {
        let input = read_input_source("{bad").unwrap();
        let error = parse_json_with_fallback(&input).unwrap_err();

        assert!(error.is_syntax());
    }

    #[test]
    fn parses_shell_stripped_object_keys() {
        let value = parse_json_with_fallback("{b:2,a:1}").unwrap();

        assert_eq!(value["a"], 1);
        assert_eq!(value["b"], 2);
    }

    #[test]
    fn keeps_input_key_order_by_default() {
        let value = parse_json_with_fallback(r#"{"b":2,"a":1}"#).unwrap();
        let output = format_json(&value, 2, true).unwrap();

        assert_eq!(output, "{\"b\":2,\"a\":1}\n");
    }

    #[test]
    fn sorts_nested_object_keys() {
        let mut value: Value = parse_json_with_fallback(r#"{"b":1,"a":{"d":4,"c":3}}"#).unwrap();
        sort_object_keys(&mut value);
        let output = format_json(&value, 2, true).unwrap();

        assert_eq!(output, "{\"a\":{\"c\":3,\"d\":4},\"b\":1}\n");
    }
}
