//! Purpose: the optional additive sections (1.1 `schema_map`, 1.2
//! `rule_presentation`, 1.4 `instance_map`) validated against the checked
//! required sections and folded into the model input. Split out of
//! `parse/mod.rs` so both files stay under the size gate; bodies moved
//! verbatim, no behavior change.
//!
//! Responsibilities: the declared-but-optional section checks in document
//! order, the shared validation context, and the checked-sections →
//! `ModelInput` assembly.
//!
//! Rationale: every additive section is `None` for the documents that
//! predate it (hash-neutral), so old fixtures keep loading and keep their
//! canonical bytes.

use super::build::{ModelInput, SchemaParts};
use super::spans::Spans;
use super::{CheckedSections, PendingEquations};
use crate::error::ParseError;
use crate::model::{Arrow, Ident, InstanceMap, RowId, RulePresentation, SchemaMap};
use std::collections::BTreeMap;

/// The optional additive sections, validated and folded into the model
/// input (schema map → rule presentation → instance map, document
/// order). Consumes the checked sections and returns the equations
/// deferred to the post-glue resolution beside the input.
pub(super) fn validate_additives(
    additives: super::RawAdditives,
    checked: CheckedSections,
    spans: &Spans,
) -> Result<(ModelInput, Option<PendingEquations>), ParseError> {
    let (doc_schema_map, doc_rule_presentation, doc_instance_map) = additives;
    let CheckedSections {
        objects,
        arrows,
        row_index,
        ..
    } = &checked;
    let ctx = AdditiveCtx {
        objects,
        arrows,
        row_index,
        spans,
    };
    let instance_map = declared_instance_map(&doc_instance_map, &ctx)?;
    let schema_map = declared_schema_map(&doc_schema_map, &ctx)?;
    let rule_presentation =
        declared_rule_presentation(&doc_rule_presentation, &checked.objects, spans)?;
    let mut checked = checked;
    let pending = checked.pending.take();
    let input = model_input(checked, schema_map, rule_presentation, instance_map);
    Ok((input, pending))
}

/// The checked sections folded into the model input: the schema
/// declarations bundled as one [`SchemaParts`], instance and additive
/// sections beside them.
fn model_input(
    checked: CheckedSections,
    schema_map: Option<SchemaMap>,
    rule_presentation: Option<RulePresentation>,
    instance_map: Option<InstanceMap>,
) -> ModelInput {
    ModelInput {
        schema: SchemaParts {
            objects: checked.objects,
            arrows: checked.arrows,
            equations: checked.equations,
            keys: checked.keys,
            coproducts: checked.coproducts,
            pullbacks: checked.pullbacks,
        },
        row_index: checked.row_index,
        values: checked.values,
        schema_map,
        rule_presentation,
        instance_map,
    }
}

/// The declared elements the additive sections are validated against: the
/// schema's objects and arrows, the instance's rows, and the span index.
struct AdditiveCtx<'a> {
    objects: &'a BTreeMap<Ident, ()>,
    arrows: &'a BTreeMap<Ident, Arrow>,
    row_index: &'a BTreeMap<String, BTreeMap<String, RowId>>,
    spans: &'a Spans,
}

/// The document's declared instance map, validated (additive 1.4) —
/// `None` for every pre-map document (hash-neutral).
fn declared_instance_map(
    doc_instance_map: &Option<super::instance_map::RawInstanceMap>,
    ctx: &AdditiveCtx,
) -> Result<Option<InstanceMap>, ParseError> {
    match doc_instance_map {
        Some(raw) => super::instance_map::build_instance_map(
            raw,
            super::instance_map::MapCtx {
                objects: ctx.objects,
                row_index: ctx.row_index,
                spans: ctx.spans,
            },
        ),
        None => Ok(None),
    }
}

/// The document's declared schema map, validated (additive 1.1) —
/// `None` for every pre-map document (hash-neutral).
fn declared_schema_map(
    doc_schema_map: &Option<super::schema_map::RawSchemaMap>,
    ctx: &AdditiveCtx,
) -> Result<Option<SchemaMap>, ParseError> {
    match doc_schema_map {
        Some(raw) => Ok(Some(super::schema_map::build_schema_map(
            raw,
            &super::schema_map::SourceMap {
                objects: ctx.objects,
                arrows: ctx.arrows,
                spans: ctx.spans,
            },
        )?)),
        None => Ok(None),
    }
}

/// The document's declared rule presentation, validated (additive 1.2) —
/// `None` for every pre-rule document (hash-neutral).
fn declared_rule_presentation(
    doc_rule_presentation: &Option<super::rule_presentation::RawRulePresentation>,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<Option<RulePresentation>, ParseError> {
    match doc_rule_presentation {
        Some(raw) => Ok(Some(super::rule_presentation::build_rule_presentation(
            raw, objects, spans,
        )?)),
        None => Ok(None),
    }
}
