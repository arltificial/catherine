//! Purpose: schema-diff classification of a model-document schema edit
//! v1 → v2 (openspec change `add-schema-diff`, beads `catherine-dvx`):
//! `accreted` (constraints only added — covariant strengthening of the
//! enforced contract), `weakened` (a constraint is no longer enforced —
//! the contravariant obligation test, exhibited by a witness instance v2
//! accepts and v1 rejects; the agent-drift detector), `strengthened-in-scope` (a mixed edit whose bounded scan
//! witnessed strengthening but found no weakening witness),
//! `equivalent-in-scope` (no separating instance within the bounds),
//! `cosmetic` (no enforced-constraint change — the negative-control
//! silence), or `unknown` (the candidate budget expired).
//!
//! Responsibilities: diff the *enforced* constraint sets — the accepted
//! equations' `(lhs, rhs)` pairs together with the accepted keys'
//! `(object, arrows)` tuples. Rationale, document order, and proposed
//! equations or keys are presentation and never classify an edit. The
//! effect on the instance space is decided by the shared bounded
//! enumeration ([`crate::search::enumerate`], deterministic
//! smallest-first order) under the kernel's own satisfaction predicate
//! ([`crate::search`]'s `is_clean_typed`): a fully resolved candidate
//! accepted by v2 and rejected by v1 is the weakening witness.
//! `weakened` emits exactly one existing-taxonomy finding —
//! `violation`/`executed`, the violated v1 constraint's canonical index
//! and its witness (an equation's row and composed values, or a key's
//! colliding pair); every other class emits zero findings.
//!
//! Rationale: no new epistemic machinery and no new finding kinds. The
//! classification is a report *beside* the findings (an outcome enum
//! plus the added constraints), and the weakening witness is what makes
//! the drift detector actionable. Honesty is the contract: "equivalent"
//! is always *in scope*, with the explored per-object row counts and
//! candidate count echoed; budget expiry is `unknown`, never
//! equivalence; a structural change (objects or arrows differ) is
//! `incomparable` — comparing instance spaces across vocabularies needs
//! a declared schema map (the Δ axis), not a guess. Equivalence is
//! decided over *fully resolved* instances of the bounded shape (the
//! search witness contract); partially defined instances differ only by
//! dangling cells, which are schema-independent when the arrows are
//! equal.

use crate::bounds::{SearchBounds, SearchScope};
use crate::search::{enumerate, is_clean_typed, Enumeration};
use catherine_core::checks;
use catherine_core::findings::{Finding, FindingKind};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{ConstraintStatus, Ident, Instance, Path, Schema};
use std::collections::BTreeMap;

/// One enforced constraint: an accepted equation's two path sides, or an
/// accepted key's object and arrow tuple. Rationale, document order, and
/// proposed status are presentation and deliberately absent — they cannot
/// change the instance space. The variant order is the enforced set's
/// canonical order (equations before keys).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnforcedConstraint {
    /// An accepted equation's two path sides.
    Equation {
        /// The equation's accepted left side.
        lhs: Path,
        /// The equation's accepted right side.
        rhs: Path,
    },
    /// An accepted key's injectivity claim.
    Key {
        /// The key's object.
        object: Ident,
        /// The key's arrows, in declared order.
        arrows: Vec<Ident>,
    },
}

/// The edit classification. Every variant is a report, not a finding;
/// `weakened` additionally carries the violated constraint via
/// [`SchemaDiffReport::findings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaDiffOutcome {
    /// No enforced-constraint change: only rationale, document order, or
    /// proposed equations/keys differ. Silent — zero findings.
    Cosmetic,
    /// Every v1-enforced constraint is still enforced and v2 adds more:
    /// a structural strengthening, decided without bounds.
    Accreted {
        /// The constraints v2 enforces beyond v1, in canonical order.
        added: Vec<EnforcedConstraint>,
    },
    /// A v1-enforced constraint is no longer enforced, witnessed by a
    /// fully resolved instance v2 accepts and v1 rejects.
    Weakened {
        /// The witness instance (v2-clean, v1-violating).
        witness: Instance,
    },
    /// A mixed edit (dropped and added constraints) whose bounded scan
    /// found no weakening witness but did witness strengthening. Never
    /// rounded into `accreted` — the scope is part of the claim.
    StrengthenedInScope {
        /// The constraints v2 enforces beyond v1, in canonical order.
        added: Vec<EnforcedConstraint>,
        /// The exact scope the bounded scan exhausted.
        scope: SearchScope,
    },
    /// No separating instance exists within the explored bounded shape:
    /// the instance space is the same *in scope* — never a global claim.
    EquivalentInScope {
        /// The exact scope the bounded scan exhausted.
        scope: SearchScope,
    },
    /// The candidate budget expired before exhaustion: a scope-bounded
    /// non-answer, never an equivalence claim.
    Unknown {
        /// The exact scope explored.
        scope: SearchScope,
    },
    /// The schemas' objects or arrows differ: instance spaces are not
    /// comparable without a declared schema map, so no classification
    /// is claimed.
    Incomparable,
}

/// The classification report: the caller's bound restated verbatim, the
/// outcome, and the findings the classification emits (exactly one
/// `violation` for `weakened`; empty for every other class).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDiffReport {
    /// The bound the bounded scan ran under (restated, spec scenario).
    pub bound: SearchBounds,
    /// The classified edit.
    pub outcome: SchemaDiffOutcome,
    /// The emitted findings (zero except for `weakened`).
    pub findings: Vec<Finding>,
}

/// Classify the edit `v1 → v2` under the caller-supplied bounds. See the
/// module docs for the contract. The bound only orders the bounded scan
/// for edits that drop an enforced constraint; `cosmetic`, `accreted`,
/// and `incomparable` are structural and need no search.
pub fn diff(v1: &Schema, v2: &Schema, bound: &SearchBounds) -> SchemaDiffReport {
    if v1.objects != v2.objects || v1.arrows != v2.arrows {
        return report(bound, SchemaDiffOutcome::Incomparable, Vec::new());
    }
    let left = enforced(v1);
    let right = enforced(v2);
    let added = surplus(&right, &left);
    let dropped = surplus(&left, &right);
    if dropped.is_empty() {
        let outcome = if added.is_empty() {
            SchemaDiffOutcome::Cosmetic
        } else {
            SchemaDiffOutcome::Accreted { added }
        };
        return report(bound, outcome, Vec::new());
    }
    classify_dropping(v1, v2, bound, added)
}

/// The bounded scan for edits that no longer enforce a constraint: first
/// `v2`-clean and `v1`-dirty candidate is the weakening witness; if the
/// space exhausts without one, the scan reports equivalence — or, when a
/// strengthening witness was seen, the mixed `strengthened-in-scope`
/// class instead of overclaiming equivalence.
fn classify_dropping(
    v1: &Schema,
    v2: &Schema,
    bound: &SearchBounds,
    added: Vec<EnforcedConstraint>,
) -> SchemaDiffReport {
    // Equal objects and arrows guarantee one table shape for both
    // schemas: build once and evaluate both constraint sets on it.
    let mut strengthened = false;
    let scanned = enumerate(v1, bound, |instance| {
        let typed = TypedInstance::build(v1, instance);
        let v1_clean = is_clean_typed(v1, &typed);
        let v2_clean = is_clean_typed(v2, &typed);
        if v2_clean && !v1_clean {
            return violation(v1, &typed).map(|finding| (instance.clone(), finding));
        }
        if v1_clean && !v2_clean {
            strengthened = true;
        }
        None
    });
    match scanned {
        Enumeration::Found((witness, finding)) => report(
            bound,
            SchemaDiffOutcome::Weakened { witness },
            vec![finding],
        ),
        Enumeration::Exhausted { scope } => {
            let outcome = if strengthened {
                SchemaDiffOutcome::StrengthenedInScope { added, scope }
            } else {
                SchemaDiffOutcome::EquivalentInScope { scope }
            };
            report(bound, outcome, Vec::new())
        }
        Enumeration::Unknown { scope } => {
            report(bound, SchemaDiffOutcome::Unknown { scope }, Vec::new())
        }
    }
}

/// The report constructor (one site, so the bound is restated verbatim).
fn report(
    bound: &SearchBounds,
    outcome: SchemaDiffOutcome,
    findings: Vec<Finding>,
) -> SchemaDiffReport {
    SchemaDiffReport {
        bound: bound.clone(),
        outcome,
        findings,
    }
}

/// The enforced constraints as a multiset: accepted equations'
/// `(lhs, rhs)` pairs and accepted keys' `(object, arrows)` tuples, with
/// duplicate declarations counted (presentation can repeat a constraint;
/// the instance space cannot distinguish the copies). Proposed keys are
/// presentation, exactly like proposed equations (design Decision 5).
fn enforced(schema: &Schema) -> BTreeMap<EnforcedConstraint, usize> {
    let mut multiset = BTreeMap::new();
    for equation in &schema.equations {
        if equation.status == ConstraintStatus::Accepted {
            let enforced = EnforcedConstraint::Equation {
                lhs: equation.lhs.clone(),
                rhs: equation.rhs.clone(),
            };
            *multiset.entry(enforced).or_insert(0) += 1;
        }
    }
    for key in &schema.keys {
        if key.status == ConstraintStatus::Accepted {
            let enforced = EnforcedConstraint::Key {
                object: key.object.clone(),
                arrows: key.arrows.clone(),
            };
            *multiset.entry(enforced).or_insert(0) += 1;
        }
    }
    multiset
}

/// The entries of `left` beyond `right`, in canonical order (multiset
/// difference — duplicates subtract one for one).
fn surplus(
    left: &BTreeMap<EnforcedConstraint, usize>,
    right: &BTreeMap<EnforcedConstraint, usize>,
) -> Vec<EnforcedConstraint> {
    let mut difference = Vec::new();
    for (constraint, &count) in left {
        let missing = count.saturating_sub(right.get(constraint).copied().unwrap_or(0));
        difference.extend(std::iter::repeat_n(constraint.clone(), missing));
    }
    difference
}

/// The first violation-shaped finding the v1 walk emits on the witness
/// (the candidate is fully resolved, so dangling cannot occur).
fn violation(schema: &Schema, typed: &TypedInstance) -> Option<Finding> {
    checks::check(schema, typed)
        .into_iter()
        .find(|finding| matches!(finding.kind, FindingKind::Violation(_)))
}
