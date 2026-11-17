//! Purpose: the P6 kata validation ledger (openspec change
//! `add-kata-validation-ledger`, beads `catherine-9vv`) — the project
//! stop condition, as code: content-addressed entries, a validator that
//! refuses bad science, the recorded rollup procedure, and the
//! kata-fixture harness shared with the P8/P9 validation work.
//!
//! Responsibilities: [`entry`] carries the entry shapes and their
//! sha256 content addresses (the P0 hash port); [`decision`] carries the
//! stop-gate decision schema and its closed `extend | hold | retire`
//! outcome vocabulary; [`catalog`] is the
//! pre-registered per-kata expectation mapping (fixed before any run);
//! [`harness`] loads a fixture, executes the kernel checks, and matches
//! findings against the catalog; [`validate`] refuses entries without a
//! prior negative control, verdicts outside the closed vocabulary,
//! stop-gate decisions whose outcome is outside `extend | hold | retire`
//! (or whose kind and payload disagree), and entries without model-hash
//! references — plus id/order/supersession integrity; [`rollup`] derives
//! the project verdict from the entries by the recorded procedure,
//! mechanically.
//!
//! Rationale: verdicts are derived, never asserted — every step the
//! rollup takes is one of the steps written into the rollup entry
//! itself (design Decision 3), and the negative control is enforced
//! first by both the validator and the record path (design Decision 2).
//! The crate is not a kernel crate: it consumes `catherine-core`'s
//! public artifacts (parse, hash, findings encoding) and stays
//! genesis-free, so P8/P9 can adopt the harness without CLI plumbing.

pub mod catalog;
pub mod decision;
pub mod entry;
pub mod harness;
pub mod rollup;
pub mod validate;
