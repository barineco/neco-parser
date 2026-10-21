# neco-json5-ast

[English](README.md)

`neco-json5-ast` は JSON5 の値を共有 `neco-ast` trait で読むためのライブラリです。

## 機能

- `Json5Value` を保持する所有 `Json5Document`
- 省略可能なフィールドキーを持つ借用 `Json5Node<'a>`
- 文字列を `Json5Document` に包む `parse` 再エクスポート
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

let doc = neco_json5_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<Json5Document, ParseError>` | `neco-json5` で解析して包む |
| `Json5Document` | `Json5Value` を保持する所有ラッパー |
| `Json5Node<'a>` | 省略可能なフィールドキーを持つ借用ビュー |
| `Json5Document::from_value` / `as_value` | 内部 Json5Value の生成と参照 |
| `Json5Node::from_value` / `as_value` | 根ビューの生成と内部 Json5Value の参照 |
| `impl StructuredDocument for Json5Document` | マッピングフィールドを返す ( 非マッピングは根単体 ) |

`impl StructuredNode for Json5Node<'_>` が実装するアクセサ:

- `kind`
- `identifier`
- `attribute`
- `children`
- `value`

## ライセンス

MIT
