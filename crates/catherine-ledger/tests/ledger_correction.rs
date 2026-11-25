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

#[path = "ledger_correction/fixtures.rs"]
mod fixtures;
mod support;

use fixtures::{kata, rollup, superseding, write_sealed, KataSpec};

use catherine_ledger::entry::{seal, Entry};
use catherine_ledger::validate::{validate_dir, Violation};

use support::{executed_arm, present, TempLedger, TestError};

#[test]
fn superseded_entry_stops_being_effective() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("superseded")?;
    let control = seal(kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
        ..Default::default()
    }))?;
    let bad = seal(kata(KataSpec {
        seq: 2,
        kata_id: "kata-1",
        verdict: "survived",
        unique: &["violation:eq0@row3"],
        ..Default::default()
    }))?;
    let fix = seal(superseding(
        kata(KataSpec {
            seq: 3,
            kata_id: "kata-1",
            verdict: "survived",
            unique: &["violation:eq0@row3"],
            baseline: Some(executed_arm()),
            ..Default::default()
        }),
        &bad,
    ))?;
    write_sealed(&ledger.dir, &[control, bad, fix])?;

    let valid = validate_dir(&ledger.dir)
        .map_err(|v| format!("a superseding correction is a valid ledger: {v:?}"))?;
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "the superseded claim stops being effective");
    let seqs: Vec<u32> = katas.iter().map(|entry| entry.content.seq).collect();
    assert_eq!(seqs, vec![1, 3], "the control and the correction remain");
    Ok(())
}

/// The pre-correction ledger — control, claim, stale rollup — plus the
/// sealed superseding correction of the claim. The shared setup for the
/// rollup-replacement cases, so each test states only its own rollup.
struct CorrectedRollup {
    control: Entry,
    claim: Entry,
    stale_rollup: Entry,
    fix: Entry,
}

impl CorrectedRollup {
    fn seal() -> Result<Self, TestError> {
        let control = seal(kata(KataSpec {
            seq: 1,
            kata_id: "kata-6",
            control: true,
            verdict: "killed",
            ..Default::default()
        }))?;
        let claim = seal(kata(KataSpec {
            seq: 2,
            kata_id: "kata-1",
            verdict: "survived",
            unique: &["violation:eq0@row3"],
            baseline: Some(executed_arm()),
            ..Default::default()
        }))?;
        let stale_rollup = seal(rollup(
            3,
            &[
                ("kata-6", control.id.as_str(), "killed"),
                ("kata-1", claim.id.as_str(), "survived"),
            ],
        ))?;
        let fix = seal(superseding(
            kata(KataSpec {
                seq: 4,
                kata_id: "kata-1",
                verdict: "survived",
                unique: &["violation:eq0@row3"],
                baseline: Some(executed_arm()),
                ..Default::default()
            }),
            &claim,
        ))?;
        Ok(Self {
            control,
            claim,
            stale_rollup,
            fix,
        })
    }

    /// The replacement rollup at seq 5, superseding the stale one, over
    /// the given (kata_id, entry_id, verdict) lines.
    fn replacement(&self, per_kata: &[(&str, &str, &str)]) -> Result<Entry, TestError> {
        Ok(seal(superseding(rollup(5, per_kata), &self.stale_rollup))?)
    }

    /// The five entries in seq order, with `replacement` last.
    fn entries(&self, replacement: Entry) -> [Entry; 5] {
        [
            self.control.clone(),
            self.claim.clone(),
            self.stale_rollup.clone(),
            self.fix.clone(),
            replacement,
        ]
    }
}

#[test]
fn rollup_after_correction_is_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("corrected-rollup")?;
    let fixture = CorrectedRollup::seal()?;
    let replacement = fixture.replacement(&[
        ("kata-6", fixture.control.id.as_str(), "killed"),
        ("kata-1", fixture.fix.id.as_str(), "survived"),
    ])?;
    write_sealed(&ledger.dir, &fixture.entries(replacement))?;

    let valid = validate_dir(&ledger.dir)
        .map_err(|v| format!("a superseded rollup is replaced, not refused: {v:?}"))?;
    let effective_rollup = present(valid.rollup(), "the corrected ledger's rollup")?;
    assert_eq!(effective_rollup.content.seq, 5);
    Ok(())
}

#[test]
fn rollup_coverage_counts_effective_entries() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("rollup-stale-id")?;
    let fixture = CorrectedRollup::seal()?;
    // The replacement rollup still lists the superseded claim's id.
    let replacement = fixture.replacement(&[
        ("kata-6", fixture.control.id.as_str(), "killed"),
        ("kata-1", fixture.claim.id.as_str(), "survived"),
        ("kata-1", fixture.fix.id.as_str(), "survived"),
    ])?;
    write_sealed(&ledger.dir, &fixture.entries(replacement))?;

    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => return Err("coverage counts effective entries".into()),
        Err(violations) => violations,
    };
    assert!(
        violations.contains(&Violation::RollupIncomplete { seq: 5 }),
        "expected RollupIncomplete(seq 5), got {violations:?}"
    );
    Ok(())
}

#[test]
fn rollup_not_last_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("rollup-not-last")?;
    let control = seal(kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
        ..Default::default()
    }))?;
    let claim = seal(kata(KataSpec {
        seq: 2,
        kata_id: "kata-1",
        verdict: "survived",
        ..Default::default()
    }))?;
    let sealed_rollup = seal(rollup(
        3,
        &[
            ("kata-6", control.id.as_str(), "killed"),
            ("kata-1", claim.id.as_str(), "survived"),
        ],
    ))?;
    let late_kata = seal(kata(KataSpec {
        seq: 4,
        kata_id: "kata-2",
        verdict: "survived",
        ..Default::default()
    }))?;
    write_sealed(&ledger.dir, &[control, claim, sealed_rollup, late_kata])?;

    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => return Err("an effective rollup must be last".into()),
        Err(violations) => violations,
    };
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::RollupMisplaced { seq: 3, .. })),
        "expected RollupMisplaced(seq 3), got {violations:?}"
    );
    Ok(())
}

#[test]
fn baseline_refusal_fires_only_on_effective_entries() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("effective-baseline")?;
    let control = seal(kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
        ..Default::default()
    }))?;
    let bad = seal(kata(KataSpec {
        seq: 2,
        kata_id: "kata-1",
        verdict: "survived",
        unique: &["violation:eq0@row3"],
        ..Default::default()
    }))?;
    // The correction repeats the claim but still has no arm: the refusal
    // now fires at the correction's seq, not the superseded entry's.
    let ineffective_fix = seal(superseding(
        kata(KataSpec {
            seq: 3,
            kata_id: "kata-1",
            verdict: "survived",
            unique: &["violation:eq0@row3"],
            ..Default::default()
        }),
        &bad,
    ))?;
    write_sealed(&ledger.dir, &[control, bad, ineffective_fix])?;

    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => return Err("the effective claim still lacks an arm".into()),
        Err(violations) => violations,
    };
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
    Ok(())
}
