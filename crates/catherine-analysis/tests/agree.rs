//! Purpose: the cross-instance agreement check's controls and fixtures
//! (beads epic `catherine-kw9`, ticket 7): identical instances are
//! silent, disjoint keys are silent (documented silence — partial
//! overlap is agreement, not a completeness failure), and a conflicting
//! overlap yields exactly one witness carrying both projected values.
//!
//! Responsibilities: parse real 1.4 documents (inline controls plus the
//! committed `fixtures/agree/` pairs) through the real parser, call
//! [`catherine_analysis::agree::agree`], and assert structurally — the
//! disagreement count, and each witness's entity, key, arrow, and both
//! value names. Never snapshots.
//!
//! Rationale: the disjoint-keys control is the check's honesty pin — a
//! naive implementation reports one-sided rows as missing, turning
//! agreement into completeness; the conflicting control pins the
//! witness shape that makes the disagreement actionable.

use catherine_analysis::agree::{agree, Disagreement};
use catherine_core::model::{Ident, Model};
use catherine_core::parse::{parse_with_rows, RowNames};

/// A schema; both sides of every control share it verbatim.
const SCHEMA: &str = r#"
  "schema": {
    "objects": ["person", "org"],
    "arrows": [{"name": "works_at", "src": "person", "tgt": "org"}],
    "equations": []
  }
"#;

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

/// Parse one 1.4 document with its presentation names.
fn side(rows: &str) -> (Model, RowNames) {
    let text = format!(
        r#"{{"format_version": "1.4", {SCHEMA}, "instance": {rows}}}"#
    );
    parse_with_rows(&text).expect("the document parses")
}

/// Both instances declare the same rows and the same projections.
const IDENTICAL_A: &str =
    r#"{"rows": {"person": ["p1", "p2"], "org": ["o1", "o2"]}, "tables": {"works_at": {"p1": "o1", "p2": "o2"}}}"#;
const IDENTICAL_B: &str = IDENTICAL_A;

/// The shared key `p1` projects to `o1` on side A and `o2` on side B;
/// `p2` agrees; side B alone declares `q9`.
const CONFLICT_A: &str =
    r#"{"rows": {"person": ["p1", "p2"], "org": ["o1", "o2"]}, "tables": {"works_at": {"p1": "o1", "p2": "o2"}}}"#;
const CONFLICT_B: &str =
    r#"{"rows": {"person": ["p1", "p2", "q9"], "org": ["o1", "o2"]}, "tables": {"works_at": {"p1": "o2", "p2": "o2"}}}"#;

/// No shared keys at all: the instances touch in name only.
const DISJOINT_A: &str =
    r#"{"rows": {"person": ["p1"], "org": ["o1"]}, "tables": {"works_at": {"p1": "o1"}}}"#;
const DISJOINT_B: &str =
    r#"{"rows": {"person": ["q1"], "org": ["o1"]}, "tables": {"works_at": {"q1": "o1"}}}"#;

/// Run the check over two parsed documents.
fn outcome(a: &str, b: &str) -> Vec<Disagreement> {
    parsed(&side(a), &side(b))
}

/// Run the check over two already-complete documents.
fn parsed(a: &(Model, RowNames), b: &(Model, RowNames)) -> Vec<Disagreement> {
    agree(&a.0, &b.0, &a.1, &b.1)
}

/// Identical instances: zero findings — agreement over every shared key.
#[test]
fn identical_instances_are_silent() {
    assert!(
        outcome(IDENTICAL_A, IDENTICAL_B).is_empty(),
        "identical instances agree everywhere"
    );
}

/// Disjoint keys: zero findings, documented silence — a row or key
/// present on only one side is agreement over the shared sub-schema,
/// never a completeness failure.
#[test]
fn disjoint_keys_are_documented_silence() {
    assert!(
        outcome(DISJOINT_A, DISJOINT_B).is_empty(),
        "one-sided rows are not findings: agreement, not completeness"
    );
}

/// Conflicting overlap: the shared key `p1` disagrees on `works_at` and
/// the witness names the entity, the key, the arrow, and both values;
/// the agreeing `p2` and one-sided `q9` stay silent.
#[test]
fn conflicting_overlap_emits_a_witness() {
    let disagreements = outcome(CONFLICT_A, CONFLICT_B);
    assert_eq!(disagreements.len(), 1, "only `p1` disagrees: {disagreements:?}");
    let witness = &disagreements[0];
    assert_eq!(witness.entity, ident("person"));
    assert_eq!(witness.key, "p1");
    assert_eq!(witness.arrow, ident("works_at"));
    assert_eq!(witness.value_a, Some(ident("o1")));
    assert_eq!(witness.value_b, Some(ident("o2")));
}

/// The committed agreeing pair agrees: side B's extra `p3` row is
/// partial overlap, and it emits nothing.
#[test]
fn agreeing_fixture_is_silent() {
    assert!(
        parsed(&load_fixture("agreeing-a"), &load_fixture("agreeing-b")).is_empty(),
        "partial overlap is agreement: side B's extra `p3` emits nothing"
    );
}

/// The committed conflicting pair disagrees on exactly the `p1` cell.
#[test]
fn conflicting_fixture_emits_one_witness() {
    let disagreements = parsed(&load_fixture("conflicting-a"), &load_fixture("conflicting-b"));
    assert_eq!(disagreements.len(), 1, "{disagreements:?}");
    assert_eq!(disagreements[0].key, "p1");
    assert_eq!(disagreements[0].value_a, Some(ident("o1")));
    assert_eq!(disagreements[0].value_b, Some(ident("o2")));
}

/// Load and parse one committed fixture document.
fn load_fixture(name: &str) -> (Model, RowNames) {
    let path = format!(
        "{}/../../fixtures/agree/{name}.ct.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("fixture exists");
    parse_with_rows(&text).expect("fixture parses")
}
