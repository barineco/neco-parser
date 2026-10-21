pub(crate) struct Scanner<'a> {
    input: &'a str,
    chars: core::str::CharIndices<'a>,
    current: Option<(usize, char)>,
}

impl<'a> Scanner<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        let mut chars = input.char_indices();
        let current = chars.next();
        Self {
            input,
            chars,
            current,
        }
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.current.map(|(_, ch)| ch)
    }

    pub(crate) fn peek_next(&self) -> Option<char> {
        let mut clone = self.chars.clone();
        clone.next().map(|(_, ch)| ch)
    }

    pub(crate) fn advance(&mut self) -> Option<char> {
        let (_, ch) = self.current?;
        self.current = self.chars.next();
        Some(ch)
    }

    pub(crate) fn expect(&mut self, expected: char) -> Result<(), crate::KdlError> {
        match self.peek() {
            Some(ch) if ch == expected => {
                self.advance();
                Ok(())
            }
            Some(ch) => Err(self.make_error(crate::KdlErrorKind::UnexpectedChar(ch))),
            None => Err(self.make_error(crate::KdlErrorKind::UnexpectedEof)),
        }
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.current.is_none()
    }

    pub(crate) fn make_error(&self, kind: crate::KdlErrorKind) -> crate::KdlError {
        self.make_error_at(self.byte_offset(), kind)
    }

    pub(crate) fn make_error_at(
        &self,
        offset: usize,
        kind: crate::KdlErrorKind,
    ) -> crate::KdlError {
        let (line, col) = line_col(self.input, offset).unwrap_or((1, 1));
        crate::KdlError { line, col, kind }
    }

    pub(crate) fn byte_offset(&self) -> usize {
        match self.current {
            Some((idx, _)) => idx,
            None => self.input.len(),
        }
    }

    pub(crate) fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.input[start..end]
    }

    pub(crate) fn skip_bom(&mut self) {
        if self.peek() == Some('\u{FEFF}') {
            self.current = self.chars.next();
        }
    }

    pub(crate) fn consume_newline(&mut self) -> bool {
        match self.peek() {
            Some(ch) if is_newline(ch) => {
                let was_cr = ch == '\r';
                self.advance();
                if was_cr && self.peek() == Some('\n') {
                    self.current = self.chars.next();
                }
                true
            }
            _ => false,
        }
    }

    pub(crate) fn save(&self) -> ScannerState<'a> {
        ScannerState {
            chars: self.chars.clone(),
            current: self.current,
            offset: self.byte_offset(),
        }
    }

    pub(crate) fn restore(&mut self, state: ScannerState<'a>) {
        self.chars = state.chars;
        self.current = state.current;
    }
}

#[derive(Clone)]
pub(crate) struct ScannerState<'a> {
    chars: core::str::CharIndices<'a>,
    current: Option<(usize, char)>,
    offset: usize,
}

impl ScannerState<'_> {
    pub(crate) fn byte_offset(&self) -> usize {
        self.offset
    }
}

/// byte offset を一始まりの行と列へ導出する唯一の変換。
///
/// 範囲外と文字の途中の offset には None を返す。改行の規則は解析と同一である。
pub fn line_col(input: &str, offset: usize) -> Option<(usize, usize)> {
    if !input.is_char_boundary(offset) {
        return None;
    }

    let mut line = 1;
    let mut col = 1;
    let mut index = 0;
    let bytes = input.as_bytes();

    while index < offset {
        let ch = input[index..].chars().next().unwrap();
        let len = ch.len_utf8();

        if is_newline(ch) {
            if ch == '\r' && bytes.get(index + len) == Some(&b'\n') {
                if offset == index + len {
                    return Some((line, col + 1));
                }
                line += 1;
                col = 1;
                index += len + 1;
                continue;
            }
            line += 1;
            col = 1;
            index += len;
            continue;
        }

        col += 1;
        index += len;
    }

    Some((line, col))
}

pub(crate) fn is_unicode_space(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}' | '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

pub(crate) fn is_newline(ch: char) -> bool {
    matches!(
        ch,
        '\u{000A}' | '\u{000D}' | '\u{0085}' | '\u{000B}' | '\u{000C}' | '\u{2028}' | '\u{2029}'
    )
}

pub(crate) fn is_disallowed(ch: char) -> bool {
    matches!(
        ch,
        '\u{0000}'..='\u{0008}'
            | '\u{000E}'..='\u{001F}'
            | '\u{007F}'
            | '\u{200E}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

pub(crate) fn is_identifier_char(ch: char) -> bool {
    !is_unicode_space(ch)
        && !is_newline(ch)
        && !is_disallowed(ch)
        && !matches!(
            ch,
            '\\' | '/' | '(' | ')' | '{' | '}' | ';' | '[' | ']' | '"' | '#' | '='
        )
}

pub(crate) const RESERVED_KEYWORDS: [&str; 6] = ["true", "false", "null", "inf", "-inf", "nan"];

pub(crate) fn is_reserved_keyword(value: &str) -> bool {
    RESERVED_KEYWORDS.contains(&value)
}

pub(crate) const RADIX_PREFIXES: [(char, u32); 3] = [('x', 16), ('o', 8), ('b', 2)];

pub(crate) fn radix_for_prefix(prefix: char) -> Option<u32> {
    RADIX_PREFIXES
        .iter()
        .find_map(|(candidate, radix)| (*candidate == prefix).then_some(*radix))
}

pub(crate) fn split_radix_prefix(value: &str) -> Option<(u32, &str)> {
    let bytes = value.as_bytes();
    if bytes.first() != Some(&b'0') {
        return None;
    }
    let prefix = bytes.get(1).copied()?.to_ascii_lowercase() as char;
    let radix = radix_for_prefix(prefix)?;
    Some((radix, &value[2..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_space_table() {
        assert!(is_unicode_space('\t'));
        assert!(is_unicode_space(' '));
        assert!(is_unicode_space('\u{00A0}'));
        assert!(is_unicode_space('\u{3000}'));
        assert!(is_unicode_space('\u{2009}'));
        assert!(!is_unicode_space('a'));
        assert!(!is_unicode_space('\n'));
    }

    #[test]
    fn newline_chars() {
        assert!(is_newline('\n'));
        assert!(is_newline('\r'));
        assert!(is_newline('\u{0085}'));
        assert!(is_newline('\u{000B}'));
        assert!(is_newline('\u{000C}'));
        assert!(is_newline('\u{2028}'));
        assert!(is_newline('\u{2029}'));
        assert!(!is_newline(' '));
    }

    #[test]
    fn disallowed_code_points() {
        assert!(is_disallowed('\u{0000}'));
        assert!(is_disallowed('\u{0008}'));
        assert!(is_disallowed('\u{000E}'));
        assert!(is_disallowed('\u{001F}'));
        assert!(is_disallowed('\u{007F}'));
        assert!(is_disallowed('\u{200E}'));
        assert!(is_disallowed('\u{FEFF}'));
        assert!(!is_disallowed('a'));
        assert!(!is_disallowed('\n'));
    }

    #[test]
    fn identifier_char_rules() {
        assert!(is_identifier_char('a'));
        assert!(is_identifier_char('-'));
        assert!(is_identifier_char('.'));
        assert!(is_identifier_char(','));
        assert!(is_identifier_char('<'));
        assert!(!is_identifier_char('\\'));
        assert!(!is_identifier_char('/'));
        assert!(!is_identifier_char('"'));
        assert!(!is_identifier_char('#'));
        assert!(!is_identifier_char('='));
        assert!(!is_identifier_char(' '));
        assert!(!is_identifier_char('\n'));
    }

    #[test]
    fn reserved_keywords_are_centralized() {
        for keyword in RESERVED_KEYWORDS {
            assert!(is_reserved_keyword(keyword));
        }
        assert!(!is_reserved_keyword("keyword"));
    }

    #[test]
    fn radix_prefixes_map_to_radices() {
        assert_eq!(split_radix_prefix("0xff"), Some((16, "ff")));
        assert_eq!(split_radix_prefix("0O70"), Some((8, "70")));
        assert_eq!(split_radix_prefix("0b10"), Some((2, "10")));
        assert_eq!(split_radix_prefix("10"), None);
    }

    #[test]
    fn scanner_tracks_byte_offset() {
        let mut s = Scanner::new("ab\ncd");
        assert_eq!(s.byte_offset(), 0);
        assert_eq!(s.advance(), Some('a'));
        assert_eq!(s.byte_offset(), 1);
        assert_eq!(s.advance(), Some('b'));
        assert_eq!(s.byte_offset(), 2);
        assert_eq!(s.advance(), Some('\n'));
        assert_eq!(s.byte_offset(), 3);
    }

    #[test]
    fn scanner_crlf_as_single_newline() {
        let mut s = Scanner::new("a\r\nb");
        assert_eq!(s.advance(), Some('a'));
        assert!(s.consume_newline());
        assert_eq!(s.byte_offset(), 3);
        assert_eq!(s.peek(), Some('b'));
    }

    #[test]
    fn scanner_bom_skip() {
        let mut s = Scanner::new("\u{FEFF}hello");
        s.skip_bom();
        assert_eq!(s.peek(), Some('h'));
        assert_eq!(s.byte_offset(), 3);
    }

    #[test]
    fn line_col_folds_crlf_and_rejects_interior_offsets() {
        let input = "ab\r\ncd";
        assert_eq!(line_col(input, 0), Some((1, 1)));
        assert_eq!(line_col(input, 2), Some((1, 3)));
        assert_eq!(line_col(input, 3), Some((1, 4)));
        assert_eq!(line_col(input, 4), Some((2, 1)));
        assert_eq!(line_col(input, 5), Some((2, 2)));
        assert_eq!(line_col(input, 6), Some((2, 3)));
        assert_eq!(line_col(input, 7), None);
    }

    #[test]
    fn line_col_rejects_offset_inside_multibyte_char() {
        let input = "π";
        assert_eq!(line_col(input, 0), Some((1, 1)));
        assert_eq!(line_col(input, 1), None);
        assert_eq!(line_col(input, 2), Some((1, 2)));
    }

    #[test]
    fn scanner_save_restore() {
        let mut s = Scanner::new("abc");
        s.advance();
        let saved = s.save();
        s.advance();
        s.advance();
        assert!(s.is_eof());
        s.restore(saved);
        assert_eq!(s.peek(), Some('b'));
    }
}
