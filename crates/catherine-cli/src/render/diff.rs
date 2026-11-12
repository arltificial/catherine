//! Purpose: the schema-diff payload (openspec change `add-schema-diff`,
//! task 7) — the `diff` envelope's `data` and the human rendering of a
//! classification.
//!
//! Responsibilities: build the one diff-envelope path (outcome string,
//! bound restated, scope when the claim is bounded, the weakening
//! witness instance, the added enforced constraints, and the findings
//! verbatim) and render the human lines: the classification head, the
//! §5 finding block for `weakened`, and one `added:` line per accreted
//! constraint.
//!
//! Rationale: one envelope shape (the genesis `Check` kind over the
//! analysis payloads); the outcome string names the epistemic state and
//! is never collapsed into "clean" — `equivalent-in-scope` and
//! `unknown` carry their scope, `cosmetic` says exactly zero findings.
//! The weakening finding is rendered by the shared §5 renderer against
//! the v1 schema (the side that enforced the equation), so its equation
//! index resolves to the exact lost constraint.

use super::analysis::{BoundView, InstanceView, ScopeView};
use super::render_one;
use super::CLI_VERSION;
use catherine_analysis::bounds::SearchScope;
use catherine_analysis::schema_diff::{EnforcedEquation, SchemaDiffOutcome, SchemaDiffReport};
use catherine_core::findings::Finding;
use catherine_core::model::Schema;
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use serde::Serialize;

/// One enforced equation as the payload spells it: arrow-name sequences.
#[derive(Serialize)]
pub struct ConstraintView {
    /// The left side's arrows, in composition order.
    pub lhs: Vec<String>,
    /// The right side's arrows, in composition order.
    pub rhs: Vec<String>,
}

fn constraint_views(added: &[EnforcedEquation]) -> Vec<ConstraintView> {
    added
        .iter()
        .map(|equation| ConstraintView {
            lhs: equation.lhs.0.iter().map(|arrow| arrow.0.clone()).collect(),
            rhs: equation.rhs.0.iter().map(|arrow| arrow.0.clone()).collect(),
        })
        .collect()
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
    let (outcome, scope, witness, added) = match &report.outcome {
        SchemaDiffOutcome::Cosmetic => ("cosmetic", None, None, Vec::new()),
        SchemaDiffOutcome::Accreted { added } => ("accreted", None, None, constraint_views(added)),
        SchemaDiffOutcome::Weakened { witness } => (
            "weakened",
            None,
            Some(InstanceView::of(witness)),
            Vec::new(),
        ),
        SchemaDiffOutcome::StrengthenedInScope { added, scope } => (
            "strengthened-in-scope",
            Some(ScopeView::of(scope)),
            None,
            constraint_views(added),
        ),
        SchemaDiffOutcome::EquivalentInScope { scope } => (
            "equivalent-in-scope",
            Some(ScopeView::of(scope)),
            None,
            Vec::new(),
        ),
        SchemaDiffOutcome::Unknown { scope } => {
            ("unknown", Some(ScopeView::of(scope)), None, Vec::new())
        }
        // The runner refuses this class before rendering (labeled error).
        SchemaDiffOutcome::Incomparable => ("incomparable", None, None, Vec::new()),
    };
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        DiffData {
            outcome,
            bound: BoundView::of(&report.bound),
            scope,
            witness,
            added,
            findings: &report.findings,
        },
        warnings,
        Vec::new(),
    )
}

/// The diff human lines. The weakening block reuses the §5 renderer with
/// the v1 schema, so the finding names the lost equation exactly.
pub fn render_diff(report: &SchemaDiffReport, v1: &Schema) -> Vec<String> {
    match &report.outcome {
        SchemaDiffOutcome::Cosmetic => {
            vec!["diff: cosmetic — no enforced-constraint change, 0 findings".to_string()]
        }
        SchemaDiffOutcome::Accreted { added } => {
            let mut lines = vec![format!(
                "diff: accreted — {} enforced constraint(s) added, 0 findings",
                added.len()
            )];
            lines.extend(added.iter().map(added_line));
            lines
        }
        SchemaDiffOutcome::Weakened { .. } => {
            let mut lines =
                vec!["diff: weakened — a v1-enforced equation is not enforced by v2".to_string()];
            lines.extend(
                report
                    .findings
                    .iter()
                    .flat_map(|finding| render_one(finding, v1)),
            );
            lines
        }
        SchemaDiffOutcome::StrengthenedInScope { added, scope } => {
            let mut lines = vec![format!(
                "diff: strengthened-in-scope — no weakening instance within the explored bounds, {} enforced constraint(s) added{}",
                added.len(),
                scope_suffix(scope)
            )];
            lines.extend(added.iter().map(added_line));
            lines
        }
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

/// One added constraint as a human line.
fn added_line(equation: &EnforcedEquation) -> String {
    format!(
        "  added: {} = {}",
        super::path_string(&equation.lhs),
        super::path_string(&equation.rhs)
    )
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
