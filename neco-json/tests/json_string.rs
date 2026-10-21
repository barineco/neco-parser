#[path = "support/refusing_allocator.rs"]
mod support;

use neco_json::{encode, JsonString, JsonValue};
use std::fmt::{self, Write};

struct Sink {
    bytes: [u8; 256],
    len: usize,
    limit: usize,
}

impl Sink {
    fn new(limit: usize) -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
            limit,
        }
    }
}

impl Write for Sink {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(fmt::Error)?;
        if end > self.limit || end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[test]
fn escape_byte_parity_preserves_legacy_uppercase_and_new_lowercase_profiles() {
    for text in ["", "plain", "\"\\", "\u{8}\u{c}\n\r\t", "/é😀"] {
        let formatted = format!("{}", JsonString::new(text));
        assert_eq!(
            formatted.as_bytes(),
            encode(&JsonValue::String(text.into())).unwrap()
        );
    }
    let controls: String = (0..=31).map(|code| char::from_u32(code).unwrap()).collect();
    let actual = format!("{}", JsonString::new(&controls));
    assert_eq!(actual, "\"\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\b\\t\\n\\u000b\\f\\r\\u000e\\u000f\\u0010\\u0011\\u0012\\u0013\\u0014\\u0015\\u0016\\u0017\\u0018\\u0019\\u001a\\u001b\\u001c\\u001d\\u001e\\u001f\"");
    assert_eq!(
        encode(&JsonValue::String("\u{1f}".into())).unwrap(),
        b"\"\\u001F\""
    );
}

#[test]
fn zero_allocation_formatter_writes_to_a_caller_owned_sink() {
    let mut sink = Sink::new(256);
    let (result, requests) =
        support::refusing(|| write!(&mut sink, "{}", JsonString::new("a\u{1f}/é😀")));
    assert_eq!(requests, 0);
    assert_eq!(result, Ok(()));
    assert_eq!(&sink.bytes[..sink.len], "\"a\\u001f/é😀\"".as_bytes());
}

#[test]
fn propagates_write_error_after_an_observed_partial_prefix() {
    let mut sink = Sink::new(3);
    let result = write!(&mut sink, "{}", JsonString::new("abcd"));
    assert_eq!(result, Err(fmt::Error));
    assert_eq!(&sink.bytes[..sink.len], b"\"ab");
}
