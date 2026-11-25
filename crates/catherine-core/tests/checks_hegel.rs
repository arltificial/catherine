//! Purpose: the executed-checks properties (openspec change
//! `add-executed-checks`, task 10, reshaped onto the findings taxonomy by
//! `add-findings-taxonomy`) — totality (empty schema or empty instance →
//! zero findings), self-loops legal and evaluated like any arrow, and the
//! findings matching an independent recomputation of the check contract
//! over arbitrary well-typed models. Findings are asserted structurally —
//! kind + witness + label — never byte snapshots; the vendored parity sets
//! in `fixtures/parity/` are the parity contract, not snapshots.
//!
//! Responsibilities: hold the three drawn properties — the contract
//! recomputation, the in-scope-witness soundness sweep and the zero-row
//! totality property — plus the two deterministic worlds the generator
//! cannot express cheaply: the empty document and the self-loop document.
//! The model generator itself lives in `common::generators` so the
//! findings-taxonomy properties draw the same one.
//!
//! Rationale: the properties are the semantic oracle of `check`, so they
//! assert on parsed values (kinds, witnesses, labels), never on rendered
//! bytes; the deterministic cases pin the two boundary documents that a
//! random draw reaches only by luck.

mod common;
#[path = "checks_properties/expected.rs"]
mod expected;
#[path = "checks_properties/hegel_scope.rs"]
mod scope;

use common::fixture;
use common::generators::arb_case;
use expected::expected_findings;

use catherine_core::checks::check;
use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Path, RowId, Schema,
};

use hegel::generators as gs;
use hegel::TestCase;
use std::collections::BTreeMap;

/// The findings match the spec recomputation, structurally, over
/// arbitrary well-typed models (self-loop arrows included — drawn shapes
/// carry src == tgt arrows and are evaluated like any arrow).
#[hegel::test]
fn findings_match_the_contract_recomputation(tc: TestCase) {
    let case = tc.draw(arb_case());
    tc.note(&format!("case: {case:?}"));
    let (schema, instance) = fixture::assemble(&case);
    let built = TypedInstance::build(&schema, &instance);
    assert_eq!(
        check(&schema, &built),
        expected_findings(&schema, &built),
        "check matches the independent recomputation (self-loops included)"
    );
}

/// Every finding's witnesses stay in scope: dangling names a declared
/// arrow and a row of its source object; violations name a valid equation
/// index, a row of the equation's domain, and composed values — never
/// witnesses without rows. Labels are epistemically honest: only
/// executed/hypothesized, mirroring equation status; the analysis kinds
/// have no producer in this crate.
#[hegel::test]
fn every_finding_carries_in_scope_witnesses(tc: TestCase) {
    let case = tc.draw(arb_case());
    tc.note(&format!("case: {case:?}"));
    let (schema, instance) = fixture::assemble(&case);
    let built = TypedInstance::build(&schema, &instance);
    for finding in check(&schema, &built) {
        assert!(
            matches!(
                finding.label,
                EpistemicLabel::Executed | EpistemicLabel::Hypothesized
            ),
            "kernel labels are executed/hypothesized only, got {:?}",
            finding.label
        );
        scope::assert_witness_in_scope((&schema, &built), &finding);
    }
}

/// A declared schema over an instance with zero rows evaluates
/// nothing: zero findings even when every arrow and equation is
/// populated.
#[hegel::test]
fn zero_row_instance_is_silent(tc: TestCase) {
    let shapes: Vec<(usize, usize)> = tc.draw(
        gs::vecs(hegel::tuples!(
            gs::integers::<usize>().min_value(0).max_value(2),
            gs::integers::<usize>().min_value(0).max_value(2),
        ))
        .min_size(1)
        .max_size(4),
    );
    tc.note(&format!("shapes: {shapes:?}"));
    let case = fixture::build_case(fixture::DrawnParts {
        shapes,
        row_counts: vec![0, 0, 0],
        cell_pool: vec![0; 4],
        eq_seeds: Vec::new(),
        statuses: Vec::new(),
    });
    let (schema, instance) = fixture::assemble(&case);
    let built = TypedInstance::build(&schema, &instance);
    assert_eq!(
        check(&schema, &built),
        vec![],
        "zero rows — zero evaluations, zero findings"
    );
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
        &self_loop_instance(SelfLoopCells {
            b_rows: 1,
            a: &[None, Some(RowId(1))],
            f: &[Some(RowId(0)), Some(RowId(0))],
            g: &[Some(RowId(0)), Some(RowId(0))],
        }),
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
        &self_loop_instance(SelfLoopCells {
            b_rows: 2,
            a: &[Some(RowId(0)), Some(RowId(0))],
            f: &[Some(RowId(0)), Some(RowId(1))],
            g: &[Some(RowId(0)), Some(RowId(1))],
        }),
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
    arrows.insert(Ident("a".into()), self_loop_arrow("A", "A"));
    arrows.insert(Ident("f".into()), self_loop_arrow("A", "B"));
    arrows.insert(Ident("g".into()), self_loop_arrow("A", "B"));
    let equations = vec![Equation {
        lhs: self_loop_path(&["f", "a"]),
        rhs: self_loop_path(&["f"]),
        rationale: "through the self-loop".into(),
        status: ConstraintStatus::Accepted,
    }];
    Schema {
        objects,
        arrows,
        equations,
        keys: Vec::new(),
        coproducts: Vec::new(),
        pullbacks: Vec::new(),
        stored: BTreeMap::new(),
    }
}

/// One self-loop schema arrow from its endpoint object names.
fn self_loop_arrow(src: &str, tgt: &str) -> Arrow {
    Arrow {
        src: Ident(src.into()),
        tgt: Ident(tgt.into()),
    }
}

/// One self-loop schema path from its arrow names.
fn self_loop_path(names: &[&str]) -> Path {
    Path(names.iter().map(|name| Ident((*name).into())).collect())
}

/// The drawn cell tables for a self-loop instance over `self_loop_schema`.
struct SelfLoopCells<'a> {
    b_rows: usize,
    a: &'a [Option<RowId>],
    f: &'a [Option<RowId>],
    g: &'a [Option<RowId>],
}

/// An instance over the self-loop schema: two A rows, `b_rows` B rows,
/// and the drawn cell tables for a, f, g.
fn self_loop_instance(cells: SelfLoopCells<'_>) -> Instance {
    let SelfLoopCells { b_rows, a, f, g } = cells;
    let mut rows = BTreeMap::new();
    rows.insert(Ident("A".into()), vec![RowId(0), RowId(1)]);
    rows.insert(Ident("B".into()), (0..b_rows).map(RowId).collect());
    let mut values = BTreeMap::new();
    values.insert(Ident("a".into()), a.to_vec());
    values.insert(Ident("f".into()), f.to_vec());
    values.insert(Ident("g".into()), g.to_vec());
    Instance { rows, values }
}
