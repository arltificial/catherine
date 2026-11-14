//! Purpose: the `null-object` pattern lens (beads `catherine-mx1.4`) —
//! the Null Object row's Identity law: the distinguished nil row is the
//! monoid's identity element, so the recorded nil-identity equations
//! (`parent ∘ top = top`, `top ∘ top = top`) hold on every node row and no
//! nil mount point survives, and the flattening schema map never silently
//! drops a child→parent link.
//!
//! Responsibilities: one `'static` [`PatternMeta`] matching the
//! disposition row, and `run` executing the model's already-declared
//! accepted equations through the kernel's own [`checks::check`] plus the
//! Δ surface's [`migrate::lost_data`] enumeration. Checks stay in the
//! existing P2 taxonomy and migrations stay in the migrate surface's own
//! `lost_row`/`lost_cell` findings; no new kind, witness, or heuristic is
//! added. A model with no declared equations is [`LensOutcome::Unknown`]
//! naming that absent surface, never an empty success.
//!
//! Rationale: the sealed kata evidence
//! (`ledger/katas/composite-null-object/`) records exactly the kernel's
//! `check` findings on the nil-identity fixtures — the floor control's
//! violation at the nil row plus its dangling child cell, the deep broken
//! parent cell, the deep node mounted on nil — and the flatten fault's
//! seven lost `parent` cells from `migrate`. Reusing those two surfaces
//! unchanged is the recorded law: the lens proves the nil element's
//! identity and totality through the equations the model already declares,
//! never the evaluated behaviour of `flatten`. `--values` is not read here
//! because this law is a rule-table-free equation surface; a malformed
//! spelling is still the CLI's refusal before the lens runs.

use crate::bounds::ScopeCaps;
use crate::pattern::{LawKind, LensFindings, LensInput, LensOutcome, PatternLens, PatternMeta};
use catherine_core::checks;
use catherine_core::instance::TypedInstance;
use catherine_core::migrate;

/// The disposition row's metadata, verbatim (`docs/ref/pattern-kata-dispositions.md`).
pub static NULL_OBJECT_META: PatternMeta = PatternMeta {
    id: "null-object",
    name: "Null Object",
    abstraction: "monoid identity element",
    law: LawKind::Identity,
    scope: "nil-identity equations (identity/totality for the nil element)",
};

/// The Null Object lens: the nil-identity equations over the declared node
/// rows, plus the Δ surface's lost-data enumeration.
///
/// Stateless and therefore a `static` value shared by the registry.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &NULL_OBJECT_META
    }

    fn run(&self, input: &LensInput<'_>, _caps: &ScopeCaps) -> LensOutcome {
        let model = input.model;
        if model.schema.equations.is_empty() {
            return LensOutcome::Unknown {
                scope: "no path equations declared; the nil-identity law has no surface to execute"
                    .to_string(),
            };
        }
        let typed = TypedInstance::build(&model.schema, &model.instance);
        LensOutcome::Findings(LensFindings {
            checks: checks::check(&model.schema, &typed),
            migrations: migrate::lost_data(model).unwrap_or_default(),
        })
    }
}
