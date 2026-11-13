//! Purpose: editor-side validation (openspec spec `model-document`,
//! scenario "Editor-side validation") — the published artifact
//! `schema/ct-schema-v1.json` rejects every schema-expressible error
//! class without the kernel, and states which checks stay kernel-side.
//!
//! Responsibilities: compile the artifact as draft 2020-12, prove a valid
//! document passes, prove each class (missing required field, JSON type
//! mismatch, bad `status` enum, ident-pattern violation, duplicate
//! array-identifier) is rejected by the artifact alone with the offending
//! keyword path pinned, prove the artifact is not the version authority,
//! and pin the documented kernel-side boundary (`unknown_arrow`,
//! `ill_typed_equation`).
//!
//! Rationale: editors and agents validate with this artifact while the
//! kernel is not running, so the artifact's coverage is a contract of its
//! own. Driving a real validator (not string-matching the artifact) is
//! what makes the claim executable; the keyword paths prove the intended
//! rule fired rather than an incidental one.

use jsonschema::Validator;
use serde_json::{json, Value};

/// The published artifact, compiled once per test from the shipped bytes.
fn validator() -> Validator {
    let schema: Value = serde_json::from_str(include_str!("../schema/ct-schema-v1.json"))
        .expect("the shipped artifact is JSON");
    jsonschema::draft202012::new(&schema).expect("the artifact is a valid draft 2020-12 schema")
}

/// One valid document to mutate per error class.
fn valid_document() -> Value {
    json!({
        "format_version": "1.0",
        "schema": {
            "objects": ["A"],
            "arrows": [{"name": "f", "src": "A", "tgt": "A"}],
            "equations": [{"lhs": ["f"], "rhs": ["f"], "status": "accepted"}]
        },
        "instance": {"rows": {"A": ["a0"]}, "tables": {"f": {"a0": "a0"}}}
    })
}

/// The canonical keyword paths the artifact reports for `instance`.
fn keyword_paths(instance: &Value) -> Vec<String> {
    let compiled = validator();
    compiled
        .iter_errors(instance)
        .map(|error| error.schema_path().as_str().to_string())
        .collect()
}

#[test]
fn a_valid_document_passes_the_published_artifact() {
    assert!(
        validator().is_valid(&valid_document()),
        "the minimal valid document must pass: {:?}",
        keyword_paths(&valid_document())
    );
}

#[test]
fn schema_expressible_errors_are_rejected_without_the_kernel() {
    let mut missing_instance = valid_document();
    missing_instance
        .as_object_mut()
        .expect("valid document is an object")
        .remove("instance");

    let mut wrong_type = valid_document();
    wrong_type["format_version"] = json!(1.0);

    let mut bad_status = valid_document();
    bad_status["schema"]["equations"][0]["status"] = json!("maybe");

    let mut bad_ident = valid_document();
    bad_ident["schema"]["objects"] = json!(["1bad"]);

    let mut duplicate_ident = valid_document();
    duplicate_ident["schema"]["objects"] = json!(["A", "A"]);

    let cases: [(&str, Value, &str); 5] = [
        ("missing required field", missing_instance, "/required"),
        (
            "JSON type mismatch",
            wrong_type,
            "/properties/format_version/type",
        ),
        (
            "bad status enum",
            bad_status,
            "/properties/schema/properties/equations/items/properties/status/enum",
        ),
        ("ident-pattern violation", bad_ident, "/$defs/ident/pattern"),
        (
            "duplicate array-identifier",
            duplicate_ident,
            "/properties/schema/properties/objects/uniqueItems",
        ),
    ];

    for (name, instance, expected_keyword_path) in cases {
        let paths = keyword_paths(&instance);
        assert!(
            !paths.is_empty(),
            "{name}: the artifact must reject it, got no errors"
        );
        assert!(
            paths.iter().any(|path| path == expected_keyword_path),
            "{name}: expected the {expected_keyword_path} keyword to fire, got {paths:?}"
        );
    }
}

#[test]
fn the_artifact_is_not_the_version_authority() {
    let mut future_major = valid_document();
    future_major["format_version"] = json!("9.9");
    assert!(
        validator().is_valid(&future_major),
        "shape-valid versions pass the artifact; the kernel decides which majors it knows"
    );
}

#[test]
fn the_artifact_documents_the_kernel_side_checks() {
    let schema: Value = serde_json::from_str(include_str!("../schema/ct-schema-v1.json"))
        .expect("the shipped artifact is JSON");
    let description = schema["description"]
        .as_str()
        .expect("the artifact carries a description");
    for kernel_side in ["unknown_arrow", "ill_typed_equation"] {
        assert!(
            description.contains(kernel_side),
            "the artifact must document that {kernel_side} remains kernel-side"
        );
    }
}
