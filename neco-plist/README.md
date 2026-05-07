# neco-plist

[日本語](README-ja.md)

`neco-plist` parses practical plist documents into a small value enum.

## Usage

### Parse

```rust
use neco_plist::{parse, PlistValue};

let value = parse("<dict><name>neco</name></dict>").unwrap();
assert!(matches!(value, PlistValue::Map(_)));
```

### Read fields

```rust
use neco_plist::{parse, PlistValue};

let value = parse("<dict><name>neco</name></dict>").unwrap();
let PlistValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, PlistValue::String(text) if text == "neco")
}));
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<PlistValue, ParseError>` | Parses practical plist documents |
| `PlistValue` | Represents scalar, list, and map values |
| `ParseError` | Reports byte position and message |

## Format support

The parser is built for configuration-shaped documents and common scalar, list, and map forms.

## License

MIT
