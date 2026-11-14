//! Purpose: the `visitor` pattern lens (beads `catherine-mx1.9`) — the
//! Visitor-as-sum-type row's Coverage law: every declared variant of the
//! node type reaches a named handler action, and each handler class is
//! named exactly once (no shadowed or derivable handler).
//!
//! Responsibilities: one `'static` [`PatternMeta`] matching the
//! disposition row, and `run` binding the fixture's declared finite
//! variant domain through [`crate::pattern::bind_domain`] and reusing
//! [`crate::rule_lens::rule_lens`]. Findings stay in the existing P2
//! taxonomy (checks channel only — the lens has no Δ surface, so
//! `migrations` is always empty); a bounded non-answer or an unevaluable
//! surface is [`LensOutcome::Unknown`] naming the exact scope, never an
//! empty success.
//!
//! Rationale: the sealed kata evidence (`ledger/katas/visitor-sum-types/`)
//! records the four rule-table classes — uncovered variant
//! (`underconstraint`), shared variant under `DeclaredDisjoint`
//! (`violation`), shadowed handler under `PriorityOrder`, and derivable
//! handler (`redundant`) — as exactly what the rule lens already proves
//! over the declared variant domain. Reusing `rule_lens` unchanged keeps
//! the lens honest: it proves coverage and handler-class shape, never the
//! visitor method bodies' return values, and it adds no heuristic beyond
//! the recorded law.

use crate::bounds::ScopeCaps;
use crate::pattern::{
    bind_domain, describe_rule_scope, LawKind, LensFindings, LensInput, LensOutcome, PatternLens,
    PatternMeta,
};
use crate::rule_lens::{rule_lens, RuleLensOutcome};

/// The disposition row's metadata, verbatim (Visitor: sum type /
/// exhaustiveness, variant coverage + handler classes).
pub static VISITOR_META: PatternMeta = PatternMeta {
    id: "visitor",
    name: "Visitor",
    abstraction: "sum type / exhaustiveness",
    law: LawKind::Coverage,
    scope: "variant coverage + handler classes; no evaluation semantics",
};

/// The Visitor lens: rule-table coverage over the declared variant domain.
///
/// Stateless and therefore a `static` value shared by the registry.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &VISITOR_META
    }

    fn run(&self, input: &LensInput<'_>, caps: &ScopeCaps) -> LensOutcome {
        let Some(presentation) = input.model.rule_presentation.as_ref() else {
            return LensOutcome::Unknown {
                scope: "no rule_presentation section; nothing evaluated".to_string(),
            };
        };
        let domain = match bind_domain(presentation, input.values) {
            Ok(domain) => domain,
            Err(scope) => return LensOutcome::Unknown { scope },
        };
        match rule_lens(presentation, &domain, caps) {
            RuleLensOutcome::Findings(checks) => LensOutcome::Findings(LensFindings {
                checks,
                migrations: Vec::new(),
            }),
            RuleLensOutcome::Unknown { scope } => LensOutcome::Unknown {
                scope: describe_rule_scope(&scope),
            },
        }
    }
}
