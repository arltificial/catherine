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
//! goalpost move the ledger exists to prevent. The five cases share one
//! shape — (probe id, unique set, arm mode, expected word) — so they
//! are one table-driven test, not five copies.

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

/// One derivation case: the probe id, the unique detections it claims,
/// the baseline arm it cites (`None` = no arm), and the verdict the
/// arithmetic must derive.
struct Case {
    probe_id: &'static str,
    uniques: &'static [&'static str],
    mode: Option<BaselineMode>,
    verdict: Option<Verdict>,
}

/// The closed derivation table: every arm-mode × unique-set combination
/// and the honest verdict word for each.
const CASES: &[Case] = &[
    // An id absent from the committed catalog derives nothing, no matter
    // how strong the evidence looks.
    Case {
        probe_id: "probe-job-board",
        uniques: &["unique-witness"],
        mode: Some(BaselineMode::Executed),
        verdict: None,
    },
    // Unique detections survive only over an executed arm.
    Case {
        probe_id: "probe-migration",
        uniques: &["lost-row-witness"],
        mode: Some(BaselineMode::Executed),
        verdict: Some(Verdict::Survived),
    },
    // A cited-only arm caps the claim at inconclusive.
    Case {
        probe_id: "probe-migration",
        uniques: &["lost-row-witness"],
        mode: Some(BaselineMode::Cited),
        verdict: Some(Verdict::Inconclusive),
    },
    // A uniqueness claim with no arm cannot outrun the missing baseline.
    Case {
        probe_id: "probe-migration",
        uniques: &["lost-row-witness"],
        mode: None,
        verdict: None,
    },
    // An empty unique set is parity — the baseline already caught it.
    Case {
        probe_id: "probe-migration",
        uniques: &[],
        mode: None,
        verdict: Some(Verdict::Killed),
    },
];

#[test]
fn the_derivation_table_is_closed() {
    for case in CASES {
        let arm = case.mode.map(arm);
        let uniques = case
            .uniques
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>();
        let derived = derive_probe_verdict(case.probe_id, &uniques, arm.as_ref());
        assert_eq!(
            derived.ok(),
            case.verdict,
            "`{}` with uniques {uniques:?} over mode {:?} must derive {:?}",
            case.probe_id,
            case.mode,
            case.verdict,
        );
    }
}
