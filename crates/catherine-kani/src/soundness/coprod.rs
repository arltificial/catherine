//! Purpose: the coproduct family's witness obligations, split out of
//! `soundness.rs` under the complexity caps (beads `catherine-6g2`,
//! ledger `0007`) — coverage and disjointness re-derived independently
//! from the interned tables, sharing no code with the kernel walk. The
//! claim is split per witness shape (2026-11-24): coverage half and
//! collision half, one soundness harness each.

use catherine_engine::engine::{Bounded, EngineTyped, Nid, RowId, ARROWS_CAP, ROWS_CAP};

/// The coproduct claim's coverage half, re-derived independently: the
/// arrow images resolved straight from the interned tables as scalar
/// facts, then the coverage claim. The claim is split per witness shape
/// (Decision 4, 2026-11-24), and each half's re-derivation is fused
/// per arrow (resolve and membership in one claim, nothing
/// materialized): the
/// earlier rebuild of the walk's image table — even mirror-free —
/// quadrupled the check harness's formula (117 k generated VCCs); the
/// collision half is `assert_coproduct_collision_sound`'s and the two
/// harnesses' union carries the family's claim. Both declare unwind 4 —
/// the shape's concrete loop bound (2 trips + exit); the mirror-era 8
/// unrolled past need and peaked at 9.93 GB uncapped, over the scope.
pub(crate) fn assert_coproduct_coverage_sound(ctx: &super::Ctx, row: usize) {
    let coproduct = ctx.schema.coproducts[ctx.index].as_ref().unwrap();
    for a in 0..coproduct.arrows.len() {
        let (complete, covers) = arrow_image_facts(ctx.typed, coproduct.arrows[a], row);
        assert!(complete);
        assert!(!covers);
    }
}

/// The coproduct claim's collision half, re-derived independently (see
/// [`assert_coproduct_coverage_sound`] for the split and fusion
/// rationale): every covering image is a named injection, and at least
/// two named injections cover the row.
pub(crate) fn assert_coproduct_collision_sound(
    ctx: &super::Ctx,
    row: usize,
    injections: &Bounded<Nid, ARROWS_CAP>,
) {
    let coproduct = ctx.schema.coproducts[ctx.index].as_ref().unwrap();
    let mut named_covering = 0;
    for a in 0..coproduct.arrows.len() {
        let (complete, covers) = arrow_image_facts(ctx.typed, coproduct.arrows[a], row);
        let is_named = injection_named(injections, coproduct.arrows[a]);
        assert!(complete);
        if covers {
            assert!(is_named);
            named_covering += 1;
        } else {
            assert!(!is_named);
        }
    }
    assert!(named_covering >= 2);
}

/// One arrow's image facts, resolved straight from the interned table:
/// `complete` — every cell resolved; `covers` — the row is among the
/// resolved cells. Two plain slot scans, nothing materialized (a
/// `Bounded<(Nid, Bounded<u64, ROWS_CAP>), ARROWS_CAP>` rebuild's
/// nested-array stores and merged-lens scans quadrupled the formula;
/// these scalars carry the identical claims).
fn arrow_image_facts(typed: &EngineTyped, arrow: Nid, row: usize) -> (bool, bool) {
    let table = &typed.tables[arrow];
    (table_complete(table), table_covers(table, row))
}

/// Every cell of the arrow's table resolved (no dangling cells).
fn table_complete(table: &Bounded<RowId, ROWS_CAP>) -> bool {
    for idx in 0..table.len() {
        if table.get(idx).copied().flatten().is_none() {
            return false;
        }
    }
    true
}

/// The row is among the arrow's resolved cells.
fn table_covers(table: &Bounded<RowId, ROWS_CAP>, row: usize) -> bool {
    for idx in 0..table.len() {
        if let Some(value) = table.get(idx).copied().flatten() {
            if value.0 as u64 == row as u64 {
                return true;
            }
        }
    }
    false
}

/// Plain slot scan: is the arrow one of the witness's named injections?
fn injection_named(injections: &Bounded<Nid, ARROWS_CAP>, arrow: Nid) -> bool {
    for k in 0..injections.len() {
        if injections[k] == Some(arrow) {
            return true;
        }
    }
    false
}
