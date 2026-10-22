use alloc::vec::Vec;

use crate::{EncodeError, JsonValue};

/// Encodes a `JsonValue` into its minimal JSON byte representation.
pub fn encode(value: &JsonValue) -> Result<Vec<u8>, EncodeError> {
    let mut output = Vec::new();
    encode_value(value, &mut output)?;
    Ok(output)
}

fn encode_value(value: &JsonValue, output: &mut Vec<u8>) -> Result<(), EncodeError> {
    match value {
        JsonValue::Null => output.extend_from_slice(b"null"),
        JsonValue::Bool(true) => output.extend_from_slice(b"true"),
        JsonValue::Bool(false) => output.extend_from_slice(b"false"),
        JsonValue::Number(number) => output.extend_from_slice(number.raw().as_bytes()),
        JsonValue::String(text) => encode_string(text, output),
        JsonValue::Array(items) => {
            output.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                encode_value(item, output)?;
            }
            output.push(b']');
        }
        JsonValue::Object(fields) => {
            output.push(b'{');
            for (index, (key, value)) in fields.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                encode_string(key, output);
                output.push(b':');
                encode_value(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn encode_string(text: &str, output: &mut Vec<u8>) {
    output.push(b'"');
    for ch in text.chars() {
        match crate::json_string::classify(ch) {
            crate::json_string::Escape::Short(escaped) => {
                output.extend_from_slice(escaped.as_bytes())
            }
            crate::json_string::Escape::Control(code) => {
                encode_control_escape(u32::from(code), output)
            }
            crate::json_string::Escape::Verbatim => {
                let mut buffer = [0_u8; 4];
                let encoded = ch.encode_utf8(&mut buffer);
                output.extend_from_slice(encoded.as_bytes());
            }
        }
    }
    output.push(b'"');
}

fn encode_control_escape(code: u32, output: &mut Vec<u8>) {
    output.extend_from_slice(br#"\u00"#);
    output.push(hex_digit(((code >> 4) & 0x0F) as u8));
    output.push(hex_digit((code & 0x0F) as u8));
}

fn hex_digit(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        10..=15 => b'A' + (value - 10),
        _ => unreachable!("hex digit is always in range"),
    }
}

#[cfg(test)]
mod tests {
    fn number(value: f64) -> JsonValue {
        JsonNumber::from_f64(value).unwrap().into()
    }

    use alloc::vec;

    use super::encode;
    use crate::{parse, EncodeError, JsonNumber, JsonValue};

    #[test]
    fn encodes_basic_variants() {
        assert_eq!(encode(&JsonValue::Null).unwrap(), b"null");
        assert_eq!(encode(&JsonValue::Bool(true)).unwrap(), b"true");
        assert_eq!(encode(&JsonValue::Bool(false)).unwrap(), b"false");
        assert_eq!(encode(&number(42.5)).unwrap(), b"42.5");
        assert_eq!(
            encode(&JsonValue::String("neco".into())).unwrap(),
            b"\"neco\""
        );
        assert_eq!(encode(&JsonValue::Array(vec![])).unwrap(), b"[]");
        assert_eq!(encode(&JsonValue::Object(vec![])).unwrap(), b"{}");
    }

    #[test]
    fn constructor_rejects_non_finite_numbers() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                JsonNumber::from_f64(value),
                Err(EncodeError::NonFiniteNumber)
            );
        }
    }

    #[test]
    fn escapes_strings() {
        let value = JsonValue::String("quote:\" slash:\\\n\r\t".into());
        assert_eq!(encode(&value).unwrap(), br#""quote:\" slash:\\\n\r\t""#);

        let control = JsonValue::String("\u{0000}\u{0008}\u{000C}\u{001f}".into());
        assert_eq!(encode(&control).unwrap(), br#""\u0000\b\f\u001F""#);
    }

    #[test]
    fn encodes_integer_valued_f64_using_current_representation() {
        assert_eq!(encode(&number(42.0)).unwrap(), b"42");
        assert_eq!(encode(&number(-0.0)).unwrap(), b"-0");
    }

    #[test]
    fn escapes_null_character_in_string() {
        let value = JsonValue::String("a\u{0000}b".into());
        assert_eq!(encode(&value).unwrap(), br#""a\u0000b""#);
    }

    #[test]
    fn encodes_nested_structures() {
        let value = JsonValue::Object(vec![
            ("name".into(), JsonValue::String("neco".into())),
            (
                "items".into(),
                JsonValue::Array(vec![
                    JsonValue::Null,
                    JsonValue::Bool(true),
                    JsonValue::Object(vec![("x".into(), number(1.0))]),
                ]),
            ),
        ]);

        assert_eq!(
            encode(&value).unwrap(),
            br#"{"name":"neco","items":[null,true,{"x":1}]}"#
        );
    }

    #[test]
    fn encode_parse_roundtrip_for_finite_values() {
        let value = JsonValue::Object(vec![
            ("null".into(), JsonValue::Null),
            ("bool".into(), JsonValue::Bool(true)),
            ("int".into(), number(7.0)),
            ("float".into(), number(-3.25)),
            ("text".into(), JsonValue::String("ねこ\njson".into())),
            (
                "array".into(),
                JsonValue::Array(vec![number(0.5), JsonValue::String("\"quoted\"".into())]),
            ),
            (
                "object".into(),
                JsonValue::Object(vec![("nested".into(), JsonValue::Bool(false))]),
            ),
        ]);

        let encoded = encode(&value).unwrap();
        let decoded = parse(&encoded).expect("encoded bytes should parse");
        assert_eq!(decoded, value);
    }

    #[test]
    fn encode_parse_roundtrip_for_complex_nested_structure() {
        let value = JsonValue::Object(vec![
            (
                "meta".into(),
                JsonValue::Object(vec![
                    ("version".into(), number(1.0)),
                    (
                        "tags".into(),
                        JsonValue::Array(vec![
                            JsonValue::String("alpha".into()),
                            JsonValue::String("beta".into()),
                            JsonValue::Null,
                        ]),
                    ),
                ]),
            ),
            (
                "items".into(),
                JsonValue::Array(vec![
                    JsonValue::Object(vec![
                        ("id".into(), JsonValue::String("one".into())),
                        ("active".into(), JsonValue::Bool(true)),
                        ("score".into(), number(1.5e2)),
                    ]),
                    JsonValue::Array(vec![
                        JsonValue::String("nested".into()),
                        JsonValue::Object(vec![(
                            "control".into(),
                            JsonValue::String("x\u{0000}y".into()),
                        )]),
                    ]),
                ]),
            ),
            ("empty".into(), JsonValue::Object(vec![])),
        ]);

        let encoded = encode(&value).unwrap();
        let decoded = parse(&encoded).expect("encoded bytes should parse");
        assert_eq!(decoded, value);
    }
}
