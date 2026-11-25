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
//! sorted order, equations, keys, coproducts, and pullbacks are stored in
//! canonical sorted order,
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

mod additives;
mod build;
mod coproducts;
mod deferred;
mod equations;
mod instance_map;
mod keys;
mod predicate;
mod pullbacks;
mod rows;
mod rule_presentation;
mod schema;
mod schema_map;
mod serde_error;
mod spans;
mod tables;

use crate::error::{ErrorCode, ParseError, SemanticError};
use crate::model::{Arrow, Coproduct, Equation, Ident, Key, Model, Pullback, RowId};
use rows::check_rows;
pub use rows::RowNames;

use additives::validate_additives;
use build::build_model;
use coproducts::{check_coproducts, CoproductCtx};
pub(crate) use deferred::{parse_deferred, PendingEquations};
use equations::check_equations;
use keys::{check_keys, KeyCtx};
use pullbacks::{check_pullbacks, PullbackCtx};
use schema::{check_arrows, check_objects, check_version, RawDoc, RawSchema};
use serde_error::doc_err;
use spans::{scan_spans, Spans};
use std::collections::BTreeMap;
use tables::check_tables;

/// Parse a `.ct.json` v1 model document. Accepted additive minors of the
/// current major (1) are capped at `KNOWN_MINOR` (schema.rs, currently 6);
/// higher minors are rejected with `unsupported_version` — never guessed.
/// Malformed presentations rejected with line-accurate
/// errors. Check order (documented in the crate README): version →
/// schema objects → arrows → equations → keys → coproducts → pullbacks →
/// instance rows → tables → schema map → rule presentation.
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

/// The document's sections after the version gate: the required
/// `schema`/`instance` pair (each rejected in document order — `schema`
/// first, then `instance`) plus the three optional additive sections.
/// Split out of `validate_with` (mechanical lint refactor; no behavior
/// change) so `validate_with`'s forwarding calls stay inside the
/// function-size gate.
/// The required `schema`/`instance` pair plus the three optional additive
/// sections, in document order.
type RawSections = (RawSchema, schema::RawInstance, RawAdditives);

/// The optional additive sections, in document order.
type RawAdditives = (
    Option<schema_map::RawSchemaMap>,
    Option<rule_presentation::RawRulePresentation>,
    Option<instance_map::RawInstanceMap>,
);

fn required_sections(doc: RawDoc, spans: &Spans) -> Result<RawSections, ParseError> {
    let RawDoc {
        schema,
        instance,
        schema_map,
        rule_presentation,
        instance_map,
        ..
    } = doc;
    let Some(schema) = schema else {
        return Err(spans.syntax("", "missing required field `schema`"));
    };
    let Some(instance) = instance else {
        return Err(spans.syntax("", "missing required field `instance`"));
    };
    Ok((
        schema,
        instance,
        (schema_map, rule_presentation, instance_map),
    ))
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
/// 8. pullbacks (additive 1.6) — apex declaredness, exactly two legs
///    sourced at the apex, exactly two same-target cospan arrows, no
///    repeated arrow name, status, canonical order (absent/empty =
///    hash-neutral);
/// 9. instance rows — repeated JSON keys caught from the span scan before
///    serde's last-wins map can drop them; object keys declared; row idents
///    checked per list (unknown instance keys already failed
///    deserialization, never read as an empty section);
/// 10. instance tables — repeated keys, declared arrows, declared src
///     rows, values that are declared target rows or `null`;
/// 11. instance map (additive 1.4) — declared target names preserved as
///     data, total coverage of every declared row (absent = hash-neutral);
/// 12. schema map (additive 1.1) — declared data only, totality over the
///     declared source elements, targets declared (absent = hash-neutral);
/// 13. rule presentation (additive 1.2) — declared domain, typed columns,
///     decidable predicates, required actions, declared policy (absent =
///     hash-neutral);
/// 14. the normalized model — dense rows, total value vectors, canonical
///     equation, key, coproduct, and pullback order — plus the [`RowNames`]
///     side table (the presentation names, indexed by those dense ids); the
///     names never enter the model.
fn validate_with(
    doc: RawDoc,
    spans: &Spans,
    defer_equations: bool,
) -> Result<(Model, RowNames, Option<PendingEquations>), ParseError> {
    let version = check_version(&doc, spans)?;
    let (schema, instance, additives) = required_sections(doc, spans)?;
    let sections = SectionCtx {
        schema: &schema,
        instance: &instance,
        spans,
    };
    let checked = check_sections(&sections, defer_equations)?;
    let (input, pending) = validate_additives(additives, checked, spans)?;
    let (mut model, rows) = build_model(version, input);
    apply_stored(&mut model, schema.stored);
    Ok((model, rows, pending))
}

/// The schema and instance checks' output, in document order: the merged
/// name sets, the declared arrows, the equations (plus any deferred to
/// the post-glue resolution), the dense rows, the total value vectors,
/// and the structural declarations.
struct CheckedSections {
    objects: BTreeMap<Ident, ()>,
    arrows: BTreeMap<Ident, Arrow>,
    equations: Vec<Equation>,
    pending: Option<PendingEquations>,
    row_index: BTreeMap<String, BTreeMap<String, RowId>>,
    values: BTreeMap<Ident, Vec<Option<RowId>>>,
    keys: Vec<Key>,
    coproducts: Vec<Coproduct>,
    pullbacks: Vec<Pullback>,
}

/// The required sections as one check target: the raw schema, the raw
/// instance, and the span index.
struct SectionCtx<'a> {
    schema: &'a RawSchema,
    instance: &'a schema::RawInstance,
    spans: &'a Spans,
}

/// The equation phase's inputs: the declared arrows, the span index, and
/// whether composition defers sibling-arrow paths.
struct EqCheck<'a> {
    arrows: &'a BTreeMap<Ident, Arrow>,
    spans: &'a Spans,
    defer_equations: bool,
}

/// The declared schema elements shared by the structural checks.
struct Decl<'a> {
    objects: &'a BTreeMap<Ident, ()>,
    arrows: &'a BTreeMap<Ident, Arrow>,
    spans: &'a Spans,
}

/// The required sections' checks, in document order: schema objects →
/// arrows → equations → instance rows → instance tables → the structural
/// declarations (keys, coproducts, pullbacks). Split out of
/// `validate_with` (mechanical lint refactor; no behavior change) so both
/// halves stay inside the function-size gate.
fn check_sections(
    sections: &SectionCtx,
    defer_equations: bool,
) -> Result<CheckedSections, ParseError> {
    let schema = sections.schema;
    let spans = sections.spans;
    let objects = check_objects(schema, spans)?;
    let arrows = check_arrows(schema, &objects, spans)?;
    let (equations, pending) = resolved_equations(
        schema,
        EqCheck {
            arrows: &arrows,
            spans,
            defer_equations,
        },
    )?;
    let (row_index, values, structure) = checked_data_sections(sections, &objects, &arrows)?;
    Ok(CheckedSections {
        objects,
        arrows,
        equations,
        pending,
        row_index,
        values,
        keys: structure.0,
        coproducts: structure.1,
        pullbacks: structure.2,
    })
}

/// The instance and structural sections' results, in check order.
type DataSections = (RowIndex, Values, RawStructure);

/// The instance and structural sections over the schema's checked
/// objects and arrows: rows, value tables, keys, coproducts, pullbacks.
fn checked_data_sections(
    sections: &SectionCtx,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, Arrow>,
) -> Result<DataSections, ParseError> {
    let (row_index, values) = checked_instance(sections, objects, arrows, sections.spans)?;
    let structure = check_structure(
        sections.schema,
        &Decl {
            objects,
            arrows,
            spans: sections.spans,
        },
    )?;
    Ok((row_index, values, structure))
}

/// The instance's object → row → declared id table (dense-sorted ids).
type RowIndex = BTreeMap<String, BTreeMap<String, RowId>>;
/// The instance's per-arrow cell tables over the indexed rows.
type Values = BTreeMap<Ident, Vec<Option<RowId>>>;

/// The instance section's row index and tables, checked against the
/// schema's objects and arrows.
fn checked_instance(
    sections: &SectionCtx,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<(RowIndex, Values), ParseError> {
    let row_index = check_rows(sections.instance, objects, spans)?;
    let values = check_tables(
        sections.instance,
        tables::TableCtx {
            arrows,
            row_index: &row_index,
            spans,
        },
    )?;
    Ok((row_index, values))
}

/// The three structural checks over the same (schema, objects, arrows)
/// triple, gathered so `validate_with` stays inside the file-size gate's
/// per-function cap without changing check order or messages.
/// Keys, coproducts, and pullbacks discovered in one schema, in check order.
type RawStructure = (Vec<Key>, Vec<Coproduct>, Vec<Pullback>);

fn check_structure(schema: &RawSchema, decl: &Decl) -> Result<RawStructure, ParseError> {
    let spans = decl.spans;
    let keys = check_keys(
        schema,
        KeyCtx {
            objects: decl.objects,
            arrows: decl.arrows,
            spans,
        },
    )?;
    let coproducts = check_coproducts(
        schema,
        CoproductCtx {
            objects: decl.objects,
            arrows: decl.arrows,
            spans,
        },
    )?;
    let pullbacks = check_pullbacks(
        schema,
        PullbackCtx {
            objects: decl.objects,
            arrows: decl.arrows,
            spans,
        },
    )?;
    Ok((keys, coproducts, pullbacks))
}

/// Additive `schema.stored` (ticket `catherine-kw9.3`): presentation
/// metadata for the CLI's granularity lint. Keys naming undeclared
/// arrows are carried as declared data — the lint reads only the
/// declared-arrow intersection, so a stale key can never mask a
/// finding by accident.
fn apply_stored(model: &mut Model, stored: BTreeMap<String, String>) {
    model.schema.stored = stored
        .into_iter()
        .map(|(arrow, column)| (Ident(arrow), column))
        .collect();
}

/// The equation phase: standalone documents run the parser's own resolver
/// over their arrows immediately; composition documents defer a path that
/// names a sibling's arrow to the post-glue resolution (design Decision
/// 3).
fn resolved_equations(
    schema: &schema::RawSchema,
    check: EqCheck,
) -> Result<(Vec<Equation>, Option<PendingEquations>), ParseError> {
    if check.defer_equations {
        deferred::check_equations(&schema.equations, check.arrows, check.spans)
    } else {
        Ok((
            check_equations(&schema.equations, check.arrows, check.spans)?,
            None,
        ))
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
