//! Purpose: the recorded evidence of the composite-null-object kata
//! (`catherine-lgl.1`) — the committed ledger is validated, every kata
//! fixture's model hash is pinned, the control went first and
//! self-negated, and each positive entry's unique set is the arithmetic
//! the executed conventional baseline arm was run for.
//!
//! Responsibilities: recompute each fixture's canonical model hash (a
//! silent fixture edit must break the entry's content address), validate
//! `ledger/katas/composite-null-object/` through the real validator, and
//! assert the control-first / self-negation / executed-baseline rules
//! the shared kata contract fixes.
//!
//! Rationale: the baseline script and this test are the two frozen
//! things; the ledger entry is the science between them.

use std::path::Path;

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::entry::BaselineMode;
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const BASELINE_SHA: &str = "83d443b58296df3cdb5213efff6888fe974f190e8e01fe85684aa390469b4a6a";

/// One fixture's model hash, recomputed from its canonical model bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// The six committed fixtures' identities are pinned: an edit to any of
/// them must be a new ledger entry, never a silent change.
#[test]
fn fixture_model_hashes_are_pinned() {
    let pinned = [
        (
            "fixtures/katas/composite-null-object/composite-null-object.ct.json",
            "sha256:7c92da76b39d448a0dd1d53716c335fe52a8bf3e7cb39607543245da9a4da07d",
        ),
        (
            "fixtures/katas/composite-null-object/floor/floor-composite-null-object.ct.json",
            "sha256:958e42df3ce2b6caa4bb50e48346b0d9e6d2ef4e47d9e856e0fdfb6624ae6cea",
        ),
        (
            "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json",
            "sha256:e6cee3da7836480d3655cf27883177855de6b78f139c26b4e9d007ebce7b11c8",
        ),
        (
            "fixtures/katas/composite-null-object/fault-broken-parent-link.ct.json",
            "sha256:237469904b2571d61ca780739c09bd9b6f7f6f61771a3096736150924b4f3c81",
        ),
        (
            "fixtures/katas/composite-null-object/fault-nil-as-real-leaf.ct.json",
            "sha256:c5a40e2efb6ae1026fda516511bbdbc740170194bcabb5595eba0760500b466e",
        ),
        (
            "fixtures/katas/composite-null-object/fault-lost-child-migrate.ct.json",
            "sha256:a7804500e75c05e2bc88f3a156a986b0f455aa6efc0a97c6a4f08b75bb78f3bf",
        ),
    ];
    for (path, hash) in pinned {
        assert_eq!(fixture_hash(path), hash, "fixture hash drift: {path}");
    }
}

/// The ledger validates and the control is the first kata entry, `killed`
/// with an empty unique set: the floor self-negates.
#[test]
fn control_goes_first_and_self_negates() {
    let dir = Path::new(ROOT).join("ledger/katas/composite-null-object");
    let valid = validate_dir(&dir).expect("the kata ledger validates");
    assert_eq!(valid.entries.len(), 4, "control + three positives");
    let kata_entries = valid.kata_entries();
    assert_eq!(kata_entries.len(), 4);
    let control = kata_entries[0].content.kata.clone().expect("kata record");
    assert!(control.control, "the floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());
    assert_eq!(control.fixture_hash, fixture_hash(&control.fixture));
    assert_eq!(
        control.catalog_hash, None,
        "this kata id is not a registered probe"
    );
}

/// Each positive entry carries a non-empty unique set, backed by the
/// executed baseline arm whose pin names the frozen script's sha256, and
/// its recorded fixture hash recomputes.
#[test]
fn positives_carry_the_preregistered_unique_elements() {
    let dir = Path::new(ROOT).join("ledger/katas/composite-null-object");
    let valid = validate_dir(&dir).expect("the kata ledger validates");
    let expected = vec![
        vec!["dangling:parent@Node#3".to_string()],
        vec!["violation:eq0@Node#4".to_string()],
        (0..7)
            .map(|row| format!("lost_cell:parent#{row}"))
            .collect(),
    ];
    for (entry, elements) in valid.kata_entries()[1..].iter().zip(expected) {
        let kata = entry.content.kata.as_ref().expect("kata record");
        assert!(!kata.control);
        assert_eq!(kata.comparison.unique_detections, elements);
        let baseline = kata
            .comparison
            .baseline
            .as_ref()
            .expect("a unique claim needs an executed baseline arm");
        assert_eq!(baseline.mode, BaselineMode::Executed);
        assert!(
            baseline.artifact.contains(BASELINE_SHA),
            "the arm pins the committed script: {}",
            baseline.artifact
        );
        assert_eq!(kata.fixture_hash, fixture_hash(&kata.fixture));
        assert_eq!(
            entry.content.model_hashes,
            vec![kata.fixture_hash.clone()],
            "the validated model is the fixture it names"
        );
    }
}
