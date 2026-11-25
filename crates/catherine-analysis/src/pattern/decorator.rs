//! Purpose: the `Decorator` pattern lens (beads `catherine-mx1.7`) — the
//! wrapper-law row read through the stacking transparency equation, over
//! the sealed evidence in `ledger/katas/wrapper-laws/`.
//!
//! Responsibilities: [`Lens`]'s metadata (id `decorator`, abstraction
//! "wrapper law preservation", law [`LawKind::Order`], the disposition
//! row's coverage-only scope) and its `run`: execute the model's declared
//! path equations through the kernel's own `check` surface (the same
//! operation the ledger's `check` command recorded) and report those P2
//! findings unchanged. A `--values` list is refused as `Unknown` because
//! this lens has no rule-table surface; the lens never ignores the flag.
//!
//! Rationale: the recorded kata states the stacking law as one of the
//! wrapper fixture's declared path equations — the fixture carries no
//! `rule_presentation` — so the lens reuses `TypedInstance::build` and
//! `checks::check` rather than the rule lens (design Decision 4's
//! "equation lenses reuse the kernel operations their kata commands
//! recorded"). It owns no new heuristic, kind, or witness. Wrapper laws
//! are coverage-only evidence: the kata verdict records "executed,
//! coverage-only" because a complete cross-product sample collapses the
//! unique set, so the scope field carries that honesty boundary instead of
//! a detection-rate claim. The lens fills only the `checks` channel; it
//! has no migration (Δ) surface.

use crate::pattern::{LawKind, LensCall, LensFindings, LensOutcome, PatternLens, PatternMeta};
use catherine_core::checks;
use catherine_core::instance::TypedInstance;

/// The disposition row's metadata, verbatim (design Decision 2).
pub static DECORATOR_META: PatternMeta = PatternMeta {
    id: "decorator",
    name: "Decorator",
    abstraction: "wrapper law preservation",
    law: LawKind::Order,
    scope: "stacking transparency equation; coverage-only",
};

/// The Decorator lens: the kernel's equation checks over the wrapper
/// fixtures.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &DECORATOR_META
    }

    fn run(&self, call: LensCall<'_>) -> LensOutcome {
        let LensCall { input, caps: _caps } = call;
        if let Some(scope) = refuse_values(input.values) {
            return LensOutcome::Unknown { scope };
        }
        let typed = TypedInstance::build(&input.model.schema, &input.model.instance);
        let checks = checks::check(&input.model.schema, &typed);
        LensOutcome::Findings(LensFindings {
            checks,
            migrations: Vec::new(),
        })
    }
}

/// This lens reads the model's declared equations, not a rule table: a
/// supplied `--values` cannot bind anywhere, so it is an unevaluable
/// request naming the mismatch rather than a silently ignored flag.
fn refuse_values(values: &[String]) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    Some(format!(
        "`--values {}` supplied, but the decorator lens has no rule-table surface; nothing evaluated",
        values.join(" ")
    ))
}
