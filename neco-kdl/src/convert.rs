/// Format-agnostic value type for KDL conversion.
///
/// This enum serves as an intermediate representation between KDL documents
/// and other data formats (JSON, CBOR, etc.) without requiring external
/// dependencies in the neco-kdl crate.
///
/// Integer(i64) and Float(f64) are kept distinct, unlike JSON's single Number(f64).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    /// Order-preserving map of key-value pairs.
    Object(Vec<(String, Value)>),
}

use crate::{KdlDocument, KdlEntry, KdlNode, KdlNumber, KdlValue};

#[derive(Debug, Clone, PartialEq)]
/// Failure returned when a KDL document and `Value` cannot be converted losslessly.
pub struct ValueError {
    /// The conversion rule that rejected the input.
    pub reason: ValueErrorReason,
}

impl ValueError {
    fn new(reason: ValueErrorReason) -> Self {
        Self { reason }
    }

    /// Returns the conversion rule that rejected the input.
    pub fn reason(&self) -> &ValueErrorReason {
        &self.reason
    }
}

impl core::fmt::Display for ValueError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "KDL value conversion failed: {:?}", self.reason)
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Clone, PartialEq)]
/// Explicit reasons that a KDL document and `Value` cannot be converted losslessly.
pub enum ValueErrorReason {
    TopLevelMustBeObject,
    NestedCollectionInArray,
    EmptyArrayCannotBeConverted,
    PropertyCannotBeConverted,
    MixedArgumentsAndChildren,
    UnsupportedTypeAnnotation(String),
    NumberCannotBeConverted(String),
}

/// Converts a `Value` into a `KdlDocument`.
///
/// Only `Value::Object` can be represented as a KDL document (since KDL is a
/// collection of named nodes). Each key in the object becomes a node name.
///
/// Conversion rules:
/// - Primitive values (Bool/Integer/Float/String/Null) → single positional argument
/// - Array values → multiple positional arguments
/// - Nested Object values → children block
///
/// # Errors
///
/// Returns an error for a non-Object top level, an empty Array, a nested
/// collection inside an Array, or a number that cannot be constructed.
pub fn value_to_kdl_document(value: &Value) -> Result<KdlDocument, ValueError> {
    match value {
        Value::Object(fields) => {
            let nodes = fields
                .iter()
                .map(|(key, val)| value_to_kdl_node(key, val))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(KdlDocument { nodes })
        }
        _ => Err(ValueError::new(ValueErrorReason::TopLevelMustBeObject)),
    }
}

/// Converts a `KdlDocument` back into a `Value`.
///
/// The document is always decoded as a `Value::Object` where each node name
/// becomes a key.
///
/// Decoding rules for node arguments:
/// - No positional arguments and no children → `Value::Null`
/// - Exactly one positional argument, no children → scalar value
/// - Two or more positional arguments, no children → `Value::Array`
/// - Children block (no positional arguments) → nested `Value::Object`
///
/// # Errors
///
/// Returns an error if a node cannot be decoded (e.g., mixed arguments and
/// children, or an unrecognised value type).
pub fn kdl_document_to_value(doc: &KdlDocument) -> Result<Value, ValueError> {
    let mut fields = Vec::with_capacity(doc.nodes.len());
    for node in &doc.nodes {
        let val = kdl_node_to_value(node)?;
        fields.push((node.name.clone(), val));
    }
    Ok(Value::Object(fields))
}

/// A sentinel type-annotation string used to distinguish `Value::Float` from
/// `Value::Integer` when both are stored as KDL numbers.
///
/// KDL's `KdlNumber` stores a raw string; we use a `(f64)` type annotation on
/// the argument to mark that the original value was a `Value::Float`.
const FLOAT_TYPE: &str = "f64";

/// Converts a single key-value pair into a `KdlNode`.
fn value_to_kdl_node(key: &str, value: &Value) -> Result<KdlNode, ValueError> {
    match value {
        Value::Object(fields) => {
            let children = fields
                .iter()
                .map(|(k, v)| value_to_kdl_node(k, v))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(KdlNode {
                ty: None,
                name: key.to_string(),
                entries: Vec::new(),
                children: Some(children),
            })
        }

        Value::Array(items) => {
            if items.is_empty() {
                return Err(ValueError::new(
                    ValueErrorReason::EmptyArrayCannotBeConverted,
                ));
            }
            let entries = items
                .iter()
                .map(primitive_to_argument)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(KdlNode {
                ty: None,
                name: key.to_string(),
                entries,
                children: None,
            })
        }

        _ => {
            let entry = primitive_to_argument(value)?;
            Ok(KdlNode {
                ty: None,
                name: key.to_string(),
                entries: vec![entry],
                children: None,
            })
        }
    }
}

/// Converts a primitive `Value` (non-Array, non-Object) into a `KdlEntry::Argument`.
///
/// `Value::Float` gets a `(f64)` type annotation so that round-trip decoding
/// can distinguish it from `Value::Integer`.
fn primitive_to_argument(value: &Value) -> Result<KdlEntry, ValueError> {
    match value {
        Value::Null => Ok(KdlEntry::Argument {
            ty: None,
            value: KdlValue::Null,
        }),
        Value::Bool(b) => Ok(KdlEntry::Argument {
            ty: None,
            value: KdlValue::Bool(*b),
        }),
        Value::Integer(i) => Ok(KdlEntry::Argument {
            ty: None,
            value: KdlValue::Number(i64_to_kdl_number(*i)?),
        }),
        Value::Float(f) => Ok(KdlEntry::Argument {
            ty: Some(FLOAT_TYPE.to_string()),
            value: KdlValue::Number(f64_to_kdl_number(*f)?),
        }),
        Value::String(s) => Ok(KdlEntry::Argument {
            ty: None,
            value: KdlValue::String(s.clone()),
        }),
        Value::Array(_) | Value::Object(_) => {
            Err(ValueError::new(ValueErrorReason::NestedCollectionInArray))
        }
    }
}

/// Decodes a `KdlNode` into a `Value`.
fn kdl_node_to_value(node: &KdlNode) -> Result<Value, ValueError> {
    if node
        .entries
        .iter()
        .any(|entry| matches!(entry, KdlEntry::Property { .. }))
    {
        return Err(ValueError::new(ValueErrorReason::PropertyCannotBeConverted));
    }
    if let Some(ty) = &node.ty {
        return Err(ValueError::new(
            ValueErrorReason::UnsupportedTypeAnnotation(ty.clone()),
        ));
    }
    for entry in &node.entries {
        let KdlEntry::Argument {
            ty: Some(ty),
            value,
        } = entry
        else {
            continue;
        };
        if ty != FLOAT_TYPE || !matches!(value, KdlValue::Number(_)) {
            return Err(ValueError::new(
                ValueErrorReason::UnsupportedTypeAnnotation(ty.clone()),
            ));
        }
    }

    let args: Vec<&KdlEntry> = node
        .entries
        .iter()
        .filter(|e| matches!(e, KdlEntry::Argument { .. }))
        .collect();

    match (args.len(), node.children.as_ref()) {
        (0, None) => Ok(Value::Null),

        (0, Some(children)) => {
            let mut fields = Vec::with_capacity(children.len());
            for child in children {
                let v = kdl_node_to_value(child)?;
                fields.push((child.name.clone(), v));
            }
            Ok(Value::Object(fields))
        }

        (1, None) => kdl_entry_to_value(args[0]),

        (_, None) => {
            let items = args
                .iter()
                .map(|e| kdl_entry_to_value(e))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::Array(items))
        }

        (_, Some(_)) => Err(ValueError::new(ValueErrorReason::MixedArgumentsAndChildren)),
    }
}

/// Decodes a single `KdlEntry::Argument` into a scalar `Value`.
fn kdl_entry_to_value(entry: &KdlEntry) -> Result<Value, ValueError> {
    let (ty, kdl_val) = match entry {
        KdlEntry::Argument { ty, value } => (ty.as_deref(), value),
        KdlEntry::Property { .. } => {
            return Err(ValueError::new(ValueErrorReason::PropertyCannotBeConverted))
        }
    };

    match kdl_val {
        KdlValue::Null => Ok(Value::Null),
        KdlValue::Bool(b) => Ok(Value::Bool(*b)),
        KdlValue::String(s) => Ok(Value::String(s.clone())),
        KdlValue::Number(n) => {
            if ty == Some(FLOAT_TYPE) {
                match n.as_f64() {
                    Some(f) => Ok(Value::Float(f)),
                    None => Err(ValueError::new(ValueErrorReason::NumberCannotBeConverted(
                        n.raw().to_string(),
                    ))),
                }
            } else {
                match n.as_i64() {
                    Some(i) => Ok(Value::Integer(i)),
                    None => match n.as_f64() {
                        Some(f) => Ok(Value::Float(f)),
                        None => Err(ValueError::new(ValueErrorReason::NumberCannotBeConverted(
                            n.raw().to_string(),
                        ))),
                    },
                }
            }
        }
    }
}

fn i64_to_kdl_number(i: i64) -> Result<KdlNumber, ValueError> {
    let raw = i.to_string();
    KdlNumber::new(raw.clone(), Some(i), Some(i as f64))
        .map_err(|_| ValueError::new(ValueErrorReason::NumberCannotBeConverted(raw)))
}

fn f64_to_kdl_number(f: f64) -> Result<KdlNumber, ValueError> {
    let raw = format_f64(f);
    KdlNumber::new(raw.clone(), None, Some(f))
        .map_err(|_| ValueError::new(ValueErrorReason::NumberCannotBeConverted(raw)))
}

/// Formats an f64 as a KDL-parseable decimal string with an explicit dot.
fn format_f64(f: f64) -> String {
    if f.is_nan() {
        return "#nan".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 {
            "#inf".to_string()
        } else {
            "#-inf".to_string()
        };
    }

    let s = format!("{f}");
    if s.contains('.') || s.contains('e') || s.contains('E') || s.starts_with('#') {
        s
    } else {
        format!("{s}.0")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(v: &Value) -> Value {
        let doc = value_to_kdl_document(v).expect("encode failed");
        kdl_document_to_value(&doc).expect("decode failed")
    }

    #[test]
    fn roundtrip_single_string_node() {
        let input = Value::Object(vec![(
            "text".to_string(),
            Value::String("hello".to_string()),
        )]);
        let output = roundtrip(&input);
        assert_eq!(input, output);
    }

    #[test]
    fn roundtrip_array_node() {
        let input = Value::Object(vec![(
            "langs".to_string(),
            Value::Array(vec![
                Value::String("en".to_string()),
                Value::String("ja".to_string()),
            ]),
        )]);
        let output = roundtrip(&input);
        assert_eq!(input, output);
    }

    #[test]
    fn roundtrip_nested_object() {
        let root_ref = Value::Object(vec![
            ("cid".to_string(), Value::String("bafyCID".to_string())),
            ("uri".to_string(), Value::String("at://x".to_string())),
        ]);
        let input = Value::Object(vec![(
            "reply".to_string(),
            Value::Object(vec![("root".to_string(), root_ref)]),
        )]);
        let output = roundtrip(&input);
        assert_eq!(input, output);
    }

    #[test]
    fn roundtrip_integer() {
        let input = Value::Object(vec![("count".to_string(), Value::Integer(42))]);
        let output = roundtrip(&input);
        assert_eq!(input, output);
        match &output {
            Value::Object(fields) => {
                assert_eq!(fields[0].1, Value::Integer(42));
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn roundtrip_float() {
        let input = Value::Object(vec![("ratio".to_string(), Value::Float(2.5))]);
        let output = roundtrip(&input);
        match &output {
            Value::Object(fields) => {
                if let Value::Float(f) = fields[0].1 {
                    assert!((f - 2.5_f64).abs() < 1e-10);
                } else {
                    panic!("expected Float, got {:?}", fields[0].1);
                }
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn non_finite_floats_are_accepted() {
        for (value, raw) in [
            (f64::INFINITY, "#inf"),
            (f64::NEG_INFINITY, "#-inf"),
            (f64::NAN, "#nan"),
        ] {
            let parsed = crate::parse(&format!("number {raw}")).unwrap();
            let decoded = kdl_document_to_value(&parsed).unwrap();
            let Value::Object(fields) = decoded else {
                panic!("expected object")
            };
            let Value::Float(decoded) = fields[0].1 else {
                panic!("expected float")
            };
            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded, value);
            }

            let input = Value::Object(vec![("number".to_string(), Value::Float(value))]);
            let document = value_to_kdl_document(&input).unwrap();
            let KdlEntry::Argument {
                value: KdlValue::Number(number),
                ..
            } = &document.nodes[0].entries[0]
            else {
                panic!("expected number")
            };
            assert_eq!(number.raw(), raw);

            let decoded = kdl_document_to_value(&document).unwrap();
            let Value::Object(fields) = decoded else {
                panic!("expected object")
            };
            let Value::Float(decoded) = fields[0].1 else {
                panic!("expected float")
            };
            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded, value);
            }
        }
    }

    #[test]
    fn integer_and_float_are_distinct() {
        let int_input = Value::Object(vec![("n".to_string(), Value::Integer(1))]);
        let flt_input = Value::Object(vec![("n".to_string(), Value::Float(1.0))]);

        let int_out = roundtrip(&int_input);
        let flt_out = roundtrip(&flt_input);

        assert_ne!(int_out, flt_out);

        match &int_out {
            Value::Object(f) => assert!(matches!(f[0].1, Value::Integer(_))),
            _ => panic!(),
        }
        match &flt_out {
            Value::Object(f) => assert!(matches!(f[0].1, Value::Float(_))),
            _ => panic!(),
        }
    }

    #[test]
    fn roundtrip_null() {
        let input = Value::Object(vec![("deleted".to_string(), Value::Null)]);
        let output = roundtrip(&input);
        assert_eq!(input, output);
    }

    #[test]
    fn roundtrip_bool() {
        let input = Value::Object(vec![
            ("active".to_string(), Value::Bool(true)),
            ("deleted".to_string(), Value::Bool(false)),
        ]);
        let output = roundtrip(&input);
        assert_eq!(input, output);
    }

    #[test]
    fn roundtrip_negative_integer() {
        let input = Value::Object(vec![("offset".to_string(), Value::Integer(-7))]);
        let output = roundtrip(&input);
        assert_eq!(input, output);
    }

    #[test]
    fn top_level_non_object_is_error() {
        let result = value_to_kdl_document(&Value::String("oops".to_string()));
        assert_eq!(
            result.unwrap_err().reason,
            ValueErrorReason::TopLevelMustBeObject
        );
    }

    #[test]
    fn roundtrip_via_text() {
        use crate::{normalize, parse};

        let input = Value::Object(vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Integer(30)),
            ("score".to_string(), Value::Float(9.5)),
            ("active".to_string(), Value::Bool(true)),
            (
                "meta".to_string(),
                Value::Object(vec![(
                    "role".to_string(),
                    Value::String("admin".to_string()),
                )]),
            ),
        ]);

        let doc = value_to_kdl_document(&input).unwrap();
        let text = normalize(&doc);
        let doc2 = parse(&text).unwrap();
        let output = kdl_document_to_value(&doc2).unwrap();

        match (&input, &output) {
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len());
                assert_eq!(a[0], b[0]); // name
                assert_eq!(a[1], b[1]); // age
                if let (Value::Float(fa), Value::Float(fb)) = (&a[2].1, &b[2].1) {
                    assert!((fa - fb).abs() < 1e-10);
                } else {
                    panic!("expected Float for score");
                }
                assert_eq!(a[3], b[3]); // active
                assert_eq!(a[4], b[4]); // meta
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn empty_node_decoded_as_null() {
        let doc = KdlDocument {
            nodes: vec![KdlNode {
                ty: None,
                name: "empty".to_string(),
                entries: vec![],
                children: None,
            }],
        };
        let val = kdl_document_to_value(&doc).unwrap();
        assert_eq!(val, Value::Object(vec![("empty".to_string(), Value::Null)]));
    }

    #[test]
    fn empty_object_roundtrips_through_structure_and_text() {
        use crate::{parse, serialize};

        let input = Value::Object(vec![("empty".to_string(), Value::Object(vec![]))]);
        let doc = value_to_kdl_document(&input).unwrap();
        assert_eq!(doc.nodes[0].children, Some(vec![]));
        assert_eq!(kdl_document_to_value(&doc).unwrap(), input);

        let text = serialize(&doc);
        assert_eq!(text, "empty {\n}\n");
        let reparsed = parse(&text).unwrap();
        assert_eq!(reparsed.nodes[0].children, Some(vec![]));
        assert_eq!(kdl_document_to_value(&reparsed).unwrap(), input);
    }

    #[test]
    fn empty_array_is_rejected() {
        let input = Value::Object(vec![("empty".to_string(), Value::Array(vec![]))]);
        assert_eq!(
            value_to_kdl_document(&input).unwrap_err().reason,
            ValueErrorReason::EmptyArrayCannotBeConverted
        );
    }

    #[test]
    fn nested_collections_in_array_are_rejected() {
        for nested in [Value::Array(vec![]), Value::Object(vec![])] {
            let input = Value::Object(vec![(
                "items".to_string(),
                Value::Array(vec![Value::Integer(1), nested]),
            )]);
            assert_eq!(
                value_to_kdl_document(&input).unwrap_err().reason,
                ValueErrorReason::NestedCollectionInArray
            );
        }
    }

    #[test]
    fn properties_are_rejected_with_and_without_arguments() {
        for input in ["node key=1", "node 1 key=2"] {
            let doc = crate::parse(input).unwrap();
            assert_eq!(
                kdl_document_to_value(&doc).unwrap_err().reason,
                ValueErrorReason::PropertyCannotBeConverted
            );
        }
    }

    #[test]
    fn unsupported_type_annotations_are_rejected() {
        for (input, expected) in [
            (
                "(node_ty)node 1",
                ValueErrorReason::UnsupportedTypeAnnotation("node_ty".to_string()),
            ),
            (
                "node (entry_ty)1",
                ValueErrorReason::UnsupportedTypeAnnotation("entry_ty".to_string()),
            ),
            (
                "node (f64)\"text\"",
                ValueErrorReason::UnsupportedTypeAnnotation("f64".to_string()),
            ),
        ] {
            let doc = crate::parse(input).unwrap();
            assert_eq!(kdl_document_to_value(&doc).unwrap_err().reason, expected);
        }
    }

    #[test]
    fn mixed_arguments_and_children_are_rejected() {
        let doc = crate::parse("node 1 { child }").unwrap();
        assert_eq!(
            kdl_document_to_value(&doc).unwrap_err().reason,
            ValueErrorReason::MixedArgumentsAndChildren
        );
    }

    #[test]
    fn number_decode_preserves_supported_values_and_rejects_uninterpreted_raw() {
        let hex = crate::parse("node 0xff").unwrap();
        assert_eq!(
            kdl_document_to_value(&hex).unwrap(),
            Value::Object(vec![("node".to_string(), Value::Integer(255))])
        );

        let infinity = crate::parse("node #inf").unwrap();
        let decoded = kdl_document_to_value(&infinity).unwrap();
        let Value::Object(fields) = decoded else {
            panic!("expected object")
        };
        let Value::Float(value) = fields[0].1 else {
            panic!("expected float")
        };
        assert!(value.is_infinite() && value > 0.0);

        let invalid = KdlDocument {
            nodes: vec![KdlNode {
                ty: None,
                name: "node".to_string(),
                entries: vec![KdlEntry::Argument {
                    ty: None,
                    value: KdlValue::Number(KdlNumber {
                        raw: "999999999999999999999999999999999999999999".to_string(),
                        as_i64: None,
                        as_f64: None,
                    }),
                }],
                children: None,
            }],
        };
        assert_eq!(
            kdl_document_to_value(&invalid).unwrap_err().reason,
            ValueErrorReason::NumberCannotBeConverted(
                "999999999999999999999999999999999999999999".to_string()
            )
        );
    }

    #[test]
    fn rejection_priority_is_deterministic() {
        let argument = KdlEntry::Argument {
            ty: Some("entry_ty".to_string()),
            value: KdlValue::Number(KdlNumber {
                raw: "1".to_string(),
                as_i64: Some(1),
                as_f64: Some(1.0),
            }),
        };
        let property = KdlEntry::Property {
            key: "key".to_string(),
            ty: None,
            value: KdlValue::Bool(true),
        };
        let child = KdlNode {
            ty: None,
            name: "child".to_string(),
            entries: vec![],
            children: None,
        };
        let reason = |ty, entries| {
            let doc = KdlDocument {
                nodes: vec![KdlNode {
                    ty,
                    name: "node".to_string(),
                    entries,
                    children: Some(vec![child.clone()]),
                }],
            };
            kdl_document_to_value(&doc).unwrap_err().reason
        };

        assert_eq!(
            reason(
                Some("node_ty".to_string()),
                vec![argument.clone(), property]
            ),
            ValueErrorReason::PropertyCannotBeConverted
        );
        assert_eq!(
            reason(Some("node_ty".to_string()), vec![argument.clone()]),
            ValueErrorReason::UnsupportedTypeAnnotation("node_ty".to_string())
        );
        assert_eq!(
            reason(None, vec![argument.clone()]),
            ValueErrorReason::UnsupportedTypeAnnotation("entry_ty".to_string())
        );
        let untyped = KdlEntry::Argument {
            ty: None,
            value: argument.value().clone(),
        };
        assert_eq!(
            reason(None, vec![untyped]),
            ValueErrorReason::MixedArgumentsAndChildren
        );
    }
}
