# neco-parser

[English](README.md)

`neco-parser` は構造化テキスト形式 (KDL, JSON) を型付き表現にパースする Rust crate 群。

`barineco/neco-crates` から切り出された parser 専用 repository で、 各 crate は独立した release cycle を持つ。 外部依存はゼロまたは最小限を志向し、 parser の性質が許す範囲で `no_std` に対応する。

## Crates

| crate | 概要 | 内部依存 | 主な外部依存 |
|---|---|---|---|
| [`neco-kdl`](./neco-kdl) | KDL v2 のパーサー・シリアライザ・document builder | なし | なし |
| [`neco-json`](./neco-json) | `no_std` 環境向けの最小 JSON コーデック | なし | なし |
| [`neco-kdl-ast`](./neco-kdl-ast) | KDL v2 文書の structured ast layer ( namespace path / cross-reference / structured naming / dot-prefix 入れ子 ) | `neco-kdl` | なし |

## ライセンス

MIT
