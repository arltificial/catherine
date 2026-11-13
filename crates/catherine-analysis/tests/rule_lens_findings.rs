//! Purpose: the four Decision-1 finding mappings over the committed
//! golden fixtures (openspec change `add-rule-lens` phase B, task 6) —
//! gap → underconstraint/hypothesized, overlap under `DeclaredDisjoint` →
//! violation/executed, shadow and derivable → redundant/executed — plus
//! the policy cases that must not report, and `UNKNOWN { scope }` on
//! budget expiry.
//!
//! Responsibilities: run the analysis-layer producer over each fixture's
//! documented finite domain, assert the exact `Finding` (label + kind +
//! witness) and its serialized JSON fields, prove that a priority policy
//! and an absent policy produce no overlap finding, and prove a short
//! budget is a first-class `Unknown` never a partial finding set.
//!
//! Rationale: no new finding kinds exist (Decision 1), so the producer is
//! pinned against the P2 taxonomy's existing witness shapes, and the
//! policy-dependence tests enforce Decision 3's "no implicit precedence"
//! at the output boundary. Assertions are structural (`PartialEq` and
//! parsed JSON fields), never snapshots; the printed compact JSON is
//! evidence for the phase report only.

use catherine_analysis::bounds::{ScopeCaps, UNCOVERED_INPUTS_QUESTION};
use catherine_analysis::rule_lens::{rule_lens, RuleLensOutcome};
use catherine_core::findings::{
    DomainQuestion, EpistemicLabel, EquationWitness, Finding, FindingKind, RedundantWitness,
    UnderconstraintWitness, ViolationWitness,
};
use catherine_core::model::{
    CmpOp, ColumnType, Ident, Literal, Path, Policy, Predicate, RowId, Rule, RulePresentation,
};
use catherine_core::parse::parse;
use catherine_core::rule_lens::{RuleDomain, RuleScope};
use std::collections::BTreeMap;

fn fixture(name: &str) -> RulePresentation {
    let path = format!(
        "{}/../../fixtures/rule_lens/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("golden fixture exists");
    let model = parse(&text).expect("golden fixture parses");
    model
        .rule_presentation
        .expect("the fixture declares a rule presentation")
}

fn int_domain(table: &RulePresentation, column: &str, values: &[i64]) -> RuleDomain {
    let supplied: BTreeMap<Ident, Vec<Literal>> = [(
        Ident(column.to_string()),
        values.iter().copied().map(Literal::Int).collect(),
    )]
    .into_iter()
    .collect();
    RuleDomain::from_presentation(table, supplied).expect("domain matches the presentation")
}

fn fixture_outcome(
    name: &str,
    column: &str,
    values: &[i64],
) -> (RulePresentation, RuleDomain, Vec<Finding>) {
    let table = fixture(name);
    let domain = int_domain(&table, column, values);
    let RuleLensOutcome::Findings(findings) = rule_lens(&table, &domain, &ScopeCaps::default())
    else {
        panic!("{name}: the default budget covers the fixture domain");
    };
    (table, domain, findings)
}

fn rule(name: &str, predicate: Predicate, action: &str) -> Rule {
    Rule {
        name: Ident(name.to_string()),
        predicate,
        action: action.to_string(),
    }
}

fn cmp(column: &str, op: CmpOp, value: i64) -> Predicate {
    Predicate::Cmp {
        column: Ident(column.to_string()),
        op,
        value: Literal::Int(value),
    }
}

/// gap → underconstraint, hypothesized, with the uncovered input class
/// and every uncovered input as an example.
#[test]
fn gap_fixture_maps_to_a_hypothesized_underconstraint() {
    let (_, _, findings) = fixture_outcome("gap.ct.json", "age", &[10, 20, 30, 70]);
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Hypothesized,
            kind: FindingKind::Underconstraint(UnderconstraintWitness {
                case: DomainQuestion::UncoveredInputs {
                    class: "age = 70".to_string(),
                    examples: vec![RowId(3)],
                },
                question: UNCOVERED_INPUTS_QUESTION.to_string(),
            }),
        }]
    );
    let json = serde_json::to_value(&findings).expect("findings serialize");
    assert_eq!(json[0]["label"].as_str(), Some("hypothesized"));
    assert_eq!(json[0]["kind"].as_str(), Some("underconstraint"));
    assert_eq!(
        json[0]["witness"]["case"].as_str(),
        Some("uncovered_inputs")
    );
    assert_eq!(
        json[0]["witness"]["detail"]["class"].as_str(),
        Some("age = 70")
    );
    assert_eq!(
        json[0]["witness"]["detail"]["examples"][0].as_u64(),
        Some(3)
    );
    println!(
        "gap: {}",
        serde_json::to_string(&findings[0]).expect("serializes")
    );
}

/// overlap under DeclaredDisjoint → violation, executed, witness encoded
/// in `EquationWitness` (see `catherine_analysis::rule_lens` docs).
#[test]
fn overlap_fixture_maps_to_an_executed_violation() {
    let (_, _, findings) = fixture_outcome("overlap-disjoint.ct.json", "amount", &[50, 100, 200]);
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: 0,
                row: RowId(1),
                lhs: RowId(0),
                rhs: RowId(1),
            })),
        }]
    );
    let json = serde_json::to_value(&findings).expect("findings serialize");
    assert_eq!(json[0]["kind"].as_str(), Some("violation"));
    assert_eq!(json[0]["witness"]["row"].as_u64(), Some(1));
    assert_eq!(json[0]["witness"]["lhs"].as_u64(), Some(0));
    assert_eq!(json[0]["witness"]["rhs"].as_u64(), Some(1));
    println!(
        "overlap: {}",
        serde_json::to_string(&findings[0]).expect("serializes")
    );
}

/// shadow → redundant, executed, the dead rule named with its covering
/// earlier rule.
#[test]
fn shadow_fixture_maps_to_an_executed_redundancy() {
    let (_, _, findings) = fixture_outcome("shadow.ct.json", "n", &[1, 2, 3, 4, 5, 6]);
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Redundant(RedundantWitness {
                derivable: Path(vec![Ident("even".to_string())]),
                reducing: Path(vec![Ident("all".to_string())]),
            }),
        }]
    );
    let json = serde_json::to_value(&findings).expect("findings serialize");
    assert_eq!(json[0]["kind"].as_str(), Some("redundant"));
    assert_eq!(json[0]["witness"]["derivable"][0].as_str(), Some("even"));
    assert_eq!(json[0]["witness"]["reducing"][0].as_str(), Some("all"));
    println!(
        "shadow: {}",
        serde_json::to_string(&findings[0]).expect("serializes")
    );
}

/// derivable → redundant, executed, the rule and its two combining
/// factors.
#[test]
fn derivable_fixture_maps_to_an_executed_redundancy() {
    let values: Vec<i64> = (1..=15).collect();
    let (_, _, findings) = fixture_outcome("derivable.ct.json", "n", &values);
    assert_eq!(
        findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Redundant(RedundantWitness {
                derivable: Path(vec![Ident("fizzbuzz".to_string())]),
                reducing: Path(vec![Ident("fizz".to_string()), Ident("buzz".to_string())]),
            }),
        }]
    );
    let json = serde_json::to_value(&findings).expect("findings serialize");
    assert_eq!(
        json[0]["witness"]["derivable"][0].as_str(),
        Some("fizzbuzz")
    );
    assert_eq!(json[0]["witness"]["reducing"][1].as_str(), Some("buzz"));
    println!(
        "derivable: {}",
        serde_json::to_string(&findings[0]).expect("serializes")
    );
}

/// The same regions under a priority policy are legal: no overlap finding.
#[test]
fn priority_policy_reports_no_overlap_finding() {
    let mut table = fixture("overlap-disjoint.ct.json");
    table.policy = Some(Policy::PriorityOrder);
    let domain = int_domain(&table, "amount", &[50, 100, 200]);
    let RuleLensOutcome::Findings(findings) = rule_lens(&table, &domain, &ScopeCaps::default())
    else {
        panic!("the default budget covers the fixture domain");
    };
    assert!(findings.is_empty(), "priority order resolves the overlap");
}

/// No declared policy: overlap is a diagnostic, mapped to no finding —
/// Decision 1 pins overlap only under DeclaredDisjoint.
#[test]
fn an_absent_policy_reports_no_taxonomy_finding_for_overlap() {
    let table = RulePresentation {
        domain: Ident("Input".to_string()),
        columns: [(Ident("age".to_string()), ColumnType::Int)]
            .into_iter()
            .collect(),
        rules: vec![
            rule("young", cmp("age", CmpOp::Le, 20), "young"),
            rule("old", cmp("age", CmpOp::Gt, 20), "old"),
            rule("vip", cmp("age", CmpOp::Ge, 20), "vip"),
        ],
        policy: None,
    };
    let domain = int_domain(&table, "age", &[10, 20, 70]);
    let RuleLensOutcome::Findings(findings) = rule_lens(&table, &domain, &ScopeCaps::default())
    else {
        panic!("the default budget covers the declared domain");
    };
    assert!(
        findings.is_empty(),
        "an undeclared policy never yields an overlap finding"
    );
}

/// A budget smaller than the declared domain is `UNKNOWN` with the exact
/// scope — never a partial finding set.
#[test]
fn a_short_budget_yields_unknown_with_the_explored_scope() {
    let table = fixture("gap.ct.json");
    let domain = int_domain(&table, "age", &[10, 20, 30, 70]);
    let caps = ScopeCaps {
        max_candidates: 3,
        ..ScopeCaps::default()
    };
    assert_eq!(
        rule_lens(&table, &domain, &caps),
        RuleLensOutcome::Unknown {
            scope: RuleScope {
                columns: [(Ident("age".to_string()), 4)].into_iter().collect(),
                candidates_explored: 3,
            },
        }
    );
}

/// A zero budget is `UNKNOWN` with the empty scope, never a clean result.
#[test]
fn a_zero_budget_yields_unknown_with_the_empty_scope() {
    let table = fixture("gap.ct.json");
    let domain = int_domain(&table, "age", &[10, 20, 30, 70]);
    let caps = ScopeCaps {
        max_candidates: 0,
        ..ScopeCaps::default()
    };
    assert_eq!(
        rule_lens(&table, &domain, &caps),
        RuleLensOutcome::Unknown {
            scope: RuleScope {
                columns: BTreeMap::new(),
                candidates_explored: 0,
            },
        }
    );
}

/// Repeated runs serialize identically (sorted findings, ordered sets).
#[test]
fn repeated_runs_are_deterministic() {
    let (table, domain, first) =
        fixture_outcome("derivable.ct.json", "n", &(1..=15).collect::<Vec<_>>());
    let RuleLensOutcome::Findings(second) = rule_lens(&table, &domain, &ScopeCaps::default())
    else {
        panic!("the default budget covers the fixture domain");
    };
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_string(&first).expect("serializes"),
        serde_json::to_string(&second).expect("serializes")
    );
}
