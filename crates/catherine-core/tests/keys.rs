//! Purpose: the declarative-keys parse contract (format 1.3, openspec
//! change `add-declarative-keys`): `schema.keys` round-trips and
//! canonicalizes, malformed keys are rejected at their nodes, absent and
//! empty key lists are hash-neutral, and the published artifact covers
//! the section's syntactic surface.
//!
//! Responsibilities: pin the canonical `(object, arrows, status)` order,
//! the `invalid_key` defect classes with their offending nodes, the
//! `invalid_status` reuse, the version gate (1.3 known, 1.4 rejected),
//! and the hash-stability guard (keys are omitted from canonical bytes
//! when empty, so every pre-1.3 identity is untouched).
//!
//! Rationale: keys are schema constraints, not presentation (design
//! Decision 1): their presence must move the model hash, their absence
//! must not. Assertions are structural; the only byte-level pins are the
//! canonical-omission guard and the artifact's accept/reject verdicts.

use catherine_core::canonical::{canonical_bytes, serialize};
use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::hash::model_hash;
use catherine_core::model::{ConstraintStatus, Ident, Key};
use catherine_core::parse::parse;
use serde_json::{json, Value};

/// A 1.3 document with a `keys` placeholder and one arrow per object.
fn key_document(keys: &str) -> String {
    format!(
        r#"{{
  "format_version": "1.3",
  "schema": {{
    "objects": ["capture", "value"],
    "arrows": [
      {{"name": "capture_source", "src": "capture", "tgt": "value"}},
      {{"name": "capture_edition", "src": "capture", "tgt": "value"}}
    ],
    "equations": [],
    "keys": [
      {keys}
    ]
  }},
  "instance": {{
    "rows": {{"capture": ["c1"], "value": ["v1"]}},
    "tables": {{"capture_source": {{"c1": "v1"}}, "capture_edition": {{"c1": "v1"}}}}
  }}
}}"#
    )
}

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

fn key(object: &str, arrows: &[&str], status: ConstraintStatus) -> Key {
    Key {
        object: ident(object),
        arrows: arrows.iter().map(|arrow| ident(arrow)).collect(),
        status,
    }
}

/// Scenario "Keys round-trip and canonicalize": arbitrary input order is
/// presentation; the parsed schema carries the keys in canonical
/// `(object, arrows, status)` order and canonical bytes are independent
/// of the input order.
#[test]
fn keys_round_trip_and_canonicalize() {
    let first = parse(&key_document(
        r#"{"object": "capture", "arrows": ["capture_edition", "capture_source"], "status": "accepted"},
           {"object": "capture", "arrows": ["capture_edition"], "status": "proposed"},
           {"object": "capture", "arrows": ["capture_edition"], "status": "accepted"}"#,
    ))
    .expect("the 1.3 document parses");
    let second = parse(&key_document(
        r#"{"object": "capture", "arrows": ["capture_edition"], "status": "accepted"},
           {"object": "capture", "arrows": ["capture_edition", "capture_source"]},
           {"object": "capture", "arrows": ["capture_edition"], "status": "proposed"}"#,
    ))
    .expect("the reordered 1.3 document parses");
    assert_eq!(
        first.schema.keys,
        vec![
            key("capture", &["capture_edition"], ConstraintStatus::Accepted),
            key("capture", &["capture_edition"], ConstraintStatus::Proposed),
            key(
                "capture",
                &["capture_edition", "capture_source"],
                ConstraintStatus::Accepted
            ),
        ],
        "canonical order is (object, arrows, status); absent status defaults to accepted"
    );
    assert_eq!(first.schema.keys, second.schema.keys);
    assert_eq!(
        canonical_bytes(&first),
        canonical_bytes(&second),
        "input order never reaches the canonical bytes"
    );
    let reparsed = parse(&serialize(&first)).expect("canonical bytes parse");
    assert_eq!(reparsed, first, "serialize/parse is a fixed point");
}

/// Scenario "Malformed key is rejected at its node": every defect class is
/// `invalid_key` at the offending node, never a silent repair or drop.
#[test]
fn malformed_key_is_rejected_at_its_node() {
    // The key sits on line 11, indented six spaces; the columns are the
    // offending value's first character (pinned by hand).
    let cases: [(&str, usize, &str); 5] = [
        (
            r#"{"object": "C", "arrows": ["capture_source"]}"#,
            18,
            "undeclared object",
        ),
        (
            r#"{"object": "capture", "arrows": ["q"]}"#,
            40,
            "undeclared arrow",
        ),
        (
            r#"{"object": "capture", "arrows": ["g"]}"#,
            40,
            "arrow not sourced at the key object",
        ),
        (
            r#"{"object": "capture", "arrows": []}"#,
            39,
            "empty arrow list",
        ),
        (
            r#"{"object": "capture", "arrows": ["capture_source", "capture_source"]}"#,
            58,
            "duplicate arrow name",
        ),
    ];
    for (key_json, column, what) in cases {
        let single = key_document(key_json);
        let ParseError::Semantic(semantic) = parse(&single).expect_err(what) else {
            panic!("{what}: expected a semantic rejection");
        };
        assert_eq!(
            semantic.code,
            ErrorCode::InvalidKey,
            "{what}: invalid_key expected, got {:?} ({}:{})",
            semantic.code,
            semantic.line,
            semantic.column
        );
        assert_eq!(semantic.line, 11, "{what}: the key's line");
        assert_eq!(semantic.column, column, "{what}: the offending node");
        assert!(
            semantic.message.contains("key"),
            "{what}: the message names the key: {}",
            semantic.message
        );
    }
}

/// The "Ill-formed key" scenario's own gate: a duplicate arrow is
/// rejected at the second occurrence, and an undeclared object names no
/// key that was silently repaired.
#[test]
fn ill_formed_key() {
    let single =
        key_document(r#"{"object": "capture", "arrows": ["capture_source", "capture_source"]}"#);
    let ParseError::Semantic(semantic) = parse(&single).expect_err("duplicate arrow") else {
        panic!("a duplicate arrow is a semantic rejection");
    };
    assert_eq!(semantic.code, ErrorCode::InvalidKey);
    assert_eq!((semantic.line, semantic.column), (11, 58));
    assert!(
        semantic.message.contains("duplicate"),
        "the message names the defect: {}",
        semantic.message
    );
}

/// Scenario "Proposed key is preserved as a question": the key is parsed
/// like any other and the accepted-key set is untouched.
#[test]
fn proposed_key_is_preserved_as_a_question() {
    let model = parse(&key_document(
        r#"{"object": "capture", "arrows": ["capture_source"], "status": "accepted"},
           {"object": "capture", "arrows": ["capture_edition"], "status": "proposed"}"#,
    ))
    .expect("the document parses");
    assert_eq!(
        model.schema.keys,
        vec![
            key("capture", &["capture_edition"], ConstraintStatus::Proposed),
            key("capture", &["capture_source"], ConstraintStatus::Accepted),
        ]
    );
    let accepted: Vec<&Key> = model
        .schema
        .keys
        .iter()
        .filter(|key| key.status == ConstraintStatus::Accepted)
        .collect();
    assert_eq!(accepted.len(), 1, "the proposed key is not accepted");
}

/// Scenario "Absent keys are hash-neutral": a 1.2 document with no keys
/// hashes exactly like the same document carrying `"keys": []`, and the
/// empty section never reaches the canonical bytes.
#[test]
fn absent_keys_are_hash_neutral() {
    let without = r#"{
  "format_version": "1.2",
  "schema": {
    "objects": ["A"],
    "arrows": [{"name": "f", "src": "A", "tgt": "A"}],
    "equations": [{"lhs": ["f"], "rhs": ["f"], "status": "accepted"}]
  },
  "instance": {"rows": {"A": ["a"]}, "tables": {"f": {"a": "a"}}}
}"#;
    let with_empty = without.replace(
        "\"equations\": [{\"lhs\": [\"f\"], \"rhs\": [\"f\"], \"status\": \"accepted\"}]",
        "\"equations\": [{\"lhs\": [\"f\"], \"rhs\": [\"f\"], \"status\": \"accepted\"}], \"keys\": []",
    );
    let absent = parse(without).expect("the keyless document parses");
    let empty = parse(&with_empty).expect("the empty key list parses");
    assert!(empty.schema.keys.is_empty());
    assert_eq!(canonical_bytes(&absent), canonical_bytes(&empty));
    assert_eq!(model_hash(&absent), model_hash(&empty));
    assert!(
        !canonical_bytes(&empty).contains("\"keys\""),
        "an empty key list is omitted from canonical bytes"
    );

    let with_key = parse(&key_document(
        r#"{"object": "capture", "arrows": ["capture_source"]}"#,
    ))
    .expect("the keyed document parses");
    let keyless = parse(&key_document("")).expect("the keyless 1.3 document parses");
    assert_ne!(
        model_hash(&with_key),
        model_hash(&keyless),
        "a declared key moves the model hash"
    );
}

/// Scenario "Current major parses": 1.3 is known; 1.4 is not.
#[test]
fn current_major_parses_a_1_3_document() {
    let model = parse(&key_document(
        r#"{"object": "capture", "arrows": ["capture_source"]}"#,
    ))
    .expect("1.3 is the current known minor");
    assert_eq!(model.format_version.minor, 3);

    let too_new =
        key_document("").replace("\"format_version\": \"1.3\"", "\"format_version\": \"1.4\"");
    let ParseError::Semantic(semantic) = parse(&too_new).expect_err("1.4 is unknown") else {
        panic!("an unknown minor is a semantic rejection");
    };
    assert_eq!(semantic.code, ErrorCode::UnsupportedVersion);
    assert_eq!((semantic.line, semantic.column), (2, 21));
    assert!(
        semantic.message.contains("known version is 1.3"),
        "the message states the cap: {}",
        semantic.message
    );
}

/// Scenario "Editor-side validation": the published artifact covers the
/// key section's syntactic surface — a well-formed key passes, an empty
/// or duplicate `arrows` list is rejected without the kernel.
#[test]
fn keys_are_covered_by_the_artifact() {
    let artifact: Value =
        serde_json::from_str(include_str!("../schema/ct-schema-v1.json")).expect("artifact JSON");
    let validator = jsonschema::draft202012::new(&artifact).expect("the artifact is draft 2020-12");

    let base = json!({
        "format_version": "1.3",
        "schema": {
            "objects": ["capture", "value"],
            "arrows": [
                {"name": "capture_source", "src": "capture", "tgt": "value"}
            ],
            "equations": [],
            "keys": [{"object": "capture", "arrows": ["capture_source"]}]
        },
        "instance": {"rows": {"capture": [], "value": []}, "tables": {"capture_source": {}}}
    });
    assert!(
        validator.is_valid(&base),
        "a well-formed key passes the artifact: {:?}",
        validator
            .iter_errors(&base)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
    );

    let mut empty = base.clone();
    empty["schema"]["keys"][0]["arrows"] = json!([]);
    assert!(
        !validator.is_valid(&empty),
        "an empty arrow list is rejected"
    );

    let mut duplicate = base;
    duplicate["schema"]["keys"][0]["arrows"] = json!(["capture_source", "capture_source"]);
    assert!(
        !validator.is_valid(&duplicate),
        "a duplicate arrow is rejected"
    );
}
