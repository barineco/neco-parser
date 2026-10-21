# neco-json-ast

[English](README.md)

`neco-json-ast` は JSON の値を共有 `neco-ast` trait で読むためのライブラリです。

## 機能

- `JsonValue` を保持する所有 `JsonDocument`
- 省略可能なフィールドキーを持つ借用 `JsonNode<'a>`
- バイト列を `JsonDocument` に包む `parse` 再エクスポート
- 上位のオブジェクトフィールドをノード列として返す `StructuredDocument` 実装
- `StructuredNode` 実装が提供するアクセサ:
  - `kind`
  - `identifier`
  - `attribute`
  - `children`
  - `value`

## 使い方

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_json_ast::parse(br#"{"name":"neco"}"#).unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &[u8]) -> Result<JsonDocument, ParseError>` | `neco-json` で解析して包む |
| `JsonDocument` | `JsonValue` を保持する所有ラッパー |
| `JsonNode<'a>` | 省略可能なフィールドキーを持つ借用ビュー |
| `JsonDocument::from_value` / `as_value` | 内部 JsonValue の生成と参照 |
| `JsonNode::from_value` / `as_value` | 根ビューの生成と内部 JsonValue の参照 |
| `impl StructuredDocument for JsonDocument` | オブジェクトフィールドを返す ( 非オブジェクトは根単体 ) |

`impl StructuredNode for JsonNode<'_>` が実装するアクセサ:

- `kind`
- `identifier`
- `attribute`
- `children`
- `value`

## ライセンス

MIT
