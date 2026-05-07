# neco-toml

[日本語](README-ja.md)

`neco-toml` parses practical TOML documents into a small value enum.

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
| `parse(input: &str) -> Result<TomlValue, ParseError>` | Parses practical TOML documents |
| `TomlValue` | Represents scalar, list, and map values |
| `ParseError` | Reports byte position and message |

## Format support

The parser is built for configuration-shaped documents and common scalar, list, and map forms.

## License

MIT
