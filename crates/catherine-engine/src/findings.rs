//! The findings epistemic label — the one piece of the findings
//! taxonomy the engine itself needs (check families label their
//! findings by constraint status). The full taxonomy (finding shapes,
//! witnesses, JSON) lives in `catherine-core::findings`, which
//! re-exports this type so downstream paths are unchanged.

/// How a finding was established — its epistemic weight. Exactly three
/// variants are constructible: `executed` (kernel-computed results),
/// `hypothesized` (domain questions: proposed-equation failures,
/// underconstraint), `unknown` (scope-bounded non-answers). `proved`
/// and `illustrated` are deliberately absent: they belong to later
/// layers, if ever, and the kernel never fabricates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EpistemicLabel {
    /// Established by kernel execution (accepted-equation failure,
    /// dangling cell): the kernel computed the fact itself.
    Executed,
    /// Raised as a question (proposed-equation failure,
    /// underconstraint): the fact is a question under review, not a
    /// settled failure.
    Hypothesized,
    /// A scope-bounded non-answer: analysis that reports what it could
    /// not decide within its bounds. Reserved for the analysis layer
    /// (P4); no producer exists in this crate.
    Unknown,
}

/// The epistemic weight of a constraint failure: accepted constraints
/// are executed, proposed ones are questions (the one label rule
/// shared by equations, keys, coproducts, and pullbacks).
pub fn label_for(status: crate::model::ConstraintStatus) -> EpistemicLabel {
    match status {
        crate::model::ConstraintStatus::Accepted => EpistemicLabel::Executed,
        crate::model::ConstraintStatus::Proposed => EpistemicLabel::Hypothesized,
    }
}
