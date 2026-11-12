//! Purpose: the rule lens's semantic kernel (openspec change
//! `add-rule-lens`, phase B; design.md Decisions 2/3/5) — rules as
//! subobjects of a declared, finite input domain X, effective regions
//! E_i = R_i \ ⋃_{j<i} R_j under the declared resolution policy, the four
//! bug classes (gap, overlap, shadow, derivable), bounded enumeration
//! with `UNKNOWN { scope }`, and table equivalence by distinguishing
//! witness.
//!
//! Responsibilities: [`RuleDomain`] (the bounded value enumeration of X,
//! validated against the presentation's typed columns), [`evaluate`] (the
//! tiny decidable predicate language), [`analyze`] / [`analyze_with_bounds`]
//! (exact and budget-bounded analysis; a budget hit is an explicit
//! [`RuleOutcome::Unknown`], never a silent clean result), [`RuleAnalysis`]
//! (regions, effective regions, gap, overlaps, shadow, derivable — plus
//! [`RuleAnalysis::class_summary`], the four-class summary in the oracle's
//! order), and [`equivalence`] (same induced total action on X within the
//! declared domain, or the first differing input as witness).
//!
//! Rationale: the semantics are ported from the self-checking Python
//! oracle `prototype/rule_lens_sim.py` (design.md Decision 5) and are
//! deliberately kept free of findings-taxonomy types — the lens *finds*
//! classes and witnesses, the analysis crate maps them onto the existing
//! taxonomy (design.md Decision 1). Enumeration is plain mixed-radix over
//! the caller-supplied finite value lists, in declared column order with
//! the first column varying slowest (the oracle's `itertools.product`
//! order), so witnesses, gap examples, and budget expiry are
//! deterministic. Predicates are evaluated structurally; no solver and no
//! implicit precedence exist here.

mod analysis;
mod domain;
mod equivalence;

pub use analysis::{
    analyze, analyze_with_bounds, RuleAnalysis, RuleClass, RuleDerivation, RuleOutcome,
    RuleOverlap, RuleScope,
};
pub use domain::{evaluate, RuleDomain, RuleDomainError, RuleInput};
pub use equivalence::{equivalence, EquivalenceOutcome};
