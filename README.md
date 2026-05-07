# neco-parser

[日本語](README-ja.md)

`neco-parser` is a collection of Rust crates for parsing structured text formats into typed representations.

Each parser crate focuses on one text format and exposes a small Rust API for parsing and value inspection. The `*-ast` crates adapt those values to shared structured access traits from `neco-ast`.

The crates in this repository use no external Rust crate dependencies.

## Crates

| crate | description | built on |
|---|---|---|
| [`neco-ast`](./neco-ast) | Shared structured access traits and value types | none |
| [`neco-kdl`](./neco-kdl) | KDL v2 parser, serializer, and document builder | none |
| [`neco-json`](./neco-json) | JSON parser, encoder, and typed field access helpers | none |
| [`neco-kdl-ast`](./neco-kdl-ast) | Structured AST layer for KDL v2 documents | `neco-kdl`, `neco-ast` |
| [`neco-json-ast`](./neco-json-ast) | Structured access for JSON values | `neco-json`, `neco-ast` |
| [`neco-json5`](./neco-json5) | JSON5 subset parser | none |
| [`neco-json5-ast`](./neco-json5-ast) | Structured access for JSON5 values | `neco-json5`, `neco-ast` |
| [`neco-plist`](./neco-plist) | plist XML subset parser | none |
| [`neco-plist-ast`](./neco-plist-ast) | Structured access for plist values | `neco-plist`, `neco-ast` |
| [`neco-toml`](./neco-toml) | TOML subset parser | none |
| [`neco-toml-ast`](./neco-toml-ast) | Structured access for TOML values | `neco-toml`, `neco-ast` |
| [`neco-xml`](./neco-xml) | XML subset parser | none |
| [`neco-xml-ast`](./neco-xml-ast) | Structured access for XML values | `neco-xml`, `neco-ast` |
| [`neco-yml`](./neco-yml) | YAML subset parser | none |
| [`neco-yml-ast`](./neco-yml-ast) | Structured access for YAML values | `neco-yml`, `neco-ast` |

## License

MIT
