//! Purpose: sim/kernel parity for rule-table equivalence and the bounded
//! rule-lens outcome (openspec change `add-rule-lens` phase B, tasks 5/6;
//! design.md Decisions 3/5) — the oracle's four equivalence cases and the
//! `UNKNOWN { scope }` discipline.
//!
//! Responsibilities: reordered FizzBuzz equivalence, shadowed-rule
//! removal equivalence, order-sensitivity with witness `(2,)`, and gap
//! witness `(70,)`; then budget expiry (`UNKNOWN` with the exact column
//! sizes and candidates explored), the zero-budget empty scope, and
//! completion at exactly the domain size. No silent clean result exists
//! under a short budget.
//!
//! Rationale: equivalence claims are proven within the declared domain,
//! never asserted; the witness is the first differing input in the
//! oracle's enumeration order. The bounded tests pin the honesty rule
//! reused from P4: an expired budget is a first-class non-answer.

mod common;

use catherine_core::model::{CmpOp, ColumnType, Policy, RulePresentation};
use catherine_core::rule_lens::{
    analyze_with_bounds, equivalence, EquivalenceOutcome, RuleClass, RuleDomain, RuleOutcome,
};
use common::rule_lens::{cmp, int_domain, integer, ints, modulus, presentation, rule, set};

fn fizzbuzz() -> Vec<catherine_core::model::Rule> {
    vec![
        rule("fizzbuzz", modulus("n", 15, 0), "FizzBuzz"),
        rule("fizz", modulus("n", 3, 0), "Fizz"),
        rule("buzz", modulus("n", 5, 0), "Buzz"),
        rule("plain", cmp("n", CmpOp::Ge, integer(1)), "n"),
    ]
}

fn fizzbuzz_swapped() -> Vec<catherine_core::model::Rule> {
    vec![
        rule("fizzbuzz", modulus("n", 15, 0), "FizzBuzz"),
        rule("buzz", modulus("n", 5, 0), "Buzz"),
        rule("fizz", modulus("n", 3, 0), "Fizz"),
        rule("plain", cmp("n", CmpOp::Ge, integer(1)), "n"),
    ]
}

fn default_all() -> Vec<catherine_core::model::Rule> {
    vec![rule("all", cmp("n", CmpOp::Ge, integer(1)), "default")]
}

fn default_then_even() -> Vec<catherine_core::model::Rule> {
    vec![
        rule("all", cmp("n", CmpOp::Ge, integer(1)), "default"),
        rule("even", modulus("n", 2, 0), "even"),
    ]
}

fn even_then_default() -> Vec<catherine_core::model::Rule> {
    vec![
        rule("even", modulus("n", 2, 0), "even"),
        rule("all", cmp("n", CmpOp::Ge, integer(1)), "default"),
    ]
}

fn six() -> RuleDomain {
    let table = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        default_all(),
    );
    int_domain(&table, "n", &[1, 2, 3, 4, 5, 6])
}

fn fifteen() -> RuleDomain {
    let table = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        fizzbuzz(),
    );
    int_domain(&table, "n", &(1..=15).collect::<Vec<_>>())
}

fn age_table() -> RulePresentation {
    presentation(
        Some(Policy::PriorityOrder),
        &[("age", ColumnType::Int)],
        vec![
            rule("kid", cmp("age", CmpOp::Lt, integer(18)), "kid"),
            rule(
                "adult",
                catherine_core::model::Predicate::And(vec![
                    cmp("age", CmpOp::Ge, integer(18)),
                    cmp("age", CmpOp::Lt, integer(65)),
                ]),
                "adult",
            ),
        ],
    )
}

/// `reordered_fizzbuzz_is_equivalent`: fizz/buzz order is immaterial
/// behind fizzbuzz.
#[test]
fn reordered_fizzbuzz_is_equivalent() {
    let left = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        fizzbuzz(),
    );
    let right = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        fizzbuzz_swapped(),
    );
    assert_eq!(
        equivalence(&left, &right, &fifteen()),
        EquivalenceOutcome::EquivalentInScope { bound: 15 }
    );
}

/// `shadowed_rule_drops_without_witness`: `even` never fires behind
/// `all`, so both tables induce "default" everywhere.
#[test]
fn shadowed_rule_drops_without_witness() {
    let left = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        default_then_even(),
    );
    let right = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        default_all(),
    );
    assert_eq!(
        equivalence(&left, &right, &six()),
        EquivalenceOutcome::EquivalentInScope { bound: 6 }
    );
}

/// `order_matters_when_earlier_rule_narrows`: 2 → "even" vs "default".
#[test]
fn order_matters_when_earlier_rule_narrows() {
    let left = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        even_then_default(),
    );
    let right = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        default_all(),
    );
    assert_eq!(
        equivalence(&left, &right, &six()),
        EquivalenceOutcome::Distinguishing {
            input: ints(2),
            index: 1,
            bound: 6,
        }
    );
}

/// `gap_input_distinguishes_tables`: 70 is a gap on the left and
/// "senior" on the right.
#[test]
fn gap_input_distinguishes_tables() {
    let left = age_table();
    let right = presentation(Some(Policy::PriorityOrder), &[("age", ColumnType::Int)], {
        let mut rules = left.rules.clone();
        rules.push(rule("senior", cmp("age", CmpOp::Ge, integer(65)), "senior"));
        rules
    });
    let domain = int_domain(&left, "age", &[10, 20, 30, 70]);
    assert_eq!(
        equivalence(&left, &right, &domain),
        EquivalenceOutcome::Distinguishing {
            input: ints(70),
            index: 3,
            bound: 4,
        }
    );
}

/// A budget smaller than the domain is `UNKNOWN` with the exact scope;
/// nothing is claimed about the unvisited inputs.
#[test]
fn a_short_budget_yields_unknown_with_the_explored_scope() {
    let table = age_table();
    let domain = int_domain(&table, "age", &[10, 20, 30, 70]);
    assert_eq!(
        analyze_with_bounds(&table, &domain, 3),
        RuleOutcome::Unknown {
            scope: catherine_core::rule_lens::RuleScope {
                columns: [(catherine_core::model::Ident("age".to_string()), 4)]
                    .into_iter()
                    .collect(),
                candidates_explored: 3,
            },
        }
    );
}

/// A zero budget is `UNKNOWN` with the empty scope — P4's zero-size rule,
/// never an empty-but-clean claim.
#[test]
fn a_zero_budget_yields_unknown_with_the_empty_scope() {
    let table = age_table();
    let domain = int_domain(&table, "age", &[10, 20, 30, 70]);
    assert_eq!(
        analyze_with_bounds(&table, &domain, 0),
        RuleOutcome::Unknown {
            scope: catherine_core::rule_lens::RuleScope {
                columns: std::collections::BTreeMap::new(),
                candidates_explored: 0,
            },
        }
    );
}

/// Completing at exactly the domain size is an analysis, not `UNKNOWN`.
#[test]
fn a_budget_equal_to_the_domain_completes() {
    let table = age_table();
    let domain = int_domain(&table, "age", &[10, 20, 30, 70]);
    let RuleOutcome::Analyzed(analysis) = analyze_with_bounds(&table, &domain, 4) else {
        panic!("a budget of |X| must complete");
    };
    assert_eq!(analysis.gap, set(&domain, &[ints(70)]));
    assert_eq!(analysis.class_summary(), vec![RuleClass::Gap]);
}
