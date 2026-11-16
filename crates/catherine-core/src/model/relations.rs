//! Purpose: the two declarative constraint relations over arrow tuples —
//! keys (the mono side, format 1.3) and coproducts (the colimit side,
//! format 1.5). Split out of `model.rs` so the model module stays inside
//! the repo's file-size gate.
//!
//! Responsibilities: define [`Key`] and [`Coproduct`] and their canonical
//! sort keys, `(object, arrows, status)`. Both are parsed-and-validated
//! shapes: the parser guarantees non-empty, duplicate-free, declared
//! arrows with the relation's own typing, so the types carry no validity
//! re-check.
//!
//! Rationale: one declaration grammar means one canonicalization path —
//! sharing the module keeps the two `sort_key` implementations and their
//! status rule visibly identical.

use super::{ConstraintStatus, Ident};

/// A declarative key: injectivity of an object's arrow-value tuple over
/// that object's rows (format 1.3, `add-declarative-keys`). The arrows
/// are generating morphisms sourced at `object`; the parser guarantees
/// non-empty, duplicate-free, declared, and source-typed, so the type
/// carries no validity re-check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// The object the injectivity claim ranges over.
    pub object: Ident,
    /// The key's arrows, in the declared order (the tuple's slots).
    pub arrows: Vec<Ident>,
    /// Accepted keys are executed constraints; proposed keys are
    /// questions (the equation status rule, verbatim).
    pub status: ConstraintStatus,
}

impl Key {
    /// The canonical sort key — `(object, arrows, status)`. Key list
    /// order in the input file is presentation (design Decision 4): the
    /// parser stores keys sorted by this key and the serializer emits
    /// them in the same order defensively.
    pub(crate) fn sort_key(&self) -> (&Ident, &Vec<Ident>, &'static str) {
        (&self.object, &self.arrows, self.status.as_str())
    }
}

/// A declared coproduct (format 1.5, openspec change
/// `add-coproduct-exhaustiveness`): the variant set of `object` over the
/// injection arrows `V_k → object`. The declaration asserts coverage
/// (every row of `object` lies in some injection's image) and
/// disjointness (no row lies in two images). The arrows are declared
/// generating morphisms targeted at `object`; the parser guarantees the
/// list non-empty, duplicate-free, declared, and injective-shaped, so
/// the type carries no validity re-check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coproduct {
    /// The object the coverage and disjointness claims range over.
    pub object: Ident,
    /// The injection arrows, in the declared order.
    pub arrows: Vec<Ident>,
    /// Accepted coproducts are executed constraints; proposed ones are
    /// questions (the equation and key status rule, verbatim).
    pub status: ConstraintStatus,
}

impl Coproduct {
    /// The canonical sort key — `(object, arrows, status)`. Coproduct
    /// list order in the input file is presentation (design Decision 4):
    /// the parser stores coproducts sorted by this key and the serializer
    /// emits them in the same order defensively.
    pub(crate) fn sort_key(&self) -> (&Ident, &Vec<Ident>, &'static str) {
        (&self.object, &self.arrows, self.status.as_str())
    }
}
