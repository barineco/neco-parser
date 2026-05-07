# neco-plist-ast

[日本語](README-ja.md)

necosystems series structured access layer for plist values.

## Features

- Owned `PlistDocument` wrapping `PlistValue` for cross-crate trait use
- Borrowed `PlistNode<'a>` carrying an optional field key
- `parse` re-export that turns `&str` into `PlistDocument`
- `StructuredDocument` impl exposing mapping fields as top-level nodes
- `StructuredNode` impl providing `kind`, `identifier`, `attribute`, `children`, and `value`

## Usage

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_plist_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<PlistDocument, ParseError>` | Parses input via `neco-plist` and wraps the value |
| `PlistDocument` | Owned wrapper around `PlistValue` |
| `PlistNode<'a>` | Borrowed view with an optional field key |
| `PlistDocument::from_value` / `as_value` | Constructs from or reads the inner `PlistValue` |
| `PlistNode::from_value` / `as_value` | Constructs a root view or reads the inner `PlistValue` |
| `impl StructuredDocument for PlistDocument` | `nodes()` yields mapping fields, or the root value |
| `impl StructuredNode for PlistNode<'_>` | Implements `kind`, `identifier`, `attribute`, `children`, and `value` |

## License

MIT
