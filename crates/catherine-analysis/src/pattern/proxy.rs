//! Purpose: the Proxy pattern lens (beads `catherine-mx1.8`) — the
//! wrapper-law evidence read through the identity-preservation equation,
//! over the sealed fixtures in `ledger/katas/wrapper-laws/`.
//!
//! Responsibilities: [`Lens`]'s metadata (id `proxy`, abstraction "wrapper
//! law preservation", law [`LawKind::Coverage`], the disposition row's
//! coverage-only scope) and its `run`: execute the kernel's declared
//! path-equation and totality walk ([`check`]) over the typed instance and
//! report its P2 checks unchanged. The equation/arrow surface is the
//! lens's only input, so a document with no declared arrow or equation is
//! [`LensOutcome::Unknown`] naming that surface, as is a `--values`
//! declaration the equation law cannot consume — never an empty success.
//! The lens fills only the `checks` channel; it has no migration (Δ)
//! surface.
//!
//! Rationale: the kata states the proxy laws as equations the kernel
//! already executes (`proxies ∘ key = proxy_key`, `proxies ∘ result =
//! proxy_result`, plus totality over every declared arrow), and the ledger's
//! recorded witnesses are exactly `check`'s violation/dangling witnesses
//! (`violation:eq4@row1` for the rewired identity, the adapter/decorator
//! dangling cells for totality). The lens therefore owns no new kind,
//! witness, heuristic, or detection-rate claim — it is the core walk mapped
//! into the seam's typed outcome. Coverage-only is the disposition row's
//! honesty boundary: a complete cross-product sample collapses the unique
//! set, so the scope names coverage rather than a defect class.

use crate::bounds::ScopeCaps;
use crate::pattern::{LawKind, LensFindings, LensInput, LensOutcome, PatternLens, PatternMeta};
use catherine_core::checks::check;
use catherine_core::instance::TypedInstance;

/// The disposition row's metadata, verbatim (design Decision 2).
pub static PROXY_META: PatternMeta = PatternMeta {
    id: "proxy",
    name: "Proxy",
    abstraction: "wrapper law preservation",
    law: LawKind::Coverage,
    scope: "identity preservation through delegation; coverage-only",
};

/// The Proxy lens: the kernel's executed equation/totality walk.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &PROXY_META
    }

    fn run(&self, input: &LensInput<'_>, _caps: &ScopeCaps) -> LensOutcome {
        if !input.values.is_empty() {
            return LensOutcome::Unknown {
                scope: "`--values` declares a rule domain, but the proxy law is an \
                        equation/totality surface; nothing evaluated"
                    .to_string(),
            };
        }
        if input.model.schema.arrows.is_empty() && input.model.schema.equations.is_empty() {
            return LensOutcome::Unknown {
                scope: "no declared arrow or equation surface; nothing evaluated".to_string(),
            };
        }
        let typed = TypedInstance::build(&input.model.schema, &input.model.instance);
        LensOutcome::Findings(LensFindings {
            checks: check(&input.model.schema, &typed),
            migrations: Vec::new(),
        })
    }
}
