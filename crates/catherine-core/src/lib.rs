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
//! §3); deps are locked to serde/serde_json (+ proptest for tests).

pub mod canonical;
pub mod checks;
pub mod error;
pub mod findings;
pub mod hash;
pub mod instance;
pub mod model;
pub mod parse;
