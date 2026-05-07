#![doc = include_str!("../README.md")]

use neco_ast::{
    StructuredDocument as AstStructuredDocument, StructuredField, StructuredNumber, StructuredValue,
};
use neco_toml::{parse as parse_value, ParseError, TomlValue};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct TomlDocument {
    value: TomlValue,
}

#[derive(Debug, Clone, Copy)]
pub struct TomlNode<'a> {
    key: Option<&'a str>,
    value: &'a TomlValue,
}

pub fn parse(input: &str) -> Result<TomlDocument, ParseError> {
    parse_value(input).map(TomlDocument::from_value)
}

impl TomlDocument {
    pub fn from_value(value: TomlValue) -> Self {
        Self { value }
    }

    pub fn as_value(&self) -> &TomlValue {
        &self.value
    }
}

impl<'a> TomlNode<'a> {
    pub fn from_value(value: &'a TomlValue) -> Self {
        Self { key: None, value }
    }

    pub fn as_value(&self) -> &'a TomlValue {
        self.value
    }
}

impl<'a> AstStructuredDocument<'a> for TomlDocument {
    type Node = TomlNode<'a>;

    fn nodes(&'a self) -> Vec<Self::Node> {
        match &self.value {
            TomlValue::Map(fields) => fields
                .iter()
                .map(|(key, value)| TomlNode {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            _ => vec![TomlNode::from_value(&self.value)],
        }
    }
}

impl<'a> neco_ast::StructuredNode<'a> for TomlNode<'a> {
    fn kind(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.key.unwrap_or("root"))
    }

    fn identifier(&self) -> Option<Cow<'a, str>> {
        self.key.map(Cow::Borrowed)
    }

    fn attribute(&self, key: &str) -> Option<StructuredValue<'a>> {
        match self.value {
            TomlValue::Map(fields) => fields
                .iter()
                .find_map(|(field, value)| (field == key).then(|| value_to_structured(value))),
            _ => None,
        }
    }

    fn children(&self) -> Vec<Self> {
        match self.value {
            TomlValue::Map(fields) => fields
                .iter()
                .map(|(key, value)| TomlNode {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            TomlValue::List(values) => values
                .iter()
                .map(|value| TomlNode {
                    key: Some("item"),
                    value,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    fn value(&self) -> StructuredValue<'a> {
        value_to_structured(self.value)
    }
}

fn value_to_structured<'a>(value: &'a TomlValue) -> StructuredValue<'a> {
    match value {
        TomlValue::Null => StructuredValue::Null,
        TomlValue::Bool(value) => StructuredValue::Bool(*value),
        TomlValue::Number(value) => StructuredValue::Number(StructuredNumber::from_f64(*value)),
        TomlValue::String(value) => StructuredValue::String(Cow::Borrowed(value.as_str())),
        TomlValue::List(values) => {
            StructuredValue::Sequence(values.iter().map(value_to_structured).collect())
        }
        TomlValue::Map(fields) => StructuredValue::Mapping(
            fields
                .iter()
                .map(|(key, value)| StructuredField {
                    key: Cow::Borrowed(key.as_str()),
                    value: value_to_structured(value),
                })
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, value_to_structured, TomlDocument, TomlNode};
    use neco_ast::{StructuredDocument, StructuredNode, StructuredValue};
    use neco_toml::TomlValue;

    const SAMPLE: &str = "name = neco\nenabled = true\nitems = [one, two]\n";

    #[test]
    fn parse_document() {
        assert!(parse(SAMPLE).is_ok());
    }

    #[test]
    fn document_nodes_read_map_fields() {
        let doc = parse(SAMPLE).expect("parse");
        assert_eq!(doc.nodes().len(), 3);
    }

    #[test]
    fn node_kind_uses_field_key() {
        let doc = parse(SAMPLE).expect("parse");
        assert_eq!(doc.nodes()[0].kind(), "name");
    }

    #[test]
    fn node_identifier_uses_field_key() {
        let doc = parse(SAMPLE).expect("parse");
        assert_eq!(doc.nodes()[0].identifier().as_deref(), Some("name"));
    }

    #[test]
    fn mapping_attribute_reads_string() {
        let value = TomlValue::Map(vec![("name".into(), TomlValue::String("neco".into()))]);
        let node = TomlNode::from_value(&value);
        assert_eq!(node.attribute_str("name").as_deref(), Some("neco"));
    }

    #[test]
    fn mapping_attribute_reads_bool() {
        let value = TomlValue::Map(vec![("enabled".into(), TomlValue::Bool(true))]);
        let node = TomlNode::from_value(&value);
        assert_eq!(node.attribute_bool("enabled"), Some(true));
    }

    #[test]
    fn missing_attribute_is_none() {
        let value = TomlValue::Map(Vec::new());
        let node = TomlNode::from_value(&value);
        assert!(node.attribute("missing").is_none());
    }

    #[test]
    fn list_children_are_items() {
        let value = TomlValue::List(vec![TomlValue::String("one".into())]);
        let node = TomlNode::from_value(&value);
        assert_eq!(node.children()[0].kind(), "item");
    }

    #[test]
    fn value_mapping_preserves_key() {
        let value = TomlValue::Map(vec![("name".into(), TomlValue::String("neco".into()))]);
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_mapping().expect("mapping")[0].key, "name");
    }

    #[test]
    fn scalar_value_preserves_string() {
        let value = TomlValue::String("neco".into());
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_str(), Some("neco"));
    }

    #[test]
    fn document_from_value_keeps_root_scalar() {
        let doc = TomlDocument::from_value(TomlValue::String("neco".into()));
        assert_eq!(
            doc.nodes()[0].value(),
            StructuredValue::String("neco".into())
        );
    }
}
