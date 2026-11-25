//! Purpose: the naturality runners' labeled errors — the map-resolution
//! refusals both sweeps report through the shared `emit_error` path.
//! Split out of `runners/naturality.rs` verbatim (mechanical lint
//! refactor; no behavior change) so the runner stays inside the repo's
//! file-size gate.
//!
//! Responsibilities: name each `NaturalityError`/`MigrationError` arm as
//! a labeled error — `unknown_map_target` (the same wording for the
//! same-schema sweep and the migration sweep), `incomplete_instance_map`,
//! `missing_instance_map`, and the migration-only `missing_schema_map`.
//!
//! Rationale: target resolution is construction-time, so these are
//! refusals of declared data, not findings; the wording is shared between
//! the two sweeps so a fixture pins one message, not two.

use crate::render::LabeledError;
use catherine_analysis::naturality::{MigrationError, NaturalityError};

/// The migration-mode labeled errors (catherine-kw9.2): the same
/// map-resolution refusals as the same-schema sweep, plus the absent
/// `schema_map` refusal (unreachable through the dispatch branch, which
/// requires the map — kept for totality of the analysis error type).
pub(super) fn migration_naturality_error(
    error: &MigrationError,
    second_path: &str,
) -> LabeledError {
    match error {
        MigrationError::UnknownMapTarget {
            object,
            source_row,
            target_row,
        } => unknown_map_target_error(object, source_row, target_row, second_path),
        MigrationError::IncompleteInstanceMap { object, row } => {
            incomplete_instance_map_error(object, row)
        }
        MigrationError::MissingSchemaMap => missing_schema_map_error(),
        MigrationError::MissingInstanceMap => missing_instance_map_error(),
    }
}

/// The shared `unknown_map_target` arm constructor: the same wording for
/// the same-schema sweep and the migration sweep.
pub(super) fn unknown_map_target_error(
    object: &catherine_core::model::Ident,
    source_row: &str,
    target_row: &str,
    second_path: &str,
) -> LabeledError {
    LabeledError {
        code: "unknown_map_target".to_string(),
        line: None,
        column: None,
        message: format!(
            "instance map cell for `{}` row `{source_row}` names target row `{target_row}`, which `{second_path}` does not declare",
            object.0
        ),
        hint: "map every source row to a row the target instance declares, or fix the target document",
    }
}

/// The shared `incomplete_instance_map` arm constructor.
pub(super) fn incomplete_instance_map_error(
    object: &catherine_core::model::Ident,
    row: &catherine_core::model::RowId,
) -> LabeledError {
    LabeledError {
        code: "incomplete_instance_map".to_string(),
        line: None,
        column: None,
        message: format!(
            "instance map does not cover row #{} of `{}`",
            row.0, object.0
        ),
        hint: "map every row of every schema object; the map must be total",
    }
}

/// The shared `missing_instance_map` arm constructor.
pub(super) fn missing_instance_map_error() -> LabeledError {
    LabeledError {
        code: "missing_instance_map".to_string(),
        line: None,
        column: None,
        message: "the source document declares no `instance_map`".to_string(),
        hint: "add an `instance_map` section to the source document (format 1.4)",
    }
}

/// The migration-only `missing_schema_map` arm: the source document
/// declares no `schema_map`.
pub(super) fn missing_schema_map_error() -> LabeledError {
    LabeledError {
        code: "missing_schema_map".to_string(),
        line: None,
        column: None,
        message: "the source document declares no `schema_map`".to_string(),
        hint:
            "add a `schema_map` section to the source document (format 1.1) naming the Δ migration",
    }
}

/// The labeled errors for the same-schema sweep: `unknown_map_target`
/// naming the object, the mapped source row, and the missing target row.
pub(super) fn naturality_error(error: &NaturalityError, second_path: &str) -> LabeledError {
    match error {
        NaturalityError::UnknownMapTarget {
            object,
            source_row,
            target_row,
        } => unknown_map_target_error(object, source_row, target_row, second_path),
        NaturalityError::MissingInstanceMap => missing_instance_map_error(),
        NaturalityError::IncompleteInstanceMap { object, row } => {
            incomplete_instance_map_error(object, row)
        }
    }
}
