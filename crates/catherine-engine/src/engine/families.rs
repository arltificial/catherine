//! The executed check families over interned tables — the parsed walk's
//! five families and the fiber helper, emitting compact int sort keys.

use super::{
    engine_compose, Bounded, CompactFinding, CompactViolation, EngineSchema, EngineTyped, SortKey,
    ROWS_CAP,
};
use crate::{findings::EpistemicLabel, model::RowId};

mod coprod;
mod keys;
mod pullback;

pub(crate) use coprod::coproduct_findings;
pub(crate) use keys::key_findings;
pub(crate) use pullback::pullback_findings;

pub(crate) use super::push_bounded;

/// Per-slot violation flags — the dangling family's exclusion set as
/// flat reads: one plain pass over the violations table (the old
/// per-cell membership scan was a symbolic-length inner loop
/// re-enumerated per cell, the measured walker symex wall; ledger
/// 0006), then `violated[nid][idx]` answers per cell.
pub(crate) fn violation_flags(typed: &EngineTyped) -> Vec<Bounded<bool, ROWS_CAP>> {
    let mut violated: Vec<Bounded<bool, ROWS_CAP>> = (0..typed.tables.len())
        .map(|_| Bounded::<bool, ROWS_CAP>::with_len(ROWS_CAP))
        .collect();
    let mut vi = 0;
    while vi < typed.typing_violations.len() {
        if let Some(&Some((seen, at, _))) = typed.typing_violations.get(vi) {
            if let Some(flag) = violated.get_mut(seen) {
                flag[at] = Some(true);
            }
        }
        vi += 1;
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
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
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
                push_bounded(
                    out,
                    (
                        (1, 0, 0, nid as u64, idx as u64, 0, 0, 0),
                        EpistemicLabel::Executed,
                        CompactFinding::Dangling {
                            arrow: nid,
                            src: idx,
                        },
                    ),
                );
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
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
) {
    for (index, equation) in schema.equations.iter().enumerate() {
        let Some(domain) = equation.domain.and_then(|d| typed.row_counts[d]) else {
            continue;
        };
        for row in 0..domain {
            let Some(lhs) = engine_compose(typed, &equation.lhs, RowId(row)) else {
                continue;
            };
            let Some(rhs) = engine_compose(typed, &equation.rhs, RowId(row)) else {
                continue;
            };
            if lhs != rhs {
                push_equation_finding(out, index, row, lhs.0, rhs.0, equation.status);
            }
        }
    }
}

/// One equation-disagreement finding for a composed row pair.
fn push_equation_finding(
    out: &mut Vec<(SortKey, EpistemicLabel, CompactFinding)>,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
    status: crate::model::ConstraintStatus,
) {
    push_bounded(
        out,
        (
            (0, 0, 0, index as u64, row as u64, lhs as u64, rhs as u64, 0),
            crate::findings::label_for(status),
            CompactFinding::Violation(CompactViolation::Equation {
                index,
                row,
                lhs,
                rhs,
            }),
        ),
    );
}
