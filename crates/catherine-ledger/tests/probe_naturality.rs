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

use std::path::Path;

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::catalog::{probe_catalog_hash, probe_expectations};
use catherine_ledger::entry::Verdict;
use catherine_ledger::harness::derive_probe_verdict;
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// Both probe fixtures are pinned: an edit to a committed probe fixture
/// must be a new ledger entry, never a silent change.
#[test]
fn naturality_fixture_hashes_are_pinned() {
    assert_eq!(
        fixture_hash("fixtures/probes/naturality/crm.ct.json"),
        "sha256:9bb77131b3a800a6cb0b6d082aad4cec571358f7b821c85252774671a90c4d5b"
    );
    assert_eq!(
        fixture_hash("fixtures/probes/naturality/warehouse_break.ct.json"),
        "sha256:ca16f21500712bc3ccdd178bdad3ccdba362540987eb70279548495e7023972c"
    );
}

/// The probe ledger validates, the control went first and self-negated,
/// the wrong-question arm records the null result, and every recorded
/// verdict is the one the comparison arithmetic derives.
#[test]
fn naturality_probe_ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/probes/naturality");
    let valid = validate_dir(&dir).expect("the naturality probe ledger validates");
    assert_eq!(valid.entries.len(), 2, "control + wrong-question arm");

    let katas = valid.kata_entries();
    let control = katas[0].content.kata.as_ref().expect("kata");
    assert!(control.control, "the floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());

    let wrong_question = katas[1].content.kata.as_ref().expect("kata");
    assert_eq!(wrong_question.kata_id, "probe-naturality");
    assert!(!wrong_question.control, "the second entry is the probe arm");
    assert!(
        wrong_question.comparison.unique_detections.is_empty(),
        "the baseline arm detects the fault too: parity, no unique detections"
    );
    assert_eq!(
        wrong_question.verdict, "killed",
        "parity derives killed — the wrong-question gap is recorded, not a uniqueness claim"
    );
    assert!(
        !wrong_question.expressibility_gap.is_empty(),
        "the wrong-question gap is the recorded contribution"
    );

    for entry in &katas {
        let kata = entry.content.kata.as_ref().expect("kata");
        assert_eq!(kata.kata_id, "probe-naturality", "the registered probe id");
        assert_eq!(
            kata.expected,
            probe_expectations("probe-naturality"),
            "the embedded expectations are the pre-registered catalog"
        );
        assert_eq!(
            kata.catalog_hash.as_deref(),
            probe_catalog_hash("probe-naturality").as_deref(),
            "the entry cites the committed catalog"
        );
        assert_eq!(
            kata.fixture_hash,
            fixture_hash(&kata.fixture),
            "the recorded fixture hash is the canonical model hash"
        );
        assert_eq!(
            entry.content.model_hashes,
            vec![
                fixture_hash("fixtures/probes/naturality/crm.ct.json"),
                fixture_hash("fixtures/probes/naturality/warehouse_break.ct.json"),
            ],
            "the validated models are the fixture pair it names"
        );

        let derived = derive_probe_verdict(
            &kata.kata_id,
            &kata.comparison.unique_detections,
            kata.comparison.baseline.as_ref(),
        )
        .expect("a registered probe with a comparison derives a verdict");
        let recorded: Verdict = kata
            .verdict
            .parse()
            .expect("recorded verdict in vocabulary");
        assert_eq!(
            recorded, derived,
            "the recorded verdict for `{}` must equal the derivation from its own comparison",
            kata.name
        );
    }
}
