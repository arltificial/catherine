//! Purpose: the RED gate (tasks.md task 1) — the ledger validator
//! rejects bad science: a kata entry without a prior negative control,
//! a verdict outside the closed vocabulary, an entry without model-hash
//! references. Plus the append-only integrity pins (id recomputation,
//! seq order, supersession) and the recorded rollup procedure's
//! outcomes. Assertions are structural (named rejections, derived
//! verdicts) — never snapshots.

use std::fs;
use std::path::PathBuf;

use catherine_ledger::entry::{
    entry_id, seal, Comparison, EntryContent, Fired, KataRecord, KataVerdict, RunLog, Verdict,
};
use catherine_ledger::rollup::{derive, Rollup};
use catherine_ledger::validate::{validate_dir, ValidLedger, Violation};

/// A throwaway ledger directory per test (unique suffix, cleaned up).
struct TempLedger(PathBuf);

impl TempLedger {
    fn new(tag: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("catherine-ledger-red-{tag}-{}", std::process::id()));
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
        recorded: "2026-11-12".into(),
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

/// The minimal rollup content over the given per-kata verdicts.
fn rollup(seq: u32, per_kata: &[(&str, &str, &str)]) -> EntryContent {
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
    }
}

/// GREEN shape: control first, then a positive kata, then the rollup —
/// the ledger the validator accepts. The rollup's `kata_entry_ids` are
/// the real sealed ids (coverage is set equality against the effective
/// kata entries, not a placeholder line count).
fn green_contents() -> Vec<EntryContent> {
    let control = kata(1, "kata-6", true, "killed");
    let positive = kata(2, "kata-1", false, "survived");
    let control_id = seal(control.clone()).expect("control seals").id;
    let positive_id = seal(positive.clone()).expect("positive seals").id;
    vec![
        control,
        positive,
        rollup(
            3,
            &[
                ("kata-6", control_id.as_str(), "killed"),
                ("kata-1", positive_id.as_str(), "survived"),
            ],
        ),
    ]
}

#[test]
fn rejects_kata_entry_without_prior_negative_control() {
    let ledger = TempLedger::new("no-control");
    ledger.write(&[kata(1, "kata-1", false, "survived")]);
    let violations = validate_dir(&ledger.0).expect_err("a control-less kata entry is refused");
    assert!(
        violations.contains(&Violation::NegativeControlMissing { seq: 1 }),
        "expected NegativeControlMissing, got {violations:?}"
    );
}

#[test]
fn rejects_verdict_outside_closed_vocabulary() {
    let ledger = TempLedger::new("bad-verdict");
    ledger.write(&[
        kata(1, "kata-6", true, "killed"),
        kata(2, "kata-1", false, "partial"),
    ]);
    let violations = validate_dir(&ledger.0).expect_err("an off-vocabulary verdict is refused");
    assert!(
        violations.contains(&Violation::InvalidVerdict {
            seq: 2,
            verdict: "partial".into()
        }),
        "expected InvalidVerdict(partial), got {violations:?}"
    );
}

#[test]
fn rejects_entry_without_model_hash_references() {
    let ledger = TempLedger::new("no-hashes");
    let mut control = kata(1, "kata-6", true, "killed");
    control.model_hashes = vec![];
    ledger.write(&[control]);
    let violations = validate_dir(&ledger.0).expect_err("an entry without model hashes is refused");
    assert!(
        violations.contains(&Violation::MissingModelHashes { seq: 1 }),
        "expected MissingModelHashes, got {violations:?}"
    );
}

#[test]
fn rejects_entry_whose_id_does_not_match_content() {
    let ledger = TempLedger::new("bad-id");
    let mut contents = green_contents();
    let entry = seal(contents.remove(0)).expect("seals");
    let tampered = format!(r#"{{"id":"sha256:{}","seq":1}}"#, "0".repeat(64));
    let hex = entry.id.trim_start_matches("sha256:");
    fs::write(ledger.0.join(format!("0001-{}.json", &hex[..8])), tampered).expect("tampered file");
    ledger.write(&contents);
    let violations = validate_dir(&ledger.0).expect_err("a sealed-id mismatch is refused");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::IdMismatch { .. })),
        "expected IdMismatch, got {violations:?}"
    );
}

#[test]
fn rejects_seq_out_of_order() {
    let ledger = TempLedger::new("bad-seq");
    ledger.write(&[kata(2, "kata-6", true, "killed")]);
    let violations =
        validate_dir(&ledger.0).expect_err("a ledger not starting at seq 1 is refused");
    assert!(
        violations.contains(&Violation::SeqOrder {
            found: 2,
            expected: 1
        }),
        "expected SeqOrder, got {violations:?}"
    );
}

#[test]
fn rejects_control_that_did_not_self_negate() {
    let ledger = TempLedger::new("control-survived");
    ledger.write(&[kata(1, "kata-6", true, "survived")]);
    let violations = validate_dir(&ledger.0).expect_err("a surviving control is refused");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::ControlDidNotSelfNegate { .. })),
        "expected ControlDidNotSelfNegate, got {violations:?}"
    );
}

#[test]
fn rejects_supersession_pointing_at_missing_entry() {
    let ledger = TempLedger::new("bad-supersession");
    let mut control = kata(1, "kata-6", true, "killed");
    control.supersedes = Some("sha256:does-not-exist".into());
    ledger.write(&[control]);
    let violations = validate_dir(&ledger.0).expect_err("a dangling supersession is refused");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::BadSupersession { .. })),
        "expected BadSupersession, got {violations:?}"
    );
}

#[test]
fn accepts_control_first_then_katas_then_rollup() {
    let ledger = TempLedger::new("green");
    ledger.write(&green_contents());
    let valid: ValidLedger = validate_dir(&ledger.0).expect("the green ledger passes");
    assert_eq!(valid.kata_entries().len(), 2);
    assert!(valid.rollup().is_some());
}

#[test]
fn superseding_entry_is_accepted_when_target_exists() {
    let ledger = TempLedger::new("supersession");
    let first = seal(kata(1, "kata-6", true, "inconclusive")).expect("seals");
    let mut correction = kata(2, "kata-6", true, "killed");
    correction.supersedes = Some(first.id.clone());
    ledger.write(&[first.content, correction]);
    let validated = validate_dir(&ledger.0);
    assert!(
        validated.is_ok(),
        "a well-formed supersession passes, got {:?}",
        validated.err()
    );
}

#[test]
fn rollup_survived_when_control_killed_and_positives_survived() {
    let entries = [
        kata(1, "kata-6", true, "killed"),
        kata(2, "kata-1", false, "survived"),
    ];
    let sealed: Vec<_> = entries.iter().map(|c| seal(c.clone()).unwrap()).collect();
    let refs: Vec<&_> = sealed.iter().collect();
    let roll: Rollup = derive(&refs).expect("derives");
    assert_eq!(roll.verdict, Verdict::Survived);
    assert_eq!(roll.control_gate, "passed");
}

#[test]
fn rollup_killed_when_a_positive_kata_is_killed() {
    let entries = [
        kata(1, "kata-6", true, "killed"),
        kata(2, "kata-1", false, "killed"),
    ];
    let sealed: Vec<_> = entries.iter().map(|c| seal(c.clone()).unwrap()).collect();
    let refs: Vec<&_> = sealed.iter().collect();
    assert_eq!(derive(&refs).expect("derives").verdict, Verdict::Killed);
}

#[test]
fn rollup_invalid_run_when_the_control_fails_to_self_negate() {
    let entries = [
        kata(1, "kata-6", true, "survived"),
        kata(2, "kata-1", false, "survived"),
    ];
    let sealed: Vec<_> = entries.iter().map(|c| seal(c.clone()).unwrap()).collect();
    let refs: Vec<&_> = sealed.iter().collect();
    assert_eq!(derive(&refs).expect("derives").verdict, Verdict::InvalidRun);
}

#[test]
fn rollup_inconclusive_when_a_positive_kata_is_inconclusive() {
    let entries = [
        kata(1, "kata-6", true, "killed"),
        kata(2, "kata-1", false, "inconclusive"),
    ];
    let sealed: Vec<_> = entries.iter().map(|c| seal(c.clone()).unwrap()).collect();
    let refs: Vec<&_> = sealed.iter().collect();
    assert_eq!(
        derive(&refs).expect("derives").verdict,
        Verdict::Inconclusive
    );
}

#[test]
fn entry_id_matches_documented_content_digest() {
    let content = kata(9, "kata-6", true, "killed");
    let expected = entry_id(&content).expect("id computes");
    assert!(expected.starts_with("sha256:"));
    assert_eq!(expected.len(), "sha256:".len() + 64);
}

/// Unused-import guard for the harness-facing citation helper the
/// fixtures exercise through the ledger bins.
#[allow(dead_code)]
fn _fired_shape(_: Fired) {}
