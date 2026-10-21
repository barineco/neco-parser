#[path = "support/refusing_allocator.rs"]
mod support;

use neco_json::{parse_borrowed, StringDecodeError};
use std::borrow::Cow;

#[test]
fn zero_allocation_unescaped_borrow_and_escaped_comparison() {
    let plain = parse_borrowed("\"/é😀\"").unwrap();
    let escaped = parse_borrowed("\"\\u00e9\\ud83d\\ude00\"").unwrap();
    let (result, requests) = support::refusing(|| plain.as_str());
    assert_eq!(requests, 0);
    assert!(matches!(result.unwrap().unwrap(), Cow::Borrowed("/é😀")));
    let (result, requests) = support::refusing(|| escaped.string_eq("é😀"));
    assert_eq!(requests, 0);
    assert_eq!(result, Ok(true));
    let (result, requests) = support::refusing(|| escaped.string_eq("é😁"));
    assert_eq!(requests, 0);
    assert_eq!(result, Ok(false));
}

#[test]
fn allocation_failed_required_is_the_exact_decoded_utf8_byte_count() {
    for (input, expected, required) in [
        ("\"\\n\"", "\n", 1),
        ("\"\\u00e9\\ud83d\\ude00\"", "é😀", 6),
        ("\"prefix\\n水\"", "prefix\n水", 10),
    ] {
        let value = parse_borrowed(input).unwrap();
        let (result, requests) = support::refusing(|| value.as_str());
        assert_eq!(requests, 1);
        assert_eq!(
            result,
            Err(StringDecodeError::AllocationFailed { required })
        );
        let decoded = value.as_str().unwrap().unwrap();
        assert!(matches!(decoded, Cow::Owned(_)));
        assert_eq!(decoded, expected);
        assert_eq!(decoded.len(), required);
    }
}
