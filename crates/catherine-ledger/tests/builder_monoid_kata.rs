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

mod support;

use std::path::Path;

use catherine_core::hash::{model_hash, sha256_digest};
use catherine_core::parse::parse;
use catherine_ledger::entry::{BaselineMode, Entry};
use catherine_ledger::validate::{validate_dir, ValidLedger};
use support::{present, TestError};

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

/// The committed kata ledger, validated; a refusal becomes the test's
/// error with the violations spelled out.
fn validated(dir: &Path) -> Result<ValidLedger, TestError> {
    validate_dir(dir)
        .map_err(|violations| format!("the builder-monoid ledger refused: {violations:?}").into())
}

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> Result<String, TestError> {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}"))?;
    let document = parse(&text)?;
    Ok(model_hash(&document))
}

/// The baseline script's pinned identity (`--hash` prints the hex).
fn baseline_hash() -> Result<String, TestError> {
    let bytes = std::fs::read(format!("{ROOT}/{BASELINE}"))?;
    Ok(sha256_digest(&bytes))
}

/// The positive entries in ledger order: the fault they carry and the
/// unique count the pre-registered arithmetic freezes.
const POSITIVES: [(&str, usize); 4] = [
    ("dropped-step", 25),
    ("duplicated-step", 0),
    ("out-of-order-step", 1),
    ("optional-part-dropped", 1),
];

/// The fixtures' identities are pinned: an edit to any committed fixture
/// must be a new ledger entry, never a silent change.
#[test]
fn fixture_hashes_are_pinned() -> Result<(), TestError> {
    for (path, pinned) in FIXTURES {
        assert_eq!(fixture_hash(path)?, pinned, "{path} drifted");
    }
    Ok(())
}

/// The baseline arm is executed, committed, and pinned by sha256.
#[test]
fn baseline_artifact_is_the_pinned_script() -> Result<(), TestError> {
    assert_eq!(
        baseline_hash()?,
        "sha256:9d7b1066b77e9feedef981a4e1711a1af691ad99958cedeea5c2ec0408ef9ff4"
    );
    Ok(())
}

/// The ledger validates and opens with a self-negated near-floor control.
#[test]
fn ledger_starts_with_the_self_negated_control() -> Result<(), TestError> {
    let valid = validated(Path::new(ROOT).join(LEDGER).as_path())?;
    assert_eq!(valid.entries.len(), 5, "control + four positives");
    assert_control(&valid)?;
    Ok(())
}

/// The ledger's positives carry the pre-registered unique arithmetic and
/// executed baseline arms; zero unique is a legitimate killed outcome.
#[test]
fn positives_record_the_preregistered_unique_counts() -> Result<(), TestError> {
    let valid = validated(Path::new(ROOT).join(LEDGER).as_path())?;
    assert_eq!(valid.kata_entries().len(), 5, "control + four positives");
    for (entry, (name, unique)) in valid.kata_entries()[1..].iter().zip(POSITIVES) {
        assert_positive(entry, PositiveSpec { name, unique })?;
    }
    Ok(())
}

/// One positive entry's identity pin and unique arithmetic.
struct PositiveSpec {
    name: &'static str,
    unique: usize,
}

/// The control went first, self-negated, and names the executed arm.
fn assert_control(valid: &ValidLedger) -> Result<(), TestError> {
    let entry = valid.kata_entries()[0];
    let record = present(entry.content.kata.as_ref(), "kata record")?;
    assert!(record.identity.control, "the near-floor control is first");
    assert_eq!(record.identity.kata_id, "builder-monoid");
    assert_eq!(
        record.derivation.verdict, "killed",
        "the control self-negated"
    );
    assert!(record.derivation.comparison.unique_detections.is_empty());
    assert_eq!(
        record.provenance.fixture_hash,
        fixture_hash(&record.provenance.fixture)?
    );
    assert_eq!(
        entry.content.model_hashes.len(),
        3,
        "base + two floor faults"
    );
    let baseline = present(
        record.derivation.comparison.baseline.as_ref(),
        "the control's executed arm",
    )?;
    assert_eq!(baseline.mode, BaselineMode::Executed);
    assert_eq!(baseline.detections.len(), 2, "both floor faults caught");
    Ok(())
}

/// One positive entry: identity, canonical element count, verdict, and
/// the executed baseline arm that made the arithmetic legitimate.
fn assert_positive(entry: &Entry, spec: PositiveSpec) -> Result<(), TestError> {
    let PositiveSpec { name, unique } = spec;
    let record = present(entry.content.kata.as_ref(), "kata record")?;
    assert!(!record.identity.control, "{name} is a positive");
    assert_eq!(record.identity.kata_id, "builder-monoid");
    assert_eq!(
        record.provenance.fixture_hash,
        fixture_hash(&record.provenance.fixture)?
    );
    assert_eq!(
        entry.content.model_hashes,
        vec![record.provenance.fixture_hash.clone()],
        "the validated model is the fixture it names"
    );
    assert_eq!(
        record.derivation.comparison.unique_detections.len(),
        unique,
        "{name} unique count"
    );
    assert_eq!(
        record.derivation.verdict,
        if unique == 0 { "killed" } else { "survived" },
        "{name} verdict follows the arithmetic"
    );
    let baseline = present(
        record.derivation.comparison.baseline.as_ref(),
        "a baseline arm is recorded",
    )?;
    assert_eq!(baseline.mode, BaselineMode::Executed);
    assert!(
        baseline.artifact.contains(&baseline_hash()?),
        "{name} pins the executed script"
    );
    Ok(())
}
