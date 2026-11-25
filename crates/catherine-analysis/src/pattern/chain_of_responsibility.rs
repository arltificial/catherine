//! Purpose: the Chain of Responsibility pattern lens (beads
//! `catherine-mx1.13`) — the rule-table evidence in
//! `ledger/katas/cor-strategy-rules/`, read as declared handler order and
//! checked by the rule lens's gap / shadow / overlap / derivable laws over
//! the declared request domain.
//!
//! Responsibilities: one `'static` [`PatternMeta`] matching the
//! disposition row for `chain-of-responsibility`, and `run` binding the
//! fixture's declared finite domain through [`bind_domain`] and reusing
//! [`catherine_analysis::rule_lens::rule_lens`]. It reports the exact
//! scope of a bounded non-answer and of every unevaluable surface, and
//! fills only the checks channel (the pattern has no migration surface).
//!
//! Rationale: the kata records handler guards as rule predicates and chain
//! order as `PriorityOrder`, so a dead handler is an empty effective
//! region, an unclaimed request class is a gap, and a strategy branch is
//! a derivable refinement — exactly the rule lens's four classes. The lens
//! adds no heuristic of its own and no new kind or witness; parity rows
//! (faults the scripted dispatch sample already caught) stay at their
//! recorded multiset, including the recorded empty ones.

use crate::pattern::{
    bind_domain, describe_rule_scope, LawKind, LensCall, LensFindings, LensOutcome, PatternLens,
    PatternMeta,
};
use crate::rule_lens::{rule_lens, RuleLensOutcome};

/// The disposition row's metadata (the registry key is the kebab-case id
/// the expectation table and replay Meter use).
pub static COR_META: PatternMeta = PatternMeta {
    id: "chain-of-responsibility",
    name: "Chain of Responsibility",
    abstraction: "handler-order rule lens",
    law: LawKind::Order,
    scope: "rule-lens regions above the triviality floor",
};

/// The Chain of Responsibility lens: rule-table regions over the declared
/// handler order.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &COR_META
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
