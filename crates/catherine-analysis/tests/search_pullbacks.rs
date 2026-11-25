//! Purpose: bounded search's contradiction witness when an accepted
//! pullback alone exhausts the bounds (openspec change
//! `add-pullback-checks`) — the finding names the pullback instead of
//! leaving `scope` empty.
//!
//! Responsibilities: pin that a bounded shape whose fiber is non-empty
//! while the apex has zero rows exhausts, and the contradiction finding
//! carries `pullbacks: [0]` with no equation, key, or coproduct indices;
//! pin the serialized witness as `{"scope":[],"pullbacks":[0]}`.
//!
//! Rationale: the square's coverage verdict is an executed constraint, so
//! exhaustion is a proof over every accepted family, not equations alone.
//! The witness must name the accepted constraints the search enforced.

use catherine_analysis::bounds::SearchBounds;
use catherine_analysis::search::{search, SearchOutcome};
use catherine_core::findings::{ContradictionWitness, EpistemicLabel, FindingKind};
use catherine_core::model::{Arrow, ConstraintStatus, Ident, Pullback, Schema};
use std::collections::BTreeMap;

fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

/// The pullback `P` over `p1: P → A`, `p2: P → B` and `f: A → C`,
/// `g: B → C`, accepted (an executed constraint).
fn pullback() -> Pullback {
    Pullback {
        apex: ident("P"),
        legs: vec![ident("p1"), ident("p2")],
        cospan: vec![ident("f"), ident("g")],
        status: ConstraintStatus::Accepted,
    }
}

/// Four objects and the square `P → A → C ← B ← P`; no equations, keys,
/// or coproducts — the accepted pullback is the only constraint.
fn schema() -> Schema {
    let objects = ["P", "A", "B", "C"]
        .iter()
        .map(|name| (ident(name), ()))
        .collect::<BTreeMap<_, _>>();
    let arrows = [
        ("p1", "P", "A"),
        ("p2", "P", "B"),
        ("f", "A", "C"),
        ("g", "B", "C"),
    ]
    .iter()
    .map(|(name, src, tgt)| {
        (
            ident(name),
            Arrow {
                src: ident(src),
                tgt: ident(tgt),
            },
        )
    })
    .collect::<BTreeMap<_, _>>();
    Schema {
        objects,
        arrows,
        equations: Vec::new(),
        keys: Vec::new(),
        coproducts: Vec::new(),
        pullbacks: vec![pullback()],
        stored: BTreeMap::new(),
    }
}

/// One row each in `A`, `B`, `C` and no row in the apex `P`: the fiber
/// pair `(0, 0)` is non-empty, but no apex row can cover it, so every
/// fully resolved candidate fails the accepted pullback's coverage.
fn bound() -> SearchBounds {
    let max_rows = [("P", 0), ("A", 1), ("B", 1), ("C", 1)]
        .iter()
        .map(|(name, rows)| (ident(name), *rows))
        .collect::<BTreeMap<_, _>>();
    SearchBounds {
        max_rows,
        max_candidates: 100,
    }
}

#[test]
fn pullback_only_contradiction_names_the_pullback() {
    let report = search(&schema(), &bound());
    let finding = match report.outcome {
        SearchOutcome::Contradiction { finding } => finding,
        other => panic!("expected exhaustion to prove a contradiction, got {other:?}"),
    };
    assert_eq!(finding.label, EpistemicLabel::Executed);
    match &finding.kind {
        FindingKind::Contradiction(ContradictionWitness {
            scope,
            keys,
            coproducts,
            pullbacks,
        }) => {
            assert!(scope.is_empty(), "no equations are enforced");
            assert!(keys.is_empty(), "no keys are enforced");
            assert!(coproducts.is_empty(), "no coproducts are enforced");
            assert_eq!(pullbacks.as_slice(), &[0], "the accepted pullback is named");
        }
        other => panic!("expected a contradiction witness, got {other:?}"),
    }
    assert_eq!(
        serde_json::to_string(&finding).expect("finding serializes"),
        r#"{"label":"executed","kind":"contradiction","witness":{"scope":[],"pullbacks":[0]}}"#,
        "empty families are omitted from the pullback-only witness"
    );
}
