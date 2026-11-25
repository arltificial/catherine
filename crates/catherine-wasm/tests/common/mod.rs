//! Shared wasm-test helper: decode one report string as JSON, naming
//! the failure instead of a bare expect.

/// Decode one report string as JSON, naming the failure instead of a
/// bare expect.
pub fn from_json(report: &str) -> serde_json::Value {
    serde_json::from_str(report).unwrap_or_else(|error| panic!("report is not JSON: {error}"))
}
