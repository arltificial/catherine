//! Purpose: the `Adapter` pattern lens (beads `catherine-mx1.6`) — the
//! wrapper-law row read through its link-level delegation and field laws,
//! over the sealed evidence in `ledger/katas/wrapper-laws/`.
//!
//! Responsibilities: [`Lens`]'s metadata (id `adapter`, abstraction
//! "wrapper law preservation", law [`LawKind::Coverage`], the disposition
//! row's coverage-only scope) and its `run`: execute the kernel's own
//! [`check`] over the built model and report its P2 findings unchanged.
//! The `checks` channel is the whole result; the lens has no migration (Δ)
//! surface, so `migrations` stays empty.
//!
//! Rationale: the recorded kata states the delegation and field laws as
//! the wrapper fixture's declared path equations (`link ∘ target_field =
//! wrapper_field`), and the dropped-cell faults as unresolved arrow cells.
//! Both are exactly what `check` already executes — equation violations
//! with their witness and one `dangling` finding per unresolved cell — so
//! the lens owns no new heuristic, kind, or witness. It delegates to the
//! kernel op the ledger itself ran (`catherine check`), which is also why
//! the shared wrapper fixtures reproduce their recorded multisets through
//! this lens. Wrapper laws are coverage-only evidence: the kata verdict
//! records "executed, coverage-only" because a complete cross-product
//! sample collapses the unique set, so the scope field carries that
//! honesty boundary instead of a detection-rate claim.

use crate::bounds::ScopeCaps;
use crate::pattern::{LawKind, LensFindings, LensInput, LensOutcome, PatternLens, PatternMeta};
use catherine_core::checks::check;
use catherine_core::instance::TypedInstance;

/// The disposition row's metadata, verbatim.
pub static ADAPTER_META: PatternMeta = PatternMeta {
    id: "adapter",
    name: "Adapter",
    abstraction: "wrapper law preservation",
    law: LawKind::Coverage,
    scope: "link-level delegation/field laws; coverage-only",
};

/// The Adapter lens: the kernel's path-equation and totality check over
/// the wrapper fixture's declared laws.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &ADAPTER_META
    }

    fn run(&self, input: &LensInput<'_>, _caps: &ScopeCaps) -> LensOutcome {
        let typed = TypedInstance::build(&input.model.schema, &input.model.instance);
        LensOutcome::Findings(LensFindings {
            checks: check(&input.model.schema, &typed),
            migrations: Vec::new(),
        })
    }
}
