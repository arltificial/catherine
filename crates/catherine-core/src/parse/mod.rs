//! Purpose: parse `.ct.json` v1 model documents into [`Model`] values,
//! rejecting malformed presentations with line-accurate errors.
//!
//! Responsibilities: gate JSON syntax through serde_json (its own
//! line/column), locate every JSON node by pointer with a span scan so
//! semantic errors carry the line/column of the *offending node*, and
//! enforce the stable semantic error codes (`duplicate_ident`,
//! `unknown_arrow`, `ill_typed_equation`, `invalid_ident`,
//! `invalid_status`, `unsupported_version`, `unknown_field`, plus the
//! named additive-section extensions) — the fixture harness's expected
//! shape. Parse
//! normalizes presentation away: row names become dense per-object ids in
//! sorted order, equations, keys, and coproducts are stored in canonical
//! sorted order,
//! and every declared arrow carries a total value vector (`None` =
//! dangling).
//!
//! Rationale: design Decision 3 — "line-accurate" means the line/column of
//! the offending JSON node, not of a bespoke text syntax. Structural
//! malformations outside the stable codes (missing required fields, JSON
//! type mismatches, references to undeclared objects/arrows/rows) surface
//! as `ParseError::Syntax` with a human message; the codes are the stable
//! semantic contract the golden fixtures pin. A field the accepted format
//! version does not define is `unknown_field`, never dropped: additive
//! minors above `KNOWN_MINOR` fail the version gate, so tolerance inside
//! an accepted version could only hide typos (catherine-bak). Dangling
//! `null` references parse as `None` — a representable value, never
//! dropped or repaired (acset-core `dangling_is_a_value`). Check order is
//! the document order documented in the crate README, so an error is
//! predictable, never order-dependent on the walk.
//!
//! Names stay outside the parsed model by construction: [`parse_with_rows`]
//! returns the same [`Model`] as [`parse`] beside a [`RowNames`] side table
//! (object → names indexed by dense [`RowId`]), so callers that must spell
//! a value (the `rules --project` bridge) read the table while `Instance`
//! equality, the canonical bytes, and the model hash remain name-blind.

mod build;
mod coproducts;
mod deferred;
mod instance_map;
mod keys;
mod predicate;
mod rows;
mod rule_presentation;
mod schema;
mod schema_map;
mod spans;

use crate::error::{ErrorCode, ParseError, SemanticError};
use crate::model::{
    Arrow, Equation, Ident, InstanceMap, Model, RowId, RulePresentation, SchemaMap,
};
use rows::check_rows;
pub use rows::RowNames;

use build::build_model;
use coproducts::check_coproducts;
pub(crate) use deferred::{parse_deferred, PendingEquations};
use keys::check_keys;
use schema::{check_arrows, check_equations, check_objects, check_version, RawDoc};
use spans::{scan_spans, Spans};
use std::collections::BTreeMap;

/// Parse a `.ct.json` v1 model document. Accepted additive minors of the
/// current major (1) are capped at `KNOWN_MINOR` (schema.rs, currently 5);
/// higher minors are rejected with `unsupported_version` — never guessed.
/// Malformed presentations rejected with line-accurate
/// errors. Check order (documented in the crate README): version →
/// schema objects → arrows → equations → keys → coproducts → instance rows → tables →
/// schema map → rule presentation.
pub fn parse(text: &str) -> Result<Model, ParseError> {
    parse_with_rows(text).map(|(model, _)| model)
}

/// [`parse`], additionally returning the presentation names it normalized
/// away ([`RowNames`]). The model half is byte-identical to [`parse`]'s
/// result — the names are a side table, not part of the model, so hash
/// and canonical form are untouched.
pub fn parse_with_rows(text: &str) -> Result<(Model, RowNames), ParseError> {
    let doc: RawDoc = serde_json::from_str(text).map_err(doc_err)?;
    let spans = scan_spans(text);
    let (model, rows, _) = validate_with(doc, &spans, false)?;
    Ok((model, rows))
}

/// A labeled rejection at the offending node, shared by the additive-section
/// passes (schema map, rule presentation, predicate language).
fn reject(spans: &Spans, pointer: &str, code: ErrorCode, message: String) -> ParseError {
    spans.semantic(pointer, code, message)
}

/// The parse composition, step by step (each step delegated):
/// 1. version gate — unknown majors are never interpreted, not a partial
///    or guessed parse;
/// 2. required top-level sections — structural;
/// 3. schema objects — charset and duplicate idents (second occurrence);
/// 4. schema arrows — name charset + duplicates, src/tgt charset and
///    declaredness (the stable `unknown_arrow` is about equation paths);
/// 5. equations — path idents, declared arrows, endpoint typing, status,
///    canonical order (file order is presentation — Decision 4); in the
///    composition mode ([`parse_deferred`]) this slot runs against the
///    document's own arrows and a path naming a sibling document's arrow
///    is carried to the post-glue resolution instead of rejected;
/// 6. keys (additive 1.3) — object declaredness, a non-empty
///    duplicate-free arrow tuple sourced at the object, status, canonical
///    order (absent/empty = hash-neutral);
/// 7. coproducts (additive 1.5) — object declaredness, a non-empty
///    duplicate-free injection list `V_k → C`, status, canonical order
///    (absent/empty = hash-neutral);
/// 8. instance rows — repeated JSON keys caught from the span scan before
///    serde's last-wins map can drop them; object keys declared; row idents
///    checked per list (unknown instance keys already failed
///    deserialization, never read as an empty section);
/// 9. instance tables — repeated keys, declared arrows, declared src
///    rows, values that are declared target rows or `null`;
/// 10. instance map (additive 1.4) — declared target names preserved as
///     data, total coverage of every declared row (absent = hash-neutral);
/// 11. schema map (additive 1.1) — declared data only, totality over the
///     declared source elements, targets declared (absent = hash-neutral);
/// 12. rule presentation (additive 1.2) — declared domain, typed columns,
///     decidable predicates, required actions, declared policy (absent =
///     hash-neutral);
/// 13. the normalized model — dense rows, total value vectors, canonical
///     equation, key, and coproduct order — plus the [`RowNames`] side
///     table (the presentation names, indexed by those dense ids); the
///     names never enter the model.
fn validate_with(
    doc: RawDoc,
    spans: &Spans,
    defer_equations: bool,
) -> Result<(Model, RowNames, Option<PendingEquations>), ParseError> {
    let version = check_version(&doc, spans)?;
    let Some(schema) = doc.schema else {
        return Err(spans.syntax("", "missing required field `schema`"));
    };
    let Some(instance) = doc.instance else {
        return Err(spans.syntax("", "missing required field `instance`"));
    };
    let objects = check_objects(&schema, spans)?;
    let arrows = check_arrows(&schema, &objects, spans)?;
    let (equations, pending) = resolved_equations(&schema, &arrows, spans, defer_equations)?;
    let keys = check_keys(&schema, &objects, &arrows, spans)?;
    let coproducts = check_coproducts(&schema, &objects, &arrows, spans)?;
    let row_index = check_rows(&instance, &objects, spans)?;
    let values = check_tables(&instance, &arrows, &row_index, spans)?;
    let instance_map = declared_instance_map(&doc.instance_map, &objects, &row_index, spans)?;
    let schema_map = declared_schema_map(&doc.schema_map, &objects, &arrows, spans)?;
    let rule_presentation = declared_rule_presentation(&doc.rule_presentation, &objects, spans)?;
    let (model, rows) = build_model(
        version,
        objects,
        arrows,
        equations,
        keys,
        coproducts,
        row_index,
        values,
        schema_map,
        rule_presentation,
        instance_map,
    );
    Ok((model, rows, pending))
}

/// The equation phase: standalone documents run the parser's own resolver
/// over their arrows immediately; composition documents defer a path that
/// names a sibling's arrow to the post-glue resolution (design Decision
/// 3).
fn resolved_equations(
    schema: &schema::RawSchema,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
    defer_equations: bool,
) -> Result<(Vec<Equation>, Option<PendingEquations>), ParseError> {
    if defer_equations {
        deferred::check_equations(&schema.equations, arrows, spans)
    } else {
        Ok((check_equations(&schema.equations, arrows, spans)?, None))
    }
}

/// The document's declared instance map, validated (additive 1.4) —
/// `None` for every pre-map document (hash-neutral).
fn declared_instance_map(
    doc_instance_map: &Option<instance_map::RawInstanceMap>,
    objects: &BTreeMap<Ident, ()>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    spans: &Spans,
) -> Result<Option<InstanceMap>, ParseError> {
    match doc_instance_map {
        Some(raw) => instance_map::build_instance_map(raw, objects, row_index, spans),
        None => Ok(None),
    }
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

/// The document's declared rule presentation, validated (additive 1.2) —
/// `None` for every pre-rule document (hash-neutral).
fn declared_rule_presentation(
    doc_rule_presentation: &Option<rule_presentation::RawRulePresentation>,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<Option<RulePresentation>, ParseError> {
    match doc_rule_presentation {
        Some(raw) => Ok(Some(rule_presentation::build_rule_presentation(
            raw, objects, spans,
        )?)),
        None => Ok(None),
    }
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

/// Convert a serde_json deserialization error into a `ParseError`,
/// keeping serde_json's own line/column and stripping its location suffix
/// from the message (the position travels in the fields, not the text).
/// A field the accepted format version does not define is promoted to the
/// stable `unknown_field` code: the parser refuses it, never drops it
/// (catherine-bak; a dropped constraint key can turn a violated model
/// into a false clean).
fn doc_err(err: serde_json::Error) -> ParseError {
    let text = err.to_string();
    let message = text.split(" at line ").next().unwrap_or(&text);
    match unknown_field_parts(message) {
        Some((field, expected)) => ParseError::Semantic(SemanticError {
            line: err.line(),
            column: err.column(),
            code: ErrorCode::UnknownField,
            message: format!(
                "unknown field `{field}`: the accepted format version does not define it{expected}"
            ),
        }),
        None => ParseError::Syntax {
            line: err.line(),
            column: err.column(),
            message: message.to_string(),
        },
    }
}

/// Split serde's `unknown field `x`, expected one of ...` message into
/// the field name and a `; expected ...` suffix (empty when serde named
/// no expectation).
fn unknown_field_parts(message: &str) -> Option<(&str, String)> {
    let rest = message.strip_prefix("unknown field `")?;
    let (field, tail) = rest.split_once('`')?;
    let expected = tail.trim_start_matches(", ").trim();
    let suffix = if expected.is_empty() {
        String::new()
    } else {
        format!("; {expected}")
    };
    Some((field, suffix))
}
