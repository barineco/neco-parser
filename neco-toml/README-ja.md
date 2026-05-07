# neco-toml

[English](README.md)

`neco-toml` は実用的な TOML 文書を読み込みます。小さな値型、`parse` 関数、`ParseError` 型を提供します。

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
| `parse(input: &str) -> Result<TomlValue, ParseError>` | 実用的な TOML 文書の読み込み |
| `TomlValue` | スカラー、リスト、マップの値 |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定ファイル形の文書と、よく使われるスカラー、リスト、マップ形式を扱います。

## ライセンス

MIT
