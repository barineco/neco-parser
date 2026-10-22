use alloc::string::{String, ToString};

use crate::borrowed::JsonKind;
use crate::scan;
use crate::{EncodeError, ParseError, ParseErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonNumber {
    raw: String,
}

impl JsonNumber {
    pub fn new(raw: impl Into<String>) -> Result<Self, ParseError> {
        let raw = raw.into();
        let scanned = scan::document(raw.as_bytes())?;
        if scanned.kind != JsonKind::Number || scanned.start != 0 || scanned.end != raw.len() {
            return Err(ParseError {
                kind: ParseErrorKind::InvalidNumber,
                position: scanned.start,
            });
        }
        Ok(Self { raw })
    }

    pub fn from_i64(value: i64) -> Self {
        Self {
            raw: value.to_string(),
        }
    }

    pub fn from_u64(value: u64) -> Self {
        Self {
            raw: value.to_string(),
        }
    }

    pub fn from_f64(value: f64) -> Result<Self, EncodeError> {
        if !value.is_finite() {
            return Err(EncodeError::NonFiniteNumber);
        }
        Ok(Self {
            raw: value.to_string(),
        })
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.raw
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.raw.parse().ok()
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.raw.parse().ok()
    }

    pub(crate) fn from_validated(raw: &str) -> Self {
        Self { raw: raw.into() }
    }
}

impl From<i64> for JsonNumber {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

impl From<u64> for JsonNumber {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

#[cfg(test)]
mod tests {
    use super::JsonNumber;
    use crate::EncodeError;

    #[test]
    fn raw_and_interpreted_values_are_independent() {
        let exact = JsonNumber::new("9007199254740993").unwrap();
        assert_eq!(exact.raw(), "9007199254740993");
        assert_eq!(exact.as_u64(), Some(9_007_199_254_740_993));
        assert_eq!(exact.as_f64(), Some(9_007_199_254_740_992.0));

        let large = JsonNumber::new("1e400").unwrap();
        assert_eq!(large.raw(), "1e400");
        assert_eq!(large.as_f64(), None);
    }

    #[test]
    fn integer_accessors_follow_the_preserved_spelling() {
        let integer = JsonNumber::new("7").unwrap();
        let decimal = JsonNumber::new("7.0").unwrap();
        let exponent = JsonNumber::new("7e0").unwrap();
        assert_eq!(integer.as_i64(), Some(7));
        assert_eq!(decimal.as_i64(), None);
        assert_eq!(exponent.as_i64(), None);
        assert_ne!(integer, decimal);
        assert_eq!(decimal.as_f64(), Some(7.0));
    }

    #[test]
    fn constructors_preserve_supported_numeric_domains() {
        assert_eq!(JsonNumber::from_i64(i64::MIN).as_i64(), Some(i64::MIN));
        assert_eq!(JsonNumber::from_i64(i64::MAX).as_i64(), Some(i64::MAX));
        assert_eq!(JsonNumber::from_u64(u64::MAX).as_u64(), Some(u64::MAX));
        let negative_zero = JsonNumber::from_f64(-0.0).unwrap();
        assert_eq!(negative_zero.raw(), "-0");
        assert_eq!(negative_zero.as_i64(), Some(0));
        assert_eq!(negative_zero.as_u64(), None);
        assert_eq!(
            JsonNumber::from_f64(f64::INFINITY),
            Err(EncodeError::NonFiniteNumber)
        );
    }

    #[test]
    fn constructor_rejects_non_number_json() {
        for invalid in [
            "",
            "null",
            "01",
            "+1",
            ".5",
            "1.",
            "1e",
            " 1",
            "1 ",
            "1 trailing",
        ] {
            assert!(JsonNumber::new(invalid).is_err(), "{invalid:?}");
        }
    }
}
