# neco-xml-ast

[English](README.md)

`neco-xml-ast` は XML の値を共有 `neco-ast` trait で読むためのライブラリです。

## 機能

- `XmlValue` を保持する所有 `XmlDocument`
- 省略可能なフィールドキーを持つ借用 `XmlNode<'a>`
- 文字列を `XmlDocument` に包む `parse` 再エクスポート
- 上位のマッピングフィールドをノード列として返す `StructuredDocument` 実装
- `StructuredNode` 実装が提供するアクセサ:
  - `kind`
  - `identifier`
  - `attribute`
  - `children`
  - `value`

## 使い方

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_xml_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<XmlDocument, ParseError>` | `neco-xml` で解析して包む |
| `XmlDocument` | `XmlValue` を保持する所有ラッパー |
| `XmlNode<'a>` | 省略可能なフィールドキーを持つ借用ビュー |
| `XmlDocument::from_value` / `as_value` | 内部 XmlValue の生成と参照 |
| `XmlNode::from_value` / `as_value` | 根ビューの生成と内部 XmlValue の参照 |
| `impl StructuredDocument for XmlDocument` | マッピングフィールドを返す ( 非マッピングは根単体 ) |

`impl StructuredNode for XmlNode<'_>` が実装するアクセサ:

- `kind`
- `identifier`
- `attribute`
- `children`
- `value`

## ライセンス

MIT
