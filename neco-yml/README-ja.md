# neco-yml

[English](README.md)

`neco-yml` は外部依存ゼロの YAML 部分集合パーサーです。

## 機能

- 行指向の `key: value` 解析
- 空値の後続 `- item` 行をリストへ変換
- `null` / `bool` / `number` / `string` のスカラー解析
- フィールド順序を `Vec<(String, YmlValue)>` として保持
- 行指向の位置とメッセージを持つ `ParseError`

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
| `parse(input: &str) -> Result<YmlValue, ParseError>` | 対応する YAML 部分集合の読み込み |
| `YmlValue` | `Null`、 `Bool`、 `Number(f64)`、 `String`、 `List`、 順序付き `Map` |
| `ParseError` | 行位置とメッセージ |

## 対応範囲

平坦な設定ファイル形の YAML とスカラー / 単純リストを扱います。 重複キーは出現順で保持し、 アンカー、 タグ、 マージキー、 YAML 文書ストリームは実装しません。

## ライセンス

MIT
