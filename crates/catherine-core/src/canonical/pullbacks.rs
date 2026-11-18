//! Purpose: the canonical `schema.pullbacks` entries (additive 1.6,
//! openspec change `add-pullback-checks`) — split out of `value.rs` so
//! the canonical serializer stays inside the repo's file-size gate.
//!
//! Responsibilities: emit each declaration as `{"apex", "legs",
//! "cospan", "status"}`, sorted by the model's canonical sort key
//! `(apex, legs, cospan, status)` even for programmatically constructed
//! schemas; the caller omits an empty list entirely.
//!
//! Rationale: the parser stores declarations sorted, but constructed
//! values must serialize the same way (the `schema_export` discipline);
//! the section is present iff non-empty so every pre-1.6 document keeps
//! its exact canonical bytes and hash.

use crate::model::{Pullback, Schema};
use serde_json::Value;

use super::value::{obj, string};

/// The canonical pullback entries, sorted by
/// `(apex, legs, cospan, status)`.
pub(super) fn pullback_values(schema: &Schema) -> Vec<Value> {
    let mut pullbacks: Vec<&Pullback> = schema.pullbacks.iter().collect();
    pullbacks.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    pullbacks
        .iter()
        .map(|pullback| pullback_value(pullback))
        .collect()
}

/// One canonical pullback entry: the apex, its two legs, the two cospan
/// arrows, and the constraint status.
fn pullback_value(pullback: &Pullback) -> Value {
    obj([
        ("apex", string(&pullback.apex.0)),
        (
            "legs",
            Value::Array(pullback.legs.iter().map(|leg| string(&leg.0)).collect()),
        ),
        (
            "cospan",
            Value::Array(
                pullback
                    .cospan
                    .iter()
                    .map(|arrow| string(&arrow.0))
                    .collect(),
            ),
        ),
        ("status", string(pullback.status.as_str())),
    ])
}
