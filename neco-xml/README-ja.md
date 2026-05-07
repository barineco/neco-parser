# neco-xml

[English](README.md)

`neco-xml` は外部依存ゼロの XML 部分集合パーサーです。

## 機能

- 要素名を順序付きマップのフィールドへ変換
- 入れ子要素を再帰的に変換
- テキスト内容を `null` / `bool` / `number` / `string` 値として解析
- 空要素を空文字列へ変換
- バイト位置とメッセージを持つ `ParseError`

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
| `parse(input: &str) -> Result<XmlValue, ParseError>` | 対応する XML 部分集合の読み込み |
| `XmlValue` | `Null`、 `Bool`、 `Number(f64)`、 `String`、 `List`、 順序付き `Map` |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定データとして使うコンパクトな XML 要素木を扱います。 要素名をフィールドに写し、 順序を保持します。 名前空間、 DTD、 実体展開、 スキーマ検証は実装しません。

## ライセンス

MIT
