use crate::scan::{radix_for_prefix, split_radix_prefix, Scanner};
use crate::{KdlError, KdlErrorKind, KdlNumber, NumberError};

impl KdlNumber {
    /// Constructs a [`KdlNumber`] after verifying that `raw` is a valid KDL number
    /// whose interpretation matches the given integer and floating-point values.
    pub fn new(
        raw: impl Into<String>,
        as_i64: Option<i64>,
        as_f64: Option<f64>,
    ) -> Result<Self, NumberError> {
        let raw = raw.into();
        let mut scanner = Scanner::new(&raw);
        let parsed = if raw.starts_with('#') {
            parse_keyword_number(&mut scanner)
        } else {
            parse_number(&mut scanner)
        };
        let Ok(Some(parsed)) = parsed else {
            return Err(NumberError);
        };
        if !scanner.is_eof()
            || parsed.as_i64 != as_i64
            || !option_f64_matches(parsed.as_f64, as_f64)
        {
            return Err(NumberError);
        }
        Ok(parsed)
    }

    pub(crate) fn from_i64(value: i64) -> Self {
        let raw = value.to_string();
        let (as_i64, as_f64) = interpret_decimal(&raw, DecimalKind::Integer);
        Self {
            raw,
            as_i64,
            as_f64,
        }
    }
}

fn option_f64_matches(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) if left.is_nan() && right.is_nan() => true,
        (Some(left), Some(right)) => left.to_bits() == right.to_bits(),
        (None, None) => true,
        _ => false,
    }
}

pub(crate) fn parse_number(s: &mut Scanner<'_>) -> Result<Option<KdlNumber>, KdlError> {
    let ch = match s.peek() {
        Some(ch) => ch,
        None => return Ok(None),
    };

    let is_sign = ch == '+' || ch == '-';
    let is_digit = ch.is_ascii_digit();

    if !is_sign && !is_digit {
        return Ok(None);
    }

    let saved = s.save();
    let start = s.byte_offset();

    let has_sign = if is_sign {
        s.advance();
        true
    } else {
        false
    };

    match s.peek() {
        Some(ch2) if ch2.is_ascii_digit() => {}
        Some('.') => {
            s.restore(saved);
            return Ok(None);
        }
        _ if has_sign => {
            s.restore(saved);
            return Ok(None);
        }
        _ => {
            s.restore(saved);
            return Ok(None);
        }
    }

    if s.peek() == Some('0') {
        if let Some(radix) = s.peek_next().and_then(radix_for_prefix) {
            return parse_prefixed(s, start, radix);
        }
    }

    consume_integer(s);

    let has_dot = if s.peek() == Some('.') {
        match s.peek_next() {
            Some(ch) if ch.is_ascii_digit() => {
                s.advance();
                consume_integer(s);
                true
            }
            _ => {
                s.advance();
                return Err(s.make_error(KdlErrorKind::InvalidNumber));
            }
        }
    } else {
        false
    };

    let has_exp = if matches!(s.peek(), Some('e') | Some('E')) {
        s.advance();
        if matches!(s.peek(), Some('+') | Some('-')) {
            s.advance();
        }
        if !matches!(s.peek(), Some(ch) if ch.is_ascii_digit()) {
            return Err(s.make_error(KdlErrorKind::InvalidNumber));
        }
        consume_integer(s);

        if matches!(s.peek(), Some('e') | Some('E')) {
            return Err(s.make_error(KdlErrorKind::InvalidNumber));
        }
        true
    } else {
        false
    };

    if let Some(ch) = s.peek() {
        if crate::scan::is_identifier_char(ch) && ch != '.' {
            return Err(s.make_error(KdlErrorKind::InvalidNumber));
        }
    }

    let end = s.byte_offset();
    let raw = s.slice(start, end).to_string();

    let kind = if has_dot || has_exp {
        DecimalKind::Float
    } else {
        DecimalKind::Integer
    };
    let (as_i64, as_f64) = interpret_decimal(&raw, kind);

    Ok(Some(KdlNumber {
        raw,
        as_i64,
        as_f64,
    }))
}

pub(crate) fn parse_keyword_number(s: &mut Scanner<'_>) -> Result<Option<KdlNumber>, KdlError> {
    if s.peek() != Some('#') {
        return Ok(None);
    }

    let saved = s.save();
    let start = s.byte_offset();
    s.advance();

    match s.peek() {
        Some('i') => {
            s.advance();
            if s.peek() == Some('n') {
                s.advance();
                if s.peek() == Some('f') {
                    s.advance();
                    let end = s.byte_offset();
                    let raw = s.slice(start, end).to_string();
                    return Ok(Some(KdlNumber {
                        raw,
                        as_i64: None,
                        as_f64: Some(f64::INFINITY),
                    }));
                }
            }
            s.restore(saved);
            Ok(None)
        }
        Some('-') => {
            s.advance();
            if s.peek() == Some('i') {
                s.advance();
                if s.peek() == Some('n') {
                    s.advance();
                    if s.peek() == Some('f') {
                        s.advance();
                        let end = s.byte_offset();
                        let raw = s.slice(start, end).to_string();
                        return Ok(Some(KdlNumber {
                            raw,
                            as_i64: None,
                            as_f64: Some(f64::NEG_INFINITY),
                        }));
                    }
                }
            }
            s.restore(saved);
            Ok(None)
        }
        Some('n') => {
            s.advance();
            if s.peek() == Some('a') {
                s.advance();
                if s.peek() == Some('n') {
                    s.advance();
                    let end = s.byte_offset();
                    let raw = s.slice(start, end).to_string();
                    return Ok(Some(KdlNumber {
                        raw,
                        as_i64: None,
                        as_f64: Some(f64::NAN),
                    }));
                }
            }
            s.restore(saved);
            Ok(None)
        }
        _ => {
            s.restore(saved);
            Ok(None)
        }
    }
}

fn parse_prefixed(
    s: &mut Scanner<'_>,
    start: usize,
    radix: u32,
) -> Result<Option<KdlNumber>, KdlError> {
    s.advance();
    s.advance();

    match s.peek() {
        Some(ch) if ch.is_digit(radix) => {
            s.advance();
        }
        _ => return Err(s.make_error(KdlErrorKind::InvalidNumber)),
    }

    while let Some(ch) = s.peek() {
        if ch.is_digit(radix) || ch == '_' {
            s.advance();
        } else {
            break;
        }
    }

    if let Some(ch) = s.peek() {
        if crate::scan::is_identifier_char(ch) {
            return Err(s.make_error(KdlErrorKind::InvalidNumber));
        }
    }

    let end = s.byte_offset();
    let raw = s.slice(start, end).to_string();

    let clean: String = raw.chars().filter(|&c| c != '_').collect();
    let (as_i64, as_f64) = interpret_prefixed(&clean, radix);

    Ok(Some(KdlNumber {
        raw,
        as_i64,
        as_f64,
    }))
}

fn consume_integer(s: &mut Scanner<'_>) {
    while let Some(ch) = s.peek() {
        if ch.is_ascii_digit() || ch == '_' {
            s.advance();
        } else {
            break;
        }
    }
}

enum DecimalKind {
    Integer,
    Float,
}

fn interpret_decimal(raw: &str, kind: DecimalKind) -> (Option<i64>, Option<f64>) {
    let clean: String = raw.chars().filter(|&c| c != '_').collect();
    match kind {
        DecimalKind::Float => (None, clean.parse::<f64>().ok()),
        DecimalKind::Integer => (clean.parse::<i64>().ok(), clean.parse::<f64>().ok()),
    }
}

fn interpret_prefixed(clean: &str, radix: u32) -> (Option<i64>, Option<f64>) {
    let (neg, digits) = strip_sign_and_prefix(clean);
    let magnitude = radix_digits_to_decimal(digits, radix);
    let decimal = if neg {
        format!("-{magnitude}")
    } else {
        magnitude
    };
    interpret_decimal(&decimal, DecimalKind::Integer)
}

pub(crate) fn radix_digits_to_decimal(digits: &str, radix: u32) -> String {
    let mut result: Vec<u8> = vec![0];

    for ch in digits.chars() {
        let digit_val = ch.to_digit(radix).expect("parser-validated digit") as u8;

        let mut carry: u16 = 0;
        for d in result.iter_mut() {
            let prod = (*d as u16) * (radix as u16) + carry;
            *d = (prod % 10) as u8;
            carry = prod / 10;
        }
        while carry > 0 {
            result.push((carry % 10) as u8);
            carry /= 10;
        }

        let mut carry: u16 = digit_val as u16;
        for d in result.iter_mut() {
            let sum = (*d as u16) + carry;
            *d = (sum % 10) as u8;
            carry = sum / 10;
            if carry == 0 {
                break;
            }
        }
        while carry > 0 {
            result.push((carry % 10) as u8);
            carry /= 10;
        }
    }

    while result.len() > 1 && *result.last().unwrap() == 0 {
        result.pop();
    }
    result.iter().rev().map(|d| (b'0' + d) as char).collect()
}

fn strip_sign_and_prefix(s: &str) -> (bool, &str) {
    let (neg, rest) = if let Some(r) = s.strip_prefix('-') {
        (true, r)
    } else if let Some(r) = s.strip_prefix('+') {
        (false, r)
    } else {
        (false, s)
    };
    let digits = split_radix_prefix(rest)
        .map(|(_, digits)| digits)
        .unwrap_or(rest);
    (neg, digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(input: &str) -> KdlNumber {
        let mut s = Scanner::new(input);
        parse_number(&mut s).unwrap().unwrap()
    }

    fn kw_num(input: &str) -> KdlNumber {
        let mut s = Scanner::new(input);
        parse_keyword_number(&mut s).unwrap().unwrap()
    }

    #[test]
    fn decimal_integer() {
        let n = num("42 ");
        assert_eq!(n.raw, "42");
        assert_eq!(n.as_i64, Some(42));
    }

    #[test]
    fn decimal_with_underscore() {
        let n = num("1_000 ");
        assert_eq!(n.raw, "1_000");
        assert_eq!(n.as_i64, Some(1000));
    }

    #[test]
    fn decimal_float() {
        let n = num("1.23 ");
        assert_eq!(n.raw, "1.23");
        assert_eq!(n.as_i64, None);
        assert!((n.as_f64.unwrap() - 1.23_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn decimal_exponent() {
        let n = num("1.5e2 ");
        assert_eq!(n.as_f64, Some(150.0));
    }

    #[test]
    fn decimal_exponent_no_dot() {
        let n = num("1e10 ");
        assert_eq!(n.as_f64, Some(1e10));
    }

    #[test]
    fn hex_number() {
        let n = num("0xff ");
        assert_eq!(n.as_i64, Some(255));
    }

    #[test]
    fn octal_number() {
        let n = num("0o77 ");
        assert_eq!(n.as_i64, Some(63));
    }

    #[test]
    fn binary_number() {
        let n = num("0b1010 ");
        assert_eq!(n.as_i64, Some(10));
    }

    #[test]
    fn signed_number() {
        let n = num("-42 ");
        assert_eq!(n.as_i64, Some(-42));

        let n = num("+42 ");
        assert_eq!(n.as_i64, Some(42));
    }

    #[test]
    fn signed_hex() {
        let n = num("-0xff ");
        assert_eq!(n.as_i64, Some(-255));
    }

    #[test]
    fn keyword_inf() {
        let n = kw_num("#inf");
        assert_eq!(n.raw, "#inf");
        assert_eq!(n.as_f64, Some(f64::INFINITY));
    }

    #[test]
    fn keyword_neg_inf() {
        let n = kw_num("#-inf");
        assert_eq!(n.raw, "#-inf");
        assert_eq!(n.as_f64, Some(f64::NEG_INFINITY));
    }

    #[test]
    fn keyword_nan() {
        let n = kw_num("#nan");
        assert_eq!(n.raw, "#nan");
        assert!(n.as_f64.unwrap().is_nan());
    }

    #[test]
    fn overflow_preserves_raw() {
        let big = "99999999999999999999 ";
        let n = num(big);
        assert_eq!(n.raw, "99999999999999999999");
        assert_eq!(n.as_i64, None);

        assert!(n.as_f64.is_some());
    }

    #[test]
    fn prefixed_overflow_has_nearest_f64() {
        let n = num("0xffffffffffffffff ");
        assert_eq!(n.as_i64, None);
        assert_eq!(n.as_f64, Some(18446744073709551615_u64 as f64));
    }

    #[test]
    fn prefixed_and_decimal_integers_share_interpretation() {
        for (prefixed, decimal) in [
            ("0x7fffffffffffffff ", "9223372036854775807 "),
            ("-0x8000000000000000 ", "-9223372036854775808 "),
            ("0x1_0000_0000_0000_0000 ", "18446744073709551616 "),
            ("-0b0 ", "-0 "),
        ] {
            let (p, d) = (num(prefixed), num(decimal));
            assert_eq!(p.as_i64, d.as_i64, "{prefixed}");
            assert_eq!(
                p.as_f64.map(f64::to_bits),
                d.as_f64.map(f64::to_bits),
                "{prefixed}"
            );
        }
    }

    #[test]
    fn from_i64_matches_parsed_decimal() {
        for value in [i64::MIN, -1, 0, 9007199254740993, i64::MAX] {
            let built = KdlNumber::from_i64(value);
            let parsed = num(&format!("{value} "));
            assert_eq!(built.raw, parsed.raw);
            assert_eq!(built.as_i64, parsed.as_i64);
            assert_eq!(built.as_f64, parsed.as_f64);
        }
    }

    #[test]
    fn not_a_number() {
        let mut s = Scanner::new("foo ");
        assert!(parse_number(&mut s).unwrap().is_none());
    }

    #[test]
    fn sign_only_not_a_number() {
        let mut s = Scanner::new("+ ");
        assert!(parse_number(&mut s).unwrap().is_none());
    }
}
