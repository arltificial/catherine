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

use crate::findings::{EpistemicLabel, Finding};
use crate::instance::TypedInstance;
use crate::model::{ConstraintStatus, Schema};

/// Execute every declared constraint — equations, keys, coproducts, and
/// pullbacks — on every applicable row of the typed instance, over the
/// interned execution core (`engine`). The finding contract is the
/// module doc above; the engine emits the same walks over dense ids, and
/// its int-key sort is exactly the taxonomy order this function used to
/// sort under (id order is name order by construction — see
/// `engine::EngineSchema::compile`).
pub fn check(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    let compiled = crate::engine::EngineSchema::compile(schema);
    let typed = compiled.compile_typed(instance);
    crate::engine::engine_check(&compiled, &typed)
}

/// The epistemic weight of a constraint failure: accepted constraints are
/// executed, proposed ones are questions (the one label rule shared by
/// equations, keys, coproducts, and pullbacks).
pub(crate) fn label_for(status: ConstraintStatus) -> EpistemicLabel {
    match status {
        ConstraintStatus::Accepted => EpistemicLabel::Executed,
        ConstraintStatus::Proposed => EpistemicLabel::Hypothesized,
    }
}
