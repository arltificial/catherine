//! Purpose: bounded instance search (openspec change `add-bounded-analysis`,
//! task 6) — explicit bounds in, an honest outcome out: a witness
//! instance, a proven-unsatisfiable contradiction finding (only when
//! exhaustion *completes* within bounds), or `UNKNOWN { scope }` with the
//! exact scope explored.
//!
//! Responsibilities: enumerate candidate instances over the declared
//! bounded shape with a deterministic smallest-first order (unresolved
//! cells first, then row ids ascending — the counterexample-minimality
//! discipline folded into the enumeration), evaluate each candidate with
//! the kernel's own `check` (a candidate witnesses satisfiability iff
//! every cell resolves and every *accepted* equation holds — proposed
//! equations fail as `hypothesized` questions and never block a
//! witness), and never claim what the
//! bounds did not establish: a budget that expires before exhaustion is
//! `UNKNOWN`, a zero-size bound is `UNKNOWN` with the empty scope — never
//! a contradiction — and "no violation found" is never an output.
//!
//! Rationale: plain hand-rolled enumeration over the finite row spaces —
//! no solver (ticket `anti_goal`). The witness contract is a *fully
//! resolved* instance: every arrow value resolves and every accepted
//! equation holds on every applicable row. Candidates that satisfy
//! equations only by dangling short-circuits prove nothing about the
//! constraints, so they do not witness — a documented limit: only fully
//! resolved candidates witness. Consequence, documented in the
//! crate README: pure path equalities are collapse-satisfiable
//! (one-row-collapse satisfies every equation), so a proven contradiction
//! arises exactly when the bounds make full resolution impossible — the
//! honest limit of the equality-only kernel language (DESIGN §4.1 keeps
//! injectivity and disequality out of v1). The accepted/proposed split is
//! P1's: only accepted equations constrain the search; proposed equations
//! are questions, not satisfiability constraints.

use crate::bounds::{contradiction_finding, SearchBounds, SearchScope};
use catherine_core::checks;
use catherine_core::findings::{EpistemicLabel, Finding, FindingKind};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Ident, Instance, RowId, Schema};
use std::collections::BTreeMap;

/// The search report: the caller's bound restated verbatim, and the
/// outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchReport {
    /// The bound the search ran under (restated, spec scenario).
    pub bound: SearchBounds,
    /// The honest outcome.
    pub outcome: SearchOutcome,
}

/// The search outcome. One of the three honest answers — never a naked
/// "no violation found".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchOutcome {
    /// The smallest fully resolved instance satisfying every accepted
    /// equation within the bounds (smallest-first enumeration).
    Witness {
        /// The witness instance: every arrow resolves, every accepted
        /// equation holds on every applicable row.
        instance: Instance,
    },
    /// Exhaustion completed within the bounds and no candidate was
    /// fully resolved and clean: the accepted equation set admits no
    /// instance in this scope. The finding carries the canonical
    /// equation indices and the `executed` label — a kernel-computed
    /// proof.
    Contradiction {
        /// The contradiction finding (P2 taxonomy, `executed`).
        finding: Finding,
    },
    /// The budget expired before exhaustion: a scope-bounded
    /// non-answer, labeled `unknown` by its producer and carried with
    /// the exact scope explored. First-class — never collapsed into
    /// clean, a finding, or an error.
    Unknown {
        /// The exact scope explored.
        scope: SearchScope,
    },
}

/// Search for a fully resolved instance satisfying every accepted
/// equation within the bounds. See the module docs for the contract.
pub fn search(schema: &Schema, bound: &SearchBounds) -> SearchReport {
    // Effective row counts: per schema object, the bound's value or zero.
    let counts = effective_counts(schema, bound);
    // Zero-size bound: nothing anywhere to search — UNKNOWN with the
    // empty scope, never a contradiction claim (spec scenario,
    // ticket anti_goal).
    let explored_rows = nonempty_counts(&counts);
    if explored_rows.is_empty() {
        return unknown_report(bound.clone(), BTreeMap::new(), 0);
    }
    // The enforced equation set: the accepted equations' canonical
    // indices, in order — the contradiction witness's scope.
    let accepted = accepted_indices(schema);
    // Cell plan and its mixed-radix counter: big-endian (the first
    // cell varies slowest), so candidate k is the k-th smallest
    // assignment. Exhaustion is the counter wrapping — no space size is
    // ever computed or claimed.
    let (cells, radices) = cell_plan(schema, &counts);
    let mut digits: Vec<usize> = vec![0; cells.len()];
    let mut explored: usize = 0;
    loop {
        // Budget check first: the budget expired before exhaustion is
        // UNKNOWN with the exact explored scope — never a contradiction.
        if explored == bound.max_candidates {
            return unknown_report(bound.clone(), explored_rows, explored);
        }
        // Materialize the candidate at the current counter position.
        let instance = materialize(&cells, &digits, &counts);
        explored += 1;
        if is_clean(schema, &instance) {
            return witness_report(bound.clone(), instance);
        }
        if !increment(&mut digits, &radices) {
            // The counter wrapped: exhaustion completed within the
            // bounds with no witness — the proof is real.
            return contradiction_report(bound.clone(), accepted);
        }
    }
}

/// The WITNESS report: the bound restated verbatim, the found instance.
fn witness_report(bound: SearchBounds, instance: Instance) -> SearchReport {
    SearchReport {
        bound,
        outcome: SearchOutcome::Witness { instance },
    }
}

/// The CONTRADICTION report: the kernel-computed proof over the accepted
/// equations' canonical indices.
fn contradiction_report(bound: SearchBounds, accepted: Vec<usize>) -> SearchReport {
    SearchReport {
        bound,
        outcome: SearchOutcome::Contradiction {
            finding: contradiction_finding(accepted),
        },
    }
}

/// Per schema object, the bound's row count or zero.
fn effective_counts(schema: &Schema, bound: &SearchBounds) -> BTreeMap<Ident, usize> {
    schema
        .objects
        .keys()
        .map(|object| {
            (
                object.clone(),
                bound.max_rows.get(object).copied().unwrap_or(0),
            )
        })
        .collect()
}

/// The objects with a nonzero row count — the scope actually explored.
fn nonempty_counts(counts: &BTreeMap<Ident, usize>) -> BTreeMap<Ident, usize> {
    counts
        .iter()
        .filter(|(_, &count)| count > 0)
        .map(|(object, &count)| (object.clone(), count))
        .collect()
}

/// The UNKNOWN report: the bound restated verbatim, the exact scope
/// explored.
fn unknown_report(
    bound: SearchBounds,
    rows: BTreeMap<Ident, usize>,
    candidates: usize,
) -> SearchReport {
    SearchReport {
        bound,
        outcome: SearchOutcome::Unknown {
            scope: SearchScope {
                rows,
                candidates_explored: candidates,
            },
        },
    }
}

/// The accepted equations' canonical indices, in declaration order.
fn accepted_indices(schema: &Schema) -> Vec<usize> {
    schema
        .equations
        .iter()
        .enumerate()
        .filter(|(_, equation)| equation.status == ConstraintStatus::Accepted)
        .map(|(index, _)| index)
        .collect()
}

/// Cell plan: one slot per (arrow, source row), arrow-major (the
/// BTreeMap's sorted order) then row-ascending. Each slot's options:
/// the unresolved value first, then the target rows ascending — the
/// smallest-first order (fewest resolved references explored first).
/// Returns the slots and the radices of the mixed-radix counter.
fn cell_plan(
    schema: &Schema,
    counts: &BTreeMap<Ident, usize>,
) -> (Vec<(Ident, usize, usize)>, Vec<usize>) {
    let mut cells: Vec<(Ident, usize, usize)> = Vec::new(); // (arrow, source row, option count)
    for (name, arrow) in &schema.arrows {
        let n_src = counts.get(&arrow.src).copied().unwrap_or(0);
        let n_tgt = counts.get(&arrow.tgt).copied().unwrap_or(0);
        for row in 0..n_src {
            cells.push((name.clone(), row, n_tgt + 1));
        }
    }
    let radices: Vec<usize> = cells.iter().map(|(_, _, options)| *options).collect();
    (cells, radices)
}

/// Materialize the candidate at the current counter position: each
/// digit 0 is the unresolved cell, digit d > 0 is `RowId(d - 1)`.
fn materialize(
    cells: &[(Ident, usize, usize)],
    digits: &[usize],
    counts: &BTreeMap<Ident, usize>,
) -> Instance {
    let mut values: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for (slot, (name, _, _)) in cells.iter().enumerate() {
        let digit = digits[slot];
        let cell = if digit == 0 {
            None
        } else {
            Some(RowId(digit - 1))
        };
        values.entry(name.clone()).or_default().push(cell);
    }
    Instance {
        rows: counts
            .iter()
            .map(|(object, &count)| (object.clone(), (0..count).map(RowId).collect::<Vec<_>>()))
            .collect(),
        values,
    }
}

/// The satisfaction predicate: the candidate witnesses satisfiability
/// iff every arrow cell resolves (no dangling finding) and every
/// *accepted* equation holds (no `executed` violation). Proposed
/// equations fail with the `hypothesized` label — questions under
/// review, not satisfiability constraints — so a `hypothesized`
/// violation never blocks a witness.
fn is_clean(schema: &Schema, instance: &Instance) -> bool {
    let typed = TypedInstance::build(schema, instance);
    checks::check(schema, &typed).into_iter().all(|finding| {
        match finding.kind {
            // Dangling short-circuits prove nothing: never clean.
            FindingKind::Dangling(_) => false,
            // Accepted-equation failures are kernel-computed (`executed`);
            // proposed-equation failures are questions (`hypothesized`).
            FindingKind::Violation(_) => finding.label != EpistemicLabel::Executed,
            // No other kind is emitted by the kernel walk.
            _ => true,
        }
    })
}

/// Advance the big-endian mixed-radix counter. `false` = wrapped: the
/// whole space has been enumerated.
fn increment(digits: &mut [usize], radices: &[usize]) -> bool {
    for slot in (0..digits.len()).rev() {
        digits[slot] += 1;
        if digits[slot] < radices[slot] {
            return true;
        }
        digits[slot] = 0;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bounds::SearchBounds;
    use catherine_core::model::{Arrow, Equation, Path};

    /// One object `a` with the endoarrow `f`; the accepted equation
    /// `f;f;f = f` (holds on the swap, on constants, on identities) and
    /// the proposed equation `f = f;f` (rejects the swap — it is a
    /// question, not a constraint).
    fn swap_schema() -> Schema {
        let mut objects = BTreeMap::new();
        objects.insert(Ident("a".to_string()), ());
        let mut arrows = BTreeMap::new();
        arrows.insert(
            Ident("f".to_string()),
            Arrow {
                src: Ident("a".to_string()),
                tgt: Ident("a".to_string()),
            },
        );
        let eq = |lhs: &[&str], rhs: &[&str], status| Equation {
            lhs: Path(lhs.iter().map(|s| Ident(s.to_string())).collect()),
            rhs: Path(rhs.iter().map(|s| Ident(s.to_string())).collect()),
            rationale: String::new(),
            status,
        };
        Schema {
            objects,
            arrows,
            equations: vec![
                eq(&["f", "f", "f"], &["f"], ConstraintStatus::Accepted),
                eq(&["f"], &["f", "f"], ConstraintStatus::Proposed),
            ],
        }
    }

    fn two_row_bound() -> SearchBounds {
        let mut max_rows = BTreeMap::new();
        max_rows.insert(Ident("a".to_string()), 2);
        SearchBounds {
            max_rows,
            max_candidates: 1000,
        }
    }

    /// The swap instance: fully resolved, the accepted equation holds
    /// (`f³ = f` on an involution), the proposed equation fails
    /// (`f ≠ f²`). `is_clean` judges satisfiability by the *accepted*
    /// equations only — a hypothesized violation marks a question under
    /// review, never a blocked witness.
    #[test]
    fn is_clean_accepts_accepted_clean_proposed_dirty() {
        let schema = swap_schema();
        let mut values = BTreeMap::new();
        values.insert(Ident("f".to_string()), vec![Some(RowId(1)), Some(RowId(0))]);
        let instance = Instance {
            rows: BTreeMap::from([(Ident("a".to_string()), vec![RowId(0), RowId(1)])]),
            values,
        };
        assert!(is_clean(&schema, &instance));
    }

    /// End to end: the same schema under search returns a Witness, never
    /// a Contradiction — a proposed-equation violation must not block
    /// the outcome even if every enumerated candidate carried one.
    #[test]
    fn search_returns_witness_despite_proposed_violations() {
        let schema = swap_schema();
        let report = search(&schema, &two_row_bound());
        assert!(
            matches!(report.outcome, SearchOutcome::Witness { .. }),
            "expected a witness, got {:?}",
            report.outcome
        );
    }
}
