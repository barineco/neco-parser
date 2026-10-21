# neco-json

[日本語](README-ja.md)

neco-json is a minimal JSON codec with zero external dependencies that runs in `no_std` + `alloc` environments.
It provides parsing, encoding, and type-safe field access.
It also provides lightweight `ToJson` / `FromJson` conversion traits built on JsonValue.

neco-json provides two read paths whose memory allocation behavior differs.

- Owned value tree: the `JsonValue` returned by `parse`
- Borrowed reading: the `JsonRef` returned by `parse_borrowed`, borrowing the input

The owned tree represents numbers as `f64` and allocates arrays and strings through Rust's infallible allocation API. The application's allocator and the Rust runtime handle allocation failure.

Borrowed validation and most reads use the input and stack values. Decoding an escaped string with `as_str()` is the allocation-bearing operation; allocation failure returns `Err`.

The borrowed-reading section describes allocation by operation, and the string-decoding section gives the decoding details.

## Features

- `#![no_std]` with `alloc`
- `parse(&[u8]) -> Result<JsonValue, ParseError>`
- `encode(&JsonValue) -> Result<Vec<u8>, EncodeError>`
- `JsonValue` variants for null, bool, number, string, array, and object
- Typed accessors for required and optional object fields
- Lightweight `ToJson` / `FromJson` traits for common Rust values
- `parse_borrowed(&str) -> Result<JsonRef<'_>, ParseError>`: allocation-free validation that shares its grammar and failure positions with `parse` (containers may nest up to 128 levels), with iterators over arrays and objects
- `JsonString`: writes an escaped JSON string straight to a caller-owned `core::fmt::Write` sink without allocating

## Usage

### Parse

```rust
use neco_json::{parse, JsonValue};

let json = br#"{"name":"neco","score":42.5,"active":true}"#;
let value = parse(json).unwrap();
```

### Access fields

```rust
use neco_json::{parse, JsonValue};

let json = br#"{"name":"neco","score":42.5,"active":true,"tag":null}"#;
let value = parse(json).unwrap();

// required fields : error if missing or wrong type
let name   = value.required_str("name").unwrap();   // "neco"
let score  = value.required_f64("score").unwrap();  // 42.5
let active = value.required_bool("active").unwrap(); // true

// optional fields : Ok(None) when absent or null
let tag = value.optional_str("tag").unwrap(); // None
```

### Encode

```rust
extern crate alloc;
use neco_json::{encode, JsonValue};
use alloc::vec;

let value = JsonValue::Object(vec![
    ("x".into(), JsonValue::Number(1.0)),
    ("ok".into(), JsonValue::Bool(true)),
]);
let bytes = encode(&value).unwrap(); // b"{\"x\":1,\"ok\":true}"
```

### `ToJson` / `FromJson`

```rust
use neco_json::{FromJson, ToJson};

let json = vec![1_u64, 2, 3].to_json();
let restored = Vec::<u64>::from_json(&json).unwrap();
assert_eq!(restored, vec![1, 2, 3]);
```

### Borrow without allocating

`parse_borrowed` validates the input and returns a `JsonRef` that borrows it.
Reading the value kind, the raw text, the position, a bool, an integer, array and object iteration, and string comparison runs on the borrowed input and on values held on the stack, so none of them allocates.
`as_str()`, which returns a string as a `Cow`, allocates only when the string contains escapes.
The next example reads a duplicate object key and an array with a repeated element.
It uses plain `for` loops and a fixed-size stack array instead of `Vec::collect`, and it reads only strings without escapes, so the whole walk allocates nothing.

```rust
use neco_json::{parse_borrowed, JsonKind};

let json = parse_borrowed(r#"{"a":1,"a":2,"tags":["x","x"]}"#).unwrap();

let mut a_values = [0_u64; 2];
let mut a_count = 0;
for entry in json.object_entries().unwrap() {
    let (key, value) = entry.unwrap();
    assert_eq!(key.kind(), JsonKind::String); // object keys are always strings
    if key.string_eq("a").unwrap() {
        a_values[a_count] = value.as_u64().unwrap(); // exact decimal integer, not an f64 conversion
        a_count += 1;
    } else if key.string_eq("tags").unwrap() {
        let mut tag_count = 0;
        for item in value.array_items().unwrap() {
            assert_eq!(item.unwrap().as_str().unwrap().unwrap(), "x");
            tag_count += 1;
        }
        assert_eq!(tag_count, 2); // both "x" elements are kept, in input order
    }
}
assert_eq!(a_count, 2);
assert_eq!(a_values, [1, 2]); // the duplicate key "a" is kept, in input order
```

`as_u64()` reads a decimal integer literal at or below `u64::MAX`.
It returns `None` for a literal with a negative sign, a fraction, or an exponent, and for a value out of range.
For integers at or below `2^53`, this exact value agrees with the owned API's `f64` number.
Above that bound, some integers can no longer be represented exactly as `f64`, so the owned value may differ from the input.
`as_u64()` always keeps the input's exact value.

### Decode strings only when needed

`as_str()` decodes a string value.
For any other kind, it returns `Ok(None)`.
A string with no escapes borrows the input with no allocation.
A string with escapes performs one checked allocation, sized to the exact decoded UTF-8 byte count.
`string_eq()` compares a decoded string with a `&str` while decoding, without allocating either way.

```rust
use neco_json::{parse_borrowed, StringDecodeError};
use std::borrow::Cow;

let plain = parse_borrowed("\"neco\"").unwrap();
assert!(matches!(plain.as_str().unwrap(), Some(Cow::Borrowed("neco")))); // borrows the input

let escaped = parse_borrowed("\"ne\\u0063o\"").unwrap();
assert_eq!(escaped.string_eq("neco"), Ok(true)); // compares while decoding, no allocation
match escaped.as_str() {
    Ok(Some(value)) => assert_eq!(value, "neco"), // a checked allocation sized to the exact decoded length
    Err(StringDecodeError::AllocationFailed { required }) => {
        // the allocator rejected exactly `required` decoded UTF-8 bytes
        let _ = required;
    }
    other => panic!("{other:?}"),
}
```

### Write a JSON string without allocating

`JsonString` writes its quoted, escaped form to any `core::fmt::Write` sink the caller supplies.
`JsonString` itself performs no allocation.
In the example below, the allocation belongs to `String`'s own buffer growth, not to `JsonString`.
A caller that writes into a fixed-size stack buffer instead keeps the whole call allocation-free.

```rust
use core::fmt::Write;
use neco_json::JsonString;

let mut buffer = String::new(); // the caller owns this buffer; its growth is the caller's allocation
write!(&mut buffer, "{}", JsonString::new("cat\u{1f}")).unwrap();
assert_eq!(buffer, "\"cat\\u001f\""); // U+001F as lowercase \u001f

// `encode` writes the same U+001F as uppercase \u001F
use neco_json::{encode, JsonValue};
assert_eq!(
    encode(&JsonValue::String("cat\u{1f}".into())).unwrap(),
    b"\"cat\\u001F\""
);
```

## API

### Top-level functions

| Item | Description |
|------|-------------|
| `parse(input: &[u8]) -> Result<JsonValue, ParseError>` | Parse a byte slice as JSON |
| `parse_borrowed(input: &str) -> Result<JsonRef<'_>, ParseError>` | Validate JSON and return a borrowed, allocation-free view of `input`; containers may nest up to 128 levels |
| `encode(value: &JsonValue) -> Result<Vec<u8>, EncodeError>` | Encode a `JsonValue` to compact JSON bytes |
| `ToJson` / `FromJson` | Lightweight traits for converting Rust values to and from JsonValue |

### `JsonValue`

Represents any JSON value.

```
Null | Bool(bool) | Number(f64) | String(String) | Array(Vec<JsonValue>) | Object(Vec<(String, JsonValue)>)
```

#### Type checks

| Item | Description |
|------|-------------|
| `is_null()` | Returns true if `Null` |
| `is_bool()` | Returns true if `Bool` |
| `is_number()` | Returns true if `Number` |
| `is_string()` | Returns true if `String` |
| `is_array()` | Returns true if `Array` |
| `is_object()` | Returns true if `Object` |

#### Value extraction (`Option`)

| Item | Description |
|------|-------------|
| `as_bool() -> Option<bool>` | Extract bool |
| `as_f64() -> Option<f64>` | Extract number |
| `as_i64() -> Option<i64>` | Extract a number that is an integer representable as `i64` |
| `as_u64() -> Option<u64>` | Extract a number that is a non-negative integer representable as `u64` |
| `as_str() -> Option<&str>` | Extract string slice |
| `as_array() -> Option<&[JsonValue]>` | Extract array slice |
| `as_object() -> Option<&[(String, JsonValue)]>` | Extract object field slice |
| `get(key) -> Option<&JsonValue>` | Look up an object field by key |

#### Required field accessors

Return `Err(AccessError)` when the value is not an object, the field is missing, or the type differs.

| Item | Description |
|------|-------------|
| `required_str(key) -> Result<&str, AccessError>` | Required string field |
| `required_bool(key) -> Result<bool, AccessError>` | Required bool field |
| `required_f64(key) -> Result<f64, AccessError>` | Required number field |
| `required_array(key) -> Result<&[JsonValue], AccessError>` | Required array field |
| `required_object(key) -> Result<&[(String, JsonValue)], AccessError>` | Required object field |

#### Optional field accessors

Return `Ok(None)` when the field is absent or null; `Err(AccessError)` on type mismatch.

| Item | Description |
|------|-------------|
| `optional_str(key) -> Result<Option<&str>, AccessError>` | Optional string field |
| `optional_bool(key) -> Result<Option<bool>, AccessError>` | Optional bool field |
| `optional_f64(key) -> Result<Option<f64>, AccessError>` | Optional number field |
| `optional_array(key) -> Result<Option<&[JsonValue]>, AccessError>` | Optional array field |

### `JsonKind`

The syntactic kind returned by `JsonRef::kind()`.

```
Null | Bool | Number | String | Array | Object
```

### `JsonRef`

A validated JSON value that borrows the original input.
The only public constructor is `parse_borrowed`.
Child values come from `array_items()` or `object_entries()`.

| Item | Description |
|------|-------------|
| `kind() -> JsonKind` | Syntactic kind of this value |
| `raw() -> &str` | Exact input substring this value occupies (quotes included for strings) |
| `span() -> Range<usize>` | Half-open byte range of this value in the original input; `input[span()] == raw()` |
| `as_bool() -> Option<bool>` | Boolean value, or `None` for another kind |
| `as_u64() -> Option<u64>` | Exact decimal integer within u64, or `None` for a negative sign, a fraction, an exponent, or an overflow |
| `as_str() -> Result<Option<Cow<'_, str>>, StringDecodeError>` | Decoded string, or `Ok(None)` for any other kind; see the allocation contract in the usage example above |
| `string_eq(needle: &str) -> Result<bool, ParseError>` | Compares the decoded string with `needle` while decoding, without allocating |
| `array_items() -> Option<JsonArrayItems<'_>>` | Iterator over array elements, or `None` for another kind |
| `object_entries() -> Option<JsonObjectEntries<'_>>` | Iterator over object entries, or `None` for another kind |

### `JsonArrayItems` / `JsonObjectEntries`

Allocation-free iterators returned by `array_items()` or `object_entries()`.
Elements and entries are yielded in input order.
Duplicate object keys are kept, matching `JsonValue::Object`.
A grammar violation in one element yields `Err` once; every later call returns `None`.

| Item | Description |
|------|-------------|
| `JsonArrayItems<'a>: Iterator<Item = Result<JsonRef<'a>, ParseError>>` | One item per array element |
| `JsonObjectEntries<'a>: Iterator<Item = Result<(JsonRef<'a>, JsonRef<'a>), ParseError>>` | One (key, value) pair per object entry; the key is always `JsonKind::String` |

### `StringDecodeError`

Failure while decoding an escaped string through `as_str()`.

| Item | Description |
|------|-------------|
| `CapacityOverflow { input_len: usize }` | The decoded length would exceed the addressable string size |
| `AllocationFailed { required: usize }` | The allocator rejected the exact decoded UTF-8 byte count `required` |
| `Malformed(ParseError)` | Character decoding failed at an original input byte position |

### `JsonString`

Writes a borrowed string as a quoted JSON string to a caller-owned `core::fmt::Write` sink, without allocating on its own.

| Item | Description |
|------|-------------|
| `JsonString::new(value: &str) -> JsonString<'_>` | Wraps `value` for JSON string output |
| `Display::fmt` | Writes the quoted, escaped form; sink errors propagate as `core::fmt::Error` |

`JsonString` classifies each character into three groups.

```
short escape                            : " \ \b \f \n \r \t
remaining C0 controls (U+0000-U+001F)   : lowercase \u00xx
everything else, including / and U+007F : verbatim
```

This classification is shared with the `encode` function above.
The two differ in the letter case of the hexadecimal digits written in the `\u00XX` form.
`encode` uses uppercase digits, and `JsonString` uses lowercase digits.
The lowercase form follows ECMAScript 2024's `JSON.stringify`.
Specification: https://tc39.es/ecma262/2024/multipage/structured-data.html#sec-unicodeescape

### Error types

| Item | Description |
|------|-------------|
| `ParseError` | Parse failure with byte `position` and kind |
| `ParseErrorKind` | Specific parse failure reason |
| `EncodeError` | Encode failure : currently only `NonFiniteNumber` |
| `AccessError` | Field access failure: NotAnObject, MissingField, TypeMismatch |
| `StringDecodeError` | `as_str()` decoding failure; variants are listed above |

## License

MIT
