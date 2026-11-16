//! Purpose: the deferred equations parse surface (openspec change
//! `add-model-composition`, design Decision 3) — document parsing whose
//! equation phase validates against the document's own arrows first and
//! carries a genuinely cross-document claim to the post-glue stage.
//!
//! Responsibilities: `parse_deferred` validates a document exactly like
//! `parse_with_rows` except that a path element naming no arrow of this
//! document is deferred instead of rejected; it returns the normalized
//! model, the presentation row names, the pending equations with their span
//! scan, and the arrow/table-cell node positions the glue's conflict
//! reports cite.
//!
//! Rationale: a seam equation may name a sibling document's arrows, so
//! declaredness is only decidable over the pushout. The deferral is narrow
//! on purpose — an undeclared path arrow is the *only* deferrable failure,
//! because endpoint typing, idents, and status are decided the same way
//! over the pushout as over the document (a shared arrow keeps its
//! endpoints, so a local mismatch can never be repaired). The equation slot
//! still runs the parser's own `check_equations` against the document's
//! arrows, so a document that resolves there is validated in parse order
//! with parse's codes, and a deferred document is resolved by that *same*
//! function over the merged arrows (one resolver, two arrow sets). Spans
//! travel with the pending equations, so the post-glue error is
//! byte-identical (code, position, message) to the single-document one.

use super::schema::{RawDoc, RawEquation, RawSchema};
use super::spans::{scan_spans, Spans};
use super::{syntax_err, validate_with, RowNames};
use crate::error::{ErrorCode, ParseError};
use crate::model::{Arrow, Equation, Ident, Model};
use std::collections::BTreeMap;

/// A parsed document whose equations may name a sibling document's
/// arrows: the normalized model (its equation list empty while they are
/// pending), the presentation row names, the pending equations, and the
/// declaration/cell node positions the glue's conflict reports cite.
pub(crate) struct DeferredModel {
    pub(crate) model: Model,
    pub(crate) rows: RowNames,
    /// `Some` when the equation phase was deferred to the glue.
    pub(crate) pending: Option<PendingEquations>,
    /// `/schema/arrows/{i}` position keyed by arrow name.
    pub(crate) arrow_positions: BTreeMap<String, (usize, usize)>,
    /// `/instance/tables/{arrow}/{row}` position keyed by (arrow, row).
    pub(crate) cell_positions: BTreeMap<(String, String), (usize, usize)>,
}

/// Equations whose name resolution awaits the pushout, with the span scan
/// that reproduces the parse-time error shape at the offending node.
#[derive(Debug, Clone)]
pub(crate) struct PendingEquations {
    equations: Vec<RawEquation>,
    spans: Spans,
}

impl PendingEquations {
    /// Resolve the paths over `arrows` with the parser's own check and
    /// order — the pushout's arrows for a seam document, the document's
    /// own for everything else.
    pub(crate) fn resolve(
        &self,
        arrows: &BTreeMap<Ident, Arrow>,
    ) -> Result<Vec<Equation>, ParseError> {
        super::schema::check_equations(&self.equations, arrows, &self.spans)
    }
}

/// [`super::parse_with_rows`] for the composition surface: the equation
/// slot runs the parser's checks against the document's own arrows and
/// defers only a failure the pushout may satisfy.
pub(crate) fn parse_deferred(text: &str) -> Result<DeferredModel, ParseError> {
    let doc: RawDoc = serde_json::from_str(text).map_err(syntax_err)?;
    let spans = scan_spans(text);
    let arrow_positions = arrow_positions(doc.schema.as_ref(), &spans);
    let cell_positions = cell_positions(doc.instance.as_ref(), &spans);
    let (model, rows, pending) = validate_with(doc, &spans, true)?;
    Ok(DeferredModel {
        model,
        rows,
        pending,
        arrow_positions,
        cell_positions,
    })
}

/// The equation slot in deferred mode: the parser's own validation against
/// `arrows`. A success means the document needs no post-glue resolution; a
/// failure the pushout may satisfy becomes `PendingEquations`; every other
/// failure is raised here, in the parser's order.
pub(super) fn check_equations(
    equations: &[RawEquation],
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<(Vec<Equation>, Option<PendingEquations>), ParseError> {
    match super::schema::check_equations(equations, arrows, spans) {
        Ok(resolved) => Ok((resolved, None)),
        Err(error) => deferred_or(error, equations, spans),
    }
}

/// Carry a deferrable failure to the glue; raise every other failure.
fn deferred_or(
    error: ParseError,
    equations: &[RawEquation],
    spans: &Spans,
) -> Result<(Vec<Equation>, Option<PendingEquations>), ParseError> {
    if !deferrable(&error) {
        return Err(error);
    }
    let pending = PendingEquations {
        equations: equations.to_vec(),
        spans: spans.clone(),
    };
    Ok((Vec::new(), Some(pending)))
}

/// Whether the failure is the one resolution the pushout may satisfy: a
/// path element naming no arrow this document declares. Endpoint typing is
/// never deferrable — a commonly declared arrow keeps its endpoints, so a
/// mismatch over the document's own arrows cannot be repaired by a union.
fn deferrable(error: &ParseError) -> bool {
    let ParseError::Semantic(semantic) = error else {
        return false;
    };
    matches!(semantic.code, ErrorCode::UnknownArrow)
}

/// The declared arrow nodes (`/schema/arrows/{i}`), keyed by name.
fn arrow_positions(schema: Option<&RawSchema>, spans: &Spans) -> BTreeMap<String, (usize, usize)> {
    let Some(schema) = schema else {
        return BTreeMap::new();
    };
    schema
        .arrows
        .iter()
        .enumerate()
        .filter_map(|(i, arrow)| {
            spans
                .position(&format!("/schema/arrows/{i}"))
                .map(|pos| (arrow.name.clone(), pos))
        })
        .collect()
}

/// The valued table cells (`/instance/tables/{arrow}/{row}`), keyed by
/// (arrow, source row name).
fn cell_positions(
    instance: Option<&super::schema::RawInstance>,
    spans: &Spans,
) -> BTreeMap<(String, String), (usize, usize)> {
    let Some(instance) = instance else {
        return BTreeMap::new();
    };
    let mut positions = BTreeMap::new();
    for (arrow, table) in &instance.tables {
        for row in table.keys() {
            if let Some(pos) = spans.position(&format!("/instance/tables/{arrow}/{row}")) {
                positions.insert((arrow.clone(), row.clone()), pos);
            }
        }
    }
    positions
}
