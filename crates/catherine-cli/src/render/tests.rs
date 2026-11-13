//! The presentation layer's unit pins: the §5 finding blocks (equation
//! and key witnesses), the dangling/analysis renderings, and the label
//! suffix rule. Split out of `render.rs` so both files stay under the
//! size gate; no behavior change.

use super::*;
use catherine_core::findings::{
    ContradictionWitness, DanglingWitness, RedundantWitness, UnderconstraintWitness,
};
use catherine_core::model::{Arrow, ConstraintStatus, Equation, Ident, Key};
use std::collections::BTreeMap;

fn finding(kind: FindingKind, label: EpistemicLabel) -> Finding {
    Finding { label, kind }
}

/// The bowling schema, minimally reconstructed for the transcript
/// pin: the four arrows and the one canonical equation.
fn bowling_schema() -> Schema {
    let mut arrows = BTreeMap::new();
    for (name, src, tgt) in [
        ("staff_unit", "Assignment", "Staff"),
        ("assigned_staff", "Staff", "Unit"),
        ("equipment_unit", "Assignment", "Equipment"),
        ("assigned_item", "Equipment", "Unit"),
    ] {
        arrows.insert(
            Ident(name.to_string()),
            Arrow {
                src: Ident(src.to_string()),
                tgt: Ident(tgt.to_string()),
            },
        );
    }
    Schema {
        objects: BTreeMap::new(),
        arrows,
        equations: vec![Equation {
            lhs: Path(vec![
                Ident("staff_unit".to_string()),
                Ident("assigned_staff".to_string()),
            ]),
            rhs: Path(vec![
                Ident("equipment_unit".to_string()),
                Ident("assigned_item".to_string()),
            ]),
            rationale: "a row's staff and its equipment land in the same unit".to_string(),
            status: ConstraintStatus::Accepted,
        }],
        keys: Vec::new(),
    }
}

#[test]
fn violation_renders_the_s5_block() {
    let schema = bowling_schema();
    let lines = render_one(
        &finding(
            FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: 0,
                row: RowId(3),
                lhs: RowId(1),
                rhs: RowId(2),
            })),
            EpistemicLabel::Executed,
        ),
        &schema,
    );
    assert_eq!(
        lines,
        vec![
            "violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item",
            "  at row Assignment#3: lhs=Unit#1 rhs=Unit#2",
        ]
    );
}

#[test]
fn key_violation_renders_both_rows() {
    let mut schema = bowling_schema();
    schema.keys.push(Key {
        object: Ident("Assignment".to_string()),
        arrows: vec![
            Ident("staff_unit".to_string()),
            Ident("equipment_unit".to_string()),
        ],
        status: ConstraintStatus::Accepted,
    });
    let lines = render_one(
        &finding(
            FindingKind::Violation(ViolationWitness::Key(KeyWitness {
                key: 0,
                rows: [RowId(3), RowId(7)],
            })),
            EpistemicLabel::Executed,
        ),
        &schema,
    );
    assert_eq!(
        lines,
        vec![
            "violation: key-0 Assignment(staff_unit∘equipment_unit)",
            "  at rows Assignment#3, Assignment#7",
        ]
    );
}

#[test]
fn non_executed_labels_append_to_the_head_line() {
    let schema = bowling_schema();
    let lines = render_one(
        &finding(
            FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: 0,
                row: RowId(3),
                lhs: RowId(1),
                rhs: RowId(2),
            })),
            EpistemicLabel::Hypothesized,
        ),
        &schema,
    );
    assert!(lines[0].ends_with(" (hypothesized)"), "{}", lines[0]);
    assert_eq!(lines[1], "  at row Assignment#3: lhs=Unit#1 rhs=Unit#2");
}

#[test]
fn dangling_renders_arrow_and_source_row() {
    let schema = bowling_schema();
    let lines = render_one(
        &finding(
            FindingKind::Dangling(DanglingWitness {
                arrow: Ident("assigned_staff".to_string()),
                src: RowId(0),
            }),
            EpistemicLabel::Executed,
        ),
        &schema,
    );
    assert_eq!(
        lines,
        vec!["dangling: arrow assigned_staff", "  at row Staff#0"]
    );
}

#[test]
fn redundant_renders_labeled_lines() {
    // The three analysis classes have no P4 producer; constructed
    // findings pin their renderings at the unit level (brief §3
    // criterion 4). No schema anchor is needed.
    let schema = Schema::default();
    let lines = render_one(
        &finding(
            FindingKind::Redundant(RedundantWitness {
                derivable: Path(vec![Ident("f".to_string()), Ident("g".to_string())]),
                reducing: Path(vec![Ident("h".to_string())]),
            }),
            EpistemicLabel::Unknown,
        ),
        &schema,
    );
    assert_eq!(lines, vec!["redundant: f∘g (unknown)", "  derivable as h",]);
}

#[test]
fn contradiction_renders_labeled_lines() {
    let schema = Schema::default();
    let lines = render_one(
        &finding(
            FindingKind::Contradiction(ContradictionWitness { scope: vec![0, 2] }),
            EpistemicLabel::Unknown,
        ),
        &schema,
    );
    assert_eq!(
        lines,
        vec!["contradiction: eq-0, eq-2 admit no instance (unknown)"]
    );
}

#[test]
fn underconstraint_renders_labeled_lines() {
    let schema = Schema::default();
    let lines = render_one(
        &finding(
            FindingKind::Underconstraint(UnderconstraintWitness {
                case: DomainQuestion::ParallelPaths {
                    lhs: Path(vec![Ident("staff_unit".to_string())]),
                    rhs: Path(vec![Ident("equipment_unit".to_string())]),
                },
                question: "have no declared equation?".to_string(),
            }),
            EpistemicLabel::Hypothesized,
        ),
        &schema,
    );
    assert_eq!(
        lines,
        vec![
            "underconstraint: parallel paths staff_unit / equipment_unit (hypothesized)",
            "  have no declared equation?",
        ]
    );
}
