//! Purpose: the recorded evidence of `probe-composition`
//! (`add-model-composition`, ro5u EXCL-001) — the committed composition
//! probe ledger validates, its fixture hashes recompute from the canonical
//! model bytes, its expectations are exactly the pre-registered catalog,
//! and every recorded verdict equals the arithmetic derivation from the
//! entry's own comparison data.
//!
//! Responsibilities: validate the ledger directory through the real
//! validator, pin the floor-control and seam fixtures' model hashes, check
//! that the control went first and self-negated while the seam arm records
//! the parity null result with its named gap, and compare each recorded
//! verdict with [`catherine_ledger::harness::derive_probe_verdict`].
//!
//! Rationale: `ledger-check` validates the closed vocabulary but cannot
//! notice a verdict word that contradicts the arithmetic. The derivation
//! assertion makes that drift a test failure; the fixture-hash pins keep
//! the entry from citing bytes that no longer exist, so a fixture edit must
//! become a new ledger entry, never a silent change.

mod probe_support;

use std::path::Path;

use catherine_ledger::validate::validate_dir;

use probe_support::{
    assert_entry_cites_the_registration, assert_fixture_hash_is_current, fixture_hash, present,
    TestError, ROOT,
};

/// The four documents the seam arm validated: the committed hand-merge and
/// the three gap-suite inputs it unions, in the entry's recorded order.
const SEAM_MODELS: &[&str] = &[
    "fixtures/probes/composition/hand_merged_seam.ct.json",
    "crates/catherine-cli/tests/fixtures/gap_suite/composition_billing.ct.json",
    "crates/catherine-cli/tests/fixtures/gap_suite/composition_shipping.ct.json",
    "crates/catherine-cli/tests/fixtures/gap_suite/composition_seam_claim.ct.json",
];

/// The two validated composition fixtures, path and pinned canonical hash.
const PINNED: [(&str, &str); 2] = [
    (
        "fixtures/probes/composition/floor_dangling.ct.json",
        "sha256:4e251b9ff3f7a7d3169b29a8990a806c2f8f990dbaeff7d784d848194c8350cf",
    ),
    (
        "fixtures/probes/composition/hand_merged_seam.ct.json",
        "sha256:579d5b0db32c01cefabe9fea30b3725b08269e3fa6f852dfa84a4759c25885d5",
    ),
];

/// Both probe fixtures are pinned: an edit to a committed probe fixture
/// must be a new ledger entry, never a silent change.
#[test]
fn composition_fixture_hashes_are_pinned() -> Result<(), TestError> {
    for (path, pin) in PINNED {
        assert_eq!(fixture_hash(path)?, pin, "{path} drifted from its pin");
    }
    Ok(())
}

/// The control went first, self-negated, and its recorded fixture is the
/// single document the two arms each executed.
#[test]
fn composition_control_arm_self_negates() -> Result<(), TestError> {
    let dir = Path::new(ROOT).join("ledger/probes/composition");
    let valid = validate_dir(&dir).map_err(|violations| format!("{violations:?}"))?;
    assert_eq!(valid.entries.len(), 2, "control + seam arm");
    let katas = valid.kata_entries();
    let control = present(katas[0].content.kata.as_ref(), "kata")?;
    assert!(control.identity.control, "the floor control is first");
    assert_eq!(
        control.derivation.verdict, "killed",
        "the control self-negated"
    );
    assert!(
        control.derivation.comparison.unique_detections.is_empty(),
        "the single-document check already reports the dangling cell"
    );
    assert_eq!(
        control.provenance.fixture, "fixtures/probes/composition/floor_dangling.ct.json",
        "the control runs the floor fixture"
    );
    assert_eq!(
        katas[0].content.model_hashes,
        vec![fixture_hash(&control.provenance.fixture)?],
        "the one validated model is the fixture it names"
    );
    Ok(())
}

/// The probe arm records the pre-registered expectations and names its
/// eliminated-merge-step claim as a gap, never as a unique detection; every
/// recorded verdict is the one the comparison arithmetic derives.
#[test]
fn composition_probe_ledger_records_the_preregistered_evidence() -> Result<(), TestError> {
    let dir = Path::new(ROOT).join("ledger/probes/composition");
    let valid = validate_dir(&dir).map_err(|violations| format!("{violations:?}"))?;
    let katas = valid.kata_entries();
    let seam = present(katas[1].content.kata.as_ref(), "kata")?;
    assert!(!seam.identity.control, "the second entry is the probe arm");
    assert_eq!(
        seam.identity.kata_id, "probe-composition",
        "the registered probe id"
    );
    assert!(
        seam.derivation.comparison.unique_detections.is_empty(),
        "the hand-merged baseline detects the same row: parity, no unique detections"
    );
    assert_eq!(
        seam.derivation.verdict, "killed",
        "parity derives killed — the merge-step elimination is a gap, not a detection"
    );
    assert!(
        !seam.expressibility_gap.is_empty(),
        "the eliminated hand-merge step is the recorded contribution"
    );
    let seam_models = SEAM_MODELS
        .iter()
        .map(|path| fixture_hash(path))
        .collect::<Result<Vec<_>, TestError>>()?;
    assert_eq!(
        katas[1].content.model_hashes, seam_models,
        "the validated models are the hand-merge plus the three seam inputs"
    );
    for entry in &katas {
        assert_entry_cites_the_registration(entry, "probe-composition")?;
        assert_fixture_hash_is_current(entry)?;
    }
    Ok(())
}
