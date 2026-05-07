#![doc = include_str!("../README.md")]

use neco_ast::{
    StructuredDocument as AstStructuredDocument, StructuredField, StructuredNumber, StructuredValue,
};
use neco_json5::{parse as parse_value, Json5Value, ParseError};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct Json5Document {
    value: Json5Value,
}

#[derive(Debug, Clone, Copy)]
pub struct Json5Node<'a> {
    key: Option<&'a str>,
    value: &'a Json5Value,
}

pub fn parse(input: &str) -> Result<Json5Document, ParseError> {
    parse_value(input).map(Json5Document::from_value)
}

impl Json5Document {
    pub fn from_value(value: Json5Value) -> Self {
        Self { value }
    }

    pub fn as_value(&self) -> &Json5Value {
        &self.value
    }
}

impl<'a> Json5Node<'a> {
    pub fn from_value(value: &'a Json5Value) -> Self {
        Self { key: None, value }
    }

    pub fn as_value(&self) -> &'a Json5Value {
        self.value
    }
}

impl<'a> AstStructuredDocument<'a> for Json5Document {
    type Node = Json5Node<'a>;

    fn nodes(&'a self) -> Vec<Self::Node> {
        match &self.value {
            Json5Value::Map(fields) => fields
                .iter()
                .map(|(key, value)| Json5Node {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            _ => vec![Json5Node::from_value(&self.value)],
        }
    }
}

impl<'a> neco_ast::StructuredNode<'a> for Json5Node<'a> {
    fn kind(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.key.unwrap_or("root"))
    }

    fn identifier(&self) -> Option<Cow<'a, str>> {
        self.key.map(Cow::Borrowed)
    }

    fn attribute(&self, key: &str) -> Option<StructuredValue<'a>> {
        match self.value {
            Json5Value::Map(fields) => fields
                .iter()
                .find_map(|(field, value)| (field == key).then(|| value_to_structured(value))),
            _ => None,
        }
    }

    fn children(&self) -> Vec<Self> {
        match self.value {
            Json5Value::Map(fields) => fields
                .iter()
                .map(|(key, value)| Json5Node {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            Json5Value::List(values) => values
                .iter()
                .map(|value| Json5Node {
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

fn value_to_structured<'a>(value: &'a Json5Value) -> StructuredValue<'a> {
    match value {
        Json5Value::Null => StructuredValue::Null,
        Json5Value::Bool(value) => StructuredValue::Bool(*value),
        Json5Value::Number(value) => StructuredValue::Number(StructuredNumber::from_f64(*value)),
        Json5Value::String(value) => StructuredValue::String(Cow::Borrowed(value.as_str())),
        Json5Value::List(values) => {
            StructuredValue::Sequence(values.iter().map(value_to_structured).collect())
        }
        Json5Value::Map(fields) => StructuredValue::Mapping(
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
    use super::{parse, value_to_structured, Json5Document, Json5Node};
    use neco_ast::{StructuredDocument, StructuredNode, StructuredValue};
    use neco_json5::Json5Value;

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
        let value = Json5Value::Map(vec![("name".into(), Json5Value::String("neco".into()))]);
        let node = Json5Node::from_value(&value);
        assert_eq!(node.attribute_str("name").as_deref(), Some("neco"));
    }

    #[test]
    fn mapping_attribute_reads_bool() {
        let value = Json5Value::Map(vec![("enabled".into(), Json5Value::Bool(true))]);
        let node = Json5Node::from_value(&value);
        assert_eq!(node.attribute_bool("enabled"), Some(true));
    }

    #[test]
    fn missing_attribute_is_none() {
        let value = Json5Value::Map(Vec::new());
        let node = Json5Node::from_value(&value);
        assert!(node.attribute("missing").is_none());
    }

    #[test]
    fn list_children_are_items() {
        let value = Json5Value::List(vec![Json5Value::String("one".into())]);
        let node = Json5Node::from_value(&value);
        assert_eq!(node.children()[0].kind(), "item");
    }

    #[test]
    fn value_mapping_preserves_key() {
        let value = Json5Value::Map(vec![("name".into(), Json5Value::String("neco".into()))]);
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_mapping().expect("mapping")[0].key, "name");
    }

    #[test]
    fn scalar_value_preserves_string() {
        let value = Json5Value::String("neco".into());
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_str(), Some("neco"));
    }

    #[test]
    fn document_from_value_keeps_root_scalar() {
        let doc = Json5Document::from_value(Json5Value::String("neco".into()));
        assert_eq!(
            doc.nodes()[0].value(),
            StructuredValue::String("neco".into())
        );
    }
}
