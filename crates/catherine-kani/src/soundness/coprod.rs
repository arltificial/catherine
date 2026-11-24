//! Purpose: the coproduct family's witness obligations, split out of
//! `soundness.rs` under the complexity caps (beads `catherine-6g2`,
//! ledger `0007`) — coverage and disjointness re-derived independently
//! from the interned tables, sharing no code with the kernel walk. The
//! claim is split per witness shape (2026-11-24): coverage half and
//! collision half, one soundness harness each.

use catherine_engine::engine::{Bounded, EngineSchema, EngineTyped, Nid, ARROWS_CAP, ROWS_CAP};

/// The coproduct claim's coverage half, re-derived independently: the
/// arrow images resolved straight from the interned tables as scalar
/// facts, then the coverage claim. The claim is split per witness shape
/// (Decision 4, 2026-11-24), and each half's re-derivation is fused
/// (resolve and membership in one pass, nothing materialized): the
/// earlier rebuild of the walk's image table — even mirror-free —
/// quadrupled the check harness's formula (117 k generated VCCs); the
/// collision half is `assert_coproduct_collision_sound`'s and the two
/// harnesses' union carries the family's claim. Both declare unwind 4 —
/// the shape's concrete loop bound (2 trips + exit); the mirror-era 8
/// unrolled past need and peaked at 9.93 GB uncapped, over the scope.
pub(crate) fn assert_coproduct_coverage_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
) {
    let coproduct = schema.coproducts[index].as_ref().unwrap();
    let mut partial = false;
    let mut covered = false;
    let mut a = 0;
    while a < coproduct.arrows.len() {
        let (complete, covers) = arrow_image_facts(typed, coproduct.arrows[a], row);
        if !complete {
            partial = true;
        }
        if complete && covers {
            covered = true;
        }
        a += 1;
    }
    assert!(!partial);
    assert!(!covered);
}

/// The coproduct claim's collision half, re-derived independently (see
/// [`assert_coproduct_coverage_sound`] for the split and fusion
/// rationale): every covering image is a named injection, and at least
/// two named injections cover the row.
pub(crate) fn assert_coproduct_collision_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
    injections: &Bounded<Nid, ARROWS_CAP>,
) {
    let coproduct = schema.coproducts[index].as_ref().unwrap();
    let mut partial = false;
    let mut named_covering = 0;
    let mut a = 0;
    while a < coproduct.arrows.len() {
        let (complete, covers) = arrow_image_facts(typed, coproduct.arrows[a], row);
        if !complete {
            partial = true;
        }
        let is_named = injection_named(injections, coproduct.arrows[a]);
        if complete && covers {
            assert!(is_named || partial);
            if is_named {
                named_covering += 1;
            }
        } else if complete {
            assert!(!is_named || partial);
        }
        a += 1;
    }
    assert!(named_covering >= 2);
}

/// One arrow's image facts, resolved straight from the interned table:
/// `complete` — every cell resolved; `covers` — the row is among the
/// resolved cells, checked while resolving so no image table is
/// materialized (a `Bounded<(Nid, Bounded<u64, ROWS_CAP>), ARROWS_CAP>`
/// rebuild's nested-array stores and merged-lens scans quadrupled the
/// formula; these scalars carry the identical claims).
fn arrow_image_facts(typed: &EngineTyped, arrow: Nid, row: usize) -> (bool, bool) {
    let table = &typed.tables[arrow];
    let mut complete = true;
    let mut covers = false;
    let mut idx = 0;
    while idx < table.len() {
        match table.get(idx).copied().flatten() {
            Some(value) => covers |= value.0 as u64 == row as u64,
            None => complete = false,
        }
        idx += 1;
    }
    (complete, covers)
}

/// Plain slot scan: is the arrow one of the witness's named injections?
fn injection_named(injections: &Bounded<Nid, ARROWS_CAP>, arrow: Nid) -> bool {
    let mut k = 0;
    while k < injections.len() {
        if injections[k] == Some(arrow) {
            return true;
        }
        k += 1;
    }
    false
}
