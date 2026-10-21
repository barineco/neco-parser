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

#[test]
fn span_structure_and_identity() {
    let input = "(t)node 1 key=2 kv=(u8)3 {\n  child\n}\n";
    let document = parse_spanned(input).unwrap();
    assert_eq!(document.nodes().len(), 1);
    let node = &document.nodes()[0];

    assert_eq!(node.ty(), Some("t"));
    assert_eq!(node.name(), "node");

    assert_eq!(node.span(), span(0, 36));
    assert_eq!(&input[0..36], "(t)node 1 key=2 kv=(u8)3 {\n  child\n}");

    let entries = node.entries();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].span(), span(8, 9));
    assert_eq!(&input[8..9], "1");

    assert_eq!(entries[1].span(), span(10, 15));
    assert_eq!(&input[10..15], "key=2");

    assert_eq!(entries[2].span(), span(16, 24));
    assert_eq!(&input[16..24], "kv=(u8)3");

    let children = node.children().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name(), "child");
    assert_eq!(children[0].span(), span(29, 34));
    assert_eq!(&input[29..34], "child");

    for entry in entries {
        assert!(node.span().start <= entry.span().start && entry.span().end <= node.span().end);
    }
    for child in children {
        assert!(node.span().start <= child.span().start && child.span().end <= node.span().end);
    }

    let only_name = parse_spanned("node\n").unwrap();
    assert_eq!(only_name.nodes()[0].span(), span(0, 4));

    let typed_name = parse_spanned("(t)node   \n").unwrap();
    assert_eq!(typed_name.nodes()[0].span(), span(0, 7));

    let trailing_slashdash_entry = parse_spanned("node a /- b\n").unwrap();
    let node = &trailing_slashdash_entry.nodes()[0];
    assert_eq!(node.span(), span(0, 11));
    assert_eq!(node.entries().len(), 1);
    assert_eq!(node.entries()[0].span(), span(5, 6));

    let trailing_slashdash_child = parse_spanned("node /- { x }\n").unwrap();
    let node = &trailing_slashdash_child.nodes()[0];
    assert_eq!(node.span(), span(0, 13));
    assert!(node.children().is_none());
    assert!(node.entries().is_empty());

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

#[test]
fn unicode_and_crlf_spans_are_utf8_byte_ranges() {
    let input = "// c\r\nnπ \"café\"\r\n";
    let document = parse_spanned(input).unwrap();
    let node = &document.nodes()[0];

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

    assert!(input.is_char_boundary(node.span().start));
    assert!(input.is_char_boundary(node.span().end));
    assert!(input.is_char_boundary(entries[0].span().start));
    assert!(input.is_char_boundary(entries[0].span().end));

    assert_eq!(line_col(input, 6), Some((2, 1)));
    assert_eq!(line_col(input, 7), Some((2, 2)));
    assert_eq!(line_col(input, 10), Some((2, 4)));

    let bom_input = "\u{FEFF}node\n";
    let bom_document = parse_spanned(bom_input).unwrap();
    assert_eq!(bom_document.nodes()[0].span(), span(3, 7));
    assert_eq!(line_col(bom_input, 3), Some((1, 2)));

    assert_eq!(line_col(input, 8), None);
    assert_eq!(line_col(input, 15), None);
    assert_eq!(line_col(input, 20), None);

    let plain = document.into_document();
    assert_eq!(plain.nodes()[0].name(), "nπ");
    assert_eq!(plain.nodes()[0].first_string_arg(), Some("café"));
}

#[test]
fn slashdash_elements_have_no_spanned_output() {
    let input = "/-skip\nkeep a /-b c\n";
    let document = parse_spanned(input).unwrap();
    assert_eq!(document.nodes().len(), 1);
    let node = &document.nodes()[0];
    assert_eq!(node.name(), "keep");
    assert_eq!(node.span(), span(7, 19));

    let entries = node.entries();
    assert_eq!(entries.len(), 2);

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

    let plain = parse(input).unwrap();
    assert_eq!(plain.nodes().len(), 1);
    assert_eq!(plain.nodes()[0].entries().len(), 2);

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
