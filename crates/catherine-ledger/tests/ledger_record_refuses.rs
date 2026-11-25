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

mod support;

use std::fs;
use std::process::Command;

use catherine_ledger::entry::{
    seal, BaselineArm, Comparison, Derivation, EntryContent, KataIdentity, KataRecord, Provenance,
    RunLog,
};

use support::{executed_arm, ledger_digests, TempLedger, TestError};

/// The staged candidate file: written OUTSIDE the ledger directory —
/// the binary moves it into the directory itself.
fn candidate(ledger: &TempLedger) -> std::path::PathBuf {
    ledger.root.join("candidate.json")
}

/// Seal one candidate and write it OUTSIDE the ledger directory — the
/// binary stages it into the directory itself.
fn stage_candidate(ledger: &TempLedger, content: EntryContent) -> Result<(), TestError> {
    let entry = seal(content)?;
    fs::write(candidate(ledger), serde_json::to_string(&entry)?)?;
    Ok(())
}

/// The variation axes of the minimal kata fixture: the seq, which kata
/// it claims, whether it is the negative control, its verdict, and the
/// uniqueness claim (the unique detections and the baseline arm that
/// backs them). Fields default to the empty shape, so a call site names
/// only what varies.
#[derive(Default)]
struct KataSpec {
    seq: u32,
    kata_id: &'static str,
    control: bool,
    verdict: &'static str,
    unique: &'static [&'static str],
    baseline: Option<BaselineArm>,
}

/// A minimal kata record — the fields the validator reads, with the
/// uniqueness claim layered on when the spec carries one.
fn kata(spec: KataSpec) -> EntryContent {
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

/// Run the binary over the ledger directory and the staged candidate.
fn run_record(ledger: &TempLedger) -> Result<std::process::Output, TestError> {
    Ok(Command::new(env!("CARGO_BIN_EXE_ledger-record"))
        .arg(&ledger.dir)
        .arg(candidate(ledger))
        .output()?)
}

#[test]
fn ledger_record_refuses_baseline_less_uniqueness_and_reverts(
) -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("refuses-uniqueness")?;
    ledger.write(&[kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
        ..Default::default()
    })])?;
    stage_candidate(
        &ledger,
        kata(KataSpec {
            seq: 2,
            kata_id: "kata-1",
            verdict: "survived",
            unique: &["violation:eq0@row3"],
            ..Default::default()
        }),
    )?;

    let output = run_record(&ledger)?;

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
        ledger_digests(&ledger.dir)?.len(),
        1,
        "the staged record is reverted; only the control remains"
    );
    Ok(())
}

#[test]
fn ledger_record_accepts_an_executed_baseline_arm() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("accepts-executed-baseline")?;
    ledger.write(&[kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
        ..Default::default()
    })])?;
    stage_candidate(
        &ledger,
        kata(KataSpec {
            seq: 2,
            kata_id: "kata-1",
            verdict: "survived",
            unique: &["violation:eq0@row3"],
            baseline: Some(executed_arm()),
            ..Default::default()
        }),
    )?;

    let output = run_record(&ledger)?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "an executed baseline arm makes the record acceptable, stderr: {stderr}"
    );
    assert_eq!(
        ledger_digests(&ledger.dir)?.len(),
        2,
        "the staged record stays in the ledger"
    );
    Ok(())
}
