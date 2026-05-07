# neco-yml

[日本語](README-ja.md)

`neco-yml` parses practical YAML documents into a small value enum.

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
| `parse(input: &str) -> Result<YmlValue, ParseError>` | Parses practical YAML documents |
| `YmlValue` | Represents scalar, list, and map values |
| `ParseError` | Reports byte position and message |

## Format support

The parser is built for configuration-shaped documents and common scalar, list, and map forms.

## License

MIT
