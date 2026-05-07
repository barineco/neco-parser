# neco-toml

[日本語](README-ja.md)

zero dependency TOML subset parser.

## Features

- Line-oriented `key = value` parsing
- Empty values followed by `- item` lines converted to lists
- Scalar parsing for null, bool, number, and string values
- Field order preserved as `Vec<(String, TomlValue)>`
- `ParseError` with line-oriented position and message

## Usage

### Parse

```rust
use neco_toml::{parse, TomlValue};

let value = parse("name = neco").unwrap();
assert!(matches!(value, TomlValue::Map(_)));
```

### Read fields

```rust
use neco_toml::{parse, TomlValue};

let value = parse("name = neco").unwrap();
let TomlValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, TomlValue::String(text) if text == "neco")
}));
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<TomlValue, ParseError>` | Parses the supported TOML subset |
| `TomlValue` | `Null`, `Bool`, `Number(f64)`, `String`, `List`, or ordered `Map` |
| `ParseError` | Reports line position and message |

## Format support

The supported subset covers flat configuration-shaped TOML with scalar values and simple arrays. It preserves duplicate keys in source order and does not implement the full TOML v1.0 grammar.

## License

MIT
