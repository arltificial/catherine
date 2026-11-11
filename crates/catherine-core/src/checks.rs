//! Purpose: executed path-equation checks — the E3 bet. `check` evaluates
//! every declared equation on every applicable row and emits finding-typed
//! outcomes with witnesses (openspec change `add-executed-checks`).
//!
//! Responsibilities: [`check`] walks the typed instance tables — one
//! dangling outcome per unresolved cell, then one outcome per equation per
//! applicable row whose composed paths disagree — and never emits an
//! outcome without its witnesses. A dangling value short-circuits path
//! composition at that arrow: the affected row is recorded (by the
//! dangling outcome naming arrow + source row), never reported as a
//! violation, and the run continues. Equation `status` decides the
//! epistemic weight of a failure: `accepted` →
//! [`CheckOutcome::Violation`], `proposed` →
//! [`CheckOutcome::Hypothesis`] — distinguishable, never merged into one
//! severity. Totality: an empty schema or an empty instance yields zero
//! outcomes — silence after evaluation is real success, never an
//! unevaluated declaration.
//!
//! Rationale: new code (E10) — specodelic's acset has no path-equation
//! execution; equations were already new in P0. Outcome order is
//! deterministic: dangling outcomes first (arrow-name order, then source
//! row order), then per-equation outcomes in canonical equation order ×
//! domain row order, so goldens pin structural expectations exactly.

use crate::checks::CheckOutcome::{Dangling, Hypothesis, Violation};
use crate::instance::TypedInstance;
use crate::model::{ConstraintStatus, Ident, RowId, Schema};
use std::collections::BTreeSet;

/// The outcome kinds the P1 checks surface. Minimal and open: P2's
/// findings taxonomy (`catherine-1x7`) extends the enum with new kinds —
/// witnesses stay mandatory — never by merging kinds or reusing one
/// variant for two epistemic weights. A typing violation is a *build
/// report* ([`TypedInstance::typing_violations`]), not a check outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    /// An `accepted` equation failed on a row: enforcement.
    Violation {
        /// The equation's index in `Schema::equations` (canonical order).
        equation: usize,
        /// The row of the equation's domain object that failed.
        row: RowId,
        /// The composed `lhs` path value.
        lhs: RowId,
        /// The composed `rhs` path value.
        rhs: RowId,
    },
    /// A `proposed` equation failed on a row: a domain question under
    /// review, countable separately from violations — never merged into
    /// one severity.
    Hypothesis {
        /// The equation's index in `Schema::equations` (canonical order).
        equation: usize,
        /// The row of the equation's domain object that failed.
        row: RowId,
        /// The composed `lhs` path value.
        lhs: RowId,
        /// The composed `rhs` path value.
        rhs: RowId,
    },
    /// An arrow cell holds no row (dangling reference value) — witness:
    /// the arrow and the source row. Paths traversing this cell are
    /// not-evaluable and emit nothing for the affected row.
    Dangling {
        /// The arrow whose cell is unresolved.
        arrow: Ident,
        /// The source row the dangling value sits on.
        src: RowId,
    },
}

/// Execute every declared equation on every applicable row of the typed
/// instance. See the module docs for the outcome contract.
///
/// Every lookup inside the walk is total (`get`-based): for parsed
/// presentations each declared equation's lead arrow, domain rows, and
/// tables all exist and the walk is exactly "every equation on every
/// applicable row". The `continue` branches only guard against
/// programmatically assembled presentations outside that shape (an
/// equation naming an undeclared lead arrow, a domain object with no row
/// space, a path composing through a cell the presentation does not
/// carry) — they skip an inapplicable equation, never a row of an
/// applicable one.
pub fn check(schema: &Schema, instance: &TypedInstance) -> Vec<CheckOutcome> {
    let mut outcomes = Vec::new();
    // Dangling values first: one outcome per unresolved cell that is not
    // accounted for by a typing violation (a violated value never became
    // a table entry — its `None` cell is a build report, not a dangling
    // reference).
    let violated: BTreeSet<(Ident, RowId)> = instance
        .typing_violations()
        .iter()
        .map(|v| (v.arrow.clone(), v.src))
        .collect();
    for (arrow, cells) in instance.tables() {
        for (idx, cell) in cells.iter().enumerate() {
            if cell.is_none() && !violated.contains(&(arrow.clone(), RowId(idx))) {
                outcomes.push(Dangling {
                    arrow: arrow.clone(),
                    src: RowId(idx),
                });
            }
        }
    }
    // Every declared equation on every row of its domain object. A path
    // through a dangling (or violated) cell cannot compose: the affected
    // row is already recorded by its dangling outcome and emits nothing
    // here — recorded, not crashed, never a violation.
    for (index, equation) in schema.equations.iter().enumerate() {
        let Some(lead) = equation.lhs.0.first() else {
            continue;
        };
        let Some(arrow) = schema.arrows.get(lead) else {
            continue;
        };
        let Some(rows) = instance.rows().get(&arrow.src) else {
            continue;
        };
        for &row in rows {
            let lhs = instance.compose(&equation.lhs, row);
            let rhs = instance.compose(&equation.rhs, row);
            let (Some(lhs), Some(rhs)) = (lhs, rhs) else {
                continue;
            };
            if lhs != rhs {
                outcomes.push(match equation.status {
                    ConstraintStatus::Accepted => Violation {
                        equation: index,
                        row,
                        lhs,
                        rhs,
                    },
                    ConstraintStatus::Proposed => Hypothesis {
                        equation: index,
                        row,
                        lhs,
                        rhs,
                    },
                });
            }
        }
    }
    outcomes
}
