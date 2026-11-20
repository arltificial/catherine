//! Purpose: Catherine's interned execution core — the model types, the
//! findings epistemic label, and the dense-id engine (schema/instance
//! interning, build classification, path composition, every executed
//! check family). Dependency-light by design: the Kani verification
//! lane (`catherine-kani`) compiles exactly this crate plus its
//! harnesses, so bounded model checking pays for the kernel's execution
//! path and nothing else (beads `catherine-6g2`; ledger `0002`
//! revisit trigger 2 — the standalone-crate wrap that shed the
//! in-crate verification overhead).
//!
//! Responsibilities: the parsed model (`model`), the epistemic label
//! (`findings`), and the engine's two flavors — the parsed flavor
//! (owned canonical names, concrete data) and the lane flavor (id keys
//! only, zero `String` materialization, compact int-keyed findings).
//!
//! Rationale: the kernel's supported public surface stays
//! `catherine-core` (`checks`/`instance`), which wraps this crate's
//! modules verbatim; this crate is the verification lane's unit of
//! proof, and its `engine` module is `pub` precisely so the harness
//! crate can wrap it (hidden from docs — not a supported kernel API).
//! Serde support is the optional `serde` feature (default-on for the
//! parsed kernel; the lane builds without it, so no serde code is in
//! the solver's formula).

pub mod engine;
pub mod findings;
pub mod model;
