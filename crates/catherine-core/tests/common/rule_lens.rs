//! Purpose: shared builders for the rule-lens semantics tests (phase B) —
//! one spelling of the oracle's tables and the committed fixtures, so the
//! parity and fixture binaries cannot drift from each other.
//!
//! Responsibilities: typed literal/predicate/rule builders mirroring
//! `prototype/rule_lens_sim.py`'s `cmp`/`mod`/`conj`/`neg`/`rule`, a
//! presentation builder, a domain builder validated through
//! `RuleDomain::from_presentation`, and index helpers that resolve an
//! expected input tuple back to its dense enumeration index.
//!
//! Rationale: the tests state the hand-computed oracle verbatim (sets of
//! input tuples) instead of pre-computed indices; resolving through the
//! domain keeps the assertions readable and makes an enumeration-order
//! bug fail loudly. Each integration-test binary compiles this module, so
//! unused builders stay dead code by design.

#![allow(dead_code)]

use catherine_core::model::{
    CmpOp, ColumnType, Ident, Literal, Policy, Predicate, Rule, RulePresentation,
};
use catherine_core::parse::parse;
use catherine_core::rule_lens::RuleDomain;
use std::collections::{BTreeMap, BTreeSet};

pub fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}

pub fn integer(value: i64) -> Literal {
    Literal::Int(value)
}

pub fn boolean(value: bool) -> Literal {
    Literal::Bool(value)
}

pub fn text(value: &str) -> Literal {
    Literal::String(value.to_string())
}

pub fn ints(value: i64) -> Vec<Literal> {
    vec![integer(value)]
}

pub fn bools(value: bool) -> Vec<Literal> {
    vec![boolean(value)]
}

pub fn cmp(column: &str, op: CmpOp, value: Literal) -> Predicate {
    Predicate::Cmp {
        column: ident(column),
        op,
        value,
    }
}

pub fn modulus(column: &str, modulus: i64, equals: i64) -> Predicate {
    Predicate::Mod {
        column: ident(column),
        modulus,
        equals,
    }
}

pub fn conjunction(args: Vec<Predicate>) -> Predicate {
    Predicate::And(args)
}

pub fn negation(arg: Predicate) -> Predicate {
    Predicate::Not(Box::new(arg))
}

pub fn rule(name: &str, predicate: Predicate, action: &str) -> Rule {
    Rule {
        name: ident(name),
        predicate,
        action: action.to_string(),
    }
}

pub fn presentation(
    policy: Option<Policy>,
    columns: &[(&str, ColumnType)],
    rules: Vec<Rule>,
) -> RulePresentation {
    RulePresentation {
        domain: ident("Input"),
        columns: columns
            .iter()
            .map(|&(name, column_type)| (ident(name), column_type))
            .collect(),
        rules,
        policy,
    }
}

pub fn domain(presentation: &RulePresentation, values: &[(&str, Vec<Literal>)]) -> RuleDomain {
    let supplied: BTreeMap<Ident, Vec<Literal>> = values
        .iter()
        .map(|(name, values)| (ident(name), values.clone()))
        .collect();
    RuleDomain::from_presentation(presentation, supplied).expect("domain matches the presentation")
}

/// A one-int-column domain with the given values.
pub fn int_domain(presentation: &RulePresentation, column: &str, values: &[i64]) -> RuleDomain {
    let values: Vec<Literal> = values.iter().copied().map(integer).collect();
    domain(presentation, &[(column, values)])
}

/// The dense enumeration index of an expected input tuple.
pub fn position(domain: &RuleDomain, input: &[Literal]) -> usize {
    domain
        .inputs()
        .iter()
        .position(|candidate| candidate == input)
        .unwrap_or_else(|| panic!("input {input:?} is in the declared domain"))
}

/// Resolve expected input tuples to their dense enumeration indices.
pub fn set(domain: &RuleDomain, inputs: &[Vec<Literal>]) -> BTreeSet<usize> {
    inputs.iter().map(|input| position(domain, input)).collect()
}

/// Read back a set of dense indices as the input tuples they denote.
pub fn inputs_at(domain: &RuleDomain, indices: &BTreeSet<usize>) -> Vec<Vec<Literal>> {
    let all = domain.inputs();
    indices.iter().map(|&index| all[index].clone()).collect()
}

pub fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../fixtures/rule_lens/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(path).expect("golden fixture exists")
}

/// The parsed `rule_presentation` of a committed golden fixture.
pub fn fixture_presentation(name: &str) -> RulePresentation {
    let model = parse(&fixture(name)).expect("golden fixture parses");
    model
        .rule_presentation
        .expect("the fixture declares a rule presentation")
}
