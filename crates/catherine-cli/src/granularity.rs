//! Purpose: the stored-granularity lint (ticket `catherine-kw9.3`,
//! docs/USING.md's "Arrow granularity" rule, status EXECUTED) —
//! computes the findings over one loaded model and lets the shared
//! check path emit them like any other kind.
//!
//! Responsibilities: [`findings`] implements exactly two cases against
//! the instance's total row count `rows` (sum over every declared
//! object):
//!
//! 1. **Arrow cap** — `arrows > ceil(4.5·√rows)` fires an
//!    `OverCap` finding naming every declared arrow.
//! 2. **Stored column** — every arrow should reference a stored,
//!    independently writable column/table via the additive
//!    `schema.stored` map; when the cap gate trips *and* more than a
//!    third of the arrows lack a `stored` entry, an `Unnamed` finding
//!    names exactly those arrows (canonical id order).
//!
//! Rationale: the wave proposed a bare `√rows` cap, but calibration
//! against the kata corpus (the pre-registered negative controls —
//! `fixtures/katas/`) refuted it: the densest control
//! (`facade/floor`'s 10-arrow, 5-row variant) sits at 4.47·√rows, so the
//! bare bound fired dozens of ordinary well-grained models — the exact
//! noise failure that motivated the rule (the bogart pilot's 2/69).
//! The cap is therefore calibrated at **4.5·√rows** with the kata corpus
//! as the fixed negative-control set; the stored-column test is gated
//! on the same cap so a small, fully-mapped schema is never nagged.
//! Both findings share one exit class (8, below every kernel class) —
//! the lint questions the *model's shape*, not any row. Thresholds are
//! recalibrated against the controls only — never special-cased by
//! filename. A document with zero instance rows carries no calibrating
//! denominator and stays silent.

use catherine_core::findings::{
    EpistemicLabel, Finding, FindingKind, GranularityCase, GranularityWitness,
};
use catherine_core::model::Model;

/// The calibrated cap multiplier over the wave's proposed `√rows` bound.
/// See the module rationale: the kata corpus fixes the floor at ~4.47
/// (facade/floor's 10-arrow, 5-row variant), so 4.5 clears every
/// negative control with a hair of margin.
const CAP_OVER_SQRT: f64 = 4.5;

/// The stored-column finding's fraction bound: fire when `unnamed`
/// exceeds one third of the declared arrows (`3·unnamed > arrows`).
fn unnamed_shares_majority(arrows: usize, unnamed: usize) -> bool {
    3 * unnamed > arrows
}

/// The granularity findings over one loaded model, in kind order (cap
/// before unnamed — the witness's canonical order). At most two.
pub(crate) fn findings(model: &Model) -> Vec<Finding> {
    let arrows = model.schema.arrows.len();
    let rows: usize = model.instance.rows.values().map(Vec::len).sum();
    // No rows: no calibrating denominator — the lint stays silent.
    if rows == 0 || arrows == 0 {
        return Vec::new();
    }
    let cap = (CAP_OVER_SQRT * (rows as f64).sqrt()).ceil() as usize;
    if arrows <= cap {
        return Vec::new();
    }
    let mut out = vec![over_cap_finding(model, cap, rows)];
    if let Some(finding) = unnamed_finding(model, arrows) {
        out.push(finding);
    }
    out
}

/// The cap finding: every declared arrow named, canonical id order.
fn over_cap_finding(model: &Model, cap: usize, rows: usize) -> Finding {
    let all: Vec<_> = model.schema.arrows.keys().cloned().collect();
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Granularity(GranularityWitness {
            case: GranularityCase::OverCap { cap, rows },
            arrows: all,
        }),
    }
}

/// The stored-column finding when more than a third of the arrows lack
/// a `stored` entry (canonical id order); silence below that share.
fn unnamed_finding(model: &Model, arrows: usize) -> Option<Finding> {
    let unnamed: Vec<_> = model
        .schema
        .arrows
        .keys()
        .filter(|arrow| !model.schema.stored.contains_key(arrow))
        .cloned()
        .collect();
    if !unnamed_shares_majority(arrows, unnamed.len()) {
        return None;
    }
    Some(Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Granularity(GranularityWitness {
            case: GranularityCase::Unnamed {
                stored: arrows - unnamed.len(),
            },
            arrows: unnamed,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::model::{Arrow, Ident, RowId, Schema};
    use std::collections::BTreeMap;

    /// A schema with `n` arrows over `r` rows spread across one object.
    fn schema(n: usize, r: usize) -> Model {
        let mut arrows = BTreeMap::new();
        for i in 0..n {
            arrows.insert(
                Ident(format!("a{i}")),
                Arrow {
                    src: Ident("x".into()),
                    tgt: Ident("x".into()),
                },
            );
        }
        Model {
            schema: Schema {
                objects: BTreeMap::from([(Ident("x".into()), ())]),
                arrows,
                ..Schema::default()
            },
            instance: catherine_core::model::Instance {
                rows: BTreeMap::from([(Ident("x".into()), (0..r).map(RowId).collect())]),
                values: BTreeMap::new(),
            },
            ..Model::default()
        }
    }

    #[test]
    fn under_the_cap_is_silent() {
        // facade/floor's densest control shape: 10 arrows, 5 rows
        // (ratio 4.47·√rows — the calibration floor).
        assert!(findings(&schema(10, 5)).is_empty());
        // The wrapper-laws control: 15 arrows, 21 rows.
        assert!(findings(&schema(15, 21)).is_empty());
        // The re-model's target shape: 2 arrows, 293 rows.
        assert!(findings(&schema(2, 293)).is_empty());
        // No rows: no denominator, silent.
        assert!(findings(&schema(13, 0)).is_empty());
    }

    #[test]
    fn over_the_cap_fires_both_findings() {
        // 13 arrows over 6 rows: cap ceil(4.5·√6) = 12.
        let out = findings(&schema(13, 6));
        assert_eq!(out.len(), 2, "cap and unnamed both fire");
        let FindingKind::Granularity(w) = &out[0].kind else {
            panic!("granularity kind");
        };
        assert_eq!(w.case, GranularityCase::OverCap { cap: 12, rows: 6 });
        // Every arrow named in the cap witness, canonical order.
        assert_eq!(w.arrows.len(), 13);
        let FindingKind::Granularity(w) = &out[1].kind else {
            panic!("granularity kind");
        };
        // Nothing stored: the unnamed test fires, naming all 13.
        assert_eq!(w.case, GranularityCase::Unnamed { stored: 0 });
        assert_eq!(w.arrows.len(), 13);
    }

    #[test]
    fn a_third_unnamed_stays_silent_below_the_cap() {
        // 6 arrows, 100 rows: under the cap (ceil(40)), no finding even
        // though nothing is stored — the stored test is cap-gated.
        assert!(findings(&schema(6, 100)).is_empty());
    }
}
