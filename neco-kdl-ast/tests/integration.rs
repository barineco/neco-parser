use neco_kdl_ast::{CrossRef, Document, NsidPath, StructuredNode};
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
