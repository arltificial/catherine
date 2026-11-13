//! Purpose: the recorded evidence of probe 1 (`add-domain-probe-validation`
//! task 7, beads `catherine-1od`) — the committed migration probe ledger
//! is validated, its fixture hashes recomputed from the canonical model
//! bytes, its expectations are exactly the pre-registered catalog, and
//! its unique set is the arithmetic the baseline arm was executed for.
//!
//! Responsibilities: pin each probe fixture's model hash (a silent
//! fixture edit breaks the entry's content address), validate the ledger
//! directory through the real validator, and check the control's
//! self-negation plus the positives' baseline arms.
//!
//! Rationale: the entry is the science; these assertions keep it from
//! drifting away from the fixtures and the catalog it claims to have run
//! against.

use std::path::Path;

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::catalog::{probe_catalog_hash, probe_expectations};
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// The fixtures' identities are pinned: an edit to any committed probe
/// fixture must be a new ledger entry, never a silent change.
#[test]
fn probe_fixture_hashes_are_pinned() {
    assert_eq!(
        fixture_hash("fixtures/probes/migration/floor-visible-loss.ct.json"),
        "sha256:66df6081bc0a725a4e7787e47516048aa14a21d05a1c8fc9dbfa01432ebc930e"
    );
    assert_eq!(
        fixture_hash("fixtures/probes/migration/lost-data-cells.ct.json"),
        "sha256:b0f536e372ffc88bb4eeacba05f2addba0540b26eeb3aae3fa62dfda1111bee2"
    );
    assert_eq!(
        fixture_hash("fixtures/probes/migration/non-invertible-collapse.ct.json"),
        "sha256:39560fd67a3a506cb19d5929c4b237f491edfaba40cbbc9e2ee8bb90df672a23"
    );
}

/// The probe ledger validates, the control went first and self-negated,
/// and the positives carry executed baseline arms with unique sets.
#[test]
fn probe_ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/probes/migration");
    let valid = validate_dir(&dir).expect("the probe ledger validates");
    assert_eq!(valid.entries.len(), 3, "control + two inaugural probes");
    let control = &valid.kata_entries()[0].content.kata.clone().expect("kata");
    assert!(control.control, "the floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());
    for entry in &valid.kata_entries()[1..] {
        let kata = entry.content.kata.as_ref().expect("kata");
        assert_eq!(kata.kata_id, "probe-migration", "the registered probe id");
        assert!(
            !kata.comparison.unique_detections.is_empty(),
            "{} buys unique detections",
            kata.name
        );
        let baseline = kata
            .comparison
            .baseline
            .as_ref()
            .expect("an executed baseline arm carries the unique set");
        assert_eq!(
            baseline.mode,
            catherine_ledger::entry::BaselineMode::Executed
        );
        assert_eq!(
            kata.expected,
            probe_expectations("probe-migration"),
            "the embedded expectations are the pre-registered catalog"
        );
        assert_eq!(
            kata.catalog_hash.as_deref(),
            probe_catalog_hash("probe-migration").as_deref(),
            "the entry cites the committed catalog"
        );
        assert_eq!(
            entry.content.model_hashes,
            vec![kata.fixture_hash.clone()],
            "the validated model is the fixture it names"
        );
        assert_eq!(
            kata.fixture_hash,
            fixture_hash(&kata.fixture),
            "the recorded fixture hash is the canonical model hash"
        );
    }
}
