#![doc = include_str!("../README.md")]

#[derive(Debug, Clone, PartialEq)]
pub enum Json5Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Json5Value>),
    Map(Vec<(String, Json5Value)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub message: String,
}

impl ParseError {
    fn new(position: usize, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
        }
    }
}

pub fn parse(input: &str) -> Result<Json5Value, ParseError> {
    if input.trim_start().starts_with('{') {
        parse_json5_like(input)
    } else {
        parse_lines(input, "=")
    }
}

#[allow(dead_code)]
fn parse_lines(input: &str, sep: &str) -> Result<Json5Value, ParseError> {
    let mut fields = Vec::new();
    let mut current_key: Option<String> = None;
    for (line_no, raw) in input.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("- ") {
            let key = current_key.clone().unwrap_or_else(|| "items".to_string());
            push_list_item(&mut fields, key, parse_scalar(rest));
            continue;
        }
        let Some((k, v)) = line.split_once(sep) else {
            return Err(ParseError::new(line_no, "expected key/value line"));
        };
        let key = k.trim().trim_matches('[').trim_matches(']').to_string();
        let value = v.trim();
        current_key = Some(key.clone());
        if value.is_empty() {
            fields.push((key, Json5Value::List(Vec::new())));
        } else {
            fields.push((key, parse_scalar(value)));
        }
    }
    Ok(Json5Value::Map(fields))
}

#[allow(dead_code)]
fn push_list_item(fields: &mut Vec<(String, Json5Value)>, key: String, value: Json5Value) {
    if let Some((_, Json5Value::List(items))) = fields.iter_mut().rev().find(|(k, _)| *k == key) {
        items.push(value);
    } else {
        fields.push((key, Json5Value::List(vec![value])));
    }
}

fn parse_json5_like(input: &str) -> Result<Json5Value, ParseError> {
    let body = input.trim().trim_start_matches('{').trim_end_matches('}');
    let mut fields = Vec::new();
    for part in split_top_level(body) {
        let part = part.trim();
        if part.is_empty() || part.starts_with("//") {
            continue;
        }
        let Some((k, v)) = part.split_once(':') else {
            return Err(ParseError::new(0, "expected object field"));
        };
        fields.push((
            k.trim().trim_matches('"').trim_matches('\'').to_string(),
            parse_scalar(v.trim()),
        ));
    }
    Ok(Json5Value::Map(fields))
}

fn split_top_level(input: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    for (idx, ch) in input.char_indices() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '[' | '{' => depth += 1,
            ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(&input[start..idx]);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&input[start..]);
    out
}

#[allow(dead_code)]
fn parse_xml_like(input: &str) -> Result<Json5Value, ParseError> {
    let mut fields = Vec::new();
    let mut rest = input.trim();
    if let Some(start) = rest.find('>') {
        rest = &rest[start + 1..];
    }
    while let Some(open) = rest.find('<') {
        let after = &rest[open + 1..];
        if after.starts_with('/') {
            break;
        }
        let Some(end_name) = after.find('>') else {
            return Err(ParseError::new(open, "unterminated tag"));
        };
        let name = after[..end_name].trim().trim_end_matches('/').to_string();
        rest = &after[end_name + 1..];
        if after[..end_name].trim_end().ends_with('/') {
            fields.push((name, Json5Value::String(String::new())));
            continue;
        }
        let close = format!("</{}>", name);
        let Some(close_pos) = rest.find(&close) else {
            return Err(ParseError::new(open, "missing close tag"));
        };
        let text = rest[..close_pos].trim();
        let value = if text.starts_with('<') {
            parse_xml_like(text)?
        } else {
            parse_scalar(text)
        };
        fields.push((name, value));
        rest = &rest[close_pos + close.len()..];
    }
    Ok(Json5Value::Map(fields))
}

fn parse_scalar(raw: &str) -> Json5Value {
    let s = raw
        .trim()
        .trim_end_matches(',')
        .trim_matches('"')
        .trim_matches('\'');
    if s.eq_ignore_ascii_case("true") {
        return Json5Value::Bool(true);
    }
    if s.eq_ignore_ascii_case("false") {
        return Json5Value::Bool(false);
    }
    if s.eq_ignore_ascii_case("null") || s == "~" {
        return Json5Value::Null;
    }
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        return Json5Value::List(
            inner
                .split(',')
                .filter(|p| !p.trim().is_empty())
                .map(parse_scalar)
                .collect(),
        );
    }
    if let Ok(n) = s.parse::<f64>() {
        return Json5Value::Number(n);
    }
    Json5Value::String(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "{name: 'neco', enabled: true, nums: [1, 2,],}";

    #[test]
    fn case_01() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_02() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_03() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_04() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_05() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_06() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_07() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_08() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_09() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_10() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_11() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_12() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_13() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_14() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_15() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_16() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_17() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_18() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_19() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_20() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_21() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_22() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_23() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_24() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_25() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_26() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_27() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_28() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_29() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }
    #[test]
    fn case_30() {
        let v = parse(SAMPLE).expect("parse");
        assert!(matches!(v, Json5Value::Map(_)));
        assert!(matches!(v, Json5Value::Map(_)));
    }

    #[test]
    fn parses_attribute_string() {
        let v = parse(SAMPLE).expect("parse");
        assert!(map_has_string(&v, "name", "neco"));
    }

    #[test]
    fn exposes_children() {
        let v = parse(SAMPLE).expect("parse");
        assert!(map_len(&v) > 0);
    }

    fn map_has_string(value: &Json5Value, key: &str, expected: &str) -> bool {
        match value {
            Json5Value::Map(fields) => fields
                .iter()
                .any(|(k, v)| k == key && matches!(v, Json5Value::String(s) if s == expected)),
            _ => false,
        }
    }

    fn map_len(value: &Json5Value) -> usize {
        match value {
            Json5Value::Map(fields) => fields.len(),
            _ => 0,
        }
    }
}
