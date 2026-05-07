# neco-xml

[English](README.md)

`neco-xml` は実用的な XML 文書を読み込みます。小さな値型、`parse` 関数、`ParseError` 型を提供します。

## 使い方

### パース

```rust
use neco_xml::{parse, XmlValue};

let value = parse("<root><name>neco</name></root>").unwrap();
assert!(matches!(value, XmlValue::Map(_)));
```

### フィールド参照

```rust
use neco_xml::{parse, XmlValue};

let value = parse("<root><name>neco</name></root>").unwrap();
let XmlValue::Map(fields) = value else { panic!("map") };
assert!(fields.iter().any(|(key, value)| {
    key == "name" && matches!(value, XmlValue::String(text) if text == "neco")
}));
```

## API

| 項目 | 説明 |
|---|---|
| `parse(input: &str) -> Result<XmlValue, ParseError>` | 実用的な XML 文書の読み込み |
| `XmlValue` | スカラー、リスト、マップの値 |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定ファイル形の文書と、よく使われるスカラー、リスト、マップ形式を扱います。

## ライセンス

MIT
