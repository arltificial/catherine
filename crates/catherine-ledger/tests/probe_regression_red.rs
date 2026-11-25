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

mod support;

use catherine_ledger::catalog;
use catherine_ledger::entry::seal;
use catherine_ledger::rollup::derive;
use catherine_ledger::validate::{validate_dir, Violation};

use support::{present, probe_kata, ProbeSpec, TempLedger, TestError};

#[test]
fn probe_sequence_without_self_negating_floor_control_is_refused() -> Result<(), TestError> {
    let ledger = TempLedger::new("probe-no-control")?;
    ledger.write(&[probe_kata(ProbeSpec {
        seq: 1,
        kata_id: "probe-migration",
        verdict: "survived",
        ..Default::default()
    })])?;
    let Err(violations) = validate_dir(&ledger.dir) else {
        return Err("a probe with no control entry is refused: the ledger validated".into());
    };
    assert!(
        violations.contains(&Violation::NegativeControlMissing { seq: 1 }),
        "expected NegativeControlMissing, got {violations:?}"
    );
    Ok(())
}

#[test]
fn floor_control_that_did_not_self_negate_is_refused() -> Result<(), TestError> {
    let ledger = TempLedger::new("probe-control-survived")?;
    ledger.write(&[probe_kata(ProbeSpec {
        seq: 1,
        kata_id: "probe-migration-floor",
        control: true,
        verdict: "survived",
        ..Default::default()
    })])?;
    let Err(violations) = validate_dir(&ledger.dir) else {
        return Err("a control that survived is refused: the ledger validated".into());
    };
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::ControlDidNotSelfNegate { seq: 1, .. })),
        "expected ControlDidNotSelfNegate at seq 1, got {violations:?}"
    );
    Ok(())
}

#[test]
fn unregistered_probe_id_yields_empty_expectations_and_no_derivable_verdict(
) -> Result<(), TestError> {
    assert!(
        catalog::expectations("probe-job-board").is_empty(),
        "an unregistered probe id must have no pre-registered expectations"
    );

    let content = probe_kata(ProbeSpec {
        seq: 1,
        kata_id: "probe-job-board",
        verdict: "survived",
        ..Default::default()
    });
    let kata = present(content.kata.as_ref(), "kata record")?;
    assert!(
        kata.expected.is_empty(),
        "the probe entry records honesty: empty expected list"
    );
    let entry = seal(content)?;

    let ledger = TempLedger::new("probe-unregistered")?;
    ledger.write(std::slice::from_ref(&entry.content))?;
    let Err(violations) = validate_dir(&ledger.dir) else {
        return Err("an uncontrolled probe entry is refused: the ledger validated".into());
    };
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::NegativeControlMissing { .. })),
        "expected NegativeControlMissing for the unregistered probe, got {violations:?}"
    );

    let Err(error) = derive(&[&entry]) else {
        return Err("no verdict is derivable without a control entry: derive succeeded".into());
    };
    assert!(
        error.contains("negative-control"),
        "expected the missing negative-control entry named in the error, got {error:?}"
    );
    Ok(())
}
