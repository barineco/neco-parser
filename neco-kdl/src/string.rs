use crate::scan::{is_disallowed, is_identifier_char, is_newline, is_unicode_space, Scanner};
use crate::{KdlError, KdlErrorKind};

pub(crate) fn parse_string(s: &mut Scanner<'_>) -> Result<String, KdlError> {
    if let Some(q) = parse_quoted_string(s)? {
        return Ok(q);
    }
    if let Some(r) = parse_raw_string(s)? {
        return Ok(r);
    }
    if let Some(id) = parse_identifier_string(s)? {
        return Ok(id);
    }
    match s.peek() {
        Some(ch) => Err(s.make_error(KdlErrorKind::UnexpectedChar(ch))),
        None => Err(s.make_error(KdlErrorKind::UnexpectedEof)),
    }
}

pub(crate) fn parse_identifier_string(s: &mut Scanner<'_>) -> Result<Option<String>, KdlError> {
    let ch = match s.peek() {
        Some(ch) => ch,
        None => return Ok(None),
    };

    if !is_identifier_char(ch) {
        return Ok(None);
    }

    let is_sign = ch == '+' || ch == '-';
    let is_dot = ch == '.';
    let is_digit = ch.is_ascii_digit();

    if !is_sign && !is_dot && !is_digit {
        return collect_identifier(s);
    }

    if is_digit {
        return Ok(None);
    }

    let saved = s.save();

    if is_sign {
        s.advance();
        match s.peek() {
            Some(ch2) if is_identifier_char(ch2) && !ch2.is_ascii_digit() && ch2 != '.' => {
                return collect_identifier_from(s, ch);
            }

            Some('.') => {
                s.advance();
                match s.peek() {
                    Some(ch3) if is_identifier_char(ch3) && !ch3.is_ascii_digit() => {
                        let mut buf = String::new();
                        buf.push(ch);
                        buf.push('.');
                        return collect_identifier_continue(s, buf);
                    }

                    _ => {
                        let mut buf = String::new();
                        buf.push(ch);
                        buf.push('.');
                        return check_keyword(buf, &saved, s);
                    }
                }
            }

            _ => {
                let buf = String::from(ch);
                return check_keyword(buf, &saved, s);
            }
        }
    }

    if is_dot {
        s.advance();
        match s.peek() {
            Some(ch2) if is_identifier_char(ch2) && !ch2.is_ascii_digit() => {
                let mut buf = String::new();
                buf.push('.');
                return collect_identifier_continue(s, buf);
            }

            _ => {
                let buf = String::from('.');
                return check_keyword(buf, &saved, s);
            }
        }
    }

    Ok(None)
}

fn collect_identifier(s: &mut Scanner<'_>) -> Result<Option<String>, KdlError> {
    let saved = s.save();
    let mut buf = String::new();
    while let Some(ch) = s.peek() {
        if !is_identifier_char(ch) {
            break;
        }
        buf.push(ch);
        s.advance();
    }
    if buf.is_empty() {
        return Ok(None);
    }
    check_keyword(buf, &saved, s)
}

fn collect_identifier_from(s: &mut Scanner<'_>, prefix: char) -> Result<Option<String>, KdlError> {
    let mut buf = String::new();
    buf.push(prefix);
    collect_identifier_continue(s, buf)
}

fn collect_identifier_continue(
    s: &mut Scanner<'_>,
    mut buf: String,
) -> Result<Option<String>, KdlError> {
    let saved = s.save();
    while let Some(ch) = s.peek() {
        if !is_identifier_char(ch) {
            break;
        }
        buf.push(ch);
        s.advance();
    }
    check_keyword(buf, &saved, s)
}

fn check_keyword(
    buf: String,
    saved: &crate::scan::ScannerState<'_>,
    s: &mut Scanner<'_>,
) -> Result<Option<String>, KdlError> {
    if crate::scan::is_reserved_keyword(&buf) {
        Err(s.make_error_at(saved.byte_offset(), KdlErrorKind::BareKeyword))
    } else {
        Ok(Some(buf))
    }
}

pub(crate) fn parse_quoted_string(s: &mut Scanner<'_>) -> Result<Option<String>, KdlError> {
    if s.peek() != Some('"') {
        return Ok(None);
    }

    s.advance();

    if s.peek() == Some('"') {
        s.advance();
        if s.peek() == Some('"') {
            s.advance();
            return parse_multiline_quoted_body(s).map(Some);
        }

        return Ok(Some(String::new()));
    }

    parse_single_line_quoted_body(s).map(Some)
}

fn parse_single_line_quoted_body(s: &mut Scanner<'_>) -> Result<String, KdlError> {
    let mut buf = String::new();
    loop {
        match s.advance() {
            Some('"') => return Ok(buf),
            Some('\\') => {
                let escaped = parse_escape(s)?;
                if let Some(ch) = escaped {
                    buf.push(ch);
                }
            }
            Some(ch) if is_newline(ch) => {
                return Err(s.make_error(KdlErrorKind::UnclosedString));
            }
            Some(ch) if is_disallowed(ch) => {
                return Err(s.make_error(KdlErrorKind::DisallowedCodePoint(ch)));
            }
            Some(ch) => buf.push(ch),
            None => return Err(s.make_error(KdlErrorKind::UnclosedString)),
        }
    }
}

fn parse_multiline_quoted_body(s: &mut Scanner<'_>) -> Result<String, KdlError> {
    if !s.consume_newline() {
        return Err(s.make_error(KdlErrorKind::UnclosedString));
    }

    let mut raw_lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    loop {
        match s.peek() {
            Some('"') => {
                s.advance();
                if s.peek() == Some('"') {
                    s.advance();
                    if s.peek() == Some('"') {
                        s.advance();

                        raw_lines.push(current_line);
                        return process_multiline(raw_lines);
                    }

                    current_line.push('"');
                    current_line.push('"');
                    continue;
                }

                current_line.push('"');
                continue;
            }
            Some('\\') => {
                s.advance();

                match s.peek() {
                    Some(ch) if is_unicode_space(ch) || is_newline(ch) => {
                        consume_whitespace_escape(s);
                    }
                    Some(ch) => {
                        current_line.push('\\');
                        current_line.push(ch);
                        s.advance();
                    }
                    None => {
                        return Err(s.make_error(KdlErrorKind::InvalidEscape));
                    }
                }
            }
            Some(ch) if is_newline(ch) => {
                s.consume_newline();
                raw_lines.push(core::mem::take(&mut current_line));
                current_line = String::new();
            }
            Some(ch) if is_disallowed(ch) => {
                return Err(s.make_error(KdlErrorKind::DisallowedCodePoint(ch)));
            }
            Some(ch) => {
                s.advance();
                current_line.push(ch);
            }
            None => return Err(s.make_error(KdlErrorKind::UnclosedString)),
        }
    }
}

fn process_multiline(raw_lines: Vec<String>) -> Result<String, KdlError> {
    if raw_lines.is_empty() {
        return Ok(String::new());
    }

    let prefix = raw_lines.last().unwrap();

    let body_lines = &raw_lines[..raw_lines.len() - 1];

    let mut result = String::new();
    for (i, line) in body_lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }

        if line.chars().all(is_unicode_space) {
            continue;
        }

        if !line.starts_with(prefix.as_str()) {
            return Err(KdlError {
                line: 0,
                col: 0,
                kind: KdlErrorKind::InconsistentIndentation,
            });
        }

        let stripped = &line[prefix.len()..];

        let resolved = resolve_escapes(stripped)?;
        result.push_str(&resolved);
    }

    Ok(result)
}

fn resolve_escapes(input: &str) -> Result<String, KdlError> {
    let mut result = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('s') => result.push(' '),
                Some('u') => {
                    match chars.next() {
                        Some('{') => {}
                        _ => {
                            return Err(KdlError {
                                line: 0,
                                col: 0,
                                kind: KdlErrorKind::InvalidUnicodeEscape,
                            })
                        }
                    }
                    let mut hex = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) if c.is_ascii_hexdigit() && hex.len() < 6 => hex.push(c),
                            _ => {
                                return Err(KdlError {
                                    line: 0,
                                    col: 0,
                                    kind: KdlErrorKind::InvalidUnicodeEscape,
                                })
                            }
                        }
                    }
                    if hex.is_empty() {
                        return Err(KdlError {
                            line: 0,
                            col: 0,
                            kind: KdlErrorKind::InvalidUnicodeEscape,
                        });
                    }
                    let cp = u32::from_str_radix(&hex, 16).map_err(|_| KdlError {
                        line: 0,
                        col: 0,
                        kind: KdlErrorKind::InvalidUnicodeEscape,
                    })?;
                    let c = char::from_u32(cp).ok_or(KdlError {
                        line: 0,
                        col: 0,
                        kind: KdlErrorKind::InvalidUnicodeEscape,
                    })?;
                    result.push(c);
                }
                _ => {
                    return Err(KdlError {
                        line: 0,
                        col: 0,
                        kind: KdlErrorKind::InvalidEscape,
                    })
                }
            }
        } else {
            result.push(ch);
        }
    }
    Ok(result)
}

pub(crate) fn parse_raw_string(s: &mut Scanner<'_>) -> Result<Option<String>, KdlError> {
    if s.peek() != Some('#') {
        return Ok(None);
    }

    let saved = s.save();

    let mut hash_count = 0usize;
    while s.peek() == Some('#') {
        s.advance();
        hash_count += 1;
    }

    if s.peek() != Some('"') {
        s.restore(saved);
        return Ok(None);
    }
    s.advance();

    if s.peek() == Some('"') {
        let saved2 = s.save();
        s.advance();
        if s.peek() == Some('"') {
            s.advance();

            return parse_multiline_raw_body(s, hash_count).map(Some);
        }

        s.restore(saved2);
    }

    parse_single_line_raw_body(s, hash_count).map(Some)
}

fn parse_single_line_raw_body(s: &mut Scanner<'_>, hash_count: usize) -> Result<String, KdlError> {
    let mut buf = String::new();

    loop {
        match s.peek() {
            Some('"') => {
                s.advance();

                let mut matched = 0;
                while matched < hash_count && s.peek() == Some('#') {
                    s.advance();
                    matched += 1;
                }
                if matched == hash_count {
                    return Ok(buf);
                }

                buf.push('"');
                for _ in 0..matched {
                    buf.push('#');
                }
            }
            Some(ch) if is_newline(ch) => {
                return Err(s.make_error(KdlErrorKind::UnclosedString));
            }
            Some(ch) if is_disallowed(ch) => {
                return Err(s.make_error(KdlErrorKind::DisallowedCodePoint(ch)));
            }
            Some(ch) => {
                s.advance();
                buf.push(ch);
            }
            None => return Err(s.make_error(KdlErrorKind::UnclosedString)),
        }
    }
}

fn parse_multiline_raw_body(s: &mut Scanner<'_>, hash_count: usize) -> Result<String, KdlError> {
    if !s.consume_newline() {
        return Err(s.make_error(KdlErrorKind::UnclosedString));
    }

    let mut raw_lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    loop {
        match s.peek() {
            Some('"') => {
                s.advance();
                if s.peek() == Some('"') {
                    s.advance();
                    if s.peek() == Some('"') {
                        s.advance();

                        let mut matched = 0;
                        while matched < hash_count && s.peek() == Some('#') {
                            s.advance();
                            matched += 1;
                        }
                        if matched == hash_count {
                            raw_lines.push(current_line);
                            return process_multiline_raw(raw_lines);
                        }

                        current_line.push('"');
                        current_line.push('"');
                        current_line.push('"');
                        for _ in 0..matched {
                            current_line.push('#');
                        }
                        continue;
                    }
                    current_line.push('"');
                    current_line.push('"');
                    continue;
                }
                current_line.push('"');
            }
            Some(ch) if is_newline(ch) => {
                s.consume_newline();
                raw_lines.push(core::mem::take(&mut current_line));
            }
            Some(ch) if is_disallowed(ch) => {
                return Err(s.make_error(KdlErrorKind::DisallowedCodePoint(ch)));
            }
            Some(ch) => {
                s.advance();
                current_line.push(ch);
            }
            None => return Err(s.make_error(KdlErrorKind::UnclosedString)),
        }
    }
}

fn process_multiline_raw(raw_lines: Vec<String>) -> Result<String, KdlError> {
    if raw_lines.is_empty() {
        return Ok(String::new());
    }

    let prefix = raw_lines.last().unwrap();
    let body_lines = &raw_lines[..raw_lines.len() - 1];

    let mut result = String::new();
    for (i, line) in body_lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }

        if line.chars().all(is_unicode_space) {
            continue;
        }

        if !line.starts_with(prefix.as_str()) {
            return Err(KdlError {
                line: 0,
                col: 0,
                kind: KdlErrorKind::InconsistentIndentation,
            });
        }

        result.push_str(&line[prefix.len()..]);
    }

    Ok(result)
}

fn parse_escape(s: &mut Scanner<'_>) -> Result<Option<char>, KdlError> {
    match s.peek() {
        Some('"') => {
            s.advance();
            Ok(Some('"'))
        }
        Some('\\') => {
            s.advance();
            Ok(Some('\\'))
        }
        Some('b') => {
            s.advance();
            Ok(Some('\u{0008}'))
        }
        Some('f') => {
            s.advance();
            Ok(Some('\u{000C}'))
        }
        Some('n') => {
            s.advance();
            Ok(Some('\n'))
        }
        Some('r') => {
            s.advance();
            Ok(Some('\r'))
        }
        Some('t') => {
            s.advance();
            Ok(Some('\t'))
        }
        Some('s') => {
            s.advance();
            Ok(Some(' '))
        }
        Some('u') => {
            s.advance();
            parse_unicode_escape(s).map(Some)
        }

        Some(ch) if is_unicode_space(ch) || is_newline(ch) => {
            consume_whitespace_escape(s);
            Ok(None)
        }
        Some(_) => Err(s.make_error(KdlErrorKind::InvalidEscape)),
        None => Err(s.make_error(KdlErrorKind::InvalidEscape)),
    }
}

fn parse_unicode_escape(s: &mut Scanner<'_>) -> Result<char, KdlError> {
    s.expect('{')?;
    let mut hex = String::new();
    loop {
        match s.peek() {
            Some('}') => {
                s.advance();
                break;
            }
            Some(ch) if ch.is_ascii_hexdigit() => {
                if hex.len() >= 6 {
                    return Err(s.make_error(KdlErrorKind::InvalidUnicodeEscape));
                }
                hex.push(ch);
                s.advance();
            }
            _ => return Err(s.make_error(KdlErrorKind::InvalidUnicodeEscape)),
        }
    }
    if hex.is_empty() {
        return Err(s.make_error(KdlErrorKind::InvalidUnicodeEscape));
    }
    let code_point = u32::from_str_radix(&hex, 16)
        .map_err(|_| s.make_error(KdlErrorKind::InvalidUnicodeEscape))?;

    char::from_u32(code_point).ok_or_else(|| s.make_error(KdlErrorKind::InvalidUnicodeEscape))
}

fn consume_whitespace_escape(s: &mut Scanner<'_>) {
    while let Some(ch) = s.peek() {
        if is_unicode_space(ch) {
            s.advance();
        } else if is_newline(ch) {
            s.consume_newline();
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_basic() {
        let mut s = Scanner::new("foo-bar ");
        let result = parse_identifier_string(&mut s).unwrap();
        assert_eq!(result, Some("foo-bar".to_string()));
    }

    #[test]
    fn identifier_signed() {
        let mut s = Scanner::new("--this ");
        let result = parse_identifier_string(&mut s).unwrap();
        assert_eq!(result, Some("--this".to_string()));
    }

    #[test]
    fn identifier_dotted() {
        let mut s = Scanner::new(".md ");
        let result = parse_identifier_string(&mut s).unwrap();
        assert_eq!(result, Some(".md".to_string()));
    }

    #[test]
    fn identifier_sign_only() {
        let mut s = Scanner::new("+ ");
        let result = parse_identifier_string(&mut s).unwrap();
        assert_eq!(result, Some("+".to_string()));
    }

    #[test]
    fn bare_keyword_true() {
        let mut s = Scanner::new("true ");
        let err = parse_identifier_string(&mut s).unwrap_err();
        assert_eq!(err.kind, KdlErrorKind::BareKeyword);
    }

    #[test]
    fn bare_keyword_null() {
        let mut s = Scanner::new("null;");
        let err = parse_identifier_string(&mut s).unwrap_err();
        assert_eq!(err.kind, KdlErrorKind::BareKeyword);
    }

    #[test]
    fn bare_keyword_inf() {
        let mut s = Scanner::new("inf ");
        let err = parse_identifier_string(&mut s).unwrap_err();
        assert_eq!(err.kind, KdlErrorKind::BareKeyword);
    }

    #[test]
    fn bare_keyword_negative_inf() {
        let mut s = Scanner::new("-inf ");
        let err = parse_identifier_string(&mut s).unwrap_err();
        assert_eq!(err.kind, KdlErrorKind::BareKeyword);
    }

    #[test]
    fn identifier_digit_start_is_none() {
        let mut s = Scanner::new("1foo");
        let result = parse_identifier_string(&mut s).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn quoted_string_basic() {
        let mut s = Scanner::new("\"hello world\"");
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn quoted_string_escapes() {
        let mut s = Scanner::new("\"a\\nb\\tc\\\\d\\\"e\"");
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("a\nb\tc\\d\"e".to_string()));
    }

    #[test]
    fn quoted_string_unicode_escape() {
        let mut s = Scanner::new("\"\\u{1F600}\"");
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("\u{1F600}".to_string()));
    }

    #[test]
    fn quoted_string_s_escape() {
        let mut s = Scanner::new("\"hello\\sworld\"");
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn quoted_string_whitespace_escape() {
        let mut s = Scanner::new("\"hello\\   world\"");
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("helloworld".to_string()));
    }

    #[test]
    fn quoted_string_empty() {
        let mut s = Scanner::new("\"\"");
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn quoted_string_surrogate_rejected() {
        let mut s = Scanner::new("\"\\u{D800}\"");
        let err = parse_quoted_string(&mut s).unwrap_err();
        assert_eq!(err.kind, KdlErrorKind::InvalidUnicodeEscape);
    }

    #[test]
    fn parse_string_dispatches() {
        let mut s = Scanner::new("\"quoted\"");
        assert_eq!(parse_string(&mut s).unwrap(), "quoted");

        let mut s = Scanner::new("bare-id ");
        assert_eq!(parse_string(&mut s).unwrap(), "bare-id");
    }

    #[test]
    fn multiline_basic() {
        let input = "\"\"\"\n    hello\n    world\n    \"\"\"";
        let mut s = Scanner::new(input);
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("hello\nworld".to_string()));
    }

    #[test]
    fn multiline_with_indent() {
        let input = "\"\"\"\n        line1\n        line2\n    \"\"\"";
        let mut s = Scanner::new(input);
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("    line1\n    line2".to_string()));
    }

    #[test]
    fn multiline_empty() {
        let input = "\"\"\"\n    \"\"\"";
        let mut s = Scanner::new(input);
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn multiline_with_escape() {
        let input = "\"\"\"\n    hello\\nworld\n    \"\"\"";
        let mut s = Scanner::new(input);
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("hello\nworld".to_string()));
    }

    #[test]
    fn multiline_blank_line() {
        let input = "\"\"\"\n    hello\n\n    world\n    \"\"\"";
        let mut s = Scanner::new(input);
        let result = parse_quoted_string(&mut s).unwrap();
        assert_eq!(result, Some("hello\n\nworld".to_string()));
    }

    #[test]
    fn raw_string_basic() {
        let mut s = Scanner::new("#\"hello world\"#");
        let result = parse_raw_string(&mut s).unwrap();
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn raw_string_with_quotes() {
        let mut s = Scanner::new("#\"hello\"world\"#");
        let result = parse_raw_string(&mut s).unwrap();
        assert_eq!(result, Some("hello\"world".to_string()));
    }

    #[test]
    fn raw_string_double_hash() {
        let mut s = Scanner::new("##\"a\"#b\"##");
        let result = parse_raw_string(&mut s).unwrap();
        assert_eq!(result, Some("a\"#b".to_string()));
    }

    #[test]
    fn raw_string_no_escape() {
        let mut s = Scanner::new("#\"hello\\nworld\"#");
        let result = parse_raw_string(&mut s).unwrap();
        assert_eq!(result, Some("hello\\nworld".to_string()));
    }

    #[test]
    fn raw_string_empty() {
        let mut s = Scanner::new("#\"\"#");
        let result = parse_raw_string(&mut s).unwrap();
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn multiline_raw_basic() {
        let input = "#\"\"\"\n    hello\n    world\n    \"\"\"#";
        let mut s = Scanner::new(input);
        let result = parse_raw_string(&mut s).unwrap();
        assert_eq!(result, Some("hello\nworld".to_string()));
    }

    #[test]
    fn parse_string_raw_dispatch() {
        let mut s = Scanner::new("#\"raw\"#");
        assert_eq!(parse_string(&mut s).unwrap(), "raw");
    }
}
