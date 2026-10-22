# neco-kdl

[English](README.md)

`neco-kdl` は、外部のクレートに依存しない KDL v2 向けの Rust クレートです。
文書の解析、KDL テキストへの直列化、正規化出力、およびビルダーによる文書の組み立てを行えます。

## 機能

- KDL v2 の構文解析
  - 複数行文字列、生文字列、行継続のエスケープ
  - 型注釈: `(type)node`
  - slashdash コメント ( `/-` )、ブロックコメント、入れ子のコメント
  - キーワードリテラル
    - 真偽値: `#true`、`#false`
    - null: `#null`
    - 無限大: `#inf`、`#-inf`
    - 非数: `#nan`
  - 16 進数、8 進数、2 進数のリテラルとアンダースコア区切り
  - バージョンマーカー ( `/- kdl-version 2` )
- `parse_spanned`: ノードとエントリの UTF-8 バイト位置を保持する構文木の構築
- `serialize` と `Display`: 値型から KDL テキストへの直列化
- `normalize`: 公式テストスイートの `expected_kdl` と一致する正規化出力
- `Value` 変換: `KdlDocument` と形式共通の値との相互変換
- KDL 公式テストスイートの全成功入力を解析し、直列化と再解析の後も同一の文書に復元

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

KDL v2 の文書を解析し、`KdlDocument` を返します。
通常の解析は位置付き解析の結果から位置情報を除いて構築するため、両方の関数は同じ入力を受理し、同じエラーを返します。

### `parse_spanned`

```rust
pub fn parse_spanned(input: &str) -> Result<SpannedKdlDocument, KdlError>
```

KDL v2 の文書を解析し、各ノードとエントリに入力上の位置を付けた `SpannedKdlDocument` を返します。
位置は `KdlSpan` で表され、UTF-8 バイト列の半開区間 `[start, end)` を保持します。

- ノードの開始位置: 前置された型注釈の先頭 ( 型注釈を省略した場合はノード名の先頭 )
- ノードの終了位置: ノード名、または末尾のエントリ、子ブロック、slashdash エントリ、slashdash 子ブロックの直後 ( 末尾の空白、コメント、改行やセミコロンは区間の外 )
- エントリの開始位置: 引数では型注釈または値の先頭、プロパティではキーの先頭
- エントリの終了位置: 値の末尾
- slashdash の扱い: 構文木から除かれたエントリと子ブロックも、親ノードの区間に含まれる

主なアクセサ:

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

`into_document` を使うと、位置情報を除いた通常の `KdlDocument` を取り出せます。

### `line_col`

```rust
pub fn line_col(input: &str, offset: usize) -> Option<(usize, usize)>
```

バイト位置から 1 始まりの行と列を算出します。
範囲外のオフセットや UTF-8 文字の途中では `None` を返します。
改行の `\r\n` は一つの改行として数えられ、解析エラーの位置にも同じ規則が適用されます。

### `serialize`

```rust
pub fn serialize(doc: &KdlDocument) -> String
```

文書を KDL テキストへ変換します。
以下の型は `Display` も実装しています。

- `KdlDocument`
- `KdlNode`
- `KdlEntry`
- `KdlValue`

### `normalize`

```rust
pub fn normalize(doc: &KdlDocument) -> String
```

`KdlDocument` を正規化形式の文字列へ変換します。

- コメントを除去
- プロパティをキーの辞書順に並べ替え
- 重複するプロパティには、最後に出現した値を採用
- 文字列を引用符付きまたは有効な裸の識別子として出力
- インデントを 4 個のスペースに統一
- 空の子ブロックを除去 ( `node {}` は `node` )
- 数値を 10 進表記へ変換し、アンダースコアを除去
- 整数の負のゼロは符号を除去して `-0` を `0` にし、小数の -0.0 はそのまま保持
- 指数表記を大文字 E と符号付きの指数へ統一 ( `1.5e10` は `1.5E+10` )
- 整数部の先頭のゼロを除き、小数部の末尾のゼロを保持 ( 007 は 7、`00.50` は `0.50` )
- 末尾に改行を追加

### `value_to_kdl_document` / `kdl_document_to_value`

```rust
pub fn value_to_kdl_document(value: &Value) -> Result<KdlDocument, ValueError>
pub fn kdl_document_to_value(doc: &KdlDocument) -> Result<Value, ValueError>
```

`KdlDocument` と、形式共通の中間表現である `Value` を相互変換します。
JSON や CBOR などの形式との間で値を受け渡すときに利用できます。

KDL ノードと `Value` の対応規則:

- 引数がなく子ブロックもないノードは `Null` になる
- `Value::Null` は `#null` の引数 1 個を持つノードになる
- 引数が 1 個のノードはその値に対応し、2 個以上のノードは `Array` になる
- 子ブロックだけを持つノードは `Object` になる
- 入れ子の空のオブジェクトは、空の子ブロックを持つノードとして保存される
- `Integer` は型注釈なしの数値になる
- `Float` は `(f64)` の型注釈付きの数値になる
- 型注釈のない数値は、i64 に収まれば `Integer`、収まらなければ `Float` へ戻る

変換に失敗した場合は `ValueError` を返し、その原因を 7 種類の `ValueErrorReason` で表します。

- `TopLevelMustBeObject`: 最上位に配列またはプリミティブ値を指定
- `NestedCollectionInArray`: 配列要素に配列またはオブジェクトを指定
- `EmptyArrayCannotBeConverted`: 空の配列を指定
- `PropertyCannotBeConverted`: KDL ノードにプロパティが存在
- `MixedArgumentsAndChildren`: 同じ KDL ノードに引数と子ブロックが存在
- `UnsupportedTypeAnnotation(String)`: ノードの型注釈、または数値の `(f64)` という対応から外れる引数の型と値の組
- `NumberCannotBeConverted(String)`: KDL 数値の整数解釈と浮動小数点解釈がともに失敗

### 文書ビルダー

文書の構築は `KdlDocument::builder()` から始めます。
ノードの構築は `KdlNode::builder(name)` から始め、どちらもメソッドチェーンで値を追加できます。

```rust
use neco_kdl::{KdlDocument, KdlNode};

let document = KdlDocument::builder()
    .node(KdlNode::builder("greet").string_arg("hello").build())
    .node(KdlNode::builder("enabled").bool_arg(true).build())
    .build();

assert_eq!(document.nodes().len(), 2);
```

### `KdlNumber`

構築時に KDL 数値の文法と、指定した解釈値が一致することを検証します。

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
| `KdlDocument` | 解析結果のルート |
| `KdlNode` | 型、名前、エントリ、子ノードを持つノード |
| `KdlEntry` | 位置引数または名前付き引数 |
| `KdlDocumentBuilder` | 文書へノードを追加するビルダー |
| `KdlNodeBuilder` | 引数、プロパティ、子ノード、型注釈を追加するビルダー |
| `KdlSpan` | UTF-8 バイト列の半開区間 |
| `SpannedKdlDocument` | 位置付き文書のルート |
| `SpannedKdlNode` | 入力上の位置情報を持つノード |
| `SpannedKdlEntry` | 位置と通常のエントリを持つエントリ |
| `KdlValue` | 文字列、数値、真偽値、null の値 |
| `KdlNumber` | 元の表記と数値の解釈値を検証した値 |
| `NumberError` | 元の表記と解釈値の不一致 |
| `KdlError` | 行、列、種類を持つ解析エラー |
| `KdlErrorKind` | 文字、エスケープ、文字列などのエラー種別 |
| `Value` | 形式共通の中間表現 |
| `ValueError` | 変換が失敗した理由を持つエラー |
| `ValueErrorReason` | 変換の失敗を表す 7 種類の理由 |

主な型のバリアントとアクセサの構成を示します。

```text
KdlEntry = Argument | Property
KdlValue = String(String) | Number(KdlNumber) | Bool(bool) | Null
Value = Null | Bool | Integer | Float | String | Array | Object
KdlNode accessors = ty() | name() | entries() | children()
KdlError accessors = line() | col() | kind()
KdlErrorKind examples = UnexpectedChar | InvalidEscape | UnclosedString
```

## ライセンス

MIT
