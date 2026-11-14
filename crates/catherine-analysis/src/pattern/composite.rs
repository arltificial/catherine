//! Purpose: the `composite` pattern lens (beads `catherine-mx1.3`) — the
//! Composite-as-monoid row's Identity law: the composite dispatch's
//! identity/totality equations (`parent ∘ top = top`, `top ∘ top = top`)
//! execute on every row, and every child's parent link resolves.
//!
//! Responsibilities: one `'static` [`PatternMeta`] matching the
//! disposition row, and `run` reusing the kernel operations the kata's
//! recorded commands ran — `check` for the executed equation and dangling
//! findings, and `migrate` for the flattening schema map's per-cell
//! lost-data enumeration — over the built model. Both channels stay in
//! their existing shapes (P2 [`Finding`]s and [`MigrationFinding`]s); the
//! lens adds no kind, witness, or heuristic. Input with no rule table to
//! bind (a non-empty `--values`) is [`LensOutcome::Unknown`] naming the
//! mismatch, never an ignored flag or an empty success.
//!
//! Rationale: the sealed kata evidence
//! (`ledger/katas/composite-null-object/`) records exactly these surfaces:
//! the floor control's `dangling:parent@Node#0` plus
//! `violation:eq0@Node#1`, the deep broken link's dangling cell, the
//! nil-as-real-leaf L1 violation, and the flatten migration's seven
//! `lost_cell` witnesses. Check and migrate are the same calls that
//! produced those elements, so the lens proves nothing wider than the
//! recorded law. Monoid associativity is deliberately absent: the v1
//! instance language has no list/multiset/operation surface that could
//! state it, and the gap is recorded in the kata rather than approximated
//! here.
//!
//! [`Finding`]: catherine_core::findings::Finding
//! [`MigrationFinding`]: catherine_core::migrate::MigrationFinding

use crate::bounds::ScopeCaps;
use crate::pattern::{LawKind, LensFindings, LensInput, LensOutcome, PatternLens, PatternMeta};
use catherine_core::checks::check;
use catherine_core::instance::TypedInstance;
use catherine_core::migrate;

/// The disposition row's metadata, verbatim (Composite: monoid identity,
/// identity/totality equations; associativity unexpressible).
pub static COMPOSITE_META: PatternMeta = PatternMeta {
    id: "composite",
    name: "Composite",
    abstraction: "monoid (identity)",
    law: LawKind::Identity,
    scope: "identity/totality equations; associativity unexpressible",
};

/// The Composite lens: executed identity/totality equations and the
/// flattening map's lost-data enumeration over the built model.
///
/// Stateless and therefore a `static` value shared by the registry.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &COMPOSITE_META
    }

    fn run(&self, input: &LensInput<'_>, _caps: &ScopeCaps) -> LensOutcome {
        // Equation/Δ surface only: this lens has no rule table, so a
        // supplied finite domain cannot bind and must not be ignored.
        if !input.values.is_empty() {
            return LensOutcome::Unknown {
                scope: "`--values` supplied to a lens with no rule table; nothing evaluated"
                    .to_string(),
            };
        }
        let typed = TypedInstance::build(&input.model.schema, &input.model.instance);
        let checks = check(&input.model.schema, &typed);
        // The flattening map is the kata's Δ surface; its lost-data list is
        // the migrate engine's own findings, reported verbatim.
        let migrations = migrate::reindex(input.model)
            .map(|outcome| outcome.lost)
            .unwrap_or_default();
        LensOutcome::Findings(LensFindings { checks, migrations })
    }
}
