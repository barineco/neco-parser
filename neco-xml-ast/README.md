# neco-xml-ast

[日本語](README-ja.md)

Structured access layer for XML values.

## Features

- Owned `XmlDocument` wrapping `XmlValue` for cross-crate trait use
- Borrowed `XmlNode<'a>` carrying an optional field key
- `parse` re-export that turns a string slice into `XmlDocument`
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

let doc = neco_xml_ast::parse("name = neco\n").unwrap();
let node = doc.nodes().remove(0);
assert_eq!(node.kind(), "name");
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<XmlDocument, ParseError>` | Parses input via `neco-xml` and wraps the value |
| `XmlDocument` | Owned wrapper around `XmlValue` |
| `XmlNode<'a>` | Borrowed view with an optional field key |
| `XmlDocument::from_value` / `as_value` | Constructs from or reads the inner XmlValue |
| `XmlNode::from_value` / `as_value` | Constructs a root view or reads the inner XmlValue |
| `impl StructuredDocument for XmlDocument` | `nodes()` yields mapping fields, or the root value |

`impl StructuredNode for XmlNode<'_>` implements:

- `kind`
- `identifier`
- `attribute`
- `children`
- `value`

## License

MIT
