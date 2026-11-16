//! Purpose: the recorded evidence of `probe-coproduct`
//! (`add-coproduct-exhaustiveness`, ro5u CORR-003) — the committed
//! coproduct probe ledger validates, its superseded control chain is
//! intact, its fixture hashes recompute from the canonical model bytes,
//! its expectations are exactly the pre-registered catalog, and every
//! recorded verdict equals the arithmetic derivation from the entry's own
//! comparison data.
//!
//! Responsibilities: validate the ledger directory through the real
//! validator, pin the four referenced fixtures' model hashes, check that
//! seq 2 supersedes the RED control at seq 1 and that the effective
//! control self-negates, pin the control's executed model list, then bind
//! every effective entry — control and surviving probe arm — to its
//! canonical fixture hash, catalog address, pre-registered rows, and
//! arithmetic-derived verdict; the probe arm's two unique detections are
//! named explicitly.
//!
//! Rationale: `ledger-check` validates the closed vocabulary but cannot
//! notice a verdict word that contradicts the arithmetic, and
//! supersession is the ledger's correction path — the chain must be
//! pinned, not assumed. A fixture edit must become a new ledger entry,
//! never a silent change.

mod probe_support;

use std::path::Path;

use catherine_ledger::validate::validate_dir;

use probe_support::{
    assert_entry_cites_the_registration, assert_fixture_hash_is_current, fixture_hash, ROOT,
};

/// The four documents the effective entries reference, in the probe
/// entry's recorded order: the three payment arms, then the floor
/// control the control arm executed.
const PROBE_MODELS: [&str; 4] = [
    "fixtures/probes/coproduct/payments-three-variants.ct.json",
    "fixtures/probes/coproduct/payments-rogue-voucher.ct.json",
    "fixtures/probes/coproduct/payments-dual-covered.ct.json",
    "fixtures/probes/coproduct/floor-keys-caught.ct.json",
];

/// Every referenced fixture is pinned: an edit to a committed probe
/// fixture must be a new ledger entry, never a silent change.
#[test]
fn coproduct_fixture_hashes_are_pinned() {
    assert_eq!(
        fixture_hash("fixtures/probes/coproduct/floor-keys-caught.ct.json"),
        "sha256:0517667d8f596dcf4f7fd0c912e4787b36377df6add2c53ea4fe859b2ef3032a"
    );
    assert_eq!(
        fixture_hash("fixtures/probes/coproduct/payments-three-variants.ct.json"),
        "sha256:ede094298d1041ca8c13c6b8d9786086b9616f6d4a848b006e3dbf525fe64593"
    );
    assert_eq!(
        fixture_hash("fixtures/probes/coproduct/payments-rogue-voucher.ct.json"),
        "sha256:999d9c5e60070197f52b2b34ceda66187ebdf2ba560c661333fcc31ceadc14b0"
    );
    assert_eq!(
        fixture_hash("fixtures/probes/coproduct/payments-dual-covered.ct.json"),
        "sha256:c953da599e293a516e1785a213242304d9ac6fdbafd27fff6917cf9a19f2677a"
    );
}

/// The RED control went first, seq 2 supersedes it by id, and the
/// effective control self-negates against the registered catalog.
#[test]
fn coproduct_control_chain_supersedes_and_self_negates() {
    let dir = Path::new(ROOT).join("ledger/probes/coproduct");
    let valid = validate_dir(&dir).expect("the coproduct probe ledger validates");
    assert_eq!(
        valid.entries.len(),
        3,
        "RED control + correction + probe arm"
    );

    let red = valid.entries[0].content.kata.as_ref().expect("kata");
    assert_eq!(valid.entries[0].content.seq, 1, "the control went first");
    assert!(red.control, "seq 1 is the floor control");
    assert_eq!(red.verdict, "killed", "the RED control self-negated too");
    assert_eq!(
        valid.entries[1].content.supersedes.as_deref(),
        Some(valid.entries[0].id.as_str()),
        "seq 2 supersedes seq 1 by id — the ledger's correction path, never an edit"
    );

    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "the superseded control is not effective");
    let control = katas[0].content.kata.as_ref().expect("kata");
    assert!(control.control, "the effective floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(
        control.comparison.unique_detections.is_empty(),
        "the incumbent `schema.keys` check already names the collapsed pair"
    );
    assert_eq!(
        control.fixture, "fixtures/probes/coproduct/floor-keys-caught.ct.json",
        "the control runs the floor fixture"
    );
    assert_eq!(
        katas[0].content.model_hashes,
        vec![fixture_hash(&control.fixture)],
        "the one validated model is the fixture it names"
    );
}

/// The surviving probe arm records two unique detections against the
/// executed conventional arm, cites the registered catalog, and every
/// recorded verdict is the one the comparison arithmetic derives.
#[test]
fn coproduct_probe_ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/probes/coproduct");
    let valid = validate_dir(&dir).expect("the coproduct probe ledger validates");
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "control + probe arm");

    let probe = katas[1].content.kata.as_ref().expect("kata");
    assert!(
        !probe.control,
        "the second effective entry is the probe arm"
    );
    assert_eq!(probe.kata_id, "probe-coproduct", "the registered probe id");
    assert_eq!(
        probe.comparison.unique_detections,
        vec![
            "coproduct:Method@3:uncovered".to_string(),
            "coproduct:Method@2:dual(card_inj,transfer_inj)".to_string(),
        ],
        "the rogue row and the dual-covered row are the unique detections"
    );
    assert_eq!(
        probe.verdict, "survived",
        "two unique detections over an executed baseline derive survived"
    );
    assert_eq!(
        katas[1].content.model_hashes,
        PROBE_MODELS
            .iter()
            .map(|path| fixture_hash(path))
            .collect::<Vec<_>>(),
        "the validated models are the three payment arms plus the floor control"
    );
    for entry in &katas {
        assert_entry_cites_the_registration(entry, "probe-coproduct");
        assert_fixture_hash_is_current(entry);
    }
}
