//! Purpose: the composition's equation and key merge — one resolver over
//! the pushout's arrows, in the parser's own order.
//!
//! Responsibilities: resolve every input's equations (each document's
//! pending list over the merged arrows, a standalone document's already-
//! resolved list) and keep every input's equations, keys, coproducts, and
//! pullbacks as an enforced multiset in canonical order, with the
//! per-declaration source attribution.
//!
//! Rationale: design Decision 3 — the resolution function is the parser's
//! `check_equations`, called with the document's own arrow map for a
//! standalone document and with the pushout's for a seam document, so a
//! joint run reports the same first equation error, with the same code,
//! position, and message, that a lone `parse` does. Documents resolve in
//! input order (the order their parse errors surface in) and the merged
//! equations are then stored in canonical order, exactly as one document's
//! parse stores them.

use super::{
    CompositionError, CoproductSource, Document, EquationSource, KeySource, PullbackSource,
};
use crate::model::{Arrow, Coproduct, Equation, Ident, Key, Path, Pullback};
use std::collections::BTreeMap;

/// Merge every input's equations in canonical order, resolving each
/// document's paths over the merged arrows first.
pub(super) fn merge_equations(
    documents: &[Document],
    arrows: &BTreeMap<Ident, Arrow>,
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> Result<(Vec<Equation>, Vec<EquationSource>), CompositionError> {
    let resolved: Vec<Vec<(Equation, EquationSource)>> = documents
        .iter()
        .map(|doc| {
            doc.resolved_equations(arrows).map(|equations| {
                equations
                    .into_iter()
                    .map(|equation| {
                        let source = equation_source(&doc.name, &equation, arrow_sources);
                        (equation, source)
                    })
                    .collect()
            })
        })
        .collect::<Result<_, _>>()?;
    let pairs = sorted(resolved.into_iter().flatten().collect(), |a, b| {
        a.sort_key().cmp(&b.sort_key())
    });
    Ok(pairs.into_iter().unzip())
}

/// The pairs in the items' canonical sort-key order (stable for equal
/// keys, matching the parser's storage discipline).
fn sorted<T, S>(mut pairs: Vec<(T, S)>, key: impl Fn(&T, &T) -> std::cmp::Ordering) -> Vec<(T, S)> {
    pairs.sort_by(|(a, _), (b, _)| key(a, b));
    pairs
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
        .fold(Vec::new(), |sources, documents| {
            extend_unique(sources, documents)
        })
}

/// Append the elements of `source` not already present, preserving order.
fn extend_unique(mut documents: Vec<String>, source: &[String]) -> Vec<String> {
    for doc in source {
        if !documents.contains(doc) {
            documents.push(doc.clone());
        }
    }
    documents
}

/// Keys are never merged away: every declaration is kept in canonical
/// order and attributed to its declaring document.
pub(super) fn merge_keys(
    documents: &[Document],
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> (Vec<Key>, Vec<KeySource>) {
    let pairs = sorted(
        documents
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
            .collect(),
        |a, b| a.sort_key().cmp(&b.sort_key()),
    );
    pairs.into_iter().unzip()
}

/// Coproducts are never merged away: every declaration is kept in
/// canonical order and attributed to its declaring document (the keys
/// rule, reused verbatim).
pub(super) fn merge_coproducts(
    documents: &[Document],
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> (Vec<Coproduct>, Vec<CoproductSource>) {
    let pairs = sorted(
        documents
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
            .collect(),
        |a, b| a.sort_key().cmp(&b.sort_key()),
    );
    pairs.into_iter().unzip()
}

/// Pullbacks are never merged away: every declaration is kept in
/// canonical order and attributed to its declaring document (the keys
/// rule, reused verbatim). The attributed arrows are the unique union of
/// the legs and then the cospan, in first-declaration order.
pub(super) fn merge_pullbacks(
    documents: &[Document],
    arrow_sources: &BTreeMap<Ident, Vec<String>>,
) -> (Vec<Pullback>, Vec<PullbackSource>) {
    let pairs = sorted(
        documents
            .iter()
            .flat_map(|doc| {
                doc.model.schema.pullbacks.iter().map(move |pullback| {
                    let arrows = extend_unique(
                        path_sources(&Path(pullback.legs.clone()), arrow_sources),
                        &path_sources(&Path(pullback.cospan.clone()), arrow_sources),
                    );
                    (
                        pullback.clone(),
                        PullbackSource {
                            declaring: doc.name.clone(),
                            arrows,
                        },
                    )
                })
            })
            .collect(),
        |a, b| a.sort_key().cmp(&b.sort_key()),
    );
    pairs.into_iter().unzip()
}
