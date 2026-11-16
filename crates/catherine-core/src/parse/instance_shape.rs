//! Purpose: the instance-section shape gate — an `instance` object this
//! build cannot read is refused, never silently read as an empty instance.
//!
//! Responsibilities: `check_instance_shape` rejects a non-empty `instance`
//! object whose only content is keys this parser does not recognize
//! (nothing under `rows` or `tables`) with a `ParseError::Syntax` message
//! naming those keys; `{}`, empty `rows`/`tables`, and unrecognized keys
//! *beside* a recognizable section stay legal.
//!
//! Rationale: unknown keys are ignored by design (additive minors may add
//! optional fields), but that tolerance presupposes recognizable content
//! alongside them. An instance object carrying neither `rows` nor `tables`
//! is a different shape from a version this build knows; continuing would
//! report it as an empty instance and exit clean — a class-A silent
//! reinterpretation of input, not tolerance. Refusing it is the
//! no-guessed-parse rule applied one level below the version gate.

use super::schema::RawInstance;
use super::spans::Spans;
use crate::error::ParseError;

/// Reject an `instance` section with no `rows`/`tables` content but at
/// least one unrecognized key, naming the keys in the message.
pub(super) fn check_instance_shape(
    instance: &RawInstance,
    spans: &Spans,
) -> Result<(), ParseError> {
    if !instance.rows.is_empty() || !instance.tables.is_empty() || instance.extra.is_empty() {
        return Ok(());
    }
    let keys: Vec<String> = instance
        .extra
        .keys()
        .map(|key| format!("`{key}`"))
        .collect();
    Err(spans.syntax(
        "/instance",
        format!(
            "instance section has no `rows` or `tables` content (unrecognized keys: {}) — \
             an unreadable instance shape is refused, never read as empty",
            keys.join(", ")
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape gate's own unit pins: the two accepted shapes and the
    /// rejected one, independent of the full parse pipeline.
    #[test]
    fn shape_gate_accepts_empty_and_populated_sections() {
        let empty: RawInstance = serde_json::from_str("{}").expect("empty instance object");
        let spans = Spans::default();
        assert!(check_instance_shape(&empty, &spans).is_ok());

        let populated: RawInstance = serde_json::from_str(
            r#"{"rows": {"N": ["a"]}, "tables": {"f": {}}, "future_optional": true}"#,
        )
        .expect("populated instance object");
        assert!(check_instance_shape(&populated, &spans).is_ok());
    }

    #[test]
    fn shape_gate_names_the_unrecognized_keys() {
        let legacy: RawInstance =
            serde_json::from_str(r#"{"N": [{"id": "a", "f": "b"}], "M": []}"#)
                .expect("legacy instance object");
        let error = check_instance_shape(&legacy, &Spans::default())
            .expect_err("an unreadable shape is refused");
        let ParseError::Syntax { message, .. } = error else {
            panic!("the shape gate is structural: got {error:?}");
        };
        assert!(message.contains("unrecognized keys: `M`, `N`"), "{message}");
    }
}
