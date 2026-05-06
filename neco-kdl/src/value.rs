/// KDL v2 ドキュメント。ゼロ個以上のノードで構成される。
#[derive(Debug, Clone, PartialEq)]
pub struct KdlDocument {
    pub nodes: Vec<KdlNode>,
}

impl KdlDocument {
    /// ドキュメント内のノード一覧を返す。
    pub fn nodes(&self) -> &[KdlNode] {
        &self.nodes
    }
}

/// KDL v2 ノード。名前、エントリ群(argument + property)、子ノードを持つ。
#[derive(Debug, Clone, PartialEq)]
pub struct KdlNode {
    /// type annotation `(type)name`
    pub ty: Option<String>,
    /// ノード名
    pub name: String,
    /// argument と property を出現順で保持
    pub entries: Vec<KdlEntry>,
    /// children block `{ ... }`
    pub children: Option<Vec<KdlNode>>,
}

impl KdlNode {
    /// type annotation を返す。
    pub fn ty(&self) -> Option<&str> {
        self.ty.as_deref()
    }

    /// ノード名を返す。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// エントリ一覧を返す。
    pub fn entries(&self) -> &[KdlEntry] {
        &self.entries
    }

    /// 子ノードを返す。
    pub fn children(&self) -> Option<&[KdlNode]> {
        self.children.as_deref()
    }
}

impl KdlNode {
    /// named property を key で検索し、最初にマッチした値を返す。
    pub fn get(&self, key: &str) -> Option<&KdlValue> {
        self.entries().iter().find_map(|entry| match entry {
            KdlEntry::Property { key: k, value, .. } if k == key => Some(value),
            _ => None,
        })
    }
}

impl KdlNode {
    /// 最初の Argument エントリの値を返す。 Property は skip する。
    pub fn first_arg(&self) -> Option<&KdlValue> {
        self.entries.iter().find_map(|e| match e {
            KdlEntry::Argument { value, .. } => Some(value),
            _ => None,
        })
    }

    /// 最初の Argument エントリが文字列であればその参照を返す。
    ///
    /// ```
    /// use neco_kdl::parse;
    /// let doc = parse("name \"alice\"\n").unwrap();
    /// assert_eq!(doc.nodes()[0].first_string_arg(), Some("alice"));
    /// ```
    pub fn first_string_arg(&self) -> Option<&str> {
        self.first_arg().and_then(KdlValue::as_str)
    }

    /// Argument エントリのうち文字列値だけを出現順で返す。
    pub fn string_args(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().filter_map(|entry| match entry {
            KdlEntry::Argument { value, .. } => value.as_str(),
            _ => None,
        })
    }

    /// Argument エントリの値を出現順で返す。
    pub fn arg_values(&self) -> impl Iterator<Item = &KdlValue> {
        self.entries.iter().filter_map(|entry| match entry {
            KdlEntry::Argument { value, .. } => Some(value),
            _ => None,
        })
    }

    /// 同名の最初の子ノードを返す。
    ///
    /// ```
    /// use neco_kdl::parse;
    /// let doc = parse("parent {\n    item \"a\"\n    item \"b\"\n}\n").unwrap();
    /// let parent = &doc.nodes()[0];
    /// assert_eq!(parent.find_child("item").and_then(|c| c.first_string_arg()), Some("a"));
    /// ```
    pub fn find_child(&self, name: &str) -> Option<&KdlNode> {
        self.children.as_deref()?.iter().find(|c| c.name == name)
    }

    /// 同名の子ノードをすべて返す iterator。
    pub fn find_children<'s>(&'s self, name: &'s str) -> impl Iterator<Item = &'s KdlNode> + 's {
        self.children
            .as_deref()
            .into_iter()
            .flat_map(|s| s.iter())
            .filter(move |c| c.name == name)
    }

    /// 名前付き property を文字列として返す。
    pub fn string_prop(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(KdlValue::as_str)
    }

    /// 名前付き property を bool として返す。
    pub fn bool_prop(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(KdlValue::as_bool)
    }

    /// 名前付き property を i64 として返す。
    pub fn int_prop(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(KdlValue::as_i64)
    }

    /// 同名子ノードのうち first_string_arg が取れるものを集める。
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

/// ノードのエントリ。argument(位置引数)または property(名前付き引数)。
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
    /// エントリの値を返す。
    pub fn value(&self) -> &KdlValue {
        match self {
            KdlEntry::Argument { value, .. } => value,
            KdlEntry::Property { value, .. } => value,
        }
    }

    /// エントリの type annotation を返す。
    pub fn ty(&self) -> Option<&str> {
        match self {
            KdlEntry::Argument { ty, .. } | KdlEntry::Property { ty, .. } => ty.as_deref(),
        }
    }
}

/// KDL v2 の値。
#[derive(Debug, Clone, PartialEq)]
pub enum KdlValue {
    String(String),
    Number(KdlNumber),
    Bool(bool),
    Null,
}

impl KdlValue {
    /// 文字列値を返す。
    pub fn as_str(&self) -> Option<&str> {
        match self {
            KdlValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// bool 値を返す。
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            KdlValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// f64 値を返す。
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            KdlValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    /// i64 値を返す。
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            KdlValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }
}

/// 数値の raw 文字列を保持しつつ、可能な場合は解釈済み値も提供する。
///
/// KDL v2 は数値サイズに制限を置かないため、i64/f64 に収まらない値も受理する。
#[derive(Debug, Clone)]
pub struct KdlNumber {
    /// 原文(アンダースコア・プレフィックス含む)
    pub raw: String,
    /// 整数として解釈可能な場合
    pub as_i64: Option<i64>,
    /// 浮動小数点として解釈可能な場合(#inf, #-inf, #nan 含む)
    pub as_f64: Option<f64>,
}

impl KdlNumber {
    /// 原文を返す。
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// 整数として解釈可能な場合の値を返す。
    pub fn as_i64(&self) -> Option<i64> {
        self.as_i64
    }

    /// 浮動小数点として解釈可能な場合の値を返す。
    pub fn as_f64(&self) -> Option<f64> {
        self.as_f64
    }
}

impl PartialEq for KdlNumber {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

/// パースエラー。
#[derive(Debug, Clone, PartialEq)]
pub struct KdlError {
    /// 1-based 行番号
    pub(crate) line: usize,
    /// 1-based 列番号(Unicode scalar value 単位)
    pub(crate) col: usize,
    /// エラー種別
    pub(crate) kind: KdlErrorKind,
}

impl KdlError {
    /// 行番号(1-based)を返す。
    pub fn line(&self) -> usize {
        self.line
    }

    /// 列番号(1-based、Unicode scalar value 単位)を返す。
    pub fn col(&self) -> usize {
        self.col
    }

    /// エラー種別を返す。
    pub fn kind(&self) -> &KdlErrorKind {
        &self.kind
    }
}

impl core::fmt::Display for KdlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.kind)
    }
}

/// エラー種別。
#[derive(Debug, Clone, PartialEq)]
pub enum KdlErrorKind {
    /// 予期しない文字
    UnexpectedChar(char),
    /// 予期しない EOF
    UnexpectedEof,
    /// 不正な文字列エスケープ
    InvalidEscape,
    /// 不正な Unicode エスケープ
    InvalidUnicodeEscape,
    /// 不正な数値リテラル
    InvalidNumber,
    /// 禁止コードポイント
    DisallowedCodePoint(char),
    /// 裸キーワード(true, false, null, inf, -inf, nan)
    BareKeyword,
    /// ネストされていないブロックコメント終端
    UnmatchedBlockCommentEnd,
    /// 閉じられていないブロックコメント
    UnclosedBlockComment,
    /// 閉じられていない文字列
    UnclosedString,
    /// multiline string のインデント不一致
    InconsistentIndentation,
    /// slashdash の不正な位置
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

    // --- KdlValue::as_str ---

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

    // --- KdlValue::as_bool ---

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

    // --- KdlValue::as_f64 ---

    #[test]
    fn as_f64_returns_some_for_number() {
        let v = KdlValue::Number(make_number("2.5", None, Some(2.5)));
        assert_eq!(v.as_f64(), Some(2.5));
    }

    #[test]
    fn as_f64_returns_none_for_bool() {
        assert_eq!(KdlValue::Bool(true).as_f64(), None);
    }

    // --- KdlValue::as_i64 ---

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

    // --- KdlEntry::value ---

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

    // --- KdlNode::get ---

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

    // --- helpers for navigation tests ---

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

    // --- KdlNode::first_arg / first_string_arg ---

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

    // --- KdlNode::find_child / find_children ---

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

    // --- KdlNode::string_prop / bool_prop / int_prop ---

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

    // --- KdlNode::string_child_values ---

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

    // --- KdlEntry::ty ---

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
