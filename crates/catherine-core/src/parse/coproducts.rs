//! Purpose: the coproduct parse pass (additive 1.5, openspec change
//! `add-coproduct-exhaustiveness`). Split out of `schema.rs` so both
//! files stay under the size gate.
//!
//! Responsibilities: validate `schema.coproducts` — the object's ident
//! and declaredness, a non-empty duplicate-free arrow list whose arrows
//! are declared injections `V_k → C` (targeted at the coproduct object
//! and not sourced at it), and the shared status enum — and store the
//! declarations in canonical `(object, arrows, status)` order. Every
//! defect is `invalid_coproduct` at its offending node; a malformed
//! declaration is never repaired or dropped (the `invalid_key`
//! discipline, design Decision 1).
//!
//! Rationale: one declaration grammar with keys means one parse
//! discipline and one canonicalization path. The kernel check can then
//! trust the parsed shape: every declared arrow is a total injection
//! `V_k → C`, so coverage and disjointness are decidable over the
//! instance tables alone.

use super::schema::{check_ident, check_status, RawSchema};
use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Arrow, Coproduct, Ident};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// One raw coproduct declaration (format 1.5): the object, its injection
/// arrows, and the optional status (absent means accepted — the equation
/// and key rule).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawCoproduct {
    pub(super) object: String,
    pub(super) arrows: Vec<String>,
    #[serde(default)]
    pub(super) status: Option<String>,
}

/// Coproducts (additive 1.5): the object ident and declaredness, a
/// non-empty duplicate-free arrow list whose arrows are declared
/// injections into the coproduct object, and the shared status enum.
pub(super) fn check_coproducts(
    schema: &RawSchema,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Vec<Coproduct>, crate::error::ParseError> {
    let mut coproducts = Vec::new();
    for (i, raw) in schema.coproducts.iter().enumerate() {
        let base = format!("/schema/coproducts/{i}");
        check_ident(spans, &format!("{base}/object"), &raw.object)?;
        let object = Ident(raw.object.clone());
        if !objects.contains_key(&object) {
            return Err(spans.semantic(
                &format!("{base}/object"),
                ErrorCode::InvalidCoproduct,
                format!("coproduct names undeclared object `{}`", raw.object),
            ));
        }
        let arrow_names = check_coproduct_arrows(&base, raw, &object, arrows, spans)?;
        let status = check_status(raw.status.as_deref(), &base, spans, "coproduct")?;
        coproducts.push(Coproduct {
            object,
            arrows: arrow_names,
            status,
        });
    }
    // Canonical order: coproduct list order in the file is presentation
    // (design Decision 4).
    coproducts.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(coproducts)
}

/// One coproduct's injection list: non-empty, then each element
/// validated at its own node (ident, declaredness, injection shape,
/// duplicate).
fn check_coproduct_arrows(
    base: &str,
    raw: &RawCoproduct,
    object: &Ident,
    declared: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Vec<Ident>, crate::error::ParseError> {
    if raw.arrows.is_empty() {
        return Err(spans.semantic(
            &format!("{base}/arrows"),
            ErrorCode::InvalidCoproduct,
            "coproduct declares no arrows: a coverage claim needs at least one injection"
                .to_string(),
        ));
    }
    let mut seen = BTreeSet::new();
    let mut names = Vec::with_capacity(raw.arrows.len());
    for (j, name) in raw.arrows.iter().enumerate() {
        names.push(check_coproduct_arrow(
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

/// One injection at its node: an ident naming a declared arrow targeted
/// at the coproduct object and sourced away from it, not repeated within
/// the declaration.
fn check_coproduct_arrow(
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
            ErrorCode::InvalidCoproduct,
            format!("coproduct references undeclared arrow `{name}`"),
        ));
    };
    check_injection(node, name, object, definition, spans)?;
    if !seen.insert(arrow.clone()) {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidCoproduct,
            format!("duplicate arrow `{name}` in one coproduct"),
        ));
    }
    Ok(arrow)
}

/// The injection shape `V_k → C`: the arrow is targeted at the coproduct
/// object and sourced away from it.
fn check_injection(
    node: &str,
    name: &str,
    object: &Ident,
    definition: &Arrow,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    if definition.tgt != *object {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidCoproduct,
            format!(
                "coproduct arrow `{name}` targets `{}`, not the coproduct object `{}`",
                definition.tgt.0, object.0
            ),
        ));
    }
    if definition.src == *object {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidCoproduct,
            format!(
                "coproduct arrow `{name}` is sourced at the coproduct object `{}`; \
                 an injection must come from a variant object",
                object.0
            ),
        ));
    }
    Ok(())
}
