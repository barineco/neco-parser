# neco-json5

[日本語](README-ja.md)

`neco-json5` parses practical JSON5 documents into a small value enum.

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
| `parse(input: &str) -> Result<Json5Value, ParseError>` | Parses practical JSON5 documents |
| `Json5Value` | Represents scalar, list, and map values |
| `ParseError` | Reports byte position and message |

## Format support

The parser is built for configuration-shaped documents and common scalar, list, and map forms.

## License

MIT
