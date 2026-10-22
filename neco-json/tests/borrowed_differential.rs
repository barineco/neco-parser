use neco_json::{parse, parse_borrowed, JsonKind, JsonRef, JsonValue};

fn compare(owned: &JsonValue, view: JsonRef<'_>) {
    match owned {
        JsonValue::Null => assert_eq!(view.kind(), JsonKind::Null),
        JsonValue::Bool(value) => assert_eq!(view.as_bool(), Some(*value)),
        JsonValue::Number(value) => {
            assert_eq!(view.kind(), JsonKind::Number);
            assert_eq!(view.raw(), value.raw());
            assert_eq!(view.as_u64(), value.as_u64());
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
fn owned_and_borrowed_views_preserve_the_same_tokens() {
    for input in [
        "null",
        "true",
        "false",
        "0",
        "-0",
        "-13",
        "3.5",
        "1e2",
        "1E+10",
        "1e-10",
        "-0.0",
        "9007199254740993",
        "18446744073709551615",
        "1e400",
        "\"é\\n\\uD83D\\uDE00\"",
        "[null,true,false,0,-0,42,9007199254740993,\"水\"]",
        " {\"same\":0,\"same\":{\"nested\":[1,2,\"\\u00e9\"]}} ",
    ] {
        compare(
            &parse(input.as_bytes()).unwrap(),
            parse_borrowed(input).unwrap(),
        );
    }
}
