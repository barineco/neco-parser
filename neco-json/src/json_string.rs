#![deny(
    clippy::float_arithmetic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

use core::fmt::{self, Write};

/// Formats a borrowed string directly as quoted JSON without allocating.
/// Control characters use short escapes or lowercase `\u00xx`; slashes and other Unicode remain verbatim.
/// Sink failures propagate as `core::fmt::Error`.
pub struct JsonString<'a>(&'a str);

impl<'a> JsonString<'a> {
    /// Creates a JSON string view of `value`.
    pub fn new(value: &'a str) -> Self {
        Self(value)
    }
}

impl fmt::Display for JsonString<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_char('"')?;
        for ch in self.0.chars() {
            match classify(ch) {
                Escape::Verbatim => formatter.write_char(ch)?,
                Escape::Short(escaped) => formatter.write_str(escaped)?,
                Escape::Control(code) => write!(formatter, "\\u{code:04x}")?,
            }
        }
        formatter.write_char('"')
    }
}

pub(crate) enum Escape {
    Verbatim,
    Short(&'static str),
    Control(u8),
}

pub(crate) fn classify(ch: char) -> Escape {
    match ch {
        '"' => Escape::Short("\\\""),
        '\\' => Escape::Short("\\\\"),
        '\u{0008}' => Escape::Short("\\b"),
        '\u{000C}' => Escape::Short("\\f"),
        '\n' => Escape::Short("\\n"),
        '\r' => Escape::Short("\\r"),
        '\t' => Escape::Short("\\t"),
        '\u{0000}'..='\u{001F}' => Escape::Control(ch as u8),
        _ => Escape::Verbatim,
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use alloc::format;
    use core::fmt::{self, Write};

    use super::{classify, Escape, JsonString};

    #[test]
    fn control_characters_use_lowercase_four_digit_escapes() {
        let value = format!("{}", JsonString::new("\u{0000}\u{001F}"));
        assert_eq!(value.as_bytes(), b"\"\\u0000\\u001f\"");
    }

    #[test]
    fn short_escapes_match_json_spellings() {
        let value = format!("{}", JsonString::new("\"\\\u{0008}\u{000C}\n\r\t"));
        assert_eq!(value.as_bytes(), b"\"\\\"\\\\\\b\\f\\n\\r\\t\"");
    }

    #[test]
    fn slash_and_multibyte_unicode_remain_verbatim() {
        let value = format!("{}", JsonString::new("/é水"));
        assert_eq!(value.as_bytes(), "\"/é水\"".as_bytes());
        assert!(matches!(classify('/'), Escape::Verbatim));
        assert!(matches!(classify('水'), Escape::Verbatim));
    }

    struct FailingSink;

    impl Write for FailingSink {
        fn write_str(&mut self, _: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    #[test]
    fn display_propagates_sink_error() {
        let mut sink = FailingSink;
        let result = write!(&mut sink, "{}", JsonString::new("value"));
        assert!(result.is_err());
    }
}
