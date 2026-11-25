//! Purpose: the Δ schema map's arrow-assignment parse pass — totality
//! over the declared source arrows and validated Δ target paths. Split
//! out of `schema_map/mod.rs` to keep both inside the file-size gate.

use super::super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Ident, Path};
use std::collections::BTreeMap;

use super::{RawSchemaMap, SourceMap, TargetSchema};

/// The arrow assignments: every declared source arrow carries an entry —
/// `null` drops it explicitly; a target path is non-empty and every path
/// element is a declared target arrow.
pub(super) fn mapped_arrows(
    raw: &RawSchemaMap,
    source: &SourceMap,
    target: &TargetSchema,
) -> Result<BTreeMap<Ident, Option<Path>>, crate::error::ParseError> {
    let mut mapped = BTreeMap::new();
    for name in source.arrows.keys() {
        mapped.insert(name.clone(), mapped_arrow(raw, name, target)?);
    }
    reject_foreign_arrows(raw, source.arrows, source.spans)?;
    Ok(mapped)
}

/// One arrow's assignment: required, and `null` (absent) means dropped
/// explicitly.
fn mapped_arrow(
    raw: &RawSchemaMap,
    name: &Ident,
    target: &TargetSchema,
) -> Result<Option<Path>, crate::error::ParseError> {
    let pointer = format!("/schema_map/arrows/{}", name.0);
    let Some(assignment) = raw.arrows.get(&name.0) else {
        return Err(target.spans.semantic(
            &pointer,
            ErrorCode::PartialSchemaMap,
            format!("schema map is partial: arrow `{}` has no assignment — map it, or map it to null to drop it explicitly", name.0),
        ));
    };
    match assignment {
        Some(path) => declared_path(path, target, &pointer),
        None => Ok(None),
    }
}

/// The declared target path for one source arrow: non-empty, and every
/// element a declared target arrow (endpoint typing runs in
/// `check_arrow_typing` once both assignment maps exist).
fn declared_path(
    path: &[String],
    target: &TargetSchema,
    pointer: &str,
) -> Result<Option<Path>, crate::error::ParseError> {
    if path.is_empty() {
        return Err(target.spans.semantic(
            pointer,
            ErrorCode::PartialSchemaMap,
            "arrow maps to an empty path".to_string(),
        ));
    }
    for (index, element) in path.iter().enumerate() {
        let element_ident = Ident(element.clone());
        if !target.schema.arrows.contains_key(&element_ident) {
            return Err(target.spans.semantic(
                &format!("{pointer}/{index}"),
                ErrorCode::PartialSchemaMap,
                format!(
                    "arrow maps to path element `{element}`, which the target schema does not declare"
                ),
            ));
        }
    }
    let path_idents = path.iter().map(|element| Ident(element.clone())).collect();
    Ok(Some(Path(path_idents)))
}

/// Refuse assignments for source arrows the document does not declare.
fn reject_foreign_arrows(
    raw: &RawSchemaMap,
    arrows: &BTreeMap<Ident, crate::model::Arrow>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    for key in raw.arrows.keys() {
        if !arrows.contains_key(&Ident(key.clone())) {
            return Err(spans.semantic(
                &format!("/schema_map/arrows/{key}"),
                ErrorCode::PartialSchemaMap,
                format!("schema map assigns `{key}`, which the document schema does not declare"),
            ));
        }
    }
    Ok(())
}
