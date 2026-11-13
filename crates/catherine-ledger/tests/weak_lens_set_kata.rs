//! Purpose: the recorded evidence of the weak-lens-set kata
//! (`catherine-lgl.8`) — the committed slice ledger validates, its
//! fixture hashes are pinned, and the per-pattern dispositions are the
//! pre-registered ones: four weak lenses at parity, one unexpressible
//! lens at a null result, and a control that self-negates.
//!
//! Responsibilities: pin every clean/fault fixture's canonical model
//! hash, execute the kernel over the fixtures (clean = zero findings;
//! faults = the pre-registered counts), validate the ledger directory
//! through the real validator, and assert the control-first/killed/
//! empty-unique shape plus each positive's executed baseline arm.
//!
//! Rationale: same discipline as the probe ledgers — an entry is
//! evidence only while the files on disk still hash to what it cites,
//! and a null result (Flyweight) is recorded as a null, never rounded
//! up to a detection.

use std::path::Path;

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::entry::BaselineMode;
use catherine_ledger::harness::run;
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

const FIXTURES: [(&str, &str); 12] = [
    (
        "control-floor.ct.json",
        "sha256:7e048a8daeba1ff8fb0a272656157ab2a7038122f8ca5f7a38eb36e333f57063",
    ),
    (
        "control-floor-fault-stale-seen.ct.json",
        "sha256:ea80cf66a68def59528b08c7691a56287eb2e75eeea1262ebf77db0380372ccd",
    ),
    (
        "observer.ct.json",
        "sha256:4ab76b6543bee8699dd14bdcba9fb1fd444da884372b02b67a55c4b0ff4efe89",
    ),
    (
        "observer-fault-stale-delivery.ct.json",
        "sha256:2fb4a50bf7349f4d6f190587af91687d692b7f9e67f90dae2fdfda15738b592c",
    ),
    (
        "bridge.ct.json",
        "sha256:89de11623a2141ef1f27b8c8ff88becff977c34285004f683499f197dfbde300",
    ),
    (
        "bridge-fault-divergent-op.ct.json",
        "sha256:9f8d5916adb35f7a8d4b00d7a51f8e70b33fcc5539e5daa8dcda3ec01fa5033e",
    ),
    (
        "flyweight.ct.json",
        "sha256:8037c006e53d760246e537e62eb7d926d25c8090c3e1be50f18398a1b55c0223",
    ),
    (
        "flyweight-fault-duplicate-instance.ct.json",
        "sha256:62cc58cfda3499a061a7bbe48cee6f5520b8ec313c967fb9272d8cfe78729cb3",
    ),
    (
        "command.ct.json",
        "sha256:096074b79fdd8b2c95082f11c8bbfc0ff70a8977c8e5e7baa349ed3ac8458bb6",
    ),
    (
        "command-fault-undo-drift.ct.json",
        "sha256:68bd82732378eef98d62a497eed3b8774ff86f188871b8a75be5dbee2049dd89",
    ),
    (
        "prototype.ct.json",
        "sha256:9042b1d4d32dc45e614e83f6d36521ba3ad1d7432b139d77efe6e7bb28334ebd",
    ),
    (
        "prototype-fault-clone-drift.ct.json",
        "sha256:a4d2b06954ff8ad4504769c43bb968345330b303ac5474ee224479e6adfc81a6",
    ),
];

/// The positives in ledger order: fault id, fixture, element, findings.
const POSITIVES: [(&str, &str, &str, usize); 5] = [
    (
        "observer-stale-delivery",
        "observer-fault-stale-delivery.ct.json",
        "observer:seen!=subject.event@Observer#1",
        1,
    ),
    (
        "bridge-divergent-op",
        "bridge-fault-divergent-op.ct.json",
        "bridge:op!=impl_op.impl@Abstraction#1",
        1,
    ),
    (
        "flyweight-duplicate-instance",
        "flyweight-fault-duplicate-instance.ct.json",
        "flyweight:duplicate-instance@Client#1",
        0,
    ),
    (
        "command-undo-drift",
        "command-fault-undo-drift.ct.json",
        "command:after.undo!=before@Command#1",
        1,
    ),
    (
        "prototype-clone-drift",
        "prototype-fault-clone-drift.ct.json",
        "prototype:clone.intrinsic!=source.intrinsic@Copy#0",
        1,
    ),
];

/// One fixture's canonical model hash, recomputed from its bytes.
fn fixture_hash(name: &str) -> String {
    let path = format!("{ROOT}/fixtures/katas/weak-lens-set/{name}");
    let text = std::fs::read_to_string(path).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// An edit to any committed fixture must become a superseding entry.
#[test]
fn weak_lens_fixture_hashes_are_pinned() {
    let mut mismatches = Vec::new();
    for (name, pin) in FIXTURES {
        let got = fixture_hash(name);
        println!("HASH {name} {got}");
        if got != pin {
            mismatches.push(format!("{name}: {got} != {pin}"));
        }
    }
    assert!(mismatches.is_empty(), "fixture hash drift: {mismatches:?}");
}

/// The clean fixtures satisfy their lens (zero findings); the seeded
/// faults emit exactly the pre-registered counts, including Flyweight's
/// zero.
#[test]
fn weak_lens_fixtures_execute_to_the_preregistered_counts() {
    for (name, _) in FIXTURES {
        let path = Path::new(ROOT)
            .join("fixtures/katas/weak-lens-set")
            .join(name);
        let executed = run(&path);
        assert!(executed.parsed, "{name} parses");
        let expected = expected_count(name);
        assert_eq!(
            executed.findings.len(),
            expected,
            "{name} emits {expected} findings"
        );
    }
}

/// The pre-registered finding count per fixture (the null included).
fn expected_count(name: &str) -> usize {
    match name {
        "control-floor.ct.json"
        | "observer.ct.json"
        | "bridge.ct.json"
        | "flyweight.ct.json"
        | "command.ct.json"
        | "prototype.ct.json"
        | "flyweight-fault-duplicate-instance.ct.json" => 0,
        "control-floor-fault-stale-seen.ct.json"
        | "observer-fault-stale-delivery.ct.json"
        | "bridge-fault-divergent-op.ct.json"
        | "command-fault-undo-drift.ct.json"
        | "prototype-fault-clone-drift.ct.json" => 1,
        other => panic!("unregistered fixture {other}"),
    }
}

/// The ledger validates control-first, the control self-negates, and
/// each positive carries its executed baseline arm and honest gap.
#[test]
fn weak_lens_ledger_records_the_preregistered_dispositions() {
    let dir = Path::new(ROOT).join("ledger/katas/weak-lens-set");
    let valid = validate_dir(&dir).expect("the weak-lens ledger validates");
    assert_eq!(valid.entries.len(), 6, "control + five patterns");
    let control = valid.kata_entries()[0].content.kata.clone().expect("kata");
    assert!(control.control, "the near-floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());
    assert_eq!(control.expected[0].fault, "floor-stale-seen");
    assert!(control.catalog_hash.is_none(), "non-probe kata, no catalog");
    assert_eq!(control.run.findings.len(), 1);
    assert_eq!(
        control
            .comparison
            .baseline
            .as_ref()
            .expect("arm")
            .detections,
        vec!["observer-control:seen!=subject.event@Observer#0"]
    );
    for (entry, (fault, fixture, element, count)) in valid.kata_entries()[1..].iter().zip(POSITIVES)
    {
        let kata = entry.content.kata.as_ref().expect("kata");
        assert_eq!(kata.kata_id, "weak-lens-set");
        assert_eq!(kata.verdict, "killed");
        assert!(kata.comparison.unique_detections.is_empty());
        assert_eq!(kata.expected[0].fault, fault);
        assert!(kata.catalog_hash.is_none(), "{fault} has no catalog");
        assert_eq!(kata.run.findings.len(), count, "{fault} finding count");
        assert_eq!(kata.comparison.fired.len(), count, "{fault} fired count");
        assert!(!kata.expressibility_gap.is_empty(), "{fault} records a gap");
        let baseline = kata.comparison.baseline.as_ref().expect("executed arm");
        assert_eq!(baseline.mode, BaselineMode::Executed);
        assert_eq!(baseline.detections, vec![element.to_string()]);
        assert_eq!(
            kata.fixture,
            format!("fixtures/katas/weak-lens-set/{fixture}")
        );
        assert_eq!(entry.content.model_hashes, vec![kata.fixture_hash.clone()]);
        assert_eq!(kata.fixture_hash, fixture_hash(fixture));
    }
}
