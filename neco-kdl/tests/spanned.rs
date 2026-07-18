//! 位置付き構文木の span を実値で観測する検査。
//!
//! 期待する byte offset と行位置は入力から独立に手で確定した実値である。

use neco_kdl::{line_col, parse, parse_spanned, KdlEntry, KdlSpan, KdlValue};

fn span(start: usize, end: usize) -> KdlSpan {
    KdlSpan { start, end }
}

fn arg_value<'a>(entry: &'a KdlEntry) -> &'a KdlValue {
    match entry {
        KdlEntry::Argument { value, .. } => value,
        KdlEntry::Property { value, .. } => value,
    }
}

/// 判定条件 1, 3: span 構造と位置識別。
#[test]
fn span_structure_and_identity() {
    // 型注釈と各種の要素と子ノードを併せ持つ node で span を観測する。
    let input = "(t)node 1 key=2 kv=(u8)3 {\n  child\n}\n";
    let document = parse_spanned(input).unwrap();
    assert_eq!(document.nodes().len(), 1);
    let node = &document.nodes()[0];

    assert_eq!(node.ty(), Some("t"));
    assert_eq!(node.name(), "node");
    // 開始は型注釈、終端は子ブロックの閉じ括弧の直後。
    assert_eq!(node.span(), span(0, 36));
    assert_eq!(&input[0..36], "(t)node 1 key=2 kv=(u8)3 {\n  child\n}");

    let entries = node.entries();
    assert_eq!(entries.len(), 3);
    // argument: 値の先頭から末尾。
    assert_eq!(entries[0].span(), span(8, 9));
    assert_eq!(&input[8..9], "1");
    // property: key の先頭から値の末尾。
    assert_eq!(entries[1].span(), span(10, 15));
    assert_eq!(&input[10..15], "key=2");
    // 値に型注釈を持つ property: 開始は key の先頭。
    assert_eq!(entries[2].span(), span(16, 24));
    assert_eq!(&input[16..24], "kv=(u8)3");

    let children = node.children().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name(), "child");
    assert_eq!(children[0].span(), span(29, 34));
    assert_eq!(&input[29..34], "child");

    // 子要素の span は親に含まれる。
    for entry in entries {
        assert!(node.span().start <= entry.span().start && entry.span().end <= node.span().end);
    }
    for child in children {
        assert!(node.span().start <= child.span().start && child.span().end <= node.span().end);
    }

    // 要素を持たない node: 終端は name の消費直後。
    let only_name = parse_spanned("node\n").unwrap();
    assert_eq!(only_name.nodes()[0].span(), span(0, 4));

    // 型注釈と name のみの node: 末尾の空白と node terminator を含めない。
    let typed_name = parse_spanned("(t)node   \n").unwrap();
    assert_eq!(typed_name.nodes()[0].span(), span(0, 7));

    // 末尾の slashdash entry: node span に含めるが出力に残さない。
    let trailing_slashdash_entry = parse_spanned("node a /- b\n").unwrap();
    let node = &trailing_slashdash_entry.nodes()[0];
    assert_eq!(node.span(), span(0, 11));
    assert_eq!(node.entries().len(), 1);
    assert_eq!(node.entries()[0].span(), span(5, 6));

    // 末尾の slashdash 子ブロックは span に含めるが出力に残さない。
    let trailing_slashdash_child = parse_spanned("node /- { x }\n").unwrap();
    let node = &trailing_slashdash_child.nodes()[0];
    assert_eq!(node.span(), span(0, 13));
    assert!(node.children().is_none());
    assert!(node.entries().is_empty());

    // 同じ名前や同じ値を繰り返しても個別の span を持つ。
    let same_name_nodes = parse_spanned("a\na\n").unwrap();
    assert_eq!(same_name_nodes.nodes()[0].span(), span(0, 1));
    assert_eq!(same_name_nodes.nodes()[1].span(), span(2, 3));

    let same_value_args = parse_spanned("node 1 1\n").unwrap();
    let entries = same_value_args.nodes()[0].entries();
    assert_eq!(entries[0].span(), span(5, 6));
    assert_eq!(entries[1].span(), span(7, 8));

    let same_key_props = parse_spanned("node x=1 x=1\n").unwrap();
    let entries = same_key_props.nodes()[0].entries();
    assert_eq!(entries[0].span(), span(5, 8));
    assert_eq!(entries[1].span(), span(9, 12));
}

/// 判定条件 4: Unicode と CRLF の span と行位置を実値で観測する。
#[test]
fn unicode_and_crlf_spans_are_utf8_byte_ranges() {
    // 先頭コメント、多 byte の名前と文字列、CRLF 改行を含む入力。
    let input = "// c\r\nnπ \"café\"\r\n";
    let document = parse_spanned(input).unwrap();
    let node = &document.nodes()[0];

    // node 名は "nπ"、span は byte 半開区間。
    assert_eq!(node.name(), "nπ");
    assert_eq!(node.span(), span(6, 17));
    assert_eq!(&input[6..17], "nπ \"café\"");

    let entries = node.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].span(), span(10, 17));
    assert_eq!(
        arg_value(entries[0].entry()),
        &KdlValue::String("café".to_string())
    );

    // span の端は UTF-8 の文字列境界にある。
    assert!(input.is_char_boundary(node.span().start));
    assert!(input.is_char_boundary(node.span().end));
    assert!(input.is_char_boundary(entries[0].span().start));
    assert!(input.is_char_boundary(entries[0].span().end));

    // line_col の一始まりの行と列の実値。CRLF を一改行に畳む。
    assert_eq!(line_col(input, 6), Some((2, 1))); // 'n'
    assert_eq!(line_col(input, 7), Some((2, 2))); // 'π'
    assert_eq!(line_col(input, 10), Some((2, 4))); // '"'

    // 入力先頭の BOM は一文字として列に数える。
    let bom_input = "\u{FEFF}node\n";
    let bom_document = parse_spanned(bom_input).unwrap();
    assert_eq!(bom_document.nodes()[0].span(), span(3, 7));
    assert_eq!(line_col(bom_input, 3), Some((1, 2))); // 'n'

    // 多 byte 文字の途中と範囲外の offset には None を返す。
    assert_eq!(line_col(input, 8), None); // 'π' の途中
    assert_eq!(line_col(input, 15), None); // 'é' の途中
    assert_eq!(line_col(input, 20), None); // 範囲外

    // 通常 document への変換は source 由来の値だけを表す。
    let plain = document.into_document();
    assert_eq!(plain.nodes()[0].name(), "nπ");
    assert_eq!(plain.nodes()[0].first_string_arg(), Some("café"));
}

/// 判定条件 5: slashdash で除外した要素が出力に残らない。
#[test]
fn slashdash_elements_have_no_spanned_output() {
    // slashdash node と slashdash entry。
    let input = "/-skip\nkeep a /-b c\n";
    let document = parse_spanned(input).unwrap();
    assert_eq!(document.nodes().len(), 1);
    let node = &document.nodes()[0];
    assert_eq!(node.name(), "keep");
    assert_eq!(node.span(), span(7, 19));

    let entries = node.entries();
    assert_eq!(entries.len(), 2);
    // 後続の要素の span は入力位置と一致する ( 除外された 'b' で位置がずれない )。
    assert_eq!(entries[0].span(), span(12, 13));
    assert_eq!(
        arg_value(entries[0].entry()),
        &KdlValue::String("a".to_string())
    );
    assert_eq!(entries[1].span(), span(18, 19));
    assert_eq!(
        arg_value(entries[1].entry()),
        &KdlValue::String("c".to_string())
    );

    // 通常 output にも slashdash 要素は残らない。
    let plain = parse(input).unwrap();
    assert_eq!(plain.nodes().len(), 1);
    assert_eq!(plain.nodes()[0].entries().len(), 2);

    // slashdash child block。
    let input = "p /-{ a } { b }\n";
    let document = parse_spanned(input).unwrap();
    let node = &document.nodes()[0];
    assert_eq!(node.span(), span(0, 15));
    let children = node.children().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name(), "b");
    assert_eq!(children[0].span(), span(12, 13));

    let plain = parse(input).unwrap();
    let children = plain.nodes()[0].children().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name(), "b");
}
