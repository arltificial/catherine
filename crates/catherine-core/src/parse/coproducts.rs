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

use super::schema::{check_ident, check_status, RawSchema, StatusCheck};
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

/// What one coproduct declaration is checked against: the declared
/// objects and arrows plus the span index. Shared by both call sites
/// (the document schema pass and the schema-map target pass).
pub(super) struct CoproductCtx<'a> {
    pub(super) objects: &'a BTreeMap<Ident, ()>,
    pub(super) arrows: &'a BTreeMap<Ident, Arrow>,
    pub(super) spans: &'a Spans,
}

/// One coproduct's injection checks: the coproduct object, the declared
/// arrows, the span index, and the arrows seen so far within this
/// declaration.
struct InjectionCheck<'a> {
    object: &'a Ident,
    declared: &'a BTreeMap<Ident, Arrow>,
    spans: &'a Spans,
    seen: BTreeSet<Ident>,
}

/// Coproducts (additive 1.5): the object ident and declaredness, a
/// non-empty duplicate-free arrow list whose arrows are declared
/// injections into the coproduct object, and the shared status enum.
pub(super) fn check_coproducts(
    schema: &RawSchema,
    ctx: CoproductCtx,
) -> Result<Vec<Coproduct>, crate::error::ParseError> {
    let mut coproducts = Vec::new();
    for (i, raw) in schema.coproducts.iter().enumerate() {
        coproducts.push(check_coproduct(
            &format!("/schema/coproducts/{i}"),
            raw,
            &ctx,
        )?);
    }
    // Canonical order: coproduct list order in the file is presentation
    // (design Decision 4).
    coproducts.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(coproducts)
}

/// One coproduct at its node: the object's ident and declaredness, its
/// injection list, and its status.
fn check_coproduct(
    base: &str,
    raw: &RawCoproduct,
    ctx: &CoproductCtx,
) -> Result<Coproduct, crate::error::ParseError> {
    let object = coproduct_object(base, raw, ctx)?;
    let mut check = InjectionCheck {
        object: &object,
        declared: ctx.arrows,
        spans: ctx.spans,
        seen: BTreeSet::new(),
    };
    let arrows = check_coproduct_arrows(base, raw, &mut check)?;
    let status = check_status(&StatusCheck {
        status: raw.status.as_deref(),
        base,
        spans: ctx.spans,
        subject: "coproduct",
    })?;
    Ok(Coproduct {
        object,
        arrows,
        status,
    })
}

/// A coproduct's object at its own node: ident well-formedness and
/// declaredness.
fn coproduct_object(
    base: &str,
    raw: &RawCoproduct,
    ctx: &CoproductCtx,
) -> Result<Ident, crate::error::ParseError> {
    check_ident(ctx.spans, &format!("{base}/object"), &raw.object)?;
    let object = Ident(raw.object.clone());
    if !ctx.objects.contains_key(&object) {
        return Err(ctx.spans.semantic(
            &format!("{base}/object"),
            ErrorCode::InvalidCoproduct,
            format!("coproduct names undeclared object `{}`", raw.object),
        ));
    }
    Ok(object)
}

/// One coproduct's injection list: non-empty, then each element
/// validated at its own node (ident, declaredness, injection shape,
/// duplicate).
fn check_coproduct_arrows(
    base: &str,
    raw: &RawCoproduct,
    check: &mut InjectionCheck,
) -> Result<Vec<Ident>, crate::error::ParseError> {
    if raw.arrows.is_empty() {
        return Err(check.spans.semantic(
            &format!("{base}/arrows"),
            ErrorCode::InvalidCoproduct,
            "coproduct declares no arrows: a coverage claim needs at least one injection"
                .to_string(),
        ));
    }
    let names: Vec<Ident> = raw
        .arrows
        .iter()
        .enumerate()
        .map(|(j, name)| check_coproduct_arrow(&format!("{base}/arrows/{j}"), name, check))
        .collect::<Result<_, _>>()?;
    Ok(names)
}

/// One injection at its node: an ident naming a declared arrow targeted
/// at the coproduct object and sourced away from it, not repeated within
/// the declaration.
fn check_coproduct_arrow(
    node: &str,
    name: &str,
    check: &mut InjectionCheck,
) -> Result<Ident, crate::error::ParseError> {
    check_ident(check.spans, node, name)?;
    let arrow = Ident(name.to_string());
    check_injection(node, &arrow, check)?;
    if !check.seen.insert(arrow.clone()) {
        return Err(check.spans.semantic(
            node,
            ErrorCode::InvalidCoproduct,
            format!("duplicate arrow `{name}` in one coproduct"),
        ));
    }
    Ok(arrow)
}

/// The injection shape `V_k → C`: the arrow is declared, targeted at the
/// coproduct object, and sourced away from it.
fn check_injection(
    node: &str,
    arrow: &Ident,
    check: &InjectionCheck,
) -> Result<(), crate::error::ParseError> {
    let Some(definition) = check.declared.get(arrow) else {
        return Err(check.spans.semantic(
            node,
            ErrorCode::InvalidCoproduct,
            format!("coproduct references undeclared arrow `{}`", arrow.0),
        ));
    };
    injection_shape(node, arrow, definition, check)
}

/// The injection shape `V_k → C`: targeted at the coproduct object and
/// sourced away from it.
fn injection_shape(
    node: &str,
    arrow: &Ident,
    definition: &Arrow,
    check: &InjectionCheck,
) -> Result<(), crate::error::ParseError> {
    if definition.tgt != *check.object {
        return Err(check.spans.semantic(
            node,
            ErrorCode::InvalidCoproduct,
            format!(
                "coproduct arrow `{}` targets `{}`, not the coproduct object `{}`",
                arrow.0, definition.tgt.0, check.object.0
            ),
        ));
    }
    if definition.src == *check.object {
        return Err(check.spans.semantic(
            node,
            ErrorCode::InvalidCoproduct,
            format!(
                "coproduct arrow `{}` is sourced at the coproduct object `{}`; \
                 an injection must come from a variant object",
                arrow.0, check.object.0
            ),
        ));
    }
    Ok(())
}
