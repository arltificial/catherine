//! Purpose: the Memento pattern lens (beads `catherine-mx1.11`) — the
//! save/restore pair read as a Δ migration, checked by the migration
//! engine's loud lost-data enumeration.
//!
//! Responsibilities: one `'static [`PatternMeta`] matching the disposition
//! row for `memento`, and `run` invoking
//! [`catherine_core::migrate::lost_data`] on the fixture's declared schema
//! map. The lens fills only the migrations channel — the migrate surface's
//! own `lost_row`/`lost_cell` findings — and reports the exact scope of a
//! model that declares no save/restore map instead of claiming clean.
//!
//! Rationale: the sealed kata ledger records the migrate engine's findings
//! verbatim (`lost_cell:<arrow>#<row>`), and the identity control's
//! faithful restore yields an empty list, so the lens must reuse that same
//! engine rather than invent a taxonomy kind or a second heuristic. The
//! recorded non-invertibility row is the version-skew fixture's five lost
//! cells; the round-trip proof lives in the engine and adds no separate
//! finding.

use crate::pattern::{LawKind, LensCall, LensFindings, LensOutcome, PatternLens, PatternMeta};
use catherine_core::migrate::lost_data;

/// The disposition row's metadata, verbatim.
pub static MEMENTO_META: PatternMeta = PatternMeta {
    id: "memento",
    name: "Memento",
    abstraction: "delta save/restore",
    law: LawKind::Delta,
    scope: "lost-cell enumeration and non-invertibility witnesses; control self-negates",
};

/// The Memento lens: Δ lost-data enumeration over the declared schema map.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &MEMENTO_META
    }

    fn run(&self, call: LensCall<'_>) -> LensOutcome {
        let LensCall { input, caps: _caps } = call;
        let Some(migrations) = lost_data(input.model) else {
            return LensOutcome::Unknown {
                scope: "no schema_map section; no save/restore delta was declared".to_string(),
            };
        };
        LensOutcome::Findings(LensFindings {
            checks: Vec::new(),
            migrations,
        })
    }
}
