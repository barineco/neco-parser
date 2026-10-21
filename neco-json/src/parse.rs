use alloc::string::String;
use alloc::vec::Vec;

use crate::borrowed::{JsonArrayItems, JsonKind, JsonObjectEntries, JsonRef};
use crate::decode_char::DecodedChars;
use crate::scan;
use crate::{JsonValue, ParseError, ParseErrorKind};

/// Parses a JSON byte slice into a `JsonValue`.
pub fn parse(input: &[u8]) -> Result<JsonValue, ParseError> {
    let value = scan::document(input)?;
    let text = core::str::from_utf8(input).map_err(|error| ParseError {
        kind: ParseErrorKind::InvalidUtf8,
        position: error.valid_up_to(),
    })?;
    owned(JsonRef::scanned(text, value))
}

fn string(value: JsonRef<'_>) -> Result<String, ParseError> {
    let mut output = String::new();
    for character in DecodedChars::new(value.input.as_bytes(), value.start)? {
        output.push(character?);
    }
    Ok(output)
}

fn owned(value: JsonRef<'_>) -> Result<JsonValue, ParseError> {
    match value.kind() {
        JsonKind::Null => Ok(JsonValue::Null),
        JsonKind::Bool => Ok(JsonValue::Bool(value.raw() == "true")),
        JsonKind::Number => value
            .raw()
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| ParseError {
                kind: ParseErrorKind::InvalidNumber,
                position: value.end,
            }),
        JsonKind::String => string(value).map(JsonValue::String),
        JsonKind::Array => {
            let mut items = Vec::new();
            for item in JsonArrayItems::new(value) {
                items.push(owned(item?)?);
            }
            Ok(JsonValue::Array(items))
        }
        JsonKind::Object => {
            let mut fields = Vec::new();
            for entry in JsonObjectEntries::new(value) {
                let (key, value) = entry?;
                fields.push((string(key)?, owned(value)?));
            }
            Ok(JsonValue::Object(fields))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use super::parse;
    use crate::{JsonValue, ParseErrorKind};

    #[test]
    fn parses_basic_types() {
        assert_eq!(parse(b"null"), Ok(JsonValue::Null));
        assert_eq!(parse(b"true"), Ok(JsonValue::Bool(true)));
        assert_eq!(parse(b"false"), Ok(JsonValue::Bool(false)));
        assert_eq!(parse(b"0"), Ok(JsonValue::Number(0.0)));
        assert_eq!(parse(b"-12"), Ok(JsonValue::Number(-12.0)));
        assert_eq!(parse(b"3.5"), Ok(JsonValue::Number(3.5)));
        assert_eq!(parse(b"1.25e2"), Ok(JsonValue::Number(125.0)));
        assert_eq!(
            parse(br#""hello""#),
            Ok(JsonValue::String(String::from("hello")))
        );
    }

    #[test]
    fn parses_string_escapes() {
        assert_eq!(
            parse(br#""\"\\\/\b\f\n\r\t""#),
            Ok(JsonValue::String(String::from(
                "\"\\/\u{0008}\u{000C}\n\r\t"
            )))
        );
    }

    #[test]
    fn parses_unicode_escapes_and_surrogate_pairs() {
        assert_eq!(
            parse(br#""\u0041""#),
            Ok(JsonValue::String(String::from("A")))
        );
        assert_eq!(
            parse(br#""\u3042""#),
            Ok(JsonValue::String(String::from("あ")))
        );
        assert_eq!(
            parse(br#""\uD83D\uDE00""#),
            Ok(JsonValue::String(String::from("😀")))
        );
    }

    #[test]
    fn parses_nested_values() {
        assert_eq!(
            parse(br#"{"items":[{"flag":true},[null,{"n":2}]],"name":"ok"}"#),
            Ok(JsonValue::Object(vec![
                (
                    String::from("items"),
                    JsonValue::Array(vec![
                        JsonValue::Object(vec![(String::from("flag"), JsonValue::Bool(true))]),
                        JsonValue::Array(vec![
                            JsonValue::Null,
                            JsonValue::Object(vec![(String::from("n"), JsonValue::Number(2.0))]),
                        ]),
                    ]),
                ),
                (String::from("name"), JsonValue::String(String::from("ok"))),
            ]))
        );
    }

    #[test]
    fn parses_deep_nesting_at_limit() {
        let mut input = vec![b'['; 128];
        input.extend(vec![b']'; 128]);

        assert!(parse(&input).is_ok());
    }

    #[test]
    fn skips_whitespace() {
        assert_eq!(
            parse(b" \n\t\r {\"a\" : [ true , null ] } \n"),
            Ok(JsonValue::Object(vec![(
                String::from("a"),
                JsonValue::Array(vec![JsonValue::Bool(true), JsonValue::Null]),
            )]))
        );
    }

    #[test]
    fn reports_trailing_content() {
        let error = parse(b"true false").unwrap_err();
        assert_eq!(error.kind, ParseErrorKind::TrailingContent);
    }

    #[test]
    fn reports_lone_surrogates() {
        assert_eq!(
            parse(br#""\uD83D""#).unwrap_err().kind,
            ParseErrorKind::InvalidUnicodeEscape
        );
        assert_eq!(
            parse(br#""\uDE00""#).unwrap_err().kind,
            ParseErrorKind::InvalidUnicodeEscape
        );
    }

    #[test]
    fn reports_leading_zero_numbers() {
        assert_eq!(
            parse(b"01").unwrap_err().kind,
            ParseErrorKind::TrailingContent
        );
        assert_eq!(
            parse(b"-01").unwrap_err().kind,
            ParseErrorKind::TrailingContent
        );
    }

    #[test]
    fn parses_all_supported_number_forms() {
        assert_eq!(parse(b"0"), Ok(JsonValue::Number(0.0)));
        assert_eq!(parse(b"-0"), Ok(JsonValue::Number(-0.0)));
        assert_eq!(parse(b"0.5"), Ok(JsonValue::Number(0.5)));
        assert_eq!(parse(b"-0.5"), Ok(JsonValue::Number(-0.5)));
        assert_eq!(parse(b"1e10"), Ok(JsonValue::Number(1e10)));
        assert_eq!(parse(b"1E+10"), Ok(JsonValue::Number(1e10)));
        assert_eq!(parse(b"1e-10"), Ok(JsonValue::Number(1e-10)));
        assert_eq!(parse(b"1.5e2"), Ok(JsonValue::Number(150.0)));
        assert_eq!(parse(b"-1.5E-3"), Ok(JsonValue::Number(-1.5e-3)));
    }

    #[test]
    fn rejects_additional_leading_zero_forms() {
        assert_eq!(
            parse(b"00").unwrap_err().kind,
            ParseErrorKind::TrailingContent
        );
        assert_eq!(
            parse(b"-00").unwrap_err().kind,
            ParseErrorKind::TrailingContent
        );
        assert_eq!(parse(b"-0"), Ok(JsonValue::Number(-0.0)));
    }

    #[test]
    fn parses_solidus_escape_explicitly() {
        assert_eq!(parse(br#""\/""#), Ok(JsonValue::String(String::from("/"))));
    }

    #[test]
    fn accepts_all_json_whitespace_around_values() {
        assert_eq!(parse(b"\t\r\n 42 \r\n\t"), Ok(JsonValue::Number(42.0)));
        assert_eq!(
            parse(b"\r\n[\t1,\n2,\r3 \t]\n"),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]))
        );
        assert_eq!(
            parse(b"\t{\n\"a\"\r:\ttrue \n}\r"),
            Ok(JsonValue::Object(vec![(
                String::from("a"),
                JsonValue::Bool(true)
            )]))
        );
    }

    #[test]
    fn parses_empty_object_array_and_empty_string_key() {
        assert_eq!(parse(b"{}"), Ok(JsonValue::Object(vec![])));
        assert_eq!(parse(b"[]"), Ok(JsonValue::Array(vec![])));
        assert_eq!(
            parse(br#"{"":1}"#),
            Ok(JsonValue::Object(vec![(
                String::from(""),
                JsonValue::Number(1.0)
            )]))
        );
    }

    #[test]
    fn parses_unicode_escape_boundary_values() {
        assert_eq!(
            parse(br#""\u0000""#),
            Ok(JsonValue::String(String::from("\u{0000}")))
        );
        assert_eq!(
            parse(br#""\uFFFF""#),
            Ok(JsonValue::String(String::from("\u{FFFF}")))
        );
    }

    #[test]
    fn rejects_utf8_bom_prefix() {
        assert_eq!(
            parse(b"\xEF\xBB\xBFnull").unwrap_err().kind,
            ParseErrorKind::UnexpectedCharacter(0xEF)
        );
    }

    #[test]
    fn rejects_invalid_single_token_and_trailing_comma_inputs() {
        for input in [b"]".as_slice(), b"}", b",", b":"] {
            assert!(matches!(
                parse(input).unwrap_err().kind,
                ParseErrorKind::UnexpectedCharacter(_)
            ));
        }

        assert_eq!(
            parse(b"[,]").unwrap_err().kind,
            ParseErrorKind::UnexpectedCharacter(b',')
        );
        assert_eq!(
            parse(b"{,}").unwrap_err().kind,
            ParseErrorKind::UnexpectedCharacter(b',')
        );
        assert_eq!(
            parse(br#"{"key"}"#).unwrap_err().kind,
            ParseErrorKind::UnexpectedCharacter(b'}')
        );
        assert_eq!(
            parse(b"[1,]").unwrap_err().kind,
            ParseErrorKind::UnexpectedCharacter(b']')
        );
    }

    #[test]
    fn reports_max_depth_exceeded() {
        let mut input = vec![b'['; 129];
        input.extend(vec![b']'; 129]);
        assert_eq!(
            parse(&input).unwrap_err().kind,
            ParseErrorKind::NestingTooDeep
        );
    }

    #[test]
    fn reports_invalid_escape_sequences() {
        assert_eq!(
            parse(br#""\x""#).unwrap_err().kind,
            ParseErrorKind::InvalidEscape
        );
    }

    #[test]
    fn reports_exact_error_positions_for_post_advance_failures() {
        let invalid_escape = parse(br#""\x""#).unwrap_err();
        assert_eq!(invalid_escape.kind, ParseErrorKind::InvalidEscape);
        assert_eq!(invalid_escape.position, 2);

        let invalid_literal = parse(b"tXue").unwrap_err();
        assert_eq!(
            invalid_literal.kind,
            ParseErrorKind::UnexpectedCharacter(b'X')
        );
        assert_eq!(invalid_literal.position, 1);

        let control = parse(b"\"hello\nworld\"").unwrap_err();
        assert_eq!(control.kind, ParseErrorKind::UnexpectedCharacter(b'\n'));
        assert_eq!(control.position, 6);

        let invalid_unicode = parse(br#""\u12G4""#).unwrap_err();
        assert_eq!(invalid_unicode.kind, ParseErrorKind::InvalidUnicodeEscape);
        assert_eq!(invalid_unicode.position, 5);
    }

    #[test]
    fn reports_empty_input() {
        assert_eq!(parse(b"").unwrap_err().kind, ParseErrorKind::UnexpectedEnd);
    }

    #[test]
    fn parses_empty_array_and_object() {
        assert_eq!(parse(b"[]"), Ok(JsonValue::Array(vec![])));
        assert_eq!(parse(b"{}"), Ok(JsonValue::Object(vec![])));
    }

    #[test]
    fn parses_negative_exponent() {
        assert_eq!(parse(b"1e-2"), Ok(JsonValue::Number(0.01)));
        assert_eq!(parse(b"5E+3"), Ok(JsonValue::Number(5000.0)));
    }

    #[test]
    fn parses_multibyte_utf8_in_string() {
        assert_eq!(
            parse("\"日本語\"".as_bytes()),
            Ok(JsonValue::String(String::from("日本語")))
        );
    }

    #[test]
    fn reports_control_character_in_string() {
        assert!(parse(b"\"hello\nworld\"").is_err());
    }

    #[test]
    fn reports_invalid_utf8_sequences_in_string() {
        assert_eq!(
            parse(b"\"\x80\"").unwrap_err().kind,
            ParseErrorKind::InvalidUtf8
        );
        assert_eq!(
            parse(b"\"\xC0\xAF\"").unwrap_err().kind,
            ParseErrorKind::InvalidUtf8
        );
        assert_eq!(
            parse(b"\"\xE3\x81\"").unwrap_err().kind,
            ParseErrorKind::InvalidUtf8
        );
    }
}
