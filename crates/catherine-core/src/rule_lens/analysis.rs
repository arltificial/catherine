//! Purpose: exact and bounded rule-table analysis (openspec change
//! `add-rule-lens` phase B; DESIGN §4.4, design.md Decisions 1/3) — the
//! effective regions and the four bug classes over a declared finite
//! domain, with `UNKNOWN { scope }` on budget expiry.
//!
//! Responsibilities: [`analyze`] (exhaustive over the declared domain —
//! the sim-parity path), [`analyze_with_bounds`] (the same analysis under
//! a candidate budget; expiry is [`RuleOutcome::Unknown`] with the exact
//! column sizes and candidate count explored, mirroring the P4
//! bounded-search discipline), and the structural result types —
//! regions R_i, effective regions E_i (under `PriorityOrder`/`FirstMatch`
//! by rule order; `DeclaredDisjoint` keeps regions intact; no policy
//! means no effective regions), gap, pair overlaps, shadowed rules, and
//! derivable rules — plus [`RuleAnalysis::class_summary`], the four-class
//! sequence in the oracle's order.
//!
//! Rationale: the kernel reports facts, not taxonomy labels — the
//! analysis crate maps these structures onto the existing finding kinds
//! (design.md Decision 1), so this module never imports the findings
//! taxonomy. Sets are dense input indices into `RuleAnalysis::inputs`
//! (deterministic and cheap; witnesses need indices anyway), and every
//! iteration is ascending, so the result is reproducible by construction.
//! A zero budget is `UNKNOWN` with the empty scope (P4's zero-size rule),
//! never an empty-but-clean analysis; an empty declared domain is a
//! completed exhaustion and yields the honest empty result.

use super::domain::{evaluate, RuleDomain, RuleInput};
use crate::model::{Ident, Policy, RulePresentation};
use std::collections::{BTreeMap, BTreeSet};

/// One pair of rules whose regions intersect, with the shared inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleOverlap {
    /// The earlier rule's index in the presentation's priority chain.
    pub left: usize,
    /// The later rule's index.
    pub right: usize,
    /// The intersecting inputs (dense indices into `RuleAnalysis::inputs`).
    pub shared: BTreeSet<usize>,
}

/// A rule whose region is exactly the intersection of two other rules' —
/// the boolean-combination derivability of DESIGN §4.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDerivation {
    /// The derivable rule's index.
    pub rule: usize,
    /// The two combining rule indices, ascending.
    pub factors: [usize; 2],
}

/// The four rule-table bug classes, in the oracle's finding order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleClass {
    /// X is not covered by the rule regions (partial action assignment).
    Gap,
    /// Two regions intersect (always diagnosed; a finding only under
    /// `DeclaredDisjoint`, where precedence is not declared).
    Overlap,
    /// A rule's effective region is empty: dead under the policy.
    Shadow,
    /// A rule's region is a boolean combination of two others.
    Derivable,
}

/// The structural analysis of one rule table over one declared domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleAnalysis {
    /// The enumerated inputs, in declared order.
    pub inputs: Vec<RuleInput>,
    /// R_i: each rule's region as dense input indices.
    pub regions: Vec<BTreeSet<usize>>,
    /// E_i under the declared policy; `None` when no policy is declared
    /// (Decision 3: no implicit precedence, so no effective regions).
    pub effective: Option<Vec<BTreeSet<usize>>>,
    /// X \ ⋃ R_i — the uncovered inputs.
    pub gap: BTreeSet<usize>,
    /// Pairwise region intersections (empty under a priority policy,
    /// where order resolves them by construction).
    pub overlaps: Vec<RuleOverlap>,
    /// Rules with E_i = ∅ (ascending); `None` when no policy is declared.
    pub shadow: Option<Vec<usize>>,
    /// Rules derivable as the intersection of two others.
    pub derivable: Vec<RuleDerivation>,
}

/// The exact shape a budget exhausted: every declared column size and the
/// candidate count explored (the rule lens's `UNKNOWN { scope }`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleScope {
    /// Declared value counts per column; empty when nothing was explored.
    pub columns: BTreeMap<Ident, usize>,
    /// How many candidate inputs the enumeration materialized.
    pub candidates_explored: usize,
}

/// The bounded outcome: a completed analysis, or the exact scope of a
/// non-answer. Never a partial analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleOutcome {
    /// Exhaustion completed within the budget.
    Analyzed(RuleAnalysis),
    /// The budget expired: `UNKNOWN { scope }`.
    Unknown {
        /// The exact scope explored.
        scope: RuleScope,
    },
}

impl RuleAnalysis {
    /// The four-class summary in the oracle's order: gap once, overlap
    /// once, shadow per dead rule, derivable per derivation. This is the
    /// semantic content the analysis crate maps to finding kinds.
    pub fn class_summary(&self) -> Vec<RuleClass> {
        let mut classes = Vec::new();
        if !self.gap.is_empty() {
            classes.push(RuleClass::Gap);
        }
        if !self.overlaps.is_empty() {
            classes.push(RuleClass::Overlap);
        }
        if let Some(shadow) = &self.shadow {
            classes.extend(shadow.iter().map(|_| RuleClass::Shadow));
        }
        classes.extend(self.derivable.iter().map(|_| RuleClass::Derivable));
        classes
    }
}

/// Analyze the whole declared domain exhaustively.
pub fn analyze(presentation: &RulePresentation, domain: &RuleDomain) -> RuleAnalysis {
    analyze_inputs(presentation, domain, domain.inputs())
}

/// Analyze under a candidate budget. A budget that expires before
/// exhaustion is `UNKNOWN { scope }` — never a contradiction, never a
/// clean result. A zero budget is `UNKNOWN` with the empty scope.
pub fn analyze_with_bounds(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    max_candidates: usize,
) -> RuleOutcome {
    if max_candidates == 0 {
        return RuleOutcome::Unknown {
            scope: RuleScope {
                columns: BTreeMap::new(),
                candidates_explored: 0,
            },
        };
    }
    match domain.enumerate_within(max_candidates) {
        Ok(inputs) => RuleOutcome::Analyzed(analyze_inputs(presentation, domain, inputs)),
        Err(explored) => RuleOutcome::Unknown {
            scope: RuleScope {
                columns: domain.column_sizes(),
                candidates_explored: explored,
            },
        },
    }
}

/// Analyze a materialized input list (the enumeration completed).
fn analyze_inputs(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    inputs: Vec<RuleInput>,
) -> RuleAnalysis {
    let regions = regions(presentation, domain, &inputs);
    let gap = gap(&regions, inputs.len());
    let effective = effective_regions(presentation.policy, &regions);
    let overlaps = overlaps(presentation.policy, &regions);
    let shadow = shadow_rules(&effective);
    let derivable = derivations(&regions);
    RuleAnalysis {
        inputs,
        regions,
        effective,
        gap,
        overlaps,
        shadow,
        derivable,
    }
}

/// R_i for every rule: the inputs its predicate accepts.
fn regions(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    inputs: &[RuleInput],
) -> Vec<BTreeSet<usize>> {
    presentation
        .rules
        .iter()
        .map(|rule| {
            inputs
                .iter()
                .enumerate()
                .filter(|(_, input)| evaluate(&rule.predicate, input, domain))
                .map(|(index, _)| index)
                .collect()
        })
        .collect()
}

/// X \ ⋃ R_i.
fn gap(regions: &[BTreeSet<usize>], universe: usize) -> BTreeSet<usize> {
    let mut covered = BTreeSet::new();
    for region in regions {
        covered.extend(region.iter().copied());
    }
    (0..universe)
        .filter(|input| !covered.contains(input))
        .collect()
}

/// E_i = R_i \ ⋃_{j<i} R_j under a priority policy; the untouched
/// regions under `DeclaredDisjoint`; `None` when no policy is declared.
fn effective_regions(
    policy: Option<Policy>,
    regions: &[BTreeSet<usize>],
) -> Option<Vec<BTreeSet<usize>>> {
    match policy? {
        Policy::PriorityOrder | Policy::FirstMatch => Some(priority_effective(regions)),
        Policy::DeclaredDisjoint => Some(regions.to_vec()),
    }
}

/// The priority-chain computation: rule order is the only precedence.
fn priority_effective(regions: &[BTreeSet<usize>]) -> Vec<BTreeSet<usize>> {
    let mut covered = BTreeSet::new();
    let mut effective = Vec::with_capacity(regions.len());
    for region in regions {
        effective.push(region.difference(&covered).copied().collect());
        covered.extend(region.iter().copied());
    }
    effective
}

/// Pairwise intersections. Under a priority policy order resolves them by
/// construction, so none are reported; otherwise (declared-disjoint or no
/// policy) every intersecting pair is.
fn overlaps(policy: Option<Policy>, regions: &[BTreeSet<usize>]) -> Vec<RuleOverlap> {
    if matches!(policy, Some(Policy::PriorityOrder | Policy::FirstMatch)) {
        return Vec::new();
    }
    (0..regions.len())
        .flat_map(|left| ((left + 1)..regions.len()).map(move |right| (left, right)))
        .filter_map(|(left, right)| overlap_between(left, right, regions))
        .collect()
}

/// One ascending pair's intersection, if it has any shared input.
fn overlap_between(left: usize, right: usize, regions: &[BTreeSet<usize>]) -> Option<RuleOverlap> {
    let shared = regions[left]
        .intersection(&regions[right])
        .copied()
        .collect::<BTreeSet<usize>>();
    if shared.is_empty() {
        return None;
    }
    Some(RuleOverlap {
        left,
        right,
        shared,
    })
}

/// Rules whose effective region is empty, ascending; `None` keeps the
/// no-policy case absent rather than guessing.
fn shadow_rules(effective: &Option<Vec<BTreeSet<usize>>>) -> Option<Vec<usize>> {
    effective.as_ref().map(|regions| {
        regions
            .iter()
            .enumerate()
            .filter(|(_, region)| region.is_empty())
            .map(|(index, _)| index)
            .collect()
    })
}

/// R_k = R_i ∩ R_j for every ordered pair i < j (both ≠ k), rule-major
/// then pair-lexicographic — the oracle's enumeration.
fn derivations(regions: &[BTreeSet<usize>]) -> Vec<RuleDerivation> {
    let mut derivations = Vec::new();
    for rule in 0..regions.len() {
        derivations.extend(derivations_for(rule, regions));
    }
    derivations
}

/// The combining pairs for one derivable rule.
fn derivations_for(rule: usize, regions: &[BTreeSet<usize>]) -> Vec<RuleDerivation> {
    let others: Vec<usize> = (0..regions.len()).filter(|&index| index != rule).collect();
    others
        .iter()
        .enumerate()
        .flat_map(|(position, &left)| {
            others[position + 1..]
                .iter()
                .map(move |&right| (left, right))
        })
        .filter(|&(left, right)| combines(regions, rule, left, right))
        .map(|(left, right)| RuleDerivation {
            rule,
            factors: [left, right],
        })
        .collect()
}

/// Whether R_left ∩ R_right equals R_rule.
fn combines(regions: &[BTreeSet<usize>], rule: usize, left: usize, right: usize) -> bool {
    regions[left]
        .intersection(&regions[right])
        .copied()
        .eq(regions[rule].iter().copied())
}
