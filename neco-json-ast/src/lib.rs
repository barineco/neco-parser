#![doc = include_str!("../README.md")]

use neco_ast::{
    StructuredDocument as AstStructuredDocument, StructuredField, StructuredNumber, StructuredValue,
};
use neco_json::{parse as parse_value, JsonValue, ParseError};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonDocument {
    value: JsonValue,
}

#[derive(Debug, Clone, Copy)]
pub struct JsonNode<'a> {
    key: Option<&'a str>,
    value: &'a JsonValue,
}

pub fn parse(input: &[u8]) -> Result<JsonDocument, ParseError> {
    parse_value(input).map(JsonDocument::from_value)
}

impl JsonDocument {
    pub fn from_value(value: JsonValue) -> Self {
        Self { value }
    }

    pub fn as_value(&self) -> &JsonValue {
        &self.value
    }
}

impl<'a> JsonNode<'a> {
    pub fn from_value(value: &'a JsonValue) -> Self {
        Self { key: None, value }
    }

    pub fn as_value(&self) -> &'a JsonValue {
        self.value
    }
}

impl<'a> AstStructuredDocument<'a> for JsonDocument {
    type Node = JsonNode<'a>;

    fn nodes(&'a self) -> Vec<Self::Node> {
        match &self.value {
            JsonValue::Object(fields) => fields
                .iter()
                .map(|(key, value)| JsonNode {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            _ => vec![JsonNode::from_value(&self.value)],
        }
    }
}

impl<'a> neco_ast::StructuredNode<'a> for JsonNode<'a> {
    fn kind(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.key.unwrap_or("root"))
    }

    fn identifier(&self) -> Option<Cow<'a, str>> {
        self.key.map(Cow::Borrowed)
    }

    fn attribute(&self, key: &str) -> Option<StructuredValue<'a>> {
        match self.value {
            JsonValue::Object(fields) => fields
                .iter()
                .find_map(|(field, value)| (field == key).then(|| value_to_structured(value))),
            _ => None,
        }
    }

    fn children(&self) -> Vec<Self> {
        match self.value {
            JsonValue::Object(fields) => fields
                .iter()
                .map(|(key, value)| JsonNode {
                    key: Some(key.as_str()),
                    value,
                })
                .collect(),
            JsonValue::Array(values) => values
                .iter()
                .map(|value| JsonNode {
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

fn value_to_structured<'a>(value: &'a JsonValue) -> StructuredValue<'a> {
    match value {
        JsonValue::Null => StructuredValue::Null,
        JsonValue::Bool(value) => StructuredValue::Bool(*value),
        JsonValue::Number(value) => StructuredValue::Number(StructuredNumber::from_f64(*value)),
        JsonValue::String(value) => StructuredValue::String(Cow::Borrowed(value.as_str())),
        JsonValue::Array(values) => {
            StructuredValue::Sequence(values.iter().map(value_to_structured).collect())
        }
        JsonValue::Object(fields) => StructuredValue::Mapping(
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
    use super::{parse, value_to_structured, JsonDocument, JsonNode};
    use neco_ast::{StructuredDocument, StructuredNode, StructuredValue};
    use neco_json::JsonValue;

    const SAMPLE: &[u8] = br#"{"name":"neco","enabled":true,"items":["one","two"]}"#;

    #[test]
    fn parse_document() {
        assert!(parse(SAMPLE).is_ok());
    }

    #[test]
    fn document_nodes_read_object_fields() {
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
    fn object_attribute_reads_string() {
        let value = JsonValue::Object(vec![("name".into(), JsonValue::String("neco".into()))]);
        let node = JsonNode::from_value(&value);
        assert_eq!(node.attribute_str("name").as_deref(), Some("neco"));
    }

    #[test]
    fn object_attribute_reads_bool() {
        let value = JsonValue::Object(vec![("enabled".into(), JsonValue::Bool(true))]);
        let node = JsonNode::from_value(&value);
        assert_eq!(node.attribute_bool("enabled"), Some(true));
    }

    #[test]
    fn missing_attribute_is_none() {
        let value = JsonValue::Object(Vec::new());
        let node = JsonNode::from_value(&value);
        assert!(node.attribute("missing").is_none());
    }

    #[test]
    fn array_children_are_items() {
        let value = JsonValue::Array(vec![JsonValue::String("one".into())]);
        let node = JsonNode::from_value(&value);
        assert_eq!(node.children()[0].kind(), "item");
    }

    #[test]
    fn value_mapping_preserves_key() {
        let value = JsonValue::Object(vec![("name".into(), JsonValue::String("neco".into()))]);
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_mapping().expect("mapping")[0].key, "name");
    }

    #[test]
    fn scalar_value_preserves_string() {
        let value = JsonValue::String("neco".into());
        let structured = value_to_structured(&value);
        assert_eq!(structured.as_str(), Some("neco"));
    }

    #[test]
    fn document_from_value_keeps_root_scalar() {
        let doc = JsonDocument::from_value(JsonValue::String("neco".into()));
        assert_eq!(
            doc.nodes()[0].value(),
            StructuredValue::String("neco".into())
        );
    }
}
