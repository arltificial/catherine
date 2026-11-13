//! The model as a canonical-shape JSON value. Split out of `canonical.rs`
//! verbatim (mechanical lint refactor); no behavior change.

use crate::model::{
    Equation, Key, Literal, Model, Predicate, RowId, Rule, RulePresentation, Schema, SchemaMap,
};
use serde_json::{Map, Value};

use super::writer::write_value;

/// Canonical serialization: a valid `.ct.json` v1 document (dense
/// per-object row names `_<id>` in id order) that re-parses to the same
/// model. This is the byte sequence the model hash is taken over.
pub fn serialize(model: &Model) -> String {
    write_value(&canonical_value(model))
}

/// The model as a JSON value in canonical shape: sorted enumerations,
/// dense per-object row names, total per-arrow tables.
pub fn canonical_value(model: &Model) -> Value {
    let mut sections: Vec<(&str, Value)> = vec![
        (
            "format_version",
            string(&format!(
                "{}.{}",
                model.format_version.major, model.format_version.minor
            )),
        ),
        ("instance", instance_value(model)),
        ("schema", schema_value(model)),
    ];
    // The additive 1.1 section is emitted only when present: absent
    // sections are invisible to canonical bytes, so pre-map documents
    // keep their pre-map identity (the ledger's content addressing).
    if let Some(map) = &model.schema_map {
        sections.push(("schema_map", schema_map_value(map)));
    }
    // The additive 1.2 section is likewise emitted only when present, so
    // pre-rule documents keep their pre-rule identity.
    if let Some(presentation) = &model.rule_presentation {
        sections.push(("rule_presentation", rule_presentation_value(presentation)));
    }
    obj_vec(sections)
}

/// The canonical `instance` section: dense per-object row names and total
/// per-arrow tables.
fn instance_value(model: &Model) -> Value {
    let mut rows = Map::new();
    for (object, ids) in &model.instance.rows {
        let names: Vec<Value> = (0..ids.len()).map(|i| string(&row_name(i))).collect();
        rows.insert(object.0.clone(), Value::Array(names));
    }
    let mut tables = Map::new();
    for (arrow, cells) in &model.instance.values {
        tables.insert(arrow.0.clone(), table_value(cells));
    }
    obj([
        ("rows", Value::Object(rows)),
        ("tables", Value::Object(tables)),
    ])
}

/// One arrow's canonical table: keys and values spelled as dense row
/// names, dangling cells as `null`.
fn table_value(cells: &[Option<RowId>]) -> Value {
    let mut emitted = Map::new();
    for (i, cell) in cells.iter().enumerate() {
        let value = match cell {
            Some(RowId(j)) => string(&row_name(*j)),
            None => Value::Null,
        };
        emitted.insert(row_name(i), value);
    }
    Value::Object(emitted)
}

/// The canonical `schema` section: sorted objects, arrows, and equations
/// (equations sorted by their canonical sort key — the parser already
/// stores them sorted, but constructed values must serialize the same
/// way, specodelic's `schema_export` discipline).
fn schema_value(model: &Model) -> Value {
    schema_section(&model.schema)
}

/// The canonical `schema` section: sorted objects, arrows, equations, and
/// keys (equations and keys sorted by their canonical sort keys — the
/// parser already stores them sorted, but constructed values must
/// serialize the same way, specodelic's `schema_export` discipline). An
/// empty key list is omitted entirely: pre-1.3 documents keep their exact
/// canonical bytes and hash (design Decision 1).
fn schema_section(schema: &Schema) -> Value {
    let mut equations: Vec<&Equation> = schema.equations.iter().collect();
    equations.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    let mut keys: Vec<&Key> = schema.keys.iter().collect();
    keys.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    let arrows: Vec<Value> = schema
        .arrows
        .iter()
        .map(|(name, arrow)| {
            obj([
                ("name", string(&name.0)),
                ("src", string(&arrow.src.0)),
                ("tgt", string(&arrow.tgt.0)),
            ])
        })
        .collect();
    let equation_values: Vec<Value> = equations.iter().map(|e| equation_value(e)).collect();
    let mut entries = vec![
        (
            "objects",
            Value::Array(schema.objects.keys().map(|o| string(&o.0)).collect()),
        ),
        ("arrows", Value::Array(arrows)),
        ("equations", Value::Array(equation_values)),
    ];
    if !keys.is_empty() {
        entries.push((
            "keys",
            Value::Array(keys.iter().map(|key| key_value(key)).collect()),
        ));
    }
    obj_vec(entries)
}

/// One canonical key entry: the object, its arrow sequence in declared
/// order, and the constraint status.
fn key_value(key: &Key) -> Value {
    obj([
        ("object", string(&key.object.0)),
        (
            "arrows",
            Value::Array(key.arrows.iter().map(|arrow| string(&arrow.0)).collect()),
        ),
        ("status", string(key.status.as_str())),
    ])
}

/// The canonical `schema_map` section: the validated target schema plus
/// the object/arrow assignments (dropped elements as `null`).
fn schema_map_value(map: &SchemaMap) -> Value {
    obj_vec(vec![
        ("target_schema", schema_section(&map.target)),
        (
            "objects",
            Value::Object(
                map.objects
                    .iter()
                    .map(|(object, assignment)| {
                        let value = match assignment {
                            Some(target) => string(&target.0),
                            None => Value::Null,
                        };
                        (object.0.clone(), value)
                    })
                    .collect(),
            ),
        ),
        (
            "arrows",
            Value::Object(
                map.arrows
                    .iter()
                    .map(|(arrow, assignment)| {
                        let value = match assignment {
                            Some(assignment_path) => path(assignment_path),
                            None => Value::Null,
                        };
                        (arrow.0.clone(), value)
                    })
                    .collect(),
            ),
        ),
    ])
}

/// The canonical `rule_presentation` section: columns sorted by name, the
/// rules in priority order (written order is semantic — never sorted), and
/// the policy only when declared.
fn rule_presentation_value(presentation: &RulePresentation) -> Value {
    let columns: Vec<Value> = presentation
        .columns
        .iter()
        .map(|(name, column_type)| {
            obj([
                ("name", string(&name.0)),
                ("type", string(column_type.as_str())),
            ])
        })
        .collect();
    let rules: Vec<Value> = presentation.rules.iter().map(rule_value).collect();
    let mut entries = vec![
        ("domain", string(&presentation.domain.0)),
        ("columns", Value::Array(columns)),
        ("rules", Value::Array(rules)),
    ];
    if let Some(policy) = presentation.policy {
        entries.push(("policy", string(policy.as_str())));
    }
    obj_vec(entries)
}

/// One canonical rule entry.
fn rule_value(rule: &Rule) -> Value {
    obj([
        ("name", string(&rule.name.0)),
        ("action", string(&rule.action)),
        ("predicate", predicate_value(&rule.predicate)),
    ])
}

/// One canonical predicate node, in the format's kind-tagged shape.
fn predicate_value(predicate: &Predicate) -> Value {
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

/// One canonical equation entry.
fn equation_value(equation: &Equation) -> Value {
    obj([
        ("lhs", path(&equation.lhs)),
        ("rationale", string(&equation.rationale)),
        ("rhs", path(&equation.rhs)),
        ("status", string(equation.status.as_str())),
    ])
}

fn obj<const N: usize>(entries: [(&str, Value); N]) -> Value {
    obj_vec(entries.into_iter().collect())
}

fn obj_vec(entries: Vec<(&str, Value)>) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn string(s: &str) -> Value {
    Value::String(s.to_string())
}

fn path(p: &crate::model::Path) -> Value {
    Value::Array(p.0.iter().map(|i| string(&i.0)).collect())
}

/// The presentation spelling of a dense row id inside serialized bytes:
/// `_`-prefixed so it is a valid ident (leading digits are
/// `invalid_ident`).
fn row_name(i: usize) -> String {
    format!("_{i}")
}
