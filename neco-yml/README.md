# neco-yml

[日本語](README-ja.md)

zero dependency YAML subset parser.

## Features

- Line-oriented `key: value` parsing
- Empty values followed by `- item` lines converted to lists
- Scalar parsing for null, bool, number, and string values
- Field order preserved as `Vec<(String, YmlValue)>`
- `ParseError` with line-oriented position and message

## Usage

### Parse

```rust
use neco_yml::{parse, YmlValue};

let value = parse("name: neco").unwrap();
assert!(matches!(value, YmlValue::Map(_)));
```

### Read fields

```rust
use neco_yml::{parse, YmlValue};

let value = parse("name: neco").unwrap();
let YmlValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, YmlValue::String(text) if text == "neco")
}));
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<YmlValue, ParseError>` | Parses the supported YAML subset |
| `YmlValue` | `Null`, `Bool`, `Number(f64)`, `String`, `List`, or ordered `Map` |
| `ParseError` | Reports line position and message |

## Format support

The supported subset covers flat configuration-shaped YAML with scalar values and simple lists. It preserves duplicate keys in source order and does not implement anchors, tags, merge keys, or full YAML document streams.

## License

MIT
