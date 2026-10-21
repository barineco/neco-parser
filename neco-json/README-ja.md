# neco-json

[English](README.md)

外部依存ゼロ、`no_std` + `alloc` 環境で動作する最小 JSON codec です。
パース、エンコード、型安全なフィールドアクセスを備えます。
JsonValue ベースの軽量な `ToJson` / `FromJson` 変換 trait も提供します。

neco-json の読み取り経路は 2 つあり、メモリ確保の扱いが経路ごとに異なります。

- 所有する値の木: `parse` が返す `JsonValue`
- 借用読み取り: `parse_borrowed` が返し、入力を借用する `JsonRef`

所有する値の木の数値表現は `f64` で、配列と文字列の領域は、失敗を呼び出し側へ返さない Rust の確保 API で用意されます。確保に失敗したときの処理の担い手は、アプリケーションのアロケータと Rust 処理系です。

借用読み取りでは、検証と大半の読み取りが入力の借用とスタック上の値だけで動き、確保が生じる場面はエスケープを含む文字列を `as_str()` でデコードするときだけです。失敗は `Err` の戻り値として返ります。

読み取りごとの確保の有無は借用読み取りの節を、デコードの詳細は文字列デコードの節を参照してください。

## 機能

- `#![no_std]` + `alloc` 動作
- `parse(&[u8]) -> Result<JsonValue, ParseError>`
- `encode(&JsonValue) -> Result<Vec<u8>, EncodeError>`
- `JsonValue` で扱う JSON の型 ( null・bool・number・string・array・object )
- 必須・省略可能なオブジェクトフィールドの型付きアクセサ
- Rust 値との相互変換 `ToJson` / `FromJson` trait
- `parse_borrowed(&str) -> Result<JsonRef<'_>, ParseError>`: `parse` と文法・失敗位置を共有するメモリ確保なしの検証 ( ネストは 128 段まで ) で、配列とオブジェクトの反復子を返す
- `JsonString`: 呼び出し側が持つ `core::fmt::Write` の書き込み先へ、引用符付きのエスケープ済み文字列をメモリ確保なしで書き出す

## 使い方

### パース

```rust
use neco_json::{parse, JsonValue};

let json = br#"{"name":"neco","score":42.5,"active":true}"#;
let value = parse(json).unwrap();
```

### フィールドアクセス

```rust
use neco_json::{parse, JsonValue};

let json = br#"{"name":"neco","score":42.5,"active":true,"tag":null}"#;
let value = parse(json).unwrap();

// 必須フィールド : 欠落・型不一致はエラー
let name   = value.required_str("name").unwrap();    // "neco"
let score  = value.required_f64("score").unwrap();   // 42.5
let active = value.required_bool("active").unwrap(); // true

// 省略可能フィールド : 欠落・null のとき Ok(None)
let tag = value.optional_str("tag").unwrap(); // None
```

### エンコード

```rust
extern crate alloc;
use neco_json::{encode, JsonValue};
use alloc::vec;

let value = JsonValue::Object(vec![
    ("x".into(), JsonValue::Number(1.0)),
    ("ok".into(), JsonValue::Bool(true)),
]);
let bytes = encode(&value).unwrap(); // b"{\"x\":1,\"ok\":true}"
```

### `ToJson` / `FromJson`

```rust
use neco_json::{FromJson, ToJson};

let json = vec![1_u64, 2, 3].to_json();
let restored = Vec::<u64>::from_json(&json).unwrap();
assert_eq!(restored, vec![1, 2, 3]);
```

### 借用してメモリ確保なしで読み取る

`parse_borrowed` は入力を検証し、その入力を借用する `JsonRef` を返します。
値の種類、生の文字列、位置、真偽値、整数、配列とオブジェクトの反復、文字列の比較は、入力の借用とスタック上の値だけで動き、メモリ確保は生じません。
文字列を `Cow` として取り出す `as_str()` は、エスケープを含む場合にだけ確保が生じます。
次の例は、重複するオブジェクトのキーと、同じ要素を 2 回持つ配列を読み取ります。
`Vec::collect` の代わりに通常の `for` 反復と固定長のスタック配列を使い、無エスケープの文字列だけを読むため、走査全体のメモリ確保がゼロになります。

```rust
use neco_json::{parse_borrowed, JsonKind};

let json = parse_borrowed(r#"{"a":1,"a":2,"tags":["x","x"]}"#).unwrap();

let mut a_values = [0_u64; 2];
let mut a_count = 0;
for entry in json.object_entries().unwrap() {
    let (key, value) = entry.unwrap();
    assert_eq!(key.kind(), JsonKind::String); // オブジェクトのキーは常に文字列
    if key.string_eq("a").unwrap() {
        a_values[a_count] = value.as_u64().unwrap(); // f64 変換を経ない厳密な 10 進整数値
        a_count += 1;
    } else if key.string_eq("tags").unwrap() {
        let mut tag_count = 0;
        for item in value.array_items().unwrap() {
            assert_eq!(item.unwrap().as_str().unwrap().unwrap(), "x");
            tag_count += 1;
        }
        assert_eq!(tag_count, 2); // 2 件の "x" が入力の出現順のまま残る
    }
}
assert_eq!(a_count, 2);
assert_eq!(a_values, [1, 2]); // 重複するキー "a" が入力の出現順のまま残る
```

`as_u64()` は、`u64::MAX` 以下の 10 進整数の表記を読み取ります。
負号・小数部・指数部を含む表記と、範囲を超える値では `None` を返します。
`2^53` 以下の整数では、この厳密な値は所有 API の `f64` の数値と一致します。
この範囲を超えると、一部の整数は `f64` で近似値になり、所有 API の値が入力からずれる場合があります。
`as_u64()` は常に入力どおりの厳密な値を保ちます。

### 必要な時だけ文字列をデコードする

`as_str()` は文字列値をデコードし、それ以外の種類では `Ok(None)` を返します。
無エスケープの文字列は入力を借用するだけで、メモリ確保は生じません。
エスケープを含む文字列では、デコード後の正確な UTF-8 バイト数に合わせて確保を 1 回だけ行います。
`string_eq()` は、デコードしながら `&str` と比較し、メモリ確保はゼロです。

```rust
use neco_json::{parse_borrowed, StringDecodeError};
use std::borrow::Cow;

let plain = parse_borrowed("\"neco\"").unwrap();
assert!(matches!(plain.as_str().unwrap(), Some(Cow::Borrowed("neco")))); // 入力を借用するだけ

let escaped = parse_borrowed("\"ne\\u0063o\"").unwrap();
assert_eq!(escaped.string_eq("neco"), Ok(true)); // デコードしながら比較する。メモリ確保なし
match escaped.as_str() {
    Ok(Some(value)) => assert_eq!(value, "neco"), // デコード後の正確な長さへ確保する
    Err(StringDecodeError::AllocationFailed { required }) => {
        // アロケータがちょうど `required` バイトの確保を拒否した
        let _ = required;
    }
    other => panic!("{other:?}"),
}
```

### メモリ確保なしで JSON 文字列を書き出す

`JsonString` は、呼び出し側が渡す `core::fmt::Write` 先へ、引用符付きのエスケープ済み文字列を書き込みます。
`JsonString` 自身の書き込みは、メモリ確保ゼロです。
次の例で生じる確保は、`String` のバッファ伸長に属します。
固定長のスタック上バッファを書き込み先にすれば、呼び出し全体のメモリ確保をゼロに保てます。

```rust
use core::fmt::Write;
use neco_json::JsonString;

let mut buffer = String::new(); // このバッファは呼び出し側が持ち、伸長の確保も呼び出し側に属する
write!(&mut buffer, "{}", JsonString::new("cat\u{1f}")).unwrap();
assert_eq!(buffer, "\"cat\\u001f\""); // U+001F は小文字の \u001f になる

// `encode` では、同じ U+001F が大文字の \u001F になる
use neco_json::{encode, JsonValue};
assert_eq!(
    encode(&JsonValue::String("cat\u{1f}".into())).unwrap(),
    b"\"cat\\u001F\""
);
```

## API

### トップレベル関数

| 項目 | 説明 |
|------|------|
| `parse(input: &[u8]) -> Result<JsonValue, ParseError>` | バイト列を JSON としてパースする |
| `parse_borrowed(input: &str) -> Result<JsonRef<'_>, ParseError>` | JSON を検証し、`input` を借用する、メモリ確保なしの借用値を返す。ネストは 128 段まで受け入れる |
| `encode(value: &JsonValue) -> Result<Vec<u8>, EncodeError>` | `JsonValue` を最小 JSON バイト列にエンコードする |
| `ToJson` / `FromJson` | Rust の値と相互変換する軽量 trait |

### `JsonValue`

JSON 値を表す enum。

```
Null | Bool(bool) | Number(f64) | String(String) | Array(Vec<JsonValue>) | Object(Vec<(String, JsonValue)>)
```

#### 型判定

| 項目 | 説明 |
|------|------|
| `is_null()` | `Null` のとき true |
| `is_bool()` | `Bool` のとき true |
| `is_number()` | `Number` のとき true |
| `is_string()` | `String` のとき true |
| `is_array()` | `Array` のとき true |
| `is_object()` | `Object` のとき true |

#### 値取得 (`Option`)

| 項目 | 説明 |
|------|------|
| `as_bool() -> Option<bool>` | bool を取り出す |
| `as_f64() -> Option<f64>` | 数値を取り出す |
| `as_i64() -> Option<i64>` | `i64` で表せる整数値を取り出す |
| `as_u64() -> Option<u64>` | `u64` で表せる 0 以上の整数値を取り出す |
| `as_str() -> Option<&str>` | 文字列スライスを取り出す |
| `as_array() -> Option<&[JsonValue]>` | 配列スライスを取り出す |
| `as_object() -> Option<&[(String, JsonValue)]>` | オブジェクトフィールドのスライスを取り出す |
| `get(key) -> Option<&JsonValue>` | キーでオブジェクトフィールドを検索する |

#### 必須フィールドアクセサ

オブジェクトでない・フィールド欠落・型不一致のとき `Err(AccessError)` を返す。

| 項目 | 説明 |
|------|------|
| `required_str(key) -> Result<&str, AccessError>` | 必須文字列フィールド |
| `required_bool(key) -> Result<bool, AccessError>` | 必須 bool フィールド |
| `required_f64(key) -> Result<f64, AccessError>` | 必須数値フィールド |
| `required_array(key) -> Result<&[JsonValue], AccessError>` | 必須配列フィールド |
| `required_object(key) -> Result<&[(String, JsonValue)], AccessError>` | 必須オブジェクトフィールド |

#### 省略可能フィールドアクセサ

フィールド欠落・null のとき `Ok(None)`、型不一致のとき `Err(AccessError)` を返す。

| 項目 | 説明 |
|------|------|
| `optional_str(key) -> Result<Option<&str>, AccessError>` | 省略可能文字列フィールド |
| `optional_bool(key) -> Result<Option<bool>, AccessError>` | 省略可能 bool フィールド |
| `optional_f64(key) -> Result<Option<f64>, AccessError>` | 省略可能数値フィールド |
| `optional_array(key) -> Result<Option<&[JsonValue]>, AccessError>` | 省略可能配列フィールド |

### `JsonKind`

`JsonRef::kind()` が返す構文上の種類です。

```
Null | Bool | Number | String | Array | Object
```

### `JsonRef`

元の入力を借用する、検証済みの JSON 値です。
公開構築子は `parse_borrowed` だけです。
子要素は `array_items()` または `object_entries()` から得ます。

| 項目 | 説明 |
|------|------|
| `kind() -> JsonKind` | この値の構文上の種類 |
| `raw() -> &str` | この値が占める入力の厳密な部分文字列 ( 文字列では引用符を含む ) |
| `span() -> Range<usize>` | 元の入力に対するバイトオフセットの半開区間。`input[span()] == raw()` が成立する |
| `as_bool() -> Option<bool>` | bool 値。他の種類では `None` |
| `as_u64() -> Option<u64>` | u64 の範囲に収まる厳密な 10 進整数値。負号・小数部・指数部・範囲超過では `None` |
| `as_str() -> Result<Option<Cow<'_, str>>, StringDecodeError>` | デコード済み文字列。文字列以外では `Ok(None)`。確保の契約は上記の使用例を参照 |
| `string_eq(needle: &str) -> Result<bool, ParseError>` | デコードしながら `needle` と比較する。メモリ確保なし |
| `array_items() -> Option<JsonArrayItems<'_>>` | 配列要素の反復子。他の種類では `None` |
| `object_entries() -> Option<JsonObjectEntries<'_>>` | オブジェクトエントリの反復子。他の種類では `None` |

### `JsonArrayItems` / `JsonObjectEntries`

`array_items()` または `object_entries()` が返す、メモリ確保なしの反復子です。
要素とエントリは入力の出現順で返され、重複するオブジェクトのキーは `JsonValue::Object` と同じ方針でそのまま残ります。
要素の文法違反は 1 回だけ `Err` を返し、以後の呼び出しでは `None` が返ります。

| 項目 | 説明 |
|------|------|
| `JsonArrayItems<'a>: Iterator<Item = Result<JsonRef<'a>, ParseError>>` | 配列要素ごとに 1 件 |
| `JsonObjectEntries<'a>: Iterator<Item = Result<(JsonRef<'a>, JsonRef<'a>), ParseError>>` | オブジェクトエントリごとに key と value の組 1 件。前者は常に `JsonKind::String` |

### `StringDecodeError`

`as_str()` がエスケープを含む文字列をデコードする際の失敗です。

| 項目 | 説明 |
|------|------|
| `CapacityOverflow { input_len: usize }` | デコード後の長さがアドレス可能な文字列サイズを超える |
| `AllocationFailed { required: usize }` | アロケータが、デコード後の正確な UTF-8 バイト数の確保を拒否した |
| `Malformed(ParseError)` | 元の入力のバイト位置で文字のデコードに失敗した |

### `JsonString`

呼び出し側が持つ `core::fmt::Write` 先へ、借用した文字列を引用符付き JSON 文字列として書き出します。
自身はメモリ確保ゼロです。
書き込み先の失敗は `core::fmt::Error` としてそのまま伝搬します。

| 項目 | 説明 |
|------|------|
| `JsonString::new(value: &str) -> JsonString<'_>` | 借用した文字列を JSON 文字列出力用に包む |
| `Display::fmt` | 引用符付きのエスケープ済み形式を書き込む |

`JsonString` は、各文字を 3 つの分類のいずれかへ振り分けます。

| 分類 | 対象の文字 | 出力形式 |
|---|---|---|
| 短いエスケープ | 引用符、逆斜線、バックスペース、改ページ、改行、復帰、タブの 7 文字 | 定義済みの 2 文字からなるエスケープ |
| 残りの制御文字 | 短いエスケープを除く、コードポイント U+0000 から U+001F までの文字 | 4 桁 16 進数の小文字エスケープ |
| 逐語出力 | スラッシュ、コードポイント U+007F、その他すべての文字 | 元の文字をそのまま出力 |

この分類は、上記の `encode` 関数と共通です。
両者の違いは、`\u00XX` の形で書く 16 進数の大小文字にあります。
`encode` は大文字を使い、`JsonString` は小文字を使います。
小文字の表記は、ECMAScript 2024 の `JSON.stringify` に従います。
仕様: https://tc39.es/ecma262/2024/multipage/structured-data.html#sec-unicodeescape

### エラー型

| 項目 | 説明 |
|------|------|
| `ParseError` | バイト `position` と kind を持つパースエラー |
| `ParseErrorKind` | パース失敗の具体的な原因 |
| `EncodeError` | エンコードエラー : 現在は `NonFiniteNumber` のみ |
| `AccessError` | フィールドアクセスエラー: NotAnObject, MissingField, TypeMismatch |
| `StringDecodeError` | `as_str()` のデコード失敗。種類は上記を参照 |

## ライセンス

MIT
