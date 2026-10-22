use neco_json::{encode, parse, JsonNumber, JsonValue};

#[test]
fn crate_build_contract_is_visible_from_public_api() {
    let value = parse(br#"{"ok":true}"#).expect("public parse should work");
    assert_eq!(value.required_bool("ok"), Ok(true));
    assert_eq!(
        encode(&value).expect("public encode should work"),
        br#"{"ok":true}"#
    );
}

#[test]
fn parse_complex_structure_and_read_with_accessors() {
    let payload = br#"{
        "repo":"did:plc:alice",
        "active":true,
        "score":42.5,
        "profile":{"handle":"alice.test","labels":["dev",null,"cat"]},
        "posts":[
            {"id":"p1","counts":{"likes":3,"replies":0}},
            {"id":"p2","counts":{"likes":5,"replies":1}}
        ]
    }"#;

    let value = parse(payload).expect("complex payload should parse");
    assert_eq!(value.required_str("repo"), Ok("did:plc:alice"));
    assert_eq!(value.required_bool("active"), Ok(true));
    assert_eq!(value.required_f64("score"), Ok(42.5));

    let profile = value.get("profile").expect("profile should exist");
    assert_eq!(profile.required_str("handle"), Ok("alice.test"));
    let labels = profile
        .required_array("labels")
        .expect("labels should be array");
    assert_eq!(labels.len(), 3);
    assert_eq!(labels[0].as_str(), Some("dev"));
    assert!(labels[1].is_null());
    assert_eq!(labels[2].as_str(), Some("cat"));

    let posts = value
        .required_array("posts")
        .expect("posts should be array");
    assert_eq!(posts.len(), 2);
    let first = posts[0].as_object().expect("post should be object");
    let first = JsonValue::Object(first.to_vec());
    assert_eq!(first.required_str("id"), Ok("p1"));
    let counts = first.get("counts").expect("counts should exist");
    assert_eq!(counts.required_f64("likes"), Ok(3.0));
    assert_eq!(counts.required_f64("replies"), Ok(0.0));
}

#[test]
fn unicode_surrogate_pair_decodes_through_public_api() {
    let value = parse(br#"{"emoji":"\uD83D\uDE80"}"#).expect("unicode payload should parse");
    assert_eq!(value.required_str("emoji"), Ok("🚀"));
}

fn finite_number(value: f64) -> JsonValue {
    JsonNumber::from_f64(value).unwrap().into()
}

#[test]
fn roundtrip_preserves_complex_value_equivalence() {
    let value = JsonValue::Object(vec![
        (
            "meta".into(),
            JsonValue::Object(vec![
                ("version".into(), finite_number(1.0)),
                ("cursor".into(), JsonValue::Null),
            ]),
        ),
        (
            "items".into(),
            JsonValue::Array(vec![
                JsonValue::Object(vec![
                    ("id".into(), JsonValue::String("one".into())),
                    ("enabled".into(), JsonValue::Bool(true)),
                ]),
                JsonValue::Object(vec![
                    ("id".into(), JsonValue::String("two".into())),
                    ("enabled".into(), JsonValue::Bool(false)),
                    ("score".into(), finite_number(-1.5e-3)),
                ]),
            ]),
        ),
        (
            "message".into(),
            JsonValue::String("hello\u{0000}world".into()),
        ),
    ]);

    let encoded = encode(&value).expect("encode should succeed");
    let reparsed = parse(&encoded).expect("encoded value should parse");
    assert_eq!(reparsed, value);
    let items = reparsed
        .required_array("items")
        .expect("items should be array");
    assert_eq!(items.len(), 2);
}

#[test]
fn owned_numbers_preserve_valid_json_spelling() {
    for input in [
        "0",
        "-0",
        "-12",
        "3.5",
        "1e2",
        "1E+10",
        "1e-10",
        "-0.0",
        "9007199254740993",
        "18446744073709551615",
        "1e400",
    ] {
        let value = parse(input.as_bytes()).unwrap();
        let number = value.as_number().unwrap();
        assert_eq!(number.raw(), input);
        assert_eq!(encode(&value).unwrap(), input.as_bytes());
        assert_eq!(parse(&encode(&value).unwrap()).unwrap(), value);
    }

    assert_eq!(
        parse(b"9007199254740993")
            .unwrap()
            .as_number()
            .unwrap()
            .as_u64(),
        Some(9_007_199_254_740_993)
    );
    assert_eq!(parse(b"1e400").unwrap().as_number().unwrap().as_f64(), None);
}

#[test]
fn depth_limit_is_enforced() {
    let mut input = vec![b'['; 129];
    input.extend(vec![b']'; 129]);
    let error = parse(&input).expect_err("depth overflow should fail");
    assert!(matches!(
        error.kind,
        neco_json::ParseErrorKind::NestingTooDeep
    ));
}

#[test]
fn end_to_end_atproto_like_payload_parses_and_accesses() {
    let response = br#"{
        "did":"did:plc:alice",
        "handle":"alice.test",
        "active":true,
        "viewer":{"muted":false,"blockedBy":null},
        "followersCount":128,
        "labels":[{"val":"verified","src":"did:plc:labeler"}]
    }"#;

    let value = parse(response).expect("ATProto-like response should parse");
    assert_eq!(value.required_str("did"), Ok("did:plc:alice"));
    assert_eq!(value.required_str("handle"), Ok("alice.test"));
    assert_eq!(value.required_bool("active"), Ok(true));
    assert_eq!(value.required_f64("followersCount"), Ok(128.0));

    let viewer = value.get("viewer").expect("viewer should exist");
    assert_eq!(viewer.required_bool("muted"), Ok(false));
    assert_eq!(viewer.optional_bool("blockedBy"), Ok(None));

    let labels = value
        .required_array("labels")
        .expect("labels should be array");
    assert_eq!(labels.len(), 1);
    let label = labels[0].as_object().expect("label should be object");
    let label = JsonValue::Object(label.to_vec());
    assert_eq!(label.required_str("val"), Ok("verified"));
    assert_eq!(label.required_str("src"), Ok("did:plc:labeler"));
}
