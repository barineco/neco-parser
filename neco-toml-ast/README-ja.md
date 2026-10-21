# neco-toml-ast

[English](README.md)

`neco-toml-ast` は TOML の値を共有 `neco-ast` trait で読むためのライブラリです。

## 機能

- `TomlValue` を保持する所有 `TomlDocument`
- 省略可能なフィールドキーを持つ借用 `TomlNode<'a>`
- 文字列を `TomlDocument` に包む `parse` 再エクスポート
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

let doc = neco_toml_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<TomlDocument, ParseError>` | `neco-toml` で解析して包む |
| `TomlDocument` | `TomlValue` を保持する所有ラッパー |
| `TomlNode<'a>` | 省略可能なフィールドキーを持つ借用ビュー |
| `TomlDocument::from_value` / `as_value` | 内部 TomlValue の生成と参照 |
| `TomlNode::from_value` / `as_value` | 根ビューの生成と内部 TomlValue の参照 |
| `impl StructuredDocument for TomlDocument` | マッピングフィールドを返す ( 非マッピングは根単体 ) |

`impl StructuredNode for TomlNode<'_>` が実装するアクセサ:

- `kind`
- `identifier`
- `attribute`
- `children`
- `value`

## ライセンス

MIT
