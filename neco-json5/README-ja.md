# neco-json5

[English](README.md)

`neco-json5` は実用的な JSON5 文書を読み込みます。小さな値型、`parse` 関数、`ParseError` 型を提供します。

## 使い方

### パース

```rust
use neco_json5::{parse, Json5Value};

let value = parse("{name: 'neco'}").unwrap();
assert!(matches!(value, Json5Value::Map(_)));
```

### フィールド参照

```rust
use neco_json5::{parse, Json5Value};

let value = parse("{name: 'neco'}").unwrap();
let Json5Value::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, Json5Value::String(text) if text == "neco")
}));
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<Json5Value, ParseError>` | 実用的な JSON5 文書の読み込み |
| `Json5Value` | スカラー、リスト、マップの値 |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定ファイル形の文書と、よく使われるスカラー、リスト、マップ形式を扱います。

## ライセンス

MIT
