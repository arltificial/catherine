//! The validator RED tests' fixtures: the minimal kata entry builder
//! (`KataSpec` + [`kata`]), the GREEN ledger contents, and the refusal
//! case table the shape-rejection test loops over. Split out of the
//! test binary verbatim (mechanical size refactor); contents unchanged.

use catherine_ledger::entry::{
    Comparison, Derivation, EntryContent, KataIdentity, KataRecord, KataVerdict, Provenance, RunLog,
};

use catherine_ledger::entry::seal;
use catherine_ledger::validate::Violation;

use super::support::TestError;

/// The variation axes of the minimal kata fixture: the seq, which kata
/// it claims, whether it is the negative control, and its verdict.
/// Fields default to the empty shape, so a call site names only what
/// varies.
#[derive(Default)]
pub(super) struct KataSpec {
    pub(super) seq: u32,
    pub(super) kata_id: &'static str,
    pub(super) control: bool,
    pub(super) verdict: &'static str,
}

/// A minimal kata record — the fields the validator reads.
pub(super) fn kata(spec: KataSpec) -> EntryContent {
    let KataSpec {
        seq,
        kata_id,
        control,
        verdict,
    } = spec;
    EntryContent {
        seq,
        kind: "kata".into(),
        recorded: "2026-11-12".into(),
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
                    unique_detections: vec![],
                    baseline: None,
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
        recorded: "2026-11-12".into(),
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

/// GREEN shape: control first, then a positive kata, then the rollup —
/// the ledger the validator accepts. The rollup's `kata_entry_ids` are
/// the real sealed ids (coverage is set equality against the effective
/// kata entries, not a placeholder line count).
pub(super) fn green_contents() -> Result<Vec<EntryContent>, TestError> {
    let control = kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
    });
    let positive = kata(KataSpec {
        seq: 2,
        kata_id: "kata-1",
        control: false,
        verdict: "survived",
    });
    let control_id = seal(control.clone())?.id;
    let positive_id = seal(positive.clone())?.id;
    Ok(vec![
        control,
        positive,
        rollup(
            3,
            &[
                ("kata-6", control_id.as_str(), "killed"),
                ("kata-1", positive_id.as_str(), "survived"),
            ],
        ),
    ])
}

/// One content-level refusal case: a throwaway ledger's contents and
/// the named violation the validator must report for them.
pub(super) struct RefusalCase {
    pub(super) tag: &'static str,
    pub(super) what: &'static str,
    pub(super) contents: Vec<EntryContent>,
    pub(super) violation: Violation,
}

/// The content-level refusals: each bad entry shape names its violation
/// at the offending seq — a control-less kata, an off-vocabulary
/// verdict, a missing model-hash citation, and a ledger not starting at
/// seq 1.
pub(super) fn refusal_cases() -> Vec<RefusalCase> {
    vec![
        RefusalCase {
            tag: "no-control",
            what: "a control-less kata entry",
            contents: vec![kata(KataSpec {
                seq: 1,
                kata_id: "kata-1",
                control: false,
                verdict: "killed",
            })],
            violation: Violation::NegativeControlMissing { seq: 1 },
        },
        RefusalCase {
            tag: "bad-verdict",
            what: "an off-vocabulary verdict",
            contents: vec![
                kata(KataSpec {
                    seq: 1,
                    kata_id: "kata-6",
                    control: true,
                    verdict: "killed",
                }),
                kata(KataSpec {
                    seq: 2,
                    kata_id: "kata-1",
                    control: false,
                    verdict: "partial",
                }),
            ],
            violation: Violation::InvalidVerdict {
                seq: 2,
                verdict: "partial".into(),
            },
        },
        RefusalCase {
            tag: "no-hashes",
            what: "an entry without model hashes",
            contents: vec![EntryContent {
                model_hashes: vec![],
                ..kata(KataSpec {
                    seq: 1,
                    kata_id: "kata-6",
                    control: true,
                    verdict: "killed",
                })
            }],
            violation: Violation::MissingModelHashes { seq: 1 },
        },
        RefusalCase {
            tag: "bad-seq",
            what: "a ledger not starting at seq 1",
            contents: vec![kata(KataSpec {
                seq: 2,
                kata_id: "kata-6",
                control: true,
                verdict: "killed",
            })],
            violation: Violation::SeqOrder {
                found: 2,
                expected: 1,
            },
        },
    ]
}
