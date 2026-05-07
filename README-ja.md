# neco-parser

[English](README.md)

`neco-parser` は構造化テキスト形式を型付き表現へ読み込む Rust crate 群。

各パーサー crate は 1 つのテキスト形式を扱い、読み込みと値の参照のための小さな Rust API を提供します。`*-ast` crate は、それらの値を `neco-ast` の共有構造参照 trait へ変換します。

このリポジトリの crate は、外部 Rust crate に依存しません。

## crate 一覧

| crate | 概要 | 内部依存 |
|---|---|---|
| [`neco-ast`](./neco-ast) | 共有構造参照 trait と値型 | なし |
| [`neco-kdl`](./neco-kdl) | KDL v2 のパーサー、シリアライザ、文書ビルダー | なし |
| [`neco-json`](./neco-json) | JSON のパース、エンコード、型付きフィールド参照 | なし |
| [`neco-kdl-ast`](./neco-kdl-ast) | KDL v2 文書向け構造化 AST 層 | `neco-kdl`, `neco-ast` |
| [`neco-json-ast`](./neco-json-ast) | JSON 値の構造参照 | `neco-json`, `neco-ast` |
| [`neco-json5`](./neco-json5) | JSON5 subset パーサー | なし |
| [`neco-json5-ast`](./neco-json5-ast) | JSON5 値の構造参照 | `neco-json5`, `neco-ast` |
| [`neco-plist`](./neco-plist) | plist XML subset パーサー | なし |
| [`neco-plist-ast`](./neco-plist-ast) | plist 値の構造参照 | `neco-plist`, `neco-ast` |
| [`neco-toml`](./neco-toml) | TOML subset パーサー | なし |
| [`neco-toml-ast`](./neco-toml-ast) | TOML 値の構造参照 | `neco-toml`, `neco-ast` |
| [`neco-xml`](./neco-xml) | XML subset パーサー | なし |
| [`neco-xml-ast`](./neco-xml-ast) | XML 値の構造参照 | `neco-xml`, `neco-ast` |
| [`neco-yml`](./neco-yml) | YAML subset パーサー | なし |
| [`neco-yml-ast`](./neco-yml-ast) | YAML 値の構造参照 | `neco-yml`, `neco-ast` |

## ライセンス

MIT
