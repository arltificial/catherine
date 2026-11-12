//! Purpose: parse `.ct.json` v1 model documents into [`Model`] values,
//! rejecting malformed presentations with line-accurate errors.
//!
//! Responsibilities: gate JSON syntax through serde_json (its own
//! line/column), locate every JSON node by pointer with a span scan so
//! semantic errors carry the line/column of the *offending node*, and
//! enforce the six stable error codes (`duplicate_ident`, `unknown_arrow`,
//! `ill_typed_equation`, `invalid_ident`, `invalid_status`,
//! `unsupported_version`) — the fixture harness's expected shape. Parse
//! normalizes presentation away: row names become dense per-object ids in
//! sorted order, equations are stored in canonical sorted order, and
//! every declared arrow carries a total value vector (`None` = dangling).
//!
//! Rationale: design Decision 3 — "line-accurate" means the line/column of
//! the offending JSON node, not of a bespoke text syntax. Structural
//! malformations outside the six stable codes (missing required fields,
//! JSON type mismatches, references to undeclared objects/arrows/rows)
//! surface as `ParseError::Syntax` with a human message; the six codes are
//! the stable semantic contract the golden fixtures pin. Dangling `null`
//! references parse as `None` — a representable value, never dropped or
//! repaired (acset-core `dangling_is_a_value`). Check order is the
//! document order documented in the crate README, so an error is
//! predictable, never order-dependent on the walk.

mod schema;
mod schema_map;
mod spans;

use crate::error::{ErrorCode, ParseError, SemanticError};
use crate::model::{Arrow, Equation, Ident, Instance, Model, RowId, Schema, SchemaMap, Version};
use schema::{check_arrows, check_equations, check_ident, check_objects, check_version, RawDoc};
use spans::{scan_spans, Spans};
use std::collections::BTreeMap;

/// Parse a `.ct.json` v1 model document. Accepted additive minors of the
/// current major (1) are capped at `KNOWN_MINOR` (schema.rs, currently 0);
/// higher minors are rejected with `unsupported_version` — never guessed.
/// Malformed presentations rejected with line-accurate
/// errors. Check order (documented in the crate README): version →
/// schema objects → arrows → equations → instance rows → tables.
pub fn parse(text: &str) -> Result<Model, ParseError> {
    let doc: RawDoc = serde_json::from_str(text).map_err(syntax_err)?;
    let spans = scan_spans(text);
    validate(doc, &spans)
}

fn validate(doc: RawDoc, spans: &Spans) -> Result<Model, ParseError> {
    // 1. The version gate: content under an unknown major is never
    // interpreted — not a partial or guessed parse.
    let version = check_version(&doc, spans)?;
    // 2. Required top-level sections — structural.
    let Some(schema) = doc.schema else {
        return Err(spans.syntax("", "missing required field `schema`"));
    };
    let Some(instance) = doc.instance else {
        return Err(spans.syntax("", "missing required field `instance`"));
    };
    // 3. Schema objects: charset and duplicate idents (second occurrence).
    let objects = check_objects(&schema, spans)?;
    // 4. Schema arrows: name charset + duplicates; src/tgt charset and
    // declaredness (structural — the six-code `unknown_arrow` is about
    // equation paths).
    let arrows = check_arrows(&schema, &objects, spans)?;
    // 5. Equations: path idents, declared arrows, endpoint typing, status;
    // canonical order (equation list order in the file is presentation —
    // design Decision 4).
    let equations = check_equations(&schema, &arrows, spans)?;
    // 6. Instance rows: repeated JSON keys (dropped silently by serde's
    // last-wins map) are caught from the span scan before they can be
    // lost; object keys must be declared; row idents checked per list.
    let row_index = check_rows(&instance, &objects, spans)?;
    // 7. Instance tables: repeated keys, declared arrows, declared src
    // rows, and values that are declared target rows or `null`.
    let values = check_tables(&instance, &arrows, &row_index, spans)?;
    // 8. The schema map (additive 1.1): declared data only — totality
    // over the declared source objects and arrows, targets declared in
    // the target schema. Absent in every pre-map document (hash-neutral).
    let schema_map = declared_schema_map(&doc.schema_map, &objects, &arrows, spans)?;
    // 9. The normalized model: dense rows per declared object, total
    // value vectors, canonical equation order. Presentation names never
    // reach it.
    Ok(build_model(
        version, objects, arrows, equations, row_index, values, schema_map,
    ))
}

/// The document's declared schema map, validated (additive 1.1) —
/// `None` for every pre-map document (hash-neutral).
fn declared_schema_map(
    doc_schema_map: &Option<schema_map::RawSchemaMap>,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Option<SchemaMap>, ParseError> {
    match doc_schema_map {
        Some(raw) => Ok(Some(schema_map::build_schema_map(
            raw, objects, arrows, spans,
        )?)),
        None => Ok(None),
    }
}

/// Instance rows: declared object keys, per-list ident and duplicate
/// checks, then dense ids in sorted-name order — the sorted enumeration
/// is the first-reference order (design Decision 4).
fn check_rows(
    instance: &schema::RawInstance,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<BTreeMap<String, BTreeMap<String, RowId>>, ParseError> {
    check_dup_keys(spans, "/instance/rows", "in instance rows")?;
    for obj_name in instance.rows.keys() {
        if !objects.contains_key(&Ident(obj_name.clone())) {
            return Err(spans.syntax_key(
                "/instance/rows",
                obj_name,
                format!("instance rows name undeclared object `{obj_name}`"),
            ));
        }
    }
    let mut row_index: BTreeMap<String, BTreeMap<String, RowId>> = BTreeMap::new();
    for (obj_name, rows) in &instance.rows {
        row_index.insert(obj_name.clone(), row_ids(obj_name, rows, spans)?);
    }
    Ok(row_index)
}

/// One object's rows: ident charset + duplicate check, then dense ids in
/// sorted-name order.
fn row_ids(
    obj_name: &str,
    rows: &[String],
    spans: &Spans,
) -> Result<BTreeMap<String, RowId>, ParseError> {
    let mut seen = BTreeMap::new();
    for (j, row) in rows.iter().enumerate() {
        let node = format!("/instance/rows/{obj_name}/{j}");
        check_ident(spans, &node, row)?;
        if seen.contains_key(row) {
            return Err(spans.semantic(
                &node,
                ErrorCode::DuplicateIdent,
                format!("duplicate row ident `{row}` on object `{obj_name}`"),
            ));
        }
        seen.insert(row.clone(), ());
    }
    Ok(seen
        .keys()
        .enumerate()
        .map(|(i, name)| (name.clone(), RowId(i)))
        .collect())
}

/// Instance tables: repeated keys, declared arrows, declared src rows,
/// and values that are declared target rows or `null`. Every declared
/// arrow carries a total vector over its source object's rows; an
/// unspecified cell parses as `None` (dangling).
fn check_tables(
    instance: &schema::RawInstance,
    arrows: &BTreeMap<Ident, Arrow>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    spans: &Spans,
) -> Result<BTreeMap<Ident, Vec<Option<RowId>>>, ParseError> {
    check_table_dup_keys(spans)?;
    for arrow_name in instance.tables.keys() {
        if !arrows.contains_key(&Ident(arrow_name.clone())) {
            return Err(spans.syntax_key(
                "/instance/tables",
                arrow_name,
                format!("instance table names undeclared arrow `{arrow_name}`"),
            ));
        }
    }
    let mut values: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for (name, arrow) in arrows {
        let n = row_index.get(&arrow.src.0).map_or(0, BTreeMap::len);
        values.insert(name.clone(), vec![None; n]);
    }
    for (arrow_name, table) in &instance.tables {
        let arrow = &arrows[&Ident(arrow_name.clone())];
        check_table_cells(arrow_name, arrow, table, row_index, &mut values, spans)?;
    }
    Ok(values)
}

/// Repeated keys inside one table are a duplicate row ident.
fn check_table_dup_keys(spans: &Spans) -> Result<(), ParseError> {
    if let Some(dup) = spans
        .dup_keys
        .iter()
        .find(|d| d.object.starts_with("/instance/tables/"))
    {
        let arrow = dup.object.trim_start_matches("/instance/tables/");
        return Err(ParseError::Semantic(SemanticError {
            line: dup.line,
            column: dup.column,
            code: ErrorCode::DuplicateIdent,
            message: format!("duplicate row ident `{}` in table `{arrow}`", dup.key),
        }));
    }
    Ok(())
}

/// One table's cells: src row must exist, target row must exist (or the
/// cell is an explicit `null` — the same dangling value as an absent
/// cell).
fn check_table_cells(
    arrow_name: &str,
    arrow: &Arrow,
    table: &BTreeMap<String, Option<String>>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    values: &mut BTreeMap<Ident, Vec<Option<RowId>>>,
    spans: &Spans,
) -> Result<(), ParseError> {
    let src_rows = row_index.get(&arrow.src.0);
    let tgt_rows = row_index.get(&arrow.tgt.0);
    for (row_name, value) in table {
        let key_at = format!("/instance/tables/{arrow_name}");
        let src_id = check_table_src(arrow_name, arrow, row_name, src_rows, &key_at, spans)?;
        let Some(target) = value else {
            continue; // explicit `null` — the same dangling value as an absent cell
        };
        let Some(tgt_id) = tgt_rows.and_then(|m| m.get(target)) else {
            return Err(spans.syntax(
                &format!("{key_at}/{row_name}"),
                format!(
                    "table `{arrow_name}` maps `{row_name}` to `{target}`, which is not a row of `{}`",
                    arrow.tgt.0
                ),
            ));
        };
        values.get_mut(&Ident(arrow_name.to_string())).unwrap()[src_id.0] = Some(*tgt_id);
    }
    Ok(())
}

/// A table key must be a declared row of the arrow's source object.
fn check_table_src(
    arrow_name: &str,
    arrow: &Arrow,
    row_name: &str,
    src_rows: Option<&BTreeMap<String, RowId>>,
    key_at: &str,
    spans: &Spans,
) -> Result<RowId, ParseError> {
    let Some(src_map) = src_rows else {
        return Err(spans.syntax_key(
            key_at,
            row_name,
            format!(
                "table `{arrow_name}` keys row `{row_name}`, but `{}` has no declared rows",
                arrow.src.0
            ),
        ));
    };
    let Some(src_id) = src_map.get(row_name) else {
        return Err(spans.syntax_key(
            key_at,
            row_name,
            format!(
                "table `{arrow_name}` keys row `{row_name}`, which is not a row of `{}`",
                arrow.src.0
            ),
        ));
    };
    Ok(*src_id)
}

/// The normalized model: dense rows per declared object, total value
/// vectors, canonical equation order.
#[allow(clippy::too_many_arguments)]
fn build_model(
    version: Version,
    objects: BTreeMap<Ident, ()>,
    arrows: BTreeMap<Ident, Arrow>,
    equations: Vec<Equation>,
    row_index: BTreeMap<String, BTreeMap<String, RowId>>,
    values: BTreeMap<Ident, Vec<Option<RowId>>>,
    schema_map: Option<SchemaMap>,
) -> Model {
    let rows_out = objects
        .keys()
        .map(|o| {
            let n = row_index.get(&o.0).map_or(0, BTreeMap::len);
            (o.clone(), (0..n).map(RowId).collect())
        })
        .collect();
    let schema = Schema {
        objects,
        arrows,
        equations,
    };
    Model {
        format_version: version,
        schema,
        instance: Instance {
            rows: rows_out,
            values,
        },
        schema_map,
    }
}

fn check_dup_keys(spans: &Spans, pointer: &str, where_: &str) -> Result<(), ParseError> {
    if let Some(dup) = spans.dup_keys.iter().find(|d| d.object == pointer) {
        return Err(ParseError::Semantic(SemanticError {
            line: dup.line,
            column: dup.column,
            code: ErrorCode::DuplicateIdent,
            message: format!("duplicate object ident `{}` {where_}", dup.key),
        }));
    }
    Ok(())
}

/// Convert a serde_json error into the structural `Syntax` shape, keeping
/// serde_json's own line/column and stripping its location suffix from the
/// message (the position travels in the fields, not the text).
fn syntax_err(err: serde_json::Error) -> ParseError {
    let message = err.to_string();
    let message = message
        .split(" at line ")
        .next()
        .unwrap_or(&message)
        .to_string();
    ParseError::Syntax {
        line: err.line(),
        column: err.column(),
        message,
    }
}
