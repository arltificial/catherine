//! Purpose: the committed rule-lens golden fixtures analysed by the
//! kernel (openspec change `add-rule-lens` phase B, task 5) — each
//! fixture's documented domain reproduces the oracle's structure, so the
//! documents in `fixtures/rule_lens/` and the semantics agree.
//!
//! Responsibilities: parse each fixture, bind the sim's finite domain
//! (`README.md` names the case), analyse it, and assert the bug class,
//! the policy-dependent effective regions, and the exact witness data the
//! analysis crate later maps to a finding.
//!
//! Rationale: phase A pinned the format; this test pins that the format's
//! four fixtures are semantically the four classes the README claims.
//! Findings mapping is tested in `catherine-analysis` (the taxonomy
//! producer layer), keeping this binary about kernel semantics.

mod common;

use catherine_core::rule_lens::{analyze, RuleClass, RuleDerivation, RuleOverlap};
use common::rule_lens::{fixture_presentation, int_domain, ints, set};

#[test]
fn the_gap_fixture_has_one_uncovered_input() {
    let table = fixture_presentation("gap.ct.json");
    let domain = int_domain(&table, "age", &[10, 20, 30, 70]);
    let analysis = analyze(&table, &domain);
    assert_eq!(analysis.gap, set(&domain, &[ints(70)]));
    assert_eq!(analysis.shadow, Some(vec![]));
    assert!(analysis.overlaps.is_empty());
    assert!(analysis.derivable.is_empty());
    assert_eq!(analysis.class_summary(), vec![RuleClass::Gap]);
}

#[test]
fn the_overlap_fixture_shares_exactly_amount_100() {
    let table = fixture_presentation("overlap-disjoint.ct.json");
    let domain = int_domain(&table, "amount", &[50, 100, 200]);
    let analysis = analyze(&table, &domain);
    assert_eq!(
        analysis.overlaps,
        vec![RuleOverlap {
            left: 0,
            right: 1,
            shared: set(&domain, &[ints(100)]),
        }]
    );
    assert_eq!(analysis.class_summary(), vec![RuleClass::Overlap]);
}

#[test]
fn the_shadow_fixture_kills_the_even_rule() {
    let table = fixture_presentation("shadow.ct.json");
    let values: Vec<i64> = (1..=6).collect();
    let domain = int_domain(&table, "n", &values);
    let analysis = analyze(&table, &domain);
    assert_eq!(analysis.shadow, Some(vec![1]), "`even` is dead");
    assert_eq!(analysis.class_summary(), vec![RuleClass::Shadow]);
}

#[test]
fn the_derivable_fixture_combines_fizz_and_buzz() {
    let table = fixture_presentation("derivable.ct.json");
    let values: Vec<i64> = (1..=15).collect();
    let domain = int_domain(&table, "n", &values);
    let analysis = analyze(&table, &domain);
    assert_eq!(
        analysis.derivable,
        vec![RuleDerivation {
            rule: 0,
            factors: [1, 2],
        }]
    );
    assert_eq!(analysis.class_summary(), vec![RuleClass::Derivable]);
}
