//! Purpose: the composition's equation and key merge — one resolver over
//! the pushout's arrows, in the parser's own order.
//!
//! Responsibilities: resolve every input's equations (each document's
//! pending list over the merged arrows, a standalone document's already-
//! resolved list) and keep every input's equations and keys as an
//! enforced multiset in canonical order, with the per-declaration source
//! attribution.
//!
//! Rationale: design Decision 3 — the resolution function is the parser's
//! `check_equations`, called with the document's own arrow map for a
//! standalone document and with the pushout's for a seam document, so a
//! joint run reports the same first equation error, with the same code,
//! position, and message, that a lone `parse` does. Documents resolve in
//! input order (the order their parse errors surface in) and the merged
//! equations are then stored in canonical order, exactly as one document's
//! parse stores them.

use super::{CompositionError, CoproductSource, Document, EquationSource, KeySource};
use crate::model::{Arrow, Coproduct, Equation, Ident, Key, Path};
use std::collections::BTreeMap;

/// Merge every input's equations in canonical order, resolving each
/// document's paths over the merged arrows first.
pub(super) fn merge_equations(
    documents: &[Document],
    arrows: &BTreeMap<Ident, Arrow>,
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> Result<(Vec<Equation>, Vec<EquationSource>), CompositionError> {
    let mut pairs: Vec<(Equation, EquationSource)> = Vec::new();
    for doc in documents {
        for equation in doc.resolved_equations(arrows)? {
            let source = equation_source(&doc.name, &equation, arrow_sources);
            pairs.push((equation, source));
        }
    }
    pairs.sort_by(|(a, _), (b, _)| a.sort_key().cmp(&b.sort_key()));
    Ok(pairs.into_iter().unzip())
}

/// The attribution of one resolved equation: its declaring document and
/// the documents behind each path.
fn equation_source(
    declaring: &str,
    equation: &Equation,
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> EquationSource {
    EquationSource {
        declaring: declaring.to_string(),
        lhs: path_sources(&equation.lhs, arrow_sources),
        rhs: path_sources(&equation.rhs, arrow_sources),
    }
}

/// The union of the documents declaring a path's arrows, first-declaration
/// order.
fn path_sources(path: &Path, arrow_sources: &BTreeMap<Ident, Vec<String>>) -> Vec<String> {
    path.0
        .iter()
        .filter_map(|name| arrow_sources.get(name))
        .fold(Vec::new(), |mut sources, documents| {
            extend_unique(&mut sources, documents);
            sources
        })
}

/// Append the elements of `source` not already present, preserving order.
fn extend_unique(documents: &mut Vec<String>, source: &[String]) {
    for doc in source {
        if !documents.contains(doc) {
            documents.push(doc.clone());
        }
    }
}

/// Keys are never merged away: every declaration is kept in canonical
/// order and attributed to its declaring document.
pub(super) fn merge_keys(
    documents: &[Document],
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> (Vec<Key>, Vec<KeySource>) {
    let mut pairs: Vec<(Key, KeySource)> = documents
        .iter()
        .flat_map(|doc| {
            doc.model.schema.keys.iter().map(move |key| {
                (
                    key.clone(),
                    KeySource {
                        declaring: doc.name.clone(),
                        arrows: path_sources(&Path(key.arrows.clone()), arrow_sources),
                    },
                )
            })
        })
        .collect();
    pairs.sort_by(|(a, _), (b, _)| a.sort_key().cmp(&b.sort_key()));
    pairs.into_iter().unzip()
}

/// Coproducts are never merged away: every declaration is kept in
/// canonical order and attributed to its declaring document (the keys
/// rule, reused verbatim).
pub(super) fn merge_coproducts(
    documents: &[Document],
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> (Vec<Coproduct>, Vec<CoproductSource>) {
    let mut pairs: Vec<(Coproduct, CoproductSource)> = documents
        .iter()
        .flat_map(|doc| {
            doc.model.schema.coproducts.iter().map(move |coproduct| {
                (
                    coproduct.clone(),
                    CoproductSource {
                        declaring: doc.name.clone(),
                        arrows: path_sources(&Path(coproduct.arrows.clone()), arrow_sources),
                    },
                )
            })
        })
        .collect();
    pairs.sort_by(|(a, _), (b, _)| a.sort_key().cmp(&b.sort_key()));
    pairs.into_iter().unzip()
}
