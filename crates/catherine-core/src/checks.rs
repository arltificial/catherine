//! Purpose: executed path-equation checks — the E3 bet. `check` evaluates
//! every declared equation on every applicable row and emits labeled
//! findings with witnesses (openspec changes `add-executed-checks` and
//! `add-findings-taxonomy`).
//!
//! Responsibilities: [`check`] walks the typed instance tables — one
//! dangling finding per unresolved cell, one finding per equation per
//! applicable row whose composed paths disagree, one finding per
//! unordered pair of rows whose declared key tuple agrees, one
//! finding per row of a declared coproduct whose coverage or
//! disjointness claim fails, and one finding per offending row or fiber
//! pair of a declared pullback whose commutes, covers, or unique verdict
//! fails — and never
//! emits a finding without its witness. A dangling value short-circuits
//! path composition at that arrow: the affected row is recorded (by the
//! dangling finding naming arrow + source row), never reported as a
//! violation, never grouped into a key collision, and never used to
//! decide coproduct coverage (an unresolved injection cell could reach
//! any row, so uncovered-row findings wait for every image); a row
//! already in two *complete* injection images is still a decidable
//! disjointness violation, and the run continues. A pullback follows the
//! same rule: a dangling leg cell leaves that row's commutativity and
//! uniqueness undefined and suspends coverage for the declaration (an
//! unknown projection could occupy any fiber pair), while a dangling
//! cospan cell only removes its pair from the decidable fiber.
//! Constraint `status` decides the
//! epistemic label of a violation-shaped finding: `accepted` →
//! `executed`, `proposed` → `hypothesized` — one kind, countable apart by
//! label, never merged into one severity. Totality: an empty schema or an
//! empty instance yields zero findings — silence after evaluation is real
//! success, not an unevaluated declaration.
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
//! The result is algebraically a monoid: per-equation batches of the
//! same model add additively once sorted — each finding belongs to
//! exactly one equation or dangling cell, and the sort normalizes walk
//! order to canonical form. Sub-model checks do NOT compose the same
//! way: a combined model can carry cross-object equations no sub-model
//! sees, so the additivity is over batches of one model, never over
//! schema fragments. Pullbacks are the additive 1.6 constraint family
//! (`add-pullback-checks`): their three verdicts are ordinary violation
//! findings with a new witness arm, so they ride the same label rule,
//! the same dangling discipline, and the same sort — no new kind, exit
//! code, or severity.

use crate::engine::{engine_families, Bounded, EngineSchema, EngineTyped};
use crate::findings::Finding;
use crate::instance::TypedInstance;
use crate::model::Schema;
use crate::render::render_compact;
use serde::Serialize;

/// What the check walk actually executed (openspec change
/// `add-execution-scope`): the honesty floor that makes a clean result
/// distinguishable from a vacuous one — a below-floor document, a
/// constraint-free schema, and a zero-row instance would otherwise all
/// serialize the same empty envelope as a genuine pass. The three keys
/// are the public payload contract (`data.scope`): snake_case, integers,
/// always present on check/findings payloads — zeros are the honest
/// empty scope, never omission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExecutionScope {
    /// Accepted constraints the check walk executed. Zero ⇒ nothing was
    /// checked regardless of the findings.
    pub constraints_checked: usize,
    /// Instance rows the walk evaluated: the sum of every object's row
    /// space (the walks iterate those spaces). Zero ⇒ vacuous instance.
    pub rows_evaluated: usize,
    /// Accepted equations that bind zero rows — the subtle vacuity:
    /// constraints and rows can both be non-zero while every equation
    /// ranges over an empty table and the walk executes nothing.
    pub equations_over_zero_rows: usize,
}

/// Execute every declared constraint — equations, keys, coproducts, and
/// pullbacks — on every applicable row of the typed instance, over the
/// interned execution core (`engine`), and report the [`ExecutionScope`]
/// of the same walk. The one path both `check` and `findings` consume:
/// the scope is computed here, from the walk's own compiled surfaces, so
/// the two subcommands can never disagree about it. The finding contract
/// is the module doc above; the engine emits the same walks over dense
/// ids, and its int-key sort is exactly the taxonomy order this function
/// used to sort under (id order is name order by construction — see
/// `engine::EngineSchema::compile`).
pub fn check_with_scope(
    schema: &Schema,
    instance: &TypedInstance,
) -> (Vec<Finding>, ExecutionScope) {
    let compiled = crate::engine::EngineSchema::compile(schema);
    let typed = compile_typed(&compiled, instance);
    let scope = execution_scope(&compiled, &typed);
    let findings = engine_families(&compiled, &typed)
        .into_iter()
        .map(|(key, label, compact)| {
            let _ = key;
            render_compact(&compiled, label, &compact)
        })
        .collect();
    (findings, scope)
}

/// The check walk without the scope — the pre-existing entry point,
/// unchanged for every consumer that does not need the execution scope.
pub fn check(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    check_with_scope(schema, instance).0
}

/// The walk's scope, over surfaces already compiled: accepted,
/// applicable constraints across the four families; the summed row
/// spaces; and the accepted equations whose domain table is empty (the
/// walk skips them — the vacuity the third key names).
fn execution_scope(compiled: &EngineSchema, typed: &EngineTyped) -> ExecutionScope {
    let row = |nid: usize| typed.row_counts[nid].unwrap_or(0);
    let equations = compiled
        .equations
        .iter()
        .filter(|equation| {
            equation.status == crate::model::ConstraintStatus::Accepted && equation.domain.is_some()
        })
        .collect::<Vec<_>>();
    let accepted = count_accepted(&compiled.keys, |key| &key.status)
        + count_accepted(&compiled.coproducts, |c| &c.status)
        + count_accepted(&compiled.pullbacks, |pb| &pb.status);
    ExecutionScope {
        constraints_checked: equations.len() + accepted,
        rows_evaluated: typed.row_counts.iter().map(|rows| rows.unwrap_or(0)).sum(),
        equations_over_zero_rows: equations
            .iter()
            .filter(|equation| row(equation.domain.unwrap()) == 0)
            .count(),
    }
}

/// The accepted, walk-applicable declarations in one family's compiled
/// list (`None` entries are the walk's skip cases — never counted).
fn count_accepted<T>(
    list: &[Option<T>],
    status_of: impl Fn(&T) -> &crate::model::ConstraintStatus,
) -> usize {
    list.iter()
        .flatten()
        .filter(|constraint| *status_of(constraint) == crate::model::ConstraintStatus::Accepted)
        .count()
}

/// Intern a typed instance's tables against a compiled presentation
/// (the parsed-surface check entry: the tables already exist).
fn compile_typed(compiled: &EngineSchema, typed: &TypedInstance) -> EngineTyped {
    let row_counts = compiled
        .object_names
        .iter()
        .map(|name| typed.rows().get(name).map(Vec::len))
        .collect();
    let tables = compiled
        .arrow_names
        .iter()
        .map(|name| Bounded::from_option_vec(typed.tables().get(name).cloned().unwrap_or_default()))
        .collect();
    let mut typing_violations = Bounded::new();
    for v in typed.typing_violations() {
        if let Some(nid) = compiled.arrow_nid(&v.arrow) {
            typing_violations.push((nid, v.src.0, v.value));
        }
    }
    let mut dangling = Bounded::new();
    for d in typed.dangling() {
        if let Some(nid) = compiled.arrow_nid(&d.arrow) {
            dangling.push((nid, d.src.0));
        }
    }
    EngineTyped {
        row_counts,
        tables,
        typing_violations,
        dangling,
    }
}
