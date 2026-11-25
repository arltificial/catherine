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
pub type TestError = Box<dyn std::error::Error>;

/// Unwrap an `Option` as a named test failure: the `?`-propagating
/// replacement for `Option::unwrap` in test bodies.
pub fn present<T>(value: Option<T>, what: &str) -> Result<T, TestError> {
    value.ok_or_else(|| format!("{what} absent").into())
}

/// One fixture's model hash, recomputed from its canonical bytes.
pub fn fixture_hash(path: &str) -> Result<String, TestError> {
    let text = std::fs::read_to_string(format!("{ROOT}/{path}"))?;
    let document = parse(&text)?;
    Ok(model_hash(&document))
}

/// Every claim-bearing pin of one effective probe entry: the registered
/// id, the pre-registered rows, the committed catalog address, and the
/// verdict the comparison arithmetic derives from the entry's own
/// comparison data.
pub fn assert_entry_cites_the_registration(entry: &Entry, probe_id: &str) -> Result<(), TestError> {
    let kata = present(entry.content.kata.as_ref(), "kata record")?;
    assert_eq!(kata.identity.kata_id, probe_id, "the registered probe id");
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
        &kata.identity.kata_id,
        &kata.derivation.comparison.unique_detections,
        kata.derivation.comparison.baseline.as_ref(),
    )?;
    let recorded: Verdict = kata.derivation.verdict.parse()?;
    assert_eq!(
        recorded, derived,
        "the recorded verdict for `{}` must equal the derivation from its own comparison",
        kata.identity.name
    );
    Ok(())
}

/// The entry's `fixture_hash` is the canonical model hash of the fixture
/// it names (`KataRecord::fixture_hash`), recomputed from the current
/// bytes — a fixture edit must be a new entry, never a silent change.
pub fn assert_fixture_hash_is_current(entry: &Entry) -> Result<(), TestError> {
    let kata = present(entry.content.kata.as_ref(), "kata record")?;
    assert_eq!(
        kata.provenance.fixture_hash,
        fixture_hash(&kata.provenance.fixture)?,
        "the recorded fixture hash is the canonical model hash"
    );
    Ok(())
}
