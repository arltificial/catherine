//! Purpose: the rule lens as a taxonomy producer (openspec change
//! `add-rule-lens` phase B, task 6; design.md Decision 1) — maps the
//! kernel's structural analysis onto the existing finding kinds, with
//! bounded enumeration and `UNKNOWN { scope }` on budget expiry.
//!
//! Responsibilities: [`rule_lens`] runs `catherine-core`'s bounded rule
//! analysis under the shared [`ScopeCaps::max_candidates`] budget and
//! returns either the mapped findings or the exact [`RuleScope`] explored;
//! [`findings`] maps an already-computed analysis: gap → `underconstraint`
//! (`hypothesized`, `UncoveredInputs` with the class and dense example
//! indices), overlap under `DeclaredDisjoint` → `violation` (`executed`),
//! shadow → `redundant` (`executed`, the rule and the earlier rules
//! covering it), derivable → `redundant` (`executed`, the rule and its two
//! combining factors). Output is sorted by the taxonomy's total order.
//!
//! Witness encoding (no new finding variants exist — Decision 1): gap uses
//! the reserved `DomainQuestion::UncoveredInputs` arm; shadow/derivable use
//! `RedundantWitness` with rule names as one-element/two-element paths.
//! An overlap has no dedicated witness shape, so it reuses
//! `EquationWitness` as the declared-disjointness claim: `equation` is the
//! lower-indexed colliding rule, `row` is the witness input's dense
//! enumeration index, and `lhs`/`rhs` are the two colliding rule indices.
//! One violation is emitted per (pair, shared input), so the witness set is
//! exact rather than a summary.
//!
//! Rationale: the kernel never imports the taxonomy (structural facts stay
//! inspectable and testable on their own); the analysis layer owns the
//! label/witness mapping and the shared budget, exactly as it does for the
//! P4 producers. Overlap with no declared policy is a diagnostic in the
//! analysis (`RuleClass::Overlap`) but maps to no finding — Decision 1
//! pins overlap only under `DeclaredDisjoint`, and Decision 3 forbids
//! implying precedence that was never declared.

use crate::bounds::{redundant_finding, uncovered_inputs_finding, ScopeCaps};
use catherine_core::findings::{EpistemicLabel, EquationWitness, Finding, FindingKind};
use catherine_core::model::{Ident, Literal, Path, Policy, RowId, RulePresentation};
use catherine_core::rule_lens::{
    analyze_with_bounds, RuleAnalysis, RuleDomain, RuleOutcome, RuleOverlap, RuleScope,
};

/// The rule lens's bounded result: mapped findings, or the exact scope of
/// a budget-bounded non-answer. Never a partial finding set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleLensOutcome {
    /// Exhaustion completed within the budget; the mapped findings.
    Findings(Vec<Finding>),
    /// The candidate budget expired: `UNKNOWN { scope }`.
    Unknown {
        /// The exact scope explored.
        scope: RuleScope,
    },
}

/// Run the bounded rule lens under the shared scope caps and map the
/// result onto the findings taxonomy.
pub fn rule_lens(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    caps: &ScopeCaps,
) -> RuleLensOutcome {
    match analyze_with_bounds(presentation, domain, caps.max_candidates) {
        RuleOutcome::Analyzed(analysis) => {
            RuleLensOutcome::Findings(findings(presentation, domain, &analysis))
        }
        RuleOutcome::Unknown { scope } => RuleLensOutcome::Unknown { scope },
    }
}

/// Map a completed analysis onto the existing finding kinds (Decision 1),
/// sorted by the taxonomy's total order.
pub fn findings(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    analysis: &RuleAnalysis,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(gap_finding(domain, analysis));
    if presentation.policy == Some(Policy::DeclaredDisjoint) {
        findings.extend(overlap_findings(analysis));
    }
    findings.extend(shadow_findings(presentation, analysis));
    findings.extend(derivable_findings(presentation, analysis));
    findings.sort();
    findings
}

/// The gap finding: `underconstraint`/`hypothesized` naming the first
/// uncovered input as the class and every uncovered input as an example.
fn gap_finding(domain: &RuleDomain, analysis: &RuleAnalysis) -> Option<Finding> {
    let first = analysis.gap.first().copied()?;
    let class = render_input(domain, &analysis.inputs[first]);
    let examples = analysis.gap.iter().map(|&input| RowId(input)).collect();
    Some(uncovered_inputs_finding(class, examples))
}

/// One violation per (colliding pair, shared input). See the module docs
/// for the `EquationWitness` encoding.
fn overlap_findings(analysis: &RuleAnalysis) -> Vec<Finding> {
    let mut findings = Vec::new();
    for overlap in &analysis.overlaps {
        for &input in &overlap.shared {
            findings.push(overlap_finding(overlap, input));
        }
    }
    findings
}

fn overlap_finding(overlap: &RuleOverlap, input: usize) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(EquationWitness {
            equation: overlap.left,
            row: RowId(input),
            lhs: RowId(overlap.left),
            rhs: RowId(overlap.right),
        }),
    }
}

/// One redundant finding per dead rule: the rule reduces to the earlier
/// rules whose regions meet it.
fn shadow_findings(presentation: &RulePresentation, analysis: &RuleAnalysis) -> Vec<Finding> {
    let Some(shadow) = &analysis.shadow else {
        return Vec::new();
    };
    shadow
        .iter()
        .map(|&rule| {
            redundant_finding(
                Path(vec![rule_name(presentation, rule)]),
                Path(covering_rules(presentation, analysis, rule)),
            )
        })
        .collect()
}

/// The earlier rules whose regions intersect a shadowed rule's region —
/// together they cover it by the definition of an empty effective region.
fn covering_rules(
    presentation: &RulePresentation,
    analysis: &RuleAnalysis,
    rule: usize,
) -> Vec<Ident> {
    (0..rule)
        .filter(|&earlier| !analysis.regions[earlier].is_disjoint(&analysis.regions[rule]))
        .map(|earlier| rule_name(presentation, earlier))
        .collect()
}

/// One redundant finding per derivable rule: the rule and its two
/// combining factors, ascending.
fn derivable_findings(presentation: &RulePresentation, analysis: &RuleAnalysis) -> Vec<Finding> {
    analysis
        .derivable
        .iter()
        .map(|derivation| {
            let reducing = derivation
                .factors
                .iter()
                .map(|&factor| rule_name(presentation, factor))
                .collect();
            redundant_finding(
                Path(vec![rule_name(presentation, derivation.rule)]),
                Path(reducing),
            )
        })
        .collect()
}

fn rule_name(presentation: &RulePresentation, index: usize) -> Ident {
    presentation.rules[index].name.clone()
}

/// Render an input as `column = value` pairs for the gap class name.
fn render_input(domain: &RuleDomain, input: &[Literal]) -> String {
    let parts: Vec<String> = domain
        .columns()
        .iter()
        .zip(input)
        .map(|(column, value)| format!("{} = {}", column.0, render_literal(value)))
        .collect();
    if parts.is_empty() {
        "the empty input".to_string()
    } else {
        parts.join(", ")
    }
}

fn render_literal(value: &Literal) -> String {
    match value {
        Literal::Int(value) => value.to_string(),
        Literal::Bool(value) => value.to_string(),
        Literal::String(value) => format!("{value:?}"),
    }
}
