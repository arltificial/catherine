//! Purpose: the executed-checks properties (proptest, never
//! insta-snapshots; openspec change `add-executed-checks`, task 10,
//! reshaped onto the findings taxonomy by `add-findings-taxonomy`) —
//! totality (empty schema or empty instance → zero findings), self-loops
//! legal and evaluated like any arrow, and the findings matching an
//! independent recomputation of the check contract over arbitrary
//! well-typed models. Findings are asserted structurally — kind +
//! witness + label — never byte snapshots; the vendored parity sets in
//! `fixtures/parity/` are the parity contract, not snapshots.

mod common;
#[path = "checks_properties/expected.rs"]
mod expected;

use common::fixture::{self, FixtureCase};
use expected::expected_findings;

use catherine_core::checks::check;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Path, RowId, Schema,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

type Case = FixtureCase;

/// Build the case from drawn parts via the shared fixture builder, then
/// sort the equations into the parser's canonical storage order.
fn build_case(
    shapes: &[(usize, usize)],
    row_counts: &[usize],
    cell_pool: &[usize],
    eq_seeds: &[(usize, usize)],
    statuses: &[bool],
) -> Case {
    let mut case = fixture::build_case(shapes, row_counts, cell_pool, eq_seeds, statuses);
    case.equations.sort_by(|a, b| {
        (&a.lhs, &a.rhs, &a.rationale, a.status.as_str()).cmp(&(
            &b.lhs,
            &b.rhs,
            &b.rationale,
            b.status.as_str(),
        ))
    });
    case
}

fn arb_case() -> impl Strategy<Value = Case> {
    (1usize..=3usize, 1usize..=4usize)
        .prop_flat_map(|(n_objects, n_arrows)| {
            (
                Just(n_objects),
                proptest::collection::vec((0..n_objects, 0..n_objects), n_arrows),
                proptest::collection::vec(0usize..=3usize, n_objects),
            )
        })
        .prop_flat_map(|(_n_objects, shapes, row_counts)| {
            let n_cells: usize = shapes
                .iter()
                .enumerate()
                .map(|(i, &(s, _))| row_counts[s] * (i + 1))
                .sum::<usize>()
                .max(1);
            let cell_pool = proptest::collection::vec(0usize..=8usize, n_cells);
            let eq_seeds = proptest::collection::vec((0usize..=7usize, 0usize..=7usize), 0..=3);
            let statuses = proptest::collection::vec(any::<bool>(), 0..=3);
            (
                Just(shapes),
                Just(row_counts),
                cell_pool,
                eq_seeds,
                statuses,
            )
        })
        .prop_map(|(shapes, row_counts, cell_pool, eq_seeds, statuses)| {
            build_case(&shapes, &row_counts, &cell_pool, &eq_seeds, &statuses)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The findings match the spec recomputation, structurally, over
    /// arbitrary well-typed models (self-loop arrows included — drawn
    /// shapes carry src == tgt arrows and are evaluated like any arrow).
    #[test]
    fn findings_match_the_contract_recomputation(case in arb_case()) {
        let (schema, instance) = common::fixture::assemble(&case);
        let built = TypedInstance::build(&schema, &instance);
        prop_assert_eq!(
            check(&schema, &built),
            expected_findings(&schema, &built),
            "check matches the independent recomputation (self-loops included)"
        );
    }

    /// Every finding's witnesses stay in scope: dangling names a declared
    /// arrow and a row of its source object; violations name a valid
    /// equation index, a row of the equation's domain, and composed
    /// values — never witnesses without rows. Labels are epistemically
    /// honest: only executed/hypothesized, mirroring equation status; the
    /// analysis kinds have no producer in this crate.
    #[test]
    fn every_finding_carries_in_scope_witnesses(case in arb_case()) {
        let (schema, instance) = common::fixture::assemble(&case);
        let built = TypedInstance::build(&schema, &instance);
        for finding in check(&schema, &built) {
            prop_assert!(
                matches!(finding.label, EpistemicLabel::Executed | EpistemicLabel::Hypothesized),
                "kernel labels are executed/hypothesized only, got {:?}",
                finding.label
            );
            match finding.kind {
                FindingKind::Dangling(w) => {
                    prop_assert_eq!(finding.label, EpistemicLabel::Executed, "dangling is executed");
                    let cells = built.tables().get(&w.arrow);
                    prop_assert!(cells.is_some(), "dangling arrow `{w:?}` declared");
                    let n = built.rows().get(&schema.arrows[&w.arrow].src).map(Vec::len).unwrap_or(0);
                    prop_assert!(w.src.0 < n, "dangling src row in the arrow's source space");
                }
                FindingKind::Violation(ViolationWitness::Equation(w)) => {
                    prop_assert!(w.equation < schema.equations.len(), "equation index valid");
                    let eq = &schema.equations[w.equation];
                    prop_assert_eq!(
                        finding.label == EpistemicLabel::Hypothesized,
                        eq.status == ConstraintStatus::Proposed,
                        "the label mirrors the equation's status"
                    );
                    let lead = &schema.arrows[&eq.lhs.0[0]];
                    let n = built.rows().get(&lead.src).map(Vec::len).unwrap_or(0);
                    prop_assert!(w.row.0 < n, "row is a row of the equation's domain object");
                    let tgt_space = built.rows().get(&schema.arrows[&eq.lhs.0[eq.lhs.0.len() - 1]].tgt).map(Vec::len).unwrap_or(0);
                    prop_assert!(w.lhs.0 < tgt_space && w.rhs.0 < tgt_space, "composed values are rows of the path's target");
                }

                FindingKind::Violation(ViolationWitness::Key(w)) => {
                    prop_assert!(w.key < schema.keys.len(), "key index valid");
                    let key = &schema.keys[w.key];
                    let n = built.rows().get(&key.object).map(Vec::len).unwrap_or(0);
                    prop_assert!(
                        w.rows[0].0 < n && w.rows[1].0 < n,
                        "witness rows are rows of the key's object"
                    );
                    prop_assert!(w.rows[0] < w.rows[1], "witness rows ascending");
                    prop_assert_eq!(
                        finding.label == EpistemicLabel::Hypothesized,
                        key.status == ConstraintStatus::Proposed,
                        "the label mirrors the key's status"
                    );
                }
                FindingKind::Redundant(_)
                | FindingKind::Contradiction(_)
                | FindingKind::Underconstraint(_) => {
                    prop_assert!(
                        false,
                        "the analysis kinds have no kernel producer (P4's), got {:?}",
                        finding.kind
                    );
                }
            }
        }
    }

    /// A declared schema over an instance with zero rows evaluates
    /// nothing: zero findings even when every arrow and equation is
    /// populated.
    #[test]
    fn zero_row_instance_is_silent(shapes in proptest::collection::vec((0usize..3, 0usize..3), 1..=4)) {
        let case = build_case(&shapes, &[0, 0, 0], &[0; 4], &[], &[]);
        let (schema, instance) = common::fixture::assemble(&case);
        let built = TypedInstance::build(&schema, &instance);
        prop_assert_eq!(check(&schema, &built), vec![], "zero rows — zero evaluations, zero findings");
    }
}

/// Totality: the empty document — empty schema, empty instance — yields
/// zero findings. Silence is real success, not an unevaluated
/// declaration.
#[test]
fn empty_schema_and_empty_instance_are_silent() {
    let schema = Schema::default();
    let instance = Instance::default();
    let built = TypedInstance::build(&schema, &instance);
    assert_eq!(
        check(&schema, &built),
        vec![],
        "the empty document checks to zero findings"
    );
}

/// Self-loops are legal and evaluated like any arrow: a self-loop in the
/// middle of a path composes through its stored cell — a dangling cell in
/// the self-loop is reported and short-circuits its row, and when the
/// self-loop routes a row's two equation sides to different rows, the
/// difference is a witnessed violation on that row.
#[test]
fn self_loop_arrows_are_legal_and_evaluated() {
    // Objects A, B; arrows: a: A→A (self-loop), f: A→B, g: A→B; equation
    // f∘a = f (accepted). Endpoints: f∘a: A→A→B = A→B, f: A→B.
    let schema = self_loop_schema();

    // Case 1: the self-loop leaves row 0 dangling and maps row 1 to
    // itself; f maps both rows to B's only row. Row 0 cannot compose
    // (dangling, reported); row 1 composes equal (silent).
    let case_one = TypedInstance::build(
        &schema,
        &self_loop_instance(
            1,
            &[None, Some(RowId(1))],
            &[Some(RowId(0)), Some(RowId(0))],
            &[Some(RowId(0)), Some(RowId(0))],
        ),
    );
    assert_eq!(
        check(&schema, &case_one),
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Dangling(DanglingWitness {
                arrow: Ident("a".into()),
                src: RowId(0),
            }),
        }],
        "the self-loop composes like any arrow; its dangling cell speaks, the run continues"
    );

    // Case 2: two rows in B; the self-loop maps row 1 away to row 0 and
    // f maps row 1 to B's second row — row 1's two routes disagree, and
    // the violation witnesses the composition through the self-loop.
    let case_two = TypedInstance::build(
        &schema,
        &self_loop_instance(
            2,
            &[Some(RowId(0)), Some(RowId(0))],
            &[Some(RowId(0)), Some(RowId(1))],
            &[Some(RowId(0)), Some(RowId(1))],
        ),
    );
    // Row 0: f(a(0)) = f(0) = 0 vs f(0) = 0 — silent. Row 1:
    // f(a(1)) = f(0) = 0 vs f(1) = 1 — violation.
    assert_eq!(
        check(&schema, &case_two),
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: 0,
                row: RowId(1),
                lhs: RowId(0), // f(a(1)) = f(0) = 0
                rhs: RowId(1), // f(1) = 1
            })),
        }],
        "the self-loop's composition difference is a witnessed violation on row 1"
    );
}

/// The self-loop document: objects A, B; arrows a: A→A (self-loop),
/// f: A→B, g: A→B; the accepted equation f∘a = f. Endpoints: f∘a:
/// A→A→B = A→B, f: A→B.
fn self_loop_schema() -> Schema {
    let mut objects = BTreeMap::new();
    for name in ["A", "B"] {
        objects.insert(Ident(name.into()), ());
    }
    let mut arrows = BTreeMap::new();
    arrows.insert(
        Ident("a".into()),
        Arrow {
            src: Ident("A".into()),
            tgt: Ident("A".into()),
        },
    );
    arrows.insert(
        Ident("f".into()),
        Arrow {
            src: Ident("A".into()),
            tgt: Ident("B".into()),
        },
    );
    arrows.insert(
        Ident("g".into()),
        Arrow {
            src: Ident("A".into()),
            tgt: Ident("B".into()),
        },
    );
    let equations = vec![Equation {
        lhs: Path(vec![Ident("f".into()), Ident("a".into())]),
        rhs: Path(vec![Ident("f".into())]),
        rationale: "through the self-loop".into(),
        status: ConstraintStatus::Accepted,
    }];
    Schema {
        objects,
        arrows,
        equations,
        keys: Vec::new(),
    }
}

/// An instance over the self-loop schema: two A rows, `b_rows` B rows,
/// and the drawn cell tables for a, f, g.
fn self_loop_instance(
    b_rows: usize,
    a_cells: &[Option<RowId>],
    f_cells: &[Option<RowId>],
    g_cells: &[Option<RowId>],
) -> Instance {
    let mut rows = BTreeMap::new();
    rows.insert(Ident("A".into()), vec![RowId(0), RowId(1)]);
    rows.insert(Ident("B".into()), (0..b_rows).map(RowId).collect());
    let mut values = BTreeMap::new();
    values.insert(Ident("a".into()), a_cells.to_vec());
    values.insert(Ident("f".into()), f_cells.to_vec());
    values.insert(Ident("g".into()), g_cells.to_vec());
    Instance { rows, values }
}
