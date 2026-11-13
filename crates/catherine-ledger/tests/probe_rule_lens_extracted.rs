//! Purpose: the recorded evidence of probe 2 (`add-domain-probe-validation`
//! tasks 8-9, beads `catherine-1od`) — the extracted-table probe ledger
//! validates, its fixtures' canonical hashes are pinned, its embedded
//! expectations are the pre-registered catalog, and the unique set is
//! the one the sampled conventional review could not reach.
//!
//! Responsibilities: pin every probe fixture's model hash, validate the
//! ledger directory, and check the near-floor control self-negated while
//! the above-floor entry carries an executed baseline arm.
//!
//! Rationale: same discipline as probe 1 — the entry is evidence only
//! while the files on disk still hash to what it cites.

use std::path::Path;

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::catalog::{probe_catalog_hash, probe_expectations};
use catherine_ledger::validate::validate_dir;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

const FIXTURES: [(&str, &str); 9] = [
    (
        "fixtures/probes/rule_lens/extracted/cct-288-97.ct.json",
        "sha256:5acfe10d73c3e152a39a370b7dffbd738495e93788f4501473a4b7ba6d020e10",
    ),
    (
        "fixtures/probes/rule_lens/extracted/faults/01-gap-remove-general-2023-01.ct.json",
        "sha256:fc2872fb79e8ea1aee1d4c5b7d92bf019b0c29b00f8168e10476c208b3e4a215",
    ),
    (
        "fixtures/probes/rule_lens/extracted/faults/02-gap-remove-zona-2022-12.ct.json",
        "sha256:1fd1fc6ee2938ad020a7c56392f4775fab4b0f0c1976a4bf92dd8352ca806f7c",
    ),
    (
        "fixtures/probes/rule_lens/extracted/faults/03-overlap-declared-disjoint.ct.json",
        "sha256:071195f28a2513a0be324c46ad0d7be135439d9f3c0243087b57de074cf848cc",
    ),
    (
        "fixtures/probes/rule_lens/extracted/faults/04-shadow-duplicate-general-2022-10.ct.json",
        "sha256:ad42e3151a2c4a1843795e5b6454d6cfdda51e17aa0592cf9dc826f518a989b5",
    ),
    (
        "fixtures/probes/rule_lens/extracted/faults/05-shadow-widen-zona-2022-10.ct.json",
        "sha256:d738ba480d351f2867f53ecdb3c8c8f74e69a6f1533acb71d9bd88da7b4c9954",
    ),
    (
        "fixtures/probes/rule_lens/floor/cct-1492-15.ct.json",
        "sha256:86bb190e10ddc5300c7145396508694e34c126c35d1417458a6ecab85b94c39d",
    ),
    (
        "fixtures/probes/rule_lens/floor/faults/01-gap-remove-row.ct.json",
        "sha256:bc190b1b40baf34f03521a9193baf0c4a5c84d959d68e62cf2b36459d7fa73cf",
    ),
    (
        "fixtures/probes/rule_lens/floor/faults/02-overlap-different-cap.ct.json",
        "sha256:ff54051c0da223c17d84c81de2e065591dfce185f52bdb9e18ac1039302e877f",
    ),
];

/// One fixture's model hash, recomputed from its canonical bytes.
fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// Every extracted-table probe fixture is pinned: an edit must become a
/// superseding ledger entry.
#[test]
fn rule_probe_fixture_hashes_are_pinned() {
    let mut mismatches = Vec::new();
    for (path, pin) in FIXTURES {
        let got = fixture_hash(path);
        println!("HASH {path} {got}");
        if got != pin {
            mismatches.push(format!("{path}: {got} != {pin}"));
        }
    }
    assert!(mismatches.is_empty(), "fixture hash drift: {mismatches:?}");
}

/// The probe ledger validates with the control first, and the positive
/// entry carries a pre-registered catalog, an executed baseline arm, and
/// the derivable gap recorded rather than fabricated.
#[test]
fn rule_probe_ledger_records_the_preregistered_evidence() {
    let dir = Path::new(ROOT).join("ledger/probes/rule-lens-extracted");
    let valid = validate_dir(&dir).expect("the extracted-table probe ledger validates");
    assert_eq!(valid.entries.len(), 2, "control + above-floor probe");
    let control = valid.kata_entries()[0].content.kata.clone().expect("kata");
    assert!(control.control, "the near-floor control is first");
    assert_eq!(control.verdict, "killed", "the control self-negated");
    assert!(control.comparison.unique_detections.is_empty());

    let above = valid.kata_entries()[1].content.kata.clone().expect("kata");
    assert_eq!(above.kata_id, "probe-rule-lens-extracted");
    assert_eq!(
        above.expected,
        probe_expectations("probe-rule-lens-extracted"),
        "the embedded expectations are the pre-registered catalog"
    );
    assert_eq!(
        above.catalog_hash.as_deref(),
        probe_catalog_hash("probe-rule-lens-extracted").as_deref(),
        "the entry cites the committed catalog"
    );
    assert_eq!(
        above.comparison.unique_detections.len(),
        4,
        "one unsampled gap and three dead/widened shadow detections"
    );
    let baseline = above
        .comparison
        .baseline
        .as_ref()
        .expect("an executed baseline arm carries the unique set");
    assert_eq!(
        baseline.mode,
        catherine_ledger::entry::BaselineMode::Executed
    );
    assert!(
        above
            .expressibility_gap
            .iter()
            .any(|gap| gap.invariant.contains("derivable")),
        "the unreached derivable expectation is recorded as a gap"
    );
    assert_eq!(
        above.fixture_hash,
        fixture_hash(&above.fixture),
        "the recorded fixture hash is the canonical model hash"
    );
}
