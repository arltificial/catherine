//! Purpose: presentation — the human renderer (the DESIGN §5 finding
//! format and the labeled-error format), the deduplicated genesis-vibes
//! envelope construction, and the remediation-hint table for P0's
//! stable codes.
//!
//! Responsibilities: render each finding as its labeled line(s) — kind
//! label first, witness facts on the two-space-indented continuation
//! line, paths joined with `∘`, disagreement as `≠`, row ids as
//! `Object#n` — the §5 transcript's exact format for the violation
//! block; render parse/build failures as labeled errors carrying
//! line/column, the stable code, and a remediation hint (never a stack
//! trace); build the `--json` envelopes one way for both subcommands:
//! findings under `data.findings` exactly as `catherine-core`
//! serializes them (no second serialization, no reshaping), errors as
//! the envelope's `ErrorResult` form with the kernel's message and
//! position unmodified under it.
//!
//! Rationale: the epistemic label rides on every finding in the JSON
//! payload; in human text the kernel-computed default (`executed`)
//! stays off the line so the §5 transcript is byte-reproducible, while
//! `hypothesized`/`unknown` findings append their label. The three
//! analysis kinds have no producer until P4; their renderings are
//! pinned by unit tests over constructed findings and their phrasing
//! stays minimal — P4's producers own the richer domain wording.
//!
//! Layout: the §5 finding renderer lives here; the labeled-error type
//! is [`errors`], the envelope constructors are [`envelope`], the
//! analysis view structs/renderers are [`analysis`], and the migration
//! payload is [`migrate`]. The re-exports keep the `render::` paths
//! stable for the callers.

mod analysis;
mod envelope;
mod errors;
mod migrate;
mod rules;

#[allow(unused_imports)] // pub surface kept stable at `render::`
pub use crate::render::analysis::{
    distinguish_envelope, query_envelope, render_distinguish, render_query, render_search,
    search_envelope, BoundView, DifferenceView, DistinguishData, InstanceView, NodeView,
    ObservationView, QueryData, ScopeView, SearchData,
};
#[allow(unused_imports)] // pub surface kept stable at `render::`
pub use crate::render::envelope::{
    config_warning, error_envelope, findings_envelope, unknown_follow_warning,
    unknown_rows_warning, FindingsData,
};
pub use crate::render::errors::LabeledError;
#[allow(unused_imports)] // pub surface kept stable at `render::`
pub use crate::render::migrate::{migrate_envelope, render_migrate, MigrateData};
#[allow(unused_imports)] // pub surface kept stable at `render::`
pub use crate::render::rules::{render_rules, rules_envelope, RuleScopeView, RulesData};

use catherine_core::findings::{
    DanglingWitness, DomainQuestion, EpistemicLabel, EquationWitness, Finding, FindingKind,
};
use catherine_core::model::{Path, RowId, Schema};

/// The CLI version carried in every envelope's `cli_version`.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Human finding rendering (the §5 format) ──────────────────────────

/// Render the findings as the §5 human lines: one head line per finding
/// (kind label + witness head), witness rows as two-space-indented
/// continuation lines.
pub fn render_findings(findings: &[Finding], schema: &Schema) -> Vec<String> {
    findings
        .iter()
        .flat_map(|f| render_one(f, schema))
        .collect()
}

pub(super) fn render_one(finding: &Finding, schema: &Schema) -> Vec<String> {
    let label = label_suffix(finding.label);
    match &finding.kind {
        FindingKind::Violation(w) => render_violation(w, schema, &label),
        FindingKind::Dangling(w) => render_dangling(w, schema, &label),
        FindingKind::Redundant(w) => vec![
            format!("redundant: {}{}", path_string(&w.derivable), label),
            format!("  derivable as {}", path_string(&w.reducing)),
        ],
        FindingKind::Contradiction(w) => {
            let scope = w
                .scope
                .iter()
                .map(|index| format!("eq-{index}"))
                .collect::<Vec<_>>()
                .join(", ");
            vec![format!("contradiction: {scope} admit no instance{label}")]
        }
        FindingKind::Underconstraint(w) => {
            let head = match &w.case {
                DomainQuestion::ParallelPaths { lhs, rhs } => format!(
                    "underconstraint: parallel paths {} / {}{}",
                    path_string(lhs),
                    path_string(rhs),
                    label
                ),
                DomainQuestion::UncoveredInputs { class, examples } => {
                    let rows = examples
                        .iter()
                        .map(|row| row.0.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("underconstraint: uncovered inputs {class} (rows {rows}){label}")
                }
            };
            vec![head, format!("  {}", w.question)]
        }
    }
}

/// The §5 violation block: `violation: eq-<canonical index> <lhs path>
/// ≠ <rhs path>` then the two-space-indented witness line `at row
/// <Object#n>: lhs=<Object#n> rhs=<Object#n>`. The equation's paths and
/// the row objects come from the same schema the kernel checked, so the
/// lookups are total for every kernel-emitted witness; the bare-number
/// fallbacks exist for totality under programmatically assembled input.
fn render_violation(w: &EquationWitness, schema: &Schema, label: &str) -> Vec<String> {
    let equation = schema.equations.get(w.equation);
    let head = match equation {
        Some(eq) => format!(
            "violation: eq-{} {} ≠ {}{}",
            w.equation,
            path_string(&eq.lhs),
            path_string(&eq.rhs),
            label
        ),
        None => format!("violation: eq-{}{}", w.equation, label),
    };
    let row_line = match equation {
        Some(eq) => format!(
            "  at row {}: lhs={} rhs={}",
            row_at(path_domain(schema, &eq.lhs), w.row),
            row_at(path_codomain(schema, &eq.lhs), w.lhs),
            row_at(path_codomain(schema, &eq.rhs), w.rhs),
        ),
        None => format!("  at row {}: lhs={} rhs={}", w.row.0, w.lhs.0, w.rhs.0),
    };
    vec![head, row_line]
}

/// The dangling line pair: the arrow and the source row the dangling
/// value sits on.
fn render_dangling(w: &DanglingWitness, schema: &Schema, label: &str) -> Vec<String> {
    let head = format!("dangling: arrow {}{}", w.arrow.0, label);
    let src_object = schema.arrows.get(&w.arrow).map(|a| a.src.0.as_str());
    vec![head, format!("  at row {}", row_at(src_object, w.src))]
}

/// The epistemic label as rendered text: the kernel-computed default
/// (`executed`) stays off the human line so the §5 transcript is
/// byte-reproducible; the other weights append their label.
fn label_suffix(label: EpistemicLabel) -> String {
    match label {
        EpistemicLabel::Executed => String::new(),
        EpistemicLabel::Hypothesized => " (hypothesized)".to_string(),
        EpistemicLabel::Unknown => " (unknown)".to_string(),
    }
}

/// A path as the transcript spells it: the arrow names in composition
/// order joined with `∘`.
pub fn path_string(path: &Path) -> String {
    path.0
        .iter()
        .map(|ident| ident.0.as_str())
        .collect::<Vec<_>>()
        .join("∘")
}

/// A row as the transcript spells it: `Object#n` when the object is
/// known from the schema, the bare number otherwise (totality).
fn row_at(object: Option<&str>, row: RowId) -> String {
    match object {
        Some(name) => format!("{name}#{}", row.0),
        None => row.0.to_string(),
    }
}

/// The composed path's domain: its lead arrow's src.
fn path_domain<'a>(schema: &'a Schema, path: &Path) -> Option<&'a str> {
    schema.arrows.get(path.0.first()?).map(|a| a.src.0.as_str())
}

/// The composed path's codomain: its last arrow's tgt.
fn path_codomain<'a>(schema: &'a Schema, path: &Path) -> Option<&'a str> {
    schema.arrows.get(path.0.last()?).map(|a| a.tgt.0.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::findings::{
        ContradictionWitness, DanglingWitness, RedundantWitness, UnderconstraintWitness,
    };
    use catherine_core::model::{Arrow, ConstraintStatus, Equation, Ident};
    use std::collections::BTreeMap;

    fn finding(kind: FindingKind, label: EpistemicLabel) -> Finding {
        Finding { label, kind }
    }

    /// The bowling schema, minimally reconstructed for the transcript
    /// pin: the four arrows and the one canonical equation.
    fn bowling_schema() -> Schema {
        let mut arrows = BTreeMap::new();
        for (name, src, tgt) in [
            ("staff_unit", "Assignment", "Staff"),
            ("assigned_staff", "Staff", "Unit"),
            ("equipment_unit", "Assignment", "Equipment"),
            ("assigned_item", "Equipment", "Unit"),
        ] {
            arrows.insert(
                Ident(name.to_string()),
                Arrow {
                    src: Ident(src.to_string()),
                    tgt: Ident(tgt.to_string()),
                },
            );
        }
        Schema {
            objects: BTreeMap::new(),
            arrows,
            equations: vec![Equation {
                lhs: Path(vec![
                    Ident("staff_unit".to_string()),
                    Ident("assigned_staff".to_string()),
                ]),
                rhs: Path(vec![
                    Ident("equipment_unit".to_string()),
                    Ident("assigned_item".to_string()),
                ]),
                rationale: "a row's staff and its equipment land in the same unit".to_string(),
                status: ConstraintStatus::Accepted,
            }],
        }
    }

    #[test]
    fn violation_renders_the_s5_block() {
        let schema = bowling_schema();
        let lines = render_one(
            &finding(
                FindingKind::Violation(EquationWitness {
                    equation: 0,
                    row: RowId(3),
                    lhs: RowId(1),
                    rhs: RowId(2),
                }),
                EpistemicLabel::Executed,
            ),
            &schema,
        );
        assert_eq!(
            lines,
            vec![
                "violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item",
                "  at row Assignment#3: lhs=Unit#1 rhs=Unit#2",
            ]
        );
    }

    #[test]
    fn non_executed_labels_append_to_the_head_line() {
        let schema = bowling_schema();
        let lines = render_one(
            &finding(
                FindingKind::Violation(EquationWitness {
                    equation: 0,
                    row: RowId(3),
                    lhs: RowId(1),
                    rhs: RowId(2),
                }),
                EpistemicLabel::Hypothesized,
            ),
            &schema,
        );
        assert!(lines[0].ends_with(" (hypothesized)"), "{}", lines[0]);
        assert_eq!(lines[1], "  at row Assignment#3: lhs=Unit#1 rhs=Unit#2");
    }

    #[test]
    fn dangling_renders_arrow_and_source_row() {
        let schema = bowling_schema();
        let lines = render_one(
            &finding(
                FindingKind::Dangling(DanglingWitness {
                    arrow: Ident("assigned_staff".to_string()),
                    src: RowId(0),
                }),
                EpistemicLabel::Executed,
            ),
            &schema,
        );
        assert_eq!(
            lines,
            vec!["dangling: arrow assigned_staff", "  at row Staff#0"]
        );
    }

    #[test]
    fn redundant_renders_labeled_lines() {
        // The three analysis classes have no P4 producer; constructed
        // findings pin their renderings at the unit level (brief §3
        // criterion 4). No schema anchor is needed.
        let schema = Schema::default();
        let lines = render_one(
            &finding(
                FindingKind::Redundant(RedundantWitness {
                    derivable: Path(vec![Ident("f".to_string()), Ident("g".to_string())]),
                    reducing: Path(vec![Ident("h".to_string())]),
                }),
                EpistemicLabel::Unknown,
            ),
            &schema,
        );
        assert_eq!(lines, vec!["redundant: f∘g (unknown)", "  derivable as h",]);
    }

    #[test]
    fn contradiction_renders_labeled_lines() {
        let schema = Schema::default();
        let lines = render_one(
            &finding(
                FindingKind::Contradiction(ContradictionWitness { scope: vec![0, 2] }),
                EpistemicLabel::Unknown,
            ),
            &schema,
        );
        assert_eq!(
            lines,
            vec!["contradiction: eq-0, eq-2 admit no instance (unknown)"]
        );
    }

    #[test]
    fn underconstraint_renders_labeled_lines() {
        let schema = Schema::default();
        let lines = render_one(
            &finding(
                FindingKind::Underconstraint(UnderconstraintWitness {
                    case: DomainQuestion::ParallelPaths {
                        lhs: Path(vec![Ident("staff_unit".to_string())]),
                        rhs: Path(vec![Ident("equipment_unit".to_string())]),
                    },
                    question: "have no declared equation?".to_string(),
                }),
                EpistemicLabel::Hypothesized,
            ),
            &schema,
        );
        assert_eq!(
            lines,
            vec![
                "underconstraint: parallel paths staff_unit / equipment_unit (hypothesized)",
                "  have no declared equation?",
            ]
        );
    }
}
