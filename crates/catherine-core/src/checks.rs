//! Purpose: executed path-equation checks — the E3 bet. `check` evaluates
//! every declared equation on every applicable row and emits labeled
//! findings with witnesses (openspec changes `add-executed-checks` and
//! `add-findings-taxonomy`).
//!
//! Responsibilities: [`check`] walks the typed instance tables — one
//! dangling finding per unresolved cell, then one finding per equation per
//! applicable row whose composed paths disagree — and never emits a
//! finding without its witness. A dangling value short-circuits path
//! composition at that arrow: the affected row is recorded (by the
//! dangling finding naming arrow + source row), never reported as a
//! violation, and the run continues. Equation `status` decides the
//! epistemic label of a violation-shaped finding: `accepted` →
//! `executed`, `proposed` → `hypothesized` — one kind, countable apart by
//! label, never merged into one severity. Totality: an empty schema or an
//! empty instance yields zero findings — silence after evaluation is real
//! success, never an unevaluated declaration.
//!
//! Rationale: new code (E10) — specodelic's acset has no path-equation
//! execution; equations were already new in P0. P1's `CheckOutcome` enum
//! is superseded by the findings taxonomy (evolved into `findings.rs`,
//! not wrapped — kind names the shape, the label names the weight).
//! Finding order is the taxonomy's total order: kind first in
//! declaration order (violations before dangling — P1's dangling-first
//! outcome order is superseded per the findings spec), then witness ids
//! in canonical id order, label last as tiebreak; `check` sorts before
//! returning, so repeated checks of the same model serialize
//! byte-identically (findings spec: repeated runs are diff-stable).

use crate::findings::{DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind};
use crate::instance::TypedInstance;
use crate::model::{ConstraintStatus, Ident, RowId, Schema};
use std::collections::BTreeSet;

/// Execute every declared equation on every applicable row of the typed
/// instance. See the module docs for the finding contract.
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
pub fn check(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    let mut findings = Vec::new();
    // Dangling values first in the walk (the taxonomy's kind order sorts
    // them after the violations): one finding per unresolved cell that is
    // not accounted for by a typing violation (a violated value never
    // became a table entry — its `None` cell is a build report, not a
    // dangling reference).
    let violated: BTreeSet<(Ident, RowId)> = instance
        .typing_violations()
        .iter()
        .map(|v| (v.arrow.clone(), v.src))
        .collect();
    findings.extend(dangling_findings(instance, &violated));
    findings.extend(equation_findings(schema, instance));
    // The taxonomy's total order (kind declaration order — violations
    // before dangling — then witness ids, label last) is the emitted
    // contract: the walk's push order is not, and the sorted result makes
    // repeated checks byte-identical (findings spec: repeated runs are
    // diff-stable).
    findings.sort();
    findings
}

/// One dangling finding per unresolved cell not accounted for by a typing
/// violation.
fn dangling_findings(
    instance: &TypedInstance,
    violated: &BTreeSet<(Ident, RowId)>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (arrow, cells) in instance.tables() {
        for (idx, cell) in cells.iter().enumerate() {
            if cell.is_none() && !violated.contains(&(arrow.clone(), RowId(idx))) {
                findings.push(Finding {
                    label: EpistemicLabel::Executed,
                    kind: FindingKind::Dangling(DanglingWitness {
                        arrow: arrow.clone(),
                        src: RowId(idx),
                    }),
                });
            }
        }
    }
    findings
}

/// Every declared equation on every row of its domain object. A path
/// through a dangling (or violated) cell cannot compose: the affected
/// row is already recorded by its dangling finding and emits nothing
/// here — recorded, not crashed, never a violation. The failure's
/// witness is constructed once; only the label rides on the equation's
/// status (accepted → executed, proposed → hypothesized).
fn equation_findings(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    let mut findings = Vec::new();
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
            if let Some(finding) = equation_finding(schema, instance, index, row) {
                findings.push(finding);
            }
        }
    }
    findings
}

/// The violation finding for one equation on one row, if the composed
/// paths exist and disagree.
fn equation_finding(
    schema: &Schema,
    instance: &TypedInstance,
    index: usize,
    row: RowId,
) -> Option<Finding> {
    let equation = &schema.equations[index];
    let lhs = instance.compose(&equation.lhs, row);
    let rhs = instance.compose(&equation.rhs, row);
    let (Some(lhs), Some(rhs)) = (lhs, rhs) else {
        return None;
    };
    if lhs == rhs {
        return None;
    }
    let label = match equation.status {
        ConstraintStatus::Accepted => EpistemicLabel::Executed,
        ConstraintStatus::Proposed => EpistemicLabel::Hypothesized,
    };
    Some(Finding {
        label,
        kind: FindingKind::Violation(EquationWitness {
            equation: index,
            row,
            lhs,
            rhs,
        }),
    })
}
