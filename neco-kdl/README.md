# neco-kdl

[日本語](README-ja.md)

`neco-kdl` is a zero-dependency Rust crate for parsing, serializing, normalizing, and building KDL v2 documents.

## Features

- KDL v2 parsing
  - Multiline strings, raw strings, and esclines
  - Type annotations such as `(type)node`
  - Slashdash comments ( `/-` ), block comments, and nested comments
  - Keyword literals
    - Booleans: `#true` and `#false`
    - Null: `#null`
    - Infinities: `#inf` and `#-inf`
    - Not a number: `#nan`
  - Hexadecimal, octal, and binary literals with underscore separators
  - Version marker ( `/- kdl-version 2` )
- `parse_spanned`: a syntax tree with UTF-8 byte spans for nodes and entries
- `serialize` and `Display`: serialization from value types to KDL text
- `normalize`: normalized output matching the official test suite's `expected_kdl`
- `Value` conversion: conversion between `KdlDocument` and format-independent values
- Parsing of every successful official test vector, with document equality after serialization and reparsing

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

Parses a KDL v2 document and returns a `KdlDocument`.
The function removes spans from the result of `parse_spanned`, so both entry points accept the same inputs and return the same errors.

### `parse_spanned`

```rust
pub fn parse_spanned(input: &str) -> Result<SpannedKdlDocument, KdlError>
```

Parses a KDL v2 document and attaches a source span to every node and entry. A span is a half-open range of UTF-8 byte offsets into the input. Boundaries follow these rules:

- A node span starts at the leading type annotation, or at the name when there is none.
- A node span ends immediately after the name or the last element (entry, child block, slashdash entry, or slashdash child block), excluding trailing node-space, comments, and the node terminator.
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

Converts a document back to KDL text. The following types also implement `Display`:

- `KdlDocument`
- `KdlNode`
- `KdlEntry`
- `KdlValue`

### `normalize`

```rust
pub fn normalize(doc: &KdlDocument) -> String
```

Converts a `KdlDocument` to its normalized string form. Normalization rules:

- Strips comments
- Sorts properties by key in alphabetical order
- Deduplicates properties (last occurrence wins)
- Outputs strings as quoted strings or valid bare identifiers
- Indents with 4 spaces
- Removes empty child blocks (`node {}` becomes `node`)
- Converts numbers to decimal, strips underscores
- Removes the sign from integer negative zero (`-0` becomes `0`), while keeping -0.0 for floats
- Formats exponents with uppercase E and an explicit sign (`1.5e10` becomes `1.5E+10`)
- Strips leading zeros from the integer part and keeps trailing zeros in the fractional part (007 becomes 7, `00.50` becomes `0.50`)
- Adds a trailing newline

### `value_to_kdl_document` / `kdl_document_to_value`

```rust
pub fn value_to_kdl_document(value: &Value) -> Result<KdlDocument, ValueError>
pub fn kdl_document_to_value(doc: &KdlDocument) -> Result<Value, ValueError>
```

Converts between `KdlDocument` and the format-independent `Value` type, which can carry values between KDL and formats such as JSON or CBOR.

Mapping rules between KDL nodes and `Value`:

- A node with no arguments and no child block maps to `Null`
- `Value::Null` becomes a node with a single `#null` argument
- A node with one argument maps to that value, and a node with two or more arguments maps to `Array`
- A node with only a child block maps to `Object`
- A nested empty `Value::Object` becomes a node with an empty child block
- `Integer` becomes an untyped number
- `Float` becomes a number annotated with `(f64)`
- An untyped number decodes to `Integer` when it fits in i64, or `Float` otherwise

Conversion failure returns `ValueError`, whose cause is classified into seven `ValueErrorReason` variants:

- `TopLevelMustBeObject`: an array or primitive supplied at the top level
- `NestedCollectionInArray`: an array or object used as an array element
- `EmptyArrayCannotBeConverted`: an empty array
- `PropertyCannotBeConverted`: a property in a KDL node
- `MixedArgumentsAndChildren`: arguments and a child block in one KDL node
- `UnsupportedTypeAnnotation(String)`: a node type annotation or an unsupported argument annotation-value pair; `(f64)` is supported for numbers
- `NumberCannotBeConverted(String)`: failure of both integer and floating-point interpretation

### Document builders

`KdlDocument::builder()` and `KdlNode::builder(name)` construct documents and nodes through method chaining.

```rust
use neco_kdl::{KdlDocument, KdlNode};

let document = KdlDocument::builder()
    .node(KdlNode::builder("greet").string_arg("hello").build())
    .node(KdlNode::builder("enabled").bool_arg(true).build())
    .build();

assert_eq!(document.nodes().len(), 2);
```

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
| `KdlDocument` | Root of a parsed document |
| `KdlNode` | Node with a type, name, entries, and child nodes |
| `KdlEntry` | Positional or named entry |
| `KdlDocumentBuilder` | Builder that adds nodes to a document |
| `KdlNodeBuilder` | Builder that adds arguments, properties, child nodes, and a type annotation |
| `KdlSpan` | Half-open UTF-8 byte range |
| `SpannedKdlDocument` | Root of a document with source spans |
| `SpannedKdlNode` | Node paired with its source span |
| `SpannedKdlEntry` | Entry paired with its source span |
| `KdlValue` | String, number, Boolean, or null value |
| `KdlNumber` | Validated source spelling and numeric interpretations |
| `NumberError` | Mismatch between source spelling and supplied interpretations |
| `KdlError` | Parse error with a line, column, and kind |
| `KdlErrorKind` | Character, escape, string, and related error kinds |
| `Value` | Format-independent intermediate value |
| `ValueError` | Conversion error with a classified reason |
| `ValueErrorReason` | Seven reasons for conversion failure |

The main enum variants and accessors are:

```text
KdlEntry = Argument | Property
KdlValue = String(String) | Number(KdlNumber) | Bool(bool) | Null
Value = Null | Bool | Integer | Float | String | Array | Object
KdlNode accessors = ty() | name() | entries() | children()
KdlError accessors = line() | col() | kind()
KdlErrorKind examples = UnexpectedChar | InvalidEscape | UnclosedString
```

## License

MIT
