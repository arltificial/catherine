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

use std::path::Path;

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::catalog::{probe_catalog_hash, probe_expectations};
use catherine_ledger::entry::Verdict;
use catherine_ledger::harness::derive_probe_verdict;
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The four documents the seam arm validated: the committed hand-merge and
/// the three gap-suite inputs it unions, in the entry's recorded order.
const SEAM_MODELS: &[&str] = &[
    "fixtures/probes/composition/hand_merged_seam.ct.json",
    "crates/catherine-cli/tests/fixtures/gap_suite/composition_billing.ct.json",
    "crates/catherine-cli/tests/fixtures/gap_suite/composition_shipping.ct.json",
    "crates/catherine-cli/tests/fixtures/gap_suite/composition_seam_claim.ct.json",
];

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// Both probe fixtures are pinned: an edit to a committed probe fixture
/// must be a new ledger entry, never a silent change.
#[test]
fn composition_fixture_hashes_are_pinned() {
    assert_eq!(
        fixture_hash("fixtures/probes/composition/floor_dangling.ct.json"),
        "sha256:4e251b9ff3f7a7d3169b29a8990a806c2f8f990dbaeff7d784d848194c8350cf"
    );
    assert_eq!(
        fixture_hash("fixtures/probes/composition/hand_merged_seam.ct.json"),
        "sha256:579d5b0db32c01cefabe9fea30b3725b08269e3fa6f852dfa84a4759c25885d5"
    );
}

/// The control went first, self-negated, and its recorded fixture is the
/// single document the two arms each executed.
#[test]
fn composition_control_arm_self_negates() {
    let dir = Path::new(ROOT).join("ledger/probes/composition");
    let valid = validate_dir(&dir).expect("the composition probe ledger validates");
    assert_eq!(valid.entries.len(), 2, "control + seam arm");
    let katas = valid.kata_entries();
    let control = katas[0].content.kata.as_ref().expect("kata");
    assert!(control.control, "the floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(
        control.comparison.unique_detections.is_empty(),
        "the single-document check already reports the dangling cell"
    );
    assert_eq!(
        control.fixture, "fixtures/probes/composition/floor_dangling.ct.json",
        "the control runs the floor fixture"
    );
    assert_eq!(
        katas[0].content.model_hashes,
        vec![fixture_hash(&control.fixture)],
        "the one validated model is the fixture it names"
    );
}

/// The probe arm records the pre-registered expectations and names its
/// eliminated-merge-step claim as a gap, never as a unique detection; every
/// recorded verdict is the one the comparison arithmetic derives.
#[test]
fn composition_probe_ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/probes/composition");
    let valid = validate_dir(&dir).expect("the composition probe ledger validates");
    let katas = valid.kata_entries();
    let seam = katas[1].content.kata.as_ref().expect("kata");
    assert!(!seam.control, "the second entry is the probe arm");
    assert_eq!(seam.kata_id, "probe-composition", "the registered probe id");
    assert!(
        seam.comparison.unique_detections.is_empty(),
        "the hand-merged baseline detects the same row: parity, no unique detections"
    );
    assert_eq!(
        seam.verdict, "killed",
        "parity derives killed — the merge-step elimination is a gap, not a detection"
    );
    assert!(
        !seam.expressibility_gap.is_empty(),
        "the eliminated hand-merge step is the recorded contribution"
    );
    assert_eq!(
        katas[1].content.model_hashes,
        SEAM_MODELS
            .iter()
            .map(|path| fixture_hash(path))
            .collect::<Vec<_>>(),
        "the validated models are the hand-merge plus the three seam inputs"
    );
    for entry in &katas {
        let kata = entry.content.kata.as_ref().expect("kata");
        assert_eq!(kata.kata_id, "probe-composition", "the registered probe id");
        assert_eq!(
            kata.expected,
            probe_expectations("probe-composition"),
            "the embedded expectations are the pre-registered catalog"
        );
        assert_eq!(
            kata.catalog_hash.as_deref(),
            probe_catalog_hash("probe-composition").as_deref(),
            "the entry cites the committed catalog"
        );
        assert_eq!(
            kata.fixture_hash,
            fixture_hash(&kata.fixture),
            "the recorded fixture hash is the canonical model hash"
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
