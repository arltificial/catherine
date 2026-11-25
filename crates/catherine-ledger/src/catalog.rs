//! Purpose: the pre-registered per-kata expectation mapping — what each
//! kata's faults CAN mean under catherine's path-equation language,
//! fixed before any run (design Decision 3: no goalpost moves). The
//! source of truth is the ct-catlab series verdict ledger
//! (`ak/matadero/series/ct-catlab/SERIES.md`); every entry here cites
//! its series row.
//!
//! Responsibilities: [`expectations`] maps a kata id to its expectation
//! list; an unknown kata id maps to an empty list (an unregistered kata
//! cannot run under the ledger — the entry's expectations being empty
//! is the honest record of that). Domain probes are pre-registered in
//! the same catalog surface ([`probe_expectations`], same row shape);
//! the class-level rows are the pre-registration and
//! [`probe_catalog_hash`] is its content address, which the entry
//! carries.
//!
//! Rationale: the proposal text rounds kata-3 (Vending) to "6/6" — the
//! series ledger says otherwise (F2 bounds, F3 command vocabulary, F5
//! collision caught structurally; F6 REFUTED by an executed MR≠RM
//! counterexample). The series record is ground truth; the discrepancy
//! is recorded here so the ledger entry can carry it forward.

use crate::entry::Expectation;
use catherine_core::canonical::write_value;
use catherine_core::hash::sha256_digest;

mod katas;
mod probes;

pub use katas::expectations;
use probes::{
    probe_composition, probe_coproduct, probe_migration, probe_naturality,
    probe_rule_lens_extracted,
};

/// The registered domain-probe ids. Probes ride the kata entry shape in
/// their own ledger directory (design Decision 1 revised); this list is
/// the closed set the validator recognizes.
pub const PROBE_IDS: [&str; 5] = [
    "probe-migration",
    "probe-rule-lens-extracted",
    "probe-naturality",
    "probe-composition",
    "probe-coproduct",
];

/// Whether the id names a registered domain probe.
pub fn is_probe(id: &str) -> bool {
    PROBE_IDS.contains(&id)
}

/// The pre-registered expectations for one probe id (empty for an
/// unregistered probe — the same discipline as [`expectations`]: an
/// unknown id cannot derive a verdict).
pub fn probe_expectations(probe_id: &str) -> Vec<Expectation> {
    match probe_id {
        "probe-migration" => probe_migration(),
        "probe-rule-lens-extracted" => probe_rule_lens_extracted(),
        "probe-naturality" => probe_naturality(),
        "probe-composition" => probe_composition(),
        "probe-coproduct" => probe_coproduct(),
        _ => Vec::new(),
    }
}

/// The content address of a probe's pre-registered catalog: `sha256:`
/// over the canonical bytes of its rows, via the kernel's normative
/// writer (the ledger's established id discipline). `None` when the id
/// has no rows — there is no catalog to reference.
pub fn probe_catalog_hash(probe_id: &str) -> Option<String> {
    let rows = probe_expectations(probe_id);
    if rows.is_empty() {
        return None;
    }
    let value = serde_json::to_value(&rows).ok()?;
    Some(sha256_digest(write_value(&value).as_bytes()))
}
