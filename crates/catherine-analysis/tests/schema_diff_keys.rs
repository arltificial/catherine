//! Purpose: declarative keys in schema-diff classification (format 1.3,
//! openspec change `add-declarative-keys`, design Decision 5): the
//! enforced set is accepted equations ∪ accepted keys, proposed keys are
//! presentation, and a dropped accepted key yields the existing
//! `violation`/`executed` finding with a `KeyWitness`.
//!
//! Responsibilities: pin `accreted` naming an added key, `weakened` with
//! a key witness whose instance re-checks clean in v2 and rejected in v1,
//! the cosmetic silence of a proposed-key edit, and that a proposed key
//! collision is `hypothesized` — never a satisfiability constraint.
//!
//! Rationale: keys are schema constraints, so adding one never widens the
//! instance space and dropping one is exactly the drift the weakening
//! witness must exhibit. No new classification machinery: the bounded
//! scan, the label rule, and the taxonomy are shared with equations.

use catherine_analysis::bounds::SearchBounds;
use catherine_analysis::schema_diff::{diff, EnforcedConstraint, SchemaDiffOutcome};
use catherine_analysis::search::{search, SearchOutcome};
use catherine_core::checks;
use catherine_core::findings::{EpistemicLabel, FindingKind, KeyWitness, ViolationWitness};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Arrow, ConstraintStatus, Ident, Key, Schema};
use std::collections::BTreeMap;

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

/// One object `A` with the endoarrow `f`, no equations, and the given
/// keys.
fn endo_schema(keys: Vec<Key>) -> Schema {
    let objects = [(ident("A"), ())].into_iter().collect();
    let arrows = [(
        ident("f"),
        Arrow {
            src: ident("A"),
            tgt: ident("A"),
        },
    )]
    .into_iter()
    .collect();
    Schema {
        objects,
        arrows,
        equations: Vec::new(),
        keys,
    }
}

/// The key `(A, [f])` with the given status.
fn key(status: ConstraintStatus) -> Key {
    Key {
        object: ident("A"),
        arrows: vec![ident("f")],
        status,
    }
}

/// Two rows of `A`, enough candidates to exhaust the shape.
fn bound() -> SearchBounds {
    SearchBounds {
        max_rows: [(ident("A"), 2)].into_iter().collect::<BTreeMap<_, _>>(),
        max_candidates: 64,
    }
}

/// The expected enforced key constraint.
fn enforced_key() -> EnforcedConstraint {
    EnforcedConstraint::Key {
        object: ident("A"),
        arrows: vec![ident("f")],
    }
}

/// Scenario "Constraints only added": an accepted key is enforced, so the
/// edit accretes with zero findings.
#[test]
fn added_key_is_accreted() {
    let v1 = endo_schema(Vec::new());
    let v2 = endo_schema(vec![key(ConstraintStatus::Accepted)]);
    let report = diff(&v1, &v2, &bound());
    match report.outcome {
        SchemaDiffOutcome::Accreted { added } => {
            assert_eq!(added, vec![enforced_key()], "the added key is named")
        }
        other => panic!("accreted expected, got {other:?}"),
    }
    assert!(report.findings.is_empty(), "accretion is a report");
}

/// Scenario "Silent constraint-weakening detected": dropping an accepted
/// key yields one executed violation with the key's canonical index and
/// the colliding rows; the witness re-checks clean in v2 and rejected in
/// v1.
#[test]
fn dropped_key_is_weakened_with_key_witness() {
    let v1 = endo_schema(vec![key(ConstraintStatus::Accepted)]);
    let v2 = endo_schema(Vec::new());
    let report = diff(&v1, &v2, &bound());
    assert_eq!(report.findings.len(), 1, "exactly one weakening finding");
    let finding = &report.findings[0];
    assert_eq!(finding.label, EpistemicLabel::Executed);
    match &finding.kind {
        FindingKind::Violation(ViolationWitness::Key(witness)) => assert_eq!(
            witness,
            &KeyWitness {
                key: 0,
                rows: [
                    catherine_core::model::RowId(0),
                    catherine_core::model::RowId(1)
                ],
            },
            "the dropped key's canonical index and its colliding pair"
        ),
        other => panic!("key violation expected, got {other:?}"),
    }
    let SchemaDiffOutcome::Weakened { witness } = &report.outcome else {
        panic!("weakened expected, got {:?}", report.outcome);
    };
    assert!(
        checks::check(&v2, &TypedInstance::build(&v2, witness)).is_empty(),
        "v2 accepts the witness"
    );
    assert_eq!(
        checks::check(&v1, &TypedInstance::build(&v1, witness)).first(),
        Some(finding),
        "v1 rejects the witness with the reported finding first"
    );
}

/// Scenario "Presentation-only edit is silent": a proposed key is not
/// enforced, so adding or dropping it never classifies the edit.
#[test]
fn proposed_key_edit_is_cosmetic() {
    let v1 = endo_schema(vec![key(ConstraintStatus::Proposed)]);
    let v2 = endo_schema(Vec::new());
    let report = diff(&v1, &v2, &bound());
    assert!(
        matches!(report.outcome, SchemaDiffOutcome::Cosmetic),
        "proposed keys are presentation, got {:?}",
        report.outcome
    );
    assert!(report.findings.is_empty());
}

/// Scenario "Accepted equations constrain, proposed equations question"
/// applied to keys: a proposed key collision is `hypothesized` — the
/// search returns a witness that violates it, because proposals never
/// block cleanliness.
#[test]
fn proposed_key_does_not_block_search_cleanliness() {
    let schema = endo_schema(vec![key(ConstraintStatus::Proposed)]);
    let report = search(&schema, &bound());
    let SearchOutcome::Witness { instance } = report.outcome else {
        panic!(
            "a proposed key never makes the space unsatisfiable: {:?}",
            report.outcome
        );
    };
    let findings = checks::check(&schema, &TypedInstance::build(&schema, &instance));
    assert!(
        findings
            .iter()
            .any(|finding| finding.label == EpistemicLabel::Hypothesized),
        "the witness violates the proposed key as a question"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.label != EpistemicLabel::Executed),
        "nothing executed blocks the witness"
    );
}
