//! Purpose: the serde_json error translation seam — serde's own position
//! and message, with an undefined field promoted to the stable
//! `unknown_field` code (catherine-bak). Split out of `parse/mod.rs` so
//! the parser entry point stays inside the repo's file-size gate.
//!
//! Responsibilities: convert a [`serde_json::Error`] into a
//! [`ParseError`], keeping serde_json's line/column and stripping its
//! location suffix from the message (the position travels in the fields,
//! not the text); recognise serde's `unknown field \`x\`` message and
//! carry the field name and its expectation suffix.
//!
//! Rationale: an unknown field is never silently dropped — a dropped
//! constraint key can turn a violated model into a false clean. Additive
//! minors above `KNOWN_MINOR` fail the version gate, so inside an
//! accepted version an unknown key can only be a mistake.

use crate::error::{ErrorCode, ParseError, SemanticError};

/// Convert a `serde_json` deserialization error into a [`ParseError`].
pub(super) fn doc_err(err: serde_json::Error) -> ParseError {
    let text = err.to_string();
    let message = text.split(" at line ").next().unwrap_or(&text);
    match unknown_field_parts(message) {
        Some((field, expected)) => ParseError::Semantic(SemanticError {
            line: err.line(),
            column: err.column(),
            code: ErrorCode::UnknownField,
            message: format!(
                "unknown field `{field}`: the accepted format version does not define it{expected}"
            ),
        }),
        None => ParseError::Syntax {
            line: err.line(),
            column: err.column(),
            message: message.to_string(),
        },
    }
}

/// Split serde's `unknown field \`x\`, expected one of ...` message into
/// the field name and a `; expected ...` suffix (empty when serde named
/// no expectation).
fn unknown_field_parts(message: &str) -> Option<(&str, String)> {
    let rest = message.strip_prefix("unknown field `")?;
    let (field, tail) = rest.split_once('`')?;
    let expected = tail.trim_start_matches(", ").trim();
    let suffix = if expected.is_empty() {
        String::new()
    } else {
        format!("; {expected}")
    };
    Some((field, suffix))
}
