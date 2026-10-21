#![deny(
    clippy::float_arithmetic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

use crate::borrowed::JsonKind;
use crate::decode_char::DecodedChars;
use crate::{ParseError, ParseErrorKind};

pub(crate) const MAX_DEPTH: usize = 128;

pub(crate) struct ScannedValue {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: JsonKind,
}

pub(crate) fn whitespace(input: &[u8], mut position: usize) -> usize {
    while matches!(input.get(position), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        position += 1;
    }
    position
}

pub(crate) fn document(input: &[u8]) -> Result<ScannedValue, ParseError> {
    let mut scanner = Scanner {
        input,
        position: 0,
        depth: 0,
    };
    let value = scanner.value()?;
    scanner.position = whitespace(input, scanner.position);
    if scanner.position < input.len() {
        return Err(scanner.error(ParseErrorKind::TrailingContent));
    }
    Ok(value)
}

pub(crate) fn value_at(input: &[u8], position: usize) -> Result<ScannedValue, ParseError> {
    Scanner {
        input,
        position,
        depth: 0,
    }
    .value()
}

struct Scanner<'a> {
    input: &'a [u8],
    position: usize,
    depth: usize,
}

impl Scanner<'_> {
    fn error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError {
            kind,
            position: self.position,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    fn expect(&mut self, expected: u8) -> Result<(), ParseError> {
        let position = self.position;
        match self.peek() {
            Some(byte) => {
                self.position += 1;
                if byte == expected {
                    Ok(())
                } else {
                    Err(ParseError {
                        kind: ParseErrorKind::UnexpectedCharacter(byte),
                        position,
                    })
                }
            }
            None => Err(self.error(ParseErrorKind::UnexpectedEnd)),
        }
    }

    fn literal(&mut self, expected: &[u8]) -> Result<(), ParseError> {
        for byte in expected {
            self.expect(*byte)?;
        }
        Ok(())
    }

    fn value(&mut self) -> Result<ScannedValue, ParseError> {
        self.position = whitespace(self.input, self.position);
        let start = self.position;
        let kind = match self.peek() {
            Some(b'n') => {
                self.literal(b"null")?;
                JsonKind::Null
            }
            Some(b't') => {
                self.literal(b"true")?;
                JsonKind::Bool
            }
            Some(b'f') => {
                self.literal(b"false")?;
                JsonKind::Bool
            }
            Some(b'"') => {
                self.string()?;
                JsonKind::String
            }
            Some(b'[') => {
                self.array()?;
                JsonKind::Array
            }
            Some(b'{') => {
                self.object()?;
                JsonKind::Object
            }
            Some(b'-' | b'0'..=b'9') => {
                self.number()?;
                JsonKind::Number
            }
            Some(byte) => return Err(self.error(ParseErrorKind::UnexpectedCharacter(byte))),
            None => return Err(self.error(ParseErrorKind::UnexpectedEnd)),
        };
        Ok(ScannedValue {
            start,
            end: self.position,
            kind,
        })
    }

    fn number(&mut self) -> Result<(), ParseError> {
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        match self.peek() {
            Some(b'0') => self.position += 1,
            Some(b'1'..=b'9') => {
                self.position += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(self.error(ParseErrorKind::InvalidNumber)),
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            let start = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == start {
                return Err(self.error(ParseErrorKind::InvalidNumber));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            let start = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == start {
                return Err(self.error(ParseErrorKind::InvalidNumber));
            }
        }
        Ok(())
    }

    fn string(&mut self) -> Result<(), ParseError> {
        let mut chars = DecodedChars::new(self.input, self.position)?;
        for character in chars.by_ref() {
            character?;
        }
        self.position = chars.offset();
        Ok(())
    }

    fn enter(&mut self) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            Err(self.error(ParseErrorKind::NestingTooDeep))
        } else {
            Ok(())
        }
    }

    fn array(&mut self) -> Result<(), ParseError> {
        self.expect(b'[')?;
        self.enter()?;
        self.position = whitespace(self.input, self.position);
        if self.peek() == Some(b']') {
            self.position += 1;
            self.depth -= 1;
            return Ok(());
        }
        loop {
            self.value()?;
            self.position = whitespace(self.input, self.position);
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    self.depth -= 1;
                    return Ok(());
                }
                Some(byte) => return Err(self.error(ParseErrorKind::UnexpectedCharacter(byte))),
                None => return Err(self.error(ParseErrorKind::UnexpectedEnd)),
            }
        }
    }

    fn object(&mut self) -> Result<(), ParseError> {
        self.expect(b'{')?;
        self.enter()?;
        self.position = whitespace(self.input, self.position);
        if self.peek() == Some(b'}') {
            self.position += 1;
            self.depth -= 1;
            return Ok(());
        }
        loop {
            self.position = whitespace(self.input, self.position);
            match self.peek() {
                Some(b'"') => self.string()?,
                Some(byte) => return Err(self.error(ParseErrorKind::UnexpectedCharacter(byte))),
                None => return Err(self.error(ParseErrorKind::UnexpectedEnd)),
            }
            self.position = whitespace(self.input, self.position);
            self.expect(b':')?;
            self.value()?;
            self.position = whitespace(self.input, self.position);
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    self.depth -= 1;
                    return Ok(());
                }
                Some(byte) => return Err(self.error(ParseErrorKind::UnexpectedCharacter(byte))),
                None => return Err(self.error(ParseErrorKind::UnexpectedEnd)),
            }
        }
    }
}
