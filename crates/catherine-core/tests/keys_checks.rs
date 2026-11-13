//! Purpose: declarative-key execution (format 1.3, openspec change
//! `add-declarative-keys`): every declared key is checked as an
//! injectivity claim over its object's rows, with one `violation` finding
//! per colliding unordered pair and the `KeyWitness` naming the key's
//! canonical index and both rows.
//!
//! Responsibilities: pin the kernel-checks scenarios (collision witness,
//! three-way collision as three pairs, dangling-cell exclusion, clean-key
//! silence), the accepted/proposed label split, the exact key-witness
//! JSON, and the equation-witness JSON's byte-identity under the new
//! untagged union.
//!
//! Rationale: silence after evaluation is real success, not an
//! unevaluated declaration; a row with an unresolved cell is already
//! reported by its own dangling finding and never double-reported as a
//! collision (design Decision 4). The label carries the weight — accepted
//! keys execute, proposed keys hypothesize — while the kind stays one of
//! the five.

use catherine_core::checks::check;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, KeyWitness,
    ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, RowId};
use catherine_core::parse::parse;

/// A 1.3 document over `capture` (three rows) and `value` (two rows), the
/// key section and tables spliced in per test.
const DOC: &str = r#"{
  "format_version": "1.3",
  "schema": {
    "objects": ["capture", "value"],
    "arrows": [
      {"name": "capture_source", "src": "capture", "tgt": "value"},
      {"name": "capture_edition", "src": "capture", "tgt": "value"}
    ],
    "equations": [],
    "keys": [__KEYS__]
  },
  "instance": {
    "rows": {"capture": ["c1", "c2", "c3"], "value": ["v1", "v2"]},
    "tables": {
      "capture_source": {__SOURCE__},
      "capture_edition": {__EDITION__}
    }
  }
}"#;

const ACCEPTED: &str = r#"{"object": "capture", "arrows": ["capture_source", "capture_edition"], "status": "accepted"}"#;
const PROPOSED: &str = r#"{"object": "capture", "arrows": ["capture_source", "capture_edition"], "status": "proposed"}"#;

fn document(keys: &str, source: &str, edition: &str) -> String {
    DOC.replace("__KEYS__", keys)
        .replace("__SOURCE__", source)
        .replace("__EDITION__", edition)
}

/// Parse → build → check, the kernel's one pipeline.
fn findings(keys: &str, source: &str, edition: &str) -> Vec<Finding> {
    let text = document(keys, source, edition);
    let model = parse(&text).expect("the fixture parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    check(&model.schema, &built)
}

/// The colliding pair's tables: `c1` and `c2` agree on both key arrows,
/// `c3` differs.
const COLLIDING: &str = r#""c1": "v1", "c2": "v1", "c3": "v2""#;

/// A key violation finding with the given label and rows.
fn key_finding(label: EpistemicLabel, rows: [usize; 2]) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Key(KeyWitness {
            key: 0,
            rows: [RowId(rows[0]), RowId(rows[1])],
        })),
    }
}

/// Scenario "Colliding tuple carries its witness": exactly one executed
/// violation for the pair, naming key 0 and both rows ascending.
#[test]
fn colliding_tuple_carries_its_witness() {
    let findings = findings(ACCEPTED, COLLIDING, COLLIDING);
    assert_eq!(
        findings,
        vec![key_finding(EpistemicLabel::Executed, [0, 1])]
    );
}

/// Scenario "Three-way collision is three pairs": each unordered pair is
/// its own demonstrated failure, in the taxonomy's canonical order.
#[test]
fn three_way_collision_is_three_pairs() {
    let all_equal = r#""c1": "v1", "c2": "v1", "c3": "v1""#;
    let findings = findings(ACCEPTED, all_equal, all_equal);
    assert_eq!(
        findings,
        vec![
            key_finding(EpistemicLabel::Executed, [0, 1]),
            key_finding(EpistemicLabel::Executed, [0, 2]),
            key_finding(EpistemicLabel::Executed, [1, 2]),
        ],
        "one finding per unordered pair, rows ascending"
    );
}

/// Scenario "Dangling key cell is not a collision": the unresolved cell
/// is reported by its dangling finding only; the row never participates.
#[test]
fn dangling_key_cell_is_not_a_collision() {
    // c1's `capture_source` cell is absent (dangling); the edition cell is
    // resolved, so exactly one dangling finding exists. c2 and c3 differ.
    let findings = findings(
        ACCEPTED,
        r#""c2": "v1", "c3": "v2""#,
        r#""c1": "v1", "c2": "v1", "c3": "v2""#,
    );
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Dangling(DanglingWitness {
                arrow: Ident("capture_source".to_string()),
                src: RowId(0),
            }),
        }],
        "the dangling cell is reported once and never as a key collision"
    );
}

/// Scenario "Clean key emits nothing": distinct tuples yield silence.
#[test]
fn clean_key_emits_nothing() {
    let findings = findings(
        ACCEPTED,
        r#""c1": "v1", "c2": "v2", "c3": "v1""#,
        r#""c1": "v1", "c2": "v1", "c3": "v2""#,
    );
    assert!(findings.is_empty(), "a clean key emits no finding");
}

/// A proposed key's collision is a question: the same witness, labeled
/// `hypothesized`.
#[test]
fn proposed_key_collision_is_hypothesized() {
    let findings = findings(PROPOSED, COLLIDING, COLLIDING);
    assert_eq!(
        findings,
        vec![key_finding(EpistemicLabel::Hypothesized, [0, 1])]
    );
}

/// The key witness's pinned JSON: `{"key": n, "rows": [lo, hi]}` under
/// the untagged union, distinguishable from an equation witness without
/// consulting the schema; serde round-trips are lossless.
#[test]
fn key_violation_carries_its_rows() {
    let findings = findings(ACCEPTED, COLLIDING, COLLIDING);
    let text = serde_json::to_string(&findings).expect("findings serialize");
    assert_eq!(
        text,
        r#"[{"label":"executed","kind":"violation","witness":{"key":0,"rows":[0,1]}}]"#
    );
    let back: Vec<Finding> = serde_json::from_str(&text).expect("findings deserialize");
    assert_eq!(back, findings);
}

/// The equation-witness encoding is unchanged by the untagged union: the
/// serialized bytes are exactly the pre-key shape.
#[test]
fn equation_violation_json_is_unchanged() {
    let finding = Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: 0,
            row: RowId(1),
            lhs: RowId(2),
            rhs: RowId(3),
        })),
    };
    let text = serde_json::to_string(&finding).expect("finding serializes");
    assert_eq!(
        text,
        r#"{"label":"executed","kind":"violation","witness":{"equation":0,"row":1,"lhs":2,"rhs":3}}"#
    );
    let back: Finding = serde_json::from_str(&text).expect("finding deserializes");
    assert_eq!(back, finding);
}
