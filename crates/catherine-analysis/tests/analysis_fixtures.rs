//! The five analysis fixtures (openspec change `add-bounded-analysis`,
//! RED tasks 1–5) plus determinism assertions.
//!
//! Purpose: pin the concrete analysis outcomes tasks.md names —
//! contradiction proven by completed exhaustion within bounds, `UNKNOWN`
//! under a budget smaller than the space (and the zero-size rule),
//! the parallel-path domain question, redundancy proven by rewriting,
//! and bounded distinguishability over an explicit observation set.
//!
//! Responsibilities: build real kernel schemas/instances (fixture
//! builders below — real kernel documents, never fabricated findings),
//! call the analysis ops with explicit bounds, and assert structurally
//! (`PartialEq` on outcome values, `serde_json::Value` field checks for
//! the serialized witness shapes). Never byte-compared snapshots: the
//! determinism tests compare two fresh runs against each other, not a
//! stored string.
//!
//! Rationale: assertions are structural per the repo's anti-snapshot
//! discipline (DESIGN §3); every finding is asserted through the P2
//! taxonomy shapes (`findings.rs`), so the producers cannot drift from
//! the pinned witnesses without failing here.

mod common;

#[path = "analysis_fixtures/builders.rs"]
mod builders;

#[path = "analysis_fixtures/tests.rs"]
mod tests;
