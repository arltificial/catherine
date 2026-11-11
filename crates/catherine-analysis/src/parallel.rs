//! Purpose: parallel-path detection over the schema graph (openspec
//! change `add-bounded-analysis`, task 8) — the discovery mechanism
//! (kata-8 pattern): identical endpoints, no declared equation relating
//! them → `underconstraint` findings labeled `hypothesized`, phrased as
//! the domain question "should these agree?".
//!
//! Responsibilities: enumerate composable paths of length 1..=max over
//! the schema's arrows (deterministic: arrow names in sorted order,
//! level-order extension), group them by their `(src, tgt)` endpoints,
//! and emit one hypothesized finding per unordered pair no declared
//! equation relates — either orientation counts as relating. Never a
//! violation, never a failure: a question.
//!
//! Rationale: the enumeration is bounded by the caller-supplied path
//! cap (a cyclic schema generates infinitely many paths — the cap is
//! what keeps the op total and deterministic, and its default lives in
//! the shared scope-caps config with the README documenting it). Only
//! the *pair existence* is reported; whether the paths *should* agree is
//! a domain decision the tool raises, never answers — `hypothesized` is
//! the epistemic weight, and the witness carries the P2 flattened shape
//! (`case`/`detail` first, the question text last). The
//! `DomainQuestion::UncoveredInputs` arm is P8-reserved and is never
//! produced here (hard constraint 9).

use crate::bounds::{parallel_paths_finding, ScopeCaps};
use catherine_core::findings::Finding;
use catherine_core::model::{Ident, Path, Schema};
use std::collections::{BTreeMap, BTreeSet};

/// Paths in the schema with identical endpoints and no declared equation
/// relating them, up to `max_path_length` arrows each. Findings come out
/// in the taxonomy's total order (sorted).
pub fn underconstraint(schema: &Schema, caps: &ScopeCaps) -> Vec<Finding> {
    let max_path_length = caps.max_path_length;
    // Group composable paths by their endpoints. BTreeMap/BTreeSet
    // discipline: sorted by construction, so the pair enumeration is
    // deterministic.
    let mut by_endpoints: BTreeMap<(Ident, Ident), BTreeSet<Path>> = BTreeMap::new();
    // Level 1: the generating arrows — only when the cap admits paths
    // of length 1 at all. A cap of 0 enumerates nothing: the cap bounds
    // the op from the start, never after the fact.
    let mut frontier = seed_level(schema, max_path_length, &mut by_endpoints);
    // Extend level by level up to the cap. A path may revisit objects
    // (self-loops are legal); the length cap keeps the space finite.
    extend_levels(schema, max_path_length, &mut frontier, &mut by_endpoints);
    // One hypothesis per unordered pair, in sorted path order.
    let mut findings = pair_hypotheses(schema, &by_endpoints);
    findings.sort();
    findings
}

/// Level 1 of the enumeration: the generating arrows, seeded into the
/// endpoint grouping. An empty frontier when the cap admits no paths.
fn seed_level(
    schema: &Schema,
    max_path_length: usize,
    by_endpoints: &mut BTreeMap<(Ident, Ident), BTreeSet<Path>>,
) -> Vec<(Ident, Ident, Vec<Ident>)> {
    if max_path_length == 0 {
        return Vec::new();
    }
    let singles: Vec<(Ident, Ident, Vec<Ident>)> = schema
        .arrows
        .iter()
        .map(|(name, arrow)| (arrow.src.clone(), arrow.tgt.clone(), vec![name.clone()]))
        .collect();
    for (src, tgt, path) in &singles {
        by_endpoints
            .entry((src.clone(), tgt.clone()))
            .or_default()
            .insert(Path(path.clone()));
    }
    singles
}

/// Extend level by level up to the cap, one arrow at a time, in the
/// same level-order and arrow order as before.
fn extend_levels(
    schema: &Schema,
    max_path_length: usize,
    frontier: &mut Vec<(Ident, Ident, Vec<Ident>)>,
    by_endpoints: &mut BTreeMap<(Ident, Ident), BTreeSet<Path>>,
) {
    for _ in 1..max_path_length.max(1) {
        let mut next: Vec<(Ident, Ident, Vec<Ident>)> = Vec::new();
        for (src, current, path) in &*frontier {
            extend_one(schema, src, current, path, by_endpoints, &mut next);
        }
        *frontier = next;
    }
}

/// Extend one frontier path by every arrow that continues it — the
/// extension step of the level-order walk.
fn extend_one(
    schema: &Schema,
    src: &Ident,
    current: &Ident,
    path: &[Ident],
    by_endpoints: &mut BTreeMap<(Ident, Ident), BTreeSet<Path>>,
    next: &mut Vec<(Ident, Ident, Vec<Ident>)>,
) {
    for (name, arrow) in &schema.arrows {
        if &arrow.src != current {
            continue;
        }
        let mut extended = path.to_vec();
        extended.push(name.clone());
        by_endpoints
            .entry((src.clone(), arrow.tgt.clone()))
            .or_default()
            .insert(Path(extended.clone()));
        next.push((src.clone(), arrow.tgt.clone(), extended));
    }
}

/// One hypothesis per unordered pair of same-endpoint paths, in sorted
/// path order — the pairs no declared equation relates.
fn pair_hypotheses(
    schema: &Schema,
    by_endpoints: &BTreeMap<(Ident, Ident), BTreeSet<Path>>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for paths in by_endpoints.values() {
        let ordered: Vec<&Path> = paths.iter().collect();
        findings.extend(unrelated_pairs(&ordered, &schema.equations));
    }
    findings
}

/// The unrelated pairs within one endpoint group, in index order.
fn unrelated_pairs(
    ordered: &[&Path],
    equations: &[catherine_core::model::Equation],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for first in 0..ordered.len() {
        for second in first + 1..ordered.len() {
            let (lhs, rhs) = (ordered[first], ordered[second]);
            if related(equations, lhs, rhs) {
                continue;
            }
            findings.push(parallel_paths_finding(lhs.clone(), rhs.clone()));
        }
    }
    findings
}

/// Whether any declared equation relates the two paths — in either
/// orientation. A related pair is *constrained*; the hypothesis is only
/// for pairs the schema never forces to agree.
fn related(equations: &[catherine_core::model::Equation], lhs: &Path, rhs: &Path) -> bool {
    equations.iter().any(|equation| {
        (&equation.lhs == lhs && &equation.rhs == rhs)
            || (&equation.lhs == rhs && &equation.rhs == lhs)
    })
}
