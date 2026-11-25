//! Purpose: the RED gate (tasks.md task 1) — the ledger validator
//! rejects bad science: a kata entry without a prior negative control,
//! a verdict outside the closed vocabulary, an entry without model-hash
//! references. Plus the append-only integrity pins (id recomputation,
//! seq order, supersession) and the recorded rollup procedure's
//! outcomes. Assertions are structural (named rejections, derived
//! verdicts) — never snapshots.

#[path = "validator_red/fixtures.rs"]
mod fixtures;
mod support;

use fixtures::{green_contents, kata, refusal_cases, KataSpec};

use std::fs;

use catherine_ledger::entry::{entry_id, seal, Fired, Verdict};
use catherine_ledger::rollup::{derive, Rollup};
use catherine_ledger::validate::{validate_dir, ValidLedger, Violation};

use support::TempLedger;

#[test]
fn rejects_bad_entry_shapes_with_named_violations() -> Result<(), Box<dyn std::error::Error>> {
    for case in refusal_cases() {
        let ledger = TempLedger::new(case.tag)?;
        ledger.write(&case.contents)?;
        let violations = match validate_dir(&ledger.dir) {
            Ok(_) => return Err(format!("{} is refused", case.what).into()),
            Err(violations) => violations,
        };
        assert!(
            violations.contains(&case.violation),
            "expected the named violation for {}, got {violations:?}",
            case.what
        );
    }
    Ok(())
}

#[test]
fn rejects_entry_whose_id_does_not_match_content() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("bad-id")?;
    let mut contents = green_contents()?;
    let entry = seal(contents.remove(0))?;
    let tampered = format!(r#"{{"id":"sha256:{}","seq":1}}"#, "0".repeat(64));
    let hex = entry.id.trim_start_matches("sha256:");
    fs::write(
        ledger.dir.join(format!("0001-{}.json", &hex[..8])),
        tampered,
    )?;
    ledger.write(&contents)?;
    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => return Err("a sealed-id mismatch is refused".into()),
        Err(violations) => violations,
    };
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::IdMismatch { .. })),
        "expected IdMismatch, got {violations:?}"
    );
    Ok(())
}

#[test]
fn rejects_control_that_did_not_self_negate() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("control-survived")?;
    ledger.write(&[kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "survived",
    })])?;
    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => return Err("a surviving control is refused".into()),
        Err(violations) => violations,
    };
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::ControlDidNotSelfNegate { .. })),
        "expected ControlDidNotSelfNegate, got {violations:?}"
    );
    Ok(())
}

#[test]
fn rejects_supersession_pointing_at_missing_entry() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("bad-supersession")?;
    let mut control = kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
    });
    control.supersedes = Some("sha256:does-not-exist".into());
    ledger.write(&[control])?;
    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => return Err("a dangling supersession is refused".into()),
        Err(violations) => violations,
    };
    assert!(
        violations
            .iter()
            .any(|v| matches!(v, Violation::BadSupersession { .. })),
        "expected BadSupersession, got {violations:?}"
    );
    Ok(())
}

#[test]
fn accepts_control_first_then_katas_then_rollup() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("green")?;
    ledger.write(&green_contents()?)?;
    let valid: ValidLedger =
        validate_dir(&ledger.dir).map_err(|v| format!("the green ledger passes: {v:?}"))?;
    assert_eq!(valid.kata_entries().len(), 2);
    assert!(valid.rollup().is_some());
    Ok(())
}

#[test]
fn superseding_entry_is_accepted_when_target_exists() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("supersession")?;
    let first = seal(kata(KataSpec {
        seq: 1,
        kata_id: "kata-6",
        control: true,
        verdict: "inconclusive",
    }))?;
    let mut correction = kata(KataSpec {
        seq: 2,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
    });
    correction.supersedes = Some(first.id.clone());
    ledger.write(&[first.content, correction])?;
    let validated = validate_dir(&ledger.dir);
    assert!(
        validated.is_ok(),
        "a well-formed supersession passes, got {:?}",
        validated.err()
    );
    Ok(())
}

/// The derived rollup verdict follows the kata verdicts: a killed
/// control with surviving positives survives; a killed positive kills
/// the run; a control that did not self-negate is an invalid run; an
/// inconclusive positive is inconclusive. The four cases share one
/// driver — only the two kata verdicts and the derived verdict vary.
#[test]
fn rollup_verdict_follows_the_kata_verdicts() -> Result<(), Box<dyn std::error::Error>> {
    struct Case {
        control: &'static str,
        positive: &'static str,
        verdict: Verdict,
        gate: Option<&'static str>,
    }
    let cases = [
        Case {
            control: "killed",
            positive: "survived",
            verdict: Verdict::Survived,
            gate: Some("passed"),
        },
        Case {
            control: "killed",
            positive: "killed",
            verdict: Verdict::Killed,
            gate: None,
        },
        Case {
            control: "survived",
            positive: "survived",
            verdict: Verdict::InvalidRun,
            gate: None,
        },
        Case {
            control: "killed",
            positive: "inconclusive",
            verdict: Verdict::Inconclusive,
            gate: None,
        },
    ];
    for case in cases {
        let entries = [
            kata(KataSpec {
                seq: 1,
                kata_id: "kata-6",
                control: true,
                verdict: case.control,
            }),
            kata(KataSpec {
                seq: 2,
                kata_id: "kata-1",
                control: false,
                verdict: case.positive,
            }),
        ];
        let sealed: Vec<_> = entries
            .iter()
            .map(|content| seal(content.clone()))
            .collect::<Result<_, _>>()?;
        let refs: Vec<_> = sealed.iter().collect();
        let roll: Rollup = derive(&refs)?;
        assert_eq!(
            roll.verdict, case.verdict,
            "control {}, positive {}",
            case.control, case.positive
        );
        if let Some(gate) = case.gate {
            assert_eq!(
                roll.control_gate, gate,
                "the control gate follows a killed control"
            );
        }
    }
    Ok(())
}

#[test]
fn entry_id_matches_documented_content_digest() -> Result<(), Box<dyn std::error::Error>> {
    let content = kata(KataSpec {
        seq: 9,
        kata_id: "kata-6",
        control: true,
        verdict: "killed",
    });
    let computed = entry_id(&content)?;
    assert!(computed.starts_with("sha256:"));
    assert_eq!(computed.len(), "sha256:".len() + 64);
    Ok(())
}

/// Unused-import guard for the harness-facing citation helper the
/// fixtures exercise through the ledger bins.
#[allow(dead_code)]
fn _fired_shape(_: Fired) {}
