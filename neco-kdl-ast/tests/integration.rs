use neco_kdl_ast::{
    kdl::serialize, AxisForm, Convention, CrossRef, Document, Marker, NsidPath,
    PropertyChildForm, StructuredNode,
};
use std::collections::HashSet;

#[test]
fn laplan_corpus_parses_all_required_kinds() {
    let input = include_str!("fixtures/laplan-axiom-corpus.lex");
    let doc = Document::parse(input).expect("parse laplan corpus");
    let kinds: HashSet<String> = doc.structured_nodes().map(|n| n.kind().display()).collect();
    for expected in [
        "lex",
        "cratis",
        "morph",
        "morph.derives",
        "func.family",
        "law",
        "inverse",
        "dual",
        "handler",
        "chain",
        "import",
        "lexicon",
        "face",
    ] {
        assert!(
            kinds.contains(expected),
            "laplan corpus missing kind: {expected}"
        );
    }
}

#[test]
fn laplan_corpus_resolves_cross_refs() {
    let input = include_str!("fixtures/laplan-axiom-corpus.lex");
    let doc = Document::parse(input).expect("parse");
    let handler = doc
        .find_by_kind(&NsidPath::parse("handler"))
        .next()
        .expect("handler");
    let chain = handler
        .find_child_kind(&NsidPath::parse("chain"))
        .next()
        .expect("chain");
    let steps: Vec<_> = chain.children().collect();
    assert!(!steps.is_empty());
    for step in &steps {
        let cr = step.cross_ref_arg().expect("step has arg").expect("parses");
        assert!(!cr.nsid().is_empty());
    }
}

#[test]
fn neployer_corpus_app_target_bindings() {
    let input = include_str!("fixtures/neployer-config-corpus.kdl");
    let doc = Document::parse(input).expect("parse");
    let root = doc
        .find_by_kind(&NsidPath::parse("app"))
        .next()
        .expect("app");
    assert_eq!(
        root.attribute_str("reason"),
        Some("owned outside package manager")
    );
    assert!(root
        .find_child_kind(&NsidPath::parse("target"))
        .next()
        .is_some());
    assert!(root
        .find_child_kind(&NsidPath::parse("bindings"))
        .next()
        .is_some());
    assert!(root
        .find_child_kind(&NsidPath::parse("services"))
        .next()
        .is_some());
    assert!(root
        .find_child_kind(&NsidPath::parse("palette"))
        .next()
        .is_some());
}

#[test]
fn atproto_corpus_fragment_in_cross_ref() {
    let input = include_str!("fixtures/atproto-lexicon-corpus.kdl");
    let doc = Document::parse(input).expect("parse");
    let cr = CrossRef::parse("app.bsky.actor.defs#profileViewDetailed").expect("parse");
    assert_eq!(cr.fragment(), Some("profileViewDetailed"));
    assert_eq!(cr.nsid().display(), "app.bsky.actor.defs");
    assert!(doc.resolve(&cr).is_some());
}

#[test]
fn atproto_corpus_type_annotation_pervasive() {
    let input = include_str!("fixtures/atproto-lexicon-corpus.kdl");
    let doc = Document::parse(input).expect("parse");
    let annotation_count: usize = doc
        .structured_nodes()
        .flat_map(collect_recursive_annotations)
        .count();
    assert!(
        annotation_count >= 10,
        "atproto fixture must have at least 10 typed entries, got {annotation_count}"
    );
}

fn collect_recursive_annotations(n: StructuredNode<'_>) -> Vec<()> {
    let mut out = Vec::new();
    if n.type_annotation().is_some() {
        out.push(());
    }
    for c in n.children() {
        out.extend(collect_recursive_annotations(c));
    }
    out
}

/// driver 2: default Convention で render_as が identity、 既存 read accessor 全種が同値返却
///
/// spec § 不変条件 が列挙する read accessor 9 種を全て assert する:
/// `attribute_str` / `attribute_bool` / `node_name_as_nsid` / `dot_chain_depth` /
/// `dot_chain_kind` / `type_annotation` / `structured_name` / `structured_name_form_x` /
/// `structured_name_form_y`。 fixture は dot-chain ( axis 2 procedure ) + type annotation
/// ( axis 4 ) + property entry ( axis 3 ) + form X kind keyword ( axis 5 ) を含む。
#[test]
fn default_convention_preserves_existing_accessor_behavior() {
    let input = r#"
        some.node "name" value="something" is=#false count=3 {
            child "first"
            child "second"
            chain {
                call "a"
                .call "b"
                ..call "c"
            }
            (i32)typed-input "x"
            lex "kind.qualified.id" {
                procedure "p"
            }
        }
    "#;
    let original = Document::parse(input).expect("parse");
    let rendered = original.render_as(&Convention::default());

    // serialize 比較で identity 確認 ( default Convention は全軸 OFF なので render_as は no-op )
    assert_eq!(serialize(original.as_kdl()), serialize(rendered.as_kdl()));

    let original_node = original
        .find_by_kind(&NsidPath::parse("some.node"))
        .next()
        .expect("orig node");
    let rendered_node = rendered
        .find_by_kind(&NsidPath::parse("some.node"))
        .next()
        .expect("rendered node");

    // attribute_str / attribute_bool ( count は int だが attribute_int は実装に不在、
    // 数値属性は attribute_str で raw 文字列として読む経路で互換性を assert )
    assert_eq!(
        original_node.attribute_str("value"),
        rendered_node.attribute_str("value")
    );
    assert_eq!(
        original_node.attribute_bool("is"),
        rendered_node.attribute_bool("is")
    );
    assert_eq!(
        original_node.attribute_str("count"),
        rendered_node.attribute_str("count")
    );

    // node_name_as_nsid
    assert_eq!(
        original_node.node_name_as_nsid(),
        rendered_node.node_name_as_nsid()
    );

    // structured_name / structured_name_form_x / structured_name_form_y
    assert_eq!(
        original_node.structured_name(),
        rendered_node.structured_name()
    );
    assert_eq!(
        original_node.structured_name_form_x(),
        rendered_node.structured_name_form_x()
    );
    assert_eq!(
        original_node.structured_name_form_y(),
        rendered_node.structured_name_form_y()
    );

    // type_annotation ( 子 node の typed-input から取得 )
    let original_typed = original_node
        .children()
        .find(|c| c.dot_chain_kind() == "typed-input")
        .expect("orig typed-input");
    let rendered_typed = rendered_node
        .children()
        .find(|c| c.dot_chain_kind() == "typed-input")
        .expect("rendered typed-input");
    assert_eq!(
        original_typed.type_annotation(),
        rendered_typed.type_annotation()
    );

    // dot_chain_depth / dot_chain_kind ( chain 内の dot prefix node から )
    let original_chain = original_node
        .children()
        .find(|c| c.dot_chain_kind() == "chain")
        .expect("orig chain");
    let rendered_chain = rendered_node
        .children()
        .find(|c| c.dot_chain_kind() == "chain")
        .expect("rendered chain");
    let original_dot_calls: Vec<_> = original_chain.children().collect();
    let rendered_dot_calls: Vec<_> = rendered_chain.children().collect();
    assert_eq!(original_dot_calls.len(), rendered_dot_calls.len());
    for (o, r) in original_dot_calls.iter().zip(rendered_dot_calls.iter()) {
        assert_eq!(o.dot_chain_depth(), r.dot_chain_depth());
        assert_eq!(o.dot_chain_kind(), r.dot_chain_kind());
    }
}

/// driver 4: 軸 1 ( namespace ) round-trip lossless
///
/// 入力は canonical collapsed form ( 単一 dot-chain leaf )。 expand → nested form、
/// collapse → 元の dot-chain。 round-trip は canonical form 上で同一性を保つ。
#[test]
fn roundtrip_axis_1_namespace() {
    let input = "encoding.base64.encode \"impl\"\n";
    let original = Document::parse(input).expect("parse");
    let original_str = serialize(original.as_kdl());

    let expand_conv = Convention::new().with_namespace_form(AxisForm::Expand);
    let collapse_conv = Convention::new().with_namespace_form(AxisForm::Collapse);
    let cycled = original
        .render_as(&expand_conv)
        .render_as(&collapse_conv)
        .render_as(&expand_conv)
        .render_as(&collapse_conv);
    assert_eq!(serialize(cycled.as_kdl()), original_str);
}

/// driver 4: 軸 2 ( procedure ) round-trip lossless
#[test]
fn roundtrip_axis_2_procedure() {
    let input = "chain {\n    call \"a\"\n    .call \"b\"\n}\n";
    let original = Document::parse(input).expect("parse");
    let original_str = serialize(original.as_kdl());

    // Convention.markers に Marker::Kind("call") を載せて axis 2 transform の対象 kind を提供
    let expand_conv = Convention::new()
        .with_marker(Marker::Kind("call".to_owned()))
        .with_procedure_form(AxisForm::Expand);
    let collapse_conv = Convention::new()
        .with_marker(Marker::Kind("call".to_owned()))
        .with_procedure_form(AxisForm::Collapse);
    let cycled = original
        .render_as(&expand_conv)
        .render_as(&collapse_conv)
        .render_as(&expand_conv)
        .render_as(&collapse_conv);
    assert_eq!(serialize(cycled.as_kdl()), original_str);
}

/// driver 4: 軸 3 ( property-child ) round-trip lossless ( 融合形のみ )
///
/// 入力は canonical collapsed form ( property のみ、 flat key-value 子を持たない )。
/// expand → properties が flat key-value 子に展開、 collapse → properties に戻る。
/// 入力に flat key-value 子が混在すると collapse 時に property に同化されるため
/// canonical input でのみ round-trip lossless が保証される。
#[test]
fn roundtrip_axis_3_property_child() {
    let input = "node \"name\" value=\"something\" is=#false count=3\n";
    let original = Document::parse(input).expect("parse");
    let original_str = serialize(original.as_kdl());

    let expand_conv = Convention::new().with_property_child_form(PropertyChildForm::Expand);
    let collapse_conv = Convention::new().with_property_child_form(PropertyChildForm::Collapse);
    let cycled = original
        .render_as(&expand_conv)
        .render_as(&collapse_conv)
        .render_as(&expand_conv)
        .render_as(&collapse_conv);
    assert_eq!(serialize(cycled.as_kdl()), original_str);
}

/// driver 4: 軸 4 ( type annotation ) round-trip lossless ( marker 込み )
#[test]
fn roundtrip_axis_4_type_annotation() {
    let input = "input { (i32)value }\n";
    let original = Document::parse(input).expect("parse");
    let original_str = serialize(original.as_kdl());

    let marker = Marker::Prefix(':');
    let expand_conv = Convention::new()
        .with_marker(marker.clone())
        .with_type_annotation_form(AxisForm::ExpandWithMarker(marker.clone()));
    let collapse_conv = Convention::new()
        .with_marker(marker.clone())
        .with_type_annotation_form(AxisForm::CollapseWithMarker(marker.clone()));
    let cycled = original
        .render_as(&expand_conv)
        .render_as(&collapse_conv)
        .render_as(&expand_conv)
        .render_as(&collapse_conv);
    assert_eq!(serialize(cycled.as_kdl()), original_str);
}

/// driver 4: 軸 5 ( kind keyword ) round-trip lossless ( marker 込み )
#[test]
fn roundtrip_axis_5_kind_keyword() {
    let input = "lex \"foo\" { procedure { } }\n";
    let original = Document::parse(input).expect("parse");
    let original_str = serialize(original.as_kdl());

    let marker = Marker::Prefix('@');
    let expand_conv = Convention::new()
        .with_marker(marker.clone())
        .with_kind_keyword_form(AxisForm::ExpandWithMarker(marker.clone()));
    let collapse_conv = Convention::new()
        .with_marker(marker.clone())
        .with_kind_keyword_form(AxisForm::CollapseWithMarker(marker.clone()));
    let cycled = original
        .render_as(&expand_conv)
        .render_as(&collapse_conv)
        .render_as(&expand_conv)
        .render_as(&collapse_conv);
    assert_eq!(serialize(cycled.as_kdl()), original_str);
}

/// driver 3 補助: render_as が public API として呼べる ( compile-time )
#[test]
fn render_as_public_api_present() {
    let doc = Document::parse("node \"x\"\n").expect("parse");
    let _ = doc.render_as(&Convention::default());
}
