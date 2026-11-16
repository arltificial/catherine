//! Purpose: the composition interface check — whether the inputs share any
//! declared identity at all.
//!
//! Responsibilities: the empty-interface predicate and the namespaced
//! declared-identity sets: objects, arrows, and `(object, row)` pairs.
//!
//! Rationale: a composition's interface is namespaced — a row is
//! `(object, row)`, not a bare name, so `Alpha.x` and `Beta.x` are two
//! declarations, and an arrow called `x` is not an interface with a row
//! called `x`. Comparing one flat string set would suppress the warning on
//! exactly the typo-shaped disjoint unions it exists for. A disjoint union
//! proceeds but is never silent; a singleton glue is not a composition and
//! is never flagged.

use super::Document;
use crate::model::Ident;
use std::collections::BTreeSet;

/// A composition with two or more inputs sharing no declared identity is a
/// disjoint union; a singleton glue is never flagged.
pub(super) fn empty_interface(documents: &[Document]) -> bool {
    documents.len() > 1 && !shares_an_identity(documents)
}

/// One document's declared identities, namespaced by kind.
struct Interface {
    objects: BTreeSet<Ident>,
    arrows: BTreeSet<Ident>,
    rows: BTreeSet<(Ident, Ident)>,
}

/// Whether every input shares at least one object, arrow, or `(object,
/// row)` pair — the intersection is taken per namespace and then combined,
/// so cross-kind name reuse is never an interface.
fn shares_an_identity(documents: &[Document]) -> bool {
    let mut shared = interface_of(&documents[0]);
    for doc in &documents[1..] {
        shared.retain_shared_with(&interface_of(doc));
    }
    !(shared.objects.is_empty() && shared.arrows.is_empty() && shared.rows.is_empty())
}

impl Interface {
    /// Narrow every namespace to the identities this document also
    /// declares.
    fn retain_shared_with(&mut self, other: &Self) {
        self.objects = self.objects.intersection(&other.objects).cloned().collect();
        self.arrows = self.arrows.intersection(&other.arrows).cloned().collect();
        self.rows = self.rows.intersection(&other.rows).cloned().collect();
    }
}

/// Every declared identity in one document: objects, arrows, and rows
/// keyed by their declaring object.
fn interface_of(doc: &Document) -> Interface {
    let objects = doc.model.schema.objects.keys().cloned().collect();
    let arrows = doc.model.schema.arrows.keys().cloned().collect();
    let rows = doc
        .rows
        .iter()
        .flat_map(|(object, names)| names.iter().map(move |name| (object.clone(), name.clone())))
        .collect();
    Interface {
        objects,
        arrows,
        rows,
    }
}
