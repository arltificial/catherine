//! The declarative-key parse pass (additive 1.3, openspec change
//! `add-declarative-keys`). Split out of `schema.rs` so both files stay
//! under the size gate; no behavior change.
//!
//! Responsibilities: validate `schema.keys` — the object's ident and
//! declaredness, a non-empty duplicate-free arrow tuple whose arrows are
//! declared and sourced at the key's object, and the shared status enum —
//! and store the keys in canonical `(object, arrows, status)` order.
//! Every defect is `invalid_key` at its offending node; a malformed key is
//! never repaired or dropped (design Decision 6).

use super::schema::{check_ident, check_status};
use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Arrow, Ident, Key};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// One raw key declaration (format 1.3): the object, its arrow tuple, and
/// the optional status (absent means accepted — the equation rule).
#[derive(Deserialize)]
pub(super) struct RawKey {
    pub(super) object: String,
    pub(super) arrows: Vec<String>,
    #[serde(default)]
    pub(super) status: Option<String>,
}

/// Declarative keys (additive 1.3): the object ident and declaredness, a
/// non-empty duplicate-free arrow list whose arrows are declared and
/// sourced at the key's object, and the shared status enum.
pub(super) fn check_keys(
    schema: &super::schema::RawSchema,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Vec<Key>, crate::error::ParseError> {
    let mut keys = Vec::new();
    for (i, raw) in schema.keys.iter().enumerate() {
        let base = format!("/schema/keys/{i}");
        check_ident(spans, &format!("{base}/object"), &raw.object)?;
        let object = Ident(raw.object.clone());
        if !objects.contains_key(&object) {
            return Err(spans.semantic(
                &format!("{base}/object"),
                ErrorCode::InvalidKey,
                format!("key names undeclared object `{}`", raw.object),
            ));
        }
        let arrow_names = check_key_arrows(&base, raw, &object, arrows, spans)?;
        let status = check_status(raw.status.as_deref(), &base, spans, "key")?;
        keys.push(Key {
            object,
            arrows: arrow_names,
            status,
        });
    }
    // Canonical order: key list order in the file is presentation
    // (design Decision 4).
    keys.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(keys)
}

/// One key's arrow tuple: non-empty, then each element validated at its
/// own node (ident, declaredness, source, duplicate).
fn check_key_arrows(
    base: &str,
    raw: &RawKey,
    object: &Ident,
    declared: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Vec<Ident>, crate::error::ParseError> {
    if raw.arrows.is_empty() {
        return Err(spans.semantic(
            &format!("{base}/arrows"),
            ErrorCode::InvalidKey,
            "key declares no arrows: an injectivity claim needs at least one arrow".to_string(),
        ));
    }
    let mut seen = BTreeSet::new();
    let mut names = Vec::with_capacity(raw.arrows.len());
    for (j, name) in raw.arrows.iter().enumerate() {
        names.push(check_key_arrow(
            &format!("{base}/arrows/{j}"),
            name,
            object,
            declared,
            &mut seen,
            spans,
        )?);
    }
    Ok(names)
}

/// One key arrow at its node: an ident naming a declared arrow sourced at
/// the key's object, not repeated within the key.
fn check_key_arrow(
    node: &str,
    name: &str,
    object: &Ident,
    declared: &BTreeMap<Ident, Arrow>,
    seen: &mut BTreeSet<Ident>,
    spans: &Spans,
) -> Result<Ident, crate::error::ParseError> {
    check_ident(spans, node, name)?;
    let arrow = Ident(name.to_string());
    let Some(definition) = declared.get(&arrow) else {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidKey,
            format!("key references undeclared arrow `{name}`"),
        ));
    };
    if definition.src != *object {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidKey,
            format!(
                "key arrow `{name}` is sourced at `{}`, not at the key's object `{}`",
                definition.src.0, object.0
            ),
        ));
    }
    if !seen.insert(arrow.clone()) {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidKey,
            format!("duplicate arrow `{name}` in one key"),
        ));
    }
    Ok(arrow)
}
