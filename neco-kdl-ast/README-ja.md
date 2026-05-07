# neco-kdl-ast

[English](README.md)

KDL v2 文書のための `necosystems series` 構造化 AST 層。 `neco-kdl` パーサーの上に名前空間パス、 参照、 構造化命名、 5 軸の同型変換を載せ、 共有 `neco-ast` 構造参照 trait も実装する。

本 crate には 2 つの動作層がある。 読み取り側は `KdlDocument` の上に名前空間パス、 型注釈、 `property` と `child` の対称、 `form X` / `form Y` の `kind` キーワードを解釈するアクセサを文書を変更せずに提供する。 変換側は利用者が宣言する `Convention` ( 予約語の `Marker` と接頭辞文字 ) を受け取り、 5 軸の包み同型に沿って文書を書き換える。 各軸は往復可能。

## 使い方

### パース、 構文解析

```rust
use neco_kdl_ast::Document;

let input = r#"
    cratis "encoding" version=1 {
        provides {
            axiom "encoding.base64.encode"
            axiom "encoding.base64.decode"
        }
    }
"#;
let doc = Document::parse(input).unwrap();

for node in doc.structured_nodes() {
    println!("{}", node.kind());
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 参照の解決

```rust
use neco_kdl_ast::CrossRef;

let cr = CrossRef::parse("app.bsky.actor.defs#profileViewDetailed").unwrap();
assert_eq!(cr.nsid().display(), "app.bsky.actor.defs");
assert_eq!(cr.fragment(), Some("profileViewDetailed"));
```

### 利用者の設定による同値構造変換

```rust
use neco_kdl_ast::{Convention, Document, Marker};

let app_marker = Marker::Prefix(':');
let conv = Convention::new().with_marker(app_marker.clone());

let input = r#"(app)com.vscodium.codium-insiders { bindings {} }"#;
let doc = Document::parse(input).unwrap();
let (doc, _) = doc.expand_type_annotations(&app_marker, &conv);
let (doc, _) = doc.nest(&conv);
let (doc, _) = doc.collapse_type_annotations(&app_marker, &conv);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Marker` 境界によって `:app` の直下の子は名前空間展開から保護される。 逆ドメイン形式の識別子 `com.vscodium.codium-insiders` は往復で元に戻る。

## API

### 中核型

| 型 | 説明 |
|------|------|
| `NsidPath` | ドット区切りの名前空間パス、 FS パスとの双方向変換 |
| `CrossRef` | `<NsidPath>#<fragment>` 形式の参照、 3 種類の構文形を吸収 |
| `CrossRefParseError` | 参照文字列の解析エラー種別 |
| `StructuredName` | `{ kind: Option<NsidPath>, identifier: NsidPath }`、 `form X` と `form Y` を統一表現 |
| `StructuredNode<'a>` | `KdlNode` の借用ビュー、 `kind` / `identifier` / 入れ子の深さを解釈 |
| `Document` | `KdlDocument` の所有ラッパー、 索引参照 / 解決 / 配置検査 / 変換を提供 |
| `Convention` | 利用者が宣言する予約 `Marker` のリスト、 変換が参照する |
| `Marker` | `Kind(String)` または `Prefix(char)` の予約識別子、 包みの修飾子として機能 |

### 読み取りアクセサ

| メソッド | 軸 |
|--------|------|
| `node_name_as_nsid` | 名前空間 |
| `dot_chain_depth` / `dot_chain_kind` | 手続き |
| `attribute_str` / `attribute_bool` / `attribute_int` | `property` と `child` の等価 |
| `type_annotation` | 型注釈 |
| `structured_name` / `structured_name_form_x` / `structured_name_form_y` | `kind` キーワード |

### 変換 ( 10 メソッド )

各軸が展開と畳込みの対を持つ。 全変換は `&Convention` を取り、 `Marker` 境界を尊重する。 登録された `Marker` の直下の子は保護され、 孫以下は通常処理。

| 軸 | 展開 ( `expand` ) | 畳込み ( `collapse` ) |
|------|--------|----------|
| 名前空間 | `Document::nest` | `Document::flatten` |
| 手続き | `Document::expand_dot_chain` | `Document::collapse_dot_chain` |
| `property` と `child` | `Document::expand_properties` | `Document::collapse_properties` |
| 型注釈 | `Document::expand_type_annotations` | `Document::collapse_type_annotations` |
| `kind` キーワード | `Document::expand_kind_keyword` | `Document::collapse_kind_keyword` |

### 配置検査

| 型 | 説明 |
|------|------|
| `LayoutMode` | `Strict1To1` / `Bundle` / `CratisDir` |
| `LayoutViolation` と `LayoutViolationKind` | FS パスと NSID の整合性検査結果 |

### 共有構造参照

| 項目 | 説明 |
|------|------|
| `neco_ast::StructuredNode<'a>` | `StructuredNode<'a>` が実装する共有 trait |
| `neco_ast::StructuredDocument<'a>` | `Document` が実装する共有 trait |
| `StructuredFacade<'a>` | 既存 KDL 利用者向けに保持する局所 trait |

詳細仕様と動作例は [文書索引](docs/INDEX.md) にある。

## ライセンス

MIT
