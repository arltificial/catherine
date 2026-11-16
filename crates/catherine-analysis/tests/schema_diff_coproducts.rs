//! Purpose: coproducts in schema-diff classification (openspec change
//! `add-coproduct-exhaustiveness`, design Decision 3): the enforced set
//! becomes accepted equations ∪ accepted keys ∪ accepted coproducts, so
//! adding an accepted coproduct is `accreted`, dropping one is caught by
//! the existing bounded weakening scan, and a proposed coproduct is
//! presentation.
//!
//! Responsibilities: pin `accreted` naming the added coproduct;
//! `weakened` with a witness instance that carries a rogue variant —
//! v2-clean and v1-rejected by a coproduct violation; and the cosmetic
//! silence of a proposed-coproduct edit.
//!
//! Rationale: no new classification machinery — the kernel's satisfaction
//! predicate already treats `executed` violations as unclean, so the scan
//! needs only the enlarged constraint set (the keys change's integration
//! shape, reused verbatim). The witness is asserted structurally: v2
//! accepts a rogue variant the v1 declaration rejects.

use catherine_analysis::bounds::SearchBounds;
use catherine_analysis::schema_diff::{diff, EnforcedConstraint, SchemaDiffOutcome};
use catherine_core::checks;
use catherine_core::findings::{EpistemicLabel, FindingKind, ViolationWitness};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Arrow, ConstraintStatus, Coproduct, Ident, Schema};
use std::collections::BTreeMap;

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

/// The coproduct `C = V1 + V2` with the given status.
fn coproduct(status: ConstraintStatus) -> Coproduct {
    Coproduct {
        object: ident("C"),
        arrows: vec![ident("i1"), ident("i2")],
        status,
    }
}

/// Three objects and the two injections `i1: V1 -> C`, `i2: V2 -> C`.
fn schema(coproducts: Vec<Coproduct>) -> Schema {
    let mut objects = BTreeMap::new();
    for name in ["C", "V1", "V2"] {
        objects.insert(ident(name), ());
    }
    let mut arrows = BTreeMap::new();
    arrows.insert(
        ident("i1"),
        Arrow {
            src: ident("V1"),
            tgt: ident("C"),
        },
    );
    arrows.insert(
        ident("i2"),
        Arrow {
            src: ident("V2"),
            tgt: ident("C"),
        },
    );
    Schema {
        objects,
        arrows,
        equations: Vec::new(),
        keys: Vec::new(),
        coproducts,
    }
}

/// Enough rows for a three-row `C` table and both injection sources.
fn bound() -> SearchBounds {
    let max_rows = [("C", 3), ("V1", 2), ("V2", 2)]
        .into_iter()
        .map(|(name, rows)| (ident(name), rows))
        .collect::<BTreeMap<_, _>>();
    SearchBounds {
        max_rows,
        max_candidates: 256,
    }
}

/// Scenario "Constraints only added": an accepted coproduct is enforced,
/// so the edit accretes with zero findings and names the new constraint.
#[test]
fn added_coproduct_is_accreted() {
    let report = diff(
        &schema(Vec::new()),
        &schema(vec![coproduct(ConstraintStatus::Accepted)]),
        &bound(),
    );
    match report.outcome {
        SchemaDiffOutcome::Accreted { added } => assert_eq!(
            added,
            vec![EnforcedConstraint::Coproduct {
                object: ident("C"),
                arrows: vec![ident("i1"), ident("i2")],
            }],
            "the added coproduct is named"
        ),
        other => panic!("accreted expected, got {other:?}"),
    }
    assert!(report.findings.is_empty(), "accretion is a report");
}

/// Scenario "Silent constraint-weakening detected": dropping an accepted
/// coproduct yields one executed coproduct violation, and its witness
/// instance is accepted by v2 (rogue variant preserved) and rejected by
/// v1.
#[test]
fn dropped_coproduct_is_weakened_with_a_rogue_variant_witness() {
    let v1 = schema(vec![coproduct(ConstraintStatus::Accepted)]);
    let v2 = schema(Vec::new());
    let report = diff(&v1, &v2, &bound());
    let SchemaDiffOutcome::Weakened { witness } = &report.outcome else {
        panic!("weakened expected, got {:?}", report.outcome);
    };
    assert_eq!(report.findings.len(), 1, "exactly one weakening finding");
    assert_eq!(report.findings[0].label, EpistemicLabel::Executed);
    assert!(
        matches!(
            &report.findings[0].kind,
            FindingKind::Violation(ViolationWitness::Coproduct(_))
        ),
        "the lost constraint is named by a coproduct witness: {:?}",
        report.findings[0]
    );
    assert_eq!(
        witness.rows.get(&ident("C")).map(Vec::len),
        Some(3),
        "the witness's C table carries the rogue third variant"
    );
    assert!(
        checks::check(&v2, &TypedInstance::build(&v2, witness)).is_empty(),
        "v2 accepts the rogue variant"
    );
    let rejected = checks::check(&v1, &TypedInstance::build(&v1, witness));
    let uncovered = rejected.iter().any(|finding| {
        matches!(
            &finding.kind,
            FindingKind::Violation(ViolationWitness::Coproduct(witness))
                if witness.injections.is_empty()
        )
    });
    assert!(
        uncovered,
        "v1 rejects the witness with an uncovered rogue variant: {rejected:?}"
    );
}

/// Scenario "Presentation-only edit is silent": a proposed coproduct is
/// not enforced, so adding or dropping it never classifies the edit.
#[test]
fn proposed_coproduct_edit_is_cosmetic() {
    let report = diff(
        &schema(vec![coproduct(ConstraintStatus::Proposed)]),
        &schema(Vec::new()),
        &bound(),
    );
    assert!(
        matches!(report.outcome, SchemaDiffOutcome::Cosmetic),
        "proposed coproducts are presentation, got {:?}",
        report.outcome
    );
    assert!(report.findings.is_empty());
}
