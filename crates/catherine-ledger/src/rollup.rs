//! Purpose: the project-verdict rollup (tasks.md task 8) — the project
//! verdict derives ONLY via the recorded procedure from the kata
//! entries: no goalpost moves, no per-kata overrides.
//!
//! Responsibilities: [`derive`] walks the validated kata entries through
//! the recorded steps — control gate first, then the positive katas —
//! and produces the verdict plus the exact procedure text the rollup
//! entry records. [`PROCEDURE`] is the single source of the recorded
//! steps; the rollup entry embeds it verbatim.
//!
//! Rationale: design Decision 3 — the vocabulary is closed, a killed
//! project verdict stops the project, and P7/P8 are beads-gated on
//! `survived`. The derivation is mechanical so the orchestrator can
//! re-run it from the entries alone.

use crate::entry::{Entry, Verdict};

/// The recorded derivation procedure — embedded verbatim in the rollup
/// entry and implemented step-by-step by [`derive`].
pub const PROCEDURE: [&str; 7] = [
    "1. the ledger validates (validator green: ids sealed, seq contiguous, negative control first)",
    "2. control gate: the first kata entry is the FizzBuzz negative control with verdict killed and empty unique_detections — any other outcome makes the project invalid-run (the control failed to self-negate)",
    "3. any entry with verdict invalid-run makes the project invalid-run",
    "4. every positive kata survived (scoped) makes the project survived (scoped)",
    "5. any positive kata killed makes the project killed — the stop condition",
    "6. otherwise (a positive kata inconclusive, none killed) the project is inconclusive",
    "7. gates: P7 catherine-alr / P8 catherine-nq5 unblock only on project survived; the bead flip is orchestrator-reserved",
];

/// The derived rollup: verdict, control-gate outcome, per-kata lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rollup {
    pub verdict: Verdict,
    /// `passed` or the exact control-gate failure.
    pub control_gate: String,
    /// (kata_id, entry_id, verdict) per kata entry, in seq order.
    pub per_kata: Vec<(String, String, Verdict)>,
}

/// The kata entry's verdict, parsed against the closed vocabulary.
fn verdict_of(entry: &Entry) -> Verdict {
    entry
        .content
        .kata
        .as_ref()
        .and_then(|kata| kata.verdict.parse().ok())
        .expect("the validator closed the vocabulary")
}

/// The positive katas' verdicts (the negative control excluded).
fn positives(katas: &[&Entry]) -> Vec<Verdict> {
    katas
        .iter()
        .filter(|entry| !entry.content.kata.as_ref().is_some_and(|kata| kata.control))
        .map(|entry| verdict_of(entry))
        .collect()
}

/// The control gate: `passed` iff the FizzBuzz control self-negated
/// (killed, zero unique detections) — procedure step 2.
fn control_gate(control_kata: &crate::entry::KataRecord) -> String {
    let self_negated = control_kata.verdict.as_str() == "killed"
        && control_kata.comparison.unique_detections.is_empty();
    if self_negated {
        "passed".to_string()
    } else {
        "failed: the control did not self-negate (verdict or unique detections)".to_string()
    }
}

/// The (kata_id, entry_id, verdict) lines, in seq order.
fn per_kata(katas: &[&Entry]) -> Vec<(String, String, Verdict)> {
    katas
        .iter()
        .map(|entry| {
            let kata = entry.content.kata.as_ref().expect("kata record");
            (kata.kata_id.clone(), entry.id.clone(), verdict_of(entry))
        })
        .collect()
}

/// Derive the project verdict from validated kata entries, step by step
/// through [`PROCEDURE`]: control gate first (step 2), invalid-run
/// propagation (step 3), all-positives-survived (step 4), any-positive-
/// killed stop condition (step 5), otherwise inconclusive (step 6).
pub fn derive(entries: &[&Entry]) -> Result<Rollup, String> {
    let katas: Vec<&Entry> = entries
        .iter()
        .filter(|entry| entry.content.kind == "kata")
        .copied()
        .collect();
    let Some(control) = katas
        .iter()
        .find(|entry| entry.content.kata.as_ref().is_some_and(|kata| kata.control))
    else {
        return Err("no negative-control kata entry — the ledger cannot derive".into());
    };
    let gate = control_gate(control.content.kata.as_ref().expect("kata record"));
    let positive_verdicts = positives(&katas);

    let control_failed = gate != "passed";
    let verdict = if control_failed || positive_verdicts.contains(&Verdict::InvalidRun) {
        Verdict::InvalidRun
    } else if positive_verdicts.contains(&Verdict::Killed) {
        Verdict::Killed
    } else if positive_verdicts.contains(&Verdict::Inconclusive) {
        Verdict::Inconclusive
    } else {
        // The closed vocabulary: no killed, no inconclusive, no
        // invalid-run — every positive survived.
        Verdict::Survived
    };

    Ok(Rollup {
        verdict,
        control_gate: gate,
        per_kata: per_kata(&katas),
    })
}
