//! Purpose: the wasm boundary (openspec change `add-wasm-web-view`, beads
//! `catherine-9b5`) — the browser tab's only door into the kernel. The
//! crate hosts no logic of its own: every export is "decode the JSON
//! document, drive the kernel or analysis entry once, serialize the
//! result back to JSON".
//!
//! Responsibilities: enforce the row-count cap AT THE BOUNDARY — an
//! oversized document is rejected with the typed
//! [`BoundaryError::DocumentTooLarge`] before the parser ever sees it
//! (the kernel is not invoked on oversize, a browser-tab DoS floor) —
//! and expose `check` (kernel parse → build → executed checks, the
//! exact path the CLI drives) and `analyze` (bounded analysis over the
//! same decoded model) as JSON-string-in / JSON-string-out wasm exports
//! whose findings encoding is the kernel's own serde encoding verbatim —
//! the same bytes the CLI forwards in `data.findings`.
//!
//! Rationale: the kernel crates stay genesis-free and wasm-optional;
//! this crate adds only serde/wasm-bindgen glue over `catherine-core`
//! and `catherine-analysis`, so parity between terminal and browser is
//! structural, not maintained (one execution path, two front doors).
//! Errors cross the boundary as JSON strings carrying the typed error's
//! stable encoding (`code`-tagged), never as naked text.

pub mod boundary;

pub use boundary::BoundaryError;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// The boundary's exported entry points, re-exposed as wasm exports.
/// Each takes the document (and config) as a JSON string and returns the
/// report as a JSON string; a boundary rejection is the typed error's
/// JSON encoding on the error side.
#[wasm_bindgen]
pub fn check(document: &str) -> Result<String, JsValue> {
    boundary::check(document).map_err(|error| JsValue::from(error.to_json()))
}

/// The analysis export: bounded analysis over the decoded model. `config`
/// is the JSON encoding of [`boundary::AnalysisConfig`] (`"{}"` is the
/// all-defaults call).
#[wasm_bindgen]
pub fn analyze(document: &str, config: &str) -> Result<String, JsValue> {
    boundary::analyze(document, config).map_err(|error| JsValue::from(error.to_json()))
}

/// The document cap, so the UI can pre-flight file sizes against the
/// same number the boundary enforces (the boundary stays the authority).
#[wasm_bindgen]
pub fn max_document_bytes() -> usize {
    boundary::MAX_DOCUMENT_BYTES
}
