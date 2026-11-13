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
mod diff;
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
pub use crate::render::diff::{diff_envelope, render_diff, ConstraintView, DiffData};
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
    KeyWitness, ViolationWitness,
};
use catherine_core::model::{Ident, Path, RowId, Schema};

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

/// The §5 violation block, one branch per witness shape. The equation
/// branch renders `violation: eq-<canonical index> <lhs path>
/// ≠ <rhs path>` then the two-space-indented witness line `at row
/// <Object#n>: lhs=<Object#n> rhs=<Object#n>`; the key branch renders
/// `violation: key-<canonical index> <object>(<arrows>)` then
/// `at rows <Object#n>, <Object#m>`. The witness's declaration and the
/// row objects come from the same schema the kernel checked, so the
/// lookups are total for every kernel-emitted witness; the bare-number
/// fallbacks exist for totality under programmatically assembled input.
fn render_violation(w: &ViolationWitness, schema: &Schema, label: &str) -> Vec<String> {
    match w {
        ViolationWitness::Equation(witness) => render_equation_violation(witness, schema, label),
        ViolationWitness::Key(witness) => render_key_violation(witness, schema, label),
    }
}

/// The §5 equation-violation block.
fn render_equation_violation(w: &EquationWitness, schema: &Schema, label: &str) -> Vec<String> {
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

/// The §5 key-violation block: the key's canonical index, its object and
/// arrow tuple, and the two colliding rows ascending.
fn render_key_violation(w: &KeyWitness, schema: &Schema, label: &str) -> Vec<String> {
    let key = schema.keys.get(w.key);
    let head = match key {
        Some(key) => format!(
            "violation: key-{} {}({}){}",
            w.key,
            key.object.0,
            arrows_string(&key.arrows),
            label
        ),
        None => format!("violation: key-{}{}", w.key, label),
    };
    let object = key.map(|key| key.object.0.as_str());
    vec![
        head,
        format!(
            "  at rows {}, {}",
            row_at(object, w.rows[0]),
            row_at(object, w.rows[1])
        ),
    ]
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
    arrows_string(&path.0)
}

/// An arrow sequence as the transcript spells it: names joined with `∘`.
fn arrows_string(arrows: &[Ident]) -> String {
    arrows
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
mod tests;
