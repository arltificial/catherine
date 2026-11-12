//! Purpose: sim/kernel parity for the rule lens (openspec change
//! `add-rule-lens` phase B, task 5; design.md Decision 5) — every analysis
//! case in `prototype/rule_lens_sim.py` reproduced as a Rust test against
//! the same hand-computed expectations.
//!
//! Responsibilities: the seven oracle analysis cases — gap, overlap under
//! `DeclaredDisjoint`, priority-resolved overlap, shadow, FizzBuzz
//! derivability, bool/string typing, and the no-policy overlap
//! diagnostic — asserting regions, effective regions, gap, overlaps,
//! shadow, derivable, and the four-class summary exactly as the oracle
//! does. The oracle is the authority: a divergence here is a kernel bug.
//!
//! Rationale: the sim is the self-checking semantics oracle; porting its
//! tables verbatim (same domains, same rule order, same expectations)
//! makes any semantic drift fail loudly instead of living in prose.

mod common;

use catherine_core::model::{CmpOp, ColumnType, Policy};
use catherine_core::rule_lens::{analyze, RuleClass, RuleDerivation, RuleOverlap};
use common::rule_lens::{
    boolean, cmp, conjunction, domain, ident, int_domain, integer, ints, modulus, negation,
    presentation, rule, set, text,
};

fn ints_of(values: &[i64]) -> Vec<Vec<catherine_core::model::Literal>> {
    values.iter().map(|&value| ints(value)).collect()
}

/// `gap_age`: R_kid = {10}, R_adult = {20,30}, X = {10,20,30,70}.
#[test]
fn gap_age_matches_the_oracle() {
    let table = presentation(
        Some(Policy::PriorityOrder),
        &[("age", ColumnType::Int)],
        vec![
            rule("kid", cmp("age", CmpOp::Lt, integer(18)), "kid"),
            rule(
                "adult",
                conjunction(vec![
                    cmp("age", CmpOp::Ge, integer(18)),
                    cmp("age", CmpOp::Lt, integer(65)),
                ]),
                "adult",
            ),
        ],
    );
    let domain = int_domain(&table, "age", &[10, 20, 30, 70]);
    let analysis = analyze(&table, &domain);
    let regions = vec![
        set(&domain, &[ints(10)]),
        set(&domain, &[ints(20), ints(30)]),
    ];
    assert_eq!(analysis.regions, regions, "R_kid, R_adult");
    assert_eq!(analysis.effective, Some(regions), "no intersection here");
    assert_eq!(analysis.gap, set(&domain, &[ints(70)]), "70 uncovered");
    assert!(
        analysis.overlaps.is_empty(),
        "priority resolves nothing here"
    );
    assert_eq!(analysis.shadow, Some(vec![]), "both rules survive");
    assert!(analysis.derivable.is_empty());
    assert_eq!(analysis.class_summary(), vec![RuleClass::Gap]);
}

/// `overlap_disjoint`: R_small = {50,100}, R_large = {100,200}, shared {100}.
#[test]
fn overlap_under_declared_disjoint_matches_the_oracle() {
    let table = presentation(
        Some(Policy::DeclaredDisjoint),
        &[("amount", ColumnType::Int)],
        vec![
            rule("small", cmp("amount", CmpOp::Le, integer(100)), "small"),
            rule("large", cmp("amount", CmpOp::Ge, integer(100)), "large"),
        ],
    );
    let domain = int_domain(&table, "amount", &[50, 100, 200]);
    let analysis = analyze(&table, &domain);
    let regions = vec![
        set(&domain, &[ints(50), ints(100)]),
        set(&domain, &[ints(100), ints(200)]),
    ];
    assert_eq!(analysis.regions, regions);
    assert_eq!(
        analysis.effective,
        Some(regions),
        "declared-disjoint keeps R_i"
    );
    assert!(analysis.gap.is_empty());
    assert_eq!(
        analysis.overlaps,
        vec![RuleOverlap {
            left: 0,
            right: 1,
            shared: set(&domain, &[ints(100)]),
        }]
    );
    assert_eq!(analysis.shadow, Some(vec![]));
    assert!(analysis.derivable.is_empty());
    assert_eq!(analysis.class_summary(), vec![RuleClass::Overlap]);
}

/// `overlap_priority_resolves`: same regions, order gives 100 to `small`
/// and E_large = {200} — legal by construction, no finding class.
#[test]
fn priority_resolved_overlap_matches_the_oracle() {
    let table = presentation(
        Some(Policy::PriorityOrder),
        &[("amount", ColumnType::Int)],
        vec![
            rule("small", cmp("amount", CmpOp::Le, integer(100)), "small"),
            rule("large", cmp("amount", CmpOp::Ge, integer(100)), "large"),
        ],
    );
    let domain = int_domain(&table, "amount", &[50, 100, 200]);
    let analysis = analyze(&table, &domain);
    assert_eq!(
        analysis.effective,
        Some(vec![
            set(&domain, &[ints(50), ints(100)]),
            set(&domain, &[ints(200)]),
        ])
    );
    assert!(analysis.gap.is_empty());
    assert!(
        analysis.overlaps.is_empty(),
        "order resolves the intersection"
    );
    assert_eq!(analysis.shadow, Some(vec![]));
    assert!(analysis.derivable.is_empty());
    assert!(analysis.class_summary().is_empty());
}

/// `shadow_first_match`: R_all = X, so E_even = ∅.
#[test]
fn shadow_first_match_matches_the_oracle() {
    let table = presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        vec![
            rule("all", cmp("n", CmpOp::Ge, integer(1)), "default"),
            rule("even", modulus("n", 2, 0), "even"),
        ],
    );
    let domain = int_domain(&table, "n", &[1, 2, 3, 4, 5, 6]);
    let analysis = analyze(&table, &domain);
    assert_eq!(
        analysis.effective,
        Some(vec![
            set(&domain, &ints_of(&[1, 2, 3, 4, 5, 6])),
            set(&domain, &[]),
        ])
    );
    assert!(analysis.gap.is_empty());
    assert!(analysis.overlaps.is_empty());
    assert_eq!(analysis.shadow, Some(vec![1]), "even is dead");
    assert!(analysis.derivable.is_empty());
    assert_eq!(analysis.class_summary(), vec![RuleClass::Shadow]);
}

fn fizzbuzz_table() -> catherine_core::model::RulePresentation {
    presentation(
        Some(Policy::FirstMatch),
        &[("n", ColumnType::Int)],
        vec![
            rule("fizzbuzz", modulus("n", 15, 0), "FizzBuzz"),
            rule("fizz", modulus("n", 3, 0), "Fizz"),
            rule("buzz", modulus("n", 5, 0), "Buzz"),
            rule("plain", cmp("n", CmpOp::Ge, integer(1)), "n"),
        ],
    )
}

fn fifteen() -> catherine_core::rule_lens::RuleDomain {
    let table = fizzbuzz_table();
    int_domain(&table, "n", &(1..=15).collect::<Vec<_>>())
}

/// `derivable_fizzbuzz`: R_fizzbuzz = {15} = R_fizz ∩ R_buzz.
#[test]
fn derivable_fizzbuzz_matches_the_oracle() {
    let table = fizzbuzz_table();
    let domain = fifteen();
    let analysis = analyze(&table, &domain);
    assert_eq!(
        analysis.regions,
        vec![
            set(&domain, &ints_of(&[15])),
            set(&domain, &ints_of(&[3, 6, 9, 12, 15])),
            set(&domain, &ints_of(&[5, 10, 15])),
            set(
                &domain,
                &ints_of(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
            ),
        ]
    );
    assert_eq!(
        analysis.effective,
        Some(vec![
            set(&domain, &ints_of(&[15])),
            set(&domain, &ints_of(&[3, 6, 9, 12])),
            set(&domain, &ints_of(&[5, 10])),
            set(&domain, &ints_of(&[1, 2, 4, 7, 8, 11, 13, 14])),
        ])
    );
    assert!(analysis.gap.is_empty());
    assert!(analysis.overlaps.is_empty());
    assert_eq!(analysis.shadow, Some(vec![]));
    assert_eq!(
        analysis.derivable,
        vec![RuleDerivation {
            rule: 0,
            factors: [1, 2],
        }]
    );
    assert_eq!(analysis.class_summary(), vec![RuleClass::Derivable]);
}

fn bool_string_table() -> catherine_core::model::RulePresentation {
    presentation(
        Some(Policy::PriorityOrder),
        &[("member", ColumnType::Bool), ("tier", ColumnType::String)],
        vec![
            rule(
                "gold",
                conjunction(vec![
                    cmp("member", CmpOp::Eq, boolean(true)),
                    cmp("tier", CmpOp::Eq, text("gold")),
                ]),
                "gold",
            ),
            rule("basic", cmp("tier", CmpOp::Lt, text("gold")), "basic"),
            rule(
                "guest",
                negation(cmp("member", CmpOp::Eq, boolean(true))),
                "guest",
            ),
        ],
    )
}

fn bool_string_domain() -> catherine_core::rule_lens::RuleDomain {
    domain(
        &bool_string_table(),
        &[
            ("member", vec![boolean(true), boolean(false)]),
            ("tier", vec![text("basic"), text("gold")]),
        ],
    )
}

fn member_tier(member: bool, tier: &str) -> Vec<catherine_core::model::Literal> {
    vec![boolean(member), text(tier)]
}

/// `types_bool_string`: bool ordering `false < true`, strings byte-wise
/// (`"basic" < "gold"`).
#[test]
fn bool_and_string_regions_match_the_oracle() {
    let table = bool_string_table();
    let domain = bool_string_domain();
    assert_eq!(
        analysis_regions(&table, &domain),
        vec![
            set(&domain, &[member_tier(true, "gold")]),
            set(
                &domain,
                &[member_tier(true, "basic"), member_tier(false, "basic")]
            ),
            set(
                &domain,
                &[member_tier(false, "basic"), member_tier(false, "gold")]
            ),
        ]
    );
    let analysis = analyze(&table, &domain);
    assert!(analysis.gap.is_empty());
    assert!(analysis.overlaps.is_empty());
    assert_eq!(analysis.shadow, Some(vec![]));
    assert!(analysis.derivable.is_empty());
    assert!(analysis.class_summary().is_empty());
}

#[test]
fn bool_and_string_effective_regions_match_the_oracle() {
    let table = bool_string_table();
    let domain = bool_string_domain();
    let analysis = analyze(&table, &domain);
    assert_eq!(
        analysis.effective,
        Some(vec![
            set(&domain, &[member_tier(true, "gold")]),
            set(
                &domain,
                &[member_tier(true, "basic"), member_tier(false, "basic")]
            ),
            set(&domain, &[member_tier(false, "gold")]),
        ])
    );
}

/// `no_policy_overlap_diagnostic`: overlaps at 20 and 70, no effective
/// regions, no shadow — the diagnostic stays unmapped (Decision 3).
#[test]
fn no_policy_overlap_diagnostic_matches_the_oracle() {
    let table = presentation(
        None,
        &[("age", ColumnType::Int)],
        vec![
            rule("young", cmp("age", CmpOp::Le, integer(20)), "young"),
            rule("old", cmp("age", CmpOp::Gt, integer(20)), "old"),
            rule("vip", cmp("age", CmpOp::Ge, integer(20)), "vip"),
        ],
    );
    let domain = int_domain(&table, "age", &[10, 20, 70]);
    let analysis = analyze(&table, &domain);
    assert_eq!(analysis.effective, None, "no policy means no precedence");
    assert!(analysis.gap.is_empty());
    assert_eq!(
        analysis.overlaps,
        vec![
            RuleOverlap {
                left: 0,
                right: 2,
                shared: set(&domain, &[ints(20)]),
            },
            RuleOverlap {
                left: 1,
                right: 2,
                shared: set(&domain, &[ints(70)]),
            },
        ]
    );
    assert_eq!(analysis.shadow, None, "no effective regions to be empty");
    assert!(analysis.derivable.is_empty());
    assert_eq!(analysis.class_summary(), vec![RuleClass::Overlap]);
}

/// The oracle's `mod` is a non-negative remainder: `-1 mod 3 == 2`.
#[test]
fn modulo_uses_a_non_negative_remainder() {
    let table = presentation(
        Some(Policy::PriorityOrder),
        &[("n", ColumnType::Int)],
        vec![rule("two", modulus("n", 3, 2), "two")],
    );
    let domain = int_domain(&table, "n", &[-1, 0, 1, 2, 3]);
    assert_eq!(
        analyze(&table, &domain).regions,
        vec![set(&domain, &[ints(-1), ints(2)])]
    );
}

/// The typed columns the oracle's `AGE`/`SIX`/`FIFTEEN` cases need.
fn analysis_regions(
    table: &catherine_core::model::RulePresentation,
    domain: &catherine_core::rule_lens::RuleDomain,
) -> Vec<std::collections::BTreeSet<usize>> {
    analyze(table, domain).regions
}

#[test]
fn the_domain_builder_rejects_a_literal_of_the_wrong_type() {
    let table = presentation(
        Some(Policy::PriorityOrder),
        &[("n", ColumnType::Int)],
        vec![rule("all", cmp("n", CmpOp::Ge, integer(1)), "all")],
    );
    let supplied = [(ident("n"), vec![text("nope")])].into_iter().collect();
    assert!(matches!(
        catherine_core::rule_lens::RuleDomain::from_presentation(&table, supplied),
        Err(catherine_core::rule_lens::RuleDomainError::WrongType { .. })
    ));
}
