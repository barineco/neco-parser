# neco-plist

[English](README.md)

`neco-plist` は実用的な plist 文書を読み込みます。小さな値型、`parse` 関数、`ParseError` 型を提供します。

## 使い方

### パース

```rust
use neco_plist::{parse, PlistValue};

let value = parse("<dict><name>neco</name></dict>").unwrap();
assert!(matches!(value, PlistValue::Map(_)));
```

### フィールド参照

```rust
use neco_plist::{parse, PlistValue};

let value = parse("<dict><name>neco</name></dict>").unwrap();
let PlistValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, PlistValue::String(text) if text == "neco")
}));
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<PlistValue, ParseError>` | 実用的な plist 文書の読み込み |
| `PlistValue` | スカラー、リスト、マップの値 |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定ファイル形の文書と、よく使われるスカラー、リスト、マップ形式を扱います。

## ライセンス

MIT
