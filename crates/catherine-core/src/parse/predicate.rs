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
use super::spans::{Defect, Spans};
use crate::error::{ErrorCode, ParseError};
use crate::model::{CmpOp, ColumnType, Ident, Literal, Predicate};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// The compile context shared by one rule's predicates: the declared
/// column alphabet, the owning rule's name for messages, and the span
/// source. Node pointers stay per-call, since compilation descends into
/// child nodes.
pub(super) struct PredicateCtx<'a> {
    columns: &'a BTreeMap<Ident, ColumnType>,
    rule: &'a str,
    spans: &'a Spans,
}

/// One predicate column: its name, declared type, and the JSON pointer of
/// the `column` field that named it (carrying the type error).
struct ColumnRef {
    name: String,
    kind: ColumnType,
    pointer: String,
}

/// One predicate node under compilation: its fields and its JSON pointer.
/// Every compile helper takes the node as one bundle, so the pointer and
/// the object it indexes can never drift apart.
struct Node<'a> {
    object: &'a Map<String, Value>,
    pointer: &'a str,
}

impl<'a> Node<'a> {
    fn of(object: &'a Map<String, Value>, pointer: &'a str) -> Self {
        Self { object, pointer }
    }

    /// A child field's pointer.
    fn at(&self, field: &str) -> String {
        format!("{}/{field}", self.pointer)
    }
}

/// An unknown predicate field: the kind that refuses it, the first
/// offending key, and the kind's allowed fields.
struct Offense<'a> {
    kind: &'a str,
    first: &'a str,
    allowed: &'a [&'a str],
}

impl<'a> PredicateCtx<'a> {
    pub(super) fn new(
        columns: &'a BTreeMap<Ident, ColumnType>,
        rule: &'a str,
        spans: &'a Spans,
    ) -> Self {
        Self {
            columns,
            rule,
            spans,
        }
    }

    /// A labeled `invalid_predicate` rejection.
    fn invalid(&self, pointer: &str, message: String) -> ParseError {
        reject(self.spans, pointer, ErrorCode::InvalidPredicate, message)
    }

    /// A required string field of a predicate node.
    fn string_field<'o>(&self, node: &Node<'o>, field: &str) -> Result<&'o str, ParseError> {
        let Some(text) = node.object.get(field).and_then(Value::as_str) else {
            let message = format!("rule `{}`: predicate needs a string `{field}`", self.rule);
            let pointer = node.at(field);
            return Err(self.invalid(&pointer, message));
        };
        Ok(text)
    }

    /// Resolve a predicate's column against the declared attribute
    /// alphabet.
    fn resolve_column(&self, name: &str, pointer: &str) -> Result<ColumnRef, ParseError> {
        let Some(&column_type) = self.columns.get(&Ident(name.to_string())) else {
            let message = format!(
                "rule `{}`: predicate references undeclared column `{name}`",
                self.rule
            );
            return Err(reject(
                self.spans,
                pointer,
                ErrorCode::UnknownColumn,
                message,
            ));
        };
        Ok(ColumnRef {
            name: name.to_string(),
            kind: column_type,
            pointer: pointer.to_string(),
        })
    }

    /// A predicate's `column` field: a required string naming a declared
    /// column.
    fn typed_column(&self, node: &Node) -> Result<ColumnRef, ParseError> {
        let name = self.string_field(node, "column")?;
        let at = node.at("column");
        self.resolve_column(name, &at)
    }

    /// A comparison literal, typed by its column: the node's `value`
    /// field, which must be present and of the column's type.
    fn literal_for(&self, node: &Node, column: &ColumnRef) -> Result<Literal, ParseError> {
        let at = node.at("value");
        let Some(value) = node.object.get("value") else {
            let message = format!(
                "rule `{}`: comparison on `{}` needs a `value` literal",
                self.rule, column.name
            );
            return Err(self.invalid(&at, message));
        };
        let expected = literal_spelling(column.kind);
        let Some(literal) = typed_literal(column.kind, value) else {
            let message = format!(
                "rule `{}`: column `{}` is {}; the comparison literal must be \
                 {expected}, got `{value}`",
                self.rule,
                column.name,
                column.kind.as_str()
            );
            return Err(self.invalid(&at, message));
        };
        Ok(literal)
    }

    /// A comparison's operator: a known spelling, and for bool columns
    /// only `eq`/`ne`. The `op` node carries each defect.
    fn cmp_op(&self, node: &Node, column: &ColumnRef) -> Result<CmpOp, ParseError> {
        let op_text = self.string_field(node, "op")?;
        let Some(op) = CmpOp::parse(op_text) else {
            let message = format!(
                "rule `{}`: unknown comparison operator `{op_text}`: expected one of eq, ne, lt, le, gt, ge",
                self.rule
            );
            return Err(self.invalid(&node.at("op"), message));
        };
        if column.kind == ColumnType::Bool && !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            let message = format!(
                "rule `{}`: bool column `{}` admits only `eq` and `ne`, not `{}`",
                self.rule,
                column.name,
                op.as_str()
            );
            return Err(self.invalid(&node.at("op"), message));
        }
        Ok(op)
    }

    /// Refuse a predicate field its kind does not define, at the first
    /// offending key (a `BTreeMap` walk is alphabetical, so the choice
    /// is deterministic).
    fn reject_unknown_fields(&self, node: &Node, kind: &str) -> Result<(), ParseError> {
        let Some(allowed) = allowed_fields(kind) else {
            return Ok(()); // unknown `kind` is the stronger error below
        };
        let unknown: Vec<&str> = node
            .object
            .keys()
            .map(String::as_str)
            .filter(|key| !allowed.contains(key))
            .collect();
        let Some(first) = unknown.first() else {
            return Ok(());
        };
        let offense = Offense {
            kind,
            first,
            allowed,
        };
        Err(self.unknown_field(node, &offense))
    }

    /// The labeled `unknown_field` at the first offending key, with the
    /// kind's allowed fields spelled out.
    fn unknown_field(&self, node: &Node, offense: &Offense<'_>) -> ParseError {
        let Offense {
            kind,
            first,
            allowed,
        } = offense;
        let message = format!(
            "rule `{}`: predicate kind `{kind}` does not define field `{first}`; allowed fields: {}",
            self.rule,
            allowed
                .iter()
                .map(|field| format!("`{field}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        self.spans.semantic_key(
            (node.pointer, first),
            Defect {
                code: ErrorCode::UnknownField,
                message,
            },
        )
    }

    /// A required integer field of a `mod` node.
    fn int_field(&self, node: &Node, field: &str) -> Result<i64, ParseError> {
        node.object
            .get(field)
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                let message = format!("rule `{}`: `mod` needs an integer `{field}`", self.rule);
                let pointer = node.at(field);
                self.invalid(&pointer, message)
            })
    }
}

/// A JSON value as a literal of the given column type; `None` when it
/// does not match.
fn typed_literal(kind: ColumnType, value: &Value) -> Option<Literal> {
    Some(match kind {
        ColumnType::Int => Literal::Int(value.as_i64()?),
        ColumnType::Bool => Literal::Bool(value.as_bool()?),
        ColumnType::String => Literal::String(value.as_str()?.to_string()),
    })
}

/// The expected-spelling message fragment for a column type: what the
/// comparison literal must be.
fn literal_spelling(kind: ColumnType) -> &'static str {
    match kind {
        ColumnType::Int => "an integer",
        ColumnType::Bool => "a boolean",
        ColumnType::String => "a string",
    }
}

/// Compile one predicate node. Kinds `cmp`, `mod`, `and`, `or`, `not` are
/// the whole language.
pub(super) fn compile_predicate(
    value: &Value,
    pointer: &str,
    ctx: &PredicateCtx,
) -> Result<Predicate, ParseError> {
    let Some(object) = value.as_object() else {
        let message = format!(
            "rule `{}`: a predicate must be an object with a `kind`, got `{value}`",
            ctx.rule
        );
        return Err(ctx.invalid(pointer, message));
    };
    let kind = predicate_kind(object, pointer, ctx)?;
    let node = Node::of(object, pointer);
    ctx.reject_unknown_fields(&node, kind)?;
    match kind {
        "cmp" => compile_cmp(&node, ctx),
        "mod" => compile_mod(&node, ctx),
        "and" => Ok(Predicate::And(compile_args(&node, ctx)?)),
        "or" => Ok(Predicate::Or(compile_args(&node, ctx)?)),
        "not" => compile_not(&node, ctx),
        other => unknown_kind(other, pointer, ctx),
    }
}

/// A predicate object's required string `kind` discriminator.
fn predicate_kind<'a>(
    object: &'a serde_json::Map<String, Value>,
    pointer: &str,
    ctx: &PredicateCtx,
) -> Result<&'a str, ParseError> {
    let Some(kind) = object.get("kind").and_then(Value::as_str) else {
        let message = format!(
            "rule `{}`: predicate is missing its string `kind` discriminator",
            ctx.rule
        );
        let pointer = format!("{pointer}/kind");
        return Err(ctx.invalid(&pointer, message));
    };
    Ok(kind)
}

/// The fields each predicate kind defines. A field outside this set is
/// `unknown_field`, never silently dropped — a mistyped field would
/// otherwise compile into a predicate the author did not write.
fn allowed_fields(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "cmp" => Some(&["kind", "column", "op", "value"]),
        "mod" => Some(&["kind", "column", "modulus", "equals"]),
        "and" | "or" => Some(&["kind", "args"]),
        "not" => Some(&["kind", "arg"]),
        _ => None,
    }
}

/// The labeled rejection of a kind outside the language, at the `kind`
/// node.
fn unknown_kind(kind: &str, pointer: &str, ctx: &PredicateCtx) -> Result<Predicate, ParseError> {
    let message = format!(
        "rule `{}`: unknown predicate kind `{kind}`: expected `cmp`, `mod`, `and`, `or`, or `not`",
        ctx.rule
    );
    let pointer = format!("{pointer}/kind");
    Err(ctx.invalid(&pointer, message))
}

/// `not`: exactly one operand.
fn compile_not(node: &Node, ctx: &PredicateCtx) -> Result<Predicate, ParseError> {
    let pointer = node.at("arg");
    let Some(arg) = node.object.get("arg") else {
        let message = format!("rule `{}`: `not` needs an `arg` predicate", ctx.rule);
        return Err(ctx.invalid(&pointer, message));
    };
    let arg = compile_predicate(arg, &pointer, ctx)?;
    Ok(Predicate::Not(Box::new(arg)))
}

/// A typed comparison: the literal matches the column type, and bool
/// columns admit only `eq`/`ne`.
fn compile_cmp(node: &Node, ctx: &PredicateCtx) -> Result<Predicate, ParseError> {
    let column = ctx.typed_column(node)?;
    let op = ctx.cmp_op(node, &column)?;
    let value = ctx.literal_for(node, &column)?;
    Ok(Predicate::Cmp {
        column: Ident(column.name.clone()),
        op,
        value,
    })
}

/// An integer modulus test: int columns only, positive modulus.
fn compile_mod(node: &Node, ctx: &PredicateCtx) -> Result<Predicate, ParseError> {
    let column = ctx.typed_column(node)?;
    if column.kind != ColumnType::Int {
        let message = format!(
            "rule `{}`: `mod` applies to int columns only; `{}` is {}",
            ctx.rule,
            column.name,
            column.kind.as_str()
        );
        return Err(ctx.invalid(&column.pointer, message));
    }
    let modulus = ctx.int_field(node, "modulus")?;
    if modulus <= 0 {
        let message = format!(
            "rule `{}`: `modulus` must be a positive integer, got {modulus}",
            ctx.rule
        );
        return Err(ctx.invalid(&node.at("modulus"), message));
    }
    let equals = ctx.int_field(node, "equals")?;
    Ok(Predicate::Mod {
        column: Ident(column.name),
        modulus,
        equals,
    })
}

/// A boolean combination's operands: a non-empty array of predicates.
fn compile_args(node: &Node, ctx: &PredicateCtx) -> Result<Vec<Predicate>, ParseError> {
    let args = match node.object.get("args") {
        Some(Value::Array(args)) if !args.is_empty() => args,
        _ => {
            let message = format!(
                "rule `{}`: a boolean combination needs a non-empty `args` array",
                ctx.rule
            );
            let pointer = node.at("args");
            return Err(ctx.invalid(&pointer, message));
        }
    };
    args.iter()
        .enumerate()
        .map(|(k, arg)| compile_predicate(arg, &node.at(&format!("args/{k}")), ctx))
        .collect()
}
