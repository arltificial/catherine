//! Purpose: the GREEN gate for the probe-verdict derivation (tasks.md
//! task 3, `add-domain-probe-validation`) — the harness derives a
//! probe's verdict from its pre-registered catalog and the recorded
//! comparison, exactly as the kata path derives verdicts from recorded
//! evidence.
//!
//! Responsibilities: pin the closed derivation: an unregistered probe
//! gets no verdict; unique detections survive only over an executed
//! baseline arm; a cited-only arm caps at inconclusive; a uniqueness
//! claim with no arm gets no verdict; an empty unique set is parity
//! (killed).
//!
//! Rationale: `unique_detections` is derived arithmetically from the
//! baseline arm (design Decision 3), so the verdict must never outrun
//! the arm's execution mode. `None` is the honest answer where the
//! catalog has no expectations or the evidence cannot support any of
//! the four words; asserting a verdict there would be exactly the
//! goalpost move the ledger exists to prevent.

use catherine_ledger::entry::{BaselineArm, BaselineMode, Verdict};
use catherine_ledger::harness::derive_probe_verdict;

/// A baseline arm with a committed artifact identity and the given
/// execution mode; detections do not enter this derivation.
fn arm(mode: BaselineMode) -> BaselineArm {
    BaselineArm {
        artifact: "prototype/rule_lens_baseline.py@sha256:4f2a".into(),
        mode,
        detections: Vec::new(),
    }
}

#[test]
fn unregistered_probe_derives_no_verdict() {
    assert_eq!(
        derive_probe_verdict(
            "probe-job-board",
            &["unique-witness".to_string()],
            Some(&arm(BaselineMode::Executed)),
        ),
        None
    );
}

#[test]
fn unique_detections_with_an_executed_arm_survive() {
    assert_eq!(
        derive_probe_verdict(
            "probe-migration",
            &["lost-row-witness".to_string()],
            Some(&arm(BaselineMode::Executed)),
        ),
        Some(Verdict::Survived)
    );
}

#[test]
fn unique_detections_with_a_cited_arm_are_inconclusive() {
    assert_eq!(
        derive_probe_verdict(
            "probe-migration",
            &["lost-row-witness".to_string()],
            Some(&arm(BaselineMode::Cited)),
        ),
        Some(Verdict::Inconclusive)
    );
}

#[test]
fn unique_detections_without_an_arm_derive_no_verdict() {
    assert_eq!(
        derive_probe_verdict("probe-migration", &["lost-row-witness".to_string()], None,),
        None
    );
}

#[test]
fn parity_is_killed() {
    assert_eq!(
        derive_probe_verdict("probe-migration", &[], None),
        Some(Verdict::Killed)
    );
}
