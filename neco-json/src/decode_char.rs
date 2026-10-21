#![deny(
    clippy::float_arithmetic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

use crate::error::{ParseError, ParseErrorKind};

pub(crate) struct DecodedChars<'a> {
    input: &'a [u8],
    position: usize,
    finished: bool,
}

impl<'a> DecodedChars<'a> {
    pub(crate) fn new(input: &'a [u8], start: usize) -> Result<Self, ParseError> {
        match input.get(start).copied() {
            Some(b'"') => Ok(Self {
                input,
                position: start + 1,
                finished: false,
            }),
            Some(byte) => Err(ParseError {
                kind: ParseErrorKind::UnexpectedCharacter(byte),
                position: start,
            }),
            None => Err(ParseError {
                kind: ParseErrorKind::UnexpectedEnd,
                position: start,
            }),
        }
    }

    pub(crate) fn offset(&self) -> usize {
        self.position
    }

    fn error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError {
            kind,
            position: self.position,
        }
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.input.get(self.position).copied()?;
        self.position += 1;
        Some(byte)
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let mut value = 0_u32;
        for _ in 0..4 {
            match self.advance() {
                Some(byte @ b'0'..=b'9') => value = value * 16 + (byte - b'0') as u32,
                Some(byte @ b'a'..=b'f') => value = value * 16 + (byte - b'a' + 10) as u32,
                Some(byte @ b'A'..=b'F') => value = value * 16 + (byte - b'A' + 10) as u32,
                Some(_) => {
                    return Err(ParseError {
                        kind: ParseErrorKind::InvalidUnicodeEscape,
                        position: self.position - 1,
                    });
                }
                None => return Err(self.error(ParseErrorKind::InvalidUnicodeEscape)),
            }
        }
        Ok(value)
    }

    fn escaped(&mut self) -> Result<char, ParseError> {
        match self.advance() {
            Some(b'"') => Ok('"'),
            Some(b'\\') => Ok('\\'),
            Some(b'/') => Ok('/'),
            Some(b'b') => Ok('\u{0008}'),
            Some(b'f') => Ok('\u{000C}'),
            Some(b'n') => Ok('\n'),
            Some(b'r') => Ok('\r'),
            Some(b't') => Ok('\t'),
            Some(b'u') => {
                let high = self.hex4()?;
                if (0xD800..=0xDBFF).contains(&high) {
                    if self.advance() != Some(b'\\') || self.advance() != Some(b'u') {
                        return Err(self.error(ParseErrorKind::InvalidUnicodeEscape));
                    }
                    let low = self.hex4()?;
                    if !(0xDC00..=0xDFFF).contains(&low) {
                        return Err(self.error(ParseErrorKind::InvalidUnicodeEscape));
                    }
                    let scalar = (high - 0xD800) * 0x400 + (low - 0xDC00) + 0x10000;
                    char::from_u32(scalar)
                        .ok_or_else(|| self.error(ParseErrorKind::InvalidUnicodeEscape))
                } else if (0xDC00..=0xDFFF).contains(&high) {
                    Err(self.error(ParseErrorKind::InvalidUnicodeEscape))
                } else {
                    char::from_u32(high)
                        .ok_or_else(|| self.error(ParseErrorKind::InvalidUnicodeEscape))
                }
            }
            Some(_) => Err(ParseError {
                kind: ParseErrorKind::InvalidEscape,
                position: self.position - 1,
            }),
            None => Err(self.error(ParseErrorKind::UnexpectedEnd)),
        }
    }
}

impl Iterator for DecodedChars<'_> {
    type Item = Result<char, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let first = match self.advance() {
            Some(b'"') => {
                self.finished = true;
                return None;
            }
            Some(b'\\') => match self.escaped() {
                Ok(character) => return Some(Ok(character)),
                Err(error) => {
                    self.finished = true;
                    return Some(Err(error));
                }
            },
            Some(byte @ 0..=0x1F) => {
                self.finished = true;
                return Some(Err(ParseError {
                    kind: ParseErrorKind::UnexpectedCharacter(byte),
                    position: self.position - 1,
                }));
            }
            Some(byte) => byte,
            None => {
                self.finished = true;
                return Some(Err(self.error(ParseErrorKind::UnexpectedEnd)));
            }
        };
        let start = self.position - 1;
        let width = if first < 0x80 {
            1
        } else if (0xC2..=0xDF).contains(&first) {
            2
        } else if (0xE0..=0xEF).contains(&first) {
            3
        } else if (0xF0..=0xF4).contains(&first) {
            4
        } else {
            self.finished = true;
            return Some(Err(self.error(ParseErrorKind::InvalidUtf8)));
        };
        let mut bytes = [0_u8; 4];
        bytes[0] = first;
        for slot in bytes.iter_mut().take(width).skip(1) {
            match self.advance() {
                Some(byte) => *slot = byte,
                None => {
                    self.finished = true;
                    return Some(Err(self.error(ParseErrorKind::InvalidUtf8)));
                }
            }
        }
        let sequence = &bytes[..width];
        let valid = match sequence {
            [_, second] => (0x80..=0xBF).contains(second),
            [lead, second, third] => {
                let second_valid = match *lead {
                    0xE0 => (0xA0..=0xBF).contains(second),
                    0xE1..=0xEC | 0xEE..=0xEF => (0x80..=0xBF).contains(second),
                    0xED => (0x80..=0x9F).contains(second),
                    _ => false,
                };
                second_valid && (0x80..=0xBF).contains(third)
            }
            [lead, second, third, fourth] => {
                let second_valid = match *lead {
                    0xF0 => (0x90..=0xBF).contains(second),
                    0xF1..=0xF3 => (0x80..=0xBF).contains(second),
                    0xF4 => (0x80..=0x8F).contains(second),
                    _ => false,
                };
                second_valid && (0x80..=0xBF).contains(third) && (0x80..=0xBF).contains(fourth)
            }
            _ => true,
        };
        if !valid {
            self.finished = true;
            return Some(Err(self.error(ParseErrorKind::InvalidUtf8)));
        }
        let character = match core::str::from_utf8(&self.input[start..self.position]) {
            Ok(text) => match text.chars().next() {
                Some(character) => character,
                None => {
                    self.finished = true;
                    return Some(Err(self.error(ParseErrorKind::InvalidUtf8)));
                }
            },
            Err(_) => {
                self.finished = true;
                return Some(Err(self.error(ParseErrorKind::InvalidUtf8)));
            }
        };
        Some(Ok(character))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::DecodedChars;
    use crate::error::ParseErrorKind;

    #[test]
    fn decodes_short_unicode_and_surrogate_escapes() {
        let mut chars = DecodedChars::new(br#""\n\u03A9\uD83D\uDE00""#, 0).unwrap();
        assert_eq!(chars.next(), Some(Ok('\n')));
        assert_eq!(chars.next(), Some(Ok('Ω')));
        assert_eq!(chars.next(), Some(Ok('😀')));
        assert_eq!(chars.next(), None);
        assert_eq!(chars.offset(), 22);
    }

    #[test]
    fn decodes_multibyte_utf8() {
        let mut chars = DecodedChars::new("\"é\"".as_bytes(), 0).unwrap();
        assert_eq!(chars.next(), Some(Ok('é')));
        assert_eq!(chars.next(), None);
        assert_eq!(chars.offset(), 4);
    }

    #[test]
    fn invalid_utf8_error_is_yielded_once() {
        let mut chars = DecodedChars::new(b"\"\xE0\x80\x80\"", 0).unwrap();
        assert_eq!(
            chars.next().map(Result::err),
            Some(Some(crate::error::ParseError {
                kind: ParseErrorKind::InvalidUtf8,
                position: 4,
            }))
        );
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn invalid_escape_reports_escape_byte_and_stops() {
        let mut chars = DecodedChars::new(br#""\q""#, 0).unwrap();
        assert_eq!(
            chars.next().map(Result::err),
            Some(Some(crate::error::ParseError {
                kind: ParseErrorKind::InvalidEscape,
                position: 2,
            }))
        );
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn closing_quote_finishes_iterator_and_preserves_offset() {
        let mut chars = DecodedChars::new(b"\"x\"tail", 0).unwrap();
        assert_eq!(chars.next(), Some(Ok('x')));
        assert_eq!(chars.next(), None);
        assert_eq!(chars.offset(), 3);
        assert_eq!(chars.next(), None);
        assert_eq!(chars.offset(), 3);
    }
}
