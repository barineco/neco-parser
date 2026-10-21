# neco-xml

[日本語](README-ja.md)

zero dependency XML subset parser.

## Features

- Element names converted to ordered map fields
- Nested elements converted recursively
- Text content parsed as null, bool, number, or string values
- Empty elements converted to empty strings
- `ParseError` with byte-oriented position and message

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
| `parse(input: &str) -> Result<XmlValue, ParseError>` | Parses the supported XML subset |
| `XmlValue` | Parsed value, one of the variants below |
| `ParseError` | Reports byte position and message |

`XmlValue` variants:

- `Null`
- `Bool`
- `Number(f64)`
- `String`
- `List`
- `Map` (ordered)

## Format support

The supported subset covers compact XML element trees used as configuration data. It maps element names to fields and preserves order, independent of namespace declarations, DTD, entity expansion, and schema validation.

## License

MIT
