//! Purpose: the three schema-diff RED fixtures plus the classification
//! and bounded-honesty assertions (openspec change `add-schema-diff`,
//! tasks 1–3 and validation 9–12).
//!
//! Responsibilities: load the committed fixture pairs through the real
//! parser, call [`catherine_analysis::schema_diff::diff`] with explicit
//! bounds, and assert structurally — the classification outcome, the
//! finding through the P2 taxonomy, the witness instance's acceptance
//! under the real kernel `check` on both sides, and the scope on every
//! bounded claim. Never snapshots; a repeated run is compared against
//! itself.
//!
//! Rationale: the weakening kata is the agent-drift detector — the
//! assertion that matters is not "a class was returned" but "the
//! witness instance re-checks clean in v2 and rejected in v1". The
//! cosmetic kata is the negative-control silence, and the budget kata
//! pins that an expired budget is `unknown`, never equivalence.

use catherine_analysis::bounds::SearchBounds;
use catherine_analysis::schema_diff::{
    diff, EnforcedConstraint, SchemaDiffOutcome, SchemaDiffReport,
};
use catherine_core::checks;
use catherine_core::findings::{EpistemicLabel, FindingKind, ViolationWitness};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Equation, Ident, Path, Schema};
use catherine_core::parse::parse;
use std::collections::BTreeMap;

/// Load one side of a fixture pair through the real parser.
fn fixture(pair: &str, side: &str) -> Schema {
    let path = format!(
        "{}/../../fixtures/schema_diff/{pair}/{side}.ct.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("fixture exists");
    parse(&text).expect("fixture parses").schema
}

/// Explicit bounds: per-object row counts plus the candidate budget.
fn rows_bound(pairs: &[(&str, usize)], max_candidates: usize) -> SearchBounds {
    SearchBounds {
        max_rows: pairs
            .iter()
            .map(|(object, count)| (Ident(object.to_string()), *count))
            .collect::<BTreeMap<_, _>>(),
        max_candidates,
    }
}

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

/// The weakening katas: the classification, its single finding, and the
/// witness's honesty under the real kernel check on both schemas.
fn assert_weakened(v1: &Schema, v2: &Schema, report: &SchemaDiffReport) {
    assert_eq!(
        report.findings.len(),
        1,
        "weakening emits exactly one finding"
    );
    let finding = &report.findings[0];
    assert_eq!(finding.label, EpistemicLabel::Executed);
    match &finding.kind {
        FindingKind::Violation(ViolationWitness::Equation(witness)) => {
            assert_eq!(
                (
                    witness.equation,
                    witness.row.0,
                    witness.lhs.0,
                    witness.rhs.0
                ),
                (0, 0, 0, 1),
                "the dropped equation's row and composed values"
            );
        }
        other => panic!("equation violation expected, got {other:?}"),
    }
    let SchemaDiffOutcome::Weakened { witness } = &report.outcome else {
        panic!("weakened expected, got {:?}", report.outcome);
    };
    assert_eq!(
        witness.rows.get(&ident("Node")).map(Vec::len),
        Some(2),
        "the witness lives inside the declared bounds"
    );
    let accepted_by_v2 = checks::check(v2, &TypedInstance::build(v2, witness));
    assert!(
        accepted_by_v2.is_empty(),
        "v2 accepts the witness: {accepted_by_v2:?}"
    );
    let rejected_by_v1 = checks::check(v1, &TypedInstance::build(v1, witness));
    assert_eq!(
        rejected_by_v1.first(),
        Some(finding),
        "v1 rejects the witness with the reported finding first"
    );
}

/// The cosmetic and accreted katas share one structural assertion: the
/// presentation-only edit is silent and the added constraint is named.
fn assert_cosmetic_and_accreted() {
    let v1 = fixture("cosmetic", "v1");
    let v2 = fixture("cosmetic", "v2");
    let bound = rows_bound(&[("A", 1)], 8);
    let cosmetic = diff(&v1, &v2, &bound);
    assert!(
        matches!(cosmetic.outcome, SchemaDiffOutcome::Cosmetic),
        "rationale/order-only edit must be cosmetic, got {:?}",
        cosmetic.outcome
    );
    assert!(cosmetic.findings.is_empty(), "cosmetic edit is silent");

    let mut stronger = v1.clone();
    stronger.equations.push(Equation {
        lhs: Path(vec![ident("f")]),
        rhs: Path(vec![ident("f")]),
        rationale: "f equals itself".to_string(),
        status: ConstraintStatus::Accepted,
    });
    let accreted = diff(&v1, &stronger, &bound);
    match accreted.outcome {
        SchemaDiffOutcome::Accreted { added } => assert_eq!(
            added,
            vec![EnforcedConstraint::Equation {
                lhs: Path(vec![ident("f")]),
                rhs: Path(vec![ident("f")]),
            }],
            "the added enforced equation is named"
        ),
        other => panic!("accreted expected, got {other:?}"),
    }
    assert!(
        accreted.findings.is_empty(),
        "accretion is a report, not a finding"
    );
}

/// The mixed-edit honesty case: dropping a trivially-true constraint
/// while adding a real one must never be rounded into `accreted` — the
/// bounded scan saw strengthening and no weakening, and says exactly
/// that.
fn assert_strengthened_in_scope() {
    let mut v1 = fixture("weakened", "v2");
    v1.equations.push(Equation {
        lhs: Path(vec![ident("next")]),
        rhs: Path(vec![ident("next")]),
        rationale: "trivial constraint".to_string(),
        status: ConstraintStatus::Accepted,
    });
    let v2 = fixture("weakened", "v1");
    let report = diff(&v1, &v2, &rows_bound(&[("Node", 2)], 64));
    match &report.outcome {
        SchemaDiffOutcome::StrengthenedInScope { added, scope } => {
            assert_eq!(added.len(), 1, "the added constraint is named");
            assert_eq!(scope.candidates_explored, 9, "full exhaustion");
        }
        other => panic!("mixed edit must be strengthened-in-scope, got {other:?}"),
    }
    assert!(report.findings.is_empty());
}

#[test]
fn weakening_pair_is_caught_with_a_v2_valid_v1_invalid_witness() {
    let v1 = fixture("weakened", "v1");
    let v2 = fixture("weakened", "v2");
    let bound = rows_bound(&[("Node", 2)], 64);
    let report = diff(&v1, &v2, &bound);
    assert_weakened(&v1, &v2, &report);
    assert_eq!(
        diff(&v1, &v2, &bound),
        report,
        "repeated runs are identical"
    );
}

#[test]
fn refactor_is_equivalent_in_scope_and_cosmetic_edit_is_silent() {
    let v1 = fixture("refactor", "v1");
    let v2 = fixture("refactor", "v2");
    let bound = rows_bound(&[("A", 1), ("B", 1), ("C", 2)], 128);
    let report = diff(&v1, &v2, &bound);
    match &report.outcome {
        SchemaDiffOutcome::EquivalentInScope { scope } => {
            assert_eq!(scope.rows, bound.max_rows, "the scope is part of the claim");
            assert_eq!(
                scope.candidates_explored, 18,
                "full exhaustion (2 × 3 × 3 candidates), not a budget hit"
            );
        }
        other => panic!("refactor must prove equivalence in scope, got {other:?}"),
    }
    assert!(report.findings.is_empty(), "equivalence emits no finding");
    assert_cosmetic_and_accreted();
}

#[test]
fn budget_expiry_is_unknown_and_structural_change_is_refused() {
    let v1 = fixture("weakened", "v1");
    let v2 = fixture("weakened", "v2");
    let expired = diff(&v1, &v2, &rows_bound(&[("Node", 2)], 1));
    match &expired.outcome {
        SchemaDiffOutcome::Unknown { scope } => {
            assert_eq!(scope.candidates_explored, 1);
            assert_eq!(scope.rows.get(&ident("Node")), Some(&2));
        }
        other => panic!("an expired budget is UNKNOWN, got {other:?}"),
    }
    assert!(expired.findings.is_empty());

    let zero = diff(&v1, &v2, &rows_bound(&[], 64));
    match &zero.outcome {
        SchemaDiffOutcome::Unknown { scope } => {
            assert!(scope.rows.is_empty(), "zero-size bound has the empty scope");
            assert_eq!(scope.candidates_explored, 0);
        }
        other => panic!("zero-size bound is UNKNOWN, got {other:?}"),
    }

    let mut different = v2.clone();
    different.objects.insert(ident("Extra"), ());
    assert!(
        matches!(
            diff(&v1, &different, &rows_bound(&[("Node", 2)], 64)).outcome,
            SchemaDiffOutcome::Incomparable
        ),
        "different object vocabularies need a declared schema map"
    );
    assert_strengthened_in_scope();
}
