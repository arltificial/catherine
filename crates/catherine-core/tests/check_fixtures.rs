//! Purpose: the executed-checks golden fixtures (openspec change
//! `add-executed-checks`, RED tasks 1–4) — per fixture, the exact expected
//! outcomes (kind + witnesses) and build reports, asserted structurally,
//! never snapshots. One focused test per named shape, so the RED evidence
//! is per fixture.
//!
//! Responsibilities: pin the check contract end to end — parse →
//! [`TypedInstance::build`] → `check` — for the four RED shapes (accepted
//! violation with witnesses, dangling value, build-time typing violation,
//! proposed failure as hypothesis) plus the all-consistent silence pin.
//! The typing fixture's document carries a wrong-object table value, which
//! P0's parser gates structurally (`Syntax`), so it loads through the
//! lenient presentation loader (`common`) — the builder-level typing
//! contract (`typing_enforced_at_build`) is what the test pins.

mod common;

use common::load_presentation;

use catherine_core::checks::{check, CheckOutcome};
use catherine_core::instance::{DanglingCell, TypedInstance, TypingViolation};
use catherine_core::model::{Ident, RowId};
use catherine_core::parse::parse;

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn read_fixture(file: &str) -> String {
    std::fs::read_to_string(format!("{GOLDEN_DIR}/{file}")).expect("fixture exists")
}

/// The full pipeline for a fixture P0's parser accepts.
fn check_parsed(file: &str) -> (TypedInstance, Vec<CheckOutcome>) {
    let text = read_fixture(file);
    let model = parse(&text).expect("fixture parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    let outcomes = check(&model.schema, &built);
    (built, outcomes)
}

/// Task 1 — one violating row on an accepted equation: exactly one
/// outcome, with the full witness (equation, row, both composed values).
/// The equation covers two rows; the consistent row emits nothing — one
/// outcome per violating row, not one per equation.
#[test]
fn accepted_violation_carries_its_witnesses() {
    let (_built, outcomes) = check_parsed("violation_accepted.json");
    assert_eq!(
        outcomes,
        vec![CheckOutcome::Violation {
            equation: 0,
            row: RowId(1),
            lhs: RowId(0),
            rhs: RowId(1),
        }],
        "expected one Violation naming equation 0, row a2, lhs=b1 rhs=b2"
    );
}

/// Task 2 — a dangling arrow value: a Dangling outcome naming arrow +
/// source row; the dangling row is never a violation; the run continues
/// and the other rows' failures are still reported.
#[test]
fn dangling_value_is_reported_and_the_run_continues() {
    let (built, outcomes) = check_parsed("dangling_value.json");
    assert_eq!(
        built.dangling(),
        [DanglingCell {
            arrow: Ident("f".into()),
            src: RowId(0),
        }],
        "expected the build to record f@a1 as dangling"
    );
    assert_eq!(
        outcomes,
        vec![
            CheckOutcome::Dangling {
                arrow: Ident("f".into()),
                src: RowId(0),
            },
            CheckOutcome::Violation {
                equation: 0,
                row: RowId(2),
                lhs: RowId(0),
                rhs: RowId(1),
            },
        ],
        "expected Dangling(f, a1) then Violation(equation 0, row a3, lhs=b1 rhs=b2) — \
         the dangling row emits no violation, the others are still checked"
    );
}

/// Task 3 — an arrow value naming a row of the wrong object: a build-time
/// typing violation naming row + arrow; the value does not become a
/// morphism table entry; the check run emits no outcome for it (a typing
/// violation is a build report, not a check outcome, and not dangling).
#[test]
fn typing_violation_is_a_build_report_and_never_stored() {
    let text = read_fixture("typing_violation.json");
    assert!(
        parse(&text).is_err(),
        "P0's parser gates a wrong-object table value structurally (Syntax)"
    );
    let (schema, instance) = load_presentation(&text).expect("loader loads the presentation");
    let built = TypedInstance::build(&schema, &instance);
    assert_eq!(
        built.typing_violations(),
        [TypingViolation {
            arrow: Ident("f".into()),
            src: RowId(1),
            value: RowId(1),
        }],
        "expected one typing violation naming arrow f and row a2"
    );
    assert_eq!(
        built.tables().get(&Ident("f".into())).map(Vec::as_slice),
        Some(&[Some(RowId(0)), None][..]),
        "the violated value must not become a morphism table entry"
    );
    let outcomes = check(&schema, &built);
    assert_eq!(
        outcomes,
        vec![],
        "the typing violation is a build report — the check run stays silent"
    );
}

/// Task 4 — a proposed equation failing on a row: a Hypothesis outcome,
/// a distinct kind from Violation, separately countable; the accepted
/// equation failing on the same row is a Violation.
#[test]
fn proposed_failure_is_a_hypothesis_distinct_from_violation() {
    let (_built, outcomes) = check_parsed("proposed_hypothesis.json");
    assert_eq!(
        outcomes,
        vec![
            CheckOutcome::Hypothesis {
                equation: 0,
                row: RowId(0),
                lhs: RowId(0),
                rhs: RowId(1),
            },
            CheckOutcome::Violation {
                equation: 1,
                row: RowId(0),
                lhs: RowId(0),
                rhs: RowId(1),
            },
        ],
        "expected Hypothesis(equation 0, proposed) and Violation(equation 1, accepted) \
         — distinct kinds over the same failing row"
    );
    let hypotheses = outcomes
        .iter()
        .filter(|o| matches!(o, CheckOutcome::Hypothesis { .. }))
        .count();
    let violations = outcomes
        .iter()
        .filter(|o| matches!(o, CheckOutcome::Violation { .. }))
        .count();
    assert_eq!((hypotheses, violations), (1, 1), "separately countable");
}

/// The silence pin — an all-consistent instance yields zero outcomes:
/// silence is real success (both rows were evaluated through stored
/// values), not an unevaluated declaration.
#[test]
fn consistent_instance_is_silent() {
    let (built, outcomes) = check_parsed("consistent.json");
    assert_eq!(
        built.tables().get(&Ident("f".into())).map(Vec::as_slice),
        Some(&[Some(RowId(0)), Some(RowId(0))][..]),
        "rows were stored — the silence is post-evaluation"
    );
    assert_eq!(
        built.tables().get(&Ident("g".into())).map(Vec::as_slice),
        Some(&[Some(RowId(0)), Some(RowId(0))][..]),
        "rows were stored — the silence is post-evaluation"
    );
    assert_eq!(outcomes, vec![], "a consistent instance emits nothing");
}
