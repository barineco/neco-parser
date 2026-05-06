# neco-kdl-ast ドキュメント索引

`neco-kdl` parser の上に載る構造化 ast layer。 名前空間の path、 参照、 構造化命名、 連続した手続きを表す入れ子記法の 4 抽象を提供し、 KDL を構造化 IR の入れ物として扱う 利用者 共通の足場として機能する。

## ドキュメント

| ドキュメント | 概要 |
|---|---|
| [architecture/overview](architecture/overview.md) | 全体構成、 parser 層と ast 層の責務、 主要型一覧 |
| [reference/syntax](reference/syntax.md) | 構造化命名 / 参照 / namespace の入れ子 / 手続きの入れ子 の 構文と worked example |

## やりたいこと別

| 目的 | 参照先 |
|---|---|
| 全体像を知りたい | [architecture/overview](architecture/overview.md) |
| 構文と例を確認したい | [reference/syntax](reference/syntax.md) |
| `.` ( namespace 区切り ) と leading `.` ( 手続き深度 ) の違いを知りたい | [reference/syntax § dot 区切りの 2 軸](reference/syntax.md) |
| property / type annotation / kind keyword 等の同型表記を知りたい | [reference/syntax § 包みの観点による 5 軸の同型表記](reference/syntax.md) |
| FS path と 名前空間パス の対応を知りたい | [reference/syntax § FS path との対応](reference/syntax.md) |
| AT Protocol Lexicon を KDL で表現する例を見たい | [reference/syntax § form Y と type annotation](reference/syntax.md) |
