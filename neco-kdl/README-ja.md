# neco-kdl

ゼロ外部依存の KDL v2 パーサー/シリアライザー。設定ファイルや DSL の読み書きに使える。

## 機能

- KDL v2 仕様の完全パース
  - `multiline string`、 `raw string`、 `escline`
  - 型注釈: `(type)node`
  - `slashdash` コメント (`/-`)、ブロックコメント (`/* ... */`)、ネストコメント
  - `#true` / `#false` / `#null` / `#inf` / `#-inf` / `#nan` キーワード
  - 16 進数・8 進数・2 進数リテラル、アンダースコア区切り
  - 版マーカー (`/- kdl-version 2`)
- `parse_spanned` による位置付き構文木 ( 各要素に位置 span を付与 )
- `serialize()` および `Display` 実装によるシリアライズ ( 往復安全 )
- 正規化出力 ( 公式テストスイートの `expected_kdl` と完全一致 )
- フォーマット非依存の `Value` 変換 (`Value` <-> `KdlDocument`)
- ゼロ外部依存
- 公式テストスイート全件通過

## 使い方

```toml
[dependencies]
neco-kdl = "0.5"
```

```rust
use neco_kdl::{parse, serialize, normalize};

fn main() {
    let src = r#"
        node "hello" key=#true {
            child 42
        }
    "#;

    let doc = parse(src).unwrap();

    // ノードを走査する
    for node in doc.nodes() {
        println!("{}: {} entries", node.name(), node.entries().len());
    }

    // KDL テキストに書き戻す
    let output = serialize(&doc);
    print!("{}", output);

    // 正規化形式に変換する
    let normalized = normalize(&doc);
    print!("{}", normalized);
}
```

## API

### `parse`

```rust
pub fn parse(input: &str) -> Result<KdlDocument, KdlError>
```

KDL v2 ドキュメントをパースして `KdlDocument` を返す。結果は `parse_spanned` から導出するため、両入口は同じ入力を受理し同じエラーを返す。

### `parse_spanned`

```rust
pub fn parse_spanned(input: &str) -> Result<SpannedKdlDocument, KdlError>
```

解析結果の各要素に位置情報を付与する。span は入力 byte の半開区間である。境界の規則は次のとおり。

- node の開始は前置の型注釈、型注釈が無い場合は name の先頭。
- node の終端は最後に採用した要素の消費直後で、末尾の空白や区切りを含めない。
- entry の開始は、argument では値の型注釈または値の先頭、property では key の先頭。終端は値の末尾。
- slashdash で除外した要素も親の span に含める。

アクセサ:

```rust
pub struct KdlSpan {
    pub start: usize,
    pub end: usize,
}

impl SpannedKdlDocument {
    pub fn nodes(&self) -> &[SpannedKdlNode];
    pub fn into_document(self) -> KdlDocument;
}

impl SpannedKdlNode {
    pub fn span(&self) -> KdlSpan;
    pub fn ty(&self) -> Option<&str>;
    pub fn name(&self) -> &str;
    pub fn entries(&self) -> &[SpannedKdlEntry];
    pub fn children(&self) -> Option<&[SpannedKdlNode]>;
    pub fn into_node(self) -> KdlNode;
}

impl SpannedKdlEntry {
    pub fn span(&self) -> KdlSpan;
    pub fn entry(&self) -> &KdlEntry;
}
```

`into_document` は span を除いて `parse` と同じ結果を返す。

### `line_col`

```rust
pub fn line_col(input: &str, offset: usize) -> Option<(usize, usize)>
```

byte offset を一始まりの行と列へ導く。範囲外と文字の途中には `None` を返す。改行の規則は解析と同一で、`KdlError` の位置もこれに従う。

### `serialize`

```rust
pub fn serialize(doc: &KdlDocument) -> String
```

`KdlDocument` を KDL テキストに変換する。`KdlDocument`、`KdlNode`、`KdlEntry`、`KdlValue` はすべて `Display` も実装している。

### `normalize`

```rust
pub fn normalize(doc: &KdlDocument) -> String
```

`KdlDocument` を正規化形式の文字列に変換する。正規化ルールは以下のとおり:

- コメントを除去
- property をキーのアルファベット順に並び替え
- 重複 property は後勝ち(最後に出現した値を採用)
- 文字列はすべて引用付き文字列に統一
- 識別子として有効な文字列はアンクォート
- インデントは 4 スペース
- 数値は 10 進数に変換し、アンダースコアを除去
- 末尾改行あり

### `value_to_kdl_document` / `kdl_document_to_value`

```rust
pub fn value_to_kdl_document(value: &Value) -> Result<KdlDocument, ValueError>
pub fn kdl_document_to_value(doc: &KdlDocument) -> Result<Value, ValueError>
```

`KdlDocument` とフォーマット非依存の `Value` `enum` を相互変換する。`Value` は KDL と他フォーマット (JSON、 `CBOR` 等) の橋渡し用中間表現として使える。

空の Object は空の children block として保存する。保存できない入力は、次の 7 種の理由で弁別する。

- 空の Array
- Array 内の入れ子になった値
- property と未対応の型注釈
- argument と children の混在
- 解釈値を持たない数値

### `KdlNumber`

構築時に KDL 数値の文法と、指定した解釈値の一致を検証する。

```rust
impl KdlNumber {
    pub fn new(raw: impl Into<String>, as_i64: Option<i64>, as_f64: Option<f64>)
        -> Result<Self, NumberError>;
    pub fn raw(&self) -> &str;
    pub fn as_i64(&self) -> Option<i64>;
    pub fn as_f64(&self) -> Option<f64>;
}
```

### 主な型

| 項目 | 説明 |
|------|------|
| `KdlDocument` | パース結果のルート。`nodes()` でノード一覧を取得できる |
| `KdlNode` | ノード。`ty()`、`name()`、`entries()`、`children()` で各要素を参照 |
| `KdlEntry` | `Argument`(位置引数)または `Property`(名前付き引数) |
| `KdlSpan` | UTF-8 byte の半開区間。公開 field を持つ |
| `SpannedKdlDocument` | 位置付き document のルート。通常値へ変換できる |
| `SpannedKdlNode` | span と通常 node を持つ位置付きノード |
| `SpannedKdlEntry` | span と通常 entry を持つ位置付きエントリ |
| `KdlValue` | `String(String)`、`Number(KdlNumber)`、`Bool(bool)`、`Null` |
| `KdlNumber` | raw 文法と整数・浮動小数点の解釈値を検証する数値 |
| `NumberError` | raw 文法と解釈値の不一致 |
| `KdlError` | `line()`、`col()`(1-based)、`kind()` でエラー情報を参照 |
| `KdlErrorKind` | エラー種別(`UnexpectedChar`、`InvalidEscape`、`UnclosedString` 等) |
| `Value` | フォーマット非依存の中間表現: `Null`、`Bool`、`Integer`、`Float`、`String`、`Array`、`Object` |
| `ValueError` | `ValueErrorReason` を持つ Value 変換エラー |
| `ValueErrorReason` | 保存できない変換を弁別する 7 種の拒否理由 |

## ライセンス

MIT
