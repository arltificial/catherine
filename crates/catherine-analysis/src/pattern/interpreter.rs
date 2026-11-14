//! Purpose: the Interpreter pattern lens (beads `catherine-mx1.10`) — the
//! sum-type evidence read as recursive dispatch, checked by the rule lens's
//! finite-domain coverage law over the declared variant domain.
//!
//! Responsibilities: one `'static [`PatternMeta`] matching the disposition
//! row for `interpreter`, and `run` binding the fixture's declared finite
//! variant domain through [`bind_domain`] and reusing
//! [`catherine_analysis::rule_lens::rule_lens`]. It reports the exact scope
//! of a bounded non-answer and of every unevaluable surface, and fills only
//! the checks channel (no migration surface).
//!
//! Rationale: the kata ledger `ledger/katas/visitor-sum-types/` records the
//! same fixtures for Visitor and Interpreter — the abstraction differs only
//! in how the sum type is read, so the law, witnesses, and findings are the
//! rule lens's own, unchanged. No new kind, witness, or heuristic: the lens
//! is the rule-lens mapping and nothing else.

use crate::bounds::ScopeCaps;
use crate::pattern::{
    bind_domain, describe_rule_scope, LawKind, LensFindings, LensInput, LensOutcome, PatternLens,
    PatternMeta,
};
use crate::rule_lens::{rule_lens, RuleLensOutcome};

/// The disposition row's metadata, verbatim.
pub static INTERPRETER_META: PatternMeta = PatternMeta {
    id: "interpreter",
    name: "Interpreter",
    abstraction: "sum type / recursive dispatch",
    law: LawKind::Coverage,
    scope: "variant-domain coverage only",
};

/// The Interpreter lens: rule-table coverage over the declared variant
/// domain.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &INTERPRETER_META
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
