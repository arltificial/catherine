//! Purpose: the correction-path gate for the ledger (tasks.md task 2,
//! GREEN of `add-domain-probe-validation`) — a bad claim is corrected by
//! appending a superseding entry, and the claim-bearing rules then apply
//! to the effective ledger only. A superseded uniqueness claim without a
//! baseline arm stops being refused; a correction that repeats the bad
//! shape is refused at its own seq; the rollup must cover exactly the
//! effective kata entries and must be the final effective entry.
//!
//! Responsibilities: pin the effective-entry filter, the corrected
//! rollup's placement/coverage, and the baseline-arm refusal over
//! effective entries — including the case where the correction itself is
//! the one without an arm.
//!
//! Rationale: the ledger is append-only (entry.rs), so correcting the
//! shipped rule-lens entry means appending a superseding entry after its
//! rollup, never editing a sealed file. The validator must therefore
//! distinguish "a claim someone corrected" from "the current claim";
//! otherwise the correction path it documents is unusable. Assertions
//! are structural (named violations, effective seqs), never snapshots.

use std::fs;
use std::path::PathBuf;

use catherine_ledger::entry::{
    seal, BaselineArm, BaselineMode, Comparison, Entry, EntryContent, KataRecord, KataVerdict,
    RunLog,
};
use catherine_ledger::validate::{validate_dir, Violation};

/// A throwaway ledger directory per test (unique suffix, cleaned up).
struct TempLedger(PathBuf);

impl TempLedger {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "catherine-ledger-correction-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp ledger dir created");
        Self(dir)
    }

    /// Write already-sealed entries as the ledger's files, in order.
    /// Sealing first is what lets a later entry name an earlier id in
    /// `supersedes`.
    fn write(&self, entries: &[Entry]) {
        for entry in entries {
            let hex = entry.id.trim_start_matches("sha256:");
            let path = self
                .0
                .join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
            fs::write(path, serde_json::to_string(entry).expect("entry json"))
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
            verdict: verdict.into(),
        }),
        rollup: None,
        model_hashes: vec!["sha256:fixture".into()],
        supersedes: None,
    }
}

/// Builder: start from [`kata`] and set the uniqueness-claim fields the
/// correction cases vary.
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

/// The executed, committed arm that supports a uniqueness claim.
fn executed_arm() -> BaselineArm {
    BaselineArm {
        artifact: "prototype/migration_baseline.py (sha256:...)".into(),
        mode: BaselineMode::Executed,
        detections: vec!["row-count dry run: 0/6".into()],
    }
}

/// The minimal rollup content over the given per-kata verdicts.
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

/// `supersedes` names the sealed target, so the correction is a new
/// entry at a later seq — the append-only correction path.
fn superseding(mut correction: EntryContent, target: &Entry) -> EntryContent {
    correction.supersedes = Some(target.id.clone());
    correction
}

#[test]
fn superseded_entry_stops_being_effective() {
    let ledger = TempLedger::new("superseded");
    let control = seal(kata(1, "kata-6", true, "killed")).expect("control seals");
    let bad = seal(kata_claiming(
        2,
        "kata-1",
        false,
        "survived",
        &["violation:eq0@row3"],
        None,
    ))
    .expect("the bad claim seals");
    let fix = seal(superseding(
        kata_claiming(
            3,
            "kata-1",
            false,
            "survived",
            &["violation:eq0@row3"],
            Some(executed_arm()),
        ),
        &bad,
    ))
    .expect("the correction seals");
    ledger.write(&[control, bad, fix]);

    let valid = validate_dir(&ledger.0).expect("a superseding correction is a valid ledger");
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "the superseded claim stops being effective");
    let seqs: Vec<u32> = katas.iter().map(|entry| entry.content.seq).collect();
    assert_eq!(seqs, vec![1, 3], "the control and the correction remain");
}

#[test]
fn rollup_after_correction_is_accepted() {
    let ledger = TempLedger::new("corrected-rollup");
    let control = seal(kata(1, "kata-6", true, "killed")).expect("control seals");
    let claim = seal(kata_claiming(
        2,
        "kata-1",
        false,
        "survived",
        &["violation:eq0@row3"],
        Some(executed_arm()),
    ))
    .expect("claim seals");
    let stale_rollup = seal(rollup(
        3,
        &[
            ("kata-6", control.id.as_str(), "killed"),
            ("kata-1", claim.id.as_str(), "survived"),
        ],
    ))
    .expect("rollup seals");
    let fix = seal(superseding(
        kata_claiming(
            4,
            "kata-1",
            false,
            "survived",
            &["violation:eq0@row3"],
            Some(executed_arm()),
        ),
        &claim,
    ))
    .expect("correction seals");
    let corrected_rollup = seal(superseding(
        rollup(
            5,
            &[
                ("kata-6", control.id.as_str(), "killed"),
                ("kata-1", fix.id.as_str(), "survived"),
            ],
        ),
        &stale_rollup,
    ))
    .expect("replacement rollup seals");
    ledger.write(&[control, claim, stale_rollup, fix, corrected_rollup]);

    let valid = validate_dir(&ledger.0).expect("a superseded rollup is replaced, not refused");
    let effective_rollup = valid
        .rollup()
        .expect("the corrected ledger ends with a rollup");
    assert_eq!(effective_rollup.content.seq, 5);
}

#[test]
fn rollup_coverage_counts_effective_entries() {
    let ledger = TempLedger::new("rollup-stale-id");
    let control = seal(kata(1, "kata-6", true, "killed")).expect("control seals");
    let claim = seal(kata_claiming(
        2,
        "kata-1",
        false,
        "survived",
        &["violation:eq0@row3"],
        Some(executed_arm()),
    ))
    .expect("claim seals");
    let stale_rollup = seal(rollup(
        3,
        &[
            ("kata-6", control.id.as_str(), "killed"),
            ("kata-1", claim.id.as_str(), "survived"),
        ],
    ))
    .expect("rollup seals");
    let fix = seal(superseding(
        kata_claiming(
            4,
            "kata-1",
            false,
            "survived",
            &["violation:eq0@row3"],
            Some(executed_arm()),
        ),
        &claim,
    ))
    .expect("correction seals");
    // The replacement rollup still lists the superseded claim's id.
    let corrected_rollup = seal(superseding(
        rollup(
            5,
            &[
                ("kata-6", control.id.as_str(), "killed"),
                ("kata-1", claim.id.as_str(), "survived"),
                ("kata-1", fix.id.as_str(), "survived"),
            ],
        ),
        &stale_rollup,
    ))
    .expect("replacement rollup seals");
    ledger.write(&[control, claim, stale_rollup, fix, corrected_rollup]);

    let violations = validate_dir(&ledger.0).expect_err("coverage counts effective entries");
    assert!(
        violations.contains(&Violation::RollupIncomplete { seq: 5 }),
        "expected RollupIncomplete(seq 5), got {violations:?}"
    );
}

#[test]
fn rollup_not_last_is_refused() {
    let ledger = TempLedger::new("rollup-not-last");
    let control = seal(kata(1, "kata-6", true, "killed")).expect("control seals");
    let claim = seal(kata(2, "kata-1", false, "survived")).expect("claim seals");
    let sealed_rollup = seal(rollup(
        3,
        &[
            ("kata-6", control.id.as_str(), "killed"),
            ("kata-1", claim.id.as_str(), "survived"),
        ],
    ))
    .expect("rollup seals");
    let late_kata = seal(kata(4, "kata-2", false, "survived")).expect("late kata seals");
    ledger.write(&[control, claim, sealed_rollup, late_kata]);

    let violations = validate_dir(&ledger.0).expect_err("an effective rollup must be last");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::RollupMisplaced { seq: 3, .. })),
        "expected RollupMisplaced(seq 3), got {violations:?}"
    );
}

#[test]
fn baseline_refusal_fires_only_on_effective_entries() {
    let ledger = TempLedger::new("effective-baseline");
    let control = seal(kata(1, "kata-6", true, "killed")).expect("control seals");
    let bad = seal(kata_claiming(
        2,
        "kata-1",
        false,
        "survived",
        &["violation:eq0@row3"],
        None,
    ))
    .expect("the bad claim seals");
    // The correction repeats the claim but still has no arm: the refusal
    // now fires at the correction's seq, not the superseded entry's.
    let ineffective_fix = seal(superseding(
        kata_claiming(
            3,
            "kata-1",
            false,
            "survived",
            &["violation:eq0@row3"],
            None,
        ),
        &bad,
    ))
    .expect("the ineffective correction seals");
    ledger.write(&[control, bad, ineffective_fix]);

    let violations = validate_dir(&ledger.0).expect_err("the effective claim still lacks an arm");
    assert!(
        violations.contains(&Violation::UniqueWithoutBaseline { seq: 3 }),
        "expected UniqueWithoutBaseline(seq 3), got {violations:?}"
    );
    assert!(
        !violations
            .iter()
            .any(|v| matches!(v, Violation::UniqueWithoutBaseline { seq: 2 })),
        "the superseded claim must not be re-judged, got {violations:?}"
    );
}
