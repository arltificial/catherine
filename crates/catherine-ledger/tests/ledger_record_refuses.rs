//! Purpose: the `ledger-record` write-time gate (`add-domain-probe-validation`,
//! tasks.md task 2) — the binary seals an entry into the ledger directory
//! and re-validates the whole directory, so a record that would make the
//! ledger invalid is reverted and the named rejection printed. These tests
//! pin the baseline-arm shapes at the binary boundary: a uniqueness claim
//! without a baseline arm exits 1 and leaves the directory unchanged, and
//! the same claim backed by an executed baseline arm is recorded.
//!
//! Responsibilities: drive the real binary and assert observable behavior
//! only — exit codes, the named violation in stderr, and the ledger
//! directory's `.json` contents afterwards. Never snapshot incidental
//! output.
//!
//! Rationale: `ledger-record` refuses at write time the same shapes the
//! validator refuses, so bad science cannot enter a ledger and then be
//! noticed.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use catherine_ledger::entry::{
    seal, BaselineArm, BaselineMode, Comparison, EntryContent, KataRecord, RunLog,
};

/// A throwaway ledger directory plus the out-of-tree entry file the
/// binary consumes (unique suffix, cleaned up).
struct TempLedger {
    dir: PathBuf,
    candidate: PathBuf,
}

impl TempLedger {
    fn new(tag: &str) -> Self {
        let base = format!("catherine-ledger-record-{tag}-{}", std::process::id());
        let dir = std::env::temp_dir().join(&base);
        let candidate = std::env::temp_dir().join(format!("{base}-candidate.json"));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(&candidate);
        fs::create_dir_all(&dir).expect("temp ledger dir created");
        Self { dir, candidate }
    }

    /// Seal contents and write them as the ledger's files, in order.
    fn write(&self, contents: &[EntryContent]) {
        for content in contents {
            let entry = seal(content.clone()).expect("entry seals");
            let hex = entry.id.trim_start_matches("sha256:");
            let path = self
                .dir
                .join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
            fs::write(path, serde_json::to_string(&entry).expect("entry json"))
                .expect("ledger file written");
        }
    }

    /// Seal one candidate and write it OUTSIDE the ledger directory — the
    /// binary stages it into the directory itself.
    fn stage_candidate(&self, content: EntryContent) {
        let entry = seal(content).expect("candidate seals");
        fs::write(
            &self.candidate,
            serde_json::to_string(&entry).expect("candidate json"),
        )
        .expect("candidate entry written");
    }

    /// The ledger directory's `.json` files, sorted.
    fn json_files(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = fs::read_dir(&self.dir)
            .expect("ledger dir readable")
            .map(|entry| entry.expect("dir entry readable").path())
            .filter(|path| {
                path.extension().and_then(|extension| extension.to_str()) == Some("json")
            })
            .collect();
        files.sort();
        files
    }
}

impl Drop for TempLedger {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
        let _ = fs::remove_file(&self.candidate);
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

/// Builder: start from [`kata`] and overwrite the comparison fields the
/// uniqueness-claim cases vary.
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

/// Run the binary over the ledger directory and the staged candidate.
fn run_record(ledger: &TempLedger) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ledger-record"))
        .arg(&ledger.dir)
        .arg(&ledger.candidate)
        .output()
        .expect("ledger-record runs")
}

#[test]
fn ledger_record_refuses_baseline_less_uniqueness_and_reverts() {
    let ledger = TempLedger::new("refuses-uniqueness");
    ledger.write(&[kata(1, "kata-6", true, "killed")]);
    ledger.stage_candidate(kata_claiming(
        2,
        "kata-1",
        false,
        "survived",
        &["violation:eq0@row3"],
        None,
    ));

    let output = run_record(&ledger);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "a rejected record exits 1, stderr: {stderr}"
    );
    assert!(
        stderr.contains("UniqueWithoutBaseline"),
        "expected the named refusal in stderr, got {stderr}"
    );
    assert_eq!(
        ledger.json_files().len(),
        1,
        "the staged record is reverted; only the control remains"
    );
}

#[test]
fn ledger_record_accepts_an_executed_baseline_arm() {
    let ledger = TempLedger::new("accepts-executed-baseline");
    ledger.write(&[kata(1, "kata-6", true, "killed")]);
    ledger.stage_candidate(kata_claiming(
        2,
        "kata-1",
        false,
        "survived",
        &["violation:eq0@row3"],
        Some(BaselineArm {
            artifact: "prototype/rule_lens_baseline.py sha256:deadbeef".into(),
            mode: BaselineMode::Executed,
            detections: vec!["01-gap-remove-fallback".into()],
        }),
    ));

    let output = run_record(&ledger);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "an executed baseline arm makes the record acceptable, stderr: {stderr}"
    );
    assert_eq!(
        ledger.json_files().len(),
        2,
        "the staged record stays in the ledger"
    );
}
