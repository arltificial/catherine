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

use super::schema::{check_ident, check_status, StatusCheck};
use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Arrow, Ident, Key};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// One raw key declaration (format 1.3): the object, its arrow tuple, and
/// the optional status (absent means accepted — the equation rule).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawKey {
    pub(super) object: String,
    pub(super) arrows: Vec<String>,
    #[serde(default)]
    pub(super) status: Option<String>,
}

/// What one key declaration is checked against: the declared objects and
/// arrows plus the span index. Shared by both call sites (the document
/// schema pass and the schema-map target pass).
pub(super) struct KeyCtx<'a> {
    pub(super) objects: &'a BTreeMap<Ident, ()>,
    pub(super) arrows: &'a BTreeMap<Ident, Arrow>,
    pub(super) spans: &'a Spans,
}

/// One key's arrow checks: the key's object, the declared arrows, the
/// span index, and the arrows seen so far within this key.
struct KeyCheck<'a> {
    object: &'a Ident,
    declared: &'a BTreeMap<Ident, Arrow>,
    spans: &'a Spans,
    seen: BTreeSet<Ident>,
}

/// Declarative keys (additive 1.3): the object ident and declaredness, a
/// non-empty duplicate-free arrow list whose arrows are declared and
/// sourced at the key's object, and the shared status enum.
pub(super) fn check_keys(
    schema: &super::schema::RawSchema,
    ctx: KeyCtx,
) -> Result<Vec<Key>, crate::error::ParseError> {
    let mut keys = Vec::new();
    for (i, raw) in schema.keys.iter().enumerate() {
        keys.push(check_key(&format!("/schema/keys/{i}"), raw, &ctx)?);
    }
    // Canonical order: key list order in the file is presentation
    // (design Decision 4).
    keys.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(keys)
}

/// One key at its node: the object's ident and declaredness, its arrow
/// tuple, and its status.
fn check_key(base: &str, raw: &RawKey, ctx: &KeyCtx) -> Result<Key, crate::error::ParseError> {
    let object = key_object(base, raw, ctx)?;
    let mut check = KeyCheck {
        object: &object,
        declared: ctx.arrows,
        spans: ctx.spans,
        seen: BTreeSet::new(),
    };
    let arrows = check_key_arrows(base, raw, &mut check)?;
    let status = check_status(&StatusCheck {
        status: raw.status.as_deref(),
        base,
        spans: ctx.spans,
        subject: "key",
    })?;
    Ok(Key {
        object,
        arrows,
        status,
    })
}

/// A key's object at its own node: ident well-formedness and declaredness.
fn key_object(base: &str, raw: &RawKey, ctx: &KeyCtx) -> Result<Ident, crate::error::ParseError> {
    check_ident(ctx.spans, &format!("{base}/object"), &raw.object)?;
    let object = Ident(raw.object.clone());
    if !ctx.objects.contains_key(&object) {
        return Err(ctx.spans.semantic(
            &format!("{base}/object"),
            ErrorCode::InvalidKey,
            format!("key names undeclared object `{}`", raw.object),
        ));
    }
    Ok(object)
}

/// One key's arrow tuple: non-empty, then each element validated at its
/// own node (ident, declaredness, source, duplicate).
fn check_key_arrows(
    base: &str,
    raw: &RawKey,
    check: &mut KeyCheck,
) -> Result<Vec<Ident>, crate::error::ParseError> {
    if raw.arrows.is_empty() {
        return Err(check.spans.semantic(
            &format!("{base}/arrows"),
            ErrorCode::InvalidKey,
            "key declares no arrows: an injectivity claim needs at least one arrow".to_string(),
        ));
    }
    let names: Vec<Ident> = raw
        .arrows
        .iter()
        .enumerate()
        .map(|(j, name)| check_key_arrow(&format!("{base}/arrows/{j}"), name, check))
        .collect::<Result<_, _>>()?;
    Ok(names)
}

/// One key arrow at its node: an ident naming a declared arrow sourced at
/// the key's object, not repeated within the key.
fn check_key_arrow(
    node: &str,
    name: &str,
    check: &mut KeyCheck,
) -> Result<Ident, crate::error::ParseError> {
    check_ident(check.spans, node, name)?;
    let arrow = Ident(name.to_string());
    check_key_source(node, &arrow, check)?;
    if !check.seen.insert(arrow.clone()) {
        return Err(check.spans.semantic(
            node,
            ErrorCode::InvalidKey,
            format!("duplicate arrow `{name}` in one key"),
        ));
    }
    Ok(arrow)
}

/// The arrow is declared and sourced at the key's object.
fn check_key_source(
    node: &str,
    arrow: &Ident,
    check: &KeyCheck,
) -> Result<(), crate::error::ParseError> {
    let Some(definition) = check.declared.get(arrow) else {
        return Err(check.spans.semantic(
            node,
            ErrorCode::InvalidKey,
            format!("key references undeclared arrow `{}`", arrow.0),
        ));
    };
    if definition.src != *check.object {
        return Err(check.spans.semantic(
            node,
            ErrorCode::InvalidKey,
            format!(
                "key arrow `{}` is sourced at `{}`, not at the key's object `{}`",
                arrow.0, definition.src.0, check.object.0
            ),
        ));
    }
    Ok(())
}
