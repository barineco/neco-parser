# neco-json5

[English](README.md)

`neco-json5` は外部依存ゼロの JSON5 部分集合パーサーです。

## 機能

- 引用なし / 単引用 / 二重引用のキーを持つオブジェクト解析
- `null` / `bool` / `number` / `string` のスカラー解析
- カンマ区切りのスカラー値による配列解析
- オブジェクトフィールドを `Vec<(String, Json5Value)>` として出現順で保持
- バイト位置とメッセージを持つ `ParseError`

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
| `parse(input: &str) -> Result<Json5Value, ParseError>` | 対応する JSON5 部分集合の読み込み |
| `Json5Value` | `Null`、 `Bool`、 `Number(f64)`、 `String`、 `List`、 順序付き `Map` |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定ファイル形の JSON5 オブジェクト文書、 よく使われるスカラー値、 スカラー配列を扱います。 重複するオブジェクトキーは出現順で保持し、 JSON5 文法全体は実装しません。

## ライセンス

MIT
