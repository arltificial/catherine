//! Purpose: the published JSON Schema artifact covers `schema.coproducts`
//! (openspec change `add-coproduct-exhaustiveness`): editor-side validation
//! accepts a well-formed declaration and rejects the empty or duplicate
//! arrow list the kernel also refuses.
//!
//! Responsibilities: compile the shipped artifact under draft 2020-12 and
//! pin acceptance of a well-formed coproduct plus rejection of the two
//! `arrows` defects.
//!
//! Rationale: the keys change's artifact test, mirrored — the artifact is
//! the editor-side surface, so its `$defs.coproduct` constraints must be
//! covered by a test rather than only by review.

use serde_json::{json, Value};

/// Scenario "Editor-side validation": the published artifact covers the
/// coproduct section's syntactic surface — a well-formed coproduct passes,
/// an empty or duplicate `arrows` list is rejected without the kernel (the
/// keys artifact test, mirrored).
#[test]
fn coproducts_are_covered_by_the_artifact() {
    let artifact: Value =
        serde_json::from_str(include_str!("../schema/ct-schema-v1.json")).expect("artifact JSON");
    let validator = jsonschema::draft202012::new(&artifact).expect("the artifact is draft 2020-12");

    let base = json!({
        "format_version": "1.5",
        "schema": {
            "objects": ["C", "V1", "V2"],
            "arrows": [
                {"name": "i1", "src": "V1", "tgt": "C"},
                {"name": "i2", "src": "V2", "tgt": "C"}
            ],
            "equations": [],
            "coproducts": [{"object": "C", "arrows": ["i1", "i2"]}]
        },
        "instance": {"rows": {"C": [], "V1": [], "V2": []}, "tables": {"i1": {}, "i2": {}}}
    });
    assert!(
        validator.is_valid(&base),
        "a well-formed coproduct passes the artifact: {:?}",
        validator
            .iter_errors(&base)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
    );

    let mut empty = base.clone();
    empty["schema"]["coproducts"][0]["arrows"] = json!([]);
    assert!(
        !validator.is_valid(&empty),
        "an empty arrow list is rejected"
    );

    let mut duplicate = base;
    duplicate["schema"]["coproducts"][0]["arrows"] = json!(["i1", "i1"]);
    assert!(
        !validator.is_valid(&duplicate),
        "a duplicate arrow is rejected"
    );
}
