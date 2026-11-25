//! The pullback half of the executed checks — a declared square's
//! universal property as three decidable verdicts (commutes, covers,
//! unique) over the interned tables. The walk's `BTreeMap` grouping
//! (projection buckets and cospan fibers) is an arena-backed sorted
//! table with plain slot scans here; equal tuples keep ascending row
//! order, so buckets, fibers, and findings are exactly the walk's.

use super::super::{Bounded, CompactFinding, CompactViolation, ARROWS_CAP, ROWS_CAP};
use super::super::{CompactPullback, CompiledPullback, EngineSchema, EngineTyped, Nid};
use super::FindingSink;
use crate::findings::EpistemicLabel;

/// Per-pullback emission context: the shared report sink plus the
/// declared pullback's index and its constraint's label.
struct PullbackEmit<'a, S: FindingSink> {
    out: &'a mut S,
    index: usize,
    label: EpistemicLabel,
}

impl<S: FindingSink> PullbackEmit<'_, S> {
    /// One commutes finding for a cospan pair disagreeing on the apex
    /// row. projection, ascending rows within an ascending-projection
    /// bucket walk.
    fn commutes(&mut self, row: usize, lhs: usize, rhs: usize) {
        self.out.emit((
            (
                0,
                4,
                0,
                self.index as u64,
                row as u64,
                lhs as u64,
                rhs as u64,
                0,
            ),
            self.label,
            CompactFinding::Violation(CompactViolation::Pullback(CompactPullback::Commutes {
                index: self.index,
                row,
                lhs,
                rhs,
            })),
        ));
    }

    /// One missing-cover finding when no apex row projects onto the
    /// cospan pair through both legs.
    fn cover_missing(&mut self, projections: &ProjectionBuckets, a: usize, b: usize) {
        if pair_covered(projections, (a as u64, b as u64)) {
            return;
        }
        self.out.emit((
            (0, 4, 1, self.index as u64, a as u64, b as u64, 0, 0),
            self.label,
            CompactFinding::Violation(CompactViolation::Pullback(CompactPullback::Covers {
                index: self.index,
                pair: [a, b],
            })),
        ));
    }

    /// One unique finding per ascending row pair sharing a projection.
    fn unique_pair(&mut self, lo: usize, hi: usize) {
        self.out.emit((
            (0, 4, 2, self.index as u64, lo as u64, hi as u64, 0, 0),
            self.label,
            CompactFinding::Violation(CompactViolation::Pullback(CompactPullback::Unique {
                index: self.index,
                rows: [lo, hi],
            })),
        ));
    }
}

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
    out: &mut impl FindingSink,
) {
    for (index, pullback) in schema.pullbacks.iter().enumerate() {
        let Some(pullback) = pullback else { continue };
        let Some(apex_rows) = typed.row_counts[pullback.apex] else {
            continue;
        };
        let mut em = PullbackEmit {
            out: &mut *out,
            index,
            label: crate::findings::label_for(pullback.status),
        };
        let (projections, legs_partial) = pullback_projections(typed, &mut em, pullback, apex_rows);
        emit_pullback_unique(&mut em, &projections);
        if !legs_partial {
            let fibers = cospan_fibers(schema, typed, pullback);
            emit_pullback_covers(&mut em, &projections, &fibers);
        }
    }
}

/// One arrow's resolved value at a row, if taken.
fn resolve_at(typed: &EngineTyped, arrow: Nid, row: usize) -> Option<usize> {
    typed.tables[arrow].get(row).copied().flatten().map(|v| v.0)
}

/// Walk the apex rows: the commutes verdict per fully resolved row, and
/// the projection buckets the other verdicts need. Returns the buckets
/// (ascending projection tuples, ascending rows within) and whether any
/// leg cell was unresolved.
fn pullback_projections<S: FindingSink>(
    typed: &EngineTyped,
    em: &mut PullbackEmit<S>,
    pullback: &CompiledPullback,
    apex_rows: usize,
) -> (ProjectionBuckets, bool) {
    let [p1, p2] = pullback.legs;
    let [f, g] = pullback.cospan;
    let mut projections = ProjectionBuckets::new();
    let mut legs_partial = false;
    for row in 0..apex_rows {
        let (Some(left), Some(right)) = (resolve_at(typed, p1, row), resolve_at(typed, p2, row))
        else {
            legs_partial = true;
            continue;
        };
        let lhs = resolve_at(typed, f, left);
        let rhs = resolve_at(typed, g, right);
        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            if lhs != rhs {
                em.commutes(row, lhs, rhs);
            }
        }
        push_projection(&mut projections, (left as u64, right as u64), row);
    }
    projections.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    (projections, legs_partial)
}

/// Add one apex row to its projection's bucket, creating the bucket —
/// the walk's bucketed-map insert over ids (order restored by the
/// caller's sort; equal projections keep ascending row order). Plain
/// slot scan — no iterator find chain, no heap bucket.
fn push_projection(projections: &mut ProjectionBuckets, projection: (u64, u64), row: usize) {
    for i in 0..projections.len() {
        if let Some(Some((key, rows))) = projections.get_mut(i) {
            if *key == projection {
                rows.push(row);
                return;
            }
        }
    }
    let mut bucket = Bounded::<usize, ROWS_CAP>::new();
    bucket.push(row);
    projections.push((projection, bucket));
}

/// Uniqueness: one finding per unordered pair of apex rows sharing a
/// projection, ascending rows within an ascending-projection bucket walk.
fn emit_pullback_unique<S: FindingSink>(em: &mut PullbackEmit<S>, projections: &ProjectionBuckets) {
    for (_, rows) in projections.iter().flatten() {
        emit_bucket_pairs(em, rows);
    }
}

/// One finding per unordered pair within one projection bucket,
/// ascending rows.
fn emit_bucket_pairs<S: FindingSink>(em: &mut PullbackEmit<S>, rows: &Bounded<usize, ROWS_CAP>) {
    for position in 0..rows.len() {
        let Some(&lo) = rows[position].as_ref() else {
            break;
        };
        for hi_index in position + 1..rows.len() {
            let Some(&hi) = rows[hi_index].as_ref() else {
                break;
            };
            em.unique_pair(lo, hi);
        }
    }
}

/// The cospan's two fibers over their source row spaces — the reads the
/// covers verdict needs, split from its emission.
fn cospan_fibers(
    schema: &EngineSchema,
    typed: &EngineTyped,
    pullback: &CompiledPullback,
) -> (Fibers, Fibers) {
    let [f, g] = pullback.cospan;
    let a_fiber = fiber(
        typed,
        f,
        typed.row_counts[schema.arrows.src[f]].unwrap_or(0),
    );
    let b_fiber = fiber(
        typed,
        g,
        typed.row_counts[schema.arrows.src[g]].unwrap_or(0),
    );
    (a_fiber, b_fiber)
}

/// Coverage: every cospan fiber pair with no apex row projecting onto it
/// is one missing-cover finding, ascending targets and rows.
fn emit_pullback_covers<S: FindingSink>(
    em: &mut PullbackEmit<S>,
    projections: &ProjectionBuckets,
    fibers: &(Fibers, Fibers),
) {
    let (a_fiber, b_fiber) = fibers;
    for entry in a_fiber.iter() {
        let Some((target, a_rows)) = entry.as_ref() else {
            break;
        };
        let Some(b_rows) = matching_fiber(b_fiber, target) else {
            continue;
        };
        emit_fiber_pairs(em, projections, a_rows, b_rows);
    }
}

/// The walk's map lookup over the cospan fibers: the bucket holding one
/// target, as a plain slot scan.
fn matching_fiber<'a>(b_fiber: &'a Fibers, target: &u64) -> Option<&'a Bounded<usize, ROWS_CAP>> {
    for (b_target, b_rows) in b_fiber.iter().flatten() {
        if b_target == target {
            return Some(b_rows);
        }
    }
    None
}

/// One missing-cover finding per fiber pair with no projecting apex row,
/// ascending targets and rows.
fn emit_fiber_pairs<S: FindingSink>(
    em: &mut PullbackEmit<S>,
    projections: &ProjectionBuckets,
    a_rows: &Bounded<usize, ROWS_CAP>,
    b_rows: &Bounded<usize, ROWS_CAP>,
) {
    for ai in 0..a_rows.len() {
        let Some(&a) = a_rows[ai].as_ref() else {
            break;
        };
        for bi in 0..b_rows.len() {
            let Some(&b) = b_rows[bi].as_ref() else {
                break;
            };
            em.cover_missing(projections, a, b);
        }
    }
}

/// Whether any projection bucket covers the cospan pair — the walk's
/// membership test, as a plain slot scan.
fn pair_covered(projections: &ProjectionBuckets, pair: (u64, u64)) -> bool {
    for (key, _) in projections.iter().flatten() {
        if *key == pair {
            return true;
        }
    }
    false
}

/// One arrow's source rows grouped by resolved target value, ascending
/// rows within an ascending-target list — the walk's `fiber` helper over
/// ids. Rows with an unresolved cell have no target and are excluded.
/// Arena-backed: the groups and each bucket are fixed-cap tables, no
/// heap bucket per target.
fn fiber(typed: &EngineTyped, arrow: Nid, n_src: usize) -> Fibers {
    let mut fibers = Fibers::new();
    for row in 0..n_src {
        let Some(target) = typed.tables[arrow].get(row).copied().flatten() else {
            continue;
        };
        let target = target.0 as u64;
        if !add_to_fiber(&mut fibers, target, row) {
            let mut bucket = Bounded::<usize, ROWS_CAP>::new();
            bucket.push(row);
            fibers.push((target, bucket));
        }
    }
    fibers.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    fibers
}

/// Add one row to its target's fiber bucket; false when no bucket holds
/// the target yet — the walk's bucketed-map insert, plain slot scan.
fn add_to_fiber(fibers: &mut Fibers, target: u64, row: usize) -> bool {
    for i in 0..fibers.len() {
        if let Some(Some((t, rows))) = fibers.get_mut(i) {
            if *t == target {
                rows.push(row);
                return true;
            }
        }
    }
    false
}
