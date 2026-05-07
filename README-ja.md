# neco-parser

[English](README.md)

`neco-parser` は構造化テキスト形式を型付き表現へ読み込む Rust crate 群。

各パーサー crate は 1 つのテキスト形式を扱い、読み込みと値の参照のための小さな Rust API を提供します。

このリポジトリの crate は、外部 Rust crate に依存しません。

## crate 一覧

| crate | 概要 | 内部依存 |
|---|---|---|
| [`neco-kdl`](./neco-kdl) | KDL v2 のパーサー、シリアライザ、文書ビルダー | なし |
| [`neco-json`](./neco-json) | JSON のパース、エンコード、型付きフィールド参照 | なし |
| [`neco-kdl-ast`](./neco-kdl-ast) | KDL v2 文書向け構造参照ヘルパー | `neco-kdl` |
| [`neco-json5`](./neco-json5) | JSON5 パーサー | なし |
| [`neco-plist`](./neco-plist) | plist XML パーサー | なし |
| [`neco-toml`](./neco-toml) | TOML パーサー | なし |
| [`neco-xml`](./neco-xml) | XML パーサー | なし |
| [`neco-yml`](./neco-yml) | YAML パーサー | なし |

## ライセンス

MIT
