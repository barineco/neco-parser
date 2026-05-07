# neco-yml-ast

[日本語](README-ja.md)

necosystems series structured access layer for YAML values.

## Features

- Owned `YmlDocument` wrapping `YmlValue` for cross-crate trait use
- Borrowed `YmlNode<'a>` carrying an optional field key
- `parse` re-export that turns `&str` into `YmlDocument`
- `StructuredDocument` impl exposing mapping fields as top-level nodes
- `StructuredNode` impl providing `kind`, `identifier`, `attribute`, `children`, and `value`

## Usage

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_yml_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<YmlDocument, ParseError>` | Parses input via `neco-yml` and wraps the value |
| `YmlDocument` | Owned wrapper around `YmlValue` |
| `YmlNode<'a>` | Borrowed view with an optional field key |
| `YmlDocument::from_value` / `as_value` | Constructs from or reads the inner `YmlValue` |
| `YmlNode::from_value` / `as_value` | Constructs a root view or reads the inner `YmlValue` |
| `impl StructuredDocument for YmlDocument` | `nodes()` yields mapping fields, or the root value |
| `impl StructuredNode for YmlNode<'_>` | Implements `kind`, `identifier`, `attribute`, `children`, and `value` |

## License

MIT
