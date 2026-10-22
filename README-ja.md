# neco-parser

[English](README.md)

neco-parser は、JSON と KDL v2 の構文解析を扱う依存最小の Rust クレート群です。
解析した値の参照と、値からテキストへの出力に対応しています。

## クレート

| クレート | 概要 |
|---|---|
| [neco-kdl](./neco-kdl) | KDL v2 の解析、直列化、正規化、および文書構築 |
| [neco-json](./neco-json) | JSON の解析、直列化、および所有型と借用型による値の参照 |

## 関連ライブラリ

- [kdleam](https://github.com/barineco/kdleam): Gleam 対応の KDL v2 ライブラリ
- [neco-lean](https://github.com/barineco/neco-lean): Lean 4 のツールライブラリ ( JSON, KDL のモジュールを含む )

## ライセンス

MIT
