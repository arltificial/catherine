//! Purpose: the shared plumbing of the committed-probe evidence tests —
//! the repository-root anchor, the canonical fixture-hash recomputation,
//! and the one assertion that binds an effective probe entry to its
//! pre-registered catalog, its recorded fixture, and the verdict the
//! comparison arithmetic derives (ro5u EXCL-003).
//!
//! Responsibilities: expose [`ROOT`], [`fixture_hash`],
//! [`assert_entry_cites_the_registration`], and
//! [`assert_fixture_hash_is_current`]. Each probe binary keeps its own
//! ledger-shape assertions (control ordering, arm shape, the recorded
//! model list) local.
//!
//! Rationale: `probe-naturality`, `probe-composition`, and
//! `probe-coproduct` re-derived the same fixtures and re-asserted the
//! same catalog/derivation invariants; one copy keeps the evidence tests
//! from drifting apart. The binary-specific difference — the exact
//! validated-model list an entry names — stays in each test, because it
//! is that entry's own evidence, not plumbing.

#![allow(dead_code)]

use catherine_core::hash::model_hash;
use catherine_core::parse::parse;
use catherine_ledger::catalog::{probe_catalog_hash, probe_expectations};
use catherine_ledger::entry::{Entry, Verdict};
use catherine_ledger::harness::derive_probe_verdict;

/// The repository root, anchored at the crate manifest directory.
pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// One fixture's model hash, recomputed from its canonical bytes.
pub fn fixture_hash(path: &str) -> String {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}")).expect("fixture readable");
    model_hash(&parse(&text).expect("fixture parses"))
}

/// Every claim-bearing pin of one effective probe entry: the registered
/// id, the pre-registered rows, the committed catalog address, and the
/// verdict the comparison arithmetic derives from the entry's own
/// comparison data.
pub fn assert_entry_cites_the_registration(entry: &Entry, probe_id: &str) {
    let kata = entry.content.kata.as_ref().expect("kata");
    assert_eq!(kata.kata_id, probe_id, "the registered probe id");
    assert_eq!(
        kata.expected,
        probe_expectations(probe_id),
        "the embedded expectations are the pre-registered catalog"
    );
    assert_eq!(
        kata.catalog_hash.as_deref(),
        probe_catalog_hash(probe_id).as_deref(),
        "the entry cites the committed catalog"
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

/// The entry's `fixture_hash` is the canonical model hash of the fixture
/// it names (`KataRecord::fixture_hash`), recomputed from the current
/// bytes — a fixture edit must be a new entry, never a silent change.
pub fn assert_fixture_hash_is_current(entry: &Entry) {
    let kata = entry.content.kata.as_ref().expect("kata");
    assert_eq!(
        kata.fixture_hash,
        fixture_hash(&kata.fixture),
        "the recorded fixture hash is the canonical model hash"
    );
}
