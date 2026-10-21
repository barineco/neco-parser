# neco-yml-ast

[English](README.md)

`neco-yml-ast` は YAML の値を共有 `neco-ast` trait で読むためのライブラリです。

## 機能

- `YmlValue` を保持する所有 `YmlDocument`
- 省略可能なフィールドキーを持つ借用 `YmlNode<'a>`
- 文字列を `YmlDocument` に包む `parse` 再エクスポート
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

let doc = neco_yml_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<YmlDocument, ParseError>` | `neco-yml` で解析して包む |
| `YmlDocument` | `YmlValue` を保持する所有ラッパー |
| `YmlNode<'a>` | 省略可能なフィールドキーを持つ借用ビュー |
| `YmlDocument::from_value` / `as_value` | 内部 YmlValue の生成と参照 |
| `YmlNode::from_value` / `as_value` | 根ビューの生成と内部 YmlValue の参照 |
| `impl StructuredDocument for YmlDocument` | マッピングフィールドを返す ( 非マッピングは根単体 ) |

`impl StructuredNode for YmlNode<'_>` が実装するアクセサ:

- `kind`
- `identifier`
- `attribute`
- `children`
- `value`

## ライセンス

MIT
