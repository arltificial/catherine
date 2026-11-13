//! Purpose: the parsed model document's kernel types — the DESIGN §4.1
//! sketch, verbatim in shape.
//!
//! Responsibilities: represent a parsed `.ct.json` v1 document (schema:
//! objects, arrows, path equations with rationale + status, and
//! declarative keys with status; instance: rows and per-arrow value
//! tables with `None` = dangling) as canonical data: every collection
//! keyed by `Ident` in a `BTreeMap`, so equal models are indistinguishable
//! from their key order in the input file (design Decision 4).
//!
//! Rationale: adapted from specodelic's acset-core contracts (`src/acset/
//! schema.rs` canonical sorted order and `(source, name)`-unique morphisms;
//! `src/acset/instance.rs` interning with dense, bidirectional id spaces
//! and partial morphism storage where an unresolved reference is a
//! representable value, never dropped). Row ids are dense per object
//! (sorted first-reference order), so presentation row names never reach
//! the model and the model hash depends on structure only.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A declared identifier. The parser guarantees `[A-Za-z_][A-Za-z0-9_]*`
/// (`invalid_ident` otherwise) — the type carries no validity re-check.
/// Serde-derived for the findings taxonomy's witness payloads, which name
/// real declared ids (a `String` transparently, per newtype).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ident(pub String);

/// A dense per-object row id (`0..n`, assigned in sorted first-reference
/// order over the object's declared row names). Presentation names never
/// reach the parsed model. Serde-derived for findings witnesses (as the
/// bare row number).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
/// composition order. Serde-derived for findings witnesses (as the
/// arrow-name sequence).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

/// The schema presentation: objects, generating arrows, path equations,
/// and declarative keys. `objects` maps each ident to `()` — the map
/// *is* the sorted name set (design Decision 4, adapted from acset-core
/// `canonical_order`). Equations are stored in canonical order (sorted by
/// `(lhs, rhs, rationale, status)`) and keys in canonical order (sorted
/// by `(object, arrows, status)`), so input file order never reaches the
/// model. The key list is empty in every pre-1.3 document; an empty key
/// list is omitted from canonical bytes (hash neutrality).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema {
    pub objects: BTreeMap<Ident, ()>,
    pub arrows: BTreeMap<Ident, Arrow>,
    pub equations: Vec<Equation>,
    pub keys: Vec<Key>,
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

/// A declared schema map (Δ migration axis, openspec change
/// `add-delta-migration` phase A): the total assignment of the document
/// schema's objects and arrows into the target schema. `target` is the
/// migration's destination schema, validated by the same passes as the
/// document schema; `objects` maps each declared source object to its
/// target object or `None` — an explicit drop, never a silent one;
/// `arrows` maps each declared source arrow to a composable target path
/// or `None`. Totality is checked at parse: every declared source object
/// and arrow carries an entry, and a missing entry is a labeled
/// `partial_schema_map` build error. The map is declared data only —
/// pullback/reindex execution is phase B, so no functoriality (path
/// endpoints against mapped endpoints) is enforced here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaMap {
    pub target: Schema,
    pub objects: BTreeMap<Ident, Option<Ident>>,
    pub arrows: BTreeMap<Ident, Option<Path>>,
}

/// One declared attribute column of the rule presentation's input domain
/// X (format 1.2, add-rule-lens phase A). `Int` values are `i64`, `Bool`
/// is a two-valued domain, and `String` ordering is byte-wise (UTF-8,
/// locale-independent, deterministic — design.md Decision 2: no collation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Int,
    Bool,
    String,
}

impl ColumnType {
    /// The document spelling (also the canonical sort key, though columns
    /// are keyed by name).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Bool => "bool",
            Self::String => "string",
        }
    }

    /// The declared type for a document spelling; `None` for anything
    /// outside the three-value enum.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "int" => Some(Self::Int),
            "bool" => Some(Self::Bool),
            "string" => Some(Self::String),
            _ => None,
        }
    }
}

/// A comparison operator of the rule presentation's predicate language
/// (the tiny decidable language of DESIGN §4.4: comparisons, ∧ ∨ ¬, mod —
/// deliberately not a logic, design.md Decision 2). Boolean columns admit
/// only `Eq`/`Ne`; int and string columns admit all six.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl CmpOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Lt => "lt",
            Self::Le => "le",
            Self::Gt => "gt",
            Self::Ge => "ge",
        }
    }

    /// The operator for a document spelling; `None` for an unknown one.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "eq" => Some(Self::Eq),
            "ne" => Some(Self::Ne),
            "lt" => Some(Self::Lt),
            "le" => Some(Self::Le),
            "gt" => Some(Self::Gt),
            "ge" => Some(Self::Ge),
            _ => None,
        }
    }
}

/// A predicate literal whose variant must agree with its column's
/// [`ColumnType`] (the parser enforces the agreement; the type carries no
/// re-check). Ordered so an exact input set can be canonically sorted and
/// deduplicated (variant order `Int < Bool < String`; within a column the
/// variant is fixed, so the order is the column's own).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    Int(i64),
    Bool(bool),
    String(String),
}

/// A decidable predicate over the declared attribute columns: a typed
/// comparison, an integer modulus test, or a boolean combination. This is
/// the whole language — every extension must re-justify decidability
/// (design.md Decision 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// `column op value`, with `value`'s variant matching the column type.
    Cmp {
        column: Ident,
        op: CmpOp,
        value: Literal,
    },
    /// `column mod modulus == equals`, on an int column with a positive
    /// modulus (non-negative remainder).
    Mod {
        column: Ident,
        modulus: i64,
        equals: i64,
    },
    /// Conjunction of one or more predicates.
    And(Vec<Predicate>),
    /// Disjunction of one or more predicates.
    Or(Vec<Predicate>),
    /// Negation.
    Not(Box<Predicate>),
}

/// One rule of a rule table: a name (witness-addressable), a decidable
/// predicate (the subobject R_i of X), and the action assigned on it. The
/// action is a required non-empty string — the action assignment must be a
/// total function on X, so a rule without one is a labeled build error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub name: Ident,
    pub predicate: Predicate,
    pub action: String,
}

/// The resolution policy of a rule table (design.md Decision 3: total and
/// conventional). There is no implicit policy: `None` on the presentation
/// means no declared precedence, never a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Lowest index wins on an intersection.
    PriorityOrder,
    /// Rule order as written wins.
    FirstMatch,
    /// Intersections are declared violations; there is no precedence.
    DeclaredDisjoint,
}

impl Policy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PriorityOrder => "PriorityOrder",
            Self::FirstMatch => "FirstMatch",
            Self::DeclaredDisjoint => "DeclaredDisjoint",
        }
    }

    /// The policy for a document spelling; `None` outside the enum.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "PriorityOrder" => Some(Self::PriorityOrder),
            "FirstMatch" => Some(Self::FirstMatch),
            "DeclaredDisjoint" => Some(Self::DeclaredDisjoint),
            _ => None,
        }
    }
}

/// A rule table presented over the input domain X (DESIGN §4.4; format
/// 1.2, add-rule-lens phase A). `domain` names the declared schema object
/// the columns describe; `columns` is the attribute alphabet keyed by name
/// (declaration order is presentation); `rules` is the ordered priority
/// chain — its index *is* the rule's order, so rule order is semantic and
/// never sorted away; `policy` is the declared resolution policy, absent
/// when the table declares none. Effective regions E_i = R_i \ ⋃_{j<i} R_j
/// are computed under a priority policy (analysis phase).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePresentation {
    pub domain: Ident,
    pub columns: BTreeMap<Ident, ColumnType>,
    pub rules: Vec<Rule>,
    pub policy: Option<Policy>,
}

/// A parsed `.ct.json` model document: version, schema, and instance.
/// `schema_map` is the additive 1.1 section and `rule_presentation` the
/// additive 1.2 section — both absent (`None`) in every earlier document,
/// and skip-serialized (so hash-neutral) when absent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Model {
    pub format_version: Version,
    pub schema: Schema,
    pub instance: Instance,
    pub schema_map: Option<SchemaMap>,
    pub rule_presentation: Option<RulePresentation>,
}

impl Model {
    /// The empty document: version 1.0, no objects/arrows/equations, no
    /// rows/tables (openspec scenario: empty schema + empty instance
    /// parses cleanly).
    pub fn empty() -> Self {
        Self::default()
    }
}
