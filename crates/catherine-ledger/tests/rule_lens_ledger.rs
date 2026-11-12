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

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::rollup::derive;
use catherine_ledger::validate::validate_dir;

/// The workspace root, from this test crate's manifest directory.
fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The model hash of a workspace-relative fixture path.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(workspace_root().join(path)).expect("fixture exists");
    let model = parse(&text).expect("fixture parses");
    model_hash(&model)
}

#[test]
fn rule_lens_ledger_is_valid_and_derives_its_recorded_verdict() {
    let valid = validate_dir(&workspace_root().join("ledger/rule-lens"))
        .expect("the rule-lens ledger validates");
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "one control + one above-floor kata");

    let control = katas[0].content.kata.as_ref().expect("control kata");
    assert!(control.control, "the first entry is the negative control");
    assert_eq!(control.verdict, "killed", "the control self-negates");
    assert!(
        control.comparison.unique_detections.is_empty(),
        "the control earns no unique detection"
    );

    let above_floor = katas[1].content.kata.as_ref().expect("above-floor kata");
    assert!(!above_floor.control);
    assert_eq!(above_floor.verdict, "survived");
    assert_eq!(
        above_floor.comparison.unique_detections.len(),
        5,
        "the five structural unique detections are recorded"
    );

    for entry in &katas {
        let kata = entry.content.kata.as_ref().expect("kata record");
        assert_eq!(
            kata.fixture_hash,
            fixture_hash(&kata.fixture),
            "the fixture hash is recomputed from canonical model bytes"
        );
        assert!(
            !entry.content.model_hashes.is_empty(),
            "every entry references the model documents it validated"
        );
    }

    let recorded = valid.rollup().expect("the ledger ends with a rollup");
    let record = recorded.content.rollup.as_ref().expect("rollup record");
    let derived = derive(&katas).expect("the committed procedure derives");
    assert_eq!(record.verdict, derived.verdict.as_str());
    assert_eq!(record.control_gate, derived.control_gate);
    let per_kata: Vec<(String, String, String)> = record
        .per_kata
        .iter()
        .map(|line| {
            (
                line.kata_id.clone(),
                line.entry_id.clone(),
                line.verdict.clone(),
            )
        })
        .collect();
    let expected: Vec<(String, String, String)> = derived
        .per_kata
        .iter()
        .map(|(kata_id, entry_id, verdict)| {
            (
                kata_id.clone(),
                entry_id.clone(),
                verdict.as_str().to_string(),
            )
        })
        .collect();
    assert_eq!(per_kata, expected, "the rollup is the derived one");
    assert_eq!(
        record.kata_entry_ids,
        katas
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>(),
        "the rollup consumes every kata entry, in seq order"
    );
}
