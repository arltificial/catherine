//! Purpose: Catherine's kernel — the `.ct.json` v1 model document format,
//! its parser, canonical serialization, model hash, typed instance tables,
//! and executed path-equation checks. Everything downstream (P2 findings
//! taxonomy, P6 ledger content-addressing) consumes the artifacts this
//! crate publishes.
//!
//! Responsibilities: expose the parsed kernel types (`model`), the
//! line-accurate parser (`parse`), the byte-stable canonical serializer
//! (`canonical`), the sha256 model hash (`hash`), the typed instance
//! tables (`instance`), the executed checks (`checks`), and the public
//! findings taxonomy (`findings`) — the output contract later phases
//! print or consume.
//!
//! Rationale: kernel crates stay small, regular, and genesis-free (DESIGN
//! §3); deps are locked to serde/serde_json (+ hegeltest for tests).

pub mod canonical;
pub mod checks;
pub mod composition;
pub mod error;
pub mod findings;
pub mod hash;
pub mod instance;
pub mod migrate;
pub mod parse;
pub mod rule_lens;

/// Compact-to-public finding rendering — the parsed flavor's name
/// materialization step (the only one that touches `Finding`).
mod render;

/// The interned execution core, wrapped verbatim (`catherine-engine`)
/// — the verification lane's unit of proof (ledger `0002` revisit
/// trigger 2). `catherine_core::engine` paths are unchanged for every
/// consumer; the supported kernel surface stays `checks`/`instance`.
pub use catherine_engine::{engine, model};
