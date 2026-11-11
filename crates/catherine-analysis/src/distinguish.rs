//! Purpose: bounded behavioral distinguishability of two instances over
//! an explicit observation set (openspec change `add-bounded-analysis`,
//! task 10) — `distinguishable` naming the differing observation and
//! both values, or `equivalent-in-scope`.
//!
//! Responsibilities: evaluate each observation (a path composed from a
//! named start row) on both typed instances and report the first
//! differing observation — `Some` vs `Some` (different rows), or `Some`
//! vs `None` (one instance cannot resolve the behavior there; a missing
//! row or a dangling cell is an honest "no value" — the documented
//! limit: it reads as `None`, not a separate report) — or
//! declare the pair equivalent *in scope*. The observation set is part
//! of the result, echoed sorted and deduplicated.
//!
//! Rationale: this is an analysis *result type*, not a `Finding` — the
//! P2 taxonomy has no distinguish kind and stays closed (no new finding
//! kinds, hard constraint on the change). Schema equivalence is
//! undecidable and out of scope by name: the verdict holds exactly over
//! the echoed observation set — a scope-limited result, not a claim about
//! the schemas or the instances beyond it. Determinism: the observation
//! set is stored in a `BTreeSet`, so the walk order (and thus which
//! difference is named first) is fixed by the `(object, row, path)`
/// sort, not by caller order.
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Path, RowId};
use std::collections::BTreeSet;

/// One behavioral observation: compose `path` from `row` of `object`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Observation {
    /// The object whose row the observation starts from.
    pub object: Ident,
    /// The start row (dense id within the object's space).
    pub row: RowId,
    /// The path to compose.
    pub path: Path,
}

/// The distinguish outcome: a witnessed difference or equivalence in
/// scope. Nothing else — no schema claim exists at this layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistinguishOutcome {
    /// The named observation composes to different values on the two
    /// instances (`left` = first instance, `right` = second; `None` =
    /// unresolvable there).
    Distinguishable {
        /// The differing observation (sorted-set order).
        observation: Observation,
        /// The first instance's value.
        left: Option<RowId>,
        /// The second instance's value.
        right: Option<RowId>,
    },
    /// Every observation in the set composes identically on both
    /// instances — in scope, and no claim beyond it.
    EquivalentInScope,
}

/// The distinguish result: the observation set (part of the result,
/// sorted and deduplicated) and the outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistinguishReport {
    /// The observation set the verdict holds over, sorted.
    pub observations: Vec<Observation>,
    /// The outcome within that scope.
    pub outcome: DistinguishOutcome,
}

/// Decide bounded behavioral distinguishability of two typed instances
/// over an explicit observation set. See the module docs.
pub fn distinguish(
    left: &TypedInstance,
    right: &TypedInstance,
    observations: &[Observation],
) -> DistinguishReport {
    let set: BTreeSet<Observation> = observations.iter().cloned().collect();
    let ordered: Vec<Observation> = set.into_iter().collect();
    let mut outcome = DistinguishOutcome::EquivalentInScope;
    for observation in &ordered {
        let left_value = left.compose(&observation.path, observation.row);
        let right_value = right.compose(&observation.path, observation.row);
        if left_value != right_value {
            outcome = DistinguishOutcome::Distinguishable {
                observation: observation.clone(),
                left: left_value,
                right: right_value,
            };
            break;
        }
    }
    DistinguishReport {
        observations: ordered,
        outcome,
    }
}
