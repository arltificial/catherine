//! Purpose: the boundary cap gate (openspec change `add-wasm-web-view`,
//! RED task 1) — an oversized document is rejected with the typed
//! `DocumentTooLarge` error and the kernel is not invoked.
//!
//! Responsibilities: pin the cap behavior structurally — the error is
//! the `DocumentTooLarge` variant with the actual and maximum byte
//! counts, and an oversized document that is *also* malformed JSON
//! yields the cap error, never a parse error: the parser cannot have
//! run (a malformed document through the parser would decode `Parse`).
//!
//! Rationale: the tests drive the plain Rust boundary functions
//! (`boundary::check`), never the `#[wasm_bindgen]` wrappers — the
//! wrappers are extern stubs that panic when called on non-wasm targets;
//! their only job is name-and-stringify for the browser glue.

use catherine_wasm::boundary::{analyze, check, MAX_DOCUMENT_BYTES};
use catherine_wasm::BoundaryError;

/// A document that is malformed JSON *and* oversized: if the kernel were
/// invoked, the parser would answer `Parse` — the cap error's presence
/// is the proof it never did.
fn oversized_malformed() -> String {
    format!(
        "{{\"format_version\": \"{}\"",
        "9".repeat(MAX_DOCUMENT_BYTES)
    )
}

#[test]
fn oversized_document_yields_the_typed_cap_error() {
    let document = oversized_malformed();
    assert!(document.len() > MAX_DOCUMENT_BYTES);
    let error = check(&document).expect_err("oversize must fail closed");
    assert_eq!(
        error,
        BoundaryError::DocumentTooLarge {
            bytes: document.len(),
            max_bytes: MAX_DOCUMENT_BYTES,
        }
    );
    // And the typed error's JSON encoding carries the stable code the
    // UI matches on.
    let encoded: serde_json::Value =
        serde_json::from_str(&error.to_json()).expect("boundary error encoding decodes");
    assert_eq!(encoded["code"], "document_too_large");
}

#[test]
fn under_cap_document_reaches_the_kernel() {
    // A valid document well under the cap: the boundary must forward it
    // to the kernel, not reject it — clean parse, empty findings.
    let document = r#"{
        "format_version": "1.0",
        "schema": {"objects": [], "arrows": [], "equations": []},
        "instance": {"rows": {}, "tables": {}}
    }"#;
    let report = check(document).expect("minimal document parses");
    let value: serde_json::Value = serde_json::from_str(&report).expect("report is JSON");
    assert_eq!(value["findings"], serde_json::json!([]));
}

#[test]
fn analyze_reports_the_search_outcome_discriminator() {
    // The zero-size rule through the boundary: an empty document's own
    // row shape gives the search no row space anywhere → UNKNOWN with
    // the empty scope, never a contradiction, and the outcome field says
    // so machine-readably.
    let empty = r#"{
        "format_version": "1.0",
        "schema": {"objects": [], "arrows": [], "equations": []},
        "instance": {"rows": {}, "tables": {}}
    }"#;
    let report = analyze(empty, "{}").expect("minimal document analyzes");
    let value: serde_json::Value = serde_json::from_str(&report).expect("report is JSON");
    assert_eq!(value["outcome"], "unknown");
    assert_eq!(value["unknown"]["rows"], serde_json::json!({}));

    // One object, one row, nothing to violate: the search finds a
    // witness — and the report says "witness" instead of collapsing the
    // found instance into silence (a found witness is never
    // indistinguishable from a vacuous run).
    let one_row = r#"{
        "format_version": "1.0",
        "schema": {"objects": ["a"], "arrows": [], "equations": []},
        "instance": {"rows": {"a": ["x"]}, "tables": {}}
    }"#;
    let report = analyze(one_row, "{}").expect("one-row document analyzes");
    let value: serde_json::Value = serde_json::from_str(&report).expect("report is JSON");
    assert_eq!(value["outcome"], "witness");
    assert_eq!(value["findings"], serde_json::json!([]));
}
