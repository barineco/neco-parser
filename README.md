# neco-parser

[日本語](README-ja.md)

`neco-parser` is a collection of Rust crates for parsing structured text formats (KDL and JSON) into typed representations.

The repository was extracted from `barineco/neco-crates` so that parser crates can release independently. Member crates aim for zero or minimum external dependencies and keep `no_std`-friendly designs where the parser nature allows.

## Crates

| crate | description | internal deps | main external deps |
|---|---|---|---|
| [`neco-kdl`](./neco-kdl) | KDL v2 parser, serializer, and document builder | none | none |
| [`neco-json`](./neco-json) | minimal JSON codec for `no_std` environments | none | none |
| [`neco-kdl-ast`](./neco-kdl-ast) | structured ast layer over KDL v2 documents (namespace path, cross-reference, structured naming, dot-prefix nesting) | `neco-kdl` | none |

## License

MIT
