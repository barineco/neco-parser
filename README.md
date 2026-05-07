# neco-parser

[日本語](README-ja.md)

`neco-parser` is a collection of Rust crates for parsing structured text formats into typed representations.

Each parser crate focuses on one text format and exposes a small Rust API for parsing and value inspection.

The crates in this repository use no external Rust crate dependencies.

## Crates

| crate | description | built on |
|---|---|---|
| [`neco-kdl`](./neco-kdl) | KDL v2 parser, serializer, and document builder | none |
| [`neco-json`](./neco-json) | JSON parser, encoder, and typed field access helpers | none |
| [`neco-kdl-ast`](./neco-kdl-ast) | Structured traversal helpers for KDL v2 documents | `neco-kdl` |
| [`neco-json5`](./neco-json5) | JSON5 parser | none |
| [`neco-plist`](./neco-plist) | plist XML parser | none |
| [`neco-toml`](./neco-toml) | TOML parser | none |
| [`neco-xml`](./neco-xml) | XML parser | none |
| [`neco-yml`](./neco-yml) | YAML parser | none |

## License

MIT
