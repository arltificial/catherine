//! Purpose: the shared bound/scope types and deduplicated witness
//! builders across the analysis ops (openspec change
//! `add-bounded-analysis`, REFACTOR task 12).
//!
//! Responsibilities: [`SearchBounds`] / [`SearchScope`] — the bound-in /
//! scope-out pair every bounded op speaks; [`ScopeCaps`] — the engine's
//! configurable caps (candidate budget, path length, rewrite steps) as
//! configured values rather than magic numbers scattered through the
//! engines, with the defaults documented in the crate README's
//! decidability-boundary section; and the witness builders — one
//! construction site per P2 analysis finding shape, so no op can drift
//! from the taxonomy's label/witness contract (the P8 rule lens's
//! `UncoveredInputs` gap builder is the fourth).
//!
//! Rationale: the honesty contract lives in types: bounds are inputs,
//! a scope names exactly what was explored, and every finding the ops
//! emit is built once here — `executed` for the proven results
//! (contradiction-by-completed-exhaustion, redundant-by-derivation),
//! `hypothesized` for the domain questions, with the P2 witness shapes
//! verbatim. No new finding kinds: `UncoveredInputs` was P8-reserved and
//! is now produced only by the rule lens, through the builder here.

use catherine_core::findings::{
    ContradictionWitness, DomainQuestion, EpistemicLabel, Finding, FindingKind, RedundantWitness,
    UnderconstraintWitness,
};
use catherine_core::model::{Ident, Path, RowId};
use std::collections::BTreeMap;

/// The caller-supplied bounds: per-object row counts of the searched
/// shape and the candidate budget. Bounds are inputs — the engine never
/// grows them. Objects omitted from `max_rows` read as zero rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchBounds {
    /// The per-object row counts of the bounded shape.
    pub max_rows: BTreeMap<Ident, usize>,
    /// The maximum number of candidates the engine may enumerate before
    /// the outcome becomes `UNKNOWN` with the explored scope.
    pub max_candidates: usize,
}

/// The exact scope a non-answer explored: the effective row shape
/// (nonzero parts only — an empty map *is* the empty scope) and the
/// candidates actually enumerated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchScope {
    /// The effective nonzero per-object row counts explored.
    pub rows: BTreeMap<Ident, usize>,
    /// How many candidates the enumeration explored before stopping.
    pub candidates_explored: usize,
}

/// The analysis engines' configurable scope caps. Defaults are
/// documented in the crate README's decidability-boundary section; the
/// CLI surfaces them as flags with these defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeCaps {
    /// The search engine's default candidate budget (256).
    pub max_candidates: usize,
    /// The parallel-path walk's maximum path length (3; a cyclic schema
    /// generates unboundedly many paths — the cap keeps the op total).
    pub max_path_length: usize,
    /// The rewriting kernel's maximum derivation steps (64; a cap hit is
    /// not a proof — no redundancy claim is emitted).
    pub max_rewrite_steps: usize,
}

impl Default for ScopeCaps {
    fn default() -> Self {
        Self {
            max_candidates: 256,
            max_path_length: 3,
            max_rewrite_steps: 64,
        }
    }
}

/// The contradiction finding — the P2 taxonomy shape with the canonical
/// accepted-equation indices, in order; `executed`: a proof the kernel
/// computed (completed exhaustion).
pub fn contradiction_finding(scope: Vec<usize>) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Contradiction(ContradictionWitness { scope }),
    }
}

/// The redundancy finding — `derivable` = the declared redundant side,
/// `reducing` = the path it reduces to under the other equations;
/// `executed`: a proof by rewriting.
pub fn redundant_finding(derivable: Path, reducing: Path) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Redundant(RedundantWitness {
            derivable,
            reducing,
        }),
    }
}

/// The domain question every parallel-path hypothesis raises — part of
/// the witness contract, so the text lives with the builder.
pub const PARALLEL_PATHS_QUESTION: &str = "should these agree?";

/// The underconstraint finding — the parallel-path domain question,
/// `hypothesized`, question text last (the P2 flattened shape).
pub fn parallel_paths_finding(lhs: Path, rhs: Path) -> Finding {
    Finding {
        label: EpistemicLabel::Hypothesized,
        kind: FindingKind::Underconstraint(UnderconstraintWitness {
            case: DomainQuestion::ParallelPaths { lhs, rhs },
            question: PARALLEL_PATHS_QUESTION.to_string(),
        }),
    }
}

/// The question every rule-lens gap raises — the input class is named in
/// the witness; the question stays fixed across classes.
pub const UNCOVERED_INPUTS_QUESTION: &str = "which action applies? no rule covers this input class";

/// The underconstraint finding for a rule-lens gap — `UncoveredInputs`
/// with the class name and its example inputs (dense enumeration
/// indices), `hypothesized` until totality is declared (design.md
/// Decision 1). The P8 rule lens is the only producer of this arm.
pub fn uncovered_inputs_finding(class: String, examples: Vec<RowId>) -> Finding {
    Finding {
        label: EpistemicLabel::Hypothesized,
        kind: FindingKind::Underconstraint(UnderconstraintWitness {
            case: DomainQuestion::UncoveredInputs { class, examples },
            question: UNCOVERED_INPUTS_QUESTION.to_string(),
        }),
    }
}
