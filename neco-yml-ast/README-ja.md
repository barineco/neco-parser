# neco-yml-ast

[English](README.md)

`neco-yml-ast` は YAML の値を共有 `neco-ast` trait で読むための crate です。

## 機能

- `YmlValue` を保持する所有 `YmlDocument`
- 任意のフィールドキーを持つ借用 `YmlNode<'a>`
- `&str` を `YmlDocument` に包む `parse` 再エクスポート
- 上位のマッピングフィールドをノード列として返す `StructuredDocument` 実装
- 値型に対する `kind` / `identifier` / `attribute` / `children` / `value` を提供する `StructuredNode` 実装

## 使い方

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_yml_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<YmlDocument, ParseError>` | `neco-yml` で解析し `YmlDocument` で包む |
| `YmlDocument` | `YmlValue` を保持する所有ラッパー |
| `YmlNode<'a>` | 任意のフィールドキーを持つ借用ビュー |
| `YmlDocument::from_value` / `as_value` | 内部 `YmlValue` の生成と参照 |
| `YmlNode::from_value` / `as_value` | 根ビューの生成と内部 `YmlValue` の参照 |
| `impl StructuredDocument for YmlDocument` | `nodes()` がマッピングフィールドを返す ( 非マッピングは根単体 ) |
| `impl StructuredNode for YmlNode<'_>` | `kind` / `identifier` / `attribute` / `children` / `value` を実装 |

## ライセンス

MIT
