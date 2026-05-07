# neco-xml

[日本語](README-ja.md)

`neco-xml` parses practical XML documents into a small value enum.

## Usage

### Parse

```rust
use neco_xml::{parse, XmlValue};

let value = parse("<root><name>neco</name></root>").unwrap();
assert!(matches!(value, XmlValue::Map(_)));
```

### Read fields

```rust
use neco_xml::{parse, XmlValue};

let value = parse("<root><name>neco</name></root>").unwrap();
let XmlValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, XmlValue::String(text) if text == "neco")
}));
```

## API

| Item | Description |
|---|---|
| `parse(input: &str) -> Result<XmlValue, ParseError>` | Parses practical XML documents |
| `XmlValue` | Represents scalar, list, and map values |
| `ParseError` | Reports byte position and message |

## Format support

The parser is built for configuration-shaped documents and common scalar, list, and map forms.

## License

MIT
