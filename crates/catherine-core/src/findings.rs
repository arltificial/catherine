//! Purpose: the public findings taxonomy (openspec change
//! `add-findings-taxonomy`, beads `catherine-1x7`) — the output contract
//! every later phase prints or consumes: typed finding kinds with
//! mandatory witness payloads, an epistemic label on every finding,
//! deterministic ordering, and stable serde JSON.
//!
//! Responsibilities: define [`Finding`] (label + kind), [`FindingKind`]
//! (the five v1 kinds, each carrying its concrete witness — a finding
//! without a witness is unrepresentable), [`EpistemicLabel`] (exactly
//! `executed` / `hypothesized` / `unknown` constructible; `proved` and
//! `illustrated` do not exist in the type, so the kernel cannot fabricate
//! them — those labels belong to later layers, if ever), and the shared
//! witness payloads the kernel walk constructs once ([`EquationWitness`],
//! [`KeyWitness`], the [`ViolationWitness`] union, [`DanglingWitness`],
//! [`PullbackWitness`]).
//! Kind names the *shape* of the fact; the label names its *weight*: an
//! accepted-equation failure and a proposed-equation failure are both
//! violation-shaped findings, countable apart by label, never merged into
//! one severity — and a key collision is the same violation shape with
//! the key witness, never a sixth kind. The analysis kinds
//! ([`FindingKind::Redundant`], [`FindingKind::Contradiction`],
//! [`FindingKind::Underconstraint`]) are defined and fixture-pinned here;
//! their producers are P4's (bounded analysis) — nothing in this crate
//! emits them. The underconstraint witness's
//! [`DomainQuestion::UncoveredInputs`] arm is reserved for P8's rule-lens
//! gap mapping: defined here, with no v1 producer.
//!
//! Ordering is total and documented: by kind (variant declaration order
//! of [`FindingKind`]), then witness ids (canonical id order inside each
//! witness; within a violation, the witness variant order — equations
//! before keys before naturality before coproducts before pullbacks —
//! then ids), with the label as the final tiebreak —
//! `check` sorts its result under this order, so repeated checks of the
//! same model serialize byte-identically. Sorted findings make accumulation
//! order-free: per-equation batches of the same model concatenate as a
//! monoid (see `checks.rs` for the sub-model boundary). Serialization is
//! derived serde JSON with
//! stable field order (declaration order) and ordered data only (no
//! `HashMap` in any serialized type): P3 prints this JSON verbatim and
//! the wasm boundary (P5) consumes it — the encoding is a promise, not an
//! implementation detail. Parsed-JSON shapes: `{"label": …, "kind": …,
//! "witness": {…}}` per finding; the underconstraint witness flattens its
//! tagged domain-question union as `{"case": …, "detail": {…},
//! "question": …}` (structural witness first, question text last).
//!
//! Rationale: P1's `CheckOutcome` enum evolved into this taxonomy
//! (replaced, not wrapped) — `check` now emits `Vec<Finding>`, with P1's
//! `Hypothesis` kind re-shaped as a violation carrying the `hypothesized`
//! label (the witness data was identical; only the epistemic weight
//! differed, and the label names weight). Witnesses live in shared
//! structs so the kernel walk constructs each witness once; instance
//! build reports ([`crate::instance::TypingViolation`],
//! [`crate::instance::DanglingCell`]) stay build reports, not findings.
//! The pullback arm is an additive member of the same union
//! (`add-pullback-checks`, format 1.6), not a new kind.

use crate::model::{Ident, Path, RowId};
use serde::{Deserialize, Serialize};

/// How a finding was established — its epistemic weight. Exactly three
/// variants are constructible: `executed` (kernel-computed results),
/// `hypothesized` (domain questions: proposed-equation failures,
/// underconstraint), `unknown` (scope-bounded non-answers). `proved` and
/// `illustrated` are deliberately absent: they belong to later layers,
/// if ever, and the kernel never fabricates them.
pub use catherine_engine::findings::EpistemicLabel;

/// The violation witness: an equation that failed on a row, with both
/// composed path values. Shared by the kernel walk and the taxonomy, so
/// each failure constructs its witness once.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EquationWitness {
    /// The equation's index in `Schema::equations` (canonical order).
    pub equation: usize,
    /// The row of the equation's domain object that failed.
    pub row: RowId,
    /// The composed `lhs` path value.
    pub lhs: RowId,
    /// The composed `rhs` path value.
    pub rhs: RowId,
}

/// The key witness: an accepted or proposed key whose arrow tuple agrees
/// on two rows of its object. The key is named by its canonical index
/// (as [`EquationWitness`] names an equation); both colliding rows are
/// carried, ascending. The shared tuple values are not duplicated — they
/// are recoverable from the instance, and carrying them would make the
/// witness instance-dependent while the key declaration is not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeyWitness {
    /// The key's index in `Schema::keys` (canonical order).
    pub key: usize,
    /// The two colliding rows of the key's object, ascending.
    pub rows: [RowId; 2],
}

/// The naturality witness: a declared instance map's square fails for one
/// arrow on one source row. The arrow, the source row (dense id of the
/// arrow's source object), and both composed values are carried; a
/// composition that cannot resolve (a dangling cell on either side) is
/// `None`, rendered as a non-answer, never silently skipped.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NaturalityWitness {
    /// The arrow whose commutation square breaks.
    pub arrow: Ident,
    /// The source row of the arrow's source object.
    pub source_row: RowId,
    /// `map ∘ arrow` on the source row — `None` when unresolved.
    pub lhs: Option<RowId>,
    /// `arrow ∘ map` on the source row — `None` when unresolved.
    pub rhs: Option<RowId>,
}

/// The coproduct witness: a declared coproduct whose coverage or
/// disjointness claim failed on one row of its object. The declaration
/// is named by its canonical index (as [`EquationWitness`] names an
/// equation), so the witness stays declaration-shaped. `injections` is
/// empty when the row lies in no injection image (the coverage failure)
/// and names the colliding injections, in declared order, when it lies in
/// more than one (the disjointness failure); an empty list is omitted
/// from the JSON encoding, so the coverage witness carries no misleading
/// empty field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CoproductWitness {
    /// The coproduct's index in `Schema::coproducts` (canonical order).
    pub coproduct: usize,
    /// The row of the coproduct's object that failed.
    pub row: RowId,
    /// The injections covering the row — empty for a coverage failure.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub injections: Vec<Ident>,
}

/// The pullback witness: a declared pullback square whose universal
/// property failed for one verdict. The declaration is named by its
/// canonical index (as [`EquationWitness`] names an equation), and the
/// offender is verdict-specific: an apex row with both composed values
/// (commutes), an uncovered fiber pair (covers), or two apex rows sharing
/// a projection (unique). The arms serialize untagged; their field sets
/// (`pullback`/`row`/`lhs`/`rhs`, `pullback`/`pair`,
/// `pullback`/`rows`) are disjoint from each other and from every other
/// violation witness, so the encoding stays unambiguous without
/// consulting the schema.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PullbackWitness {
    /// A square row's two composed cospan values disagree.
    Commutes {
        /// The pullback's index in `Schema::pullbacks` (canonical order).
        pullback: usize,
        /// The failing apex row.
        row: RowId,
        /// The composed `f ∘ p1` value.
        lhs: RowId,
        /// The composed `g ∘ p2` value.
        rhs: RowId,
    },
    /// No apex row projects to this fiber pair — the square under the
    /// cospan.
    Covers {
        /// The pullback's index in `Schema::pullbacks` (canonical order).
        pullback: usize,
        /// The uncovered pair: an `A` row and a `B` row sharing a cospan
        /// target.
        pair: [RowId; 2],
    },
    /// Two apex rows project to the same fiber pair — the legs are not
    /// jointly injective.
    Unique {
        /// The pullback's index in `Schema::pullbacks` (canonical order).
        pullback: usize,
        /// The two duplicated-projecting apex rows, ascending.
        rows: [RowId; 2],
    },
}

impl PullbackWitness {
    /// The declaration's canonical index, shared by every verdict shape.
    pub fn pullback(&self) -> usize {
        match self {
            PullbackWitness::Commutes { pullback, .. }
            | PullbackWitness::Covers { pullback, .. }
            | PullbackWitness::Unique { pullback, .. } => *pullback,
        }
    }
}

/// The violation witness's shapes: an equation failure, a key collision,
/// a naturality square, a coproduct coverage/disjointness failure, or a
/// pullback square's commutes/covers/unique failure (design Decision 3;
/// the naturality arm is the additive 1.4 shape from
/// `add-instance-morphism`, the coproduct arm the additive 1.5 shape from
/// `add-coproduct-exhaustiveness`, the pullback arm the additive 1.6
/// shape from `add-pullback-checks`). Serialized untagged — the
/// serialized field sets are disjoint (`equation`/`row`/`lhs`/`rhs`,
/// `key`/`rows`, `arrow`/`source_row`/`lhs`/`rhs`,
/// `coproduct`/`row`/`injections`,
/// `pullback`/`row`/`lhs`/`rhs`, `pullback`/`pair`,
/// `pullback`/`rows`), so the equation JSON stays byte-identical and
/// deserialization is unambiguous. The variant order is the taxonomy's
/// kind-internal order: equations before keys before naturality before
/// coproducts before pullbacks.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ViolationWitness {
    /// A path equation failed on a row.
    Equation(EquationWitness),
    /// A key's arrow tuple collided on two rows.
    Key(KeyWitness),
    /// An instance map's commutation square failed.
    Naturality(NaturalityWitness),
    /// A declared coproduct's coverage or disjointness claim failed.
    Coproduct(CoproductWitness),
    /// A declared pullback's commutes, covers, or unique verdict failed.
    Pullback(PullbackWitness),
}

/// The dangling witness: an arrow cell that holds no row, naming the
/// arrow and the source row the dangling value sits on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DanglingWitness {
    /// The arrow whose cell is unresolved.
    pub arrow: Ident,
    /// The source row the dangling value sits on.
    pub src: RowId,
}

/// The redundancy witness: a declared path whose equation is derivable
/// from the others, and the path it reduces to under them (P4's
/// normal-form rewriting computes this; the taxonomy pins the shape).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RedundantWitness {
    /// The path that is derivable — the redundant equation side.
    pub derivable: Path,
    /// The path the derivable one reduces to under the remaining
    /// equations.
    pub reducing: Path,
}

/// The contradiction witness: the explicit scope — every accepted
/// constraint family (canonical indices per family, in declaration
/// order) whose joint enforcement the bounded analysis (P4) found to
/// admit no instance. The family order (equations, keys, coproducts,
/// pullbacks) is also the derived ordering's witness-id order. An empty
/// family list is omitted from the JSON encoding, so an equation-only
/// contradiction stays byte-identical.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContradictionWitness {
    /// Canonical indices into `Schema::equations`, in order.
    pub scope: Vec<usize>,
    /// Canonical indices into `Schema::keys`, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<usize>,
    /// Canonical indices into `Schema::coproducts`, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coproducts: Vec<usize>,
    /// Canonical indices into `Schema::pullbacks`, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pullbacks: Vec<usize>,
}

/// The domain-question witness of an underconstraint finding — the
/// tagged union the spec fixes: two parallel paths the schema never
/// forces to agree, or a class of inputs no equation covers
/// ([`DomainQuestion::UncoveredInputs`] is reserved for P8's rule-lens
/// gap mapping: defined here, never produced in v1). Serialized
/// adjacently tagged as `{"case": …, "detail": {…}}`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "case", content = "detail", rename_all = "snake_case")]
pub enum DomainQuestion {
    /// Parallel paths the schema never forces to agree.
    ParallelPaths {
        /// The first parallel path.
        lhs: Path,
        /// The second parallel path.
        rhs: Path,
    },
    /// A class of inputs no equation covers, with example rows (rows in
    /// their object's dense id space).
    UncoveredInputs {
        /// The uncovered input class, as a human-readable name.
        class: String,
        /// Example rows of the class.
        examples: Vec<RowId>,
    },
}

/// The underconstraint witness: the tagged domain-question witness first
/// (the structural ids the spec names), then the domain question text.
/// Serialized flattened: `{"case": …, "detail": {…}, "question": …}` —
/// the order rule (kind, then witness ids) puts the structural witness
/// ahead of the free-form text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnderconstraintWitness {
    /// The tagged domain-question witness.
    #[serde(flatten)]
    pub case: DomainQuestion,
    /// The domain question, as text.
    pub question: String,
}

/// The finding kind with its mandatory witness payload. Variant
/// declaration order is the kind order of the taxonomy's total ordering;
/// no variant exists without its witness — a finding that cannot name
/// its witnesses is unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "witness", rename_all = "snake_case")]
pub enum FindingKind {
    /// A constraint failed: the fact's shape for both epistemic weights —
    /// `executed` when the failed constraint is accepted, `hypothesized`
    /// when it is proposed. The witness is the untagged union of the
    /// violation witnesses (equation, key, naturality, coproduct,
    /// pullback); the kind names the shape, the label names the weight.
    Violation(ViolationWitness),
    /// An arrow cell holds no row: paths traversing the cell
    /// short-circuit there; the affected row is never also a violation.
    Dangling(DanglingWitness),
    /// An equation derivable from the others (P4 producer; shape only
    /// here).
    Redundant(RedundantWitness),
    /// Constraints whose joint enforcement admits no instance (P4
    /// producer; shape only here).
    Contradiction(ContradictionWitness),
    /// The schema underconstrains its domain — a question, not a failure
    /// (P4 producer; shape only here).
    Underconstraint(UnderconstraintWitness),
}

/// A finding: the public diagnostic. The label rides on the finding as
/// independent data (violation is the one kind that maps to two labels);
/// the kind carries the witness. Ordered by kind (declaration order),
/// then witness ids, label last as tiebreak — see the module docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// How the finding was established.
    pub label: EpistemicLabel,
    /// The finding kind with its mandatory witness.
    #[serde(flatten)]
    pub kind: FindingKind,
}

impl PartialOrd for Finding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Kind first (declaration order, then witness ids), label last: two
/// findings of the same kind and witness order by label — a total order,
/// so repeated checks sort identically.
impl Ord for Finding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind
            .cmp(&other.kind)
            .then_with(|| self.label.cmp(&other.label))
    }
}
