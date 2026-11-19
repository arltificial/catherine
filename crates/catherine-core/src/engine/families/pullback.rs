//! The pullback half of the executed checks — a declared square's
//! universal property as three decidable verdicts (commutes, covers,
//! unique) over the interned tables. The walk's `BTreeMap` grouping
//! (projection buckets and cospan fibers) is a sorted `Vec` with
//! binary-search membership here; equal tuples keep ascending row order,
//! so buckets, fibers, and findings are exactly the walk's.

use super::super::{CompactPullback, CompiledPullback, EngineSchema, EngineTyped, Nid, SortKey};
use super::{push_bounded, CompactFinding, CompactViolation};
use crate::findings::EpistemicLabel;

/// The apex rows bucketed by their resolved leg projection — ascending
/// projection tuples, ascending rows within each bucket.
type ProjectionBuckets = Vec<((u64, u64), Vec<usize>)>;

/// The pullback half of the executed checks — a declared square's
/// universal property as three decidable verdicts over the interned
/// tables. The walk's `BTreeMap` grouping (projection buckets and cospan
/// fibers) is a sorted `Vec` with binary-search membership here; equal
/// tuples keep ascending row order, so buckets, fibers, and findings are
/// exactly the walk's.
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
        let label = crate::checks::label_for(pullback.status);
        let [f, g] = pullback.cospan;
        let f_src = typed.row_counts[schema.arrow_src[f]].unwrap_or(0);
        let g_src = typed.row_counts[schema.arrow_src[g]].unwrap_or(0);
        let mut findings: Vec<(SortKey, EpistemicLabel, CompactFinding)> =
            Vec::with_capacity(apex_rows + apex_rows * apex_rows + f_src * g_src + 1);
        let (projections, legs_partial) =
            pullback_projections(typed, &mut findings, pullback, index, apex_rows, label);
        emit_pullback_unique(&mut findings, &projections, index, label);
        if !legs_partial {
            emit_pullback_covers(
                schema,
                typed,
                &mut findings,
                &projections,
                pullback,
                index,
                label,
            );
        }
        out.extend(findings);
    }
    let _ = schema;
}

/// Walk the apex rows: the commutes verdict per fully resolved row, and
/// the projection buckets the other verdicts need. Returns the buckets
/// (ascending projection tuples, ascending rows within) and whether any
/// leg cell was unresolved.
fn pullback_projections(
    typed: &EngineTyped,
    findings: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    pullback: &CompiledPullback,
    index: usize,
    apex_rows: usize,
    label: EpistemicLabel,
) -> (ProjectionBuckets, bool) {
    let [p1, p2] = pullback.legs;
    let [f, g] = pullback.cospan;
    let mut projections: ProjectionBuckets = Vec::with_capacity(apex_rows);
    let mut legs_partial = false;
    for row in 0..apex_rows {
        let left = typed.tables[p1].get(row).copied().flatten();
        let right = typed.tables[p2].get(row).copied().flatten();
        let (Some(left), Some(right)) = (left, right) else {
            legs_partial = true;
            continue;
        };
        let lhs = typed.tables[f].get(left.0).copied().flatten();
        let rhs = typed.tables[g].get(right.0).copied().flatten();
        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            if lhs != rhs {
                push_commutes_finding(findings, index, row, lhs.0, rhs.0, label);
            }
        }
        let projection = (left.0 as u64, right.0 as u64);
        match projections.binary_search_by(|(key, _)| key.cmp(&projection)) {
            Ok(slot) => push_bounded(&mut projections[slot].1, row),
            Err(slot) => {
                let mut bucket = Vec::with_capacity(apex_rows);
                bucket.push(row);
                projections.insert(slot, (projection, bucket));
            }
        }
    }
    (projections, legs_partial)
}

/// One commutes finding for a cospan pair disagreeing on the apex row.
/// projection, ascending rows within an ascending-projection bucket walk.
fn push_commutes_finding(
    findings: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
    label: EpistemicLabel,
) {
    push_bounded(
        findings,
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
    findings: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    projections: &[((u64, u64), Vec<usize>)],
    index: usize,
    label: EpistemicLabel,
) {
    for (_, rows) in projections {
        for position in 0..rows.len() {
            for &hi in &rows[position + 1..] {
                let lo = rows[position];
                push_bounded(
                    findings,
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
            }
        }
    }
}

/// Coverage: every cospan fiber pair with no apex row projecting onto it
/// is one missing-cover finding, ascending targets and rows.
fn emit_pullback_covers(
    schema: &EngineSchema,
    typed: &EngineTyped,
    findings: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    projections: &ProjectionBuckets,
    pullback: &CompiledPullback,
    index: usize,
    label: EpistemicLabel,
) {
    let [f, g] = pullback.cospan;
    let a_fiber = fiber(typed, f, typed.row_counts[schema.arrow_src[f]].unwrap_or(0));
    let b_fiber = fiber(typed, g, typed.row_counts[schema.arrow_src[g]].unwrap_or(0));
    for (target, a_rows) in &a_fiber {
        let Some(b_rows) = b_fiber
            .iter()
            .find(|(b_target, _)| b_target == target)
            .map(|(_, rows)| rows)
        else {
            continue;
        };
        for &a in a_rows {
            for &b in b_rows {
                emit_cover_finding(findings, projections, index, label, a, b);
            }
        }
    }
}

/// One missing-cover finding when no apex row projects onto the cospan
/// pair through both legs.
fn emit_cover_finding(
    findings: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    projections: &ProjectionBuckets,
    index: usize,
    label: EpistemicLabel,
    a: usize,
    b: usize,
) {
    let pair = (a as u64, b as u64);
    if projections
        .binary_search_by(|(key, _)| key.cmp(&pair))
        .is_err()
    {
        push_bounded(
            findings,
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
fn fiber(typed: &EngineTyped, arrow: Nid, n_src: usize) -> Vec<(u64, Vec<usize>)> {
    let mut fibers: Vec<(u64, Vec<usize>)> = Vec::with_capacity(n_src);
    for row in 0..n_src {
        let Some(target) = typed.tables[arrow].get(row).copied().flatten() else {
            continue;
        };
        let target = target.0 as u64;
        let mut hit = false;
        for entry in fibers.iter_mut() {
            if entry.0 == target {
                push_bounded(&mut entry.1, row);
                hit = true;
                break;
            }
        }
        if hit {
            continue;
        }
        let mut bucket = Vec::with_capacity(n_src);
        bucket.push(row);
        push_bounded(&mut fibers, (target, bucket));
    }
    fibers.sort_unstable_by_key(|entry| entry.0);
    fibers
}
