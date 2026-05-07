use crate::{CrossRef, CrossRefParseError, NsidPath};
use neco_ast::{StructuredField, StructuredNumber, StructuredValue};
use neco_kdl::{KdlEntry, KdlNode, KdlValue};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredName {
    pub kind: Option<NsidPath>,
    pub identifier: NsidPath,
}

#[derive(Debug, Clone, Copy)]
pub struct StructuredNode<'a> {
    node: &'a KdlNode,
}

impl<'a> StructuredNode<'a> {
    pub fn from_kdl(node: &'a KdlNode) -> Self {
        Self { node }
    }

    pub fn kind(&self) -> NsidPath {
        self.node_name_as_nsid()
    }

    pub fn identifier(&self) -> Option<NsidPath> {
        self.first_arg_as_nsid()
    }

    pub fn node_name_as_nsid(&self) -> NsidPath {
        NsidPath::parse(self.node.name())
    }

    pub fn first_arg_as_nsid(&self) -> Option<NsidPath> {
        self.node.first_string_arg().map(NsidPath::parse)
    }

    pub fn type_annotation(&self) -> Option<&'a str> {
        self.node.ty()
    }

    pub fn structured_name_form_x(&self) -> StructuredName {
        StructuredName {
            kind: Some(self.node_name_as_nsid()),
            identifier: self.first_arg_as_nsid().unwrap_or_default(),
        }
    }

    pub fn structured_name_form_y(&self) -> StructuredName {
        StructuredName {
            kind: None,
            identifier: self.node_name_as_nsid(),
        }
    }

    pub fn structured_name(&self) -> StructuredName {
        if self.first_arg_as_nsid().is_some() {
            self.structured_name_form_x()
        } else {
            self.structured_name_form_y()
        }
    }

    pub fn children(&self) -> impl Iterator<Item = StructuredNode<'a>> + 'a {
        self.node
            .children()
            .into_iter()
            .flat_map(|children| children.iter())
            .map(StructuredNode::from_kdl)
    }

    pub fn find_child_kind<'k>(
        &self,
        kind: &'k NsidPath,
    ) -> impl Iterator<Item = StructuredNode<'a>> + 'k
    where
        'a: 'k,
    {
        self.children()
            .filter(move |child| child.node_name_as_nsid() == *kind)
    }

    pub fn cross_ref_arg(&self) -> Option<Result<CrossRef, CrossRefParseError>> {
        self.node.first_string_arg().map(CrossRef::parse)
    }

    pub fn cross_ref_string_args(&self) -> Vec<Result<CrossRef, CrossRefParseError>> {
        self.node
            .entries()
            .iter()
            .filter_map(|entry| match entry {
                KdlEntry::Argument {
                    value: KdlValue::String(value),
                    ..
                } => Some(CrossRef::parse(value)),
                _ => None,
            })
            .collect()
    }

    pub fn cross_ref_prop(&self, key: &str) -> Option<Result<CrossRef, CrossRefParseError>> {
        self.node.entries().iter().find_map(|entry| match entry {
            KdlEntry::Property {
                key: property_key,
                value: KdlValue::String(value),
                ..
            } if property_key == key => Some(CrossRef::parse(value)),
            _ => None,
        })
    }

    pub fn attribute_str(&self, key: &str) -> Option<&'a str> {
        self.node.string_prop(key).or_else(|| {
            self.node
                .find_child(key)
                .and_then(KdlNode::first_string_arg)
        })
    }

    pub fn attribute_bool(&self, key: &str) -> Option<bool> {
        self.node.bool_prop(key).or_else(|| {
            self.node.find_child(key).and_then(|child| {
                child.first_arg().and_then(|value| match value {
                    KdlValue::Bool(value) => Some(*value),
                    _ => None,
                })
            })
        })
    }

    pub fn dot_chain_depth(&self) -> usize {
        self.node.name().chars().take_while(|c| *c == '.').count()
    }

    pub fn dot_chain_kind(&self) -> &'a str {
        self.node.name().trim_start_matches('.')
    }

    pub fn as_kdl(&self) -> &'a KdlNode {
        self.node
    }
}

pub trait StructuredFacade<'a>: Sized {
    fn kind(&self) -> NsidPath;
    fn identifier(&self) -> Option<NsidPath>;
    fn attribute_str(&self, key: &str) -> Option<&'a str>;
    fn type_annotation(&self) -> Option<&'a str>;
    fn children(&self) -> Vec<Self>;
}

impl<'a> StructuredFacade<'a> for StructuredNode<'a> {
    fn kind(&self) -> NsidPath {
        StructuredNode::kind(self)
    }

    fn identifier(&self) -> Option<NsidPath> {
        StructuredNode::identifier(self)
    }

    fn attribute_str(&self, key: &str) -> Option<&'a str> {
        StructuredNode::attribute_str(self, key)
    }

    fn type_annotation(&self) -> Option<&'a str> {
        StructuredNode::type_annotation(self)
    }

    fn children(&self) -> Vec<Self> {
        StructuredNode::children(self).collect()
    }
}

impl<'a> neco_ast::StructuredNode<'a> for StructuredNode<'a> {
    fn kind(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.node.name())
    }

    fn identifier(&self) -> Option<Cow<'a, str>> {
        self.node.first_string_arg().map(Cow::Borrowed)
    }

    fn attribute(&self, key: &str) -> Option<StructuredValue<'a>> {
        self.node.get(key).map(kdl_value_to_structured).or_else(|| {
            self.node.find_child(key).map(|child| {
                StructuredValue::Sequence(child.arg_values().map(kdl_value_to_structured).collect())
            })
        })
    }

    fn attribute_str(&self, key: &str) -> Option<Cow<'a, str>> {
        StructuredNode::attribute_str(self, key).map(Cow::Borrowed)
    }

    fn attribute_bool(&self, key: &str) -> Option<bool> {
        StructuredNode::attribute_bool(self, key)
    }

    fn type_annotation(&self) -> Option<Cow<'a, str>> {
        self.node.ty().map(Cow::Borrowed)
    }

    fn children(&self) -> Vec<Self> {
        StructuredNode::children(self).collect()
    }

    fn value(&self) -> StructuredValue<'a> {
        let mut fields = Vec::new();
        let mut arguments = Vec::new();
        for entry in self.node.entries() {
            match entry {
                KdlEntry::Argument { value, .. } => {
                    arguments.push(kdl_value_to_structured(value));
                }
                KdlEntry::Property { key, value, .. } => {
                    fields.push(StructuredField {
                        key: Cow::Borrowed(key.as_str()),
                        value: kdl_value_to_structured(value),
                    });
                }
            }
        }
        if !arguments.is_empty() {
            fields.push(StructuredField {
                key: Cow::Borrowed("$args"),
                value: StructuredValue::Sequence(arguments),
            });
        }
        StructuredValue::Mapping(fields)
    }
}

fn kdl_value_to_structured<'a>(value: &'a KdlValue) -> StructuredValue<'a> {
    match value {
        KdlValue::String(value) => StructuredValue::String(Cow::Borrowed(value.as_str())),
        KdlValue::Number(value) => StructuredValue::Number(StructuredNumber::from_parts(
            Cow::Borrowed(value.raw.as_str()),
            value.as_f64(),
        )),
        KdlValue::Bool(value) => StructuredValue::Bool(*value),
        KdlValue::Null => StructuredValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::{StructuredFacade, StructuredName, StructuredNode};
    use crate::{CrossRefParseError, NsidPath};
    use neco_kdl::parse;

    fn first_node(input: &str) -> neco_kdl::KdlDocument {
        parse(input).expect("valid KDL")
    }

    #[test]
    fn form_x_kind_is_node_name() {
        let doc = first_node("entity \"alpha.beta\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(
            node.structured_name_form_x(),
            StructuredName {
                kind: Some(NsidPath::parse("entity")),
                identifier: NsidPath::parse("alpha.beta")
            }
        );
    }

    #[test]
    fn form_y_kind_is_none() {
        let doc = first_node("alpha.beta\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.structured_name_form_y().kind, None);
    }

    #[test]
    fn form_y_identifier_is_node_name() {
        let doc = first_node("alpha.beta\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(
            node.structured_name_form_y().identifier,
            NsidPath::parse("alpha.beta")
        );
    }

    #[test]
    fn structured_name_default_prefers_form_x_when_first_arg_present() {
        let doc = first_node("entity \"alpha.beta\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.structured_name().kind, Some(NsidPath::parse("entity")));
    }

    #[test]
    fn structured_name_default_falls_back_to_form_y_when_no_first_arg() {
        let doc = first_node("alpha.beta\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.structured_name().kind, None);
    }

    #[test]
    fn node_name_with_dots_is_multi_segment_nsid() {
        let doc = first_node("alpha.beta.gamma\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.node_name_as_nsid().len(), 3);
    }

    #[test]
    fn first_arg_with_dots_is_multi_segment_nsid() {
        let doc = first_node("entity \"alpha.beta.gamma\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.first_arg_as_nsid().expect("first arg").len(), 3);
    }

    #[test]
    fn cross_ref_arg_for_string_first_arg() {
        let doc = first_node("edge \"alpha.beta#frag\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(
            node.cross_ref_arg()
                .expect("arg")
                .expect("cross ref")
                .fragment(),
            Some("frag")
        );
    }

    #[test]
    fn cross_ref_arg_returns_none_for_no_args() {
        let doc = first_node("edge\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert!(node.cross_ref_arg().is_none());
    }

    #[test]
    fn cross_ref_arg_returns_error_for_malformed() {
        let doc = first_node("edge \"alpha..beta\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(
            node.cross_ref_arg().expect("arg"),
            Err(CrossRefParseError::EmptySegment)
        );
    }

    #[test]
    fn cross_ref_string_args_collects_all() {
        let doc = first_node("edge \"alpha\" 1 \"beta#frag\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.cross_ref_string_args().len(), 2);
    }

    #[test]
    fn cross_ref_prop_for_named_property() {
        let doc = first_node("edge ref=\"alpha.beta\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(
            node.cross_ref_prop("ref")
                .expect("property")
                .expect("cross ref")
                .nsid(),
            &NsidPath::parse("alpha.beta")
        );
    }

    #[test]
    fn attribute_str_property_path() {
        let doc = first_node("node key=\"value\"\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.attribute_str("key"), Some("value"));
    }

    #[test]
    fn attribute_str_child_arg_path() {
        let doc = first_node("node { key \"value\" }\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.attribute_str("key"), Some("value"));
    }

    #[test]
    fn attribute_str_property_takes_precedence() {
        let doc = first_node("node key=\"property\" { key \"child\" }\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.attribute_str("key"), Some("property"));
    }

    #[test]
    fn attribute_bool_property_path() {
        let doc = first_node("node flag=#true\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.attribute_bool("flag"), Some(true));
    }

    #[test]
    fn dot_chain_depth_zero() {
        let doc = first_node("step\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.dot_chain_depth(), 0);
    }

    #[test]
    fn dot_chain_depth_one() {
        let doc = first_node(".step\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.dot_chain_depth(), 1);
    }

    #[test]
    fn dot_chain_depth_two() {
        let doc = first_node("..step\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.dot_chain_depth(), 2);
    }

    #[test]
    fn dot_chain_kind_strips_leading_dots() {
        let doc = first_node("..step\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_eq!(node.dot_chain_kind(), "step");
    }

    #[test]
    fn find_child_kind_returns_matching() {
        let doc = first_node("node { item \"a\" other \"b\" }\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        let found: Vec<_> = node.find_child_kind(&NsidPath::parse("item")).collect();
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn find_child_kind_empty_for_no_match() {
        let doc = first_node("node { item \"a\" }\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        let found: Vec<_> = node.find_child_kind(&NsidPath::parse("other")).collect();
        assert!(found.is_empty());
    }

    #[test]
    fn structured_facade_trait_impl_compiles() {
        fn assert_facade<'a, T: StructuredFacade<'a>>(_: T) {}

        let doc = first_node("node { item \"a\" }\n");
        let node = StructuredNode::from_kdl(&doc.nodes()[0]);
        assert_facade(node);
    }
}
