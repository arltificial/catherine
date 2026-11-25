//! Purpose: the Builder pattern lens (beads `catherine-mx1.5`) — the
//! monoid-accumulation row of
//! `ledger/katas/builder-monoid/`: every emitted product part traces to a
//! declared accumulation step, and the `step_prev` chain stays inside one
//! product (the splice order equation).
//!
//! Responsibilities: one `'static` [`PatternMeta`] matching the
//! disposition row, and `run` reusing the kernel operations the kata's
//! recorded commands ran — `check` for the executed order/trace findings,
//! `rewrite::redundant` for the duplicated-law witness, and
//! `migrate::lost_data` for the schema map's per-row/per-cell lost-data
//! enumeration. The two channels keep their own shapes
//! ([`Finding`] checks and [`MigrationFinding`]s); the lens adds no kind,
//! witness, or heuristic. A `--values` request (this lens has no rule
//! table) or a schema with no accepted composition equation is
//! [`LensOutcome::Unknown`] naming the mismatch, never an ignored flag or
//! an empty success.
//!
//! Rationale: the sealed evidence records exact surfaces, and the lens
//! must not widen them. Equation violations are the law's own claim, so
//! they pass through unchanged. Dangling cells are narrowed to the trace
//! law: only the *lead arrow of an accepted composition equation*, on a
//! row whose other side is fully defined, is a missing traceability link.
//! That keeps the optional part — which no step emits and no accepted
//! equation constrains — outside the law, exactly as the kata records it
//! ("the optional part is not asserted"), while the floor control's
//! surviving part with no step remains a dangling `part_step`. The
//! duplicated step row is deliberately silent: path equations cannot state
//! instance multiplicity, so that row is parity and the lens proves only
//! the duplicated *declaration* derivable (`redundant`, emitted twice by
//! `rewrite::redundant` for the two identical equations).
//!
//! [`Finding`]: catherine_core::findings::Finding
//! [`MigrationFinding`]: catherine_core::migrate::MigrationFinding

use crate::pattern::{LawKind, LensCall, LensFindings, LensOutcome, PatternLens, PatternMeta};
use crate::rewrite;
use catherine_core::checks::check;
use catherine_core::findings::{DanglingWitness, Finding, FindingKind};
use catherine_core::instance::TypedInstance;
use catherine_core::migrate;
use catherine_core::model::{ConstraintStatus, Schema};

/// The disposition row's metadata (law `Order`: the splice order equation).
pub static BUILDER_META: PatternMeta = PatternMeta {
    id: "builder",
    name: "Builder",
    abstraction: "monoid (accumulation)",
    law: LawKind::Order,
    scope: "emitted part traces to a declared step; splice order equation; \
            duplicate-row case is parity",
};

/// The Builder lens: executed trace/order law plus the accumulation map's
/// lost-data enumeration over the built model.
///
/// Stateless and therefore a `static` value shared by the registry.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &BUILDER_META
    }

    fn run(&self, call: LensCall<'_>) -> LensOutcome {
        let LensCall { input, caps } = call;
        // Equation/Δ surface only: this lens has no rule table, so a
        // supplied finite domain cannot bind and must not be ignored.
        if !input.values.is_empty() {
            return LensOutcome::Unknown {
                scope: "`--values` supplied to a lens with no rule table; nothing evaluated"
                    .to_string(),
            };
        }
        let schema = &input.model.schema;
        if !declares_law(schema) {
            return LensOutcome::Unknown {
                scope: "no accepted composition equation; the accumulation law is not declared"
                    .to_string(),
            };
        }
        let typed = TypedInstance::build(schema, &input.model.instance);
        let mut checks = law_checks(schema, &typed);
        checks.extend(rewrite::redundant(schema, caps));
        checks.sort();
        let migrations = migrate::lost_data(input.model).unwrap_or_default();
        LensOutcome::Findings(LensFindings { checks, migrations })
    }
}

/// The accumulation law is declared when at least one accepted equation
/// composes two arrows — the join of the step chain with the step's
/// product (trace) or its predecessor's product (order).
fn declares_law(schema: &Schema) -> bool {
    schema
        .equations
        .iter()
        .any(|equation| equation.status == ConstraintStatus::Accepted && equation.lhs.0.len() >= 2)
}

/// The executed checks narrowed to the law: every equation violation, and
/// only the dangling cells a composition equation's lead arrow claims.
fn law_checks(schema: &Schema, typed: &TypedInstance) -> Vec<Finding> {
    check(schema, typed)
        .into_iter()
        .filter(|finding| match &finding.kind {
            FindingKind::Violation(_) => true,
            FindingKind::Dangling(witness) => traces_a_declared_step(schema, typed, witness),
            _ => false,
        })
        .collect()
}

/// A dangling lead-arrow cell is a lost traceability link when an accepted
/// composition equation otherwise determines its row: the equation's other
/// side composes, so the only missing edge is the declared step it names.
fn traces_a_declared_step(
    schema: &Schema,
    typed: &TypedInstance,
    witness: &DanglingWitness,
) -> bool {
    schema
        .equations
        .iter()
        .filter(|equation| equation.status == ConstraintStatus::Accepted)
        .filter(|equation| equation.lhs.0.first() == Some(&witness.arrow))
        .any(|equation| typed.compose(&equation.rhs, witness.src).is_some())
}
