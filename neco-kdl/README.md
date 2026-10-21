# neco-kdl

[日本語](README-ja.md)

zero dependency KDL v2 parser, serializer, and document builder.

## Features

- Full KDL v2 specification parsing
  - Multiline strings, raw strings, escline
  - Type annotations: `(type)node`
  - Slashdash comments (`/-`), block comments (`/* ... */`), nested comments
  - Keyword literals: #true, #false, #null, #inf, #-inf, #nan
  - Hex, octal, binary literals with underscore separators
  - Version marker (`/- kdl-version 2`)
- Positional parse tree via `parse_spanned()` with a UTF-8 byte span per node and entry
- Serialization via `serialize()` and `Display` impl (roundtrip-safe)
- Normalized output (matches official test suite `expected_kdl`)
- Format-agnostic `Value` conversion, bridging to and from `KdlDocument`
- Zero external dependencies
- Passes the full official test suite

## Usage

```toml
[dependencies]
neco-kdl = "0.5"
```

```rust
use neco_kdl::{parse, serialize, normalize};

fn main() {
    let src = r#"
        node "hello" key=#true {
            child 42
        }
    "#;

    let doc = parse(src).unwrap();

    // Iterate over nodes
    for node in doc.nodes() {
        println!("{}: {} entries", node.name(), node.entries().len());
    }

    // Serialize back to KDL
    let output = serialize(&doc);
    print!("{}", output);

    // Convert to normalized form
    let normalized = normalize(&doc);
    print!("{}", normalized);
}
```

## API

### `parse`

```rust
pub fn parse(input: &str) -> Result<KdlDocument, KdlError>
```

Parses a KDL v2 document and returns a `KdlDocument`. The result is derived from `parse_spanned`, so both entry points accept the same inputs and return the same errors.

### `parse_spanned`

```rust
pub fn parse_spanned(input: &str) -> Result<SpannedKdlDocument, KdlError>
```

Parses a KDL v2 document and attaches a source span to every node and entry. A span is a half-open range of UTF-8 byte offsets into the input. Boundaries follow these rules:

- A node span starts at the leading type annotation, or at the name when there is none.
- A node span ends immediately after the name or the last adopted element (entry, child block, slashdash entry, slashdash child block), excluding trailing node-space, comments, and the node terminator.
- An entry span starts at the value type annotation or value for an argument, and at the key for a property. It ends at the value.
- Slashdash entries and child blocks are excluded from the output but included in the parent node span.

Accessors:

```rust
pub struct KdlSpan {
    pub start: usize,
    pub end: usize,
}

impl SpannedKdlDocument {
    pub fn nodes(&self) -> &[SpannedKdlNode];
    pub fn into_document(self) -> KdlDocument;
}

impl SpannedKdlNode {
    pub fn span(&self) -> KdlSpan;
    pub fn ty(&self) -> Option<&str>;
    pub fn name(&self) -> &str;
    pub fn entries(&self) -> &[SpannedKdlEntry];
    pub fn children(&self) -> Option<&[SpannedKdlNode]>;
    pub fn into_node(self) -> KdlNode;
}

impl SpannedKdlEntry {
    pub fn span(&self) -> KdlSpan;
    pub fn entry(&self) -> &KdlEntry;
}
```

`into_document` returns the same result as `parse`, without the spans.

### `line_col`

```rust
pub fn line_col(input: &str, offset: usize) -> Option<(usize, usize)>
```

Derives a 1-based line and column from a byte offset. Returns `None` for an out-of-range offset or an offset inside a multi-byte character. The newline set and CRLF folding match the parser, and `KdlError` derives its position from this function.

### `serialize`

```rust
pub fn serialize(doc: &KdlDocument) -> String
```

Converts a `KdlDocument` back to KDL text. All four node and value types implement `Display`.

### `normalize`

```rust
pub fn normalize(doc: &KdlDocument) -> String
```

Converts a `KdlDocument` to its normalized string form. Normalization rules:

- Strips comments
- Sorts properties by key in alphabetical order
- Deduplicates properties (last occurrence wins)
- Converts all strings to quoted strings
- Unquotes strings that are valid identifiers
- Indents with 4 spaces
- Converts numbers to decimal, strips underscores
- Adds trailing newline

### `value_to_kdl_document` / `kdl_document_to_value`

```rust
pub fn value_to_kdl_document(value: &Value) -> Result<KdlDocument, ValueError>
pub fn kdl_document_to_value(doc: &KdlDocument) -> Result<Value, ValueError>
```

Converts between `KdlDocument` and a format-agnostic `Value` enum, usable as an intermediate representation for bridging KDL with formats such as JSON or CBOR without external dependencies.

Conversion preserves an empty Object as an empty children block. Seven typed reasons describe unsupported or lossy inputs:

- an empty Array
- a nested collection inside an Array
- properties and unsupported type annotations
- mixed arguments and children
- a number without an interpreted value

### `KdlNumber`

The constructor validates KDL number syntax and the supplied interpretations.

```rust
impl KdlNumber {
    pub fn new(raw: impl Into<String>, as_i64: Option<i64>, as_f64: Option<f64>)
        -> Result<Self, NumberError>;
    pub fn raw(&self) -> &str;
    pub fn as_i64(&self) -> Option<i64>;
    pub fn as_f64(&self) -> Option<f64>;
}
```

### Types

| Item | Description |
|------|-------------|
| `KdlDocument` | Parse result root. Access nodes via `nodes()` |
| `KdlNode` | Node with ty(), name(), entries(), children() accessors |
| `KdlEntry` | Argument (positional) or Property (named) |
| `KdlSpan` | Half-open UTF-8 byte range with public offset fields |
| `SpannedKdlDocument` | Spanned document root, convertible to a plain document |
| `SpannedKdlNode` | Spanned node carrying a span and the plain node view |
| `SpannedKdlEntry` | Spanned entry carrying a span and the plain entry view |
| `KdlValue` | String(String), Number(KdlNumber), Bool(bool), Null |
| `KdlNumber` | Validated raw number with optional integer and floating-point interpretations |
| `NumberError` | Raw syntax and interpreted value mismatch |
| `KdlError` | Error with line(), col() (1-based), kind() |
| `KdlErrorKind` | Error variant: UnexpectedChar, InvalidEscape, UnclosedString, etc. |
| `Value` | Format-agnostic intermediate: Null, Bool, Integer, Float, String, Array, Object |
| `ValueError` | Value conversion error containing a `ValueErrorReason` |
| `ValueErrorReason` | Seven explicit rejection reasons for unsupported or lossy conversions |

## License

MIT
