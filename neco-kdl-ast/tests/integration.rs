use neco_kdl_ast::{
    kdl::serialize, AxisForm, Convention, CrossRef, Document, Marker, NsidPath, PropertyChildForm,
    StructuredNode,
};

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

#[test]
fn default_convention_preserves_existing_accessor_behavior() {
    let input = r#"
        some.node "name" value="something" is=#false count=3 {
            total 7
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

    assert_eq!(serialize(original.as_kdl()), serialize(rendered.as_kdl()));

    let original_node = original
        .find_by_kind(&NsidPath::parse("some.node"))
        .next()
        .expect("orig node");
    let rendered_node = rendered
        .find_by_kind(&NsidPath::parse("some.node"))
        .next()
        .expect("rendered node");

    assert_eq!(
        original_node.attribute_str("value"),
        rendered_node.attribute_str("value")
    );
    assert_eq!(
        original_node.attribute_bool("is"),
        rendered_node.attribute_bool("is")
    );
    assert_eq!(original_node.attribute_int("count"), Some(3));
    assert_eq!(
        original_node.attribute_int("count"),
        rendered_node.attribute_int("count")
    );
    assert_eq!(original_node.attribute_int("total"), Some(7));
    assert_eq!(
        original_node.attribute_int("total"),
        rendered_node.attribute_int("total")
    );

    assert_eq!(
        original_node.node_name_as_nsid(),
        rendered_node.node_name_as_nsid()
    );

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

#[test]
fn roundtrip_axis_2_procedure() {
    let input = "chain {\n    call \"a\"\n    .call \"b\"\n}\n";
    let original = Document::parse(input).expect("parse");
    let original_str = serialize(original.as_kdl());

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

#[test]
fn render_as_public_api_present() {
    let doc = Document::parse("node \"x\"\n").expect("parse");
    let _ = doc.render_as(&Convention::default());
}
