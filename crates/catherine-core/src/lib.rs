//! Purpose: Catherine's kernel skeleton (openspec change
//! `add-model-document-kernel`) — the `.ct.json` v1 model document format,
//! its parser, canonical serialization, and model hash. Everything
//! downstream (P1 executed checks, P6 ledger content-addressing) consumes
//! this artifact.
//!
//! Responsibilities: expose the parsed kernel types (`model`), the
//! line-accurate parser (`parse`), the byte-stable canonical serializer
//! (`canonical`), and the sha256 model hash (`hash`).
//!
//! Rationale: kernel crates stay small, regular, and genesis-free (DESIGN
//! §3); deps are locked to serde/serde_json (+ proptest for tests).

pub mod canonical;
pub mod error;
pub mod hash;
pub mod model;
pub mod parse;
