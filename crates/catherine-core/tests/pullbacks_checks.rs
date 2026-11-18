//! Purpose: executed pullback checks (format 1.6, openspec change
//! `add-pullback-checks`): a declared pullback asserts the universal
//! property over the instance — commutes, covers, unique — and each
//! failure is a `violation` carrying the matching `PullbackWitness`.
//!
//! Responsibilities: pin the kernel-checks scenarios (commutes with both
//! composed values, canonical-order coverage, uniqueness as an unordered
//! row pair, dangling cells leaving verdicts undecidable, evaluated
//! silence), the accepted/proposed label split, and the witness's
//! structural JSON.
//!
//! Rationale: the parsed declaration guarantees two apex-sourced legs and
//! a same-target cospan, so the check decides the three claims over the
//! typed tables alone. A dangling cell has no verdict — its own dangling
//! finding reports it and a partial projector suppresses coverage because
//! unknown rows could cover anything; silence after evaluation is real
//! success, not an unevaluated declaration.

use catherine_core::checks::check;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, Finding, FindingKind, PullbackWitness, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Model, RowId};
use catherine_core::parse::parse;

/// A 1.6 document over `P --p1--> A --f--> C` and `P --p2--> B --g--> C`;
/// the declaration, rows, and tables are spliced in per scenario.
const DOC: &str = r#"{
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
    "pullbacks": [__PULLBACK__]
  },
  "instance": {"rows": __ROWS__, "tables": __TABLES__}
}"#;

const ACCEPTED: &str =
    r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"], "status": "accepted"}"#;
const OMITTED: &str = r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"]}"#;
const PROPOSED: &str =
    r#"{"apex": "P", "legs": ["p1", "p2"], "cospan": ["f", "g"], "status": "proposed"}"#;

/// Parse → build → check, the kernel's one pipeline.
fn checked(pullback: &str, rows: &str, tables: &str) -> (Model, Vec<Finding>) {
    let text = DOC
        .replace("__PULLBACK__", pullback)
        .replace("__ROWS__", rows)
        .replace("__TABLES__", tables);
    let model = parse(&text).expect("the fixture parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    let findings = check(&model.schema, &built);
    (model, findings)
}

/// The dense id of an object's `index`-th parsed row — never a guessed
/// name → id map.
fn row(model: &Model, object: &str, index: usize) -> RowId {
    model.instance.rows[&Ident(object.to_string())][index]
}

fn value(model: &Model, arrow: &str, src: RowId) -> RowId {
    model.instance.values[&Ident(arrow.to_string())][src.0].expect("cell is resolved")
}

/// `(f ∘ p1, g ∘ p2)` on an apex row, derived from the parsed tables.
fn composed(model: &Model, apex: RowId) -> (RowId, RowId) {
    let a = value(model, "p1", apex);
    let b = value(model, "p2", apex);
    (value(model, "f", a), value(model, "g", b))
}

fn violation(label: EpistemicLabel, witness: PullbackWitness) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Pullback(witness)),
    }
}

fn commutes(label: EpistemicLabel, row: RowId, lhs: RowId, rhs: RowId) -> Finding {
    let witness = PullbackWitness::Commutes {
        pullback: 0,
        row,
        lhs,
        rhs,
    };
    violation(label, witness)
}

fn covers(pair: [RowId; 2]) -> Finding {
    violation(
        EpistemicLabel::Executed,
        PullbackWitness::Covers { pullback: 0, pair },
    )
}

fn unique(rows: [RowId; 2]) -> Finding {
    violation(
        EpistemicLabel::Executed,
        PullbackWitness::Unique { pullback: 0, rows },
    )
}

fn dangling(arrow: &str, src: RowId) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Dangling(DanglingWitness {
            arrow: Ident(arrow.to_string()),
            src,
        }),
    }
}

/// Scenario "Non-commuting apex row carries both composed values": one
/// apex row whose two compositions land on different C rows yields
/// exactly one executed commutes violation naming the pullback, the row,
/// and both values.
#[test]
fn non_commuting_apex_row_carries_both_composed_values() {
    let rows = r#"{"P": ["r1"], "A": ["a1"], "B": ["b1"], "C": ["c1", "c2"]}"#;
    let tables = r#"{
      "p1": {"r1": "a1"},
      "p2": {"r1": "b1"},
      "f": {"a1": "c1"},
      "g": {"b1": "c2"}
    }"#;
    let (model, findings) = checked(ACCEPTED, rows, tables);
    let apex = row(&model, "P", 0);
    let (lhs, rhs) = composed(&model, apex);
    assert_ne!(lhs, rhs, "the fixture is genuinely non-commuting");
    assert_eq!(
        findings,
        vec![commutes(EpistemicLabel::Executed, apex, lhs, rhs)],
        "exactly one violation, naming the apex row and both composed values"
    );
}

/// Scenario "Missing fiber pair is a coverage violation": four resolvable
/// pairs share one target and exactly one is projected, so the three
/// others are reported in canonical (target, A-row, B-row) order.
#[test]
fn missing_fiber_pair_is_a_coverage_violation() {
    let rows = r#"{"P": ["r1"], "A": ["a1", "a2"], "B": ["b1", "b2"], "C": ["c1"]}"#;
    let tables = r#"{
      "p1": {"r1": "a1"},
      "p2": {"r1": "b1"},
      "f": {"a1": "c1", "a2": "c1"},
      "g": {"b1": "c1", "b2": "c1"}
    }"#;
    let (model, findings) = checked(ACCEPTED, rows, tables);
    let (a0, a1) = (row(&model, "A", 0), row(&model, "A", 1));
    let (b0, b1) = (row(&model, "B", 0), row(&model, "B", 1));
    assert_eq!(
        findings,
        vec![covers([a0, b1]), covers([a1, b0]), covers([a1, b1])],
        "three uncovered pairs, a ascending then b ascending within the target"
    );
}

/// Scenario "Duplicated projection is a uniqueness violation": two apex
/// rows project to the same pair, so the square is not a pullback even
/// though it commutes and covers.
#[test]
fn duplicated_projection_is_a_uniqueness_violation() {
    let rows = r#"{"P": ["r1", "r2"], "A": ["a1"], "B": ["b1"], "C": ["c1"]}"#;
    let tables = r#"{
      "p1": {"r1": "a1", "r2": "a1"},
      "p2": {"r1": "b1", "r2": "b1"},
      "f": {"a1": "c1"},
      "g": {"b1": "c1"}
    }"#;
    let (model, findings) = checked(ACCEPTED, rows, tables);
    let (p0, p1) = (row(&model, "P", 0), row(&model, "P", 1));
    assert_eq!(
        findings,
        vec![unique([p0, p1])],
        "one duplicate projection, rows ascending; commutes and covers are clean"
    );
}

/// The dangling-leg square: `r2`'s `p1` cell is unresolved, so the
/// projector set is partial (coverage is suppressed) and `r2` has no
/// commutes or uniqueness verdict.
fn dangling_leg_square() -> (Model, Vec<Finding>) {
    let rows = r#"{"P": ["r1", "r2"], "A": ["a1"], "B": ["b1", "b2"], "C": ["c1"]}"#;
    let tables = r#"{
      "p1": {"r1": "a1"},
      "p2": {"r1": "b1", "r2": "b1"},
      "f": {"a1": "c1"},
      "g": {"b1": "c1", "b2": "c1"}
    }"#;
    checked(ACCEPTED, rows, tables)
}

/// The dangling-cospan square: `a2`'s `f` cell is unresolved, so `a2` is
/// excluded from the fiber; the remaining pair `(a1, b2)` is a decidable
/// coverage violation.
fn dangling_cospan_square() -> (Model, Vec<Finding>) {
    let rows = r#"{"P": ["r1"], "A": ["a1", "a2"], "B": ["b1", "b2"], "C": ["c1"]}"#;
    let tables = r#"{
      "p1": {"r1": "a1"},
      "p2": {"r1": "b1"},
      "f": {"a1": "c1"},
      "g": {"b1": "c1", "b2": "c1"}
    }"#;
    checked(ACCEPTED, rows, tables)
}

/// Scenario "Dangling cells keep verdicts undecidable": a dangling leg
/// cell yields only its dangling finding (never a fabricated verdict),
/// and a dangling cospan cell removes its pair from coverage.
#[test]
fn dangling_cells_keep_verdicts_undecidable() {
    let (model, findings) = dangling_leg_square();
    assert_eq!(
        findings,
        vec![dangling("p1", row(&model, "P", 1))],
        "the unresolved leg is reported as dangling, never as a verdict"
    );
    assert!(
        findings
            .iter()
            .all(|f| !matches!(f.kind, FindingKind::Violation(_))),
        "an undefined row is never reported as a pullback violation"
    );

    let (model, findings) = dangling_cospan_square();
    let (a0, a1) = (row(&model, "A", 0), row(&model, "A", 1));
    let b1 = row(&model, "B", 1);
    assert_eq!(
        findings,
        vec![covers([a0, b1]), dangling("f", a1)],
        "only the decidable pair is covered; the dangling cospan pair is excluded"
    );
    assert!(
        findings
            .iter()
            .filter_map(|f| match &f.kind {
                FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Covers {
                    pair,
                    ..
                })) => Some(pair[0]),
                _ => None,
            })
            .all(|first| first != a1),
        "the dangling cospan row never enters coverage"
    );
}

/// Scenario "Evaluated silence is real success": a clean square emits
/// nothing, and so does a zero-row apex over an empty fiber — the check
/// ran and trivially held.
#[test]
fn evaluated_pullback_silence_is_real_success() {
    let rows = r#"{"P": ["r1", "r2"], "A": ["a1", "a2"], "B": ["b1", "b2"], "C": ["c1", "c2"]}"#;
    let tables = r#"{
      "p1": {"r1": "a1", "r2": "a2"},
      "p2": {"r1": "b1", "r2": "b2"},
      "f": {"a1": "c1", "a2": "c2"},
      "g": {"b1": "c1", "b2": "c2"}
    }"#;
    let (_, findings) = checked(ACCEPTED, rows, tables);
    assert!(
        findings.is_empty(),
        "a clean pullback square emits no finding"
    );

    // No apex rows and no common cospan target: the fiber is empty, so
    // coverage holds trivially.
    let rows = r#"{"P": [], "A": ["a1"], "B": ["b1"], "C": ["c1", "c2"]}"#;
    let tables = r#"{
      "p1": {},
      "p2": {},
      "f": {"a1": "c1"},
      "g": {"b1": "c2"}
    }"#;
    let (_, findings) = checked(ACCEPTED, rows, tables);
    assert!(
        findings.is_empty(),
        "a zero-row apex over an empty fiber is clean"
    );
}

/// Scenario "Status defaults to accepted; proposed pullbacks are
/// questions": an omitted status is executed on failure, a proposed one
/// is hypothesized with the same witness shape.
#[test]
fn pullback_status_defaults_to_accepted_and_proposed_fails_as_hypothesized() {
    let rows = r#"{"P": ["r1"], "A": ["a1"], "B": ["b1"], "C": ["c1", "c2"]}"#;
    let tables = r#"{
      "p1": {"r1": "a1"},
      "p2": {"r1": "b1"},
      "f": {"a1": "c1"},
      "g": {"b1": "c2"}
    }"#;
    let (model, omitted) = checked(OMITTED, rows, tables);
    let apex = row(&model, "P", 0);
    let (lhs, rhs) = composed(&model, apex);
    assert_eq!(
        omitted,
        vec![commutes(EpistemicLabel::Executed, apex, lhs, rhs)],
        "an omitted status defaults to accepted (executed)"
    );
    let (_, proposed) = checked(PROPOSED, rows, tables);
    assert_eq!(
        proposed,
        vec![commutes(EpistemicLabel::Hypothesized, apex, lhs, rhs)],
        "a proposed failure is a question with the same witness"
    );
    assert_eq!(
        omitted[0].kind, proposed[0].kind,
        "the witness shape is identical"
    );
    assert_ne!(
        omitted[0].label, proposed[0].label,
        "only the weight differs"
    );
}

fn witness_keys(finding: &Finding) -> (Vec<String>, serde_json::Value) {
    let json = serde_json::to_value(finding).expect("finding serializes");
    let witness = json["witness"].clone();
    let mut keys: Vec<String> = witness
        .as_object()
        .expect("the witness is an object")
        .keys()
        .cloned()
        .collect();
    keys.sort_unstable();
    (keys, witness)
}

fn round_trips(finding: &Finding) {
    let text = serde_json::to_string(finding).expect("finding serializes");
    let back: Finding = serde_json::from_str(&text).expect("finding deserializes");
    assert_eq!(&back, finding, "serde round-trip is lossless");
}

/// Assert a witness's exact field set and numeric fields, then that it
/// round-trips — structural, never output-text assertions.
fn assert_fields(finding: &Finding, expected_keys: &[&str], values: &[(&str, u64)]) {
    let (keys, witness) = witness_keys(finding);
    let mut expected: Vec<String> = expected_keys.iter().map(|key| key.to_string()).collect();
    expected.sort_unstable();
    assert_eq!(
        keys, expected,
        "the witness field set is the shape's contract"
    );
    for (field, value) in values {
        assert_eq!(witness[*field].as_u64(), Some(*value), "field `{field}`");
    }
    round_trips(finding);
}

/// Scenario "Pullback violation carries the square and the offender": the
/// three witness shapes serialize to disjoint structural field sets —
/// `pullback`/`row`/`lhs`/`rhs`, `pullback`/`pair`, `pullback`/`rows`.
#[test]
fn pullback_violation_carries_the_square_and_the_offender() {
    let commutes = commutes(EpistemicLabel::Executed, RowId(5), RowId(6), RowId(7));
    assert_fields(
        &commutes,
        &["pullback", "row", "lhs", "rhs"],
        &[("pullback", 0), ("row", 5), ("lhs", 6), ("rhs", 7)],
    );
    let (_, witness) = witness_keys(&commutes);
    assert!(witness.get("pair").is_none() && witness.get("rows").is_none());

    let covers = covers([RowId(1), RowId(2)]);
    assert_fields(&covers, &["pullback", "pair"], &[("pullback", 0)]);
    let (_, witness) = witness_keys(&covers);
    assert_eq!(witness["pair"][0].as_u64(), Some(1));
    assert_eq!(witness["pair"][1].as_u64(), Some(2));

    let unique = unique([RowId(3), RowId(4)]);
    assert_fields(&unique, &["pullback", "rows"], &[("pullback", 0)]);
    let (_, witness) = witness_keys(&unique);
    assert_eq!(witness["rows"][0].as_u64(), Some(3));
    assert_eq!(witness["rows"][1].as_u64(), Some(4));
}
