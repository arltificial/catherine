//! Purpose: the recorded evidence of the `visitor-sum-types` kata
//! (`catherine-lgl.3`) — a legacy Visitor/Interpreter reverse-engineered
//! into a rule table over a finite variant domain, compared against a
//! pre-registered sampled scripted dispatch test.
//!
//! Responsibilities: pin every committed fixture's canonical model hash,
//! validate the kata ledger through the real validator, assert the
//! control went first and self-negated, assert the positive entry's
//! three unique detections against its executed baseline arm, and
//! recompute each entry's `fixture_hash` and `model_hashes`.
//!
//! Rationale: an entry's content addresses are only evidence while the
//! fixtures and the baseline artifact still hash to what the entry
//! claims. A silent fixture or baseline edit must break this test, not
//! the ledger's arithmetic.

use std::path::{Path, PathBuf};

use catherine_core::hash::{model_hash, sha256_digest};
use catherine_core::parse::parse;
use catherine_ledger::entry::BaselineMode;
use catherine_ledger::validate::{validate_dir, ValidLedger};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

const BASELINE_ARTIFACT: &str = concat!(
    "prototype/visitor_sum_types_baseline.py ",
    "sha256:25c3ac8c2e9fe61cec1d93f22842e114ceae4522b2bb78c1d2f3b0ba43c985f0"
);

/// `(path, canonical model hash)` for every fixture either entry validated.
const FIXTURES: [(&str, &str); 10] = [
    (
        "fixtures/katas/visitor-sum-types/visitor.ct.json",
        "sha256:74a5692cc776827da0779fe875b92938127b8747fce8c6578830dd508f7c3ed1",
    ),
    (
        "fixtures/katas/visitor-sum-types/faults/01-gap-missing-num.ct.json",
        "sha256:5116fc7a776cc0098078165206dd4657b34c5405a9fc2baa8ba3536ea186a8b7",
    ),
    (
        "fixtures/katas/visitor-sum-types/faults/02-overlap-duplicate-add.ct.json",
        "sha256:cc6345e7d2d3ffc8a187716bc9bd248f04649aeb95460353b5e9dc576d150d02",
    ),
    (
        "fixtures/katas/visitor-sum-types/faults/03-shadow-literal-covers-str.ct.json",
        "sha256:c9cda4b92698197eb6aa65aa2f28595f6839457d9d476d7362f65b35efb2fe27",
    ),
    (
        "fixtures/katas/visitor-sum-types/faults/04-derivable-num.ct.json",
        "sha256:1701f463fd9c0e824fbc035d7fff9a715f62803029ce6676146e53e011baae61",
    ),
    (
        "fixtures/katas/visitor-sum-types/faults/05-missing-variant-div.ct.json",
        "sha256:adf5785f857381bf94623315b1f4e5eec6081c5f7315cab3f35219f26f4f3a31",
    ),
    (
        "fixtures/katas/visitor-sum-types/floor/two-variant.ct.json",
        "sha256:86255dcd07ef6a29dd15628fac5180e5328921fb3e16687091a6bff758bc298a",
    ),
    (
        "fixtures/katas/visitor-sum-types/floor/faults/01-gap-remove-lit.ct.json",
        "sha256:fc6a57ab00fcf64f692a309490c252a1f849196d400e5aebc7fc86125a41ff46",
    ),
    (
        "fixtures/katas/visitor-sum-types/floor/faults/02-overlap-duplicate-lit.ct.json",
        "sha256:7514ccbb97414b690ae2c844f98c135d9a1d98baf3b32d01e4ac17ee384445de",
    ),
    (
        "fixtures/katas/visitor-sum-types/floor/faults/03-shadow-widen-lit.ct.json",
        "sha256:54935beccdd5a7539d5a1d47baa14346f191880cc131569ec76067f23de1d88b",
    ),
];

/// The control entry's validated models: the floor table plus its three
/// faults, in recording order.
const CONTROL_HASHES: [&str; 4] = [FIXTURES[6].1, FIXTURES[7].1, FIXTURES[8].1, FIXTURES[9].1];

/// The positive entry's validated models: the clean table plus F1–F5.
const POSITIVE_HASHES: [&str; 6] = [
    FIXTURES[0].1,
    FIXTURES[1].1,
    FIXTURES[2].1,
    FIXTURES[3].1,
    FIXTURES[4].1,
    FIXTURES[5].1,
];

fn workspace_root() -> PathBuf {
    Path::new(ROOT).to_path_buf()
}

/// One fixture's canonical model hash, recomputed from disk.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(workspace_root().join(path)).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// The committed kata ledger, validated.
fn ledger() -> ValidLedger {
    validate_dir(&workspace_root().join("ledger/katas/visitor-sum-types"))
        .expect("the visitor-sum-types kata ledger validates")
}

/// An entry's model-hash list as borrowed strings.
fn hashes(entry: &catherine_ledger::entry::Entry) -> Vec<&str> {
    entry
        .content
        .model_hashes
        .iter()
        .map(String::as_str)
        .collect()
}

#[test]
fn visitor_fixture_hashes_are_pinned() {
    for (path, expected) in FIXTURES {
        assert_eq!(fixture_hash(path), expected, "{path} drifted");
    }
}

#[test]
fn baseline_artifact_hash_is_pinned() {
    let bytes = std::fs::read(workspace_root().join("prototype/visitor_sum_types_baseline.py"))
        .expect("baseline readable");
    let digest = sha256_digest(&bytes);
    assert_eq!(
        BASELINE_ARTIFACT,
        format!("prototype/visitor_sum_types_baseline.py {digest}")
    );
}

#[test]
fn control_goes_first_and_self_negates() {
    let valid = ledger();
    let katas = valid.kata_entries();
    assert_eq!(katas.len(), 2, "one control + one positive entry");
    let entry = katas[0];
    let control = entry.content.kata.as_ref().expect("control kata");
    assert!(control.control, "the first entry is the negative control");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(
        control.comparison.unique_detections.is_empty(),
        "the floor buys no unique detections"
    );
    assert_eq!(
        control.expected.len(),
        3,
        "three floor faults pre-registered"
    );
    assert_eq!(hashes(entry), CONTROL_HASHES);
    assert_eq!(control.fixture_hash, fixture_hash(&control.fixture));
    let arm = control
        .comparison
        .baseline
        .as_ref()
        .expect("the executed baseline arm");
    assert_eq!(arm.mode, BaselineMode::Executed);
    assert_eq!(arm.artifact, BASELINE_ARTIFACT);
}

#[test]
fn positive_carries_three_unique_detections_against_an_executed_arm() {
    let valid = ledger();
    let katas = valid.kata_entries();
    let entry = katas[1];
    let positive = entry.content.kata.as_ref().expect("positive kata");
    assert!(!positive.control);
    assert_eq!(positive.verdict, "survived");
    assert_eq!(positive.expected.len(), 5, "five faults pre-registered");
    assert_eq!(hashes(entry), POSITIVE_HASHES);
    assert_eq!(positive.fixture_hash, fixture_hash(&positive.fixture));
    let unique = &positive.comparison.unique_detections;
    assert_eq!(unique.len(), 3, "F3, F4, F5 against the sampled baseline");
    for (detection, fault) in unique.iter().zip(["F3 ", "F4 ", "F5 "]) {
        assert!(
            detection.starts_with(fault),
            "{detection:?} is not the {fault}element"
        );
    }
    let arm = positive
        .comparison
        .baseline
        .as_ref()
        .expect("uniqueness requires the executed arm");
    assert_eq!(arm.mode, BaselineMode::Executed);
    assert_eq!(arm.artifact, BASELINE_ARTIFACT);
    assert_eq!(
        arm.detections.len(),
        2,
        "F1 and F2 are the parity detections"
    );
}
