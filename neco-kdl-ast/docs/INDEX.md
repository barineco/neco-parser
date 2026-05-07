# neco-kdl-ast ドキュメント索引

`neco-kdl` parser の上に載る構造化 AST layer。 名前空間の path、 参照、 構造化命名、 手続きの入れ子、 property / type annotation / kind keyword を含む 5 軸の同型変換を提供し、KDL 固有 helper と共有 `neco-ast` trait 実装を同時に持つ。

## ドキュメント

| ドキュメント | 概要 |
|---|---|
| [architecture/overview](architecture/overview.md) | 全体構成、 parser 層と AST 層の責務、 主要型一覧 |
| [reference/syntax](reference/syntax.md) | 構造化命名 / 参照 / namespace の入れ子 / 手続きの入れ子 の 構文と worked example |

## やりたいこと別

| 目的 | 参照先 |
|---|---|
| 全体像を知りたい | [architecture/overview](architecture/overview.md) |
| 構文と例を確認したい | [reference/syntax](reference/syntax.md) |
| `.` ( namespace 区切り ) と leading `.` ( 手続き深度 ) の違いを知りたい | [reference/syntax § dot 区切りの 2 軸](reference/syntax.md) |
| property / type annotation / kind keyword 等の同型表記を知りたい | [reference/syntax § 包みの観点による 5 軸の同型表記](reference/syntax.md) |
| `neco-ast` の共有構造参照 trait との関係を知りたい | [architecture/overview § 共有構造参照](architecture/overview.md) |
| FS path と 名前空間パス の対応を知りたい | [reference/syntax § FS path との対応](reference/syntax.md) |
| AT Protocol Lexicon を KDL で表現する例を見たい | [reference/syntax § form Y と type annotation](reference/syntax.md) |
