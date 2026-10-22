//! Builders for constructing KDL nodes and documents via method chaining.

use crate::value::{KdlDocument, KdlEntry, KdlNode, KdlNumber, KdlValue};

/// A builder for constructing a [`KdlNode`] via method chaining.
#[derive(Debug, Clone)]
pub struct KdlNodeBuilder {
    inner: KdlNode,
}

impl KdlNodeBuilder {
    /// Creates a builder for a node with the given `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: KdlNode {
                ty: None,
                name: name.into(),
                entries: Vec::new(),
                children: None,
            },
        }
    }

    /// Sets the node type annotation `T` in `(T)name`.
    pub fn ty(mut self, ty: impl Into<String>) -> Self {
        self.inner.ty = Some(ty.into());
        self
    }

    /// Appends a [`KdlValue`] as a positional argument.
    pub fn arg(mut self, value: KdlValue) -> Self {
        self.inner
            .entries
            .push(KdlEntry::Argument { ty: None, value });
        self
    }

    /// Appends a string as a positional argument.
    pub fn string_arg(self, value: impl Into<String>) -> Self {
        self.arg(KdlValue::String(value.into()))
    }

    /// Appends a `bool` as a positional argument.
    pub fn bool_arg(self, value: bool) -> Self {
        self.arg(KdlValue::Bool(value))
    }

    /// Appends an `i64` as a positional argument.
    pub fn int_arg(self, value: i64) -> Self {
        self.arg(KdlValue::Number(KdlNumber::from_i64(value)))
    }

    /// Appends a positional argument with a type annotation.
    pub fn typed_arg(mut self, ty: impl Into<String>, value: KdlValue) -> Self {
        self.inner.entries.push(KdlEntry::Argument {
            ty: Some(ty.into()),
            value,
        });
        self
    }

    /// Appends a [`KdlValue`] as a named property.
    pub fn prop(mut self, key: impl Into<String>, value: KdlValue) -> Self {
        self.inner.entries.push(KdlEntry::Property {
            key: key.into(),
            ty: None,
            value,
        });
        self
    }

    /// Appends a string property.
    pub fn string_prop(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.prop(key, KdlValue::String(value.into()))
    }

    /// Appends a `bool` property.
    pub fn bool_prop(self, key: impl Into<String>, value: bool) -> Self {
        self.prop(key, KdlValue::Bool(value))
    }

    /// Appends an `i64` property.
    pub fn int_prop(self, key: impl Into<String>, value: i64) -> Self {
        self.prop(key, KdlValue::Number(KdlNumber::from_i64(value)))
    }

    /// Appends a named property with a type annotation.
    pub fn typed_prop(
        mut self,
        key: impl Into<String>,
        ty: impl Into<String>,
        value: KdlValue,
    ) -> Self {
        self.inner.entries.push(KdlEntry::Property {
            key: key.into(),
            ty: Some(ty.into()),
            value,
        });
        self
    }

    /// Appends a single child node.
    pub fn child(mut self, child: KdlNode) -> Self {
        self.inner.children.get_or_insert_with(Vec::new).push(child);
        self
    }

    /// Appends child nodes from an iterator.
    pub fn children<I: IntoIterator<Item = KdlNode>>(mut self, iter: I) -> Self {
        self.inner
            .children
            .get_or_insert_with(Vec::new)
            .extend(iter);
        self
    }

    /// Consumes the builder and returns the constructed [`KdlNode`].
    pub fn build(self) -> KdlNode {
        self.inner
    }
}

/// A builder for constructing a [`KdlDocument`] via method chaining.
#[derive(Debug, Clone)]
pub struct KdlDocumentBuilder {
    inner: KdlDocument,
}

impl KdlDocumentBuilder {
    /// Creates an empty document builder.
    pub fn new() -> Self {
        Self {
            inner: KdlDocument { nodes: Vec::new() },
        }
    }

    /// Appends a single node.
    pub fn node(mut self, node: KdlNode) -> Self {
        self.inner.nodes.push(node);
        self
    }

    /// Appends nodes from an iterator.
    pub fn nodes<I: IntoIterator<Item = KdlNode>>(mut self, iter: I) -> Self {
        self.inner.nodes.extend(iter);
        self
    }

    /// Consumes the builder and returns the constructed [`KdlDocument`].
    pub fn build(self) -> KdlDocument {
        self.inner
    }
}

impl Default for KdlDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KdlNode {
    /// Returns a [`KdlNodeBuilder`] for a node with the given `name`.
    /// Chain entry and child methods, then call [`KdlNodeBuilder::build`] to finish.
    #[doc = include_str!("../docs/examples/node-builder.md")]
    pub fn builder(name: impl Into<String>) -> KdlNodeBuilder {
        KdlNodeBuilder::new(name)
    }
}

impl KdlDocument {
    /// Returns an empty [`KdlDocumentBuilder`].
    /// Chain [`KdlDocumentBuilder::node`] calls, then call [`KdlDocumentBuilder::build`] to finish.
    #[doc = include_str!("../docs/examples/document-builder.md")]
    pub fn builder() -> KdlDocumentBuilder {
        KdlDocumentBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use crate::serialize::serialize;

    #[test]
    fn node_builder_basic() {
        let n = KdlNode::builder("foo").build();
        assert_eq!(n.name(), "foo");
        assert!(n.entries().is_empty());
        assert!(n.children().is_none());
        assert!(n.ty().is_none());
    }

    #[test]
    fn node_builder_with_ty() {
        let n = KdlNode::builder("v").ty("u32").build();
        assert_eq!(n.ty(), Some("u32"));
    }

    #[test]
    fn node_builder_arg_chain() {
        let n = KdlNode::builder("n")
            .string_arg("first")
            .int_arg(42)
            .bool_arg(true)
            .build();
        assert_eq!(n.entries().len(), 3);
        assert_eq!(n.first_string_arg(), Some("first"));
    }

    #[test]
    fn node_builder_prop_chain() {
        let n = KdlNode::builder("n")
            .string_prop("color", "red")
            .int_prop("count", 7)
            .bool_prop("flag", false)
            .build();
        assert_eq!(n.string_prop("color"), Some("red"));
        assert_eq!(n.int_prop("count"), Some(7));
        assert_eq!(n.bool_prop("flag"), Some(false));
    }

    #[test]
    fn node_builder_typed_arg_and_prop() {
        let n = KdlNode::builder("n")
            .typed_arg(
                "u32",
                KdlValue::Number(KdlNumber {
                    raw: "1".to_string(),
                    as_i64: Some(1),
                    as_f64: Some(1.0),
                }),
            )
            .typed_prop(
                "k",
                "i64",
                KdlValue::Number(KdlNumber {
                    raw: "9".to_string(),
                    as_i64: Some(9),
                    as_f64: Some(9.0),
                }),
            )
            .build();
        assert_eq!(n.entries()[0].ty(), Some("u32"));
        assert_eq!(n.entries()[1].ty(), Some("i64"));
    }

    #[test]
    fn node_builder_child() {
        let c = KdlNode::builder("inner").string_arg("x").build();
        let n = KdlNode::builder("outer").child(c).build();
        assert_eq!(n.children().map(|s| s.len()), Some(1));
        assert_eq!(
            n.find_child("inner").and_then(|c| c.first_string_arg()),
            Some("x")
        );
    }

    #[test]
    fn node_builder_children_iter() {
        let kids = vec![KdlNode::builder("a").build(), KdlNode::builder("b").build()];
        let n = KdlNode::builder("p").children(kids).build();
        assert_eq!(n.children().map(|s| s.len()), Some(2));
    }

    #[test]
    fn node_builder_duplicate_props() {
        let n = KdlNode::builder("n")
            .string_prop("k", "first")
            .string_prop("k", "second")
            .build();
        assert_eq!(n.entries().len(), 2);
    }

    #[test]
    fn document_builder_basic() {
        let n = KdlNode::builder("alpha").string_arg("v").build();
        let doc = KdlDocument::builder().node(n).build();
        assert_eq!(doc.nodes().len(), 1);
        assert_eq!(doc.nodes()[0].name(), "alpha");
    }

    #[test]
    fn document_builder_nodes_iter() {
        let nodes = vec![
            KdlNode::builder("a").build(),
            KdlNode::builder("b").build(),
            KdlNode::builder("c").build(),
        ];
        let doc = KdlDocument::builder().nodes(nodes).build();
        assert_eq!(doc.nodes().len(), 3);
    }

    #[test]
    fn document_builder_default() {
        let doc = KdlDocumentBuilder::default().build();
        assert!(doc.nodes().is_empty());
    }

    #[test]
    fn roundtrip_via_builder() {
        let input = "node1 \"first\" 42 enabled=#true {\n    child1 \"a\"\n    child1 \"b\"\n    child2 (i32)7\n}\n";
        let parsed = parse(input).expect("parse");
        let original_serialized = serialize(&parsed);

        let rebuilt = KdlDocument::builder()
            .node(
                KdlNode::builder("node1")
                    .string_arg("first")
                    .int_arg(42)
                    .bool_prop("enabled", true)
                    .child(KdlNode::builder("child1").string_arg("a").build())
                    .child(KdlNode::builder("child1").string_arg("b").build())
                    .child(
                        KdlNode::builder("child2")
                            .typed_arg(
                                "i32",
                                KdlValue::Number(KdlNumber {
                                    raw: "7".to_string(),
                                    as_i64: Some(7),
                                    as_f64: Some(7.0),
                                }),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        let rebuilt_serialized = serialize(&rebuilt);
        assert_eq!(rebuilt_serialized, original_serialized);

        let reparsed = parse(&rebuilt_serialized).expect("reparse");
        assert_eq!(reparsed, parsed);
    }
}
