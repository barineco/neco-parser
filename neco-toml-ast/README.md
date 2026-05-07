# neco-toml-ast

[日本語](README-ja.md)

necosystems series structured access layer for TOML values.

## Features

- Owned `TomlDocument` wrapping `TomlValue` for cross-crate trait use
- Borrowed `TomlNode<'a>` carrying an optional field key
- `parse` re-export that turns `&str` into `TomlDocument`
- `StructuredDocument` impl exposing mapping fields as top-level nodes
- `StructuredNode` impl providing `kind`, `identifier`, `attribute`, `children`, and `value`

## Usage

```rust
use neco_ast::{StructuredDocument, StructuredNode};

let doc = neco_toml_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<TomlDocument, ParseError>` | Parses input via `neco-toml` and wraps the value |
| `TomlDocument` | Owned wrapper around `TomlValue` |
| `TomlNode<'a>` | Borrowed view with an optional field key |
| `TomlDocument::from_value` / `as_value` | Constructs from or reads the inner `TomlValue` |
| `TomlNode::from_value` / `as_value` | Constructs a root view or reads the inner `TomlValue` |
| `impl StructuredDocument for TomlDocument` | `nodes()` yields mapping fields, or the root value |
| `impl StructuredNode for TomlNode<'_>` | Implements `kind`, `identifier`, `attribute`, `children`, and `value` |

## License

MIT
