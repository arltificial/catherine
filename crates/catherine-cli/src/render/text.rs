//! Purpose: the transcript's text spellings — the shared string
//! rendering helpers behind every human line: epistemic label suffix,
//! path/arrow composition and list spellings, `Object#n` row spellings,
//! and path domain/codomain lookups.
//!
//! Responsibilities: pure formatting over the kernel's model types; no
//! finding-specific wording (the per-kind renderers own that).

use catherine_core::findings::EpistemicLabel;
use catherine_core::model::{Ident, Path, RowId, Schema};

/// A composed naturality value as the transcript spells it: `Object#n`
/// when resolved, `unresolved` when the composition has no value.
pub(super) fn naturality_value(object: Option<&str>, value: Option<RowId>) -> String {
    match (object, value) {
        (Some(name), Some(row)) => format!("{name}#{}", row.0),
        (_, Some(row)) => row.0.to_string(),
        _ => "unresolved".to_string(),
    }
}
/// The epistemic label as rendered text: the kernel-computed default
/// (`executed`) stays off the human line so the §5 transcript is
/// byte-reproducible; the other weights append their label.
pub(super) fn label_suffix(label: EpistemicLabel) -> String {
    match label {
        EpistemicLabel::Executed => String::new(),
        EpistemicLabel::Hypothesized => " (hypothesized)".to_string(),
        EpistemicLabel::Unknown => " (unknown)".to_string(),
    }
}

/// A path as the transcript spells it: the arrow names in composition
/// order joined with `∘`.
pub fn path_string(path: &Path) -> String {
    arrows_string(&path.0)
}

/// An arrow sequence as the transcript spells it: names joined with `∘`.
pub(super) fn arrows_string(arrows: &[Ident]) -> String {
    arrows
        .iter()
        .map(|ident| ident.0.as_str())
        .collect::<Vec<_>>()
        .join("∘")
}

/// An arrow list as a declaration spells it: names joined with `, ` (a
/// leg/cospan list is a relation, not a composed path).
pub(super) fn arrow_list(arrows: &[Ident]) -> String {
    arrows
        .iter()
        .map(|ident| ident.0.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A row as the transcript spells it: `Object#n` when the object is
/// known from the schema, the bare number otherwise (totality).
pub(super) fn row_at(object: Option<&str>, row: RowId) -> String {
    match object {
        Some(name) => format!("{name}#{}", row.0),
        None => row.0.to_string(),
    }
}

/// The composed path's domain: its lead arrow's src.
pub(super) fn path_domain<'a>(schema: &'a Schema, path: &Path) -> Option<&'a str> {
    schema.arrows.get(path.0.first()?).map(|a| a.src.0.as_str())
}

/// The composed path's codomain: its last arrow's tgt.
pub(super) fn path_codomain<'a>(schema: &'a Schema, path: &Path) -> Option<&'a str> {
    schema.arrows.get(path.0.last()?).map(|a| a.tgt.0.as_str())
}
