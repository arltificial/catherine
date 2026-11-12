//! Purpose: normal-form rewriting for path equivalence (openspec change
//! `add-bounded-analysis`, task 7) — redundancy is proven by derivation
//! or not claimed at all (spec: "Derivable path").
//!
//! Responsibilities: for each declared accepted equation, rewrite both of
//! its sides under the *other* accepted equations (oriented lhs→rhs,
//! leftmost-outermost scan in canonical order) and report a `redundant`
//! finding — witness `derivable` = the equation's declared side,
//! `reducing` = the common normal form — exactly when both sides reach
//! the same normal form. Absent a proof, no redundancy claim is emitted.
//!
//! Rationale and documented limits (the README's confluence section
//! starts here): the rewriting theory is the *declared accepted
//! equation set only* — derivations rest on accepted declarations, and
//! proposed equations are hypotheses that prove nothing. Rules are
//! oriented lhs→rhs; an equation whose rhs contains its lhs as a
//! contiguous subpath is skipped (its orientation cannot terminate —
//! conservatively, no proof is drawn from it). The scan is
//! deterministic (canonical equation order, leftmost-outermost, restart
//! after each rewrite) so identical inputs always produce identical
//! normal forms; the engine is NOT a confluence procedure — no critical-
//! pair completion, no Knuth–Bendix — so a normal form may depend on the
//! rule order when rules overlap. The honest consequence: rewriting can
//! *under*-report redundancy (it may miss a derivability that a
//! confluent system would find) but never *over*-report it — every
//! emitted `redundant` finding carries a witnessed derivation. A rewrite
//! step cap bounds every derivation; a cap hit is not a proof and emits
//! nothing. These limits are restated in `README.md`.

use crate::bounds::{redundant_finding, ScopeCaps};
use catherine_core::findings::Finding;
use catherine_core::model::{ConstraintStatus, Equation, Ident, Path, Schema};

/// One oriented rewrite rule: `lhs → rhs`.
struct Rule {
    lhs: Vec<Ident>,
    rhs: Vec<Ident>,
}

/// The oriented rules derivable from the accepted equations except
/// `skip` — the rewriting theory of "derivable from the others". An
/// equation whose rhs contains its lhs is skipped: rewriting toward it
/// cannot terminate, so no proof is drawn from it.
fn oriented_theory(equations: &[Equation], skip: usize) -> Vec<Rule> {
    equations
        .iter()
        .enumerate()
        .filter(|(index, equation)| {
            *index != skip
                && equation.status == ConstraintStatus::Accepted
                && !contains(&equation.rhs.0, &equation.lhs.0)
        })
        .map(|(_, equation)| Rule {
            lhs: equation.lhs.0.clone(),
            rhs: equation.rhs.0.clone(),
        })
        .collect()
}

/// Whether `needle` occurs in `haystack` as a contiguous subpath.
fn contains(haystack: &[Ident], needle: &[Ident]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// One leftmost-outermost rewrite step: the first position where any
/// rule (canonical order) applies, its lhs spliced out and its rhs
/// spliced in. `None` = the path is already in normal form.
fn rewrite_step(path: &[Ident], rules: &[Rule]) -> Option<Vec<Ident>> {
    for position in 0..path.len() {
        for rule in rules {
            if path[position..].starts_with(&rule.lhs) {
                let mut next = path[..position].to_vec();
                next.extend_from_slice(&rule.rhs);
                next.extend_from_slice(&path[position + rule.lhs.len()..]);
                return Some(next);
            }
        }
    }
    None
}

/// Rewrite `path` to its normal form under `rules`. `None` = the step
/// cap was hit: no normal form was established, so nothing is proven.
fn normal_form(path: &Path, rules: &[Rule], max_steps: usize) -> Option<Vec<Ident>> {
    let mut current = path.0.clone();
    let mut steps = 0;
    loop {
        match rewrite_step(&current, rules) {
            None => return Some(current),
            Some(next) => {
                steps += 1;
                if steps > max_steps {
                    return None;
                }
                current = next;
            }
        }
    }
}

/// Report every declared accepted equation whose two sides rewrite to
/// the same normal form under the other accepted equations. Findings
/// come out in the taxonomy's total order (sorted). This also flags the
/// proposal's third bullet: declared equations derivable from the
/// others — each accepted equation is itself a candidate.
pub fn redundant(schema: &Schema, caps: &ScopeCaps) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, equation) in schema.equations.iter().enumerate() {
        if equation.status != ConstraintStatus::Accepted {
            continue;
        }
        let theory = oriented_theory(&schema.equations, index);
        if theory.is_empty() {
            // No other accepted rules: a derivation from nothing proves
            // only literal identity, which is not a redundancy claim.
            continue;
        }
        let (Some(lhs_nf), Some(rhs_nf)) = (
            normal_form(&equation.lhs, &theory, caps.max_rewrite_steps),
            normal_form(&equation.rhs, &theory, caps.max_rewrite_steps),
        ) else {
            // A cap hit anywhere in the derivation: not a proof, no claim.
            continue;
        };
        if lhs_nf == rhs_nf {
            findings.push(redundant_finding(equation.lhs.clone(), Path(lhs_nf)));
        }
    }
    findings.sort();
    findings
}
