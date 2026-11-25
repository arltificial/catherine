//! Purpose: the published JSON Schema artifact covers `schema.pullbacks`
//! (openspec change `add-pullback-checks`): editor-side validation accepts
//! a well-formed 1.6 declaration and rejects the leg/cospan arity, the
//! duplicate-arrow lists, and the bad status the kernel also refuses.
//!
//! Responsibilities: compile the shipped artifact under draft 2020-12 and
//! pin acceptance of a well-formed pullback plus rejection of its
//! shape-only defects.
//!
//! Rationale: the coproduct artifact test, mirrored — the artifact is the
//! editor-side surface, so its `$defs.pullback` constraints must be
//! covered by a test rather than only by review. Declaredness and
//! composability stay kernel-side (`invalid_pullback`).

use serde_json::{json, Value};

/// A well-formed 1.6 pullback square over `P --p1--> A --f--> C` and
/// `P --p2--> B --g--> C`, with a total instance.
fn base() -> Value {
    json!({
        "format_version": "1.6",
        "schema": {
            "objects": ["P", "A", "B", "C"],
            "arrows": [
                {"name": "p1", "src": "P", "tgt": "A"},
                {"name": "p2", "src": "P", "tgt": "B"},
                {"name": "f", "src": "A", "tgt": "C"},
                {"name": "g", "src": "B", "tgt": "C"}
            ],
            "equations": [],
            "pullbacks": [
                {"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"], "status": "accepted"}
            ]
        },
        "instance": {
            "rows": {"P": ["r1"], "A": ["a1"], "B": ["b1"], "C": ["c1"]},
            "tables": {
                "p1": {"r1": "a1"},
                "p2": {"r1": "b1"},
                "f": {"a1": "c1"},
                "g": {"b1": "c1"}
            }
        }
    })
}

/// A copy of the document with the node at `at` — the pointer and its
/// replacement value, drawn as one group — replaced.
fn mutated(base: &Value, at: (&str, Value)) -> Value {
    let (pointer, value) = at;
    let mut document = base.clone();
    *document
        .pointer_mut(pointer)
        .expect("the pointer names a node") = value;
    document
}

/// Scenario "Editor-side validation": the published artifact covers the
/// pullback section's syntactic surface — a well-formed square passes,
/// and the arity, duplicate-list, and status defects are rejected without
/// the kernel (the coproduct artifact test, mirrored).
#[test]
fn pullbacks_are_covered_by_the_artifact() {
    let artifact: Value =
        serde_json::from_str(include_str!("../schema/ct-schema-v1.json")).expect("artifact JSON");
    let validator = jsonschema::draft202012::new(&artifact).expect("the artifact is draft 2020-12");
    let base = base();
    assert!(
        validator.is_valid(&base),
        "a well-formed 1.6 pullback passes the artifact: {:?}",
        validator
            .iter_errors(&base)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
    );

    let legs = "/schema/pullbacks/0/legs";
    let cases = [
        (legs, json!(["p1"]), "a one-leg square"),
        (legs, json!(["p1", "p2", "p3"]), "a three-leg square"),
        (legs, json!(["p1", "p1"]), "a duplicate-leg list"),
        (
            "/schema/pullbacks/0/cospan",
            json!(["f", "f"]),
            "a duplicate-cospan list",
        ),
        ("/schema/pullbacks/0/status", json!("maybe"), "a bad status"),
    ];
    for (pointer, value, label) in cases {
        assert!(
            !validator.is_valid(&mutated(&base, (pointer, value))),
            "{label} is rejected"
        );
    }
}
