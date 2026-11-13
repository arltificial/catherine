//! Purpose: the recorded evidence of the Builder-as-monoid kata
//! (`catherine-lgl.2`) — the committed ledger is validated, its fixture
//! hashes recomputed from the canonical model bytes, the near-floor
//! control self-negated first, and the positive entries carry the
//! pre-registered unique arithmetic against the executed baseline arm.
//!
//! Responsibilities: pin every kata fixture's model hash (a silent
//! fixture edit breaks the entry's content address), validate
//! `ledger/katas/builder-monoid` through the real validator, pin the
//! baseline script's sha256, and assert the recorded unique counts and
//! executed baseline arms.
//!
//! Rationale: the entry is the science; these assertions keep it from
//! drifting away from the fixtures and the pre-registered fault matrix it
//! claims to have run against.

use std::path::Path;

use catherine_core::hash::{model_hash, sha256_digest};
use catherine_core::parse::parse;
use catherine_ledger::entry::{BaselineMode, Entry, KataRecord};
use catherine_ledger::validate::{validate_dir, ValidLedger};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const LEDGER: &str = "ledger/katas/builder-monoid";
const BASELINE: &str = "prototype/builder_monoid_baseline.py";

/// The committed fixtures and their pinned canonical model hashes.
const FIXTURES: [(&str, &str); 8] = [
    (
        "fixtures/katas/builder-monoid/builder.ct.json",
        "sha256:69bff02856d0ba03cf3f21f40f23ec224b9efd5288ce43d5914f50e95475505b",
    ),
    (
        "fixtures/katas/builder-monoid/faults/01-dropped-step.ct.json",
        "sha256:5014819d6f8f812dba637ab9d1daf908560a5c9d200feeab396c0fa32fa573bf",
    ),
    (
        "fixtures/katas/builder-monoid/faults/02-duplicated-step.ct.json",
        "sha256:26722db027f2415296e3b0abbdb0f596ef577ab94b90db8248e0ef48740c0b2e",
    ),
    (
        "fixtures/katas/builder-monoid/faults/03-out-of-order-step.ct.json",
        "sha256:3f440999bbf494723186fe56ac0c570595b956af03e13bd741e85e83d4d5dbb1",
    ),
    (
        "fixtures/katas/builder-monoid/faults/04-optional-part-dropped.ct.json",
        "sha256:48c2fb954b89f210e1e8b1011a2f11fb090af3e24a8ccb6ef11baaad7358155f",
    ),
    (
        "fixtures/katas/builder-monoid/floor/single-step-builder.ct.json",
        "sha256:2aee309b81b30988f883af08763d6f8c4d6596eeb8470e7ef8dc3056e3123959",
    ),
    (
        "fixtures/katas/builder-monoid/floor/faults/01-dropped-step.ct.json",
        "sha256:f3a41c94c9f32dfa51926096a384df8e96586e9b49ff76a8afaa3d25141ed9b2",
    ),
    (
        "fixtures/katas/builder-monoid/floor/faults/02-duplicated-step.ct.json",
        "sha256:20b6fbf46bac6a72d7d8e60786cedd1d5bc171f3ab30157b4ef2395c975b0473",
    ),
];

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// The baseline script's pinned identity (`--hash` prints the hex).
fn baseline_hash() -> String {
    sha256_digest(&std::fs::read(format!("{ROOT}/{BASELINE}")).expect("baseline readable"))
}

fn kata(entry: &Entry) -> &KataRecord {
    entry.content.kata.as_ref().expect("kata-shaped entry")
}

/// The fixtures' identities are pinned: an edit to any committed fixture
/// must be a new ledger entry, never a silent change.
#[test]
fn fixture_hashes_are_pinned() {
    for (path, expected) in FIXTURES {
        assert_eq!(fixture_hash(path), expected, "{path} drifted");
    }
}

/// The baseline arm is executed, committed, and pinned by sha256.
#[test]
fn baseline_artifact_is_the_pinned_script() {
    assert_eq!(
        baseline_hash(),
        "sha256:9d7b1066b77e9feedef981a4e1711a1af691ad99958cedeea5c2ec0408ef9ff4"
    );
}

/// The ledger validates and opens with a self-negated near-floor control.
#[test]
fn ledger_starts_with_the_self_negated_control() {
    let valid = validate_dir(Path::new(ROOT).join(LEDGER).as_path())
        .expect("the builder-monoid ledger validates");
    assert_eq!(valid.entries.len(), 5, "control + four positives");
    assert_control(&valid);
}

/// The ledger's positives carry the pre-registered unique arithmetic and
/// executed baseline arms; zero unique is a legitimate killed outcome.
#[test]
fn positives_record_the_preregistered_unique_counts() {
    let valid = validate_dir(Path::new(ROOT).join(LEDGER).as_path())
        .expect("the builder-monoid ledger validates");
    assert_eq!(valid.kata_entries().len(), 5, "control + four positives");
    let expected = [
        ("dropped-step", 25usize),
        ("duplicated-step", 0),
        ("out-of-order-step", 1),
        ("optional-part-dropped", 1),
    ];
    for (entry, (name, unique)) in valid.kata_entries()[1..].iter().zip(expected) {
        assert_positive(entry, name, unique);
    }
}

/// The control went first, self-negated, and names the executed arm.
fn assert_control(valid: &ValidLedger) {
    let entry = valid.kata_entries()[0];
    let record = kata(entry);
    assert!(record.control, "the near-floor control is first");
    assert_eq!(record.kata_id, "builder-monoid");
    assert_eq!(record.verdict, "killed", "the control self-negated");
    assert!(record.comparison.unique_detections.is_empty());
    assert_eq!(record.fixture_hash, fixture_hash(&record.fixture));
    assert_eq!(
        entry.content.model_hashes.len(),
        3,
        "base + two floor faults"
    );
    let baseline = record
        .comparison
        .baseline
        .as_ref()
        .expect("the control records the executed arm");
    assert_eq!(baseline.mode, BaselineMode::Executed);
    assert_eq!(baseline.detections.len(), 2, "both floor faults caught");
}

/// One positive entry: identity, canonical element count, verdict, and
/// the executed baseline arm that made the arithmetic legitimate.
fn assert_positive(entry: &Entry, expected_id: &str, unique: usize) {
    let record = kata(entry);
    assert!(!record.control, "{expected_id} is a positive");
    assert_eq!(record.kata_id, "builder-monoid");
    assert_eq!(record.fixture_hash, fixture_hash(&record.fixture));
    assert_eq!(
        entry.content.model_hashes,
        vec![record.fixture_hash.clone()],
        "the validated model is the fixture it names"
    );
    assert_eq!(
        record.comparison.unique_detections.len(),
        unique,
        "{expected_id} unique count"
    );
    assert_eq!(
        record.verdict,
        if unique == 0 { "killed" } else { "survived" },
        "{expected_id} verdict follows the arithmetic"
    );
    let baseline = record
        .comparison
        .baseline
        .as_ref()
        .expect("a baseline arm is recorded");
    assert_eq!(baseline.mode, BaselineMode::Executed);
    assert!(
        baseline.artifact.contains(&baseline_hash()),
        "{expected_id} pins the executed script"
    );
}
