//! The pullback half of the executed checks — a declared square's
//! universal property as three decidable verdicts (commutes, covers,
//! unique) over the interned tables. The walk's `BTreeMap` grouping
//! (projection buckets and cospan fibers) is an arena-backed sorted
//! table with plain slot scans here; equal tuples keep ascending row
//! order, so buckets, fibers, and findings are exactly the walk's.

use super::super::{Bounded, CompactFinding, CompactViolation, ARROWS_CAP, ROWS_CAP};
use super::super::{CompactPullback, CompiledPullback, EngineSchema, EngineTyped, Nid, SortKey};
use super::push_bounded;
use crate::findings::EpistemicLabel;

/// The apex rows bucketed by their resolved leg projection — ascending
/// projection tuples, ascending rows within each bucket, over the arena.
type ProjectionBuckets = Bounded<((u64, u64), Bounded<usize, ROWS_CAP>), ARROWS_CAP>;

/// One arrow's source rows grouped by resolved target value — the walk's
/// fiber groups, ascending targets, ascending rows within, over the
/// arena.
type Fibers = Bounded<(u64, Bounded<usize, ROWS_CAP>), ROWS_CAP>;

/// The pullback half of the executed checks — a declared square's
/// universal property as three decidable verdicts over the interned
/// tables. The walk's `BTreeMap` grouping (projection buckets and cospan
/// fibers) is an arena-backed sorted table with plain slot scans here;
/// equal tuples keep ascending row order, so buckets, fibers, and
/// findings are exactly the walk's. Findings emit straight into the
/// walk's report vector: the final sort's keys are distinct within the
/// family's domain, so the per-pullback emission order is invisible.
pub(crate) fn pullback_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
) {
    for (index, pullback) in schema.pullbacks.iter().enumerate() {
        let Some(pullback) = pullback else { continue };
        let Some(apex_rows) = typed.row_counts[pullback.apex] else {
            continue;
        };
        let label = crate::findings::label_for(pullback.status);
        let (projections, legs_partial) =
            pullback_projections(typed, out, pullback, index, apex_rows, label);
        emit_pullback_unique(out, &projections, index, label);
        if !legs_partial {
            emit_pullback_covers(schema, typed, out, &projections, pullback, index, label);
        }
    }
    let _ = schema;
}

/// Walk the apex rows: the commutes verdict per fully resolved row, and
/// the projection buckets the other verdicts need. Returns the buckets
/// (ascending projection tuples, ascending rows within) and whether any
/// leg cell was unresolved.
fn pullback_projections(
    typed: &EngineTyped,
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    pullback: &CompiledPullback,
    index: usize,
    apex_rows: usize,
    label: EpistemicLabel,
) -> (ProjectionBuckets, bool) {
    let [p1, p2] = pullback.legs;
    let [f, g] = pullback.cospan;
    let mut projections = ProjectionBuckets::new();
    let mut legs_partial = false;
    let mut row = 0;
    while row < apex_rows {
        let left = typed.tables[p1].get(row).copied().flatten();
        let right = typed.tables[p2].get(row).copied().flatten();
        let (Some(left), Some(right)) = (left, right) else {
            legs_partial = true;
            row += 1;
            continue;
        };
        let lhs = typed.tables[f].get(left.0).copied().flatten();
        let rhs = typed.tables[g].get(right.0).copied().flatten();
        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            if lhs != rhs {
                push_commutes_finding(out, index, row, lhs.0, rhs.0, label);
            }
        }
        let projection = (left.0 as u64, right.0 as u64);
        push_projection(&mut projections, projection, row);
        row += 1;
    }
    projections.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    (projections, legs_partial)
}

/// Add one apex row to its projection's bucket, creating the bucket —
/// the walk's bucketed-map insert over ids (order restored by the
/// caller's sort; equal projections keep ascending row order). Plain
/// slot scan — no iterator find chain, no heap bucket.
fn push_projection(projections: &mut ProjectionBuckets, projection: (u64, u64), row: usize) {
    let mut i = 0;
    while i < projections.len() {
        if let Some(Some((key, rows))) = projections.get_mut(i) {
            if *key == projection {
                rows.push(row);
                return;
            }
        }
        i += 1;
    }
    let mut bucket = Bounded::<usize, ROWS_CAP>::new();
    bucket.push(row);
    projections.push((projection, bucket));
}

/// One commutes finding for a cospan pair disagreeing on the apex row.
/// projection, ascending rows within an ascending-projection bucket walk.
fn push_commutes_finding(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
    label: EpistemicLabel,
) {
    push_bounded(
        out,
        (
            (0, 4, 0, index as u64, row as u64, lhs as u64, rhs as u64, 0),
            label,
            CompactFinding::Violation(CompactViolation::Pullback(CompactPullback::Commutes {
                index,
                row,
                lhs,
                rhs,
            })),
        ),
    );
}

/// Uniqueness: one finding per unordered pair of apex rows sharing a
/// projection, ascending rows within an ascending-projection bucket walk.
fn emit_pullback_unique(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    projections: &ProjectionBuckets,
    index: usize,
    label: EpistemicLabel,
) {
    let mut b = 0;
    while b < projections.len() {
        if let Some(Some((_, rows))) = projections.get(b) {
            emit_bucket_pairs(out, rows, index, label);
        }
        b += 1;
    }
}

/// One finding per unordered pair within one projection bucket,
/// ascending rows.
fn emit_bucket_pairs(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    rows: &Bounded<usize, ROWS_CAP>,
    index: usize,
    label: EpistemicLabel,
) {
    let mut position = 0;
    while position < rows.len() {
        let Some(&lo) = rows[position].as_ref() else {
            break;
        };
        let mut hi_index = position + 1;
        while hi_index < rows.len() {
            let Some(&hi) = rows[hi_index].as_ref() else {
                break;
            };
            push_bounded(
                out,
                (
                    (0, 4, 2, index as u64, lo as u64, hi as u64, 0, 0),
                    label,
                    CompactFinding::Violation(CompactViolation::Pullback(
                        CompactPullback::Unique {
                            index,
                            rows: [lo, hi],
                        },
                    )),
                ),
            );
            hi_index += 1;
        }
        position += 1;
    }
}

/// Coverage: every cospan fiber pair with no apex row projecting onto it
/// is one missing-cover finding, ascending targets and rows.
fn emit_pullback_covers(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    projections: &ProjectionBuckets,
    pullback: &CompiledPullback,
    index: usize,
    label: EpistemicLabel,
) {
    let [f, g] = pullback.cospan;
    let a_fiber = fiber(typed, f, typed.row_counts[schema.arrow_src[f]].unwrap_or(0));
    let b_fiber = fiber(typed, g, typed.row_counts[schema.arrow_src[g]].unwrap_or(0));
    let mut a_slot = 0;
    while a_slot < a_fiber.len() {
        let Some((target, a_rows)) = a_fiber[a_slot].as_ref() else {
            break;
        };
        let Some(b_rows) = matching_fiber(&b_fiber, target) else {
            a_slot += 1;
            continue;
        };
        emit_fiber_pairs(out, projections, index, label, a_rows, b_rows);
        a_slot += 1;
    }
}

/// The walk's map lookup over the cospan fibers: the bucket holding one
/// target, as a plain slot scan.
fn matching_fiber<'a>(b_fiber: &'a Fibers, target: &u64) -> Option<&'a Bounded<usize, ROWS_CAP>> {
    let mut b_slot = 0;
    while b_slot < b_fiber.len() {
        if let Some(Some((b_target, b_rows))) = b_fiber.get(b_slot) {
            if b_target == target {
                return Some(b_rows);
            }
        }
        b_slot += 1;
    }
    None
}

/// One missing-cover finding per fiber pair with no projecting apex row,
/// ascending targets and rows.
fn emit_fiber_pairs(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    projections: &ProjectionBuckets,
    index: usize,
    label: EpistemicLabel,
    a_rows: &Bounded<usize, ROWS_CAP>,
    b_rows: &Bounded<usize, ROWS_CAP>,
) {
    let mut ai = 0;
    while ai < a_rows.len() {
        let Some(&a) = a_rows[ai].as_ref() else {
            break;
        };
        let mut bi = 0;
        while bi < b_rows.len() {
            let Some(&b) = b_rows[bi].as_ref() else {
                break;
            };
            emit_cover_finding(out, projections, index, label, a, b);
            bi += 1;
        }
        ai += 1;
    }
}

/// One missing-cover finding when no apex row projects onto the cospan
/// pair through both legs.
fn emit_cover_finding(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    projections: &ProjectionBuckets,
    index: usize,
    label: EpistemicLabel,
    a: usize,
    b: usize,
) {
    let pair = (a as u64, b as u64);
    let mut covered = false;
    let mut i = 0;
    while i < projections.len() {
        if let Some(Some((key, _))) = projections.get(i) {
            if *key == pair {
                covered = true;
                break;
            }
        }
        i += 1;
    }
    if !covered {
        push_bounded(
            out,
            (
                (0, 4, 1, index as u64, a as u64, b as u64, 0, 0),
                label,
                CompactFinding::Violation(CompactViolation::Pullback(CompactPullback::Covers {
                    index,
                    pair: [a, b],
                })),
            ),
        );
    }
}

/// One arrow's source rows grouped by resolved target value, ascending
/// rows within an ascending-target list — the walk's `fiber` helper over
/// ids. Rows with an unresolved cell have no target and are excluded.
/// Arena-backed: the groups and each bucket are fixed-cap tables, no
/// heap bucket per target.
fn fiber(typed: &EngineTyped, arrow: Nid, n_src: usize) -> Fibers {
    let mut fibers = Fibers::new();
    let mut row = 0;
    while row < n_src {
        let Some(target) = typed.tables[arrow].get(row).copied().flatten() else {
            row += 1;
            continue;
        };
        let target = target.0 as u64;
        if !add_to_fiber(&mut fibers, target, row) {
            let mut bucket = Bounded::<usize, ROWS_CAP>::new();
            bucket.push(row);
            fibers.push((target, bucket));
        }
        row += 1;
    }
    fibers.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    fibers
}

/// Add one row to its target's fiber bucket; false when no bucket holds
/// the target yet — the walk's bucketed-map insert, plain slot scan.
fn add_to_fiber(fibers: &mut Fibers, target: u64, row: usize) -> bool {
    let mut i = 0;
    while i < fibers.len() {
        if let Some(Some((t, rows))) = fibers.get_mut(i) {
            if *t == target {
                rows.push(row);
                return true;
            }
        }
        i += 1;
    }
    false
}
