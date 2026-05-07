# neco-json-ast

[English](README.md)

`neco-json-ast` は JSON の値を共有 `neco-ast` trait で読むための crate です。

## 機能

- `JsonValue` を保持する所有 `JsonDocument`
- 任意のフィールドキーを持つ借用 `JsonNode<'a>`
- `&[u8]` を `JsonDocument` に包む `parse` 再エクスポート
- 上位のオブジェクトフィールドをノード列として返す `StructuredDocument` 実装
- 値型に対する `kind` / `identifier` / `attribute` / `children` / `value` を提供する `StructuredNode` 実装

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
| `parse(input: &[u8]) -> Result<JsonDocument, ParseError>` | `neco-json` で解析し `JsonDocument` で包む |
| `JsonDocument` | `JsonValue` を保持する所有ラッパー |
| `JsonNode<'a>` | 任意のフィールドキーを持つ借用ビュー |
| `JsonDocument::from_value` / `as_value` | 内部 `JsonValue` の生成と参照 |
| `JsonNode::from_value` / `as_value` | 根ビューの生成と内部 `JsonValue` の参照 |
| `impl StructuredDocument for JsonDocument` | `nodes()` がオブジェクトフィールドを返す ( 非オブジェクトは根単体 ) |
| `impl StructuredNode for JsonNode<'_>` | `kind` / `identifier` / `attribute` / `children` / `value` を実装 |

## ライセンス

MIT
