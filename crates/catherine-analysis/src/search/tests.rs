use super::*;
use crate::bounds::SearchBounds;
use catherine_core::model::{Arrow, Equation, Path};

/// One object `a` with the endoarrow `f`; the accepted equation
/// `f;f;f = f` (holds on the swap, on constants, on identities) and
/// the proposed equation `f = f;f` (rejects the swap — it is a
/// question, not a constraint).
fn swap_schema() -> Schema {
    let objects = BTreeMap::from([(Ident("a".to_string()), ())]);
    let arrows = BTreeMap::from([(
        Ident("f".to_string()),
        Arrow {
            src: Ident("a".to_string()),
            tgt: Ident("a".to_string()),
        },
    )]);
    let eq = |lhs: &[&str], rhs: &[&str], status| Equation {
        lhs: Path(lhs.iter().map(|s| Ident(s.to_string())).collect()),
        rhs: Path(rhs.iter().map(|s| Ident(s.to_string())).collect()),
        rationale: String::new(),
        status,
    };
    Schema {
        objects,
        arrows,
        equations: vec![
            eq(&["f", "f", "f"], &["f"], ConstraintStatus::Accepted),
            eq(&["f"], &["f", "f"], ConstraintStatus::Proposed),
        ],
        keys: Vec::new(),
        coproducts: Vec::new(),
        pullbacks: Vec::new(),
        stored: BTreeMap::new(),
    }
}

fn two_row_bound() -> SearchBounds {
    let max_rows = BTreeMap::from([(Ident("a".to_string()), 2)]);
    SearchBounds {
        max_rows,
        max_candidates: 1000,
    }
}

/// The swap instance: fully resolved, the accepted equation holds
/// (`f³ = f` on an involution), the proposed equation fails
/// (`f ≠ f²`). `is_clean` judges satisfiability by the *accepted*
/// equations only — a hypothesized violation marks a question under
/// review, never a blocked witness.
#[test]
fn is_clean_accepts_accepted_clean_proposed_dirty() {
    let schema = swap_schema();
    let values = BTreeMap::from([(Ident("f".to_string()), vec![Some(RowId(1)), Some(RowId(0))])]);
    let instance = Instance {
        rows: BTreeMap::from([(Ident("a".to_string()), vec![RowId(0), RowId(1)])]),
        values,
    };
    assert!(is_clean(&schema, &instance));
}

/// End to end: the same schema under search returns a Witness, never
/// a Contradiction — a proposed-equation violation must not block
/// the outcome even if every enumerated candidate carried one.
#[test]
fn search_returns_witness_despite_proposed_violations() {
    let schema = swap_schema();
    let report = search(&schema, &two_row_bound());
    assert!(
        matches!(report.outcome, SearchOutcome::Witness { .. }),
        "expected a witness, got {:?}",
        report.outcome
    );
}
