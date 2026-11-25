//! Purpose: the recorded evidence of `probe-naturality`
//! (`add-instance-morphism`, ro5u D-01/T-04) — the committed naturality
//! probe ledger validates, its fixture hashes recompute from the
//! canonical model bytes, its expectations are exactly the pre-registered
//! catalog, and every recorded verdict equals the arithmetic derivation
//! from the entry's own comparison data.
//!
//! Responsibilities: validate the ledger directory through the real
//! validator, pin both probe fixtures' model hashes, check the control
//! self-negates while the wrong-question arm records the null result, and
//! compare each recorded verdict with
//! [`catherine_ledger::harness::derive_probe_verdict`] over the entry's
//! `unique_detections` and baseline arm.
//!
//! Rationale: `ledger-check` validates the closed vocabulary but cannot
//! notice a verdict word that contradicts the arithmetic (ro5u C-02). The
//! derivation assertion makes that drift a test failure; the fixture-hash
//! pins keep the entry from citing bytes that no longer exist.

mod probe_support;

use std::path::Path;

use catherine_ledger::validate::validate_dir;

use probe_support::{
    assert_entry_cites_the_registration, assert_fixture_hash_is_current, fixture_hash, present,
    TestError, ROOT,
};

/// Both probe fixtures, path and pinned canonical hash.
const PINNED: [(&str, &str); 2] = [
    (
        "fixtures/probes/naturality/crm.ct.json",
        "sha256:9bb77131b3a800a6cb0b6d082aad4cec571358f7b821c85252774671a90c4d5b",
    ),
    (
        "fixtures/probes/naturality/warehouse_break.ct.json",
        "sha256:ca16f21500712bc3ccdd178bdad3ccdba362540987eb70279548495e7023972c",
    ),
];

/// Both probe fixtures are pinned: an edit to a committed probe fixture
/// must be a new ledger entry, never a silent change.
#[test]
fn naturality_fixture_hashes_are_pinned() -> Result<(), TestError> {
    for (path, pin) in PINNED {
        assert_eq!(fixture_hash(path)?, pin, "{path} drifted from its pin");
    }
    Ok(())
}

/// The probe ledger validates, the control went first and self-negated,
/// the wrong-question arm records the null result, and every recorded
/// verdict is the one the comparison arithmetic derives.
#[test]
fn naturality_probe_ledger_records_the_preregistered_evidence() -> Result<(), TestError> {
    let dir = Path::new(ROOT).join("ledger/probes/naturality");
    let valid = validate_dir(&dir).map_err(|violations| format!("{violations:?}"))?;
    assert_eq!(valid.entries.len(), 2, "control + wrong-question arm");

    let katas = valid.kata_entries();
    let control = present(katas[0].content.kata.as_ref(), "kata")?;
    assert!(control.identity.control, "the floor control is first");
    assert_eq!(
        control.derivation.verdict, "killed",
        "the control self-negated"
    );
    assert!(control.derivation.comparison.unique_detections.is_empty());

    let wrong_question = present(katas[1].content.kata.as_ref(), "kata")?;
    assert_eq!(wrong_question.identity.kata_id, "probe-naturality");
    assert!(
        !wrong_question.identity.control,
        "the second entry is the probe arm"
    );
    assert!(
        wrong_question
            .derivation
            .comparison
            .unique_detections
            .is_empty(),
        "the baseline arm detects the fault too: parity, no unique detections"
    );
    assert_eq!(
        wrong_question.derivation.verdict, "killed",
        "parity derives killed — the wrong-question gap is recorded, not a uniqueness claim"
    );
    assert!(
        !wrong_question.expressibility_gap.is_empty(),
        "the wrong-question gap is the recorded contribution"
    );

    let named_pair = PINNED
        .iter()
        .map(|(path, _)| fixture_hash(path))
        .collect::<Result<Vec<_>, TestError>>()?;
    for entry in &katas {
        assert_entry_cites_the_registration(entry, "probe-naturality")?;
        assert_fixture_hash_is_current(entry)?;
        assert_eq!(
            entry.content.model_hashes, named_pair,
            "the validated models are the fixture pair it names"
        );
    }
    Ok(())
}
