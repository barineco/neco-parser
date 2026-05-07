# neco-plist-ast

[English](README.md)

`neco-plist-ast` は plist の値を共有 `neco-ast` trait で読むための crate です。

## 機能

- `PlistValue` を保持する所有 `PlistDocument`
- 任意のフィールドキーを持つ借用 `PlistNode<'a>`
- `&str` を `PlistDocument` に包む `parse` 再エクスポート
- 上位のマッピングフィールドをノード列として返す `StructuredDocument` 実装
- 値型に対する `kind` / `identifier` / `attribute` / `children` / `value` を提供する `StructuredNode` 実装

## 使い方

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_plist_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<PlistDocument, ParseError>` | `neco-plist` で解析し `PlistDocument` で包む |
| `PlistDocument` | `PlistValue` を保持する所有ラッパー |
| `PlistNode<'a>` | 任意のフィールドキーを持つ借用ビュー |
| `PlistDocument::from_value` / `as_value` | 内部 `PlistValue` の生成と参照 |
| `PlistNode::from_value` / `as_value` | 根ビューの生成と内部 `PlistValue` の参照 |
| `impl StructuredDocument for PlistDocument` | `nodes()` がマッピングフィールドを返す ( 非マッピングは根単体 ) |
| `impl StructuredNode for PlistNode<'_>` | `kind` / `identifier` / `attribute` / `children` / `value` を実装 |

## ライセンス

MIT
