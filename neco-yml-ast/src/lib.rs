#![doc = include_str!("../README.md")]

use neco_ast::{
    StructuredDocument as AstStructuredDocument, StructuredField, StructuredNumber, StructuredValue,
};
use neco_yml::{parse as parse_value, ParseError, YmlValue};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct YmlDocument {
    value: YmlValue,
}

#[derive(Debug, Clone, Copy)]
pub struct YmlNode<'a> {
    key: Option<&'a str>,
    value: &'a YmlValue,
}

pub fn parse(input: &str) -> Result<YmlDocument, ParseError> {
    parse_value(input).map(YmlDocument::from_value)
}

impl YmlDocument {
    pub fn from_value(value: YmlValue) -> Self {
        Self { value }
    }

    pub fn as_value(&self) -> &YmlValue {
        &self.value
    }
}

impl<'a> YmlNode<'a> {
    pub fn from_value(value: &'a YmlValue) -> Self {
        Self { key: None, value }
    }

    pub fn as_value(&self) -> &'a YmlValue {
        self.value
    }
}

impl<'a> AstStructuredDocument<'a> for YmlDocument {
    type Node = YmlNode<'a>;

    fn nodes(&'a self) -> Vec<Self::Node> {
        match &self.value {
            YmlValue::Map(fields) => fields
                .iter()
                .map(|(key, value)| YmlNode {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            _ => vec![YmlNode::from_value(&self.value)],
        }
    }
}

impl<'a> neco_ast::StructuredNode<'a> for YmlNode<'a> {
    fn kind(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.key.unwrap_or("root"))
    }

    fn identifier(&self) -> Option<Cow<'a, str>> {
        self.key.map(Cow::Borrowed)
    }

    fn attribute(&self, key: &str) -> Option<StructuredValue<'a>> {
        match self.value {
            YmlValue::Map(fields) => fields
                .iter()
                .find_map(|(field, value)| (field == key).then(|| value_to_structured(value))),
            _ => None,
        }
    }

    fn children(&self) -> Vec<Self> {
        match self.value {
            YmlValue::Map(fields) => fields
                .iter()
                .map(|(key, value)| YmlNode {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            YmlValue::List(values) => values
                .iter()
                .map(|value| YmlNode {
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

fn value_to_structured<'a>(value: &'a YmlValue) -> StructuredValue<'a> {
    match value {
        YmlValue::Null => StructuredValue::Null,
        YmlValue::Bool(value) => StructuredValue::Bool(*value),
        YmlValue::Number(value) => StructuredValue::Number(StructuredNumber::from_f64(*value)),
        YmlValue::String(value) => StructuredValue::String(Cow::Borrowed(value.as_str())),
        YmlValue::List(values) => {
            StructuredValue::Sequence(values.iter().map(value_to_structured).collect())
        }
        YmlValue::Map(fields) => StructuredValue::Mapping(
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
    use super::{parse, value_to_structured, YmlDocument, YmlNode};
    use neco_ast::{StructuredDocument, StructuredNode, StructuredValue};
    use neco_yml::YmlValue;

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
        let value = YmlValue::Map(vec![("name".into(), YmlValue::String("neco".into()))]);
        let node = YmlNode::from_value(&value);
        assert_eq!(node.attribute_str("name").as_deref(), Some("neco"));
    }

    #[test]
    fn mapping_attribute_reads_bool() {
        let value = YmlValue::Map(vec![("enabled".into(), YmlValue::Bool(true))]);
        let node = YmlNode::from_value(&value);
        assert_eq!(node.attribute_bool("enabled"), Some(true));
    }

    #[test]
    fn missing_attribute_is_none() {
        let value = YmlValue::Map(Vec::new());
        let node = YmlNode::from_value(&value);
        assert!(node.attribute("missing").is_none());
    }

    #[test]
    fn list_children_are_items() {
        let value = YmlValue::List(vec![YmlValue::String("one".into())]);
        let node = YmlNode::from_value(&value);
        assert_eq!(node.children()[0].kind(), "item");
    }

    #[test]
    fn value_mapping_preserves_key() {
        let value = YmlValue::Map(vec![("name".into(), YmlValue::String("neco".into()))]);
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_mapping().expect("mapping")[0].key, "name");
    }

    #[test]
    fn scalar_value_preserves_string() {
        let value = YmlValue::String("neco".into());
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_str(), Some("neco"));
    }

    #[test]
    fn document_from_value_keeps_root_scalar() {
        let doc = YmlDocument::from_value(YmlValue::String("neco".into()));
        assert_eq!(
            doc.nodes()[0].value(),
            StructuredValue::String("neco".into())
        );
    }
}
