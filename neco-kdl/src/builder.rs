//! KDL ノード / ドキュメントを method chain で組み立てる Builder API。

use crate::value::{KdlDocument, KdlEntry, KdlNode, KdlNumber, KdlValue};

/// `KdlNode` を method chain で組み立てる builder。
#[derive(Debug, Clone)]
pub struct KdlNodeBuilder {
    inner: KdlNode,
}

impl KdlNodeBuilder {
    /// 指定名の Builder を生成する。
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

    /// node-level type annotation `(T)name` の `T` を設定する。
    pub fn ty(mut self, ty: impl Into<String>) -> Self {
        self.inner.ty = Some(ty.into());
        self
    }

    /// 任意 KdlValue を Argument として追加する。
    pub fn arg(mut self, value: KdlValue) -> Self {
        self.inner
            .entries
            .push(KdlEntry::Argument { ty: None, value });
        self
    }

    /// 文字列を Argument として追加する。
    pub fn string_arg(self, value: impl Into<String>) -> Self {
        self.arg(KdlValue::String(value.into()))
    }

    /// bool を Argument として追加する。
    pub fn bool_arg(self, value: bool) -> Self {
        self.arg(KdlValue::Bool(value))
    }

    /// i64 を Argument として追加する。
    pub fn int_arg(self, value: i64) -> Self {
        self.arg(KdlValue::Number(KdlNumber {
            raw: value.to_string(),
            as_i64: Some(value),
            as_f64: Some(value as f64),
        }))
    }

    /// type annotation 付き Argument を追加する。
    pub fn typed_arg(mut self, ty: impl Into<String>, value: KdlValue) -> Self {
        self.inner.entries.push(KdlEntry::Argument {
            ty: Some(ty.into()),
            value,
        });
        self
    }

    /// 任意 KdlValue を named property として追加する。
    pub fn prop(mut self, key: impl Into<String>, value: KdlValue) -> Self {
        self.inner.entries.push(KdlEntry::Property {
            key: key.into(),
            ty: None,
            value,
        });
        self
    }

    /// 文字列 property を追加する。
    pub fn string_prop(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.prop(key, KdlValue::String(value.into()))
    }

    /// bool property を追加する。
    pub fn bool_prop(self, key: impl Into<String>, value: bool) -> Self {
        self.prop(key, KdlValue::Bool(value))
    }

    /// i64 property を追加する。
    pub fn int_prop(self, key: impl Into<String>, value: i64) -> Self {
        self.prop(
            key,
            KdlValue::Number(KdlNumber {
                raw: value.to_string(),
                as_i64: Some(value),
                as_f64: Some(value as f64),
            }),
        )
    }

    /// type annotation 付き property を追加する。
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

    /// 子ノードを 1 件追加する。
    pub fn child(mut self, child: KdlNode) -> Self {
        self.inner.children.get_or_insert_with(Vec::new).push(child);
        self
    }

    /// 子ノードを iterator から append する。
    pub fn children<I: IntoIterator<Item = KdlNode>>(mut self, iter: I) -> Self {
        self.inner
            .children
            .get_or_insert_with(Vec::new)
            .extend(iter);
        self
    }

    /// 構築した `KdlNode` を返す。
    pub fn build(self) -> KdlNode {
        self.inner
    }
}

/// `KdlDocument` を method chain で組み立てる builder。
#[derive(Debug, Clone)]
pub struct KdlDocumentBuilder {
    inner: KdlDocument,
}

impl KdlDocumentBuilder {
    /// 空のドキュメント Builder を生成する。
    pub fn new() -> Self {
        Self {
            inner: KdlDocument { nodes: Vec::new() },
        }
    }

    /// ノードを 1 件追加する。
    pub fn node(mut self, node: KdlNode) -> Self {
        self.inner.nodes.push(node);
        self
    }

    /// ノードを iterator から append する。
    pub fn nodes<I: IntoIterator<Item = KdlNode>>(mut self, iter: I) -> Self {
        self.inner.nodes.extend(iter);
        self
    }

    /// 構築した `KdlDocument` を返す。
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
    /// 指定名の `KdlNodeBuilder` を返す。
    ///
    /// ```
    /// use neco_kdl::KdlNode;
    /// let n = KdlNode::builder("greet").string_arg("hello").build();
    /// assert_eq!(n.name(), "greet");
    /// assert_eq!(n.first_string_arg(), Some("hello"));
    /// ```
    pub fn builder(name: impl Into<String>) -> KdlNodeBuilder {
        KdlNodeBuilder::new(name)
    }
}

impl KdlDocument {
    /// 空の `KdlDocumentBuilder` を返す。
    ///
    /// ```
    /// use neco_kdl::{KdlDocument, KdlNode};
    /// let doc = KdlDocument::builder()
    ///     .node(KdlNode::builder("alpha").build())
    ///     .node(KdlNode::builder("beta").build())
    ///     .build();
    /// assert_eq!(doc.nodes().len(), 2);
    /// ```
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

        // builder で再構築
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

        // 再 parse して構造一致を確認
        let reparsed = parse(&rebuilt_serialized).expect("reparse");
        assert_eq!(reparsed, parsed);
    }
}
