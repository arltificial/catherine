//! The executed check families over interned tables — the parsed walk's
//! five families and the fiber helper, emitting compact int sort keys.

use super::{
    engine_compose, Bounded, CompactFinding, CompactViolation, CompiledEquation, EngineSchema,
    EngineTyped, SortKey, REPORT_CAP, ROWS_CAP,
};
use crate::{findings::EpistemicLabel, model::RowId};

mod coprod;
mod keys;
mod pullback;

pub(crate) use coprod::coproduct_findings;
pub(crate) use keys::key_findings;
pub(crate) use pullback::pullback_findings;

pub(crate) use super::push_bounded;

/// One partition entry's bounded report table: the walker fills it
/// (ordering included — see the per-family entries) through `walk`.
fn collect_partition(
    walk: impl FnOnce(&mut Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP>),
) -> Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP> {
    let mut out: Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP> = Bounded::new();
    walk(&mut out);
    out
}

/// The families' emission sink: the walkers fill either storage — the
/// heap `Vec` (parsed flavor, `engine_families`' single sorted output)
/// or the bounded table (the lane's partitioned entries, ledger `0007`:
/// the slice sort and symbolic slice iteration were the solve-phase
/// wall's last heap machinery). Monomorphized per call site, so the
/// lane's formula inlines slot writes and the parsed flavor keeps the
/// asserted-append Vec; both arms are capacity-asserted appends, never
/// allocation branches.
pub(crate) trait FindingSink {
    fn emit(&mut self, finding: (SortKey, EpistemicLabel, CompactFinding));
}

impl FindingSink for Vec<(SortKey, EpistemicLabel, CompactFinding)> {
    fn emit(&mut self, finding: (SortKey, EpistemicLabel, CompactFinding)) {
        push_bounded(self, finding);
    }
}

impl<const CAP: usize> FindingSink for Bounded<(SortKey, EpistemicLabel, CompactFinding), CAP> {
    fn emit(&mut self, finding: (SortKey, EpistemicLabel, CompactFinding)) {
        Bounded::push(self, finding);
    }
}

/// Per-slot violation flags — the dangling family's exclusion set as
/// flat reads: one plain pass over the violations table (the old
/// per-cell membership scan was a symbolic-length inner loop
/// re-enumerated per cell, the measured walker symex wall; ledger
/// 0006), then `violated[nid][idx]` answers per cell.
pub(crate) fn violation_flags(typed: &EngineTyped) -> Vec<Bounded<bool, ROWS_CAP>> {
    let mut violated: Vec<Bounded<bool, ROWS_CAP>> = (0..typed.tables.len())
        .map(|_| Bounded::<bool, ROWS_CAP>::with_len(ROWS_CAP))
        .collect();
    for vi in 0..typed.typing_violations.len() {
        if let Some(&Some((seen, at, _))) = typed.typing_violations.get(vi) {
            if let Some(flag) = violated.get_mut(seen) {
                flag[at] = Some(true);
            }
        }
    }
    violated
}

/// One dangling finding per unresolved cell not accounted for by a
/// typing violation — the walk's dangling family over ids. The caller
/// passes the per-slot violation flags (`violated[nid][idx]` marks a
/// violated cell) derived in one pass over the violations table: a
/// table slot is `None` for a parsed-`None` cell (dangling, emitted),
/// a violated cell (flagged, suppressed), or an unclassified cell
/// (no cell vector, or a vector shorter than the row space — emitted),
/// exactly the old scan's partition without re-scanning the
/// violations table per cell (that symbolic-length inner loop was the
/// walker symex wall; see ledger 0006).
pub(crate) fn dangling_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    violated: &[Bounded<bool, ROWS_CAP>],
    out: &mut impl FindingSink,
) {
    // Emission order is unchanged: arrow id order, then cell order.
    for (nid, table) in typed.tables.iter().enumerate() {
        let flags = violated.get(nid);
        for (idx, cell) in table.iter().enumerate() {
            let flagged = flags
                .and_then(|flag| flag.get(idx))
                .map(|slot| slot == &Some(true))
                .unwrap_or(false);
            if cell.is_none() && !flagged {
                out.emit((
                    (1, 0, 0, nid as u64, idx as u64, 0, 0, 0),
                    EpistemicLabel::Executed,
                    CompactFinding::Dangling {
                        arrow: nid,
                        src: idx,
                    },
                ));
            }
        }
    }
    let _ = schema;
}

/// Every declared equation on every row of its domain object; a path
/// through an unresolved cell cannot compose, and the affected row is
/// already recorded by its dangling finding.
pub(crate) fn equation_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut impl FindingSink,
) {
    for (index, equation) in schema.equations.iter().enumerate() {
        let Some(domain) = equation.domain.and_then(|d| typed.row_counts[d]) else {
            continue;
        };
        for row in 0..domain {
            let Some((lhs, rhs)) = composed_sides(typed, equation, row) else {
                continue;
            };
            if lhs != rhs {
                push_equation_finding(
                    out,
                    index,
                    row,
                    EquationWitness {
                        lhs: lhs.0,
                        rhs: rhs.0,
                        status: equation.status,
                    },
                );
            }
        }
    }
}

/// Both compositions of one equation at one row, `None` when either
/// side dangles.
fn composed_sides(
    typed: &EngineTyped,
    equation: &CompiledEquation,
    row: usize,
) -> Option<(RowId, RowId)> {
    let lhs = engine_compose(typed, &equation.lhs, RowId(row))?;
    let rhs = engine_compose(typed, &equation.rhs, RowId(row))?;
    Some((lhs, rhs))
}

/// One composed row pair plus its constraint's status — the witness of
/// an equation-disagreement finding.
struct EquationWitness {
    lhs: usize,
    rhs: usize,
    status: crate::model::ConstraintStatus,
}

/// One equation-disagreement finding for a composed row pair.
fn push_equation_finding(
    out: &mut impl FindingSink,
    index: usize,
    row: usize,
    witness: EquationWitness,
) {
    let EquationWitness { lhs, rhs, status } = witness;
    out.emit((
        (0, 0, 0, index as u64, row as u64, lhs as u64, rhs as u64, 0),
        crate::findings::label_for(status),
        CompactFinding::Violation(CompactViolation::Equation {
            index,
            row,
            lhs,
            rhs,
        }),
    ));
}

/// The dangling family alone, compact and sorted — the check side's
/// per-family partition entry over the equation shape (the lane's
/// Decision 4 at walker granularity: one harness formula carries one
/// family's walk; ledger `0007`'s recorded trigger). Lane-only surface:
/// the parsed flavor serves [`super::engine_families`]. The output is a
/// bounded table, not a heap `Vec`: the slice sort and the symbolic
/// slice iteration were the solve-phase wall's last heap machinery
/// (measured 2026-11-23: the partitioned formulas died in the solver
/// down to 3 156 post-slicing VCCs while the sort-free build proofs
/// solved at ~1.2 GB) — slot writes and [`Bounded::sort_pushed_by`]
/// replace them, the identical stable order.
pub fn engine_dangling_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
) -> Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP> {
    // Worst case: every arrow's every source row dangles — a concrete
    // bound (the shapes hold 7 arrows × 2 rows = 14), so the capacity
    // asserts discharge and the walk never reallocates.
    let violated = violation_flags(typed);
    collect_partition(|out| {
        dangling_findings(schema, typed, &violated, out);
        out.sort_pushed_by(|a, b| a.0.cmp(&b.0));
    })
}

/// The equation family's emissions in walk order — the check side's
/// per-family partition entry (see [`engine_dangling_findings`]; the
/// bounded-table rationale is recorded there). Walk order is already
/// the family's key order (one finding per (equation, row), ascending),
/// so this entry does not sort: the merged cross-family sort lives in
/// [`super::engine_families`], and the equation partition's sort
/// obligation is the check harness's own `sort_pushed_by` call — the
/// soundness harness's claims are order-free, and dropping the sort
/// from its formula is what fits the independent re-derivation inside
/// the 8 GB reduction (2026-11-24).
pub fn engine_equation_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
) -> Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP> {
    // Worst case: every equation disagrees on every domain row —
    // concrete under the declared shapes (≤ 6 emissions), well under
    // the cap; the capacity asserts discharge.
    collect_partition(|out| equation_findings(schema, typed, out))
}

/// The key family's emissions in walk order — the check side's
/// per-family partition entry (see [`engine_dangling_findings`] and
/// [`engine_equation_findings`] for the bounded-table and walk-order
/// rationale). One finding per unordered equal-tuple pair, ascending
/// tuples and rows, so walk order is already the family's key order and
/// this entry does not sort: the sort obligation over the partition's
/// symbolic keys is the check harness's own `sort_pushed_by` call, and
/// the soundness harness consumes the emissions without it.
pub fn engine_key_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
) -> Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP> {
    // Worst case: every resolved row pair collides — concrete under the
    // declared shape (≤ 1 emission), well under the cap.
    collect_partition(|out| key_findings(schema, typed, out))
}

/// The coproduct family's emissions in walk order — the check side's
/// per-family partition entry (see [`engine_key_findings`]). One finding
/// per row whose coverage verdict is decidable and failed, ascending
/// rows, so walk order is already the family's key order.
pub fn engine_coproduct_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
) -> Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP> {
    // Worst case: every target row uncovered or colliding — concrete
    // under the declared shape (≤ 2 emissions), well under the cap.
    collect_partition(|out| coproduct_findings(schema, typed, out))
}

/// The pullback family's emissions in walk order — the check side's
/// per-family partition entry (see [`engine_key_findings`]). Walk order
/// is commutes rows, then unique bucket pairs, then covers fiber pairs
/// — NOT the family's key order (the verdict discriminators sort
/// commutes < covers < unique), so the check harness's `sort_pushed_by`
/// is load-bearing here, not just uniform: it is what keeps the sort's
/// panic-freedom over the partition's symbolic keys a proven claim.
pub fn engine_pullback_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
) -> Bounded<(SortKey, EpistemicLabel, CompactFinding), REPORT_CAP> {
    // Worst case: every resolved apex row disagrees, every fiber pair is
    // uncovered, and the apex pair shares a projection — concrete under
    // the declared shape (≤ 7 emissions), well under the cap.
    collect_partition(|out| pullback_findings(schema, typed, out))
}
