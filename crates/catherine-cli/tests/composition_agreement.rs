//! Purpose: the joint surface's agreement and precedence contract
//! (openspec change `add-model-composition`, ro5u findings
//! DRAFT-001/CORR-001/CORR-002/CORR-004/K-002/K-001) — probed against the
//! real binary.
//!
//! Responsibilities: the run-level `empty_composition_interface` warning in
//! human mode and on the joint `underconstraint` surface; the labeled
//! `row_conflict` for a shared object's row set and for a `null` cell
//! against a value; the namespaced interface intersection; the arity
//! precedence parity (one document and the same document twice report the
//! same first error); and the standalone-parseability boundary of a seam
//! claim.
//!
//! Rationale: design Decision 2 checks both claims an instance union meets,
//! Decision 3 resolves equations with one resolver so the first error never
//! depends on how many documents were passed, and the composition spec's
//! empty-interface scenario promises a signal in every output mode. A
//! cross-document claim is versioned document evidence enforced on the
//! joint run; only a claim that re-declares every arrow it names is
//! standalone-parseable, and the binary is the honest witness for both
//! halves. Assertions are structural (exit codes, stable codes, messages,
//! envelope fields).

mod common;

use common::{code, findings, parse_json, run, stderr, stdout};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
const GAP_SUITE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gap_suite");

fn fixture(path: &str) -> String {
    format!("{FIXTURES}/{path}")
}

fn gap(name: &str) -> String {
    format!("{GAP_SUITE}/{name}")
}

/// The error envelope of a failing run, parsed from stderr.
fn error_of(out: &std::process::Output, what: &str) -> serde_json::Value {
    parse_json(&stderr(out), what)
}

#[test]
fn human_mode_joint_run_prints_the_warning_on_stderr() {
    let a = fixture("composition/disjoint_a.ct.json");
    let b = fixture("composition/disjoint_b.ct.json");
    let out = run(&["check", &a, &b]);
    assert_eq!(
        code(&out),
        0,
        "a disjoint union is clean, not an error: {}",
        stderr(&out)
    );
    assert_eq!(stdout(&out), "", "a clean joint run prints no findings");
    let err = stderr(&out);
    assert!(
        err.contains("warning: "),
        "the run-level warning reaches stderr in human mode: {err}"
    );
    assert!(
        err.contains("shares no declared names"),
        "the human line carries the disjoint-union message: {err}"
    );
    assert!(
        err.contains("disjoint_a.ct.json") && err.contains("disjoint_b.ct.json"),
        "both inputs are named on stderr: {err}"
    );
}

#[test]
fn the_joint_underconstraint_run_warns_on_a_disjoint_union() {
    let a = fixture("composition/disjoint_a.ct.json");
    let b = fixture("composition/disjoint_b.ct.json");
    let out = run(&["underconstraint", &a, &b, "--json"]);
    let value = parse_json(&stdout(&out), "joint underconstraint");
    let warnings = value["warnings"].to_string();
    assert!(
        warnings.contains("empty_composition_interface"),
        "the disjoint union is labeled on the joint analysis surface: {warnings}"
    );
    assert!(
        warnings.contains("disjoint_a.ct.json") && warnings.contains("disjoint_b.ct.json"),
        "both inputs are named: {warnings}"
    );
    assert!(
        !findings(&value).is_empty(),
        "the per-document loop questions are still reported"
    );
}

#[test]
fn a_shared_object_with_different_row_sets_is_a_labeled_conflict() {
    let a = fixture("composition/row_sets_a.ct.json");
    let b = fixture("composition/row_sets_b.ct.json");
    for document in [&a, &b] {
        let alone = run(&["check", document, "--json"]);
        assert_eq!(
            code(&alone),
            0,
            "each document is clean alone: {}",
            stderr(&alone)
        );
    }
    let joint = run(&["check", &a, &b, "--json"]);
    assert_eq!(
        code(&joint),
        1,
        "a partial instance is refused, not unioned: {}",
        stderr(&joint)
    );
    assert_eq!(stdout(&joint), "", "errors never write stdout");
    let value = error_of(&joint, "row set conflict");
    assert_eq!(
        value["data"]["code"], "row_conflict",
        "stable conflict code"
    );
    let message = value["data"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("Order") && message.contains("differing rows: [o2]"),
        "the object and the differing row are named: {message}"
    );
    assert!(
        message.contains("row_sets_a.ct.json") && message.contains("row_sets_b.ct.json"),
        "both declaring documents are named: {message}"
    );
}

#[test]
fn a_null_cell_against_a_value_is_a_labeled_conflict() {
    let a = fixture("composition/null_cell.ct.json");
    let b = fixture("composition/value_cell.ct.json");
    let joint = run(&["check", &a, &b, "--json"]);
    assert_eq!(
        code(&joint),
        1,
        "a null claim against a value is refused: {}",
        stderr(&joint)
    );
    let value = error_of(&joint, "null cell conflict");
    assert_eq!(
        value["data"]["code"], "row_conflict",
        "stable conflict code"
    );
    let message = value["data"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("<dangling>") && message.contains("north"),
        "both claims are spelled: {message}"
    );
}

#[test]
fn disjoint_documents_sharing_a_row_name_are_warned() {
    let a = fixture("composition/same_row_a.ct.json");
    let b = fixture("composition/same_row_b.ct.json");
    let out = run(&["check", &a, &b, "--json"]);
    assert_eq!(
        code(&out),
        0,
        "the disjoint union is clean: {}",
        stderr(&out)
    );
    let value = parse_json(&stdout(&out), "namespaced interface");
    let warnings = value["warnings"].to_string();
    assert!(
        warnings.contains("empty_composition_interface"),
        "a bare row name shared across objects is not an interface: {warnings}"
    );
}

#[test]
fn arity_does_not_change_the_first_error() {
    let path = fixture("composition/precedence.ct.json");
    let single = run(&["check", &path, "--json"]);
    let joint = run(&["check", &path, &path, "--json"]);
    assert_eq!(code(&single), 1, "the lone document is rejected");
    assert_eq!(code(&joint), 1, "so is the joint run");
    let one = error_of(&single, "singleton precedence");
    let two = error_of(&joint, "joint precedence");
    assert_eq!(one["data"]["code"], "unknown_arrow", "the parser's code");
    assert_eq!(
        two["data"]["code"], one["data"]["code"],
        "the same bytes report the same first error code"
    );
    assert_eq!(
        two["data"]["message"], one["data"]["message"],
        "the same bytes report the same first error message"
    );
}

#[test]
fn a_resolved_status_error_is_still_reported() {
    let path = fixture("composition/precedence_status.ct.json");
    let single = run(&["check", &path, "--json"]);
    let joint = run(&["check", &path, &path, "--json"]);
    let one = error_of(&single, "singleton status");
    let two = error_of(&joint, "joint status");
    assert_eq!(code(&single), 1, "the lone document is rejected");
    assert_eq!(code(&joint), 1, "so is the joint run");
    assert_eq!(one["data"]["code"], "invalid_status", "the parser's code");
    assert_eq!(two["data"]["code"], "invalid_status", "not swallowed");
}

#[test]
fn a_cross_document_claim_is_joint_run_evidence() {
    let fragment = fixture("composition/seam_fragment.ct.json");
    let sibling = fixture("composition/seam_sibling.ct.json");
    let alone = run(&["check", &fragment, "--json"]);
    assert_eq!(
        code(&alone),
        1,
        "a claim naming a sibling's arrow is not standalone-parseable"
    );
    let value = error_of(&alone, "fragment alone");
    assert_eq!(value["data"]["code"], "unknown_arrow", "the parser's code");
    let joint = run(&["check", &fragment, &sibling, "--json"]);
    assert_eq!(
        code(&joint),
        3,
        "the joint run enforces the claim: {}",
        stderr(&joint)
    );
    let text = findings(&parse_json(&stdout(&joint), "joint seam"))
        .iter()
        .map(|finding| finding.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        text.contains("seam_fragment.ct.json") && text.contains("seam_sibling.ct.json"),
        "the claim and the sibling are attributed: {text}"
    );
}

#[test]
fn the_committed_seam_claim_is_standalone_parseable() {
    let seam = gap("composition_seam_claim.ct.json");
    let out = run(&["check", &seam, "--json"]);
    assert_eq!(
        code(&out),
        0,
        "the claim re-declares every arrow it names: {}",
        stderr(&out)
    );
    let value = parse_json(&stdout(&out), "seam claim alone");
    assert!(
        findings(&value).is_empty(),
        "the structural claim alone has no instance to violate"
    );
}

#[test]
fn a_seam_document_keeps_its_later_errors() {
    let seam = fixture("composition/seam_status.ct.json");
    let sibling = fixture("composition/seam_sibling.ct.json");
    let joint = run(&["check", &seam, &sibling, "--json"]);
    assert_eq!(
        code(&joint),
        1,
        "the seam resolves, so the later status error is reported: {}",
        stderr(&joint)
    );
    let value = error_of(&joint, "seam status");
    assert_eq!(value["data"]["code"], "invalid_status", "the parser's code");
    let message = value["data"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("`bogus`") && message.contains("must be `proposed` or `accepted`"),
        "the parser's message shape survives the pushout: {message}"
    );
}
