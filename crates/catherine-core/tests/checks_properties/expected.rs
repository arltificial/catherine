//! The independent recomputation of the check contract (written from the
//! kernel-checks + findings specs, not from `check`'s code): one executed
//! dangling finding per unresolved, non-typing-violated cell (arrow-name
//! order, then source row order), then per equation (canonical order)
//! per domain row, a violation-shaped finding labeled by status when
//! both paths compose and disagree — sorted under the taxonomy's total
//! order (kind first, then witness ids), exactly as `check` emits.

use catherine_core::findings::{
    DanglingWitness, EpistemicLabel, EquationWitness, Finding, FindingKind, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Equation, Ident, Path, RowId, Schema};

pub fn expected_findings(schema: &Schema, built: &TypedInstance) -> Vec<Finding> {
    let mut out = expected_dangling(built);
    out.extend(expected_violations(schema, built));
    out.sort();
    out
}

/// The executed dangling findings: one per unresolved cell that is not
/// already a typing violation (arrow-name order, then source row order).
fn expected_dangling(built: &TypedInstance) -> Vec<Finding> {
    let violated: Vec<(String, usize)> = built
        .typing_violations()
        .iter()
        .map(|v| (v.arrow.0.clone(), v.src.0))
        .collect();
    built
        .tables()
        .iter()
        .flat_map(|(name, cells)| dangling_in_table(name, cells, &violated))
        .collect()
}

/// The dangling findings in one arrow's table: its unresolved cells that
/// are not typing violations.
fn dangling_in_table(
    name: &Ident,
    cells: &[Option<RowId>],
    violated: &[(String, usize)],
) -> Vec<Finding> {
    cells
        .iter()
        .enumerate()
        .filter(|(i, cell)| cell.is_none() && !violated.contains(&(name.0.clone(), *i)))
        .map(|(i, _)| Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Dangling(DanglingWitness {
                arrow: name.clone(),
                src: RowId(i),
            }),
        })
        .collect()
}

/// The equation findings: per equation (canonical order) per domain row,
/// a violation-shaped finding labeled by status when both paths compose
/// and disagree.
fn expected_violations(schema: &Schema, built: &TypedInstance) -> Vec<Finding> {
    let mut out = Vec::new();
    for (index, equation) in schema.equations.iter().enumerate() {
        out.extend(equation_violations(schema, built, index, equation));
    }
    out
}

/// One equation's violation findings over its domain rows.
fn equation_violations(
    schema: &Schema,
    built: &TypedInstance,
    index: usize,
    equation: &Equation,
) -> Vec<Finding> {
    let lead = match schema.arrows.get(&equation.lhs.0[0]) {
        Some(lead) => lead,
        None => return Vec::new(),
    };
    let n_rows = built.rows().get(&lead.src).map(Vec::len).unwrap_or(0);
    (0..n_rows)
        .filter_map(|row| row_violation(built, index, equation, row))
        .collect()
}

/// One domain row's finding: the witnessed violation when both equation
/// sides compose and disagree, labeled by the equation's status.
fn row_violation(
    built: &TypedInstance,
    index: usize,
    equation: &Equation,
    row: usize,
) -> Option<Finding> {
    let lhs = naive_compose(built, &equation.lhs, row);
    let rhs = naive_compose(built, &equation.rhs, row);
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) if lhs != rhs => Some(Finding {
            label: match equation.status {
                ConstraintStatus::Accepted => EpistemicLabel::Executed,
                ConstraintStatus::Proposed => EpistemicLabel::Hypothesized,
            },
            kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: index,
                row: RowId(row),
                lhs,
                rhs,
            })),
        }),
        _ => None,
    }
}

fn naive_compose(built: &TypedInstance, path: &Path, start: usize) -> Option<RowId> {
    let mut current = start;
    for name in &path.0 {
        let cells = built.tables().get(name)?;
        let cell = cells.get(current)?;
        current = (*cell)?.0;
    }
    Some(RowId(current))
}
