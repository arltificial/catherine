//! Purpose: the canonical predicate serialization (additive 1.2, openspec
//! change `add-rules-projection`) — the rule presentation's predicate
//! nodes, comparison literals, and integers. Split out of `value.rs` so
//! the canonical serializer stays inside the repo's file-size gate.
//!
//! Responsibilities: emit each predicate in the format's kind-tagged
//! shape (`{"kind": …}` with column/op/value spellings), `and`/`or`
//! operand arrays in written order, and base-ten integers without
//! exponent or fraction.
//!
//! Rationale: the rules' written order is semantic (never sorted — the
//! `rules` bridge reads it back); only the nodes' own spellings are
//! canonical here, exactly as `value.rs` emitted them before the split.

use crate::model::{Literal, Predicate};
use serde_json::Value;

use super::value::{obj, string};

/// One canonical predicate node, in the format's kind-tagged shape.
pub(super) fn predicate_value(predicate: &Predicate) -> Value {
    match predicate {
        Predicate::Cmp { column, op, value } => obj([
            ("kind", string("cmp")),
            ("column", string(&column.0)),
            ("op", string(op.as_str())),
            ("value", literal_value(value)),
        ]),
        Predicate::Mod {
            column,
            modulus,
            equals,
        } => obj([
            ("kind", string("mod")),
            ("column", string(&column.0)),
            ("modulus", number(*modulus)),
            ("equals", number(*equals)),
        ]),
        Predicate::And(args) => boolean_combination("and", args),
        Predicate::Or(args) => boolean_combination("or", args),
        Predicate::Not(arg) => obj([("kind", string("not")), ("arg", predicate_value(arg))]),
    }
}

/// A canonical `and`/`or` node: the kind plus its operand array in written
/// order.
fn boolean_combination(kind: &str, args: &[Predicate]) -> Value {
    obj([
        ("kind", string(kind)),
        (
            "args",
            Value::Array(args.iter().map(predicate_value).collect()),
        ),
    ])
}

/// One canonical comparison literal.
fn literal_value(literal: &Literal) -> Value {
    match literal {
        Literal::Int(value) => number(*value),
        Literal::Bool(value) => Value::Bool(*value),
        Literal::String(value) => string(value),
    }
}

/// A canonical integer: base-ten, no exponent or fraction.
fn number(value: i64) -> Value {
    Value::Number(value.into())
}
