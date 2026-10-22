# neco-parser

[日本語](README-ja.md)

`neco-parser` is a group of Rust crates for parsing JSON and KDL v2. Each crate has no external Rust crate dependencies, exposes typed values, and writes those values back to text.

## Crates

| Crate | Description |
|---|---|
| [neco-kdl](./neco-kdl) | KDL v2 parser, serializer, normalizer, and document builder |
| [neco-json](./neco-json) | JSON parser, encoder, and owned and borrowed value access |

## Related libraries

- [kdleam](https://github.com/barineco/kdleam): KDL v2 library for Gleam
- [neco-lean](https://github.com/barineco/neco-lean): Tool library for Lean 4 (includes JSON and KDL modules)

## License

MIT
