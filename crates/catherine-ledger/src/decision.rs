//! Purpose: the proof-phase stop-gate decision — its entry schema and its
//! validation rules (openspec change `add-lean-kernel-model`, design
//! Decision 5): what the gate chose and what the proofs bought versus
//! fixture-only assurance, recorded as ledger data.
//!
//! Responsibilities: define [`DecisionRecord`] (the `decision` payload of
//! [`crate::entry::EntryContent`]), the closed outcome vocabulary
//! (`extend | hold | retire`), and [`decision_violations`] — the
//! kind/payload agreement and outcome-vocabulary refusals the ledger
//! validator asks for
//! ([`crate::validate::Violation::InvalidDecisionOutcome`],
//! [`crate::validate::Violation::MalformedDecision`]).
//!
//! Rationale: a decision is a terminal record like a rollup — it carries
//! no kata payload and does not participate in rollup placement or
//! coverage — so it is content-addressed and refused by the same rules as
//! every other entry. The outcome is a closed vocabulary (like the kata
//! verdict), and the content field is skipped when absent, so adding it
//! cannot rewrite the canonical bytes — or the ids — of any pre-existing
//! entry.

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::validate::Violation;

/// The closed stop-gate decision-outcome vocabulary (openspec change
/// `add-lean-kernel-model`, design Decision 5): extend | hold | retire.
const DECISION_OUTCOMES: [&str; 3] = ["extend", "hold", "retire"];

/// The proof-phase stop-gate decision (kind `decision`; openspec change
/// `add-lean-kernel-model`, design Decision 5): what the gate chose and
/// what the proofs bought versus fixture-only assurance. Decisions are
/// terminal records like rollups — they carry no kata payload and do not
/// participate in rollup placement or coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRecord {
    /// The closed outcome vocabulary: `extend` (search/migration
    /// follow-up), `hold` (maintain at landed scope), or `retire`
    /// (archive the Lean tree, keep the lessons).
    pub outcome: String,
    /// What the decision governs (e.g. the model surface at landed
    /// scope) — never the whole project by implication.
    pub scope: String,
    /// Why this outcome, in the gate's own terms.
    pub rationale: String,
    /// The theorem names that landed; empty is a statement about the
    /// proofs, not an omission.
    pub proofs: Vec<String>,
    /// Actual kernel defects the proofs surfaced (none is a finding).
    pub defects_found: Vec<String>,
    /// Theorems the surface could not state — the named holes.
    pub holes_found: Vec<String>,
    /// What the fixture corpus alone pinned versus what the proofs
    /// added — the cost/benefit accounting the spec requires.
    pub fixture_only_comparison: String,
}

/// The decision-entry refusals: the `decision` kind and its payload must
/// agree, and the outcome is the closed `extend | hold | retire`
/// vocabulary. A decision record never carries a kata verdict, so the
/// validator's verdict rule never reaches it.
pub(crate) fn decision_violations(entry: &Entry) -> Vec<Violation> {
    let seq = entry.content.seq;
    match (entry.content.kind.as_str(), &entry.content.decision) {
        ("decision", Some(record)) => {
            if DECISION_OUTCOMES.contains(&record.outcome.as_str()) {
                return Vec::new();
            }
            vec![Violation::InvalidDecisionOutcome {
                seq,
                outcome: record.outcome.clone(),
            }]
        }
        ("decision", None) => vec![Violation::MalformedDecision {
            seq,
            reason: "kind `decision` without a decision record".into(),
        }],
        (_, Some(_)) => vec![Violation::MalformedDecision {
            seq,
            reason: format!(
                "decision record on non-decision kind `{}`",
                entry.content.kind
            ),
        }],
        (_, None) => Vec::new(),
    }
}
