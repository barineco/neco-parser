/// A KDL v2 document consisting of zero or more nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct KdlDocument {
    pub nodes: Vec<KdlNode>,
}

impl KdlDocument {
    /// Returns the nodes in this document.
    pub fn nodes(&self) -> &[KdlNode] {
        &self.nodes
    }
}

/// A KDL v2 node with an optional type annotation, a name, entries, and optional child nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct KdlNode {
    /// Optional type annotation `(type)name`.
    pub ty: Option<String>,
    /// Node name.
    pub name: String,
    /// Arguments and properties in source order.
    pub entries: Vec<KdlEntry>,
    /// Child nodes from a `{ ... }` block.
    pub children: Option<Vec<KdlNode>>,
}

impl KdlNode {
    /// Returns the type annotation.
    pub fn ty(&self) -> Option<&str> {
        self.ty.as_deref()
    }

    /// Returns the node name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the entries.
    pub fn entries(&self) -> &[KdlEntry] {
        &self.entries
    }

    /// Returns the child nodes.
    pub fn children(&self) -> Option<&[KdlNode]> {
        self.children.as_deref()
    }
}

impl KdlNode {
    pub(crate) fn property_entry(&self, key: &str) -> Option<&KdlEntry> {
        self.entries.iter().rev().find(
            |entry| matches!(entry, KdlEntry::Property { key: candidate, .. } if candidate == key),
        )
    }

    /// Looks up a named property by `key` and returns the rightmost value.
    pub fn get(&self, key: &str) -> Option<&KdlValue> {
        self.property_entry(key).map(KdlEntry::value)
    }
}

impl KdlNode {
    /// Returns the value of the first argument entry, skipping properties.
    pub fn first_arg(&self) -> Option<&KdlValue> {
        self.entries.iter().find_map(|e| match e {
            KdlEntry::Argument { value, .. } => Some(value),
            _ => None,
        })
    }

    /// Returns the first argument as a string slice if it is a string.
    #[doc = include_str!("../docs/examples/first-string-argument.md")]
    pub fn first_string_arg(&self) -> Option<&str> {
        self.first_arg().and_then(KdlValue::as_str)
    }

    /// Iterates over the string values of argument entries in order.
    pub fn string_args(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().filter_map(|entry| match entry {
            KdlEntry::Argument { value, .. } => value.as_str(),
            _ => None,
        })
    }

    /// Iterates over the values of argument entries in order.
    pub fn arg_values(&self) -> impl Iterator<Item = &KdlValue> {
        self.entries.iter().filter_map(|entry| match entry {
            KdlEntry::Argument { value, .. } => Some(value),
            _ => None,
        })
    }

    /// Returns the first child node with the given `name`, or `None` if not found.
    #[doc = include_str!("../docs/examples/child-lookup.md")]
    pub fn find_child(&self, name: &str) -> Option<&KdlNode> {
        self.children.as_deref()?.iter().find(|c| c.name == name)
    }

    /// Returns an iterator over all child nodes with the given `name`.
    pub fn find_children<'s>(&'s self, name: &'s str) -> impl Iterator<Item = &'s KdlNode> + 's {
        self.children
            .as_deref()
            .into_iter()
            .flat_map(|s| s.iter())
            .filter(move |c| c.name == name)
    }

    /// Returns the value of the named property `key` as a string slice.
    pub fn string_prop(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(KdlValue::as_str)
    }

    /// Returns the value of the named property `key` as a `bool`.
    pub fn bool_prop(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(KdlValue::as_bool)
    }

    /// Returns the value of the named property `key` as an `i64`.
    pub fn int_prop(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(KdlValue::as_i64)
    }

    /// Collects `first_string_arg` from all child nodes named `child_name`.
    pub fn string_child_values(&self, child_name: &str) -> Vec<&str> {
        self.children
            .as_deref()
            .into_iter()
            .flat_map(|s| s.iter())
            .filter(|c| c.name == child_name)
            .filter_map(|c| c.first_string_arg())
            .collect()
    }
}

/// A node entry: either a positional argument or a named property.
#[derive(Debug, Clone, PartialEq)]
pub enum KdlEntry {
    Argument {
        ty: Option<String>,
        value: KdlValue,
    },
    Property {
        key: String,
        ty: Option<String>,
        value: KdlValue,
    },
}

impl KdlEntry {
    /// Returns the entry value.
    pub fn value(&self) -> &KdlValue {
        match self {
            KdlEntry::Argument { value, .. } => value,
            KdlEntry::Property { value, .. } => value,
        }
    }

    /// Returns the type annotation of this entry.
    pub fn ty(&self) -> Option<&str> {
        match self {
            KdlEntry::Argument { ty, .. } | KdlEntry::Property { ty, .. } => ty.as_deref(),
        }
    }
}

/// A half-open UTF-8 byte range in the source input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdlSpan {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

/// A KDL v2 document with source spans, returned by [`crate::parse_spanned`].
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedKdlDocument {
    nodes: Vec<SpannedKdlNode>,
}

impl SpannedKdlDocument {
    pub(crate) fn new(nodes: Vec<SpannedKdlNode>) -> Self {
        Self { nodes }
    }

    /// Returns the spanned nodes in this document.
    pub fn nodes(&self) -> &[SpannedKdlNode] {
        &self.nodes
    }

    /// Strips source spans and converts this document into a [`KdlDocument`].
    pub fn into_document(self) -> KdlDocument {
        KdlDocument {
            nodes: self
                .nodes
                .into_iter()
                .map(SpannedKdlNode::into_node)
                .collect(),
        }
    }
}

/// A KDL v2 node paired with its source span. See the crate README (`parse_spanned`) for span boundary rules.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedKdlNode {
    span: KdlSpan,
    ty: Option<String>,
    name: String,
    entries: Vec<SpannedKdlEntry>,
    children: Option<Vec<SpannedKdlNode>>,
}

impl SpannedKdlNode {
    pub(crate) fn new(
        span: KdlSpan,
        ty: Option<String>,
        name: String,
        entries: Vec<SpannedKdlEntry>,
        children: Option<Vec<SpannedKdlNode>>,
    ) -> Self {
        Self {
            span,
            ty,
            name,
            entries,
            children,
        }
    }

    /// Returns the source span of this node.
    pub fn span(&self) -> KdlSpan {
        self.span
    }

    /// Returns the type annotation.
    pub fn ty(&self) -> Option<&str> {
        self.ty.as_deref()
    }

    /// Returns the node name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the spanned entries.
    pub fn entries(&self) -> &[SpannedKdlEntry] {
        &self.entries
    }

    /// Returns the spanned child nodes.
    pub fn children(&self) -> Option<&[SpannedKdlNode]> {
        self.children.as_deref()
    }

    /// Strips source spans and converts this node into a [`KdlNode`].
    pub fn into_node(self) -> KdlNode {
        KdlNode {
            ty: self.ty,
            name: self.name,
            entries: self
                .entries
                .into_iter()
                .map(SpannedKdlEntry::into_entry)
                .collect(),
            children: self
                .children
                .map(|nodes| nodes.into_iter().map(SpannedKdlNode::into_node).collect()),
        }
    }
}

/// A KDL v2 entry paired with its source span. See the crate README (`parse_spanned`) for span boundary rules.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedKdlEntry {
    span: KdlSpan,
    entry: KdlEntry,
}

impl SpannedKdlEntry {
    pub(crate) fn new(span: KdlSpan, entry: KdlEntry) -> Self {
        Self { span, entry }
    }

    /// Returns the source span of this entry.
    pub fn span(&self) -> KdlSpan {
        self.span
    }

    /// Returns the underlying [`KdlEntry`].
    pub fn entry(&self) -> &KdlEntry {
        &self.entry
    }

    fn into_entry(self) -> KdlEntry {
        self.entry
    }
}

/// A KDL v2 value.
#[derive(Debug, Clone, PartialEq)]
pub enum KdlValue {
    String(String),
    Number(KdlNumber),
    Bool(bool),
    Null,
}

impl KdlValue {
    /// Returns the string slice if this value is a `String`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            KdlValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the boolean if this value is a `Bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            KdlValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the `f64` representation if this value is a `Number`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            KdlValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    /// Returns the `i64` representation if this value is a `Number`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            KdlValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }
}

/// Preserves a number's raw source spelling alongside its `i64` and `f64` interpretations when available.
///
/// KDL v2 places no limit on number size, so values outside the range of `i64` or `f64` are also accepted.
#[derive(Debug, Clone)]
pub struct KdlNumber {
    pub(crate) raw: String,

    pub(crate) as_i64: Option<i64>,

    pub(crate) as_f64: Option<f64>,
}

impl KdlNumber {
    /// Returns the raw source spelling.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Returns the `i64` value when the number is an integer literal that fits in `i64`.
    pub fn as_i64(&self) -> Option<i64> {
        self.as_i64
    }

    /// Returns the `f64` interpretation when available.
    pub fn as_f64(&self) -> Option<f64> {
        self.as_f64
    }
}

/// Error returned when a `KdlNumber` raw string and supplied interpretations do not match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberError;

impl core::fmt::Display for NumberError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "KDL number raw and interpreted values do not match")
    }
}

impl std::error::Error for NumberError {}

impl PartialEq for KdlNumber {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

/// A KDL parse error with a 1-based line, column, and error kind.
#[derive(Debug, Clone, PartialEq)]
pub struct KdlError {
    pub(crate) line: usize,

    pub(crate) col: usize,

    pub(crate) kind: KdlErrorKind,
}

impl KdlError {
    /// Returns the 1-based line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the 1-based column number, counted in Unicode scalar values.
    pub fn col(&self) -> usize {
        self.col
    }

    /// Returns the error kind.
    pub fn kind(&self) -> &KdlErrorKind {
        &self.kind
    }
}

impl core::fmt::Display for KdlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.kind)
    }
}

/// The specific cause of a [`KdlError`].
#[derive(Debug, Clone, PartialEq)]
pub enum KdlErrorKind {
    /// Unexpected character.
    UnexpectedChar(char),
    /// Unexpected end of input.
    UnexpectedEof,
    /// Invalid string escape sequence.
    InvalidEscape,
    /// Invalid Unicode escape sequence.
    InvalidUnicodeEscape,
    /// Invalid number literal.
    InvalidNumber,
    /// Disallowed Unicode code point.
    DisallowedCodePoint(char),
    /// A keyword literal (true, false, null, inf, -inf, or nan) used without a leading `#`.
    BareKeyword,
    /// Unmatched block comment closing delimiter `*/`.
    UnmatchedBlockCommentEnd,
    /// Unclosed block comment.
    UnclosedBlockComment,
    /// Unclosed string literal.
    UnclosedString,
    /// Inconsistent indentation in a multiline string.
    InconsistentIndentation,
    /// Slashdash (`/-`) in an invalid position.
    InvalidSlashdash,
}

impl core::fmt::Display for KdlErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedChar(c) => write!(f, "unexpected character: {:?}", c),
            Self::UnexpectedEof => write!(f, "unexpected end of input"),
            Self::InvalidEscape => write!(f, "invalid escape sequence"),
            Self::InvalidUnicodeEscape => write!(f, "invalid unicode escape"),
            Self::InvalidNumber => write!(f, "invalid number literal"),
            Self::DisallowedCodePoint(c) => {
                write!(f, "disallowed code point: U+{:04X}", *c as u32)
            }
            Self::BareKeyword => write!(f, "bare keyword (use #true, #false, #null, etc.)"),
            Self::UnmatchedBlockCommentEnd => write!(f, "unmatched */"),
            Self::UnclosedBlockComment => write!(f, "unclosed block comment"),
            Self::UnclosedString => write!(f, "unclosed string"),
            Self::InconsistentIndentation => write!(f, "inconsistent multiline string indentation"),
            Self::InvalidSlashdash => write!(f, "slashdash in invalid position"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_number(raw: &str, i: Option<i64>, f: Option<f64>) -> KdlNumber {
        KdlNumber {
            raw: raw.to_string(),
            as_i64: i,
            as_f64: f,
        }
    }

    #[test]
    fn number_constructor_accepts_matching_components() {
        assert!(KdlNumber::new("5", Some(5), Some(5.0)).is_ok());
        assert!(KdlNumber::new("1.0", None, Some(1.0)).is_ok());
        assert!(KdlNumber::new("#nan", None, Some(f64::NAN)).is_ok());
        assert!(KdlNumber::new("-0", Some(0), Some(-0.0)).is_ok());
        assert!(KdlNumber::new("-0.0", None, Some(-0.0)).is_ok());
    }

    #[test]
    fn number_constructor_rejects_invalid_raw() {
        for raw in ["", "0xG", "0x", "0b2", "+"] {
            assert_eq!(KdlNumber::new(raw, None, None), Err(NumberError));
        }
    }

    #[test]
    fn number_constructor_rejects_mismatched_components() {
        assert_eq!(KdlNumber::new("5", Some(5), Some(999.0)), Err(NumberError));
        assert_eq!(KdlNumber::new("5", None, None), Err(NumberError));
        assert_eq!(KdlNumber::new("-0", Some(0), Some(0.0)), Err(NumberError));
        assert_eq!(KdlNumber::new("-0.0", None, Some(0.0)), Err(NumberError));
    }

    #[test]
    fn as_str_returns_some_for_string() {
        let v = KdlValue::String("hello".to_string());
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn as_str_returns_none_for_number() {
        let v = KdlValue::Number(make_number("42", Some(42), Some(42.0)));
        assert_eq!(v.as_str(), None);
    }

    #[test]
    fn as_bool_returns_some_for_bool() {
        assert_eq!(KdlValue::Bool(true).as_bool(), Some(true));
        assert_eq!(KdlValue::Bool(false).as_bool(), Some(false));
    }

    #[test]
    fn as_bool_returns_none_for_string() {
        let v = KdlValue::String("true".to_string());
        assert_eq!(v.as_bool(), None);
    }

    #[test]
    fn as_f64_returns_some_for_number() {
        let v = KdlValue::Number(make_number("2.5", None, Some(2.5)));
        assert_eq!(v.as_f64(), Some(2.5));
    }

    #[test]
    fn as_f64_returns_none_for_bool() {
        assert_eq!(KdlValue::Bool(true).as_f64(), None);
    }

    #[test]
    fn as_i64_returns_some_for_integer() {
        let v = KdlValue::Number(make_number("42", Some(42), Some(42.0)));
        assert_eq!(v.as_i64(), Some(42));
    }

    #[test]
    fn as_i64_returns_none_for_float_only() {
        let v = KdlValue::Number(make_number("2.5", None, Some(2.5)));
        assert_eq!(v.as_i64(), None);
    }

    #[test]
    fn entry_value_for_argument() {
        let entry = KdlEntry::Argument {
            ty: None,
            value: KdlValue::String("arg".to_string()),
        };
        assert_eq!(entry.value(), &KdlValue::String("arg".to_string()));
    }

    #[test]
    fn entry_value_for_property() {
        let entry = KdlEntry::Property {
            key: "key".to_string(),
            ty: None,
            value: KdlValue::Bool(true),
        };
        assert_eq!(entry.value(), &KdlValue::Bool(true));
    }

    #[test]
    fn node_get_returns_some_for_existing_key() {
        let node = KdlNode {
            ty: None,
            name: "test".to_string(),
            entries: vec![
                KdlEntry::Argument {
                    ty: None,
                    value: KdlValue::String("positional".to_string()),
                },
                KdlEntry::Property {
                    key: "color".to_string(),
                    ty: None,
                    value: KdlValue::String("red".to_string()),
                },
            ],
            children: None,
        };
        assert_eq!(
            node.get("color"),
            Some(&KdlValue::String("red".to_string()))
        );
    }

    #[test]
    fn node_get_uses_rightmost_duplicate_property() {
        let node = KdlNode {
            ty: None,
            name: "test".to_string(),
            entries: vec![
                KdlEntry::Property {
                    key: "value".to_string(),
                    ty: None,
                    value: KdlValue::String("first".to_string()),
                },
                KdlEntry::Property {
                    key: "value".to_string(),
                    ty: None,
                    value: KdlValue::String("second".to_string()),
                },
            ],
            children: None,
        };

        assert_eq!(
            node.get("value"),
            Some(&KdlValue::String("second".to_string()))
        );
        assert_eq!(node.string_prop("value"), Some("second"));
        assert_eq!(node.entries().len(), 2);
    }

    #[test]
    fn node_get_returns_none_for_missing_key() {
        let node = KdlNode {
            ty: None,
            name: "test".to_string(),
            entries: vec![KdlEntry::Argument {
                ty: None,
                value: KdlValue::String("positional".to_string()),
            }],
            children: None,
        };
        assert_eq!(node.get("missing"), None);
    }

    fn arg(value: KdlValue) -> KdlEntry {
        KdlEntry::Argument { ty: None, value }
    }

    fn prop(key: &str, value: KdlValue) -> KdlEntry {
        KdlEntry::Property {
            key: key.to_string(),
            ty: None,
            value,
        }
    }

    fn typed_arg(ty: &str, value: KdlValue) -> KdlEntry {
        KdlEntry::Argument {
            ty: Some(ty.to_string()),
            value,
        }
    }

    fn typed_prop(key: &str, ty: &str, value: KdlValue) -> KdlEntry {
        KdlEntry::Property {
            key: key.to_string(),
            ty: Some(ty.to_string()),
            value,
        }
    }

    fn node(name: &str, entries: Vec<KdlEntry>, children: Option<Vec<KdlNode>>) -> KdlNode {
        KdlNode {
            ty: None,
            name: name.to_string(),
            entries,
            children,
        }
    }

    #[test]
    fn first_arg_skips_property() {
        let n = node(
            "n",
            vec![
                prop("k", KdlValue::Bool(true)),
                arg(KdlValue::String("v".to_string())),
            ],
            None,
        );
        assert_eq!(n.first_arg(), Some(&KdlValue::String("v".to_string())));
    }

    #[test]
    fn first_arg_returns_none_when_no_args() {
        let n = node("n", vec![prop("k", KdlValue::Bool(true))], None);
        assert_eq!(n.first_arg(), None);
    }

    #[test]
    fn first_string_arg_for_string() {
        let n = node("n", vec![arg(KdlValue::String("hi".to_string()))], None);
        assert_eq!(n.first_string_arg(), Some("hi"));
    }

    #[test]
    fn first_string_arg_for_non_string() {
        let n = node(
            "n",
            vec![arg(KdlValue::Number(make_number("1", Some(1), Some(1.0))))],
            None,
        );
        assert_eq!(n.first_string_arg(), None);
    }

    #[test]
    fn find_child_returns_first_match() {
        let parent = node(
            "p",
            vec![],
            Some(vec![
                node("a", vec![arg(KdlValue::String("first".to_string()))], None),
                node("a", vec![arg(KdlValue::String("second".to_string()))], None),
            ]),
        );
        assert_eq!(
            parent.find_child("a").map(|c| c.first_string_arg()),
            Some(Some("first")),
        );
    }

    #[test]
    fn find_child_returns_none_when_no_children() {
        let n = node("n", vec![], None);
        assert!(n.find_child("a").is_none());
    }

    #[test]
    fn find_children_iterates_all_matches() {
        let parent = node(
            "p",
            vec![],
            Some(vec![
                node("a", vec![arg(KdlValue::String("1".to_string()))], None),
                node("b", vec![], None),
                node("a", vec![arg(KdlValue::String("2".to_string()))], None),
            ]),
        );
        let collected: Vec<_> = parent.find_children("a").map(|c| c.name.clone()).collect();
        assert_eq!(collected, vec!["a".to_string(), "a".to_string()]);
    }

    #[test]
    fn find_children_empty_for_no_match() {
        let parent = node("p", vec![], Some(vec![node("a", vec![], None)]));
        assert_eq!(parent.find_children("z").count(), 0);
    }

    #[test]
    fn string_prop_for_string_value() {
        let n = node(
            "n",
            vec![prop("k", KdlValue::String("hello".to_string()))],
            None,
        );
        assert_eq!(n.string_prop("k"), Some("hello"));
    }

    #[test]
    fn string_prop_for_wrong_type() {
        let n = node(
            "n",
            vec![prop(
                "k",
                KdlValue::Number(make_number("1", Some(1), Some(1.0))),
            )],
            None,
        );
        assert_eq!(n.string_prop("k"), None);
    }

    #[test]
    fn string_prop_missing_key() {
        let n = node("n", vec![prop("k", KdlValue::Bool(true))], None);
        assert_eq!(n.string_prop("absent"), None);
    }

    #[test]
    fn bool_prop_for_bool_value() {
        let n = node("n", vec![prop("flag", KdlValue::Bool(true))], None);
        assert_eq!(n.bool_prop("flag"), Some(true));
    }

    #[test]
    fn bool_prop_for_wrong_type() {
        let n = node(
            "n",
            vec![prop("flag", KdlValue::String("true".to_string()))],
            None,
        );
        assert_eq!(n.bool_prop("flag"), None);
    }

    #[test]
    fn bool_prop_missing_key() {
        let n = node("n", vec![], None);
        assert_eq!(n.bool_prop("absent"), None);
    }

    #[test]
    fn int_prop_for_int_value() {
        let n = node(
            "n",
            vec![prop(
                "count",
                KdlValue::Number(make_number("42", Some(42), Some(42.0))),
            )],
            None,
        );
        assert_eq!(n.int_prop("count"), Some(42));
    }

    #[test]
    fn int_prop_for_float_only() {
        let n = node(
            "n",
            vec![prop(
                "x",
                KdlValue::Number(make_number("2.5", None, Some(2.5))),
            )],
            None,
        );
        assert_eq!(n.int_prop("x"), None);
    }

    #[test]
    fn int_prop_missing_key() {
        let n = node("n", vec![], None);
        assert_eq!(n.int_prop("absent"), None);
    }

    #[test]
    fn string_child_values_collects_string_args() {
        let parent = node(
            "p",
            vec![],
            Some(vec![
                node("item", vec![arg(KdlValue::String("a".to_string()))], None),
                node("item", vec![arg(KdlValue::String("b".to_string()))], None),
                node("other", vec![arg(KdlValue::String("z".to_string()))], None),
            ]),
        );
        assert_eq!(parent.string_child_values("item"), vec!["a", "b"]);
    }

    #[test]
    fn string_child_values_skips_non_string() {
        let parent = node(
            "p",
            vec![],
            Some(vec![
                node("item", vec![arg(KdlValue::String("a".to_string()))], None),
                node(
                    "item",
                    vec![arg(KdlValue::Number(make_number("1", Some(1), Some(1.0))))],
                    None,
                ),
            ]),
        );
        assert_eq!(parent.string_child_values("item"), vec!["a"]);
    }

    #[test]
    fn entry_ty_for_typed_argument() {
        let e = typed_arg(
            "u32",
            KdlValue::Number(make_number("1", Some(1), Some(1.0))),
        );
        assert_eq!(e.ty(), Some("u32"));
    }

    #[test]
    fn entry_ty_for_untyped_argument() {
        let e = arg(KdlValue::Bool(true));
        assert_eq!(e.ty(), None);
    }

    #[test]
    fn entry_ty_for_typed_property() {
        let e = typed_prop(
            "k",
            "i64",
            KdlValue::Number(make_number("7", Some(7), Some(7.0))),
        );
        assert_eq!(e.ty(), Some("i64"));
    }

    #[test]
    fn entry_ty_for_untyped_property() {
        let e = prop("k", KdlValue::Bool(false));
        assert_eq!(e.ty(), None);
    }
}
