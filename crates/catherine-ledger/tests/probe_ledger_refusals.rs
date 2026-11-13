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
use catherine_ledger::entry::{seal, BaselineArm, BaselineMode, Verdict};
use catherine_ledger::harness::derive_probe_verdict;
use catherine_ledger::validate::{validate_dir, Violation};

use support::{
    executed_arm, green_contents, ledger_digests, probe_kata, run_ledger_check, TempLedger, UNIQUE,
};

/// A unique claim needs a committed, executed baseline arm.
#[test]
fn unique_claim_without_a_baseline_arm_is_refused() {
    // (a) a unique claim with no baseline arm.
    let no_baseline = TempLedger::new("no-baseline");
    let mut contents = green_contents();
    contents[1]
        .kata
        .as_mut()
        .expect("probe kata")
        .comparison
        .baseline = None;
    no_baseline.write(&contents);
    let violations = validate_dir(&no_baseline.dir).expect_err("no arm, no uniqueness");
    assert!(
        violations.contains(&Violation::UniqueWithoutBaseline { seq: 2 }),
        "expected UniqueWithoutBaseline at seq 2, got {violations:?}"
    );

    // (b) a cited-only arm cannot support `survived`.
    let cited_only = TempLedger::new("cited-survived");
    let mut contents = green_contents();
    let arm = contents[1]
        .kata
        .as_mut()
        .expect("probe kata")
        .comparison
        .baseline
        .as_mut()
        .expect("green arm");
    arm.mode = BaselineMode::Cited;
    cited_only.write(&contents);
    let violations = validate_dir(&cited_only.dir).expect_err("cited-only caps at inconclusive");
    assert!(
        violations.contains(&Violation::CitedBaselineCannotSurvive { seq: 2 }),
        "expected CitedBaselineCannotSurvive at seq 2, got {violations:?}"
    );
}

/// A probe sequence is refused unless a self-negating control came first.
#[test]
fn probe_sequence_without_a_self_negating_control_is_refused() {
    // (c) a probe entry with no negative control.
    let no_control = TempLedger::new("no-control");
    no_control.write(&[probe_kata(
        1,
        "probe-migration",
        false,
        "survived",
        catalog::probe_expectations("probe-migration"),
        catalog::probe_catalog_hash("probe-migration"),
        &UNIQUE,
        Some(executed_arm()),
    )]);
    let violations = validate_dir(&no_control.dir).expect_err("an uncontrolled probe is refused");
    assert!(
        violations.contains(&Violation::NegativeControlMissing { seq: 1 }),
        "expected NegativeControlMissing at seq 1, got {violations:?}"
    );

    // (d) a control that did not self-negate.
    let surviving_control = TempLedger::new("control-survived");
    let mut contents = green_contents();
    contents[0].kata.as_mut().expect("control kata").verdict = "survived".into();
    surviving_control.write(&contents);
    let violations =
        validate_dir(&surviving_control.dir).expect_err("a control that survived is refused");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::ControlDidNotSelfNegate { seq: 1, .. })),
        "expected ControlDidNotSelfNegate at seq 1, got {violations:?}"
    );
}

#[test]
fn superseding_a_probe_entry_keeps_the_ledger_append_only() {
    let ledger = TempLedger::new("supersede");
    let mut contents = green_contents();
    let original = seal(contents[1].clone()).expect("the original probe entry seals");

    // The correction is a new entry naming the old id — the cited arm caps
    // the verdict at inconclusive, with the same unique set and catalog.
    let cited = BaselineArm {
        artifact: "docs/migration-dry-run.md (pinned 2026-10-30)".into(),
        mode: BaselineMode::Cited,
        detections: executed_arm().detections,
    };
    let mut correction = probe_kata(
        3,
        "probe-migration",
        false,
        "inconclusive",
        catalog::probe_expectations("probe-migration"),
        catalog::probe_catalog_hash("probe-migration"),
        &UNIQUE,
        Some(cited.clone()),
    );
    correction.supersedes = Some(original.id.clone());
    contents.push(correction);
    ledger.write(&contents);

    let valid = validate_dir(&ledger.dir).expect("supersession is the correction path");
    let effective_entries = valid.kata_entries();
    let seqs: Vec<u32> = effective_entries
        .iter()
        .map(|entry| entry.content.seq)
        .collect();
    assert_eq!(seqs, vec![1, 3], "the superseded entry is not effective");

    let unique: Vec<String> = UNIQUE.iter().map(|s| s.to_string()).collect();
    let effective = effective_entries[1]
        .content
        .kata
        .as_ref()
        .expect("effective probe kata");
    assert_eq!(
        effective.comparison.unique_detections, unique,
        "the correction records the same unique set"
    );
    assert_eq!(
        derive_probe_verdict("probe-migration", &unique, Some(&cited)),
        Some(Verdict::Inconclusive),
        "a cited-only arm derives inconclusive, matching the recorded correction"
    );
    assert_eq!(effective.verdict, "inconclusive");
}

#[test]
fn kata_ledger_is_byte_identical_after_probe_recording() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let kata_ledger = root.join("ledger");
    let bin = env!("CARGO_BIN_EXE_ledger-check");

    let before = run_ledger_check(bin, &kata_ledger);
    let before_digests = ledger_digests(&kata_ledger);
    assert!(
        !before_digests.is_empty(),
        "the committed kata ledger has sealed entries"
    );

    let probe_ledger = TempLedger::new("kata-byte-identity");
    probe_ledger.write(&green_contents());
    validate_dir(&probe_ledger.dir).expect("the green probe ledger validates");

    let after = run_ledger_check(bin, &kata_ledger);
    let after_digests = ledger_digests(&kata_ledger);

    assert_eq!(
        before.0, after.0,
        "ledger-check reports the same derived rollup after probe recording"
    );
    assert_eq!(before.1, after.1, "ledger-check exits the same way");
    assert_eq!(
        before_digests, after_digests,
        "no sealed kata ledger file is rewritten by probe recording"
    );
}
