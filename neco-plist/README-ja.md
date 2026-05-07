# neco-plist

[English](README.md)

`neco-plist` は外部依存ゼロの plist XML 部分集合パーサーです。

## 機能

- XML 形式の plist 辞書を順序付きマップへ変換
- 入れ子の XML 要素を再帰的に変換
- 空の XML 要素を空文字列へ変換
- `null` / `bool` / `number` / `string` のスカラー解析
- バイト位置とメッセージを持つ `ParseError`

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
| `parse(input: &str) -> Result<PlistValue, ParseError>` | 対応する plist XML 部分集合の読み込み |
| `PlistValue` | `Null`、 `Bool`、 `Number(f64)`、 `String`、 `List`、 順序付き `Map` |
| `ParseError` | バイト位置とメッセージ |

## 対応範囲

設定データとして使う軽量な XML `property-list` 形を扱います。 要素名をフィールドに写し、 フィールド順序を保持します。 バイナリ plist と Apple plist の意味論全体は、 この最小部分集合 crate では扱いません。

## ライセンス

MIT
