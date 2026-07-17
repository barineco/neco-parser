# Changelog

## 0.4.0

- 数値型の構造体フィールドを非公開化し、原文と解釈値を検証する構築関数を追加。
- 値の変換における失敗を 7 種の理由で弁別。
- 名前付き引数と未対応の型注釈を明示的に拒否し、空の連想配列を保存して空の配列を明示的に拒否。
- 型注釈内の行継続を KDL v2 のノード間空白として受理。
- 公式検査例の改変検知、負ゼロ、文字列のエスケープに対する回帰検査を追加。

## 0.3.0

- KDL v2 parser API を `builder`, `normalize`, `convert` まで含む公開面へ整理。
- `KdlDocumentBuilder` / `KdlNodeBuilder` による文書構築 API を追加。
- `KdlValue` と `Value` の相互変換、公式 test suite、README / metadata を公開前水準へ更新。

## 0.2.0

- `serialize` 関数と `Display` impl を追加 (parse → serialize の roundtrip が可能に)
- `KdlDocument`, `KdlNode`, `KdlNumber` の構造体フィールドを `pub` に変更
- `Cargo.toml` の description を更新

## 0.1.1

- `Value` enum と `value_to_kdl_document` / `kdl_document_to_value` 変換 API を追加
- `KdlNode` に `get`, `get_value` アクセサを追加
- `KdlValue` に `as_str`, `as_i64`, `as_f64`, `as_bool` アクセサを追加

## 0.1.0

- KDL v2 仕様の完全パース
- `normalize` による正規化出力
- 公式テストスイート全件通過
