//! Purpose: rule-table equivalence by distinguishing witness (openspec
//! change `add-rule-lens` phase B; DESIGN §4.4 property 1, spec
//! "Rule-table equivalence is proven by witness") — two tables are
//! equivalent iff they induce the same total action on the declared
//! domain X.
//!
//! Responsibilities: [`equivalence`] computes each table's induced action
//! per input (no match is a gap, multiple matches are ambiguous unless a
//! priority policy selects the first, a single match is its action) and
//! returns [`EquivalenceOutcome::EquivalentInScope`] with the bound, or
//! [`EquivalenceOutcome::Distinguishing`] with the first differing input
//! in enumeration order. Simplification claims are proven within the
//! declared domain, never asserted beyond it.
//!
//! Rationale: the semantics are the oracle's `induced_action` /
//! `equivalence`, verbatim — policy `PriorityOrder`/`FirstMatch` selects
//! the first matching rule, `DeclaredDisjoint` and no policy leave an
//! overlap ambiguous, and a gap is an action value distinct from every
//! declared action. Witnesses are the oracle's `fmt_input` inputs, so a
//! sim/kernel divergence is a failing parity test rather than a silent
//! semantic drift.

use super::domain::{evaluate, RuleDomain, RuleInput};
use crate::model::{Literal, Policy, Rule, RulePresentation};

/// The equivalence verdict within the declared domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquivalenceOutcome {
    /// Every input induced the same action; the bound is the number of
    /// inputs checked.
    EquivalentInScope {
        /// The number of inputs the claim holds over.
        bound: usize,
    },
    /// The first input (in declared enumeration order) where the induced
    /// actions differ.
    Distinguishing {
        /// The differing input.
        input: RuleInput,
        /// Its dense index in the domain enumeration.
        index: usize,
        /// The number of inputs in the declared domain.
        bound: usize,
    },
}

/// The action a table induces on one input: a declared action, a gap
/// (no rule matched), or ambiguity (several matched and the policy did
/// not select one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Induced<'a> {
    Gap,
    Action(&'a str),
    Ambiguous,
}

/// Compare two tables over the declared domain; the first differing
/// input is the witness. The domain must cover both tables' columns.
pub fn equivalence(
    left: &RulePresentation,
    right: &RulePresentation,
    domain: &RuleDomain,
) -> EquivalenceOutcome {
    let inputs = domain.inputs();
    for (index, input) in inputs.iter().enumerate() {
        if induced(left, domain, input) != induced(right, domain, input) {
            return EquivalenceOutcome::Distinguishing {
                input: input.clone(),
                index,
                bound: inputs.len(),
            };
        }
    }
    EquivalenceOutcome::EquivalentInScope {
        bound: inputs.len(),
    }
}

/// The oracle's `induced_action` for one input.
fn induced<'a>(
    presentation: &'a RulePresentation,
    domain: &RuleDomain,
    input: &[Literal],
) -> Induced<'a> {
    let matching: Vec<&'a Rule> = presentation
        .rules
        .iter()
        .filter(|rule| evaluate(&rule.predicate, input, domain))
        .collect();
    let Some(first) = matching.first().copied() else {
        return Induced::Gap;
    };
    if matching.len() == 1 || priority(presentation.policy) {
        Induced::Action(&first.action)
    } else {
        Induced::Ambiguous
    }
}

/// Whether the policy resolves intersections by rule order.
fn priority(policy: Option<Policy>) -> bool {
    matches!(policy, Some(Policy::PriorityOrder | Policy::FirstMatch))
}
