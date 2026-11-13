//! Purpose: the findings golden fixtures (openspec change
//! `add-findings-taxonomy`, task 5 — P1's executed-checks fixtures
//! reshaped per the taxonomy; original shapes in `add-executed-checks`)
//! — per fixture, the exact expected findings (kind + witness fields +
//! label) and build reports, asserted structurally, never snapshots. One
//! focused test per named shape, so the RED evidence is per fixture.
//!
//! Responsibilities: pin the check contract end to end — parse →
//! [`TypedInstance::build`] → `check` — for the four reshaped shapes
//! (accepted violation with witnesses, dangling value, build-time typing
//! violation, proposed failure as a hypothesized violation) plus the
//! all-consistent silence pin. Same documents, same facts as P1; the
//! expectations are now `Finding`s, ordered by the taxonomy's total order
//! (violations before dangling — P1's dangling-first order is
//! superseded). The typing fixture's document carries a wrong-object
//! table value, which P0's parser gates structurally (`Syntax`), so it
//! loads through the lenient presentation loader (`common`) — the
//! builder-level typing contract (`typing_enforced_at_build`) is what the
//! test pins.

mod common;

use common::load_presentation;

use catherine_core::checks::check;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, ViolationWitness,
};
use catherine_core::instance::{DanglingCell, TypedInstance, TypingViolation};
use catherine_core::model::{Ident, RowId};
use catherine_core::parse::parse;

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn read_fixture(file: &str) -> String {
    std::fs::read_to_string(format!("{GOLDEN_DIR}/{file}")).expect("fixture exists")
}

/// The full pipeline for a fixture P0's parser accepts.
fn check_parsed(file: &str) -> Vec<Finding> {
    let text = read_fixture(file);
    let model = parse(&text).expect("fixture parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    check(&model.schema, &built)
}

/// Task 5 (was P1 task 1) — one violating row on an accepted equation:
/// exactly one finding, `executed` label, with the full witness
/// (equation, row, both composed values). The equation covers two rows;
/// the consistent row emits nothing — one finding per violating row, not
/// one per equation.
#[test]
fn accepted_violation_carries_its_witnesses() {
    let (_built, findings) = {
        let model = parse(&read_fixture("violation_accepted.json")).expect("fixture parses");
        let built = TypedInstance::build(&model.schema, &model.instance);
        let findings = check(&model.schema, &built);
        (built, findings)
    };
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: 0,
                row: RowId(1),
                lhs: RowId(0),
                rhs: RowId(1),
            })),
        }],
        "expected one executed violation naming equation 0, row a2, lhs=b1 rhs=b2"
    );
}

/// Task 5 (was P1 task 2) — a dangling arrow value: an executed dangling
/// finding naming arrow + source row; the dangling row is never a
/// violation; the run continues and the other rows' failures are still
/// reported. List order is the taxonomy's: violations before dangling.
#[test]
fn dangling_value_is_reported_and_the_run_continues() {
    let (built, findings) = {
        let model = parse(&read_fixture("dangling_value.json")).expect("fixture parses");
        let built = TypedInstance::build(&model.schema, &model.instance);
        let findings = check(&model.schema, &built);
        (built, findings)
    };
    assert_eq!(
        built.dangling(),
        [DanglingCell {
            arrow: Ident("f".into()),
            src: RowId(0),
        }],
        "expected the build to record f@a1 as dangling"
    );
    assert_eq!(
        findings,
        vec![
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                    equation: 0,
                    row: RowId(2),
                    lhs: RowId(0),
                    rhs: RowId(1),
                })),
            },
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Dangling(DanglingWitness {
                    arrow: Ident("f".into()),
                    src: RowId(0),
                }),
            },
        ],
        "expected the executed violation (equation 0, row a3, lhs=b1 rhs=b2) before the \
         executed dangling (f@a1) — the dangling row emits no violation, the others are \
         still checked"
    );
}

/// Task 5 (was P1 task 3) — an arrow value naming a row of the wrong
/// object: a build-time typing violation naming row + arrow; the value
/// does not become a morphism table entry; the check run emits no finding
/// for it (a typing violation is a build report, not a finding, and not
/// dangling).
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
    let findings = check(&schema, &built);
    assert_eq!(
        findings,
        vec![],
        "the typing violation is a build report — the check run stays silent"
    );
}

/// Task 5 (was P1 task 4) — a proposed equation failing on a row: a
/// violation-shaped finding with the `hypothesized` label, countable
/// apart from the accepted equation's `executed` violation over the same
/// failing row. Kind names the shape; the label names the weight.
#[test]
fn proposed_failure_is_violation_shaped_and_hypothesized() {
    let findings = check_parsed("proposed_hypothesis.json");
    assert_eq!(
        findings,
        vec![
            Finding {
                label: EpistemicLabel::Hypothesized,
                kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                    equation: 0,
                    row: RowId(0),
                    lhs: RowId(0),
                    rhs: RowId(1),
                })),
            },
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                    equation: 1,
                    row: RowId(0),
                    lhs: RowId(0),
                    rhs: RowId(1),
                })),
            },
        ],
        "expected hypothesized (equation 0, proposed) then executed (equation 1, accepted) \
         — violation-shaped findings over the same failing row"
    );
    let hypothesized = findings
        .iter()
        .filter(|f| f.label == EpistemicLabel::Hypothesized)
        .count();
    let executed = findings
        .iter()
        .filter(|f| f.label == EpistemicLabel::Executed)
        .count();
    assert_eq!((hypothesized, executed), (1, 1), "separately countable");
}

/// The silence pin — an all-consistent instance yields zero findings:
/// silence is real success (both rows were evaluated through stored
/// values), not an unevaluated declaration.
#[test]
fn consistent_instance_is_silent() {
    let (built, findings) = {
        let model = parse(&read_fixture("consistent.json")).expect("fixture parses");
        let built = TypedInstance::build(&model.schema, &model.instance);
        let findings = check(&model.schema, &built);
        (built, findings)
    };
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
    assert_eq!(findings, vec![], "a consistent instance emits nothing");
}
