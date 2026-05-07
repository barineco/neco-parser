#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

use alloc::{borrow::Cow, format, string::String, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
pub enum StructuredValue<'a> {
    Null,
    Bool(bool),
    Number(StructuredNumber<'a>),
    String(Cow<'a, str>),
    Sequence(Vec<StructuredValue<'a>>),
    Mapping(Vec<StructuredField<'a>>),
}

impl<'a> StructuredValue<'a> {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value.as_ref()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => value.as_f64,
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&[StructuredValue<'a>]> {
        match self {
            Self::Sequence(values) => Some(values.as_slice()),
            _ => None,
        }
    }

    pub fn as_mapping(&self) -> Option<&[StructuredField<'a>]> {
        match self {
            Self::Mapping(fields) => Some(fields.as_slice()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructuredNumber<'a> {
    pub raw: Cow<'a, str>,
    pub as_f64: Option<f64>,
}

impl<'a> StructuredNumber<'a> {
    pub fn from_parts(raw: Cow<'a, str>, as_f64: Option<f64>) -> Self {
        Self { raw, as_f64 }
    }

    pub fn from_f64(value: f64) -> StructuredNumber<'static> {
        StructuredNumber {
            raw: Cow::Owned(format!("{value}")),
            as_f64: Some(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructuredField<'a> {
    pub key: Cow<'a, str>,
    pub value: StructuredValue<'a>,
}

pub trait StructuredNode<'a>: Sized {
    fn kind(&self) -> Cow<'a, str>;

    fn identifier(&self) -> Option<Cow<'a, str>> {
        None
    }

    fn attribute(&self, key: &str) -> Option<StructuredValue<'a>>;

    fn attribute_str(&self, key: &str) -> Option<Cow<'a, str>> {
        self.attribute(key)
            .and_then(|value| value.as_str().map(|value| Cow::Owned(String::from(value))))
    }

    fn attribute_bool(&self, key: &str) -> Option<bool> {
        self.attribute(key).and_then(|value| value.as_bool())
    }

    fn type_annotation(&self) -> Option<Cow<'a, str>> {
        None
    }

    fn children(&self) -> Vec<Self>;

    fn value(&self) -> StructuredValue<'a>;
}

pub trait StructuredDocument<'a> {
    type Node: StructuredNode<'a>;

    fn nodes(&'a self) -> Vec<Self::Node>;
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec};

    use super::{StructuredField, StructuredNumber, StructuredValue};

    #[test]
    fn string_value_exposes_str() {
        let value = StructuredValue::String(Cow::Borrowed("neco"));
        assert_eq!(value.as_str(), Some("neco"));
    }

    #[test]
    fn bool_value_exposes_bool() {
        let value = StructuredValue::Bool(true);
        assert_eq!(value.as_bool(), Some(true));
    }

    #[test]
    fn number_value_exposes_f64() {
        let value = StructuredValue::Number(StructuredNumber::from_f64(1.5));
        assert_eq!(value.as_f64(), Some(1.5));
    }

    #[test]
    fn sequence_value_exposes_slice() {
        let value = StructuredValue::Sequence(vec![StructuredValue::Null]);
        assert_eq!(value.as_sequence().expect("sequence").len(), 1);
    }

    #[test]
    fn mapping_value_exposes_fields() {
        let value = StructuredValue::Mapping(vec![StructuredField {
            key: Cow::Borrowed("name"),
            value: StructuredValue::String(Cow::Borrowed("neco")),
        }]);
        assert_eq!(value.as_mapping().expect("mapping")[0].key, "name");
    }
}
