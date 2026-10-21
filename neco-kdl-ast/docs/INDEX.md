# neco-kdl-ast ドキュメント索引

[English](INDEX-en.md)

`neco-kdl` parser の上に構造化 AST 層を載せる crate である。

次の機能を提供する。

- 名前空間パス
- 参照
- 構造化命名
- 手続きの入れ子
- property・型注釈・kind キーワードを含む 5 軸の同型変換

KDL 固有のアクセサ群と、共有 `neco-ast` trait の実装も併せ持つ。

## ドキュメント

| ドキュメント | 概要 |
|---|---|
| [architecture/overview](architecture/overview.md) | 全体構成、構文解析層と AST 層の責務、主要型一覧 |
| [reference/syntax](reference/syntax.md) | 構造化命名・参照・namespace の入れ子・手続きの入れ子の構文と具体例 |

## やりたいこと別

| 目的 | 参照先 |
|---|---|
| 全体像を知りたい | [architecture/overview](architecture/overview.md) |
| 構文と例を確認したい | [reference/syntax](reference/syntax.md) |
| `.` ( namespace 区切り ) と leading `.` ( 手続き深度 ) の違いを知りたい | [reference/syntax § dot 区切りの 2 軸](reference/syntax.md) |
| 同型表記 ( property・型注釈・kind キーワード等 ) を知りたい | [reference/syntax § 包みの観点による 5 軸の同型表記](reference/syntax.md) |
| `neco-ast` の共有構造参照 trait との関係を知りたい | [architecture/overview § 共有構造参照](architecture/overview.md) |
| FS パス と 名前空間パス の対応を知りたい | [reference/syntax § FS パス との対応](reference/syntax.md) |
| AT Protocol の Lexicon を KDL で表現する例を見たい | [reference/syntax § form Y・型注釈](reference/syntax.md) |
| `render_as` で軸ごとの正規形を一括で正規化したい | [reference/syntax § Convention の正規形宣言](reference/syntax.md) |
| `Convention` の 軸 OFF・融合形の構造的制約を知りたい | [reference/syntax § Convention の正規形宣言](reference/syntax.md) |
