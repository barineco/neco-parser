# neco-json

[English](README.md)

neco-json は、依存最小の小さな JSON コーデックです。
`#![no_std]` と `alloc` の環境で動作し、JSON の解析と出力、型付きのフィールドアクセス、および Rust の値との相互変換を行えます。

読み取り API には、メモリ確保の方法が異なる 2 つの経路があります。

- 所有型の値: `parse` が返す `JsonValue`
- 入力を借用する値: `parse_borrowed` が返す `JsonRef`

所有型では数値の入力表記を保存します。
保存先の `JsonNumber`、配列、文字列には Rust の通常の確保 API を使うため、確保に失敗したときの動作はアプリケーションのアロケータと Rust 処理系に従います。
一方、借用読み取りでは、構文の検証と大半の値の参照を入力の借用とスタック上の値だけで行い、エスケープを含む文字列を `as_str()` でデコードするときにだけメモリを確保します。
この確保やデコードに失敗したときの戻り値は `StringDecodeError` です。

## 機能

- `#![no_std]` + `alloc` での動作
- `parse(&[u8]) -> Result<JsonValue, ParseError>`
- `encode(&JsonValue) -> Result<Vec<u8>, EncodeError>`
- `JsonValue` で扱う JSON の型 ( null、真偽値、数値、文字列、配列、オブジェクト )
- 必須または省略可能なオブジェクトフィールドの型付きアクセサ
- Rust の値と相互変換する `ToJson` / `FromJson` トレイト
- `parse_borrowed(&str) -> Result<JsonRef<'_>, ParseError>`: `parse` と文法および失敗位置が共通する、メモリ確保なしの検証済み借用値 ( ネストは 128 段まで )
  - `array_items()` と `object_entries()`: 配列とオブジェクトの反復子
- `JsonString`: 呼び出し側が渡す `core::fmt::Write` の書き込み先へ、引用符付きのエスケープ済み文字列をメモリ確保なしで書き出す型

## 使い方

`Cargo.toml` に依存関係を追加します。

```toml
[dependencies]
neco-json = "0.2"
```

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
    ("x".into(), JsonValue::from(1_i64)),
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
値の種類、生の文字列、位置、真偽値、整数、配列とオブジェクトの反復、および文字列の比較は、入力の借用とスタック上の値だけで動作するため、メモリ確保は生じません。
文字列を `Cow` として取り出す `as_str()` では、エスケープを含む場合にだけメモリを確保します。
次の例では、重複するオブジェクトのキーと、同じ要素を 2 回持つ配列を読み取ります。
`Vec::collect` の代わりに通常の `for` 反復と固定長のスタック配列を使い、エスケープのない文字列だけを読むことで、走査全体のメモリ確保がゼロになります。

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

`as_u64()` は `u64::MAX` 以下の 10 進整数の表記を読み取ります。
負号、小数部、指数部を含む表記や範囲を超える値に対しては `None` を返します。
所有 API の `JsonNumber` も同じ入力表記を保存しており、同じ条件で整数値を取得できます。

### 必要な時だけ文字列をデコードする

`as_str()` は文字列値をデコードし、文字列以外の種類では `Ok(None)` を返します。
エスケープのない文字列は入力を借用するだけでメモリ確保は生じませんが、エスケープを含む文字列では、デコード後の正確な UTF-8 バイト数に合わせて確保を 1 回だけ行います。
また、`string_eq()` を使えば、メモリを確保せずにデコードしながら `&str` と比較できます。

```rust
use neco_json::{parse_borrowed, StringDecodeError};
use std::borrow::Cow;

let plain = parse_borrowed("\"neco\"").unwrap();
assert!(matches!(plain.as_str().unwrap(), Some(Cow::Borrowed("neco")))); // 入力を借用するだけ

let escaped = parse_borrowed("\"ne\\u0063o\"").unwrap();
assert_eq!(escaped.string_eq("neco"), Ok(true)); // デコードしながら比較する。メモリ確保なし
match escaped.as_str() {
    Ok(Some(value)) => assert_eq!(value, "neco"), // デコード後の正確な長さに合わせて確保する
    Err(StringDecodeError::AllocationFailed { required }) => {
        // アロケータがちょうど `required` バイトの確保を拒否した
        let _ = required;
    }
    other => panic!("{other:?}"),
}
```

### メモリ確保なしで JSON 文字列を書き出す

`JsonString` は、呼び出し側が渡す `core::fmt::Write` の書き込み先へ、引用符付きのエスケープ済み文字列を書き込みます。
`JsonString` 自身の書き込みではメモリ確保を行いません。
次の例で生じる確保は `String` 自身のバッファ伸長によるものであり、固定長のスタック上バッファを書き込み先にすれば、呼び出し全体のメモリ確保をゼロに保つことができます。

```rust
use core::fmt::Write;
use neco_json::JsonString;

let mut buffer = String::new(); // バッファと、その伸長に伴う確保は呼び出し側が保持する
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
| `parse_borrowed(input: &str) -> Result<JsonRef<'_>, ParseError>` | JSON を検証し、`input` を借用するメモリ確保なしの借用値を返す ( ネストは 128 段まで ) |
| `encode(value: &JsonValue) -> Result<Vec<u8>, EncodeError>` | `JsonValue` を最小の JSON バイト列にエンコードする |
| `ToJson` / `FromJson` | Rust の値と相互変換する軽量トレイト |

### `JsonValue`

JSON の値を表す列挙型です。

```
Null | Bool(bool) | Number(JsonNumber) | String(String) | Array(Vec<JsonValue>) | Object(Vec<(String, JsonValue)>)
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

#### 値取得 ( `Option` )

| 項目 | 説明 |
|------|------|
| `as_bool() -> Option<bool>` | 真偽値を取り出す |
| `as_number() -> Option<&JsonNumber>` | 数値と入力表記を取り出す |
| `as_f64() -> Option<f64>` | Rust の数値パーサーによる有限の解釈値を取り出す |
| `as_i64() -> Option<i64>` | `i64` で表せる整数トークンを取り出す |
| `as_u64() -> Option<u64>` | `u64` で表せる 0 以上の整数トークンを取り出す |
| `as_str() -> Option<&str>` | 文字列スライスを取り出す |
| `as_array() -> Option<&[JsonValue]>` | 配列スライスを取り出す |
| `as_object() -> Option<&[(String, JsonValue)]>` | オブジェクトフィールドのスライスを取り出す |
| `get(key) -> Option<&JsonValue>` | キーでオブジェクトフィールドを検索する |

#### 必須フィールドアクセサ

対象がオブジェクトでない場合、フィールドが欠落している場合、または型が一致しない場合は `Err(AccessError)` を返します。

| 項目 | 説明 |
|------|------|
| `required_str(key) -> Result<&str, AccessError>` | 必須の文字列フィールド |
| `required_bool(key) -> Result<bool, AccessError>` | 必須の真偽値フィールド |
| `required_f64(key) -> Result<f64, AccessError>` | 必須の数値フィールド |
| `required_array(key) -> Result<&[JsonValue], AccessError>` | 必須の配列フィールド |
| `required_object(key) -> Result<&[(String, JsonValue)], AccessError>` | 必須のオブジェクトフィールド |

#### 省略可能フィールドアクセサ

フィールドが欠落している場合や null のときは `Ok(None)` を返し、型が一致しない場合は `Err(AccessError)` を返します。

| 項目 | 説明 |
|------|------|
| `optional_str(key) -> Result<Option<&str>, AccessError>` | 省略可能な文字列フィールド |
| `optional_bool(key) -> Result<Option<bool>, AccessError>` | 省略可能な真偽値フィールド |
| `optional_f64(key) -> Result<Option<f64>, AccessError>` | 省略可能な数値フィールド |
| `optional_array(key) -> Result<Option<&[JsonValue]>, AccessError>` | 省略可能な配列フィールド |

### `JsonNumber`

JSON 数値の入力表記を保存する型です。

- `raw()`: 保存した表記を返す
- `as_i64()`: 整数部だけの表記を `i64` として読み取る
- `as_u64()`: 0 以上の整数部だけの表記を `u64` として読み取る
- `as_f64()`: Rust の数値パーサーによる有限の解釈値を返す ( 大きい値では丸めが生じる )
- `JsonNumber::new`: 一つの JSON 数値トークンを検証して構築する
- `JsonNumber::from_f64`: 有限の `f64` を Rust の表示表記で構築する

### `JsonKind`

`JsonRef::kind()` が返す構文上の種類です。

```
Null | Bool | Number | String | Array | Object
```

### `JsonRef`

元の入力を借用する検証済みの JSON 値です。
公開されている構築関数は `parse_borrowed` だけです。
子要素は `array_items()` または `object_entries()` から取得できます。

| 項目 | 説明 |
|------|------|
| `kind() -> JsonKind` | この値の構文上の種類 |
| `raw() -> &str` | この値が占める入力の厳密な部分文字列 ( 文字列では引用符を含む ) |
| `span() -> Range<usize>` | 元の入力に対するバイトオフセットの半開区間 ( `input[span()] == raw()` が成立する ) |
| `as_bool() -> Option<bool>` | 真偽値 ( 他の種類では `None` ) |
| `as_u64() -> Option<u64>` | u64 の範囲に収まる厳密な 10 進整数値 ( 負号、小数部、指数部、範囲超過では `None` ) |
| `as_str() -> Result<Option<Cow<'_, str>>, StringDecodeError>` | デコード済み文字列 ( 文字列以外では `Ok(None)`、確保の条件は上記の使用例を参照 ) |
| `string_eq(needle: &str) -> Result<bool, ParseError>` | メモリ確保なしでデコードしながら `needle` と比較する |
| `array_items() -> Option<JsonArrayItems<'_>>` | 配列要素の反復子 ( 他の種類では `None` ) |
| `object_entries() -> Option<JsonObjectEntries<'_>>` | オブジェクトエントリの反復子 ( 他の種類では `None` ) |

### `JsonArrayItems` / `JsonObjectEntries`

`array_items()` または `object_entries()` が返す、メモリ確保なしの反復子です。
要素とエントリは入力の出現順で返され、重複するオブジェクトのキーも `JsonValue::Object` と同様にそのまま残ります。
要素に文法違反がある場合は 1 回だけ `Err` を返し、以後の呼び出しでは `None` を返します。

| 項目 | 説明 |
|------|------|
| `JsonArrayItems<'a>: Iterator<Item = Result<JsonRef<'a>, ParseError>>` | 配列要素ごとに 1 件を返す |
| `JsonObjectEntries<'a>: Iterator<Item = Result<(JsonRef<'a>, JsonRef<'a>), ParseError>>` | オブジェクトエントリごとにキーと値の組を 1 件返す ( キーの種類は常に `JsonKind::String` ) |

### `StringDecodeError`

`as_str()` がエスケープを含む文字列をデコードする際のエラー型です。

| 項目 | 説明 |
|------|------|
| `CapacityOverflow { input_len: usize }` | デコード後の長さがアドレス可能な文字列サイズを超える |
| `AllocationFailed { required: usize }` | アロケータが、デコード後の正確な UTF-8 バイト数の確保を拒否した |
| `Malformed(ParseError)` | 元の入力のバイト位置で文字のデコードに失敗した |

### `JsonString`

呼び出し側が渡す `core::fmt::Write` の書き込み先へ、借用した文字列を引用符付きの JSON 文字列として書き出します。
`JsonString` 自身はメモリ確保を行わず、書き込み先の失敗は `core::fmt::Error` としてそのまま呼び出し元へ返します。

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

この文字分類は `encode` 関数と共通ですが、`\u00XX` の形で書く 16 進数の大文字・小文字が異なります。
`encode` は大文字を使います。
`JsonString` は ECMAScript 2024 の `JSON.stringify` の仕様に従って小文字で出力します。
仕様: https://tc39.es/ecma262/2024/multipage/structured-data.html#sec-unicodeescape

### エラー型

| 項目 | 説明 |
|------|------|
| `ParseError` | バイト位置 `position` とエラー種別を持つパースエラー |
| `ParseErrorKind` | パース失敗の具体的な原因 |
| `EncodeError` | エンコードエラー ( 現在の種類は `NonFiniteNumber` ) |
| `AccessError` | フィールドアクセスの失敗 |
| `StringDecodeError` | `as_str()` による文字列デコードの失敗 ( 種類は上記を参照 ) |

`AccessError` では、失敗の原因を 3 種類に区別します。

- `NotAnObject`: アクセス対象がオブジェクト以外
- `MissingField`: 必須フィールドの欠落
- `TypeMismatch`: 値と要求した型の不一致

## ライセンス

MIT
