//! Purpose: pullbacks in schema-diff classification (openspec change
//! `add-pullback-checks`, slice 3): the enforced set becomes accepted
//! equations ∪ accepted keys ∪ accepted coproducts ∪ accepted pullbacks,
//! so adding an accepted pullback is `accreted`, dropping one is caught
//! by the existing bounded weakening scan, and a proposed pullback is
//! presentation.
//!
//! Responsibilities: pin `accreted` naming the added pullback;
//! `weakened` with a witness instance that fails the square's commutes
//! claim — v2-clean and v1-rejected by a pullback violation naming
//! declaration 0; the cosmetic silence of a proposed-pullback edit; and
//! that a proposed pullback never blocks search cleanliness (its failure
//! is a hypothesis, not a constraint).
//!
//! Rationale: no new classification machinery — the kernel's satisfaction
//! predicate already treats `executed` violations as unclean, so the scan
//! needs only the enlarged constraint set (the coproduct change's
//! integration shape, reused verbatim). The witness is asserted
//! structurally: v2 accepts an instance whose square does not commute and
//! the v1 declaration rejects it.

use catherine_analysis::bounds::SearchBounds;
use catherine_analysis::schema_diff::{diff, EnforcedConstraint, SchemaDiffOutcome};
use catherine_analysis::search::{search, SearchOutcome};
use catherine_core::checks;
use catherine_core::findings::{EpistemicLabel, FindingKind, PullbackWitness, ViolationWitness};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Arrow, ConstraintStatus, Ident, Pullback, Schema};
use std::collections::BTreeMap;

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

/// The pullback `P` over `p1: P → A`, `p2: P → B` and `f: A → C`,
/// `g: B → C` with the given status.
fn pullback(status: ConstraintStatus) -> Pullback {
    Pullback {
        apex: ident("P"),
        legs: vec![ident("p1"), ident("p2")],
        cospan: vec![ident("f"), ident("g")],
        status,
    }
}

/// Four objects and the square `P → A → C ← B ← P`.
fn schema(pullbacks: Vec<Pullback>) -> Schema {
    let mut objects = BTreeMap::new();
    for name in ["P", "A", "B", "C"] {
        objects.insert(ident(name), ());
    }
    let mut arrows = BTreeMap::new();
    for (name, src, tgt) in [
        ("p1", "P", "A"),
        ("p2", "P", "B"),
        ("f", "A", "C"),
        ("g", "B", "C"),
    ] {
        arrows.insert(
            ident(name),
            Arrow {
                src: ident(src),
                tgt: ident(tgt),
            },
        );
    }
    Schema {
        objects,
        arrows,
        equations: Vec::new(),
        keys: Vec::new(),
        coproducts: Vec::new(),
        pullbacks,
        stored: BTreeMap::new(),
    }
}

/// Enough rows for a two-row `C` table and one row in each of the other
/// objects: the first fully resolved candidate commutes, so the scan
/// proceeds to a candidate whose two composed cospan values disagree.
fn bound() -> SearchBounds {
    let max_rows = [("P", 1), ("A", 1), ("B", 1), ("C", 2)]
        .into_iter()
        .map(|(name, rows)| (ident(name), rows))
        .collect::<BTreeMap<_, _>>();
    SearchBounds {
        max_rows,
        max_candidates: 1000,
    }
}

/// Two `A` rows: the first fully resolved candidate leaves the second
/// `A` row uncovered by the apex's projection, so the proposed square's
/// coverage claim fails on the witness.
fn bound_with_two_a_rows() -> SearchBounds {
    let max_rows = [("P", 1), ("A", 2), ("B", 1), ("C", 2)]
        .into_iter()
        .map(|(name, rows)| (ident(name), rows))
        .collect::<BTreeMap<_, _>>();
    SearchBounds {
        max_rows,
        max_candidates: 1000,
    }
}

/// Scenario "Constraints only added": an accepted pullback is enforced,
/// so the edit accretes with zero findings and names the new constraint.
#[test]
fn accreted_pullback_diff() {
    let report = diff(
        &schema(Vec::new()),
        &schema(vec![pullback(ConstraintStatus::Accepted)]),
        &bound(),
    );
    match report.outcome {
        SchemaDiffOutcome::Accreted { added } => assert_eq!(
            added,
            vec![EnforcedConstraint::Pullback {
                apex: ident("P"),
                legs: vec![ident("p1"), ident("p2")],
                cospan: vec![ident("f"), ident("g")],
            }],
            "the added pullback is named"
        ),
        other => panic!("accreted expected, got {other:?}"),
    }
    assert!(report.findings.is_empty(), "accretion is a report");
}

/// Scenario "Silent constraint-weakening detected": dropping an accepted
/// pullback yields one executed pullback violation, and its witness
/// instance is accepted by v2 (the square need not commute) and rejected
/// by v1's commutes check, naming declaration 0.
#[test]
fn dropped_pullback_is_weakened_with_a_witness() {
    let v1 = schema(vec![pullback(ConstraintStatus::Accepted)]);
    let v2 = schema(Vec::new());
    let report = diff(&v1, &v2, &bound());
    let SchemaDiffOutcome::Weakened { witness } = &report.outcome else {
        panic!("weakened expected, got {:?}", report.outcome);
    };
    assert_eq!(report.findings.len(), 1, "exactly one weakening finding");
    assert_eq!(report.findings[0].label, EpistemicLabel::Executed);
    let FindingKind::Violation(witness_kind) = &report.findings[0].kind else {
        panic!(
            "the lost constraint is a violation finding: {:?}",
            report.findings[0]
        );
    };
    let ViolationWitness::Pullback(square) = witness_kind else {
        panic!("the lost constraint is a pullback witness: {witness_kind:?}");
    };
    assert_eq!(
        square.pullback(),
        0,
        "the witness names declaration index 0"
    );
    assert!(
        checks::check(&v2, &TypedInstance::build(&v2, witness)).is_empty(),
        "v2 accepts the non-commuting square"
    );
    let rejected = checks::check(&v1, &TypedInstance::build(&v1, witness));
    let non_commuting = rejected.iter().any(|finding| {
        matches!(
            &finding.kind,
            FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Commutes { .. }))
        )
    });
    assert!(
        non_commuting,
        "v1 rejects the witness with a commutes violation: {rejected:?}"
    );
}

/// Scenario "Presentation-only edit is silent": a proposed pullback is
/// not enforced, so dropping it never classifies the edit.
#[test]
fn proposed_pullback_edit_is_cosmetic() {
    let report = diff(
        &schema(vec![pullback(ConstraintStatus::Proposed)]),
        &schema(Vec::new()),
        &bound(),
    );
    assert!(
        matches!(report.outcome, SchemaDiffOutcome::Cosmetic),
        "proposed pullbacks are presentation, got {:?}",
        report.outcome
    );
    assert!(report.findings.is_empty());
}

/// Scenario "Accepted equations constrain, proposed equations question"
/// applied to pullbacks: the smallest fully resolved witness leaves a
/// proposed pullback's coverage claim false as a `hypothesized`
/// question, and nothing executed blocks the witness.
#[test]
fn proposed_pullback_does_not_block_search_cleanliness() {
    let schema = schema(vec![pullback(ConstraintStatus::Proposed)]);
    let report = search(&schema, &bound_with_two_a_rows());
    let SearchOutcome::Witness { instance } = report.outcome else {
        panic!(
            "a proposed pullback never makes the space unsatisfiable: {:?}",
            report.outcome
        );
    };
    let findings = checks::check(&schema, &TypedInstance::build(&schema, &instance));
    let failed_as_question = findings.iter().any(|finding| {
        if finding.label != EpistemicLabel::Hypothesized {
            return false;
        }
        matches!(
            &finding.kind,
            FindingKind::Violation(ViolationWitness::Pullback(PullbackWitness::Covers { .. }))
        )
    });
    assert!(
        failed_as_question,
        "the witness violates the proposed pullback as a question: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.label != EpistemicLabel::Executed),
        "nothing executed blocks the witness"
    );
}
