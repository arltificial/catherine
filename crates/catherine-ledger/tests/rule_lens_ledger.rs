//! Purpose: the P8 rule-lens validation ledger's integrity check (openspec
//! change `add-rule-lens`, phase D; design.md Decision 4) — the committed
//! entries, their model-hash references, and the mechanically derived
//! verdict.
//!
//! Responsibilities: validate `ledger/rule-lens/` (ids, seq order, control
//! first); assert the near-floor control self-negated (killed, zero unique
//! detections) and the above-floor entry survived with the recorded unique
//! detections; recompute every `fixture_hash` from the fixture's canonical
//! model bytes; and assert the recorded rollup equals
//! `rollup::derive` over the kata entries — the verdict is derived, never
//! asserted.
//!
//! Rationale: the ledger's content addresses are only evidence if the
//! fixture hashes and the rollup still match the files on disk. This is the
//! same discipline the P6 ledger's validator applies, made executable for
//! the rule-lens entries.

mod probe_support;

use std::path::Path;

use catherine_ledger::rollup::derive;
use catherine_ledger::validate::validate_dir;

use probe_support::{fixture_hash, present, TestError};

#[test]
fn rule_lens_ledger_is_valid_and_derives_its_recorded_verdict() -> Result<(), TestError> {
    let root = Path::new(probe_support::ROOT);
    let valid = validate_dir(&root.join("ledger/rule-lens"))
        .map_err(|violations| format!("{violations:?}"))?;
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "one control + one above-floor kata");

    let control = present(katas[0].content.kata.as_ref(), "control kata")?;
    assert!(
        control.identity.control,
        "the first entry is the negative control"
    );
    assert_eq!(
        control.derivation.verdict, "killed",
        "the control self-negates"
    );
    assert!(
        control.derivation.comparison.unique_detections.is_empty(),
        "the control earns no unique detection"
    );

    let above_floor = present(katas[1].content.kata.as_ref(), "above-floor kata")?;
    assert!(!above_floor.identity.control);
    assert_eq!(above_floor.derivation.verdict, "survived");
    assert_eq!(
        above_floor.derivation.comparison.unique_detections.len(),
        5,
        "the five structural unique detections are recorded"
    );

    for entry in &katas {
        let kata = present(entry.content.kata.as_ref(), "kata record")?;
        assert_eq!(
            kata.provenance.fixture_hash,
            fixture_hash(&kata.provenance.fixture)?,
            "the fixture hash is recomputed from canonical model bytes"
        );
        assert!(
            !entry.content.model_hashes.is_empty(),
            "every entry references the model documents it validated"
        );
    }

    let recorded = present(valid.rollup(), "the ledger ends with a rollup")?;
    let record = present(recorded.content.rollup.as_ref(), "rollup record")?;
    let derived = derive(&katas)?;
    assert_eq!(record.verdict, derived.verdict.as_str());
    assert_eq!(record.control_gate, derived.control_gate);
    let recorded_per_kata: Vec<_> = record
        .per_kata
        .iter()
        .map(|line| (&line.kata_id, &line.entry_id, line.verdict.as_str()))
        .collect();
    let derived_per_kata: Vec<_> = derived
        .per_kata
        .iter()
        .map(|(kata_id, entry_id, verdict)| (kata_id, entry_id, verdict.as_str()))
        .collect();
    assert_eq!(
        recorded_per_kata, derived_per_kata,
        "the rollup is the derived one"
    );
    assert_eq!(
        record.kata_entry_ids,
        katas
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>(),
        "the rollup consumes every kata entry, in seq order"
    );
    Ok(())
}
