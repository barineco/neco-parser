#![deny(
    clippy::float_arithmetic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

use core::ops::Range;

use crate::scan::{self, ScannedValue};
use crate::ParseError;

/// The syntactic kind of a validated JSON value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonKind {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

/// A validated JSON value that borrows its original input.
///
/// Child views retain byte offsets into the same original input.
#[derive(Clone, Copy)]
pub struct JsonRef<'a> {
    pub(crate) input: &'a str,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: JsonKind,
}

/// Validates one complete JSON document without allocating.
/// Containers may nest up to 128 levels; syntax errors retain their original
/// byte positions in `input`.
pub fn parse_borrowed(input: &str) -> Result<JsonRef<'_>, ParseError> {
    let value = scan::document(input.as_bytes())?;
    Ok(JsonRef::scanned(input, value))
}

impl<'a> JsonRef<'a> {
    pub(crate) fn scanned(input: &'a str, value: ScannedValue) -> Self {
        Self {
            input,
            start: value.start,
            end: value.end,
            kind: value.kind,
        }
    }

    /// Returns the syntactic kind of this value.
    pub fn kind(&self) -> JsonKind {
        self.kind
    }

    /// Returns the original value text, excluding surrounding whitespace.
    pub fn raw(&self) -> &'a str {
        &self.input[self.start..self.end]
    }

    /// Returns the half-open byte range of this value in the original input.
    pub fn span(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Returns the boolean value, or `None` for another kind.
    pub fn as_bool(&self) -> Option<bool> {
        if self.kind == JsonKind::Bool {
            Some(self.raw() == "true")
        } else {
            None
        }
    }

    /// Reads a decimal integer exactly, or returns `None`.
    /// Negative signs, fractions, exponents and values above `u64::MAX`
    /// produce `None`.
    pub fn as_u64(&self) -> Option<u64> {
        if self.kind == JsonKind::Number && self.raw().bytes().all(|byte| byte.is_ascii_digit()) {
            self.raw().parse().ok()
        } else {
            None
        }
    }

    /// Iterates array elements in input order without allocating.
    pub fn array_items(&self) -> Option<JsonArrayItems<'a>> {
        if self.kind == JsonKind::Array {
            Some(JsonArrayItems::new(*self))
        } else {
            None
        }
    }

    /// Iterates object entries in input order, preserving duplicate keys.
    pub fn object_entries(&self) -> Option<JsonObjectEntries<'a>> {
        if self.kind == JsonKind::Object {
            Some(JsonObjectEntries::new(*self))
        } else {
            None
        }
    }
}

/// An allocation-free iterator over the elements of a validated array.
pub struct JsonArrayItems<'a> {
    input: &'a str,
    position: Option<usize>,
    closing: usize,
}

impl<'a> JsonArrayItems<'a> {
    pub(crate) fn new(value: JsonRef<'a>) -> Self {
        Self {
            input: value.input,
            position: Some(value.start + 1),
            closing: value.end - 1,
        }
    }
}

impl<'a> Iterator for JsonArrayItems<'a> {
    type Item = Result<JsonRef<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = scan::whitespace(self.input.as_bytes(), self.position.take()?);
        if start >= self.closing {
            return None;
        }
        let value = match scan::value_at(self.input.as_bytes(), start) {
            Ok(value) => value,
            Err(error) => return Some(Err(error)),
        };
        let next = scan::whitespace(self.input.as_bytes(), value.end);
        if next < self.closing {
            self.position = Some(next + 1);
        }
        Some(Ok(JsonRef::scanned(self.input, value)))
    }
}

/// An allocation-free iterator over the entries of a validated object.
pub struct JsonObjectEntries<'a> {
    input: &'a str,
    position: Option<usize>,
    closing: usize,
}

impl<'a> JsonObjectEntries<'a> {
    pub(crate) fn new(value: JsonRef<'a>) -> Self {
        Self {
            input: value.input,
            position: Some(value.start + 1),
            closing: value.end - 1,
        }
    }
}

impl<'a> Iterator for JsonObjectEntries<'a> {
    type Item = Result<(JsonRef<'a>, JsonRef<'a>), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = scan::whitespace(self.input.as_bytes(), self.position.take()?);
        if start >= self.closing {
            return None;
        }
        let entry = (|| {
            let key = scan::value_at(self.input.as_bytes(), start)?;
            let colon = scan::whitespace(self.input.as_bytes(), key.end);
            let value = scan::value_at(self.input.as_bytes(), colon + 1)?;
            Ok((key, value))
        })();
        let (key, value) = match entry {
            Ok(entry) => entry,
            Err(error) => return Some(Err(error)),
        };
        let next = scan::whitespace(self.input.as_bytes(), value.end);
        if next < self.closing {
            self.position = Some(next + 1);
        }
        Some(Ok((
            JsonRef::scanned(self.input, key),
            JsonRef::scanned(self.input, value),
        )))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ParseErrorKind;

    #[test]
    fn iterator_scan_errors_are_yielded_once_then_end() {
        let mut array = JsonArrayItems::new(JsonRef {
            input: "[?]",
            start: 0,
            end: 3,
            kind: JsonKind::Array,
        });
        assert_eq!(
            array.next().unwrap().err(),
            Some(ParseError {
                kind: ParseErrorKind::UnexpectedCharacter(b'?'),
                position: 1,
            })
        );
        assert!(array.next().is_none());
        let mut object = JsonObjectEntries::new(JsonRef {
            input: "{\"a\":?}",
            start: 0,
            end: 7,
            kind: JsonKind::Object,
        });
        assert_eq!(
            object.next().unwrap().err(),
            Some(ParseError {
                kind: ParseErrorKind::UnexpectedCharacter(b'?'),
                position: 5,
            })
        );
        assert!(object.next().is_none());
    }
}
