//! Purpose: the RED gate for tasks.md task 1 (`add-domain-probe-validation`)
//! — the ledger refuses uniqueness claims without an executed, committed
//! baseline arm. `unique_detections` is derived arithmetic (probe detections
//! minus baseline detections), never an assertion; a cited-only baseline
//! cannot support a `survived` verdict. Assertions are structural (named
//! violations), never snapshots.

mod support;

use catherine_ledger::entry::{
    BaselineArm, BaselineMode, Comparison, Derivation, EntryContent, KataIdentity, KataRecord,
    Provenance, RunLog,
};
use catherine_ledger::validate::{validate_dir, Violation};
use support::{TempLedger, TestError};

/// The variation axes of one validator-target kata record: where it sits
/// in the ledger, what it claims, and the uniqueness arithmetic it cites.
/// Fields default to the minimal shape, so a call site names only what
/// varies.
#[derive(Default)]
struct ClaimSpec {
    seq: u32,
    kata_id: &'static str,
    control: bool,
    verdict: &'static str,
    unique: &'static [&'static str],
    baseline: Option<BaselineArm>,
}

/// A minimal kata record — the fields the validator reads.
fn kata(spec: ClaimSpec) -> EntryContent {
    let ClaimSpec {
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
                .map(|(kata, id, verdict)| catherine_ledger::entry::KataVerdict {
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

#[test]
fn rejects_unique_detections_without_baseline_arm() -> Result<(), TestError> {
    let ledger = TempLedger::new("unique-no-baseline")?;
    ledger.write(&[
        kata(ClaimSpec {
            seq: 1,
            kata_id: "kata-6",
            control: true,
            verdict: "killed",
            ..Default::default()
        }),
        kata(ClaimSpec {
            seq: 2,
            kata_id: "kata-1",
            verdict: "survived",
            unique: &["violation:eq0@row3"],
            ..Default::default()
        }),
    ])?;
    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => panic!("a uniqueness claim without a baseline is refused"),
        Err(violations) => violations,
    };
    assert!(
        violations.contains(&Violation::UniqueWithoutBaseline { seq: 2 }),
        "a uniqueness claim without a baseline is refused: want UniqueWithoutBaseline(seq 2), got {violations:?}"
    );
    Ok(())
}

#[test]
fn rejects_survived_with_cited_only_baseline() -> Result<(), TestError> {
    let ledger = TempLedger::new("cited-baseline")?;
    ledger.write(&[
        kata(ClaimSpec {
            seq: 1,
            kata_id: "kata-6",
            control: true,
            verdict: "killed",
            ..Default::default()
        }),
        kata(ClaimSpec {
            seq: 2,
            kata_id: "kata-1",
            verdict: "survived",
            control: false,
            unique: &["violation:eq0@row3"],
            baseline: Some(BaselineArm {
                artifact: "arXiv:1603.07466 (cited only)".into(),
                mode: BaselineMode::Cited,
                detections: vec![],
            }),
        }),
    ])?;
    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => panic!("a cited-only baseline cannot survive"),
        Err(violations) => violations,
    };
    assert!(
        violations.contains(&Violation::CitedBaselineCannotSurvive { seq: 2 }),
        "a cited-only baseline cannot survive: want CitedBaselineCannotSurvive(seq 2), got {violations:?}"
    );
    Ok(())
}

#[test]
fn accepts_unique_claim_with_executed_baseline() -> Result<(), TestError> {
    let ledger = TempLedger::new("executed-baseline")?;
    ledger.write(&[
        kata(ClaimSpec {
            seq: 1,
            kata_id: "kata-6",
            control: true,
            verdict: "killed",
            ..Default::default()
        }),
        kata(ClaimSpec {
            seq: 2,
            kata_id: "kata-1",
            verdict: "survived",
            control: false,
            unique: &["violation:eq0@row3"],
            baseline: Some(BaselineArm {
                artifact: "prototype/migration_baseline.py (sha256:...)".into(),
                mode: BaselineMode::Executed,
                detections: vec!["row-count dry run: 0/6".into()],
            }),
        }),
    ])?;
    let validated = validate_dir(&ledger.dir);
    assert!(
        validated.is_ok(),
        "an executed, committed baseline supports the uniqueness claim, got {:?}",
        validated.err()
    );
    Ok(())
}

#[test]
fn accepts_parity_verdict_without_baseline() -> Result<(), TestError> {
    let ledger = TempLedger::new("parity-no-baseline")?;
    ledger.write(&[
        kata(ClaimSpec {
            seq: 1,
            kata_id: "kata-6",
            control: true,
            verdict: "killed",
            ..Default::default()
        }),
        kata(ClaimSpec {
            seq: 2,
            kata_id: "kata-1",
            verdict: "survived",
            ..Default::default()
        }),
    ])?;
    let validated = validate_dir(&ledger.dir);
    assert!(
        validated.is_ok(),
        "an absent baseline arm caps the claim at parity, got {:?}",
        validated.err()
    );
    Ok(())
}
