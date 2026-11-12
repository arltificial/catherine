//! Purpose: the rule-lens payload (`catherine rules`, task 8) — the
//! envelope's `data` and the one-line-per-finding human rendering.
//!
//! Responsibilities: build the one rules-envelope path (the Decision-1
//! findings verbatim under `data.findings`, the outcome state, and the
//! exact scope of a bounded non-answer) and render each finding on one
//! line as kind + witness summary, resolving rule indices to their
//! declared names.
//!
//! Rationale: the kernel's `Finding` serialization is the contract — the
//! CLI prints it, never re-serializes it (the `findings_envelope`
//! precedent). The human line is deliberately one line per finding (the
//! rules surface is a table review, not the §5 equation transcript), and
//! a bounded non-answer keeps its own honest line: `UNKNOWN` is never
//! rendered as clean.

use super::{path_string, CLI_VERSION};
use catherine_core::findings::{DomainQuestion, Finding, FindingKind};
use catherine_core::model::RulePresentation;
use catherine_core::rule_lens::RuleScope;
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use serde::Serialize;
use std::collections::BTreeMap;

/// The exact scope a budget-bounded non-answer explored: declared value
/// counts per column and the candidate count materialized.
#[derive(Serialize)]
pub struct RuleScopeView {
    pub columns: BTreeMap<String, usize>,
    pub candidates_explored: usize,
}

impl RuleScopeView {
    pub fn of(scope: &RuleScope) -> Self {
        Self {
            columns: scope
                .columns
                .iter()
                .map(|(column, &size)| (column.0.clone(), size))
                .collect(),
            candidates_explored: scope.candidates_explored,
        }
    }
}

/// The rules payload: the epistemic outcome (`"analyzed"` or
/// `"unknown"`), the explored scope only for a non-answer, and the
/// kernel's findings verbatim.
#[derive(Serialize)]
pub struct RulesData<'a> {
    pub outcome: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<RuleScopeView>,
    pub findings: &'a [Finding],
}

/// The one rules-envelope path: kind `Check` (the payload carries
/// findings), empty hints, the caller's warnings alongside the data.
pub fn rules_envelope<'a>(
    findings: &'a [Finding],
    scope: Option<&RuleScope>,
    warnings: Vec<Warning>,
) -> Envelope<RulesData<'a>> {
    let (outcome, scope) = match scope {
        Some(scope) => ("unknown", Some(RuleScopeView::of(scope))),
        None => ("analyzed", None),
    };
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        RulesData {
            outcome,
            scope,
            findings,
        },
        warnings,
        Vec::new(),
    )
}

/// The human lines: one line per finding (kind + witness summary), or
/// the honest clean line when the analysis completed with no findings.
pub fn render_rules(findings: &[Finding], presentation: &RulePresentation) -> Vec<String> {
    let mut lines: Vec<String> = findings
        .iter()
        .map(|finding| finding_line(finding, presentation))
        .collect();
    if lines.is_empty() {
        lines.push("rules: clean — no findings over the declared domain".to_string());
    }
    lines
}

/// One finding as a single line: the kind, the witness facts, and the
/// non-default epistemic label.
fn finding_line(finding: &Finding, presentation: &RulePresentation) -> String {
    let head = match &finding.kind {
        FindingKind::Violation(witness) => format!(
            "violation: rules `{}` and `{}` disagree on input #{}",
            rule_name(presentation, witness.lhs.0),
            rule_name(presentation, witness.rhs.0),
            witness.row.0
        ),
        FindingKind::Underconstraint(witness) => underconstraint_head(&witness.case),
        FindingKind::Redundant(witness) => format!(
            "redundant: {} derives from {}",
            path_string(&witness.derivable),
            path_string(&witness.reducing)
        ),
        FindingKind::Contradiction(witness) => contradiction_head(&witness.scope),
        FindingKind::Dangling(witness) => format!(
            "dangling: arrow {} at row #{}",
            witness.arrow.0, witness.src.0
        ),
    };
    format!("{head}{}", label_suffix(finding))
}

/// The underconstraint head: the uncovered input class or the parallel
/// paths, with the witness ids in text.
fn underconstraint_head(case: &DomainQuestion) -> String {
    match case {
        DomainQuestion::UncoveredInputs { class, examples } => {
            let rows = examples
                .iter()
                .map(|row| row.0.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("underconstraint: uncovered inputs {class} (examples {rows})")
        }
        DomainQuestion::ParallelPaths { lhs, rhs } => format!(
            "underconstraint: parallel paths {} / {}",
            path_string(lhs),
            path_string(rhs)
        ),
    }
}

/// The contradiction head: the explicit equation scope.
fn contradiction_head(scope: &[usize]) -> String {
    let scope = scope
        .iter()
        .map(|index| format!("eq-{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("contradiction: {scope} admit no instance")
}

/// The non-default epistemic weight as rendered text; `executed` stays
/// off the line (the §5 renderer's byte-reproducibility rule).
fn label_suffix(finding: &Finding) -> &'static str {
    match finding.label {
        catherine_core::findings::EpistemicLabel::Executed => "",
        catherine_core::findings::EpistemicLabel::Hypothesized => " (hypothesized)",
        catherine_core::findings::EpistemicLabel::Unknown => " (unknown)",
    }
}

/// A rule index as its declared name; the bare index is the totality
/// fallback for a programmatically assembled witness.
fn rule_name(presentation: &RulePresentation, index: usize) -> String {
    presentation
        .rules
        .get(index)
        .map(|rule| rule.name.0.clone())
        .unwrap_or_else(|| format!("#{index}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::findings::{EpistemicLabel, EquationWitness};
    use catherine_core::model::{Ident, Path, RowId, Rule};

    fn table() -> RulePresentation {
        RulePresentation {
            domain: Ident("Input".to_string()),
            columns: BTreeMap::new(),
            rules: vec![
                Rule {
                    name: Ident("small".to_string()),
                    predicate: catherine_core::model::Predicate::And(Vec::new()),
                    action: "small".to_string(),
                },
                Rule {
                    name: Ident("large".to_string()),
                    predicate: catherine_core::model::Predicate::And(Vec::new()),
                    action: "large".to_string(),
                },
            ],
            policy: None,
        }
    }

    #[test]
    fn each_finding_renders_on_one_line() {
        let findings = vec![
            Finding {
                label: EpistemicLabel::Hypothesized,
                kind: FindingKind::Underconstraint(
                    catherine_core::findings::UnderconstraintWitness {
                        case: DomainQuestion::UncoveredInputs {
                            class: "age = 70".to_string(),
                            examples: vec![RowId(3)],
                        },
                        question: "which action applies?".to_string(),
                    },
                ),
            },
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Violation(EquationWitness {
                    equation: 0,
                    row: RowId(1),
                    lhs: RowId(0),
                    rhs: RowId(1),
                }),
            },
            Finding {
                label: EpistemicLabel::Executed,
                kind: FindingKind::Redundant(catherine_core::findings::RedundantWitness {
                    derivable: Path(vec![Ident("even".to_string())]),
                    reducing: Path(vec![Ident("all".to_string())]),
                }),
            },
        ];
        let lines = render_rules(&findings, &table());
        assert_eq!(lines.len(), 3, "exactly one line per finding");
        assert_eq!(
            lines[0],
            "underconstraint: uncovered inputs age = 70 (examples 3) (hypothesized)"
        );
        assert_eq!(
            lines[1],
            "violation: rules `small` and `large` disagree on input #1"
        );
        assert_eq!(lines[2], "redundant: even derives from all");
    }

    #[test]
    fn a_clean_analysis_says_so() {
        assert_eq!(
            render_rules(&[], &table()),
            vec!["rules: clean — no findings over the declared domain".to_string()]
        );
    }
}
