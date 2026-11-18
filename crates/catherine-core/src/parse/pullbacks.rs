//! Purpose: the pullback parse pass (additive 1.6, openspec change
//! `add-pullback-checks`). Split out of `schema.rs` so both files stay
//! under the size gate.
//!
//! Responsibilities: validate `schema.pullbacks` — the apex's ident and
//! declaredness, exactly two legs and two cospan arrows (arity checked
//! before the elements), every arrow declared and not repeated across the
//! four positions, each leg sourced at the apex, each leg targeting the
//! source of the cospan arrow in its position, the two cospan arrows
//! sharing a target, and the shared status enum — and store the
//! declarations in canonical `(apex, legs, cospan, status)` order. Every
//! defect is `invalid_pullback` at its offending node; a malformed
//! declaration is never repaired or dropped (the `invalid_key` /
//! `invalid_coproduct` discipline, design Decision 1).
//!
//! Rationale: one declaration grammar with keys and coproducts means one
//! parse discipline and one canonicalization path. Because the parse
//! guarantees the composed paths are type-composable — two apex-sourced
//! legs, each targeting the source of its paired cospan arrow, over a
//! same-target cospan — commutes/covers/unique are decidable over the
//! instance tables alone, with each cospan table indexed by its paired
//! leg's target row.

use super::schema::{check_ident, check_status, RawSchema};
use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Arrow, Ident, Pullback};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// One raw pullback declaration (format 1.6): the apex, its two legs, the
/// two cospan arrows, and the optional status (absent means accepted —
/// the equation, key, and coproduct rule).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPullback {
    pub(super) apex: String,
    pub(super) legs: Vec<String>,
    pub(super) cospan: Vec<String>,
    #[serde(default)]
    pub(super) status: Option<String>,
}

/// Pullbacks (additive 1.6): the apex ident and declaredness, exactly two
/// apex-sourced and composable legs, exactly two cospan arrows sharing a
/// target, no repeated arrow name, and the shared status enum.
pub(super) fn check_pullbacks(
    schema: &RawSchema,
    objects: &BTreeMap<Ident, ()>,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Vec<Pullback>, crate::error::ParseError> {
    let mut pullbacks = Vec::new();
    for (i, raw) in schema.pullbacks.iter().enumerate() {
        let base = format!("/schema/pullbacks/{i}");
        check_ident(spans, &format!("{base}/apex"), &raw.apex)?;
        let apex = Ident(raw.apex.clone());
        if !objects.contains_key(&apex) {
            return Err(spans.semantic(
                &format!("{base}/apex"),
                ErrorCode::InvalidPullback,
                format!("pullback names undeclared object `{}`", raw.apex),
            ));
        }
        check_arity(&base, raw, spans)?;
        let mut seen = BTreeSet::new();
        let legs = check_legs(&base, raw, &apex, arrows, &mut seen, spans)?;
        let cospan = check_cospan(&base, raw, arrows, &mut seen, spans)?;
        check_cospan_targets(&base, &cospan, arrows, spans)?;
        check_composability(&base, &legs, &cospan, arrows, spans)?;
        let status = check_status(raw.status.as_deref(), &base, spans, "pullback")?;
        pullbacks.push(Pullback {
            apex,
            legs,
            cospan,
            status,
        });
    }
    // Canonical order: pullback list order in the file is presentation
    // (design Decision 4).
    pullbacks.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(pullbacks)
}

/// The square's arity: exactly two legs and two cospan arrows, checked
/// before any element so a wrong-length list is one labeled defect rather
/// than a cascade of position checks.
fn check_arity(
    base: &str,
    raw: &RawPullback,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    if raw.legs.len() != 2 {
        return Err(spans.semantic(
            &format!("{base}/legs"),
            ErrorCode::InvalidPullback,
            format!(
                "pullback declares {} leg(s): a pullback square has exactly two legs",
                raw.legs.len()
            ),
        ));
    }
    if raw.cospan.len() != 2 {
        return Err(spans.semantic(
            &format!("{base}/cospan"),
            ErrorCode::InvalidPullback,
            format!(
                "pullback declares {} cospan arrow(s): a pullback square has exactly two \
                 cospan arrows",
                raw.cospan.len()
            ),
        ));
    }
    Ok(())
}

/// The legs: each validated at its own node (ident, declaredness,
/// duplicate) and required to source at the apex.
fn check_legs(
    base: &str,
    raw: &RawPullback,
    apex: &Ident,
    declared: &BTreeMap<Ident, Arrow>,
    seen: &mut BTreeSet<Ident>,
    spans: &Spans,
) -> Result<Vec<Ident>, crate::error::ParseError> {
    let mut legs = Vec::with_capacity(raw.legs.len());
    for (j, name) in raw.legs.iter().enumerate() {
        let node = format!("{base}/legs/{j}");
        let leg = check_arrow(&node, name, declared, seen, spans)?;
        let definition = &declared[&leg];
        if definition.src != *apex {
            return Err(spans.semantic(
                &node,
                ErrorCode::InvalidPullback,
                format!(
                    "pullback leg `{name}` is sourced at `{}`, not at the apex `{}`",
                    definition.src.0, apex.0
                ),
            ));
        }
        legs.push(leg);
    }
    Ok(legs)
}

/// The cospan arrows: each validated at its own node (ident,
/// declaredness, duplicate). Their targets are checked together, after
/// both elements are known declared.
fn check_cospan(
    base: &str,
    raw: &RawPullback,
    declared: &BTreeMap<Ident, Arrow>,
    seen: &mut BTreeSet<Ident>,
    spans: &Spans,
) -> Result<Vec<Ident>, crate::error::ParseError> {
    let mut cospan = Vec::with_capacity(raw.cospan.len());
    for (j, name) in raw.cospan.iter().enumerate() {
        cospan.push(check_arrow(
            &format!("{base}/cospan/{j}"),
            name,
            declared,
            seen,
            spans,
        )?);
    }
    Ok(cospan)
}

/// One arrow at its node: an ident naming a declared arrow, not repeated
/// anywhere in the declaration (the legs and cospan together).
fn check_arrow(
    node: &str,
    name: &str,
    declared: &BTreeMap<Ident, Arrow>,
    seen: &mut BTreeSet<Ident>,
    spans: &Spans,
) -> Result<Ident, crate::error::ParseError> {
    check_ident(spans, node, name)?;
    let arrow = Ident(name.to_string());
    if !declared.contains_key(&arrow) {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidPullback,
            format!("pullback references undeclared arrow `{name}`"),
        ));
    }
    if !seen.insert(arrow.clone()) {
        return Err(spans.semantic(
            node,
            ErrorCode::InvalidPullback,
            format!("duplicate arrow `{name}` in one pullback"),
        ));
    }
    Ok(arrow)
}

/// The cospan's shared target: `tgt f = tgt g`, the second cospan
/// element's node carrying the mismatch.
fn check_cospan_targets(
    base: &str,
    cospan: &[Ident],
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    let first = &arrows[&cospan[0]];
    let second = &arrows[&cospan[1]];
    if first.tgt != second.tgt {
        return Err(spans.semantic(
            &format!("{base}/cospan/1"),
            ErrorCode::InvalidPullback,
            format!(
                "pullback cospan arrows `{}` and `{}` target different objects `{}` and `{}`",
                cospan[0].0, cospan[1].0, first.tgt.0, second.tgt.0
            ),
        ));
    }
    Ok(())
}

/// Positional composability: `leg[i]` targets the source of `cospan[i]`,
/// so both composed paths are typed and the kernel check may index each
/// cospan table by the paired leg's target row. The offending leg carries
/// the defect.
fn check_composability(
    base: &str,
    legs: &[Ident],
    cospan: &[Ident],
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    for (i, (leg, cos)) in legs.iter().zip(cospan).enumerate() {
        let leg_arrow = &arrows[leg];
        let cospan_arrow = &arrows[cos];
        if leg_arrow.tgt != cospan_arrow.src {
            return Err(spans.semantic(
                &format!("{base}/legs/{i}"),
                ErrorCode::InvalidPullback,
                format!(
                    "pullback leg `{}` targets `{}`, not the source `{}` of its paired cospan \
                     arrow `{}`",
                    leg.0, leg_arrow.tgt.0, cospan_arrow.src.0, cos.0
                ),
            ));
        }
    }
    Ok(())
}
