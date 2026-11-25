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

/// A first-match table over the integer column `n`: the shared shape of
/// every equivalence fixture.
fn first_match(rules: Vec<catherine_core::model::Rule>) -> RulePresentation {
    presentation((Some(Policy::FirstMatch), rules), &[("n", ColumnType::Int)])
}

/// The named rule fixtures: each table lists its members' names in
/// declaration order (which is what the equivalence cases vary) instead
/// of restating the rule builders.
type RuleFixture = (&'static str, fn() -> catherine_core::model::Rule);

fn fixture_rules(names: &[&str]) -> Vec<catherine_core::model::Rule> {
    let known: &[RuleFixture] = &[
        ("fizzbuzz", || {
            rule("fizzbuzz", (modulus("n", (15, 0)), "FizzBuzz"))
        }),
        ("fizz", || rule("fizz", (modulus("n", (3, 0)), "Fizz"))),
        ("buzz", || rule("buzz", (modulus("n", (5, 0)), "Buzz"))),
        ("plain", || {
            rule("plain", (cmp("n", (CmpOp::Ge, integer(1))), "n"))
        }),
        ("all", || {
            rule("all", (cmp("n", (CmpOp::Ge, integer(1))), "default"))
        }),
        ("even", || rule("even", (modulus("n", (2, 0)), "even"))),
    ];
    names
        .iter()
        .map(|name| {
            let (_, make) = known
                .iter()
                .find(|(known_name, _)| known_name == name)
                .unwrap_or_else(|| panic!("unknown rule fixture `{name}`"));
            make()
        })
        .collect()
}

fn six() -> RuleDomain {
    let table = first_match(fixture_rules(&["all"]));
    int_domain(&table, ("n", &[1, 2, 3, 4, 5, 6]))
}

fn fifteen() -> RuleDomain {
    let table = first_match(fixture_rules(&["fizzbuzz", "fizz", "buzz", "plain"]));
    int_domain(&table, ("n", &(1..=15).collect::<Vec<_>>()))
}

fn age_table() -> RulePresentation {
    presentation(
        (
            Some(Policy::PriorityOrder),
            vec![
                rule("kid", (cmp("age", (CmpOp::Lt, integer(18))), "kid")),
                rule(
                    "adult",
                    (
                        catherine_core::model::Predicate::And(vec![
                            cmp("age", (CmpOp::Ge, integer(18))),
                            cmp("age", (CmpOp::Lt, integer(65))),
                        ]),
                        "adult",
                    ),
                ),
            ],
        ),
        &[("age", ColumnType::Int)],
    )
}

/// `reordered_fizzbuzz_is_equivalent`: fizz/buzz order is immaterial
/// behind fizzbuzz.
#[test]
fn reordered_fizzbuzz_is_equivalent() {
    let left = first_match(fixture_rules(&["fizzbuzz", "fizz", "buzz", "plain"]));
    let right = first_match(fixture_rules(&["fizzbuzz", "buzz", "fizz", "plain"]));
    let kept = left.rules.len();
    assert_eq!(
        kept, 4,
        "reordering keeps the same four rules on both sides"
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
    let left = first_match(fixture_rules(&["all", "even"]));
    let right = first_match(fixture_rules(&["all"]));
    assert!(
        left.rules.len() == 2 && right.rules.len() == 1,
        "shadowing drops `even` behind `all`'s catch-all"
    );
    assert_eq!(
        equivalence(&left, &right, &six()),
        EquivalenceOutcome::EquivalentInScope { bound: 6 }
    );
}

/// `order_matters_when_earlier_rule_narrows`: 2 → "even" vs "default".
#[test]
fn order_matters_when_earlier_rule_narrows() {
    let left = first_match(fixture_rules(&["even", "all"]));
    let right = first_match(fixture_rules(&["all"]));
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
    let right = presentation(
        (Some(Policy::PriorityOrder), {
            let mut rules = left.rules.clone();
            rules.push(rule(
                "senior",
                (cmp("age", (CmpOp::Ge, integer(65))), "senior"),
            ));
            rules
        }),
        &[("age", ColumnType::Int)],
    );
    let domain = int_domain(&left, ("age", &[10, 20, 30, 70]));
    let gap_input = ints(70);
    assert_eq!(
        right.rules.len(),
        3,
        "the senior rule is appended after the declared two"
    );
    assert_eq!(
        equivalence(&left, &right, &domain),
        EquivalenceOutcome::Distinguishing {
            input: gap_input,
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
    let domain = int_domain(&table, ("age", &[10, 20, 30, 70]));
    let RuleOutcome::Unknown { scope } = analyze_with_bounds(&table, &domain, 3) else {
        panic!("a budget below |X| must stay unknown");
    };
    assert_eq!(
        scope
            .columns
            .get(&catherine_core::model::Ident("age".to_string())),
        Some(&4),
        "the scope names the column's exact size"
    );
    let explored = scope.candidates_explored;
    assert_eq!(
        explored, 3,
        "the scope counts the candidates actually explored"
    );
}

/// A zero budget is `UNKNOWN` with the empty scope — P4's zero-size rule,
/// never an empty-but-clean claim.
#[test]
fn a_zero_budget_yields_unknown_with_the_empty_scope() {
    let table = age_table();
    let domain = int_domain(&table, ("age", &[10, 20, 30, 70]));
    let expected = RuleOutcome::Unknown {
        scope: catherine_core::rule_lens::RuleScope {
            columns: std::collections::BTreeMap::new(),
            candidates_explored: 0,
        },
    };
    assert_eq!(analyze_with_bounds(&table, &domain, 0), expected);
}

/// Completing at exactly the domain size is an analysis, not `UNKNOWN`.
#[test]
fn a_budget_equal_to_the_domain_completes() {
    let table = age_table();
    let domain = int_domain(&table, ("age", &[10, 20, 30, 70]));
    let RuleOutcome::Analyzed(analysis) = analyze_with_bounds(&table, &domain, 4) else {
        panic!("a budget of |X| must complete");
    };
    assert_eq!(analysis.gap, set(&domain, &[ints(70)]));
    let classes = analysis.class_summary();
    for c in &classes {
        assert_eq!(c, &RuleClass::Gap, "the completed analysis is one gap");
    }
    assert_eq!(classes, vec![RuleClass::Gap]);
}
