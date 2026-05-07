# neco-json-ast

[日本語](README-ja.md)

necosystems series structured access layer for JSON values.

## Features

- Owned `JsonDocument` wrapping `JsonValue` for cross-crate trait use
- Borrowed `JsonNode<'a>` carrying an optional field key
- `parse` re-export that turns `&[u8]` into `JsonDocument`
- `StructuredDocument` impl exposing object fields as top-level nodes
- `StructuredNode` impl providing `kind`, `identifier`, `attribute`, `children`, and `value`

## Usage

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_json_ast::parse(br#"{"name":"neco"}"#).unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| Item | Description |
|---|---|
| `parse(input: &[u8]) -> Result<JsonDocument, ParseError>` | Parses input via `neco-json` and wraps the value |
| `JsonDocument` | Owned wrapper around `JsonValue` |
| `JsonNode<'a>` | Borrowed view with an optional field key |
| `JsonDocument::from_value` / `as_value` | Constructs from or reads the inner `JsonValue` |
| `JsonNode::from_value` / `as_value` | Constructs a root view or reads the inner `JsonValue` |
| `impl StructuredDocument for JsonDocument` | `nodes()` yields object fields, or the root value |
| `impl StructuredNode for JsonNode<'_>` | Implements `kind`, `identifier`, `attribute`, `children`, and `value` |

## License

MIT
