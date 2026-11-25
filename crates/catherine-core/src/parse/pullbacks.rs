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

use super::schema::{check_ident, check_status, RawSchema, StatusCheck};
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

/// The declared objects and arrows plus the span source, shared by every
/// pullback declaration of one schema pass.
pub(super) struct PullbackCtx<'a> {
    pub(super) objects: &'a BTreeMap<Ident, ()>,
    pub(super) arrows: &'a BTreeMap<Ident, Arrow>,
    pub(super) spans: &'a Spans,
}

/// One declaration's check: its base JSON pointer and the repeated-arrow
/// set (per declaration) over the shared context.
struct PullbackCheck<'a> {
    ctx: &'a PullbackCtx<'a>,
    base: String,
    seen: BTreeSet<Ident>,
}

/// Pullbacks (additive 1.6): the apex ident and declaredness, exactly two
/// apex-sourced and composable legs, exactly two same-target cospan arrows, no
/// repeated arrow name, and the shared status enum.
pub(super) fn check_pullbacks(
    schema: &RawSchema,
    ctx: PullbackCtx,
) -> Result<Vec<Pullback>, crate::error::ParseError> {
    let mut pullbacks = Vec::new();
    for (i, raw) in schema.pullbacks.iter().enumerate() {
        let mut check = PullbackCheck {
            ctx: &ctx,
            base: format!("/schema/pullbacks/{i}"),
            seen: BTreeSet::new(),
        };
        pullbacks.push(check.check_one(raw)?);
    }
    // Canonical order: pullback list order in the file is presentation
    // (design Decision 4).
    pullbacks.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(pullbacks)
}

impl PullbackCheck<'_> {
    /// One declaration: apex, arity, elements, composability, status.
    fn check_one(&mut self, raw: &RawPullback) -> Result<Pullback, crate::error::ParseError> {
        let apex_at = format!("{}/apex", self.base);
        check_ident(self.ctx.spans, &apex_at, &raw.apex)?;
        let apex = Ident(raw.apex.clone());
        if !self.ctx.objects.contains_key(&apex) {
            return Err(self.ctx.spans.semantic(
                &apex_at,
                ErrorCode::InvalidPullback,
                format!("pullback names undeclared object `{}`", raw.apex),
            ));
        }
        self.check_arity(raw)?;
        let legs = self.check_legs(raw, &apex)?;
        let cospan = self.check_cospan(raw)?;
        self.check_cospan_targets(&cospan)?;
        self.check_composability(&legs, &cospan)?;
        let status = check_status(&StatusCheck {
            status: raw.status.as_deref(),
            base: &self.base,
            spans: self.ctx.spans,
            subject: "pullback",
        })?;
        Ok(Pullback {
            apex,
            legs,
            cospan,
            status,
        })
    }

    /// The square's arity: exactly two legs and two cospan arrows, checked
    /// before any element so a wrong-length list is one labeled defect
    /// rather than a cascade of position checks.
    fn check_arity(&self, raw: &RawPullback) -> Result<(), crate::error::ParseError> {
        if raw.legs.len() != 2 {
            return Err(self.ctx.spans.semantic(
                &format!("{}/legs", self.base),
                ErrorCode::InvalidPullback,
                format!(
                    "pullback declares {} leg(s): a pullback square has exactly two legs",
                    raw.legs.len()
                ),
            ));
        }
        if raw.cospan.len() != 2 {
            return Err(self.ctx.spans.semantic(
                &format!("{}/cospan", self.base),
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
        &mut self,
        raw: &RawPullback,
        apex: &Ident,
    ) -> Result<Vec<Ident>, crate::error::ParseError> {
        let mut legs = Vec::with_capacity(raw.legs.len());
        for (j, name) in raw.legs.iter().enumerate() {
            let node = format!("{}/legs/{j}", self.base);
            let leg = self.check_arrow(&node, name)?;
            let definition = &self.ctx.arrows[&leg];
            if definition.src != *apex {
                return Err(self.ctx.spans.semantic(
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
    fn check_cospan(&mut self, raw: &RawPullback) -> Result<Vec<Ident>, crate::error::ParseError> {
        let mut cospan = Vec::with_capacity(raw.cospan.len());
        for (j, name) in raw.cospan.iter().enumerate() {
            let node = format!("{}/cospan/{j}", self.base);
            cospan.push(self.check_arrow(&node, name)?);
        }
        Ok(cospan)
    }

    /// One arrow at its node: an ident naming a declared arrow, not
    /// repeated anywhere in the declaration (the legs and cospan together).
    fn check_arrow(&mut self, node: &str, name: &str) -> Result<Ident, crate::error::ParseError> {
        check_ident(self.ctx.spans, node, name)?;
        let arrow = Ident(name.to_string());
        if !self.ctx.arrows.contains_key(&arrow) {
            return Err(self.ctx.spans.semantic(
                node,
                ErrorCode::InvalidPullback,
                format!("pullback references undeclared arrow `{name}`"),
            ));
        }
        if !self.seen.insert(arrow.clone()) {
            return Err(self.ctx.spans.semantic(
                node,
                ErrorCode::InvalidPullback,
                format!("duplicate arrow `{name}` in one pullback"),
            ));
        }
        Ok(arrow)
    }

    /// The cospan's shared target: `tgt f = tgt g`, the second cospan
    /// element's node carrying the mismatch.
    fn check_cospan_targets(&self, cospan: &[Ident]) -> Result<(), crate::error::ParseError> {
        let first = &self.ctx.arrows[&cospan[0]];
        let second = &self.ctx.arrows[&cospan[1]];
        if first.tgt != second.tgt {
            return Err(self.ctx.spans.semantic(
                &format!("{}/cospan/1", self.base),
                ErrorCode::InvalidPullback,
                format!(
                    "pullback cospan arrows `{}` and `{}` target different objects `{}` and `{}`",
                    cospan[0].0, cospan[1].0, first.tgt.0, second.tgt.0
                ),
            ));
        }
        Ok(())
    }

    /// Positional composability: `leg[i]` targets the source of
    /// `cospan[i]`, so both composed paths are typed and the kernel check
    /// may index each cospan table by the paired leg's target row. The
    /// offending leg carries the defect.
    fn check_composability(
        &self,
        legs: &[Ident],
        cospan: &[Ident],
    ) -> Result<(), crate::error::ParseError> {
        for (i, (leg, cos)) in legs.iter().zip(cospan).enumerate() {
            let leg_arrow = &self.ctx.arrows[leg];
            let cospan_arrow = &self.ctx.arrows[cos];
            if leg_arrow.tgt != cospan_arrow.src {
                return Err(self.ctx.spans.semantic(
                    &format!("{}/legs/{i}", self.base),
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
}
