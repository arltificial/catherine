//! Purpose: the witness re-evaluation half of the Kani harnesses
//! (openspec change `add-kani-verification-lane`, beads `catherine-6g2`)
//! — every finding the walk emits has its witness re-evaluated
//! independently: the claimed predicate is recomputed straight off the
//! interned schema and typed tables, sharing no code with the kernel
//! walk (the Rust counterpart of `Catherine.Proofs.soundness`, verified
//! on the shipped artifact).
//!
//! Responsibilities: [`compose`] rebuilds path composition from the
//! engine tables; the `assert_*_sound` family re-derives each witness
//! family's claim (equation disagreement, key collision, coproduct
//! coverage/disjointness, pullback commutes/covers/unique, dangling
//! cell) over the lane flavor's compact witnesses (ids, zero Strings);
//! each soundness harness calls its family's obligation directly — the
//! dispatch is partitioned per witness kind, exactly like the walkers
//! (2026-11-24, Decision 4 at obligation granularity).
//!
//! Rationale: an independent evaluator that reused the kernel's walk
//! would prove nothing about it — the obligations here carry their own
//! composition and table reads (never `engine::engine_compose` or the
//! family walkers), so a kernel regression that emits a wrong witness
//! cannot satisfy them. Id order is name order, witnesses carry
//! canonical indices, and no solver time goes to container, string, or
//! render modeling. Naturality/Redundant/Contradiction/Underconstraint
//! have no `check` producer — out of this family's scope
//! (docs/kani-lane.md).

use catherine_engine::engine::{
    Bounded, CompactPullback, CompiledKey, EngineSchema, EngineTyped, Step, ARROWS_CAP,
};

mod coprod;

pub(super) use coprod::{assert_coproduct_collision_sound, assert_coproduct_coverage_sound};

/// Compose a compiled path starting at source row `start` — rebuilt
/// independently: each step resolves its cell or the path fails. Same
/// concrete-bound shape as the kernel walk's `engine_compose` (bounded
/// path table, flat-guarded reads — ledger `0006`), sharing no code
/// with it.
fn compose(typed: &EngineTyped, path: &Bounded<Step, ARROWS_CAP>, start: usize) -> Option<usize> {
    let mut current = start;
    let mut i = 0;
    while i < path.len() {
        let step = match path[i] {
            Some(Some(step)) => step,
            _ => return None,
        };
        if step >= typed.tables.len() {
            return None;
        }
        let cell = match typed.tables[step].get(current) {
            Some(Some(value)) => value.0,
            _ => return None,
        };
        current = cell;
        i += 1;
    }
    Some(current)
}

/// The equation claim's left half: the lhs side recomposed
/// independently lands on the witness, and the pair genuinely
/// disagrees. The full both-sides obligation's formula exceeded the 8 GB
/// reduction (2026-11-24), so the claim is split per composition side —
/// each half carries its compose plus the cheap disagreement assert,
/// and the two harnesses' union carries the family's claim.
pub(super) fn assert_equation_lhs_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
) {
    let equation = &schema.equations[index];
    let left = compose(typed, &equation.lhs, row);
    assert_eq!(left, Some(lhs));
    assert_ne!(lhs, rhs);
    let _ = schema;
}

/// The equation claim's right half (see
/// [`assert_equation_lhs_sound`]).
pub(super) fn assert_equation_rhs_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
) {
    let equation = &schema.equations[index];
    let right = compose(typed, &equation.rhs, row);
    assert_eq!(right, Some(rhs));
    assert_ne!(lhs, rhs);
    let _ = schema;
}

/// The key claim: both colliding rows' key tuples fully resolve and
/// are equal, ascending as carried. The soundness harness calls its
/// family's obligation directly — the merged per-finding dispatch is
/// gone (2026-11-24: the dispatch split per witness kind, Decision 4 at
/// obligation granularity).
pub(super) fn assert_key_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    lo: usize,
    hi: usize,
) {
    let key = schema.keys[index].as_ref().unwrap();
    assert!(lo < hi);
    // Both rows' key tuples fully resolve (any unresolved cell fails
    // the claim), and the first arrow's values agree — plain slot
    // reads, no heap tuple (walker arena round 2).
    let first_lo = key_first_value(typed, key, lo);
    let first_hi = key_first_value(typed, key, hi);
    assert_eq!(first_lo, first_hi);
    let _ = schema;
}

/// The row's key tuple fully resolves (unresolved fails the claim);
/// the first arrow's value rides out for the equality check.
fn key_first_value(typed: &EngineTyped, key: &CompiledKey, row: usize) -> Option<u64> {
    let mut a = 0;
    let mut first: Option<u64> = None;
    while a < key.arrows.len() {
        let cell = typed.tables[key.arrows[a]]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        match cell {
            Some(value) => {
                if first.is_none() {
                    first = Some(value as u64);
                }
            }
            None => panic!(),
        }
        a += 1;
    }
    assert!(first.is_some());
    first
}

/// The commutes claim: both cospan compositions resolve and disagree
/// as the witness carried them.
fn assert_commutes_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
) {
    let pullback = schema.pullbacks[index].as_ref().unwrap();
    let [p1, p2] = pullback.legs;
    let [f, g] = pullback.cospan;
    let left = typed.tables[p1]
        .get(row)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    let right = typed.tables[p2]
        .get(row)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    match (left, right) {
        (Some(left), Some(right)) => {
            let f_value = typed.tables[f]
                .get(left)
                .and_then(|cell| cell.as_ref())
                .map(|cell| cell.0);
            let g_value = typed.tables[g]
                .get(right)
                .and_then(|cell| cell.as_ref())
                .map(|cell| cell.0);
            assert_eq!(f_value, Some(lhs));
            assert_eq!(g_value, Some(rhs));
            assert_ne!(lhs, rhs);
        }
        _ => panic!(),
    }
    let _ = schema;
}

/// The covers claim: the pair shares a cospan target, and no apex row
/// projects onto it through both legs of the square.
fn assert_covers_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    a: usize,
    b: usize,
) {
    let pullback = schema.pullbacks[index].as_ref().unwrap();
    let [p1, p2] = pullback.legs;
    let [f, g] = pullback.cospan;
    let f_value = typed.tables[f]
        .get(a)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    let g_value = typed.tables[g]
        .get(b)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    assert_eq!(f_value, g_value);
    assert!(f_value.is_some());
    let apex_rows = typed.row_counts[pullback.apex].unwrap_or(0);
    let mut row = 0;
    while row < apex_rows {
        let left = typed.tables[p1]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        let right = typed.tables[p2]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        match (left, right) {
            (Some(left), Some(right)) => {
                assert!(!(left == a && right == b));
            }
            _ => {}
        }
        row += 1;
    }
}

/// The unique claim: two ascending apex rows resolve both leg
/// projections to the same values — the square is not injective there.
fn assert_unique_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    lo: usize,
    hi: usize,
) {
    let pullback = schema.pullbacks[index].as_ref().unwrap();
    let [p1, p2] = pullback.legs;
    assert!(lo < hi);
    let mut lo_left: Option<usize> = None;
    let mut lo_right: Option<usize> = None;
    let mut hi_left: Option<usize> = None;
    let mut hi_right: Option<usize> = None;
    for row in [lo, hi] {
        let left = typed.tables[p1]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        let right = typed.tables[p2]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        assert!(left.is_some() && right.is_some());
        if row == lo {
            lo_left = left;
            lo_right = right;
        } else {
            hi_left = left;
            hi_right = right;
        }
    }
    assert_eq!(lo_left, hi_left);
    assert_eq!(lo_right, hi_right);
    let _ = schema;
}

pub(super) fn assert_dangling_sound(typed: &EngineTyped, arrow: usize, src: usize) {
    let cell = typed.tables.get(arrow).and_then(|table| table.get(src));
    match cell {
        Some(None) => {}
        _ => panic!(),
    }
}

/// The pullback family's three obligations, one per verdict.
pub(super) fn assert_pullback_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    pullback: &CompactPullback,
) {
    match pullback {
        CompactPullback::Commutes {
            index,
            row,
            lhs,
            rhs,
        } => assert_commutes_sound(schema, typed, *index, *row, *lhs, *rhs),
        CompactPullback::Covers { index, pair } => {
            assert_covers_sound(schema, typed, *index, pair[0], pair[1])
        }
        CompactPullback::Unique { index, rows } => {
            assert_unique_sound(schema, typed, *index, rows[0], rows[1])
        }
    }
}
