//! Purpose: the pullback half of the executed checks — a declared
//! pullback square's universal property evaluated as three decidable
//! verdicts over the typed instance (openspec change
//! `add-pullback-checks`, design Decision 2).
//!
//! Responsibilities: [`pullback_findings`] walks every declared pullback
//! and emits one witness-bearing violation per offending apex row
//! (commutes) and per offending row pair (unique), plus one per uncovered
//! fiber pair (covers). A dangling leg cell withholds that row's own
//! commutativity and uniqueness contribution — the row is absent from the
//! projection buckets and its dangling finding already names the cell —
//! while uniqueness findings for the complete projection pairs still
//! emit. Coverage for the whole declaration waits until every apex leg
//! cell resolves (an unknown projection could occupy any fiber pair). A
//! dangling cospan cell only removes its pair from the decidable fiber:
//! the fiber is a function of the cospan alone.
//!
//! Rationale: commutativity alone cannot distinguish a pullback from any
//! commutative square, so coverage and uniqueness are checked separately
//! (design Decision 2). The undecidability rules mirror the coproduct
//! pass: an unresolved cell keeps its own dangling finding and never
//! becomes a violation, and zero rows yield zero findings — an empty
//! apex over an empty fiber is evaluated-clean, silence after running.

use super::label_for;
use crate::findings::{EpistemicLabel, Finding, FindingKind, PullbackWitness, ViolationWitness};
use crate::instance::TypedInstance;
use crate::model::{Ident, Pullback, RowId, Schema};
use std::collections::BTreeMap;

/// One declaration's four arrows resolved to their value tables, plus the
/// row spaces of the cospan sources. `None` tables never occur for a
/// schema-built instance; the resolution still guards undeclared arrows
/// and off-shape declarations from programmatic schemas.
struct Square<'a> {
    /// `p1`'s cells, parallel to the apex rows.
    p1_table: &'a [Option<RowId>],
    /// `p2`'s cells, parallel to the apex rows.
    p2_table: &'a [Option<RowId>],
    /// `f`'s cells, parallel to the `A` rows (`f`'s source).
    f_table: &'a [Option<RowId>],
    /// `g`'s cells, parallel to the `B` rows (`g`'s source).
    g_table: &'a [Option<RowId>],
    /// The `A` row space (`f`'s source object).
    a_rows: &'a [RowId],
    /// The `B` row space (`g`'s source object).
    b_rows: &'a [RowId],
}

/// Every declared pullback, in canonical declaration order. A
/// declaration outside the parsed shape (undeclared arrow, leg not
/// sourced at the apex, cospan targets disagreeing) emits nothing — it
/// is unrepresentable through parse and only reachable from
/// programmatically assembled schemas.
pub(super) fn pullback_findings(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, pullback) in schema.pullbacks.iter().enumerate() {
        findings.extend(declaration_findings(schema, instance, index, pullback));
    }
    findings
}

/// One declaration's three verdicts. Rows are walked in ascending id
/// order (parsed row spaces are dense), so per-row witnesses and
/// projection buckets come out in canonical order.
fn declaration_findings(
    schema: &Schema,
    instance: &TypedInstance,
    index: usize,
    pullback: &Pullback,
) -> Vec<Finding> {
    let Some((square, rows)) = square_and_rows(schema, instance, pullback) else {
        return Vec::new();
    };
    let label = label_for(pullback.status);
    let mut findings = Vec::new();
    let mut projections: BTreeMap<(RowId, RowId), Vec<RowId>> = BTreeMap::new();
    let mut legs_partial = false;
    for &row in rows {
        let Some(projection) = project_row(&square, index, label, row, &mut findings) else {
            // An unresolved leg cell makes this row's commutativity
            // undefined; the dangling finding already names it, and the
            // partial projector set withholds coverage below.
            legs_partial = true;
            continue;
        };
        projections.entry(projection).or_default().push(row);
    }
    findings.extend(unique_findings(index, label, &projections));
    if !legs_partial {
        findings.extend(square.coverage_findings(index, label, &projections));
    }
    findings
}

/// The declaration's resolved square and apex row space; `None` when the
/// declaration is outside the parsed shape or its apex has no rows.
fn square_and_rows<'a>(
    schema: &'a Schema,
    instance: &'a TypedInstance,
    pullback: &'a Pullback,
) -> Option<(Square<'a>, &'a [RowId])> {
    let square = Square::resolve(schema, instance, pullback)?;
    let rows = instance.rows().get(&pullback.apex).map(Vec::as_slice)?;
    Some((square, rows))
}

/// One apex row's verdict: its projection when both leg cells resolve,
/// plus its commutes finding when the two composed values disagree.
fn project_row(
    square: &Square<'_>,
    index: usize,
    label: EpistemicLabel,
    row: RowId,
    findings: &mut Vec<Finding>,
) -> Option<(RowId, RowId)> {
    let projection = square.leg_values(row)?;
    // The verdict is a function of the projection alone; the row is
    // carried only by the witness.
    if let Some((lhs, rhs)) = square.commutes(projection) {
        findings.push(violation(
            label,
            PullbackWitness::Commutes {
                pullback: index,
                row,
                lhs,
                rhs,
            },
        ));
    }
    Some(projection)
}

/// One uniqueness finding per unordered pair of apex rows sharing a
/// projection, ascending rows (a 3-way duplication is three findings —
/// each pair is a demonstrated failure, not a summary count).
fn unique_findings(
    index: usize,
    label: EpistemicLabel,
    projections: &BTreeMap<(RowId, RowId), Vec<RowId>>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for rows in projections.values() {
        // The witness contract says ascending rows; parsed apex row spaces
        // are already ascending, so this only guards programmatic
        // instances.
        let mut rows = rows.clone();
        rows.sort_unstable();
        for (position, &lo) in rows.iter().enumerate() {
            for &hi in &rows[position + 1..] {
                findings.push(violation(
                    label,
                    PullbackWitness::Unique {
                        pullback: index,
                        rows: [lo, hi],
                    },
                ));
            }
        }
    }
    findings
}

impl<'a> Square<'a> {
    /// Resolve the declaration's arrows and tables; `None` when the
    /// declaration is outside the parsed presentation's shape.
    fn resolve(
        schema: &'a Schema,
        instance: &'a TypedInstance,
        pullback: &'a Pullback,
    ) -> Option<Self> {
        let (p1, p2) = two(&pullback.legs)?;
        let (f, g) = two(&pullback.cospan)?;
        let leg1 = schema.arrows.get(p1)?;
        let leg2 = schema.arrows.get(p2)?;
        if leg1.src != pullback.apex || leg2.src != pullback.apex {
            return None;
        }
        let cospan1 = schema.arrows.get(f)?;
        let cospan2 = schema.arrows.get(g)?;
        if cospan1.tgt != cospan2.tgt {
            return None;
        }
        let tables = instance.tables();
        let rows = instance.rows();
        let no_rows: &'a [RowId] = &[];
        Some(Self {
            p1_table: tables.get(p1)?.as_slice(),
            p2_table: tables.get(p2)?.as_slice(),
            f_table: tables.get(f)?.as_slice(),
            g_table: tables.get(g)?.as_slice(),
            a_rows: rows
                .get(&cospan1.src)
                .map_or(no_rows, |rows| rows.as_slice()),
            b_rows: rows
                .get(&cospan2.src)
                .map_or(no_rows, |rows| rows.as_slice()),
        })
    }

    /// One apex row's projection `(p1[row], p2[row])`; `None` when either
    /// leg cell is unresolved (dangling or typing-violated), which leaves
    /// the row's verdict undefined rather than violated.
    fn leg_values(&self, row: RowId) -> Option<(RowId, RowId)> {
        let left = self.p1_table.get(row.0).copied().flatten()?;
        let right = self.p2_table.get(row.0).copied().flatten()?;
        Some((left, right))
    }

    /// Both composed cospan values for one projection, when they exist
    /// and disagree: `f` applied to the first value, `g` applied to the
    /// second. A composition through a dangling cospan cell is undefined,
    /// never a violation.
    fn commutes(&self, projection: (RowId, RowId)) -> Option<(RowId, RowId)> {
        let (a, b) = projection;
        let lhs = self.f_table.get(a.0).copied().flatten()?;
        let rhs = self.g_table.get(b.0).copied().flatten()?;
        (lhs != rhs).then_some((lhs, rhs))
    }

    /// Every uncovered pair of the resolved fiber. The fiber is
    /// enumerated by common cospan target, then `A` rows, then `B` rows —
    /// all `BTreeMap`/row-id order, so missing-pair findings are
    /// countable and diff-stable.
    fn coverage_findings(
        &self,
        index: usize,
        label: EpistemicLabel,
        projections: &BTreeMap<(RowId, RowId), Vec<RowId>>,
    ) -> Vec<Finding> {
        let a_fibers = fiber(self.a_rows, self.f_table);
        let b_fibers = fiber(self.b_rows, self.g_table);
        let mut findings = Vec::new();
        for (target, a_rows) in &a_fibers {
            let Some(b_rows) = b_fibers.get(target) else {
                continue;
            };
            for &a in a_rows {
                // Ascending `B` rows by construction; the filter is
                // branchless so the coverage walk keeps its stable order
                // without a third nesting level.
                let uncovered = b_rows
                    .iter()
                    .copied()
                    .filter(|&b| !projections.contains_key(&(a, b)));
                for b in uncovered {
                    findings.push(violation(
                        label,
                        PullbackWitness::Covers {
                            pullback: index,
                            pair: [a, b],
                        },
                    ));
                }
            }
        }
        findings
    }
}

/// The declaration's exactly-two arrow names, in declared order.
fn two(arrows: &[Ident]) -> Option<(&Ident, &Ident)> {
    match arrows {
        [first, second] => Some((first, second)),
        _ => None,
    }
}

/// Group an arrow's source rows by resolved target value, ascending.
/// Rows with a dangling (or typing-violated) cell have no target and are
/// excluded — their pair is outside the decidable fiber.
fn fiber(rows: &[RowId], table: &[Option<RowId>]) -> BTreeMap<RowId, Vec<RowId>> {
    let mut fibers: BTreeMap<RowId, Vec<RowId>> = BTreeMap::new();
    for &row in rows {
        if let Some(target) = table.get(row.0).copied().flatten() {
            fibers.entry(target).or_default().push(row);
        }
    }
    for rows in fibers.values_mut() {
        rows.sort_unstable();
    }
    fibers
}

/// The pullback violation finding: the declaration's canonical index and
/// the verdict-specific offender, under the declaration's status label.
fn violation(label: EpistemicLabel, witness: PullbackWitness) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Pullback(witness)),
    }
}
