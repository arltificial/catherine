//! Purpose: the recorded evidence of the state-discovery pattern kata
//! (`catherine-lgl.4`, P2) — the committed state-machine fixtures are
//! pinned by canonical model hash, their pre-registered fault matrix is
//! frozen, and the ledger's executed findings, discovery questions, and
//! null reachability result are asserted.
//!
//! Responsibilities: pin every parseable fixture's model hash and the
//! construction probe's raw bytes, pin the conventional baseline script,
//! validate the ledger directory through the real validator, and check
//! control-first/self-negation, the executed baseline arms, the
//! hypothesized labels, and every unique-detection set.
//!
//! Rationale: the kata's claim is arithmetic (executed probe elements
//! minus baseline-identified elements), so the fixtures and the baseline
//! artifact must be content-addressed here; a silent edit breaks the pin
//! and a correction must be a superseding ledger entry. Reachability is
//! recorded as a gap and a killed entry, never upgraded to a finding.

use std::path::Path;

use catherine_core::hash::{model_hash, sha256_digest};
use catherine_core::parse::parse;
use catherine_ledger::entry::{BaselineMode, KataRecord};
use catherine_ledger::validate::{validate_dir, ValidLedger};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const KATA: &str = "fixtures/katas/state-discovery";
const BASELINE_HASH: &str =
    "sha256:7d98b54bb65ea4e4c00962fc1d0e3e57ce19acb134f5d767b1177badd11fec3e";

/// One fixture's canonical model hash, recomputed from its bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// One fixture's raw bytes hash (for the construction probe, which never
/// reaches a model).
fn raw_hash(path: &str) -> String {
    let bytes = std::fs::read(format!("{ROOT}/{path}")).expect("fixture readable");
    sha256_digest(&bytes)
}

/// The `index`-th effective kata record.
fn kata_at(valid: &ValidLedger, index: usize) -> &KataRecord {
    valid.kata_entries()[index]
        .content
        .kata
        .as_ref()
        .expect("kata record")
}

/// The fixture hashes are pinned: an edit to a committed fixture must be
/// a new ledger entry, never a silent change.
#[test]
fn state_discovery_fixture_hashes_are_pinned() {
    let pins = [
        (
            "near-floor.ct.json",
            "sha256:ab338206e4bdf7d470a038f03bdd73afc8d26001161e067215dbefa8db7046cf",
        ),
        (
            "near-floor-fault-retarget.ct.json",
            "sha256:76780e55de535b7853f56a5dca464d81519df66dea710f3206220d7614d1546e",
        ),
        (
            "near-floor-fault-missing-exit.ct.json",
            "sha256:45afdfb2318ee9d10333ff7514fd69f0ef0ec0c37505ccfd5fb5f570e65c86e7",
        ),
        (
            "dangling-target.ct.json",
            "sha256:bebfc54c8b59d7b763ee7cbbf6c2cb182f616303453c676dc57946602ac3f142",
        ),
        (
            "missing-exit.ct.json",
            "sha256:cb6498d7e1fb5bf29c36d64c7d907985816e0bb6a6c1447c9d24e95a6be9f51e",
        ),
        (
            "drift-target.ct.json",
            "sha256:371c5c531f6fb01e2289ff8f4ce595e5e4f32fbe67ea87a398e61d7d2a3185a8",
        ),
        (
            "unreachable.ct.json",
            "sha256:59802f25dc2713a9405b14666983fd03d2c63b80348c81bed2f4980139f2e2e4",
        ),
    ];
    for (name, pin) in pins {
        assert_eq!(fixture_hash(&format!("{KATA}/{name}")), pin, "{name}");
    }
    let probe = format!("{KATA}/unknown-target.ct.json");
    assert_eq!(
        raw_hash(&probe),
        "sha256:06957f4c28a400dc84496f8b40f0c440b20f52228bbfdffc244cc2cb0672b064",
        "the construction probe's bytes are pinned"
    );
    let text = std::fs::read_to_string(format!("{ROOT}/{probe}")).expect("probe readable");
    assert!(
        parse(&text).is_err(),
        "the probe must never parse as a model"
    );
}

/// The conventional baseline arm is pinned: the ledger's uniqueness
/// claims are arithmetic against exactly this artifact.
#[test]
fn state_discovery_baseline_artifact_is_pinned() {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../prototype/state_discovery_baseline.py"
    ));
    assert_eq!(sha256_digest(bytes), BASELINE_HASH);
}

/// The ledger validates, the control went first and self-negated, the
/// positives carry executed baseline arms, and the discovery questions
/// stay hypothesized.
#[test]
fn state_discovery_ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/katas/state-discovery");
    let valid = validate_dir(&dir).expect("the kata ledger validates");
    assert_eq!(valid.entries.len(), 5, "control + four recorded probes");

    let control = kata_at(&valid, 0);
    assert!(control.control, "the near-floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());
    assert_eq!(control.run.findings.len(), 2, "two injected control faults");

    assert_positive(&valid, 1, "cell:Paused/stop", "survived");
    assert_positive(&valid, 2, "cell:Stopped/stop", "survived");
    assert_positive(&valid, 3, "cell:Playing/pause", "survived");
    assert_positive(&valid, 4, "", "killed");
    assert_fixture_identity(&valid);
    assert_discovery_questions(&valid);
}

/// One positive entry's unique set and verdict.
fn assert_positive(valid: &ValidLedger, index: usize, unique: &str, verdict: &str) {
    let kata = kata_at(valid, index);
    assert!(!kata.control, "entry {} is a positive", index + 1);
    assert_eq!(kata.verdict, verdict, "{}", kata.name);
    let expected: Vec<String> = if unique.is_empty() {
        Vec::new()
    } else {
        vec![unique.to_string()]
    };
    assert_eq!(kata.comparison.unique_detections, expected, "{}", kata.name);
    let baseline = kata.comparison.baseline.as_ref().expect("baseline arm");
    assert_eq!(baseline.mode, BaselineMode::Executed);
    assert!(
        baseline.artifact.contains(BASELINE_HASH),
        "the executed arm is the pinned script"
    );
}

/// Every entry's recorded fixture hash recomputes, and its validated
/// model hashes are the fixtures it names.
fn assert_fixture_identity(valid: &ValidLedger) {
    for entry in valid.kata_entries() {
        let kata = entry.content.kata.as_ref().expect("kata");
        let recomputed = fixture_hash(&kata.fixture);
        assert_eq!(kata.fixture_hash, recomputed, "{}", kata.fixture);
        assert!(
            entry.content.model_hashes.contains(&recomputed),
            "{} validated its named fixture",
            kata.name
        );
    }
    assert_eq!(
        valid.kata_entries()[1].content.model_hashes,
        vec![fixture_hash(&format!("{KATA}/dangling-target.ct.json"))]
    );
}

/// The discovery questions are recorded with the hypothesized label and
/// never counted as detections.
fn assert_discovery_questions(valid: &ValidLedger) {
    for index in [2, 4] {
        let kata = kata_at(valid, index);
        let labels: Vec<&str> = kata
            .run
            .findings
            .iter()
            .filter(|finding| finding["kind"] == "underconstraint")
            .map(|finding| finding["label"].as_str().expect("label"))
            .collect();
        assert_eq!(
            labels,
            vec!["hypothesized", "hypothesized"],
            "{}",
            kata.name
        );
        assert!(
            kata.comparison
                .unique_detections
                .iter()
                .all(|unique| !unique.contains("underconstraint")),
            "a question is not a detection"
        );
    }
    let probe_entry = kata_at(valid, 3);
    assert_eq!(
        probe_entry.run.probes.len(),
        1,
        "the parse rejection is recorded"
    );
    assert!(probe_entry.run.probes[0].observed.contains("syntax"));
    assert!(
        probe_entry.run.probes[0].document["instance"]["tables"]["to"]["t_playing_stop"] == "Ghost"
    );
    let reachability = kata_at(valid, 4);
    assert!(reachability.comparison.unique_detections.is_empty());
    assert!(
        !reachability.expressibility_gap.is_empty(),
        "the gap is named"
    );
}
