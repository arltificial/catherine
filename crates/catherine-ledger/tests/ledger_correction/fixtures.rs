//! The correction-path tests' fixtures: the sealed-file writer, the
//! minimal kata builder (`KataSpec` + [`kata`]), the rollup builder, and
//! the superseding-entry helper. Split out of the test binary verbatim
//! (mechanical size refactor); contents unchanged.

use std::fs;
use std::path::Path;

use catherine_ledger::entry::{
    BaselineArm, Comparison, Derivation, Entry, EntryContent, KataIdentity, KataRecord,
    KataVerdict, Provenance, RunLog,
};

use super::support::TestError;
/// Write already-sealed entries as the ledger's files, in order.
/// Sealing first is what lets a later entry name an earlier id in
/// `supersedes`.
pub(super) fn write_sealed(dir: &Path, entries: &[Entry]) -> Result<(), TestError> {
    for entry in entries {
        let hex = entry.id.trim_start_matches("sha256:");
        let path = dir.join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
        fs::write(path, serde_json::to_string(entry)?)?;
    }
    Ok(())
}

/// The variation axes of the minimal kata fixture: the seq, which kata
/// it claims, whether it is the negative control, its verdict, and the
/// uniqueness claim (the unique detections and the baseline arm that
/// backs them). Fields default to the empty shape, so a call site names
/// only what varies.
#[derive(Default)]
pub(super) struct KataSpec {
    pub(super) seq: u32,
    pub(super) kata_id: &'static str,
    pub(super) control: bool,
    pub(super) verdict: &'static str,
    pub(super) unique: &'static [&'static str],
    pub(super) baseline: Option<BaselineArm>,
}

/// A minimal kata record — the fields the validator reads, with the
/// uniqueness claim layered on when the spec carries one.
pub(super) fn kata(spec: KataSpec) -> EntryContent {
    let KataSpec {
        seq,
        kata_id,
        control,
        verdict,
        unique,
        baseline,
    } = spec;
    EntryContent {
        seq,
        kind: "kata".into(),
        recorded: "2026-11-13".into(),
        kata: Some(KataRecord {
            identity: KataIdentity {
                kata_id: kata_id.into(),
                name: if control {
                    "FizzBuzz (negative control)".into()
                } else {
                    kata_id.into()
                },
                control,
            },
            provenance: Provenance {
                series_report: "runs/2026-11-02-kata06-local-attempt001/REPORT.md".into(),
                series_verdict: "killed".into(),
                fixture: "fixtures/katas/fizzbuzz.ct.json".into(),
                fixture_hash: "sha256:fixture".into(),
            },
            expected: vec![],
            run: RunLog {
                commands: vec![],
                findings: vec![],
                probes: vec![],
            },
            derivation: Derivation {
                comparison: Comparison {
                    conventional_baseline: "3/3 behavioral".into(),
                    fired: vec![],
                    unique_detections: unique.iter().map(|s| s.to_string()).collect(),
                    baseline,
                },
                procedure: vec![],
                verdict: verdict.into(),
            },
            expressibility_gap: vec![],
            discrepancies: vec![],
            catalog_hash: None,
        }),
        rollup: None,
        model_hashes: vec!["sha256:fixture".into()],
        supersedes: None,
        decision: None,
    }
}

/// The minimal rollup content over the given per-kata verdicts.
pub(super) fn rollup(seq: u32, per_kata: &[(&str, &str, &str)]) -> EntryContent {
    EntryContent {
        seq,
        kind: "rollup".into(),
        recorded: "2026-11-13".into(),
        kata: None,
        rollup: Some(catherine_ledger::entry::RollupRecord {
            procedure: catherine_ledger::rollup::PROCEDURE
                .iter()
                .map(|s| s.to_string())
                .collect(),
            kata_entry_ids: per_kata.iter().map(|(_, id, _)| id.to_string()).collect(),
            per_kata: per_kata
                .iter()
                .map(|(kata, id, verdict)| KataVerdict {
                    kata_id: kata.to_string(),
                    entry_id: id.to_string(),
                    verdict: verdict.to_string(),
                })
                .collect(),
            control_gate: "passed".into(),
            gates: catherine_ledger::entry::Gates {
                p7_catherine_alr: "blocked".into(),
                p8_catherine_nq5: "blocked".into(),
                note: "the bead flip is orchestrator-reserved".into(),
            },
            verdict: "survived".into(),
        }),
        model_hashes: vec!["sha256:rollup".into()],
        supersedes: None,
        decision: None,
    }
}

/// `supersedes` names the sealed target, so the correction is a new
/// entry at a later seq — the append-only correction path.
pub(super) fn superseding(mut correction: EntryContent, target: &Entry) -> EntryContent {
    correction.supersedes = Some(target.id.clone());
    correction
}
