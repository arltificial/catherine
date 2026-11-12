//! Purpose: example-scoped discovery over a candidate rule table
//! (openspec change `add-rule-lens` phase C, task 7) — a covered example
//! is clean, an uncovered example is a Decision-1 gap finding, and an
//! example whose induced action differs from the expected output is a
//! violation witnessed by the example.
//!
//! Responsibilities: pin the sample-scoped contract of
//! `catherine_analysis::rule_lens::discover` against the committed gap
//! fixture, including the one-finding-per-uncovered-sample aggregation,
//! the witness encoding for a differing example, and the serialized
//! finding shape. Assertions are structural (`PartialEq` and parsed JSON
//! fields), never snapshots.
//!
//! Rationale: discovery makes no exhaustiveness claim — the supplied
//! examples are the universe of the verdict, which the report's
//! `examples_checked` states and these tests read back. The taxonomy is
//! the existing one (Decision 1): gap stays `underconstraint`/
//! `hypothesized`, a differing action is a `violation`/`executed`.

use catherine_analysis::bounds::UNCOVERED_INPUTS_QUESTION;
use catherine_analysis::rule_lens::{discover, RuleExample};
use catherine_core::findings::{
    DomainQuestion, EpistemicLabel, EquationWitness, Finding, FindingKind, UnderconstraintWitness,
};
use catherine_core::model::{Ident, Literal, RowId, RulePresentation};
use catherine_core::parse::parse;
use catherine_core::rule_lens::RuleDomain;
use std::collections::BTreeMap;

fn fixture(name: &str) -> RulePresentation {
    let path = format!(
        "{}/../../fixtures/rule_lens/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("golden fixture exists");
    parse(&text)
        .expect("golden fixture parses")
        .rule_presentation
        .expect("the fixture declares a rule presentation")
}

fn domain(table: &RulePresentation) -> RuleDomain {
    let supplied: BTreeMap<Ident, Vec<Literal>> = [(
        Ident("age".to_string()),
        vec![
            Literal::Int(10),
            Literal::Int(20),
            Literal::Int(30),
            Literal::Int(70),
        ],
    )]
    .into_iter()
    .collect();
    RuleDomain::from_presentation(table, supplied).expect("domain matches the presentation")
}

fn example(age: i64, expected: &str) -> RuleExample {
    RuleExample {
        input: [(Ident("age".to_string()), Literal::Int(age))]
            .into_iter()
            .collect(),
        expected: expected.to_string(),
    }
}

/// A covered example with the expected action is clean — and the report
/// states its scope (one example checked), never exhaustiveness.
#[test]
fn a_covered_example_is_clean() {
    let table = fixture("gap.ct.json");
    let report = discover(&table, &domain(&table), &[example(30, "adult")])
        .expect("the example matches the declared vocabulary");
    assert!(report.findings.is_empty(), "the table covers age = 30");
    assert_eq!(
        report.examples_checked, 1,
        "the verdict is scoped over the supplied example, not the domain"
    );
}

/// An uncovered example is a Decision-1 gap: `underconstraint`,
/// `hypothesized`, `UncoveredInputs` naming the class and the example's
/// dense index in the sample.
#[test]
fn an_uncovered_example_is_a_gap_finding() {
    let table = fixture("gap.ct.json");
    let report = discover(&table, &domain(&table), &[example(70, "senior")])
        .expect("the example matches the declared vocabulary");
    assert_eq!(
        report.findings,
        vec![Finding {
            label: EpistemicLabel::Hypothesized,
            kind: FindingKind::Underconstraint(UnderconstraintWitness {
                case: DomainQuestion::UncoveredInputs {
                    class: "age = 70".to_string(),
                    examples: vec![RowId(0)],
                },
                question: UNCOVERED_INPUTS_QUESTION.to_string(),
            }),
        }]
    );
    assert_eq!(report.examples_checked, 1);
    let json = serde_json::to_value(&report.findings).expect("findings serialize");
    assert_eq!(json[0]["kind"].as_str(), Some("underconstraint"));
    assert_eq!(json[0]["label"].as_str(), Some("hypothesized"));
    assert_eq!(
        json[0]["witness"]["detail"]["class"].as_str(),
        Some("age = 70")
    );
    assert_eq!(
        json[0]["witness"]["detail"]["examples"][0].as_u64(),
        Some(0)
    );
    println!(
        "discovery gap: {}",
        serde_json::to_string(&report.findings[0]).expect("serializes")
    );
}

/// An example the table covers with a different action is a violation
/// witnessed by the example: the table selected rule 1 (`adult`) for
/// `age = 20`, but the expected output was `kid`.
#[test]
fn a_differing_example_is_a_violation_finding() {
    let table = fixture("gap.ct.json");
    let report = discover(&table, &domain(&table), &[example(20, "kid")])
        .expect("the example matches the declared vocabulary");
    assert_eq!(
        report.findings,
        vec![Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(EquationWitness {
                equation: 1,
                row: RowId(0),
                lhs: RowId(1),
                rhs: RowId(1),
            }),
        }]
    );
    assert_eq!(report.examples_checked, 1);
    let json = serde_json::to_value(&report.findings).expect("findings serialize");
    assert_eq!(json[0]["kind"].as_str(), Some("violation"));
    assert_eq!(json[0]["label"].as_str(), Some("executed"));
    assert_eq!(json[0]["witness"]["row"].as_u64(), Some(0));
    println!(
        "discovery violation: {}",
        serde_json::to_string(&report.findings[0]).expect("serializes")
    );
}
