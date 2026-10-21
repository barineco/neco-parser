#[path = "support/refusing_allocator.rs"]
mod support;

use neco_json::{parse_borrowed, JsonKind, ParseError, ParseErrorKind};

#[test]
fn grammar_rejects_invalid_values_with_original_error_positions() {
    for (input, kind, position) in [
        ("", ParseErrorKind::UnexpectedEnd, 0),
        ("01", ParseErrorKind::TrailingContent, 1),
        ("-", ParseErrorKind::InvalidNumber, 1),
        ("1.", ParseErrorKind::InvalidNumber, 2),
        ("1e+", ParseErrorKind::InvalidNumber, 3),
        ("true false", ParseErrorKind::TrailingContent, 5),
        ("[1,]", ParseErrorKind::UnexpectedCharacter(b']'), 3),
        ("{\"a\":}", ParseErrorKind::UnexpectedCharacter(b'}'), 5),
        ("\"\\q\"", ParseErrorKind::InvalidEscape, 2),
        ("\"\\uDC00\"", ParseErrorKind::InvalidUnicodeEscape, 7),
        ("\"a\n\"", ParseErrorKind::UnexpectedCharacter(b'\n'), 2),
        ("\"unterminated", ParseErrorKind::UnexpectedEnd, 13),
    ] {
        assert_eq!(
            parse_borrowed(input).err(),
            Some(ParseError { kind, position }),
            "{input:?}"
        );
    }
}

#[test]
fn nesting_depth_accepts_the_limit_and_rejects_the_next_level() {
    for depth in [1, 2, 127, 128, 129] {
        let input = "[".repeat(depth) + "0" + &"]".repeat(depth);
        let result = parse_borrowed(&input);
        if depth <= 128 {
            assert_eq!(result.unwrap().kind(), JsonKind::Array);
        } else {
            assert_eq!(
                result.err(),
                Some(ParseError {
                    kind: ParseErrorKind::NestingTooDeep,
                    position: 129
                })
            );
        }
    }
}

#[test]
fn span_and_duplicate_order_refer_to_the_complete_original_input() {
    let input = " \n{\"dup\":\"\\u00e9\",\"dup\":[true,18446744073709551615]}\t";
    let root = parse_borrowed(input).unwrap();
    assert_eq!(root.raw(), input.trim());
    assert_eq!(&input[root.span()], root.raw());
    let entries: Vec<_> = root
        .object_entries()
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries[0].0.string_eq("dup").unwrap());
    assert!(entries[1].0.string_eq("dup").unwrap());
    assert!(entries[0].0.span().start < entries[1].0.span().start);
    assert_eq!(entries[0].1.raw(), "\"\\u00e9\"");
    assert_eq!(entries[0].1.as_str().unwrap().unwrap(), "é");
    let items: Vec<_> = entries[1]
        .1
        .array_items()
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(items[0].as_bool(), Some(true));
    assert_eq!(items[1].as_u64(), Some(u64::MAX));
    for value in [
        root,
        entries[0].0,
        entries[0].1,
        entries[1].0,
        entries[1].1,
        items[0],
        items[1],
    ] {
        assert_eq!(&input[value.span()], value.raw());
    }
}

#[test]
fn as_u64_bounds_preserve_exact_decimal_integers() {
    for (input, expected) in [
        ("0", Some(0)),
        ("9007199254740993", Some(9_007_199_254_740_993)),
        ("18446744073709551615", Some(u64::MAX)),
        ("18446744073709551616", None),
        ("-0", None),
        ("-1", None),
        ("1.0", None),
        ("1e0", None),
        ("true", None),
        ("\"1\"", None),
    ] {
        assert_eq!(parse_borrowed(input).unwrap().as_u64(), expected, "{input}");
    }
}

#[test]
fn zero_allocation_parsing_and_container_iteration_preserve_values() {
    let input = " {\"a\":[0,1,2],\"a\":\"\\u0061\",\"z\":false} ";
    let (parsed, requests) = support::refusing(|| parse_borrowed(input));
    assert_eq!(requests, 0);
    let root = parsed.unwrap();
    let (entries, requests) = support::refusing(|| root.object_entries());
    assert_eq!(requests, 0);
    let entries = entries.unwrap();
    let (observed, requests) = support::refusing(|| -> Result<_, ParseError> {
        let mut fields = 0;
        let mut matching_keys = 0;
        let mut total = Some(0u64);
        for entry in entries {
            let (key, value) = entry?;
            fields += 1;
            matching_keys += usize::from(key.string_eq("a")?);
            if let Some(items) = value.array_items() {
                for item in items {
                    total = total.zip(item?.as_u64()).map(|(sum, item)| sum + item);
                }
            }
        }
        Ok((fields, matching_keys, total))
    });
    assert_eq!(requests, 0);
    assert_eq!(observed.unwrap(), (3, 2, Some(3)));
}
