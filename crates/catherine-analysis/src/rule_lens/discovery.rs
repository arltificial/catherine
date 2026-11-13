//! Purpose: the rule lens's discovery support (openspec change
//! `add-rule-lens` phase C, task 7) — check a candidate rule table
//! against declared input→output examples, mapping each failure onto the
//! existing taxonomy with the example as the witness.
//!
//! Responsibilities: [`discover`] validates each example against the
//! presentation's typed columns, resolves the table's induced action on
//! it (no match is a gap; a priority policy selects the first match; an
//! undeclared policy leaves intersecting matches ambiguous), and emits the
//! Decision-1 mappings: examples the table does not cover become one
//! `underconstraint`/`hypothesized` `UncoveredInputs` finding listing
//! every uncovered example index, and examples whose induced action is
//! not the expected output become `violation`/`executed` findings, one
//! per example.
//!
//! Witness encoding for a differing example (no new finding variants —
//! Decision 1): `EquationWitness.equation` is the rule the table selected
//! (the first matching rule when the policy is undeclared and several
//! match), `row` is the example's dense index in the supplied sample,
//! `lhs` is that selected rule, and `rhs` is the first matching rule
//! whose action equals the expected output (`lhs` when none does). An
//! ambiguous example — several rules match and no policy resolves them —
//! is a violation too: the table induces no unique action, which is not
//! the expected output.
//!
//! Rationale: the verdict is scoped over the supplied finite sample and
//! never claims exhaustiveness — [`DiscoveryReport::examples_checked`]
//! states the scope. The predicate language and the domain come from the
//! kernel (this module owns only the label/witness mapping, exactly as
//! the structural producer does); no enumeration is needed because the
//! sample is the universe of the claim.

use crate::bounds::uncovered_inputs_finding;
use catherine_core::findings::{
    EpistemicLabel, EquationWitness, Finding, FindingKind, ViolationWitness,
};
use catherine_core::model::{ColumnType, Ident, Literal, Policy, RowId, RulePresentation};
use catherine_core::rule_lens::{evaluate, RuleDomain};
use std::collections::BTreeMap;

/// One declared input→output example: an attribute assignment plus the
/// action a correct table must produce on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleExample {
    /// The input, keyed by declared column name.
    pub input: BTreeMap<Ident, Literal>,
    /// The action the table must assign to [`Self::input`].
    pub expected: String,
}

/// A sample-scoped discovery result: the findings over the supplied
/// examples and the number of examples checked. There is no
/// exhaustiveness claim — the sample is the scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryReport {
    /// The mapped findings, sorted by the taxonomy's total order.
    pub findings: Vec<Finding>,
    /// How many examples the verdict covers.
    pub examples_checked: usize,
}

/// A labeled rejection of an example that does not match the
/// presentation's declared input vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryError {
    /// The example supplies no value for a declared column.
    MissingColumn {
        /// The example's dense index in the supplied sample.
        example: usize,
        /// The declared column with no value.
        column: Ident,
    },
    /// The example names a column the presentation never declared.
    UnexpectedColumn {
        /// The example's dense index in the supplied sample.
        example: usize,
        /// The undeclared column.
        column: Ident,
    },
    /// A value does not have its column's declared type.
    WrongType {
        /// The example's dense index in the supplied sample.
        example: usize,
        /// The offending column.
        column: Ident,
        /// The column's declared type.
        expected: ColumnType,
    },
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingColumn { example, column } => write!(
                formatter,
                "example {example} supplies no value for column `{}`",
                column.0
            ),
            Self::UnexpectedColumn { example, column } => write!(
                formatter,
                "example {example} names column `{}`, which the rule presentation does not declare",
                column.0
            ),
            Self::WrongType {
                example,
                column,
                expected,
            } => write!(
                formatter,
                "example {example}'s value for column `{}` is not `{}`",
                column.0,
                expected.as_str()
            ),
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// Check the candidate table against every supplied example. The verdict
/// is scoped over this sample; an example outside the declared value
/// enumeration is still evaluated (predicates are structural), so callers
/// may test values the structural analysis did not enumerate.
pub fn discover(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    examples: &[RuleExample],
) -> Result<DiscoveryReport, DiscoveryError> {
    let mut uncovered = Vec::new();
    let mut class = None;
    let mut violations = Vec::new();
    for (index, example) in examples.iter().enumerate() {
        let input = aligned_input(presentation, domain, example, index)?;
        let matching = matching_rules(presentation, domain, &input);
        if matching.is_empty() {
            uncovered.push(RowId(index));
            class.get_or_insert_with(|| super::render_input(domain, &input));
            continue;
        }
        if selected_rule(presentation.policy, &matching)
            .is_some_and(|rule| presentation.rules[rule].action == example.expected)
        {
            continue;
        }
        violations.push(violation_finding(presentation, example, index, &matching));
    }
    let mut findings = violations;
    if let Some(class) = class {
        findings.push(uncovered_inputs_finding(class, uncovered));
    }
    findings.sort();
    Ok(DiscoveryReport {
        findings,
        examples_checked: examples.len(),
    })
}

/// The rules whose predicates accept the example, in priority order.
fn matching_rules(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    input: &[Literal],
) -> Vec<usize> {
    presentation
        .rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| evaluate(&rule.predicate, input, domain))
        .map(|(position, _)| position)
        .collect()
}

/// The rule the table induces on a covered example: the first match under
/// a priority policy, the only match under no policy or
/// `DeclaredDisjoint`, and `None` when several matches stay ambiguous.
fn selected_rule(policy: Option<Policy>, matching: &[usize]) -> Option<usize> {
    let first = *matching.first()?;
    let priority = matches!(policy, Some(Policy::PriorityOrder | Policy::FirstMatch));
    if priority || matching.len() == 1 {
        Some(first)
    } else {
        None
    }
}

/// The differing example as an `executed` violation (see the module docs
/// for the witness encoding).
fn violation_finding(
    presentation: &RulePresentation,
    example: &RuleExample,
    index: usize,
    matching: &[usize],
) -> Finding {
    let lhs = selected_rule(presentation.policy, matching).unwrap_or(matching[0]);
    let rhs = matching
        .iter()
        .copied()
        .find(|&rule| presentation.rules[rule].action == example.expected)
        .unwrap_or(lhs);
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: lhs,
            row: RowId(index),
            lhs: RowId(lhs),
            rhs: RowId(rhs),
        })),
    }
}

/// Align one example to the domain's declared column order, validating
/// the column set and each value's declared type.
fn aligned_input(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    example: &RuleExample,
    index: usize,
) -> Result<Vec<Literal>, DiscoveryError> {
    let columns = domain.columns();
    check_columns(columns, example, index)?;
    let mut input = Vec::with_capacity(columns.len());
    for column in columns {
        input.push(value_of(presentation, example, column, index)?);
    }
    Ok(input)
}

/// Reject an example that names a column outside the declared vocabulary.
fn check_columns(
    columns: &[Ident],
    example: &RuleExample,
    index: usize,
) -> Result<(), DiscoveryError> {
    match example
        .input
        .keys()
        .find(|column| !columns.contains(column))
    {
        Some(column) => Err(DiscoveryError::UnexpectedColumn {
            example: index,
            column: column.clone(),
        }),
        None => Ok(()),
    }
}

/// One declared column's value, validated against its declared type.
fn value_of(
    presentation: &RulePresentation,
    example: &RuleExample,
    column: &Ident,
    index: usize,
) -> Result<Literal, DiscoveryError> {
    let Some(value) = example.input.get(column) else {
        return Err(DiscoveryError::MissingColumn {
            example: index,
            column: column.clone(),
        });
    };
    let column_type = presentation
        .columns
        .get(column)
        .copied()
        .unwrap_or(ColumnType::String);
    if !matches_type(value, column_type) {
        return Err(DiscoveryError::WrongType {
            example: index,
            column: column.clone(),
            expected: column_type,
        });
    }
    Ok(value.clone())
}

/// Whether a literal has the column's declared type.
fn matches_type(value: &Literal, column_type: ColumnType) -> bool {
    matches!(
        (value, column_type),
        (Literal::Int(_), ColumnType::Int)
            | (Literal::Bool(_), ColumnType::Bool)
            | (Literal::String(_), ColumnType::String)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_sample_is_a_clean_report() {
        let table = RulePresentation {
            domain: Ident("Input".to_string()),
            columns: BTreeMap::new(),
            rules: Vec::new(),
            policy: None,
        };
        let domain = RuleDomain::from_presentation(&table, BTreeMap::new())
            .expect("the empty presentation has the empty domain");
        assert_eq!(
            discover(&table, &domain, &[]),
            Ok(DiscoveryReport {
                findings: Vec::new(),
                examples_checked: 0,
            })
        );
    }

    #[test]
    fn an_untyped_example_value_is_rejected() {
        let table = RulePresentation {
            domain: Ident("Input".to_string()),
            columns: [(Ident("n".to_string()), ColumnType::Int)]
                .into_iter()
                .collect(),
            rules: Vec::new(),
            policy: None,
        };
        let domain = RuleDomain::from_presentation(
            &table,
            [(Ident("n".to_string()), vec![Literal::Int(1)])]
                .into_iter()
                .collect(),
        )
        .expect("domain matches");
        let example = RuleExample {
            input: [(Ident("n".to_string()), Literal::String("one".to_string()))]
                .into_iter()
                .collect(),
            expected: "a".to_string(),
        };
        assert_eq!(
            discover(&table, &domain, &[example]),
            Err(DiscoveryError::WrongType {
                example: 0,
                column: Ident("n".to_string()),
                expected: ColumnType::Int,
            })
        );
    }
}
