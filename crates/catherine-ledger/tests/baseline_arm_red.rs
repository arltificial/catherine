//! Purpose: the RED gate for tasks.md task 1 (`add-domain-probe-validation`)
//! — the ledger refuses uniqueness claims without an executed, committed
//! baseline arm. `unique_detections` is derived arithmetic (probe detections
//! minus baseline detections), never an assertion; a cited-only baseline
//! cannot support a `survived` verdict. Assertions are structural (named
//! violations), never snapshots.

use std::fs;
use std::path::PathBuf;

use catherine_ledger::entry::{
    seal, BaselineArm, BaselineMode, Comparison, EntryContent, KataRecord, KataVerdict, RunLog,
};
use catherine_ledger::validate::{validate_dir, Violation};

/// A throwaway ledger directory per test (unique suffix, cleaned up).
struct TempLedger(PathBuf);

impl TempLedger {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "catherine-ledger-baseline-red-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp ledger dir created");
        Self(dir)
    }

    /// Seal contents and write them as the ledger's files, in order.
    fn write(&self, contents: &[EntryContent]) {
        for content in contents {
            let entry = seal(content.clone()).expect("entry seals");
            let hex = entry.id.trim_start_matches("sha256:");
            let path = self
                .0
                .join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
            fs::write(path, serde_json::to_string(&entry).expect("entry json"))
                .expect("ledger file written");
        }
    }
}

impl Drop for TempLedger {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A minimal kata record — the fields the validator reads.
fn kata(seq: u32, kata_id: &str, control: bool, verdict: &str) -> EntryContent {
    EntryContent {
        seq,
        kind: "kata".into(),
        recorded: "2026-11-13".into(),
        kata: Some(KataRecord {
            kata_id: kata_id.into(),
            name: if control {
                "FizzBuzz (negative control)".into()
            } else {
                kata_id.into()
            },
            control,
            series_report: "runs/2026-11-02-kata06-local-attempt001/REPORT.md".into(),
            series_verdict: "killed".into(),
            fixture: "fixtures/katas/fizzbuzz.ct.json".into(),
            fixture_hash: "sha256:fixture".into(),
            expected: vec![],
            run: RunLog {
                commands: vec![],
                findings: vec![],
                probes: vec![],
            },
            comparison: Comparison {
                conventional_baseline: "3/3 behavioral".into(),
                fired: vec![],
                unique_detections: vec![],
                baseline: None,
            },
            expressibility_gap: vec![],
            discrepancies: vec![],
            procedure: vec![],
            catalog_hash: None,
            verdict: verdict.into(),
        }),
        rollup: None,
        model_hashes: vec!["sha256:fixture".into()],
        supersedes: None,
    }
}

/// Builder: start from [`kata`] and overwrite the comparison fields the
/// uniqueness-claim tests vary.
fn kata_claiming(
    seq: u32,
    kata_id: &str,
    control: bool,
    verdict: &str,
    unique_detections: &[&str],
    baseline: Option<BaselineArm>,
) -> EntryContent {
    let mut content = kata(seq, kata_id, control, verdict);
    let comparison = &mut content
        .kata
        .as_mut()
        .expect("builder input is a kata record")
        .comparison;
    comparison.unique_detections = unique_detections.iter().map(|s| s.to_string()).collect();
    comparison.baseline = baseline;
    content
}

/// The minimal rollup content over the given per-kata verdicts.
#[allow(dead_code)] // kept shape-identical with validator_red.rs; these tests need no rollup
fn rollup(seq: u32, per_kata: &[(&str, &str, &str)]) -> EntryContent {
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
    }
}

#[test]
fn rejects_unique_detections_without_baseline_arm() {
    let ledger = TempLedger::new("unique-no-baseline");
    ledger.write(&[
        kata(1, "kata-6", true, "killed"),
        kata_claiming(
            2,
            "kata-1",
            false,
            "survived",
            &["violation:eq0@row3"],
            None,
        ),
    ]);
    let violations =
        validate_dir(&ledger.0).expect_err("a uniqueness claim without a baseline is refused");
    assert!(
        violations.contains(&Violation::UniqueWithoutBaseline { seq: 2 }),
        "expected UniqueWithoutBaseline(seq 2), got {violations:?}"
    );
}

#[test]
fn rejects_survived_with_cited_only_baseline() {
    let ledger = TempLedger::new("cited-baseline");
    ledger.write(&[
        kata(1, "kata-6", true, "killed"),
        kata_claiming(
            2,
            "kata-1",
            false,
            "survived",
            &["violation:eq0@row3"],
            Some(BaselineArm {
                artifact: "arXiv:1603.07466 (cited only)".into(),
                mode: BaselineMode::Cited,
                detections: vec![],
            }),
        ),
    ]);
    let violations = validate_dir(&ledger.0).expect_err("a cited-only baseline cannot survive");
    assert!(
        violations.contains(&Violation::CitedBaselineCannotSurvive { seq: 2 }),
        "expected CitedBaselineCannotSurvive(seq 2), got {violations:?}"
    );
}

#[test]
fn accepts_unique_claim_with_executed_baseline() {
    let ledger = TempLedger::new("executed-baseline");
    ledger.write(&[
        kata(1, "kata-6", true, "killed"),
        kata_claiming(
            2,
            "kata-1",
            false,
            "survived",
            &["violation:eq0@row3"],
            Some(BaselineArm {
                artifact: "prototype/migration_baseline.py (sha256:...)".into(),
                mode: BaselineMode::Executed,
                detections: vec!["row-count dry run: 0/6".into()],
            }),
        ),
    ]);
    let validated = validate_dir(&ledger.0);
    assert!(
        validated.is_ok(),
        "an executed, committed baseline supports the uniqueness claim, got {:?}",
        validated.err()
    );
}

#[test]
fn accepts_parity_verdict_without_baseline() {
    let ledger = TempLedger::new("parity-no-baseline");
    ledger.write(&[
        kata(1, "kata-6", true, "killed"),
        kata_claiming(2, "kata-1", false, "survived", &[], None),
    ]);
    let validated = validate_dir(&ledger.0);
    assert!(
        validated.is_ok(),
        "an absent baseline arm caps the claim at parity, got {:?}",
        validated.err()
    );
}
