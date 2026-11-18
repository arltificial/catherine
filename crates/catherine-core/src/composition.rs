//! Purpose: model composition at a seam (openspec change
//! `add-model-composition`, beads `catherine-0qz`) — the checked pushout
//! of parsed documents over their shared declared names.
//!
//! Responsibilities: [`parse_document`] parses one document with its
//! equation phase validated against the document's own arrows first and
//! only a genuinely cross-document claim deferred; [`glue`] folds two or
//! more documents into a merged [`Schema`] + [`Instance`] by declared
//! name, refusing a disagreeing overlap with a labeled
//! [`CompositionError`] (`schema_conflict` for arrow endpoints,
//! `row_conflict` for a shared object's row set or a shared row's value),
//! resolves every pending equation path over the merged arrows with the
//! parser's own resolver (a dangling element is the same `unknown_arrow` —
//! code, position, message — the single-document parser raises), and
//! reports an empty shared interface. Equations, keys, coproducts, and
//! pullbacks are enforced multisets in canonical order. The [`Glued`] side
//! tables carry per-declaration document attribution for the CLI's source
//! field.
//!
//! Rationale: design Decisions 2 and 3 — the pushout identifies shared
//! structure by declared name, presentation order never decides a merge,
//! and conflicts are labeled errors rather than last-writer-wins. Row
//! names, not dense ids, are the merge key: each document's normalized
//! dense ids are remapped through its [`RowNames`] side table into the
//! union's sorted dense space. A document that declares rows for a shared
//! object claims that row set, so two row claims must agree (a partial
//! instance is never silently unioned into a fabricated dangling cell) and
//! a shared cell's value — including the parser's `null` dangling value —
//! must agree. One resolver serves both paths: the document's own arrows
//! for a standalone document, the pushout's for a deferred one. This
//! module owns no presentation logic and emits findings only through the
//! kernel's existing checks.

use crate::error::ParseError;
use crate::model::{Arrow, Equation, Ident, Instance, Model, Schema};
use crate::parse::{parse_deferred, PendingEquations, RowNames};
use std::collections::BTreeMap;

mod instance;
mod interface;
mod merge;
mod resolve;

/// One parsed input document: its name (the attribution string), the
/// normalized model, the presentation row names needed to merge dense row
/// ids by name, the equations still awaiting the pushout, and the span
/// metadata the conflict reports cite.
#[derive(Debug, Clone)]
pub struct Document {
    name: String,
    model: Model,
    rows: RowNames,
    pending: Option<PendingEquations>,
    arrow_positions: BTreeMap<String, (usize, usize)>,
    cell_positions: BTreeMap<(String, String), (usize, usize)>,
}

/// Parse one document for composition: every document-level validation
/// runs, and only an equation path that names no arrow of this document is
/// carried to [`glue`] (design Decision 3). A document whose equations all
/// resolve here is validated in exactly the parser's order with the
/// parser's codes, so a joint run reports the same first error a lone
/// `parse` does.
pub fn parse_document(name: &str, text: &str) -> Result<Document, ParseError> {
    let parsed = parse_deferred(text)?;
    Ok(Document {
        name: name.to_string(),
        model: parsed.model,
        rows: parsed.rows,
        pending: parsed.pending,
        arrow_positions: parsed.arrow_positions,
        cell_positions: parsed.cell_positions,
    })
}

impl Document {
    /// The document's equations resolved over `arrows`: its own model's
    /// list when the equation phase already resolved at parse, otherwise
    /// the parser's own resolver run over the pushout.
    fn resolved_equations(
        &self,
        arrows: &BTreeMap<Ident, Arrow>,
    ) -> Result<Vec<Equation>, CompositionError> {
        let Some(pending) = &self.pending else {
            return Ok(self.model.schema.equations.clone());
        };
        pending
            .resolve(arrows)
            .map_err(|error| self.resolution_error(error))
    }

    /// The post-glue resolution failure in the parser's own shape: stable
    /// code, message, and offending-node position, attributed to the
    /// declaring document.
    fn resolution_error(&self, error: ParseError) -> CompositionError {
        let (code, message, position) = match error {
            ParseError::Semantic(semantic) => (
                semantic.code.as_str(),
                semantic.message,
                Some((semantic.line, semantic.column)),
            ),
            ParseError::Syntax {
                line,
                column,
                message,
            } => ("syntax", message, Some((line, column))),
        };
        let documents = vec![self.name.clone()];
        CompositionError::new(code, message, documents, position, None)
    }
}

/// A labeled composition failure: the stable code, the human message, the
/// documents the conflict or dangling reference involves, the offending
/// node's position when known, and the offending node's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionError {
    code: String,
    message: String,
    documents: Vec<String>,
    position: Option<(usize, usize)>,
    node: Option<String>,
}

impl CompositionError {
    /// The stable code: the composition conflicts (`schema_conflict`,
    /// `row_conflict`), or the parser's own code when an equation fails
    /// resolution over the pushout (`unknown_arrow`, `ill_typed_equation`,
    /// `invalid_ident`, `invalid_status`, `syntax`).
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The human message (the parse-time message shape for resolution
    /// failures).
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The documents the failure involves, in input order.
    pub fn documents(&self) -> Vec<&str> {
        self.documents.iter().map(String::as_str).collect()
    }

    /// The offending node's 1-based line/column, when the document span
    /// scan recorded one.
    pub fn position(&self) -> Option<(usize, usize)> {
        self.position
    }

    /// The offending node's name (arrow, row, or path element).
    pub fn node(&self) -> Option<&str> {
        self.node.as_deref()
    }

    fn new(
        code: &str,
        message: String,
        documents: Vec<String>,
        position: Option<(usize, usize)>,
        node: Option<String>,
    ) -> Self {
        Self {
            code: code.to_string(),
            message,
            documents,
            position,
            node,
        }
    }
}

/// The document that declared an equation and the documents declaring its
/// path elements — the CLI's source attribution for a violation witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquationSource {
    /// The declaring document.
    pub declaring: String,
    /// Documents declaring the lhs path's arrows, first-declaration order.
    pub lhs: Vec<String>,
    /// Documents declaring the rhs path's arrows, first-declaration order.
    pub rhs: Vec<String>,
}

/// The document that declared a key and the documents declaring its
/// arrows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySource {
    /// The declaring document.
    pub declaring: String,
    /// Documents declaring the key's arrows, first-declaration order.
    pub arrows: Vec<String>,
}

/// The document that declared a coproduct and the documents declaring its
/// injection arrows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoproductSource {
    /// The declaring document.
    pub declaring: String,
    /// Documents declaring the injection arrows, first-declaration order.
    pub arrows: Vec<String>,
}

/// The document that declared a pullback and the documents declaring its
/// legs and cospan arrows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullbackSource {
    /// The declaring document.
    pub declaring: String,
    /// Documents declaring the legs then the cospan arrows, the unique
    /// union in first-declaration order.
    pub arrows: Vec<String>,
}

/// The glued model: the union schema and instance plus the attribution
/// side tables and the empty-interface flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glued {
    /// The merged schema (objects/arrows by name, equations, keys,
    /// coproducts, and pullbacks as enforced multisets in canonical order).
    pub schema: Schema,
    /// The merged instance over the union's sorted dense row space.
    pub instance: Instance,
    /// True when two or more inputs share no declared identity — no
    /// object, no arrow, and no `(object, row)` pair (a disjoint union: the
    /// caller warns, never presents it as a seam check).
    pub empty_interface: bool,
    /// The input names, in input order.
    pub documents: Vec<String>,
    /// Per-equation attribution, aligned with `schema.equations`.
    pub equation_sources: Vec<EquationSource>,
    /// Per-key attribution, aligned with `schema.keys`.
    pub key_sources: Vec<KeySource>,
    /// Per-coproduct attribution, aligned with `schema.coproducts`.
    pub coproduct_sources: Vec<CoproductSource>,
    /// Per-pullback attribution, aligned with `schema.pullbacks`.
    pub pullback_sources: Vec<PullbackSource>,
    /// Per-arrow declaring documents, first-declaration order.
    pub arrow_sources: BTreeMap<Ident, Vec<String>>,
}

/// The schema-side merge products needed by the instance and attribution
/// passes.
struct SchemaParts {
    objects: BTreeMap<Ident, ()>,
    arrows: BTreeMap<Ident, Arrow>,
    arrow_sources: BTreeMap<Ident, Vec<String>>,
}

/// Glue the documents by declared name. A singleton glue is the identity on
/// its model — every dense id remaps to itself and the merged arrow set is
/// the document's own — so one resolver and one set of merge rules serve one
/// document or many.
pub fn glue(documents: &[Document]) -> Result<Glued, CompositionError> {
    let parts = merge::merge_schema(documents)?;
    let instance = instance::merge_instance(documents, &parts)?;
    let (equations, equation_sources) =
        resolve::merge_equations(documents, &parts.arrows, &parts.arrow_sources)?;
    let (keys, key_sources) = resolve::merge_keys(documents, &parts.arrow_sources);
    let (coproducts, coproduct_sources) =
        resolve::merge_coproducts(documents, &parts.arrow_sources);
    let (pullbacks, pullback_sources) = resolve::merge_pullbacks(documents, &parts.arrow_sources);
    Ok(Glued {
        schema: Schema {
            objects: parts.objects,
            arrows: parts.arrows,
            equations,
            keys,
            coproducts,
            pullbacks,
        },
        instance,
        empty_interface: interface::empty_interface(documents),
        documents: documents.iter().map(|doc| doc.name.clone()).collect(),
        equation_sources,
        key_sources,
        coproduct_sources,
        pullback_sources,
        arrow_sources: parts.arrow_sources,
    })
}
