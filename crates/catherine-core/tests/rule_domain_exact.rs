//! Purpose: the exact-input rule domain, exercised through the public
//! kernel surface (`RuleDomain::from_tuples` + `inputs`/`column_sizes`/
//! `evaluate` and the bounded analysis) — openspec change
//! `add-rules-projection`, kernel seam 2. The product-domain tests stay
//! in `rule_lens::domain`; the file split keeps both under the repo's
//! file-size gate.
//!
//! Responsibilities: pin that an exact set is stored as given (never
//! expanded to a product), that its budget expiry reports the explored
//! count, that width/type mismatches are rejected with the offending
//! column, and that predicates evaluate against exact inputs by position.
//!
//! Rationale: a projection's domain is enumerated data, not a product —
//! the failure this guards is an invented combination being reported as a
//! gap that no row has.

use catherine_core::model::{
    CmpOp, ColumnType, Ident, Literal, Policy, Predicate, RulePresentation,
};
use catherine_core::rule_lens::{
    analyze_with_bounds, evaluate, RuleDomain, RuleDomainError, RuleInput, RuleOutcome,
};
use std::collections::BTreeMap;

/// Two typed columns, sorted by name (`age` < `name`) — the exact-set
/// tests' presentation.
fn presentation() -> RulePresentation {
    RulePresentation {
        domain: Ident("Input".to_string()),
        columns: BTreeMap::from([
            (Ident("age".to_string()), ColumnType::Int),
            (Ident("name".to_string()), ColumnType::String),
        ]),
        rules: Vec::new(),
        policy: Some(Policy::DeclaredDisjoint),
    }
}

fn tuple(age: i64, name: &str) -> RuleInput {
    vec![Literal::Int(age), Literal::String(name.to_string())]
}

/// The exact set is stored and enumerated as given, never expanded to a
/// product; the budget expiry reports the count it could not finish.
#[test]
fn exact_tuples_enumerate_in_the_order_given() {
    let inputs = vec![tuple(2, "b"), tuple(1, "a")];
    let domain = RuleDomain::from_tuples(&presentation(), inputs.clone()).expect("valid set");
    assert_eq!(domain.inputs(), inputs, "the exact set, as supplied");
    assert_eq!(
        domain.column_sizes(),
        BTreeMap::from([
            (Ident("age".to_string()), 2),
            (Ident("name".to_string()), 2),
        ]),
        "an exact set reports its distinct values per column"
    );
    let expiring = analyze_with_bounds(&presentation(), &domain, 1);
    match &expiring {
        RuleOutcome::Unknown { scope } => assert_eq!(
            scope.candidates_explored, 1,
            "a one-candidate budget expires after exactly one"
        ),
        other => panic!("expected a budget expiry, got {other:?}"),
    }
}

/// A literal of the wrong variant is rejected with its column and the
/// expected type — the domain never guesses.
#[test]
fn an_exact_tuple_of_the_wrong_type_is_rejected() {
    let inputs = vec![vec![
        Literal::String("1".to_string()),
        Literal::String("a".to_string()),
    ]];
    assert_eq!(
        RuleDomain::from_tuples(&presentation(), inputs),
        Err(RuleDomainError::WrongType {
            column: Ident("age".to_string()),
            expected: ColumnType::Int,
        })
    );
}

/// An exact tuple carries one value per declared column only.
#[test]
fn an_exact_tuple_of_the_wrong_width_is_rejected() {
    assert_eq!(
        RuleDomain::from_tuples(&presentation(), vec![tuple(1, "a"), vec![Literal::Int(1)]]),
        Err(RuleDomainError::WrongWidth {
            expected: 2,
            found: 1,
        })
    );
}

/// Predicate evaluation reads exact inputs through the same column
/// positions as a product domain.
#[test]
fn exact_tuples_evaluate_predicates_by_position() {
    let inputs = vec![tuple(1, "a"), tuple(2, "b")];
    let domain = RuleDomain::from_tuples(&presentation(), inputs).expect("valid set");
    let predicate = Predicate::Cmp {
        column: Ident("age".to_string()),
        op: CmpOp::Eq,
        value: Literal::Int(2),
    };
    assert!(!evaluate(&predicate, &tuple(1, "a"), &domain));
    assert!(evaluate(&predicate, &tuple(2, "b"), &domain));
}
