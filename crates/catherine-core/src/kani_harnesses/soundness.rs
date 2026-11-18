//! Purpose: the witness-soundness re-evaluation half of the Kani
//! harnesses (openspec change `add-kani-verification-lane`) — every
//! finding `check` emits has its witness re-evaluated independently:
//! the claimed predicate is recomputed straight off the public
//! `Instance`/`Schema` fields, sharing no code with the kernel walk
//! (the Rust counterpart of `Catherine.Proofs.soundness`, verified on
//! the shipped artifact).
//!
//! Responsibilities: [`compose`] rebuilds path composition from the
//! public fields; the `assert_*_sound` family re-derives each witness
//! family's claim (equation disagreement, key collision, coproduct
//! coverage/disjointness, pullback commutes/covers/unique, dangling
//! cell); [`assert_finding_sound`] dispatches one finding to its
//! obligation.
//!
//! Rationale: an independent evaluator that reused the kernel's own
//! walk would prove nothing about it — the obligations here are written
//! against the witness payloads and the raw presentation tables, so a
//! kernel regression that emits a wrong witness cannot satisfy them.
//! Naturality/Redundant/Contradiction/Underconstraint have no `check`
//! producer (naturality is the separate `naturality` runner; the rest
//! are P4's) — recorded as out of this family's scope in
//! docs/kani-lane.md.

use super::shapes::rows_of;
use crate::findings::{Finding, FindingKind, ViolationWitness};
use crate::model::{Ident, Instance, Path, RowId, Schema};

/// Compose `path` starting at source row `start` — the kernel's walk
/// rebuilt independently from the public fields: each step resolves its
/// cell to an in-target value or the path fails to compose.
pub(super) fn compose(
    instance: &Instance,
    schema: &Schema,
    path: &Path,
    start: usize,
) -> Option<RowId> {
    let mut current = start;
    for name in &path.0 {
        let arrow = schema.arrows.get(name)?;
        if current >= rows_of(&arrow.src.0) {
            return None;
        }
        match super::cell(instance, &name.0, current)? {
            None => return None,
            Some(value) => {
                if value.0 >= rows_of(&arrow.tgt.0) {
                    return None;
                }
                current = value.0;
            }
        }
    }
    Some(RowId(current))
}

/// The equation claim: both sides recomposed from the tables land exactly
/// on the witness's values, and they disagree.
fn assert_equation_sound(
    schema: &Schema,
    instance: &Instance,
    witness: &crate::findings::EquationWitness,
) {
    let equation = &schema.equations[witness.equation];
    let lhs = compose(instance, schema, &equation.lhs, witness.row.0);
    let rhs = compose(instance, schema, &equation.rhs, witness.row.0);
    assert!(lhs == Some(witness.lhs));
    assert!(rhs == Some(witness.rhs));
    assert!(witness.lhs != witness.rhs);
}

/// The key claim: both colliding rows' key tuples fully resolve and are
/// equal, ascending as carried.
fn assert_key_sound(schema: &Schema, instance: &Instance, witness: &crate::findings::KeyWitness) {
    let key = &schema.keys[witness.key];
    let [r0, r1] = witness.rows;
    assert!(r0 < r1);
    let mut tuple0 = Vec::new();
    let mut tuple1 = Vec::new();
    for name in &key.arrows {
        tuple0.push(super::cell(instance, &name.0, r0.0));
        tuple1.push(super::cell(instance, &name.0, r1.0));
    }
    assert!(tuple0 == tuple1);
    assert!(tuple0.iter().all(|c| matches!(c, Some(Some(_)))));
}

/// The coproduct claim: coverage failure (`injections` empty) means the
/// row lies in no injection image; disjointness failure means every
/// named colliding injection covers the row and at least two images do.
fn assert_coproduct_sound(
    schema: &Schema,
    instance: &Instance,
    witness: &crate::findings::CoproductWitness,
) {
    let coproduct = &schema.coproducts[witness.coproduct];
    let mut hits = Vec::new();
    for name in &coproduct.arrows {
        let src = schema.arrows.get(name).unwrap().src.0.clone();
        let mut covers = false;
        for v in 0..rows_of(&src) {
            if super::cell(instance, &name.0, v) == Some(Some(witness.row)) {
                covers = true;
            }
        }
        if covers {
            hits.push(name.0.clone());
        }
    }
    if witness.injections.is_empty() {
        assert!(hits.is_empty());
    } else {
        for named in &witness.injections {
            assert!(hits.iter().any(|hit| hit == &named.0));
        }
        assert!(hits.len() >= 2);
    }
}

/// The commutes claim: both cospan compositions resolve and disagree
/// exactly as the witness carried them.
fn assert_pullback_commutes_sound(
    schema: &Schema,
    instance: &Instance,
    pullback: &crate::model::Pullback,
    row: RowId,
    lhs: RowId,
    rhs: RowId,
) {
    let p1 = pullback.legs[0].0.clone();
    let p2 = pullback.legs[1].0.clone();
    let f = pullback.cospan[0].0.clone();
    let g = pullback.cospan[1].0.clone();
    let left = compose(instance, schema, &Path(vec![Ident(p1), Ident(f)]), row.0);
    let right = compose(instance, schema, &Path(vec![Ident(p2), Ident(g)]), row.0);
    assert!(left == Some(lhs));
    assert!(right == Some(rhs));
    assert!(lhs != rhs);
}

/// The covers claim: the pair shares a cospan target, and no apex row
/// projects onto it through both legs.
fn assert_pullback_covers_sound(
    schema: &Schema,
    instance: &Instance,
    pullback: &crate::model::Pullback,
    pair: [RowId; 2],
) {
    let p1 = pullback.legs[0].0.clone();
    let p2 = pullback.legs[1].0.clone();
    let f = pullback.cospan[0].0.clone();
    let g = pullback.cospan[1].0.clone();
    let fa = compose(instance, schema, &Path(vec![Ident(f)]), pair[0].0);
    let gb = compose(instance, schema, &Path(vec![Ident(g)]), pair[1].0);
    assert!(fa.is_some());
    assert!(fa == gb);
    for apex in 0..rows_of(&pullback.apex.0) {
        let l = super::cell(instance, &p1, apex);
        let r = super::cell(instance, &p2, apex);
        assert!(!(l == Some(Some(pair[0])) && r == Some(Some(pair[1]))));
    }
}

/// The unique claim: two ascending apex rows resolve both leg projections
/// to the same values — the legs are not jointly injective there.
fn assert_pullback_unique_sound(
    schema: &Schema,
    instance: &Instance,
    pullback: &crate::model::Pullback,
    rows: [RowId; 2],
) {
    let [r0, r1] = rows;
    let p1 = pullback.legs[0].0.clone();
    let p2 = pullback.legs[1].0.clone();
    assert!(r0 < r1);
    let l0 = compose(instance, schema, &Path(vec![Ident(p1.clone())]), r0.0);
    let l1 = compose(instance, schema, &Path(vec![Ident(p1)]), r1.0);
    let r0v = compose(instance, schema, &Path(vec![Ident(p2.clone())]), r0.0);
    let r1v = compose(instance, schema, &Path(vec![Ident(p2)]), r1.0);
    assert!(l0.is_some() && l0 == l1);
    assert!(r0v.is_some() && r0v == r1v);
}

fn assert_pullback_sound(
    schema: &Schema,
    instance: &Instance,
    witness: &crate::findings::PullbackWitness,
) {
    let pullback = &schema.pullbacks[witness.pullback()];
    match witness {
        crate::findings::PullbackWitness::Commutes { row, lhs, rhs, .. } => {
            assert_pullback_commutes_sound(schema, instance, pullback, *row, *lhs, *rhs);
        }
        crate::findings::PullbackWitness::Covers { pair, .. } => {
            assert_pullback_covers_sound(schema, instance, pullback, *pair);
        }
        crate::findings::PullbackWitness::Unique { rows, .. } => {
            assert_pullback_unique_sound(schema, instance, pullback, *rows);
        }
    }
}

/// The independent soundness obligation for one finding: every witness
/// re-evaluated from the public fields fails exactly as claimed.
pub(super) fn assert_finding_sound(schema: &Schema, instance: &Instance, finding: &Finding) {
    match &finding.kind {
        FindingKind::Violation(ViolationWitness::Equation(witness)) => {
            assert_equation_sound(schema, instance, witness);
        }
        FindingKind::Violation(ViolationWitness::Key(witness)) => {
            assert_key_sound(schema, instance, witness);
        }
        FindingKind::Violation(ViolationWitness::Coproduct(witness)) => {
            assert_coproduct_sound(schema, instance, witness);
        }
        FindingKind::Violation(ViolationWitness::Pullback(witness)) => {
            assert_pullback_sound(schema, instance, witness);
        }
        FindingKind::Dangling(witness) => {
            // The dangling claim: that exact cell holds no row.
            assert!(super::cell(instance, &witness.arrow.0, witness.src.0) == Some(None));
        }
        FindingKind::Violation(ViolationWitness::Naturality(_))
        | FindingKind::Redundant(_)
        | FindingKind::Contradiction(_)
        | FindingKind::Underconstraint(_) => {
            // Unreachable from `check` — no producer in this walk.
        }
    }
}
