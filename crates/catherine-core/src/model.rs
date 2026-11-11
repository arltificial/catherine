//! Purpose: the parsed model document's kernel types — the DESIGN §4.1
//! sketch, verbatim in shape.
//!
//! Responsibilities: represent a parsed `.ct.json` v1 document (schema:
//! objects, arrows, path equations with rationale + status; instance: rows
//! and per-arrow value tables with `None` = dangling) as canonical data:
//! every collection keyed by `Ident` in a `BTreeMap`, so equal models are
//! indistinguishable from their key order in the input file (design
//! Decision 4).
//!
//! Rationale: adapted from specodelic's acset-core contracts (`src/acset/
//! schema.rs` canonical sorted order and `(source, name)`-unique morphisms;
//! `src/acset/instance.rs` interning with dense, bidirectional id spaces
//! and partial morphism storage where an unresolved reference is a
//! representable value, never dropped). Row ids are dense per object
//! (sorted first-reference order), so presentation row names never reach
//! the model and the model hash depends on structure only.

use std::collections::BTreeMap;

/// A declared identifier. The parser guarantees `[A-Za-z_][A-Za-z0-9_]*`
/// (`invalid_ident` otherwise) — the type carries no validity re-check.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(pub String);

/// A dense per-object row id (`0..n`, assigned in sorted first-reference
/// order over the object's declared row names). Presentation names never
/// reach the parsed model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowId(pub usize);

/// The document's declared format version: `major.minor`. The parser
/// accepts additive minors of the current major and rejects unknown majors
/// or an absent field with `unsupported_version`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
}

impl Default for Version {
    fn default() -> Self {
        Self { major: 1, minor: 0 }
    }
}

/// Per-equation constraint status. P0 fixes the *format field only*:
/// required per equation in the model, `Accepted` when absent in the
/// document, validated to the two-value enum at parse. The semantic split
/// (accepted → violations, proposed → hypotheses) is a P1 concern
/// (`add-executed-checks`) and deliberately absent here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintStatus {
    Proposed,
    Accepted,
}

impl ConstraintStatus {
    /// The document spelling of the status (also the canonical sort key).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
        }
    }
}

/// A composable arrow sequence: the path `f ∘ g ∘ …` as its arrow names in
/// composition order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(pub Vec<Ident>);

/// A generating morphism, typed with exactly one src and one tgt
/// (acset-core `morphisms_typed`, adapted). Self-loops are legal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrow {
    pub src: Ident,
    pub tgt: Ident,
}

/// A path equation `lhs = rhs` with its rationale and constraint status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    pub lhs: Path,
    pub rhs: Path,
    pub rationale: String,
    pub status: ConstraintStatus,
}

impl Equation {
    /// The canonical sort key — `(lhs, rhs, rationale, status)`. Equation
    /// list order in the input file is presentation (design Decision 4):
    /// the parser stores equations sorted by this key and the serializer
    /// emits them in the same order defensively.
    pub(crate) fn sort_key(&self) -> (&Path, &Path, &String, &'static str) {
        (&self.lhs, &self.rhs, &self.rationale, self.status.as_str())
    }
}

/// The schema presentation: objects, generating arrows, and path
/// equations. `objects` maps each ident to `()` — the map *is* the sorted
/// name set (design Decision 4, adapted from acset-core `canonical_order`).
/// Equations are stored in canonical order (sorted by
/// `(lhs, rhs, rationale, status)`), so input file order never reaches the
/// model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema {
    pub objects: BTreeMap<Ident, ()>,
    pub arrows: BTreeMap<Ident, Arrow>,
    pub equations: Vec<Equation>,
}

/// The instance presentation: per-object dense row ids and one partial
/// value vector per arrow, parallel to the arrow's source object's rows —
/// `None` = dangling reference, a representable value that is never
/// dropped, repaired, or a parse error (acset-core `dangling_is_a_value`,
/// adapted). Every declared arrow carries a total vector over its source
/// object's rows; an unspecified cell parses as `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Instance {
    pub rows: BTreeMap<Ident, Vec<RowId>>,
    pub values: BTreeMap<Ident, Vec<Option<RowId>>>,
}

/// A parsed `.ct.json` model document: version, schema, and instance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Model {
    pub format_version: Version,
    pub schema: Schema,
    pub instance: Instance,
}

impl Model {
    /// The empty document: version 1.0, no objects/arrows/equations, no
    /// rows/tables (openspec scenario: empty schema + empty instance
    /// parses cleanly).
    pub fn empty() -> Self {
        Self::default()
    }
}
