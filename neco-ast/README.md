# neco-ast

[日本語](README-ja.md)

zero dependency structured data access traits.

## Usage

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

| Type | Description |
|---|---|
| `StructuredValue<'a>` | Borrowed structured data value |
| `StructuredNumber<'a>` | Number value with raw text and parsed `f64` access |
| `StructuredField<'a>` | Mapping field with a key and value |
| `StructuredNode<'a>` | Borrowed node view over a format-specific value |
| `StructuredDocument<'a>` | Borrowed document view exposing top-level nodes |

## License

MIT
