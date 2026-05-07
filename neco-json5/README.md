# neco-json5

[日本語](README-ja.md)

zero dependency JSON5 subset parser.

## Features

- Object parsing with unquoted, single-quoted, or double-quoted keys
- Scalar parsing for null, bool, number, and string values
- Array parsing with comma-separated scalar values
- Object fields stored in source order as `Vec<(String, Json5Value)>`
- `ParseError` with byte-oriented position and message

## Usage

### Parse

```rust
use neco_json5::{parse, Json5Value};

let value = parse("{name: 'neco'}").unwrap();
assert!(matches!(value, Json5Value::Map(_)));
```

### Read fields

```rust
use neco_json5::{parse, Json5Value};

let value = parse("{name: 'neco'}").unwrap();
let Json5Value::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, Json5Value::String(text) if text == "neco")
}));
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<Json5Value, ParseError>` | Parses the supported JSON5 subset |
| `Json5Value` | `Null`, `Bool`, `Number(f64)`, `String`, `List`, or ordered `Map` |
| `ParseError` | Reports byte position and message |

## Format support

The supported subset covers configuration-shaped JSON5 object documents, common scalar values, and scalar arrays. It keeps duplicate object keys in order and does not implement the full JSON5 grammar.

## License

MIT
