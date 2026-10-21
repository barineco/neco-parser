use neco_json::{parse, parse_borrowed, JsonKind, JsonRef, JsonValue};

fn compare(owned: &JsonValue, view: JsonRef<'_>) {
    match owned {
        JsonValue::Null => assert_eq!(view.kind(), JsonKind::Null),
        JsonValue::Bool(value) => assert_eq!(view.as_bool(), Some(*value)),
        JsonValue::Number(value) => {
            assert_eq!(view.kind(), JsonKind::Number);
            assert_eq!(
                view.raw().parse::<f64>().unwrap().to_bits(),
                value.to_bits()
            );
            if view.raw().bytes().all(|byte| byte.is_ascii_digit()) {
                assert_eq!(view.as_u64(), Some(*value as u64));
            } else {
                assert_eq!(view.as_u64(), None);
            }
        }
        JsonValue::String(value) => assert_eq!(view.as_str().unwrap().unwrap(), value.as_str()),
        JsonValue::Array(values) => {
            let items: Vec<_> = view
                .array_items()
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(values.len(), items.len());
            for (owned, view) in values.iter().zip(items) {
                compare(owned, view);
            }
        }
        JsonValue::Object(fields) => {
            let entries: Vec<_> = view
                .object_entries()
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(fields.len(), entries.len());
            for ((key, value), (key_view, value_view)) in fields.iter().zip(entries) {
                assert_eq!(key_view.as_str().unwrap().unwrap(), key.as_str());
                compare(value, value_view);
            }
        }
    }
}

#[test]
fn exact_common_domain_preserves_both_public_representations() {
    for input in [
        "null",
        "true",
        "false",
        "0",
        "-0",
        "-13",
        "3.5",
        "9007199254740992",
        "\"é\\n\\uD83D\\uDE00\"",
        "[null,true,false,0,-0,42,9007199254740992,\"水\"]",
        " {\"same\":0,\"same\":{\"nested\":[1,2,\"\\u00e9\"]}} ",
    ] {
        compare(
            &parse(input.as_bytes()).unwrap(),
            parse_borrowed(input).unwrap(),
        );
    }
}
