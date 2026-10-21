use core::fmt::Write;

use crate::scan::{
    is_disallowed, is_identifier_char, is_newline, is_reserved_keyword, split_radix_prefix,
};
use crate::{KdlDocument, KdlEntry, KdlNode, KdlNumber, KdlValue};

/// `KdlDocument` を正規化形式の文字列に変換する。
///
/// 正規化ルールはクレート README を参照。
pub fn normalize(doc: &KdlDocument) -> String {
    let mut out = String::new();
    for node in &doc.nodes {
        write_node(&mut out, node, 0);
    }

    if out.is_empty() {
        out.push('\n');
    }
    out
}

fn write_node(out: &mut String, node: &KdlNode, depth: usize) {
    let indent = "    ".repeat(depth);
    out.push_str(&indent);

    if let Some(ty) = &node.ty {
        out.push('(');
        write_identifier_or_string(out, ty);
        out.push(')');
    }

    write_identifier_or_string(out, &node.name);

    let mut args: Vec<&KdlEntry> = Vec::new();
    let mut props: Vec<(&str, &KdlEntry)> = Vec::new();

    for entry in &node.entries {
        match entry {
            KdlEntry::Argument { .. } => args.push(entry),
            KdlEntry::Property { key, .. } => {
                if let Some(existing) = props.iter_mut().find(|(k, _)| *k == key.as_str()) {
                    existing.1 = entry;
                } else {
                    props.push((key, entry));
                }
            }
        }
    }

    props.sort_by_key(|(k, _)| *k);

    for entry in &args {
        out.push(' ');
        write_entry(out, entry);
    }

    for (_, entry) in &props {
        out.push(' ');
        write_entry(out, entry);
    }

    match &node.children {
        Some(children) if !children.is_empty() => {
            out.push_str(" {\n");
            for child in children {
                write_node(out, child, depth + 1);
            }
            out.push_str(&indent);
            out.push_str("}\n");
        }
        _ => {
            out.push('\n');
        }
    }
}

fn write_entry(out: &mut String, entry: &KdlEntry) {
    match entry {
        KdlEntry::Argument { ty, value } => {
            if let Some(ty) = ty {
                out.push('(');
                write_identifier_or_string(out, ty);
                out.push(')');
            }
            write_value(out, value);
        }
        KdlEntry::Property { key, ty, value } => {
            write_identifier_or_string(out, key);
            out.push('=');
            if let Some(ty) = ty {
                out.push('(');
                write_identifier_or_string(out, ty);
                out.push(')');
            }
            write_value(out, value);
        }
    }
}

fn write_value(out: &mut String, value: &KdlValue) {
    match value {
        KdlValue::String(s) => write_identifier_or_string(out, s),
        KdlValue::Number(n) => out.push_str(&normalize_number(n)),
        KdlValue::Bool(true) => out.push_str("#true"),
        KdlValue::Bool(false) => out.push_str("#false"),
        KdlValue::Null => out.push_str("#null"),
    }
}

fn write_quoted_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if is_newline(c) || is_disallowed(c) => {
                write!(out, "\\u{{{:04X}}}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_identifier_or_string(out: &mut String, s: &str) {
    if can_be_bare_identifier(s) {
        out.push_str(s);
    } else {
        write_quoted_string(out, s);
    }
}

fn can_be_bare_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    if !is_identifier_char(first) {
        return false;
    }
    for ch in chars {
        if !is_identifier_char(ch) {
            return false;
        }
    }

    if is_reserved_keyword(s) {
        return false;
    }

    let is_sign = first == '+' || first == '-';
    let is_dot = first == '.';
    let is_digit = first.is_ascii_digit();

    if is_digit {
        return false;
    }

    if is_sign {
        let second = s.chars().nth(1);
        match second {
            None => return true,
            Some(c) if c.is_ascii_digit() => return false,
            Some('.') => {
                let third = s.chars().nth(2);
                if let Some(c) = third {
                    if c.is_ascii_digit() {
                        return false;
                    }
                }
            }
            _ => {}
        }
    }

    if is_dot {
        let second = s.chars().nth(1);
        if let Some(c) = second {
            if c.is_ascii_digit() {
                return false;
            }
        }
    }

    true
}

fn normalize_number(n: &KdlNumber) -> String {
    let raw = &n.raw;

    if raw.starts_with('#') {
        return raw.clone();
    }

    let (sign, rest) = if let Some(r) = raw.strip_prefix('-') {
        ("-", r)
    } else if let Some(r) = raw.strip_prefix('+') {
        ("", r)
    } else {
        ("", raw.as_str())
    };

    if let Some((radix, digits)) = split_radix_prefix(rest) {
        let clean: String = digits.chars().filter(|&c| c != '_').collect();
        return format_prefixed_number(sign, &clean, radix);
    }

    let clean: String = rest.chars().filter(|&c| c != '_').collect();

    if let Some(e_pos) = clean.find(['e', 'E']) {
        let mantissa = &clean[..e_pos];
        let exp_part = &clean[e_pos + 1..];
        let normalized_exp = normalize_exponent(exp_part);
        let normalized_mantissa = normalize_decimal_mantissa(mantissa);

        if sign == "-" {
            format!("-{}E{}", normalized_mantissa, normalized_exp)
        } else {
            format!("{}E{}", normalized_mantissa, normalized_exp)
        }
    } else if clean.contains('.') {
        let normalized = normalize_decimal_mantissa(&clean);
        if sign == "-" {
            format!("-{normalized}")
        } else {
            normalized
        }
    } else {
        let normalized = normalize_decimal_integer(&clean);
        if sign == "-" && normalized != "0" {
            format!("-{normalized}")
        } else {
            normalized
        }
    }
}

fn format_prefixed_number(sign: &str, digits: &str, radix: u32) -> String {
    if let Ok(val) = u128::from_str_radix(digits, radix) {
        if sign == "-" && val != 0 {
            format!("-{val}")
        } else {
            format!("{val}")
        }
    } else {
        let decimal = big_radix_to_decimal(digits, radix);
        if sign == "-" && decimal != "0" {
            format!("-{decimal}")
        } else {
            decimal
        }
    }
}

fn big_radix_to_decimal(digits: &str, radix: u32) -> String {
    let mut result: Vec<u8> = vec![0];

    for ch in digits.chars() {
        let digit_val = ch.to_digit(radix).expect("parser-validated digit") as u8;

        let mut carry: u16 = 0;
        for d in result.iter_mut() {
            let prod = (*d as u16) * (radix as u16) + carry;
            *d = (prod % 10) as u8;
            carry = prod / 10;
        }
        while carry > 0 {
            result.push((carry % 10) as u8);
            carry /= 10;
        }

        let mut carry: u16 = digit_val as u16;
        for d in result.iter_mut() {
            let sum = (*d as u16) + carry;
            *d = (sum % 10) as u8;
            carry = sum / 10;
            if carry == 0 {
                break;
            }
        }
        while carry > 0 {
            result.push((carry % 10) as u8);
            carry /= 10;
        }
    }

    while result.len() > 1 && *result.last().unwrap() == 0 {
        result.pop();
    }
    result.iter().rev().map(|d| (b'0' + d) as char).collect()
}

fn normalize_decimal_integer(s: &str) -> String {
    let trimmed = s.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_decimal_mantissa(s: &str) -> String {
    if let Some(dot_pos) = s.find('.') {
        let int_part = &s[..dot_pos];
        let frac_part = &s[dot_pos + 1..];
        let int_normalized = normalize_decimal_integer(int_part);
        format!("{int_normalized}.{frac_part}")
    } else {
        normalize_decimal_integer(s)
    }
}

fn normalize_exponent(s: &str) -> String {
    if let Some(rest) = s.strip_prefix('-') {
        format!("-{}", normalize_decimal_integer(rest))
    } else if let Some(rest) = s.strip_prefix('+') {
        format!("+{}", normalize_decimal_integer(rest))
    } else {
        format!("+{}", normalize_decimal_integer(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KdlEntry, KdlNode, KdlNumber, KdlValue};

    fn make_doc(nodes: Vec<KdlNode>) -> KdlDocument {
        KdlDocument { nodes }
    }

    fn make_node(name: &str) -> KdlNode {
        KdlNode {
            ty: None,
            name: name.to_string(),
            entries: Vec::new(),
            children: None,
        }
    }

    #[test]
    fn empty_document() {
        let doc = make_doc(vec![]);
        assert_eq!(normalize(&doc), "\n");
    }

    #[test]
    fn basic_node() {
        let mut node = make_node("node");
        node.entries.push(KdlEntry::Argument {
            ty: None,
            value: KdlValue::String("hello".to_string()),
        });
        let doc = make_doc(vec![node]);

        assert_eq!(normalize(&doc), "node hello\n");
    }

    #[test]
    fn property_sort_and_dedup() {
        let mut node = make_node("node");
        node.entries.push(KdlEntry::Property {
            key: "b".to_string(),
            ty: None,
            value: KdlValue::String("1".to_string()),
        });
        node.entries.push(KdlEntry::Property {
            key: "a".to_string(),
            ty: None,
            value: KdlValue::String("2".to_string()),
        });
        node.entries.push(KdlEntry::Property {
            key: "b".to_string(),
            ty: None,
            value: KdlValue::String("3".to_string()),
        });
        let doc = make_doc(vec![node]);
        assert_eq!(normalize(&doc), "node a=\"2\" b=\"3\"\n");
    }

    #[test]
    fn number_normalization() {
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "0xff".to_string(),
                as_i64: Some(255),
                as_f64: Some(255.0),
            }),
            "255"
        );
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "0b1010".to_string(),
                as_i64: Some(10),
                as_f64: Some(10.0),
            }),
            "10"
        );
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "1.0e+10".to_string(),
                as_i64: None,
                as_f64: Some(1e10),
            }),
            "1.0E+10"
        );
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "1.23E+1000".to_string(),
                as_i64: None,
                as_f64: None,
            }),
            "1.23E+1000"
        );
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "#inf".to_string(),
                as_i64: None,
                as_f64: Some(f64::INFINITY),
            }),
            "#inf"
        );
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "+10".to_string(),
                as_i64: Some(10),
                as_f64: Some(10.0),
            }),
            "10"
        );
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "-0".to_string(),
                as_i64: Some(0),
                as_f64: Some(0.0),
            }),
            "0"
        );
    }

    #[test]
    fn type_annotation() {
        let mut node = make_node("node");
        node.ty = Some("mytype".to_string());
        node.entries.push(KdlEntry::Argument {
            ty: Some("u8".to_string()),
            value: KdlValue::Number(KdlNumber {
                raw: "42".to_string(),
                as_i64: Some(42),
                as_f64: Some(42.0),
            }),
        });
        let doc = make_doc(vec![node]);
        assert_eq!(normalize(&doc), "(mytype)node (u8)42\n");
    }

    #[test]
    fn empty_children_omitted() {
        let mut node = make_node("node");
        node.children = Some(vec![]);
        let doc = make_doc(vec![node]);
        assert_eq!(normalize(&doc), "node\n");
    }

    #[test]
    fn nested_children() {
        let inner = make_node("child");
        let mut node = make_node("parent");
        node.children = Some(vec![inner]);
        let doc = make_doc(vec![node]);
        assert_eq!(normalize(&doc), "parent {\n    child\n}\n");
    }

    #[test]
    fn identifier_judgment() {
        assert!(can_be_bare_identifier("foo"));
        assert!(can_be_bare_identifier("foo-bar"));
        assert!(can_be_bare_identifier("+"));
        assert!(can_be_bare_identifier("-"));
        assert!(can_be_bare_identifier(".md"));

        assert!(!can_be_bare_identifier(""));
        assert!(!can_be_bare_identifier("true"));
        assert!(!can_be_bare_identifier("false"));
        assert!(!can_be_bare_identifier("null"));
        assert!(!can_be_bare_identifier("inf"));
        assert!(!can_be_bare_identifier("-inf"));
        assert!(!can_be_bare_identifier("nan"));
        assert!(!can_be_bare_identifier("0node"));
        assert!(!can_be_bare_identifier("hello world"));
    }

    #[test]
    fn big_hex_conversion() {
        assert_eq!(
            normalize_number(&KdlNumber {
                raw: "0xABCDEF0123456789abcdef".to_string(),
                as_i64: None,
                as_f64: None,
            }),
            "207698809136909011942886895"
        );
    }

    #[test]
    fn keywords_and_null() {
        let mut node = make_node("node");
        node.entries.push(KdlEntry::Argument {
            ty: None,
            value: KdlValue::Bool(true),
        });
        node.entries.push(KdlEntry::Argument {
            ty: None,
            value: KdlValue::Bool(false),
        });
        node.entries.push(KdlEntry::Argument {
            ty: None,
            value: KdlValue::Null,
        });
        let doc = make_doc(vec![node]);
        assert_eq!(normalize(&doc), "node #true #false #null\n");
    }
}
