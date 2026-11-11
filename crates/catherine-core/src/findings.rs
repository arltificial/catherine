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
//! [`DanglingWitness`]). Kind names the *shape* of the fact; the label
//! names its *weight*: an accepted-equation failure and a proposed-
//! equation failure are both violation-shaped findings, countable apart
//! by label, never merged into one severity. The analysis kinds
//! ([`FindingKind::Redundant`], [`FindingKind::Contradiction`],
//! [`FindingKind::Underconstraint`]) are defined and fixture-pinned here;
//! their producers are P4's (bounded analysis) — nothing in this crate
//! emits them. The underconstraint witness's
//! [`DomainQuestion::UncoveredInputs`] arm is reserved for P8's rule-lens
//! gap mapping: defined here, with no v1 producer.
//!
//! Ordering is total and documented: by kind (variant declaration order
//! of [`FindingKind`]), then witness ids (canonical id order inside each
//! witness), with the label as the final tiebreak — `check` sorts its
//! result under this order, so repeated checks of the same model
//! serialize byte-identically. Serialization is derived serde JSON with
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

use crate::model::{Ident, Path, RowId};
use serde::{Deserialize, Serialize};

/// How a finding was established — its epistemic weight. Exactly three
/// variants are constructible: `executed` (kernel-computed results),
/// `hypothesized` (domain questions: proposed-equation failures,
/// underconstraint), `unknown` (scope-bounded non-answers). `proved` and
/// `illustrated` are deliberately absent: they belong to later layers,
/// if ever, and the kernel never fabricates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicLabel {
    /// Established by kernel execution (accepted-equation failure,
    /// dangling cell): the kernel computed the fact itself.
    Executed,
    /// Raised as a domain question (proposed-equation failure,
    /// underconstraint): the fact marks a question under review, not a
    /// settled failure.
    Hypothesized,
    /// A scope-bounded non-answer: analysis that reports what it could
    /// not decide within its bounds. Reserved for the analysis layer
    /// (P4); no producer exists in this crate.
    Unknown,
}

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

/// The contradiction witness: the explicit scope — the equations
/// (canonical indices, in order) whose joint enforcement the bounded
/// analysis (P4) found to admit no instance.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContradictionWitness {
    /// Canonical indices into `Schema::equations`, in order.
    pub scope: Vec<usize>,
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
    /// A path equation failed on a row: the fact's shape for both
    /// epistemic weights — `executed` when the failed equation is
    /// accepted, `hypothesized` when it is proposed. The kind names the
    /// shape; the label names the weight.
    Violation(EquationWitness),
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
