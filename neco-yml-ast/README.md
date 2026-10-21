# neco-yml-ast

[日本語](README-ja.md)

Structured access layer for YAML values.

## Features

- Owned `YmlDocument` wrapping `YmlValue` for cross-crate trait use
- Borrowed `YmlNode<'a>` carrying an optional field key
- `parse` re-export that turns a string slice into `YmlDocument`
- `StructuredDocument` impl exposing mapping fields as top-level nodes
- `StructuredNode` impl providing these accessor methods:
  - `kind`
  - `identifier`
  - `attribute`
  - `children`
  - `value`

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
| `YmlDocument::from_value` / `as_value` | Constructs from or reads the inner YmlValue |
| `YmlNode::from_value` / `as_value` | Constructs a root view or reads the inner YmlValue |
| `impl StructuredDocument for YmlDocument` | `nodes()` yields mapping fields, or the root value |

`impl StructuredNode for YmlNode<'_>` implements:

- `kind`
- `identifier`
- `attribute`
- `children`
- `value`

## License

MIT
