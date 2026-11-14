//! Purpose: the seam's TEST-ONLY reference lens (beads
//! `catherine-mx1.2`) — a Visitor variant-coverage lens that proves the
//! `PatternLens` contract before any production lens exists.
//!
//! Responsibilities: one `'static PatternMeta` matching the disposition
//! row for `visitor`, and `run` binding the fixture's declared finite
//! domain through [`catherine_analysis::pattern::bind_domain`] and the
//! existing `catherine_analysis::rule_lens::rule_lens`. It reports the
//! exact scope of a bounded non-answer and of every unevaluable surface.
//!
//! Rationale: the shipped `visitor` lens belongs to `catherine-mx1.9`;
//! this implementation lives under `tests/` so the foundation ships zero
//! production lenses while the replay harness still exercises the seam
//! end to end. It is included by `tests/pattern_replay.rs` via `#[path]`
//! (test scope only), never by `src/`. No new kind, witness, or heuristic:
//! the lens is the rule-lens mapping, unchanged, and it fills only the
//! checks channel (its `migrations` list stays empty — it has no Δ surface).

use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::pattern::LawKind;
use catherine_analysis::pattern::{
    bind_domain, describe_rule_scope, LensFindings, LensInput, LensOutcome, PatternLens,
    PatternMeta,
};
use catherine_analysis::rule_lens::{rule_lens, RuleLensOutcome};

/// The disposition row's metadata, verbatim (design Decision 2).
pub static VISITOR_META: PatternMeta = PatternMeta {
    id: "visitor",
    name: "Visitor",
    abstraction: "sum type / exhaustiveness",
    law: LawKind::Coverage,
    scope: "variant coverage + handler classes; no evaluation semantics",
};

/// The test-only Visitor lens: rule-table coverage over the declared
/// variant domain.
pub struct VisitorLens;

/// The one reference-lens value.
pub static VISITOR: VisitorLens = VisitorLens;

/// The test-local registry slice the replay harness composes with the
/// production [`catherine_analysis::pattern::registry`]. Test scope only:
/// production `registry()` stays empty until the orchestrator wires the
/// shipped lenses.
pub static REFERENCE_LENSES: &[&'static dyn PatternLens] = &[&VISITOR];

impl PatternLens for VisitorLens {
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

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::model::Model;

    #[test]
    fn the_reference_lens_metadata_is_the_visitor_disposition_row() {
        let meta = VISITOR.meta();
        assert_eq!(meta.id, "visitor");
        assert_eq!(meta.name, "Visitor");
        assert_eq!(meta.law, LawKind::Coverage);
        assert_eq!(meta.law.as_str(), "coverage");
        assert!(meta.scope.contains("variant coverage"));
        assert!(!meta.abstraction.is_empty());
    }

    #[test]
    fn the_reference_slice_names_the_visitor_lens() {
        assert_eq!(REFERENCE_LENSES.len(), 1);
        assert_eq!(REFERENCE_LENSES[0].meta().id, "visitor");
        assert_eq!(REFERENCE_LENSES[0].meta().law, LawKind::Coverage);
    }

    #[test]
    fn a_model_without_a_rule_presentation_is_unknown_not_clean() {
        let model = Model::empty();
        let input = LensInput {
            model: &model,
            values: &[],
        };
        match VISITOR.run(&input, &ScopeCaps::default()) {
            LensOutcome::Unknown { scope } => assert!(scope.contains("rule_presentation")),
            LensOutcome::Findings(findings) => {
                panic!("an absent surface must not be clean: {findings:?}")
            }
        }
    }

    #[test]
    fn unbound_values_are_an_unknown_scope() {
        let path = format!(
            "{}/../../fixtures/katas/visitor-sum-types/visitor.ct.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let text = std::fs::read_to_string(path).expect("fixture exists");
        let model = catherine_core::parse::parse(&text).expect("fixture parses");
        let values = vec!["nope=1".to_string()];
        let input = LensInput {
            model: &model,
            values: &values,
        };
        match VISITOR.run(&input, &ScopeCaps::default()) {
            LensOutcome::Unknown { scope } => assert!(scope.contains("does not declare")),
            LensOutcome::Findings(_) => panic!("an unbound domain must not be evaluated"),
        }
    }
}
