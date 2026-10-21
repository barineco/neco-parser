#![deny(
    clippy::float_arithmetic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

use alloc::borrow::Cow;
use alloc::string::String;
use core::fmt;

use crate::borrowed::{JsonKind, JsonRef};
use crate::decode_char::DecodedChars;
use crate::error::ParseError;

/// Failure while decoding an escaped JSON string into owned UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringDecodeError {
    /// The decoded capacity exceeds the addressable string size.
    CapacityOverflow { input_len: usize },
    /// The allocator rejected the exact decoded UTF-8 byte count.
    AllocationFailed { required: usize },
    /// Character decoding failed at an original input byte position.
    Malformed(ParseError),
}

impl fmt::Display for StringDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityOverflow { input_len } => {
                write!(
                    f,
                    "decoded string capacity overflow for input length {input_len}"
                )
            }
            Self::AllocationFailed { required } => {
                write!(f, "unable to allocate {required} bytes for decoded string")
            }
            Self::Malformed(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for StringDecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Malformed(error) => Some(error),
            Self::CapacityOverflow { .. } | Self::AllocationFailed { .. } => None,
        }
    }
}

fn checked_decoded_len(decoded_len: usize, next_len: usize) -> Option<usize> {
    let total = decoded_len.checked_add(next_len)?;
    if total > isize::MAX as usize {
        None
    } else {
        Some(total)
    }
}

impl<'a> JsonRef<'a> {
    /// Returns the decoded string, borrowing the input when it has no escapes.
    pub fn as_str(&self) -> Result<Option<Cow<'a, str>>, StringDecodeError> {
        if self.kind != JsonKind::String {
            return Ok(None);
        }

        let raw = self.raw();
        let input_len = raw.len();
        let inner = &raw[1..input_len - 1];
        if !inner.as_bytes().contains(&b'\\') {
            return Ok(Some(Cow::Borrowed(inner)));
        }

        let mut decoded_len = 0usize;
        let mut decoder = DecodedChars::new(self.input.as_bytes(), self.start)
            .map_err(StringDecodeError::Malformed)?;
        for decoded in &mut decoder {
            let character = decoded.map_err(StringDecodeError::Malformed)?;
            decoded_len = checked_decoded_len(decoded_len, character.len_utf8())
                .ok_or(StringDecodeError::CapacityOverflow { input_len })?;
        }

        let mut output = String::new();
        output
            .try_reserve_exact(decoded_len)
            .map_err(|_| StringDecodeError::AllocationFailed {
                required: decoded_len,
            })?;
        let mut decoder = DecodedChars::new(self.input.as_bytes(), self.start)
            .map_err(StringDecodeError::Malformed)?;
        for decoded in &mut decoder {
            output.push(decoded.map_err(StringDecodeError::Malformed)?);
        }
        Ok(Some(Cow::Owned(output)))
    }

    /// Compares the decoded JSON string with `needle` without allocating.
    pub fn string_eq(&self, needle: &str) -> Result<bool, ParseError> {
        if self.kind != JsonKind::String {
            return Ok(false);
        }
        let mut decoded = DecodedChars::new(self.input.as_bytes(), self.start)?;
        for expected in needle.chars() {
            match decoded.next() {
                Some(Ok(actual)) if actual == expected => {}
                Some(Ok(_)) | None => return Ok(false),
                Some(Err(error)) => return Err(error),
            }
        }
        match decoded.next() {
            Some(Ok(_)) => Ok(false),
            Some(Err(error)) => Err(error),
            None => Ok(true),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use alloc::borrow::Cow;

    use crate::borrowed::{JsonKind, JsonRef};

    use super::{checked_decoded_len, StringDecodeError};

    fn string_ref(input: &str) -> JsonRef<'_> {
        JsonRef {
            input,
            start: 0,
            end: input.len(),
            kind: JsonKind::String,
        }
    }

    #[test]
    fn unescaped_string_borrows_its_inner_slice() {
        let input = "\"plain text\"";
        let decoded = string_ref(input).as_str().unwrap().unwrap();
        assert!(matches!(decoded, Cow::Borrowed("plain text")));
    }

    #[test]
    fn escaped_unicode_and_surrogates_decode_to_utf8() {
        let decoded = string_ref("\"\\u00e9\\ud83d\\ude00\"")
            .as_str()
            .unwrap()
            .unwrap();
        assert_eq!(decoded, "é😀");
        assert!(matches!(decoded, Cow::Owned(_)));
    }

    #[test]
    fn non_string_values_have_no_string_view() {
        let value = JsonRef {
            input: "null",
            start: 0,
            end: 4,
            kind: JsonKind::Null,
        };
        assert_eq!(value.as_str().unwrap(), None);
        assert!(!value.string_eq("null").unwrap());
    }

    #[test]
    fn escaped_string_comparison_decodes_without_allocation() {
        assert!(string_ref("\"a\\n😀\"").string_eq("a\n😀").unwrap());
        assert!(!string_ref("\"a\\n😀\"").string_eq("a\n😁").unwrap());
    }

    #[test]
    fn decoded_length_checks_addition_and_isize_limit() {
        assert_eq!(checked_decoded_len(2, 3), Some(5));
        assert_eq!(checked_decoded_len(usize::MAX, 1), None);
        assert_eq!(checked_decoded_len(isize::MAX as usize, 1), None);
    }

    #[test]
    fn malformed_escape_preserves_original_input_offset() {
        let input = "prefix \"\\q\"";
        let value = JsonRef {
            input,
            start: 7,
            end: input.len(),
            kind: JsonKind::String,
        };
        let error = crate::error::ParseError {
            kind: crate::error::ParseErrorKind::InvalidEscape,
            position: 9,
        };
        assert_eq!(
            value.as_str(),
            Err(StringDecodeError::Malformed(error.clone()))
        );
        assert_eq!(value.string_eq("q"), Err(error));
    }
}
