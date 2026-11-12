//! Purpose: the rule lens's tiny decidable predicate language (format 1.2,
//! add-rule-lens phase A; design.md Decision 2) — the parser from the
//! section's kind-tagged JSON nodes into typed [`Predicate`]s.
//!
//! Responsibilities: compile `cmp` (typed comparison whose literal matches
//! its column's declared type, bool columns `eq`/`ne` only), `mod` (int
//! columns, positive modulus, non-negative remainder), `and`/`or`
//! (non-empty operand arrays), and `not` (one operand); resolve every
//! column reference against the declared alphabet; reject everything else
//! with a labeled `invalid_predicate`/`unknown_column` error at the
//! offending node.
//!
//! Rationale: the language is deliberately not a logic and not pluggable —
//! comparisons, ∧ ∨ ¬, mod is the whole grammar, and every extension must
//! re-justify decidability. Compilation is structural only: it validates
//! types and shape, never evaluates (enumeration is the analysis phase).

use super::reject;
use super::spans::Spans;
use crate::error::{ErrorCode, ParseError};
use crate::model::{CmpOp, ColumnType, Ident, Literal, Predicate};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Compile one predicate node. Kinds `cmp`, `mod`, `and`, `or`, `not` are
/// the whole language.
pub(super) fn compile_predicate(
    value: &Value,
    columns: &BTreeMap<Ident, ColumnType>,
    pointer: &str,
    rule: &str,
    spans: &Spans,
) -> Result<Predicate, ParseError> {
    let Some(object) = value.as_object() else {
        let message =
            format!("rule `{rule}`: a predicate must be an object with a `kind`, got `{value}`");
        return Err(invalid(spans, pointer, message));
    };
    let Some(kind) = object.get("kind").and_then(Value::as_str) else {
        let message =
            format!("rule `{rule}`: predicate is missing its string `kind` discriminator");
        let pointer = format!("{pointer}/kind");
        return Err(invalid(spans, &pointer, message));
    };
    match kind {
        "cmp" => compile_cmp(object, columns, pointer, rule, spans),
        "mod" => compile_mod(object, columns, pointer, rule, spans),
        "and" => Ok(Predicate::And(compile_args(
            object, columns, pointer, rule, spans,
        )?)),
        "or" => Ok(Predicate::Or(compile_args(
            object, columns, pointer, rule, spans,
        )?)),
        "not" => compile_not(object, columns, pointer, rule, spans),
        other => {
            let message = format!(
                "rule `{rule}`: unknown predicate kind `{other}`: expected `cmp`, `mod`, `and`, `or`, or `not`"
            );
            let pointer = format!("{pointer}/kind");
            Err(invalid(spans, &pointer, message))
        }
    }
}

/// `not`: exactly one operand.
fn compile_not(
    object: &Map<String, Value>,
    columns: &BTreeMap<Ident, ColumnType>,
    pointer: &str,
    rule: &str,
    spans: &Spans,
) -> Result<Predicate, ParseError> {
    let pointer = format!("{pointer}/arg");
    let Some(arg) = object.get("arg") else {
        let message = format!("rule `{rule}`: `not` needs an `arg` predicate");
        return Err(invalid(spans, &pointer, message));
    };
    let arg = compile_predicate(arg, columns, &pointer, rule, spans)?;
    Ok(Predicate::Not(Box::new(arg)))
}

/// A typed comparison: the literal matches the column type, and bool
/// columns admit only `eq`/`ne`.
fn compile_cmp(
    object: &Map<String, Value>,
    columns: &BTreeMap<Ident, ColumnType>,
    pointer: &str,
    rule: &str,
    spans: &Spans,
) -> Result<Predicate, ParseError> {
    let column = string_field(object, "column", pointer, rule, spans)?;
    let column_at = format!("{pointer}/column");
    let column_type = resolve_column(column, columns, &column_at, rule, spans)?;
    let op_text = string_field(object, "op", pointer, rule, spans)?;
    let Some(op) = CmpOp::parse(op_text) else {
        let message = format!(
            "rule `{rule}`: unknown comparison operator `{op_text}`: expected one of eq, ne, lt, le, gt, ge"
        );
        let pointer = format!("{pointer}/op");
        return Err(invalid(spans, &pointer, message));
    };
    if column_type == ColumnType::Bool && !matches!(op, CmpOp::Eq | CmpOp::Ne) {
        let message = format!(
            "rule `{rule}`: bool column `{column}` admits only `eq` and `ne`, not `{}`",
            op.as_str()
        );
        let pointer = format!("{pointer}/op");
        return Err(invalid(spans, &pointer, message));
    }
    let Some(value) = object.get("value") else {
        let message = format!("rule `{rule}`: comparison on `{column}` needs a `value` literal");
        let pointer = format!("{pointer}/value");
        return Err(invalid(spans, &pointer, message));
    };
    let value_at = format!("{pointer}/value");
    let value = literal_for(column_type, value, column, rule, &value_at, spans)?;
    Ok(Predicate::Cmp {
        column: Ident(column.to_string()),
        op,
        value,
    })
}

/// An integer modulus test: int columns only, positive modulus.
fn compile_mod(
    object: &Map<String, Value>,
    columns: &BTreeMap<Ident, ColumnType>,
    pointer: &str,
    rule: &str,
    spans: &Spans,
) -> Result<Predicate, ParseError> {
    let column = string_field(object, "column", pointer, rule, spans)?;
    let column_at = format!("{pointer}/column");
    let column_type = resolve_column(column, columns, &column_at, rule, spans)?;
    if column_type != ColumnType::Int {
        let message = format!(
            "rule `{rule}`: `mod` applies to int columns only; `{column}` is {}",
            column_type.as_str()
        );
        return Err(invalid(spans, &column_at, message));
    }
    let Some(modulus) = object.get("modulus").and_then(Value::as_i64) else {
        let message = format!("rule `{rule}`: `mod` needs an integer `modulus`");
        let pointer = format!("{pointer}/modulus");
        return Err(invalid(spans, &pointer, message));
    };
    if modulus <= 0 {
        let message = format!("rule `{rule}`: `modulus` must be a positive integer, got {modulus}");
        let pointer = format!("{pointer}/modulus");
        return Err(invalid(spans, &pointer, message));
    }
    let Some(equals) = object.get("equals").and_then(Value::as_i64) else {
        let message = format!("rule `{rule}`: `mod` needs an integer `equals`");
        let pointer = format!("{pointer}/equals");
        return Err(invalid(spans, &pointer, message));
    };
    Ok(Predicate::Mod {
        column: Ident(column.to_string()),
        modulus,
        equals,
    })
}

/// A boolean combination's operands: a non-empty array of predicates.
fn compile_args(
    object: &Map<String, Value>,
    columns: &BTreeMap<Ident, ColumnType>,
    pointer: &str,
    rule: &str,
    spans: &Spans,
) -> Result<Vec<Predicate>, ParseError> {
    let args = match object.get("args") {
        Some(Value::Array(args)) if !args.is_empty() => args,
        _ => {
            let message =
                format!("rule `{rule}`: a boolean combination needs a non-empty `args` array");
            let pointer = format!("{pointer}/args");
            return Err(invalid(spans, &pointer, message));
        }
    };
    args.iter()
        .enumerate()
        .map(|(k, arg)| {
            compile_predicate(arg, columns, &format!("{pointer}/args/{k}"), rule, spans)
        })
        .collect()
}

/// A required string field of a predicate node.
fn string_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    pointer: &str,
    rule: &str,
    spans: &Spans,
) -> Result<&'a str, ParseError> {
    let Some(text) = object.get(field).and_then(Value::as_str) else {
        let message = format!("rule `{rule}`: predicate needs a string `{field}`");
        let pointer = format!("{pointer}/{field}");
        return Err(invalid(spans, &pointer, message));
    };
    Ok(text)
}

/// Resolve a predicate's column against the declared attribute alphabet.
fn resolve_column(
    name: &str,
    columns: &BTreeMap<Ident, ColumnType>,
    pointer: &str,
    rule: &str,
    spans: &Spans,
) -> Result<ColumnType, ParseError> {
    let Some(column_type) = columns.get(&Ident(name.to_string())) else {
        let message = format!("rule `{rule}`: predicate references undeclared column `{name}`");
        return Err(reject(spans, pointer, ErrorCode::UnknownColumn, message));
    };
    Ok(*column_type)
}

/// A comparison literal, typed by its column.
fn literal_for(
    column_type: ColumnType,
    value: &Value,
    column: &str,
    rule: &str,
    pointer: &str,
    spans: &Spans,
) -> Result<Literal, ParseError> {
    let (literal, expected) = match column_type {
        ColumnType::Int => (value.as_i64().map(Literal::Int), "an integer"),
        ColumnType::Bool => (value.as_bool().map(Literal::Bool), "a boolean"),
        ColumnType::String => (
            value.as_str().map(|text| Literal::String(text.to_string())),
            "a string",
        ),
    };
    let Some(literal) = literal else {
        let message = format!(
            "rule `{rule}`: column `{column}` is {}; the comparison literal must be \
             {expected}, got `{value}`",
            column_type.as_str()
        );
        return Err(invalid(spans, pointer, message));
    };
    Ok(literal)
}

/// A labeled `invalid_predicate` rejection.
fn invalid(spans: &Spans, pointer: &str, message: String) -> ParseError {
    reject(spans, pointer, ErrorCode::InvalidPredicate, message)
}
