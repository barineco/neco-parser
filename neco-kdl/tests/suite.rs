use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn sha256(input: &[u8]) -> [u8; 32] {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const ROUND: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (input.len() as u64) * 8;
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut hash = INITIAL;
    for block in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (word, bytes) in words.iter_mut().zip(block.chunks_exact(4)) {
            *word = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = hash;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ (!e & g);
            let temporary1 = h
                .wrapping_add(sum1)
                .wrapping_add(choose)
                .wrapping_add(ROUND[index])
                .wrapping_add(words[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temporary2 = sum0.wrapping_add(majority);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temporary1);
            d = c;
            c = b;
            b = a;
            a = temporary1.wrapping_add(temporary2);
        }

        for (slot, value) in hash.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }

    let mut digest = [0_u8; 32];
    for (bytes, word) in digest.chunks_exact_mut(4).zip(hash) {
        bytes.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn sha256_hex(input: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in sha256(input) {
        write!(output, "{byte:02x}").unwrap();
    }
    output
}

fn vector_paths(base_dir: &Path) -> BTreeSet<String> {
    ["input", "expected_kdl"]
        .into_iter()
        .flat_map(|directory| {
            fs::read_dir(base_dir.join(directory))
                .unwrap()
                .map(move |entry| (directory, entry.unwrap().path()))
        })
        .filter(|(_, path)| {
            path.extension().and_then(|extension| extension.to_str()) == Some("kdl")
        })
        .map(|(directory, path)| {
            format!(
                "{directory}/{}",
                path.file_name().unwrap().to_str().unwrap()
            )
        })
        .collect()
}

#[test]
fn manifest_sha256_matches_known_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn kdl_official_test_suite_manifest() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kdl-test-cases");
    let manifest = fs::read_to_string(base_dir.join("manifest.sha256")).unwrap();
    let mut manifest_paths = BTreeSet::new();
    let mut manifest_entries = Vec::new();

    for (index, line) in manifest.lines().enumerate() {
        let line_number = index + 1;
        let (digest, relative_path) = line
            .split_once("  ")
            .unwrap_or_else(|| panic!("manifest line {line_number} has an invalid separator"));
        assert_eq!(
            digest.len(),
            64,
            "manifest line {line_number} has an invalid digest length"
        );
        assert!(
            digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "manifest line {line_number} has an invalid digest"
        );
        assert!(
            (relative_path.starts_with("input/") || relative_path.starts_with("expected_kdl/"))
                && relative_path.ends_with(".kdl")
                && relative_path.matches('/').count() == 1
                && !relative_path.contains('\\'),
            "manifest line {line_number} has an invalid path"
        );
        assert!(
            manifest_paths.insert(relative_path.to_string()),
            "manifest line {line_number} duplicates {relative_path}"
        );
        manifest_entries.push((relative_path.to_string(), digest.to_string()));
    }

    let files = vector_paths(&base_dir);
    let missing_files: Vec<_> = manifest_paths.difference(&files).collect();
    let unregistered_files: Vec<_> = files.difference(&manifest_paths).collect();
    assert!(
        missing_files.is_empty(),
        "manifest paths missing from vector files: {missing_files:?}"
    );
    assert!(
        unregistered_files.is_empty(),
        "vector files missing from manifest: {unregistered_files:?}"
    );

    for (relative_path, digest) in manifest_entries {
        let contents = fs::read(base_dir.join(&relative_path)).unwrap();
        assert_eq!(
            sha256_hex(&contents),
            digest,
            "SHA-256 mismatch for {relative_path}"
        );
    }
}

#[test]
fn negative_zero_integer_values_and_normalization() {
    for raw in ["-0", "-0x0", "-0o0", "-0b0", "-0x00", "0x0"] {
        let document = neco_kdl::parse(&format!("node {raw}\n")).unwrap();
        assert_eq!(
            document.nodes()[0].first_arg().unwrap().as_i64(),
            Some(0),
            "{raw}"
        );
        assert_eq!(neco_kdl::normalize(&document), "node 0\n", "{raw}");
    }
}

#[test]
fn negative_zero_float_normalization_preserves_sign() {
    for (raw, expected) in [("-0.0", "node -0.0\n"), ("-0e5", "node -0E+5\n")] {
        let document = neco_kdl::parse(&format!("node {raw}\n")).unwrap();
        assert_eq!(neco_kdl::normalize(&document), expected, "{raw}");
    }
}

#[test]
fn duplicate_property_access_uses_rightmost_value() {
    let document = neco_kdl::parse(
        "node value=1 flag=#false text=\"first\" value=2 flag=#true text=\"second\"\n",
    )
    .unwrap();
    let node = &document.nodes()[0];
    assert_eq!(node.int_prop("value"), Some(2));
    assert_eq!(node.bool_prop("flag"), Some(true));
    assert_eq!(node.string_prop("text"), Some("second"));
    let properties: Vec<_> = node
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            neco_kdl::KdlEntry::Property { key, value, .. } => Some((
                key.as_str(),
                value.as_i64(),
                value.as_bool(),
                value.as_str(),
            )),
            _ => None,
        })
        .collect();
    assert_eq!(
        properties,
        [
            ("value", Some(1), None, None),
            ("flag", None, Some(false), None),
            ("text", None, None, Some("first")),
            ("value", Some(2), None, None),
            ("flag", None, Some(true), None),
            ("text", None, None, Some("second")),
        ]
    );
}

#[test]
fn serialize_escape_branches_match_exact_output() {
    let document = neco_kdl::KdlDocument {
        nodes: vec![neco_kdl::KdlNode {
            ty: None,
            name: "node".to_string(),
            entries: vec![neco_kdl::KdlEntry::Argument {
                ty: None,
                value: neco_kdl::KdlValue::String(
                    "plain\"\\\n\r\t\u{0008}\u{000C}\u{000B}tail".to_string(),
                ),
            }],
            children: None,
        }],
    };

    assert_eq!(
        neco_kdl::serialize(&document),
        "node \"plain\\\"\\\\\\n\\r\\t\\b\\f\\u{000B}tail\"\n"
    );
}

#[test]
fn kdl_official_test_suite_parse() {
    let input_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kdl-test-cases/input");

    let mut pass = 0;
    let mut fail = 0;
    let mut errors = Vec::new();
    let mut hangs = Vec::new();

    let mut files: Vec<_> = fs::read_dir(&input_dir)
        .expect("input dir should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("kdl"))
        .collect();
    files.sort();

    let known_skip: [&str; 0] = [];
    let mut skipped = 0;

    for path in &files {
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        if known_skip.contains(&stem.as_str()) {
            skipped += 1;
            continue;
        }
        let should_fail = stem.ends_with("_fail");
        let input = fs::read_to_string(path).unwrap();

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = neco_kdl::parse(&input);
            let _ = tx.send(result.map(|_| ()).map_err(|e| format!("{e}")));
        });

        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => {
                if should_fail {
                    errors.push(format!("SHOULD FAIL but parsed OK: {stem}"));
                } else {
                    pass += 1;
                }
            }
            Ok(Err(e)) => {
                if should_fail {
                    fail += 1;
                } else {
                    errors.push(format!("SHOULD PASS but failed: {stem} : {e}"));
                }
            }
            Err(_) => {
                hangs.push(stem.clone());
            }
        }
    }

    println!(
        "pass: {pass}, fail (expected): {fail}, errors: {}, hangs: {}, skipped: {skipped}",
        errors.len(),
        hangs.len()
    );
    for e in &errors {
        println!("  {e}");
    }
    for h in &hangs {
        println!("  HANG: {h}");
    }
    let total_problems = errors.len() + hangs.len();
    if total_problems > 0 {
        panic!(
            "{} error(s) + {} hang(s) = {} total problem(s)",
            errors.len(),
            hangs.len(),
            total_problems
        );
    }
}

#[test]
fn official_success_vectors_roundtrip_through_serializer() {
    let input_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kdl-test-cases/input");
    let mut files: Vec<_> = fs::read_dir(input_dir)
        .expect("input dir should exist")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|extension| extension.to_str()) == Some("kdl")
                && !path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem.ends_with("_fail"))
        })
        .collect();
    files.sort();

    for path in files {
        let input = fs::read_to_string(&path).unwrap();
        let parsed =
            neco_kdl::parse(&input).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let serialized = neco_kdl::serialize(&parsed);
        let reparsed = neco_kdl::parse(&serialized)
            .unwrap_or_else(|error| panic!("{} serialized: {error}", path.display()));
        assert_eq!(reparsed, parsed, "{}", path.display());
    }
}

fn check_span_well_formedness(
    input: &str,
    node: &neco_kdl::SpannedKdlNode,
    label: &str,
) -> Result<(), String> {
    let node_span = node.span();
    if node_span.start >= node_span.end || node_span.end > input.len() {
        return Err(format!(
            "{label}: node span {node_span:?} is not a valid range"
        ));
    }
    if !input.is_char_boundary(node_span.start) || !input.is_char_boundary(node_span.end) {
        return Err(format!(
            "{label}: node span {node_span:?} is not on char boundaries"
        ));
    }

    for entry in node.entries() {
        let entry_span = entry.span();
        if entry_span.start >= entry_span.end {
            return Err(format!(
                "{label}: entry span {entry_span:?} is not a valid range"
            ));
        }
        if !input.is_char_boundary(entry_span.start) || !input.is_char_boundary(entry_span.end) {
            return Err(format!(
                "{label}: entry span {entry_span:?} is not on char boundaries"
            ));
        }
        if entry_span.start < node_span.start || entry_span.end > node_span.end {
            return Err(format!(
                "{label}: entry span {entry_span:?} is not contained in node span {node_span:?}"
            ));
        }
    }

    for child in node.children().unwrap_or(&[]) {
        let child_span = child.span();
        if child_span.start < node_span.start || child_span.end > node_span.end {
            return Err(format!(
                "{label}: child span {child_span:?} is not contained in node span {node_span:?}"
            ));
        }
        check_span_well_formedness(input, child, label)?;
    }

    Ok(())
}

#[test]
fn official_vectors_yield_well_formed_spans() {
    let input_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kdl-test-cases/input");

    let mut files: Vec<PathBuf> = fs::read_dir(&input_dir)
        .expect("input dir should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("kdl"))
        .collect();
    files.sort();

    let mut checked = 0;
    let mut errors = Vec::new();

    for path in &files {
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        if stem.ends_with("_fail") {
            continue;
        }
        let input = fs::read_to_string(path).unwrap();

        let document = match neco_kdl::parse_spanned(&input) {
            Ok(document) => document,
            Err(_) => continue,
        };

        for node in document.nodes() {
            if let Err(message) = check_span_well_formedness(&input, node, &stem) {
                errors.push(message);
            }
        }
        checked += 1;
    }

    assert!(checked > 0, "no accepted vectors were checked");
    assert!(errors.is_empty(), "span well-formedness errors: {errors:?}");
}

#[test]
fn kdl_official_test_suite_normalize() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/kdl-test-cases");
    let expected_dir = base_dir.join("expected_kdl");
    let input_dir = base_dir.join("input");

    let mut expected_files: Vec<PathBuf> = fs::read_dir(&expected_dir)
        .expect("expected_kdl dir should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("kdl"))
        .collect();
    expected_files.sort();

    let mut pass = 0;
    let mut errors = Vec::new();

    for expected_path in &expected_files {
        let stem = expected_path.file_stem().unwrap().to_str().unwrap();
        let input_path = input_dir.join(format!("{stem}.kdl"));

        if !input_path.exists() {
            continue;
        }

        let input = fs::read_to_string(&input_path).unwrap();
        let expected = fs::read_to_string(expected_path).unwrap();

        let doc = match neco_kdl::parse(&input) {
            Ok(doc) => doc,
            Err(e) => {
                errors.push(format!("PARSE FAIL: {stem} : {e}"));
                continue;
            }
        };

        let normalized = neco_kdl::normalize(&doc);

        if normalized == expected {
            pass += 1;
        } else {
            let mut diff = String::new();
            diff.push_str(&format!("  MISMATCH: {stem}\n"));
            diff.push_str(&format!("    expected: {:?}\n", expected));
            diff.push_str(&format!("    got:      {:?}\n", normalized));
            errors.push(diff);
        }
    }

    println!(
        "normalize: pass: {pass}, errors: {}, total: {}",
        errors.len(),
        pass + errors.len()
    );
    for e in &errors[..errors.len().min(10)] {
        println!("{e}");
    }
    if errors.len() > 10 {
        println!("  ... and {} more", errors.len() - 10);
    }
    if !errors.is_empty() {
        panic!("{} normalize mismatch(es)", errors.len());
    }
}
