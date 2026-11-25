//! Purpose: the Strategy pattern lens (beads `catherine-mx1.14`) — the
//! branch-selection abstraction read as rule regions, checked by the rule
//! lens's finite-domain laws over the declared selection domain.
//!
//! Responsibilities: one `'static [`PatternMeta`] matching the disposition
//! row for `strategy`, and `run` binding the fixture's declared finite
//! domain through [`bind_domain`] and reusing
//! [`catherine_analysis::rule_lens::rule_lens`]. It reports the exact scope
//! of a bounded non-answer and of every unevaluable surface, and fills only
//! the checks channel (no migration surface).
//!
//! Rationale: the kata ledger `ledger/katas/cor-strategy-rules/` records the
//! same fixtures for Chain of Responsibility and Strategy — the abstraction
//! differs only in how the regions are read (handler order versus branch
//! selection), so the law, witnesses, and findings are the rule lens's own,
//! unchanged. Parity rows therefore stay silent without a special case, and
//! no detection-rate claim is made: the lens is the rule-lens mapping and
//! nothing else.

use crate::pattern::{
    bind_domain, describe_rule_scope, LawKind, LensCall, LensFindings, LensOutcome, PatternLens,
    PatternMeta,
};
use crate::rule_lens::{rule_lens, RuleLensOutcome};

/// The disposition row's metadata, verbatim.
pub static STRATEGY_META: PatternMeta = PatternMeta {
    id: "strategy",
    name: "Strategy",
    abstraction: "branch selection as rule regions",
    law: LawKind::Order,
    scope: "rule-lens regions above the floor; no detection claim",
};

/// The Strategy lens: rule-table regions over the declared branch-selection
/// domain.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &STRATEGY_META
    }

    fn run(&self, call: LensCall<'_>) -> LensOutcome {
        let LensCall { input, caps } = call;
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
