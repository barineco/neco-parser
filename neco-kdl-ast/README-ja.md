# neco-kdl-ast

[English](README.md)

本 crate は、KDL v2 文書に対して名前空間パス、参照、構造化命名、5 軸の同型変換を提供する。

本 crate のバージョン 0.3 は、`neco-kdl` のバージョン 0.5 に依存する。
AST 変換の前後で、空の子ノードのブロックをそのまま保存する。

本 crate には 2 つの動作層がある。

- 読み取り側: `KdlDocument` の上に、名前空間パス、型注釈、プロパティと子ノードの対称、kind キーワードを解釈するアクセサを、文書を変更せずに提供する
- 変換側: 利用者が宣言する `Convention` を受け取り、5 軸の包み同型に沿って文書を書き換える。各軸は往復可能である

## 使い方

### パースと読み取り

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

### 参照のパース

```rust
use neco_kdl_ast::CrossRef;

let cr = CrossRef::parse("app.bsky.actor.defs#profileViewDetailed").unwrap();
assert_eq!(cr.nsid().display(), "app.bsky.actor.defs");
assert_eq!(cr.fragment(), Some("profileViewDetailed"));
```

### 利用者の Convention による変換

```rust
use neco_kdl_ast::{Convention, Document, Marker};

let app_marker = Marker::Prefix(':');
let conv = Convention::new().with_marker(app_marker.clone());

let input = r#"(app)com.example.myapp { bindings {} }"#;
let doc = Document::parse(input).unwrap();
let (doc, _) = doc.expand_type_annotations(&app_marker, &conv);
let (doc, _) = doc.nest(&conv);
let (doc, _) = doc.collapse_type_annotations(&app_marker, &conv);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Marker 境界によって `:app` の直下の子は名前空間展開から保護され、逆ドメイン形式の識別子は往復で元に戻る ( 例: `com.example.myapp` )。

## API

### 中核型

| 型 | 説明 |
|------|------|
| `NsidPath` | ドット区切りの名前空間パス、FS パスとの双方向変換 |
| `CrossRef` | `<NsidPath>#<fragment>` 形式の参照、3 種類の構文形を吸収 |
| `CrossRefParseError` | 参照文字列の解析エラー種別 |
| `StructuredName` | 種類と識別子の対、下記参照 |
| `StructuredNode<'a>` | KdlNode の借用ビュー、種類 / 識別子 / 入れ子の深さを解釈 |
| `Document` | `KdlDocument` の所有ラッパー型、索引参照 / 解決 / 配置検査 / 変換を提供 |
| `Convention` | 利用者が宣言する予約 `Marker` のリスト + 軸ごとの正規形宣言 |
| `Marker` | 包みの修飾子として機能する予約識別子、下記 2 形式のいずれか |
| `AxisForm` | 軸 1・2・4・5 用の正規形宣言、下記 5 種のバリアント ( enum が取り得る値の種類 ) |
| `PropertyChildForm` | 軸 3 専用の正規形宣言。融合形の 3 種類に限られる構造的制約を持つ |

`StructuredName` は `{ kind: Option<NsidPath>, identifier: NsidPath }` である。
form X と form Y を統一して表現する。

`Marker` の 2 形式:

- `Kind(String)`
- `Prefix(char)`

`AxisForm` のバリアント:

- `Off`
- `Expand`
- `Collapse`
- `ExpandWithMarker(Marker)`
- `CollapseWithMarker(Marker)`

`PropertyChildForm` のバリアント:

- `Off`
- `Expand`
- `Collapse`

### 読み取りアクセサ

| メソッド | 軸 |
|--------|------|
| `node_name_as_nsid` | 名前空間 |
| `dot_chain_depth` / `dot_chain_kind` | 手続き |
| プロパティ・子ノード 系アクセサ、下記参照 | プロパティと子ノードの等価 |
| `type_annotation` | 型注釈 |
| kind 系アクセサ、下記参照 | kind キーワード |

プロパティ・子ノード 系アクセサ:

- `attribute_str`
- `attribute_bool`
- `attribute_int`

kind 系アクセサ:

- `structured_name`
- `structured_name_form_x`
- `structured_name_form_y`

### 変換 ( 10 個の変換メソッドと、それらをまとめて呼ぶ render_as )

各軸が展開と畳込みの対を持つ。全変換は `Convention` を取る。
`Marker` 境界を尊重し、登録された `Marker` の直下の子は保護され、孫以下は通常処理する。

| 軸 | 展開 ( `expand` ) | 畳込み ( `collapse` ) |
|------|--------|----------|
| 名前空間 | `Document::nest` | `Document::flatten` |
| 手続き | `Document::expand_dot_chain` | `Document::collapse_dot_chain` |
| プロパティと子ノード | `expand_properties` | `collapse_properties` |
| 型注釈 | `Document::expand_type_annotations` | `Document::collapse_type_annotations` |
| kind キーワード | `expand_kind_keyword` | `collapse_kind_keyword` |

`render_as` は Convention の軸ごとの正規形宣言を読み、軸 5 から 4、1、2、3 の順で変換を適用する ( marker 境界保護順 )。
既定の `Convention` は全軸が off であり、この場合 `render_as` は恒等変換になる。

### 配置検査

| 型 | 説明 |
|------|------|
| `LayoutMode` | 下記 3 種のいずれか |
| `LayoutViolation` と `LayoutViolationKind` | FS パスと NSID の整合性検査結果 |

`LayoutMode` の 3 種:

- `Strict1To1`
- `Bundle`
- `CratisDir`

### 共有構造参照

| 項目 | 説明 |
|------|------|
| `neco_ast::StructuredNode<'a>` | `StructuredNode<'a>` が実装する共有 trait |
| `neco_ast::StructuredDocument<'a>` | `Document` が実装する共有 trait |
| `StructuredFacade<'a>` | 既存 KDL 利用者向けに保持する局所 trait |

詳細仕様と動作例は [文書索引](docs/INDEX.md) にある。

## ライセンス

MIT
