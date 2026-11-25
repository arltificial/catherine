//! Purpose: the refusal and invariance proofs for the domain-probe
//! ledger (`add-domain-probe-validation`, tasks.md task 4) — the
//! task-1 rejection matrix still refuses mutated probe ledgers,
//! supersession keeps the ledger append-only, and recording probes never
//! touches the sealed kata rollup.
//!
//! Responsibilities: pin the four refusals (`UniqueWithoutBaseline`,
//! `CitedBaselineCannotSurvive`, `NegativeControlMissing`,
//! `ControlDidNotSelfNegate`), prove a correction is a superseding entry,
//! and compare `ledger-check` stdout/status plus every `ledger/*.json`
//! digest before and after probe recording.
//!
//! Rationale: the kata rollup is a sealed artifact of a different
//! ledger; probe recording MUST leave it byte-identical, so invariance is
//! asserted by digest, never by convention.

mod support;

use std::path::Path;

use catherine_ledger::catalog;
use catherine_ledger::entry::{seal, BaselineArm, BaselineMode, EntryContent, Verdict};
use catherine_ledger::harness::derive_probe_verdict;
use catherine_ledger::validate::{validate_dir, Violation};

use support::{
    executed_arm, green_contents, ledger_digests, present, probe_kata, run_ledger_check, ProbeSpec,
    TempLedger, TestError, UNIQUE,
};

/// A mutated ledger must come back refused; a clean pass is the failure.
fn refused(ledger: &TempLedger, what: &str) -> Result<Vec<Violation>, TestError> {
    match validate_dir(&ledger.dir) {
        Err(violations) => Ok(violations),
        Ok(_) => Err(format!("{what}: the mutated ledger validated").into()),
    }
}

/// A unique claim needs a committed, executed baseline arm.
#[test]
fn unique_claim_without_a_baseline_arm_is_refused() -> Result<(), TestError> {
    // (a) a unique claim with no baseline arm.
    let no_baseline = TempLedger::new("no-baseline")?;
    let mut contents = green_contents();
    let probe = present(contents[1].kata.as_mut(), "probe kata")?;
    probe.derivation.comparison.baseline = None;
    no_baseline.write(&contents)?;
    let violations = refused(&no_baseline, "no arm, no uniqueness")?;
    assert!(
        violations.contains(&Violation::UniqueWithoutBaseline { seq: 2 }),
        "expected UniqueWithoutBaseline at seq 2, got {violations:?}"
    );

    // (b) a cited-only arm cannot support `survived`.
    let cited_only = TempLedger::new("cited-survived")?;
    let mut contents = green_contents();
    let probe = present(contents[1].kata.as_mut(), "probe kata")?;
    let arm = present(probe.derivation.comparison.baseline.as_mut(), "green arm")?;
    arm.mode = BaselineMode::Cited;
    cited_only.write(&contents)?;
    let violations = refused(&cited_only, "cited-only caps at inconclusive")?;
    assert!(
        violations.contains(&Violation::CitedBaselineCannotSurvive { seq: 2 }),
        "expected CitedBaselineCannotSurvive at seq 2, got {violations:?}"
    );
    Ok(())
}

/// A probe sequence is refused unless a self-negating control came first.
#[test]
fn probe_sequence_without_a_self_negating_control_is_refused() -> Result<(), TestError> {
    // (c) a probe entry with no negative control.
    let no_control = TempLedger::new("no-control")?;
    no_control.write(&[probe_kata(ProbeSpec {
        seq: 1,
        kata_id: "probe-migration",
        verdict: "survived",
        expected: catalog::probe_expectations("probe-migration"),
        catalog_hash: catalog::probe_catalog_hash("probe-migration"),
        unique: &UNIQUE,
        baseline: Some(executed_arm()),
        ..Default::default()
    })])?;
    let violations = refused(&no_control, "an uncontrolled probe is refused")?;
    assert!(
        violations.contains(&Violation::NegativeControlMissing { seq: 1 }),
        "expected NegativeControlMissing at seq 1, got {violations:?}"
    );

    // (d) a control that did not self-negate.
    let surviving_control = TempLedger::new("control-survived")?;
    let mut contents = green_contents();
    let control = present(contents[0].kata.as_mut(), "control kata")?;
    control.derivation.verdict = "survived".into();
    surviving_control.write(&contents)?;
    let violations = refused(&surviving_control, "a control that survived is refused")?;
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::ControlDidNotSelfNegate { seq: 1, .. })),
        "expected ControlDidNotSelfNegate at seq 1, got {violations:?}"
    );
    Ok(())
}

/// The superseding correction entry: a new probe naming the superseded
/// id (append-only supersession, never an edit), its verdict capped at
/// inconclusive by the cited-only baseline arm, with the recorded
/// probes' unique set and catalog.
fn correction_entry(superseded_id: &str, baseline: BaselineArm) -> EntryContent {
    let mut correction = probe_kata(ProbeSpec {
        seq: 3,
        kata_id: "probe-migration",
        verdict: "inconclusive",
        expected: catalog::probe_expectations("probe-migration"),
        catalog_hash: catalog::probe_catalog_hash("probe-migration"),
        unique: &UNIQUE,
        baseline: Some(baseline),
        ..Default::default()
    });
    correction.supersedes = Some(superseded_id.to_string());
    correction
}

#[test]
fn superseding_a_probe_entry_keeps_the_ledger_append_only() -> Result<(), TestError> {
    let ledger = TempLedger::new("supersede")?;
    let mut contents = green_contents();
    let original = seal(contents[1].clone())?;

    // The correction is a new entry naming the old id — the cited arm caps
    // the verdict at inconclusive, with the same unique set and catalog.
    let cited = BaselineArm {
        artifact: "docs/migration-dry-run.md (pinned 2026-10-30)".into(),
        mode: BaselineMode::Cited,
        detections: executed_arm().detections,
    };
    contents.push(correction_entry(&original.id, cited.clone()));
    ledger.write(&contents)?;

    let valid = validate_dir(&ledger.dir).map_err(|violations| format!("{violations:?}"))?;
    let effective_entries = valid.kata_entries();
    let seqs: Vec<u32> = effective_entries
        .iter()
        .map(|entry| entry.content.seq)
        .collect();
    assert_eq!(seqs, vec![1, 3], "the superseded entry is not effective");

    let unique: Vec<String> = UNIQUE.iter().map(|s| s.to_string()).collect();
    let effective = present(
        effective_entries[1].content.kata.as_ref(),
        "effective probe kata",
    )?;
    assert_eq!(
        effective.derivation.comparison.unique_detections, unique,
        "the correction records the same unique set"
    );
    assert_eq!(
        derive_probe_verdict("probe-migration", &unique, Some(&cited))?,
        Verdict::Inconclusive,
        "a cited-only arm derives inconclusive, matching the recorded correction"
    );
    assert_eq!(effective.derivation.verdict, "inconclusive");
    Ok(())
}

#[test]
fn kata_ledger_is_byte_identical_after_probe_recording() -> Result<(), TestError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let kata_ledger = root.join("ledger");
    let bin = env!("CARGO_BIN_EXE_ledger-check");

    let before = run_ledger_check(bin, &kata_ledger)?;
    let before_digests = ledger_digests(&kata_ledger)?;
    assert!(
        !before_digests.is_empty(),
        "the committed kata ledger has sealed entries"
    );

    let probe_ledger = TempLedger::new("kata-byte-identity")?;
    probe_ledger.write(&green_contents())?;
    validate_dir(&probe_ledger.dir).map_err(|violations| format!("{violations:?}"))?;

    let after = run_ledger_check(bin, &kata_ledger)?;
    let after_digests = ledger_digests(&kata_ledger)?;

    assert_eq!(
        before.0, after.0,
        "ledger-check reports the same derived rollup after probe recording"
    );
    assert_eq!(before.1, after.1, "ledger-check exits the same way");
    assert_eq!(
        before_digests, after_digests,
        "no sealed kata ledger file is rewritten by probe recording"
    );
    Ok(())
}
