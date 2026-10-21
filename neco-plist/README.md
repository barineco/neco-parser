# neco-plist

[日本語](README-ja.md)

zero dependency plist XML subset parser.

## Features

- XML-shaped plist dictionaries converted to ordered maps
- Nested XML elements converted recursively
- Empty XML elements converted to empty strings
- Scalar parsing for null, bool, number, and string values
- `ParseError` with byte-oriented position and message

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
| `parse(input: &str) -> Result<PlistValue, ParseError>` | Parses the supported plist XML subset |
| `PlistValue` | Parsed value, one of the variants below |
| `ParseError` | Reports byte position and message |

`PlistValue` variants:

- `Null`
- `Bool`
- `Number(f64)`
- `String`
- `List`
- `Map` (ordered)

## Format support

The supported subset covers lightweight XML property-list shapes used as configuration data. It maps element names to fields and preserves field order, independent of binary plist and full Apple plist semantics.

## License

MIT
