# neco-json5-ast

[日本語](README-ja.md)

necosystems series structured access layer for JSON5 values.

## Features

- Owned `Json5Document` wrapping `Json5Value` for cross-crate trait use
- Borrowed `Json5Node<'a>` carrying an optional field key
- `parse` re-export that turns `&str` into `Json5Document`
- `StructuredDocument` impl exposing mapping fields as top-level nodes
- `StructuredNode` impl providing `kind`, `identifier`, `attribute`, `children`, and `value`

## Usage

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_json5_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<Json5Document, ParseError>` | Parses input via `neco-json5` and wraps the value |
| `Json5Document` | Owned wrapper around `Json5Value` |
| `Json5Node<'a>` | Borrowed view with an optional field key |
| `Json5Document::from_value` / `as_value` | Constructs from or reads the inner `Json5Value` |
| `Json5Node::from_value` / `as_value` | Constructs a root view or reads the inner `Json5Value` |
| `impl StructuredDocument for Json5Document` | `nodes()` yields mapping fields, or the root value |
| `impl StructuredNode for Json5Node<'_>` | Implements `kind`, `identifier`, `attribute`, `children`, and `value` |

## License

MIT
