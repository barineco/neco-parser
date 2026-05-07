# neco-toml

[English](README.md)

`neco-toml` は外部依存ゼロの TOML 部分集合パーサーです。

## 機能

- 行指向の `key = value` 解析
- 空値の後続 `- item` 行をリストへ変換
- `null` / `bool` / `number` / `string` のスカラー解析
- フィールド順序を `Vec<(String, TomlValue)>` として保持
- 行指向の位置とメッセージを持つ `ParseError`

## 使い方

### パース

```rust
use neco_toml::{parse, TomlValue};

let value = parse("name = neco").unwrap();
assert!(matches!(value, TomlValue::Map(_)));
```

### フィールド参照

```rust
use neco_toml::{parse, TomlValue};

let value = parse("name = neco").unwrap();
let TomlValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, TomlValue::String(text) if text == "neco")
}));
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<TomlValue, ParseError>` | 対応する TOML 部分集合の読み込み |
| `TomlValue` | `Null`、 `Bool`、 `Number(f64)`、 `String`、 `List`、 順序付き `Map` |
| `ParseError` | 行位置とメッセージ |

## 対応範囲

平坦な設定ファイル形の TOML とスカラー / 単純配列を扱います。重複キーは出現順で保持し、 TOML v1.0 文法全体は実装しません。

## ライセンス

MIT
