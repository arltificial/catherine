//! Purpose: the schema-diff payload (openspec change `add-schema-diff`,
//! task 7) — the `diff` envelope's `data` and the human rendering of a
//! classification.
//!
//! Responsibilities: build the one diff-envelope path (outcome string,
//! bound restated, scope when the claim is bounded, the weakening
//! witness instance, the added enforced constraints, and the findings
//! verbatim) and render the human lines: the classification head, the
//! §5 finding block for `weakened`, and one `added:` line per accreted
//! constraint (an equation as `lhs = rhs`, a key as `key Object(a∘b)`,
//! a coproduct as `coproduct C(i1∘i2)`, a pullback as
//! `pullback P(p1∘p2 → f∘g)`).
//!
//! Rationale: one envelope shape (the genesis `Check` kind over the
//! analysis payloads); the outcome string names the epistemic state and
//! is never collapsed into "clean" — `equivalent-in-scope` and
//! `unknown` carry their scope, `cosmetic` says exactly zero findings.
//! The weakening finding is rendered by the shared §5 renderer against
//! the v1 schema (the side that enforced the equation), so its equation
//! index resolves to the exact lost constraint.

use super::analysis::{unknown_hint, BoundView, InstanceView, ScopeView};
use super::render_one;
use super::CLI_VERSION;
use catherine_analysis::bounds::SearchScope;
use catherine_analysis::schema_diff::{EnforcedConstraint, SchemaDiffOutcome, SchemaDiffReport};
use catherine_core::findings::Finding;
use catherine_core::model::Schema;
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use serde::Serialize;

/// One enforced constraint as the payload spells it. Untagged: the
/// equation shape stays exactly `{"lhs": [...], "rhs": [...]}` (the
/// pre-key payload is byte-identical), a key adds
/// `{"object": ..., "arrows": [...]}`, and the coproduct and pullback
/// arms add their unconditional `kind` discriminator.
#[derive(Serialize)]
#[serde(untagged)]
pub enum ConstraintView {
    /// An accepted equation's two arrow-name sequences.
    Equation {
        /// The left side's arrows, in composition order.
        lhs: Vec<String>,
        /// The right side's arrows, in composition order.
        rhs: Vec<String>,
    },
    /// An accepted key's object and arrow-name sequence.
    Key {
        /// The key's object.
        object: String,
        /// The key's arrows, in composition order.
        arrows: Vec<String>,
    },
    /// An accepted coproduct's object and injection-name sequence. The
    /// unconditional `kind` discriminator keeps the untagged payload
    /// distinct from a key's `{object, arrows}` shape.
    Coproduct {
        /// The coproduct's object.
        object: String,
        /// The injection arrows, in declared order.
        arrows: Vec<String>,
        /// The payload discriminator: always `"coproduct"`.
        kind: &'static str,
    },
    /// An accepted pullback's apex and square arrow-name sequences. The
    /// unconditional `kind` discriminator keeps the untagged payload
    /// distinct from every other shape, exactly as the coproduct arm's
    /// does.
    Pullback {
        /// The square's apex object.
        apex: String,
        /// The legs, in declared order.
        legs: Vec<String>,
        /// The cospan arrows, in declared order.
        cospan: Vec<String>,
        /// The payload discriminator: always `"pullback"`.
        kind: &'static str,
    },
}

fn constraint_views(added: &[EnforcedConstraint]) -> Vec<ConstraintView> {
    added
        .iter()
        .map(|constraint| match constraint {
            EnforcedConstraint::Equation { lhs, rhs } => ConstraintView::Equation {
                lhs: arrow_names(&lhs.0),
                rhs: arrow_names(&rhs.0),
            },
            EnforcedConstraint::Key { object, arrows } => ConstraintView::Key {
                object: object.0.clone(),
                arrows: arrow_names(arrows),
            },
            EnforcedConstraint::Coproduct { object, arrows } => ConstraintView::Coproduct {
                object: object.0.clone(),
                arrows: arrow_names(arrows),
                kind: "coproduct",
            },
            EnforcedConstraint::Pullback { apex, legs, cospan } => ConstraintView::Pullback {
                apex: apex.0.clone(),
                legs: arrow_names(legs),
                cospan: arrow_names(cospan),
                kind: "pullback",
            },
        })
        .collect()
}

/// An arrow sequence as the payload spells it.
fn arrow_names(arrows: &[catherine_core::model::Ident]) -> Vec<String> {
    arrows.iter().map(|arrow| arrow.0.clone()).collect()
}

/// The diff payload: the outcome string is the classification, the
/// scope is present exactly when the claim is bounded, the witness is
/// present exactly for `weakened`, and `findings` is the classification's
/// output verbatim (one violation for `weakened`, empty otherwise).
#[derive(Serialize)]
pub struct DiffData<'a> {
    /// `cosmetic` | `accreted` | `weakened` | `strengthened-in-scope` |
    /// `equivalent-in-scope` | `unknown`.
    pub outcome: &'a str,
    /// The bound the scan ran under (restated).
    pub bound: BoundView,
    /// The exact scope of a bounded claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<ScopeView>,
    /// The answerability hint, present exactly for `outcome: "unknown"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// The weakening witness instance (v2-clean, v1-violating).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<InstanceView>,
    /// The constraints v2 enforces beyond v1 (accreted classes only).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub added: Vec<ConstraintView>,
    /// The classification's findings (empty except for `weakened`).
    pub findings: &'a [Finding],
}

/// The one diff-envelope path.
pub fn diff_envelope(report: &SchemaDiffReport, warnings: Vec<Warning>) -> Envelope<DiffData<'_>> {
    let (outcome, scope, hint, witness, added) = outcome_parts(&report.outcome);
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        DiffData {
            outcome,
            bound: BoundView::of(&report.bound),
            scope,
            hint,
            witness,
            added,
            findings: &report.findings,
        },
        warnings,
        Vec::new(),
    )
}

/// The payload fields of one classification, as a tuple (the match
/// arms' shape).
type Parts = (
    &'static str,
    Option<ScopeView>,
    Option<String>,
    Option<InstanceView>,
    Vec<ConstraintView>,
);

/// The parts of an unbounded, unhinted claim: no scope, no hint, no
/// witness, and the given accreted constraints.
fn plain_parts(outcome: &'static str, added: Vec<ConstraintView>) -> Parts {
    (outcome, None, None, None, added)
}

/// The parts of a bounded claim that is not unknown: the scope, no
/// hint, no witness.
fn scoped_parts(outcome: &'static str, scope: &SearchScope, added: Vec<ConstraintView>) -> Parts {
    (outcome, Some(ScopeView::of(scope)), None, None, added)
}

/// The parts of the `unknown` classification: the scope plus the shared
/// answerability hint ([`unknown_hint`] — the one hint source).
fn unknown_parts(scope: &SearchScope) -> Parts {
    let view = ScopeView::of(scope);
    let hint = unknown_hint(&view);
    ("unknown", Some(view), Some(hint), None, Vec::new())
}

/// One classification as the payload fields: the outcome string, the
/// scope exactly when the claim is bounded, the answerability hint
/// exactly for `unknown`, the witness exactly for `weakened`, and the
/// accreted constraints (empty otherwise).
fn outcome_parts(outcome: &SchemaDiffOutcome) -> Parts {
    match outcome {
        SchemaDiffOutcome::Cosmetic => plain_parts("cosmetic", Vec::new()),
        SchemaDiffOutcome::Accreted { added } => plain_parts("accreted", constraint_views(added)),
        SchemaDiffOutcome::Weakened { witness } => (
            "weakened",
            None,
            None,
            Some(InstanceView::of(witness)),
            Vec::new(),
        ),
        SchemaDiffOutcome::StrengthenedInScope { added, scope } => scoped_parts(
            "strengthened-in-scope",
            scope,
            constraint_views(added),
        ),
        SchemaDiffOutcome::EquivalentInScope { scope } => {
            scoped_parts("equivalent-in-scope", scope, Vec::new())
        }
        SchemaDiffOutcome::Unknown { scope } => unknown_parts(scope),
        // The runner refuses this class before rendering (labeled error).
        SchemaDiffOutcome::Incomparable => plain_parts("incomparable", Vec::new()),
    }
}

/// The diff human lines. The weakening block reuses the §5 renderer with
/// the v1 schema, so the finding names the lost equation exactly.
pub fn render_diff(report: &SchemaDiffReport, v1: &Schema) -> Vec<String> {
    match &report.outcome {
        SchemaDiffOutcome::Cosmetic => {
            vec!["diff: cosmetic — no enforced-constraint change, 0 findings".to_string()]
        }
        SchemaDiffOutcome::Accreted { added } => accreted_lines(added),
        SchemaDiffOutcome::Weakened { .. } => weakened_lines(report, v1),
        SchemaDiffOutcome::StrengthenedInScope { added, scope } => strengthened_lines(added, scope),
        SchemaDiffOutcome::EquivalentInScope { scope } => vec![format!(
            "diff: equivalent-in-scope — no separating instance within the explored bounds{}",
            scope_suffix(scope)
        )],
        SchemaDiffOutcome::Unknown { scope } => vec![format!(
            "diff: unknown — budget expired after {} candidate(s) within bounds",
            scope.candidates_explored
        )],
        SchemaDiffOutcome::Incomparable => vec![
            "diff: incomparable — objects or arrows differ, no classification claimed".to_string(),
        ],
    }
}

/// The `accreted` human lines: the head count, then one line per added
/// constraint.
fn accreted_lines(added: &[EnforcedConstraint]) -> Vec<String> {
    let mut lines = vec![format!(
        "diff: accreted — {} enforced constraint(s) added, 0 findings",
        added.len()
    )];
    lines.extend(added.iter().map(added_line));
    lines
}

/// The `weakened` human lines: the head, then the §5 finding block
/// rendered against the v1 schema (the side that enforced the constraint).
fn weakened_lines(report: &SchemaDiffReport, v1: &Schema) -> Vec<String> {
    let mut lines =
        vec!["diff: weakened — a v1-enforced constraint is not enforced by v2".to_string()];
    lines.extend(
        report
            .findings
            .iter()
            .flat_map(|finding| render_one(finding, v1)),
    );
    lines
}

/// The `strengthened-in-scope` human lines: the head with the scope tail,
/// then one line per added constraint.
fn strengthened_lines(added: &[EnforcedConstraint], scope: &SearchScope) -> Vec<String> {
    let mut lines = vec![format!(
        "diff: strengthened-in-scope — no weakening instance within the explored bounds, {} enforced constraint(s) added{}",
        added.len(),
        scope_suffix(scope)
    )];
    lines.extend(added.iter().map(added_line));
    lines
}

/// One added constraint as a human line: an equation as `lhs = rhs`, a
/// key as `key Object(a∘b)`, a coproduct as `coproduct C(i1∘i2)`, a
/// pullback as `pullback P(p1∘p2 → f∘g)`.
fn added_line(constraint: &EnforcedConstraint) -> String {
    match constraint {
        EnforcedConstraint::Equation { lhs, rhs } => format!(
            "  added: {} = {}",
            super::path_string(lhs),
            super::path_string(rhs)
        ),
        EnforcedConstraint::Key { object, arrows } => {
            format!("  added: key {}({})", object.0, arrows_joined(arrows))
        }
        EnforcedConstraint::Coproduct { object, arrows } => {
            format!("  added: coproduct {}({})", object.0, arrows_joined(arrows))
        }
        EnforcedConstraint::Pullback { apex, legs, cospan } => format!(
            "  added: pullback {}({} → {})",
            apex.0,
            arrows_joined(legs),
            arrows_joined(cospan)
        ),
    }
}

/// An arrow sequence as a human line spells it (`a∘b`, declared order).
fn arrows_joined(arrows: &[catherine_core::model::Ident]) -> String {
    arrows
        .iter()
        .map(|arrow| arrow.0.as_str())
        .collect::<Vec<_>>()
        .join("∘")
}

/// The scope tail of a bounded verdict: candidates explored and the
/// per-object row shape (an empty shape is spelled `none`).
fn scope_suffix(scope: &SearchScope) -> String {
    let rows = if scope.rows.is_empty() {
        "none".to_string()
    } else {
        scope
            .rows
            .iter()
            .map(|(object, count)| format!("{}={count}", object.0))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(" ({} candidate(s), rows {rows})", scope.candidates_explored)
}
