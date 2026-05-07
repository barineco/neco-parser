# neco-yml

[English](README.md)

`neco-yml` は実用的な YAML 文書を読み込みます。小さな値型、`parse` 関数、`ParseError` 型を提供します。

## 使い方

### パース

```rust
use neco_yml::{parse, YmlValue};

let value = parse("name: neco").unwrap();
assert!(matches!(value, YmlValue::Map(_)));
```

### フィールド参照

```rust
use neco_yml::{parse, YmlValue};

let value = parse("name: neco").unwrap();
let YmlValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, YmlValue::String(text) if text == "neco")
}));
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<YmlValue, ParseError>` | 実用的な YAML 文書の読み込み |
| `YmlValue` | スカラー、リスト、マップの値 |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定ファイル形の文書と、よく使われるスカラー、リスト、マップ形式を扱います。

## ライセンス

MIT
