//! Purpose: regression pins for the probe ledger (tasks.md task 1). A
//! probe ledger directory reuses the kata shape and therefore the
//! existing refusals: no control, a control that did not self-negate, an
//! unregistered probe id with no derivable verdict. Nothing new is
//! implemented here — the probe directories themselves land in a later
//! task, so these pins must pass against today's validator.
//!
//! Responsibilities: pin the two control obligations for probe ids and
//! the unknown-kata discipline (`catalog::expectations` empty, no rollup
//! verdict derivable) so a later probe-directory task cannot loosen them.
//!
//! Rationale: a probe is a kata record with a different id namespace; if
//! it were exempt from the negative-control gate it would be a place for
//! verdict/record contradictions to hide — exactly ct-catlab's own
//! failure mode (REPORT methodology §1). Assertions are structural
//! (named rejections), never snapshots.

use std::fs;
use std::path::PathBuf;

use catherine_ledger::entry::{seal, Comparison, EntryContent, KataRecord, RunLog};
use catherine_ledger::rollup::derive;
use catherine_ledger::validate::{validate_dir, Violation};

/// A throwaway ledger directory per test (unique suffix, cleaned up).
struct TempLedger(PathBuf);

impl TempLedger {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "catherine-ledger-probe-red-{tag}-{}",
            std::process::id()
        ));
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

/// A minimal kata record — the fields the validator reads. Probe ids ride
/// the same record shape as kata ids.
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
            catalog_hash: None,
            verdict: verdict.into(),
        }),
        rollup: None,
        model_hashes: vec!["sha256:fixture".into()],
        supersedes: None,
    }
}

#[test]
fn probe_sequence_without_self_negating_floor_control_is_refused() {
    let ledger = TempLedger::new("probe-no-control");
    ledger.write(&[kata(1, "probe-migration", false, "survived")]);
    let violations = validate_dir(&ledger.0).expect_err("a probe with no control entry is refused");
    assert!(
        violations.contains(&Violation::NegativeControlMissing { seq: 1 }),
        "expected NegativeControlMissing, got {violations:?}"
    );
}

#[test]
fn floor_control_that_did_not_self_negate_is_refused() {
    let ledger = TempLedger::new("probe-control-survived");
    ledger.write(&[kata(1, "probe-migration-floor", true, "survived")]);
    let violations = validate_dir(&ledger.0).expect_err("a control that survived is refused");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::ControlDidNotSelfNegate { seq: 1, .. })),
        "expected ControlDidNotSelfNegate at seq 1, got {violations:?}"
    );
}

#[test]
fn unregistered_probe_id_yields_empty_expectations_and_no_derivable_verdict() {
    assert!(
        catherine_ledger::catalog::expectations("probe-job-board").is_empty(),
        "an unregistered probe id must have no pre-registered expectations"
    );

    let content = kata(1, "probe-job-board", false, "survived");
    assert!(
        content
            .kata
            .as_ref()
            .expect("kata record")
            .expected
            .is_empty(),
        "the probe entry records honesty: empty expected list"
    );
    let entry = seal(content).expect("entry seals");

    let ledger = TempLedger::new("probe-unregistered");
    ledger.write(&[entry.content.clone()]);
    let violations = validate_dir(&ledger.0).expect_err("an uncontrolled probe entry is refused");
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::NegativeControlMissing { .. })),
        "expected NegativeControlMissing for the unregistered probe, got {violations:?}"
    );

    let error = derive(&[&entry]).expect_err("no verdict is derivable without a control entry");
    assert!(
        error.contains("negative-control"),
        "expected the missing negative-control entry named in the error, got {error:?}"
    );
}
