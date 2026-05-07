# neco-ast

[English](README.md)

`neco-ast` は `neco-parser` の各 crate が生成した構造データを、形式中立の trait と値型で読むための crate です。

## 使い方

```rust
use neco_ast::{StructuredField, StructuredValue};
use std::borrow::Cow;

let value = StructuredValue::Mapping(vec![StructuredField {
    key: Cow::Borrowed("name"),
    value: StructuredValue::String(Cow::Borrowed("neco")),
}]);

assert_eq!(value.as_mapping().unwrap()[0].key, "name");
```

## API

| 型 | 説明 |
|---|---|
| `StructuredValue<'a>` | 借用された構造データ値 |
| `StructuredNumber<'a>` | raw text と `f64` 解釈を持つ数値 |
| `StructuredField<'a>` | key と値を持つ mapping field |
| `StructuredNode<'a>` | 形式別の値に対する借用 node view |
| `StructuredDocument<'a>` | top-level node を列挙する借用 document view |

## ライセンス

MIT
