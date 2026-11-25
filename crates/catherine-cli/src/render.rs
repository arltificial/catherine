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
//! `hypothesized`/`unknown` findings append their label. A pullback
//! violation keeps the block's two-line shape: the head names the
//! declaration (index, apex, legs, cospan) and the witness line names
//! only the verdict's offender — the failed square's row and composed
//! values, the uncovered fiber pair, or the duplicate-projecting rows.
//! The three analysis kinds have no producer until P4; their renderings
//! are pinned by unit tests over constructed findings and their phrasing
//! stays minimal — P4's producers own the richer domain wording.
//!
//! Layout: the §5 finding renderer lives here; the labeled-error type
//! is [`errors`], the envelope constructors are [`envelope`], the
//! analysis view structs/renderers are [`analysis`], and the migration
//! payload is [`migrate`]. The re-exports keep the `render::` paths
//! stable for the callers.

mod analysis;
pub(crate) mod diff;
mod envelope;
mod errors;
mod migrate;
mod rules;
mod text;

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
    config_warning, duplicate_document_warning, empty_composition_warning, error_envelope,
    findings_envelope, missing_instance_map_warning, sourced_findings_envelope,
    unknown_follow_warning, unknown_rows_warning, unresolved_naturality_square_warning,
    FindingsData, SourcedFinding, SourcedFindingsData,
};
pub use crate::render::errors::LabeledError;
#[allow(unused_imports)] // pub surface kept stable at `render::`
pub use crate::render::migrate::{migrate_envelope, render_migrate, MigrateData};
#[allow(unused_imports)] // pub surface kept stable at `render::`
pub use crate::render::rules::{render_rules, rules_envelope, RuleScopeView, RulesData};

use catherine_core::findings::{
    CoproductWitness, DanglingWitness, DomainQuestion, EquationWitness, Finding, FindingKind,
    GranularityCase, GranularityWitness, KeyWitness, MigrationNaturalityWitness, NaturalityWitness,
    PullbackWitness, UnderconstraintWitness, ViolationWitness,
};
use catherine_core::model::{Ident, Pullback, RowId, Schema};

/// The public surface keeps the stable `render::path_string` path.
pub use self::text::path_string;
use self::text::{
    arrow_list, arrows_string, label_suffix, naturality_value, path_codomain, path_domain, row_at,
};

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
            vec![format!("{}{label}", rules::contradiction_head(w))]
        }
        FindingKind::Underconstraint(w) => render_underconstraint(w, &label),
        FindingKind::Granularity(w) => render_granularity(w, &label),
    }
}

/// The underconstraint block: the case's head line and the open question.
fn render_underconstraint(w: &UnderconstraintWitness, label: &str) -> Vec<String> {
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

/// The granularity block: the case's head line and the offending arrows.
fn render_granularity(w: &GranularityWitness, label: &str) -> Vec<String> {
    let names = |arrows: &[Ident]| {
        arrows
            .iter()
            .map(|a| a.0.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let head = match &w.case {
        GranularityCase::OverCap { cap, rows } => format!(
            "granularity: {} arrows over {rows} rows exceeds the stored-granularity cap {cap}{}",
            w.arrows.len(),
            label
        ),
        GranularityCase::Unnamed { stored } => format!(
            "granularity: {} of {} arrows declare no stored column{}",
            w.arrows.len(),
            w.arrows.len() + stored,
            label
        ),
    };
    vec![head, format!("  arrows: {}", names(&w.arrows))]
}

/// The §5 violation block, one branch per witness shape. The equation
/// branch renders `violation: eq-<canonical index> <lhs path>
/// ≠ <rhs path>` then the two-space-indented witness line `at row
/// <Object#n>: lhs=<Object#n> rhs=<Object#n>`; the key branch renders
/// `violation: key-<canonical index> <object>(<arrows>)` then
/// `at rows <Object#n>, <Object#m>`; the pullback branch renders
/// `violation: pullback-<canonical index> <apex>(<legs> → <cospan>)`
/// then the verdict's offender. The witness's declaration and the row
/// objects come from the same schema the kernel checked, so the lookups
/// are total for every kernel-emitted witness; the bare-number fallbacks
/// exist for totality under programmatically assembled input.
fn render_violation(w: &ViolationWitness, schema: &Schema, label: &str) -> Vec<String> {
    match w {
        ViolationWitness::Equation(witness) => render_equation_violation(witness, schema, label),
        ViolationWitness::Key(witness) => render_key_violation(witness, schema, label),
        ViolationWitness::Naturality(witness) => {
            render_naturality_violation(witness, schema, label)
        }
        ViolationWitness::Coproduct(witness) => render_coproduct_violation(witness, schema, label),
        ViolationWitness::Pullback(witness) => render_pullback_violation(witness, schema, label),
        // Interim honest arm so the binary builds for every suite while
        // the `add-migration-naturality` render polish lands: the head
        // names the broken square, the witness row carries the values.
        ViolationWitness::MigrationNaturality(witness) => {
            render_migration_naturality_violation(witness, label)
        }
    }
}

/// The interim migration-naturality block: the head names the broken
/// square; the witness row carries both sides' values (`none` when a
/// side is undefined).
fn render_migration_naturality_violation(
    witness: &MigrationNaturalityWitness,
    label: &str,
) -> Vec<String> {
    let row = |v: &Option<RowId>| {
        v.map(|r| r.0.to_string())
            .unwrap_or_else(|| "none".to_string())
    };
    let path = witness
        .path
        .iter()
        .map(|a| a.0.as_str())
        .collect::<Vec<_>>()
        .join(" ; ");
    vec![
        format!(
            "violation: migration naturality {} ; {} ≠ {} ; migrate{label}",
            witness.operation.0, path, path
        ),
        format!(
            "  at row #{}: lhs={} rhs={}",
            witness.source_row.0,
            row(&witness.lhs),
            row(&witness.rhs)
        ),
    ]
}

/// The coproduct violation block: the declaration's canonical index, its
/// object and injections, then the failed row. A coverage failure spells
/// the row `uncovered`; a disjointness failure names every covering
/// injection in the declaration's order.
fn render_coproduct_violation(w: &CoproductWitness, schema: &Schema, label: &str) -> Vec<String> {
    let coproduct = schema.coproducts.get(w.coproduct);
    let head = match coproduct {
        Some(coproduct) => format!(
            "violation: coproduct-{} {}({}){}",
            w.coproduct,
            coproduct.object.0,
            arrows_string(&coproduct.arrows),
            label
        ),
        None => format!("violation: coproduct-{}{}", w.coproduct, label),
    };
    let object = coproduct.map(|coproduct| coproduct.object.0.as_str());
    let row = row_at(object, w.row);
    let detail = if w.injections.is_empty() {
        format!("  at row {row}: uncovered — no declared injection covers it")
    } else {
        format!(
            "  at row {row}: covered by {} (disjointness)",
            arrows_string(&w.injections)
        )
    };
    vec![head, detail]
}

/// The pullback violation block: the declaration's canonical index, its
/// apex and arrow lists, then the verdict's offender — the apex row and
/// both composed values (commutes), the uncovered fiber pair (covers),
/// or the two apex rows projecting to one pair (unique). The objects on
/// the witness line come from the declaration's apex and its cospan
/// arrows, so a kernel-emitted witness always spells `Object#n`; a
/// programmatically assembled witness whose declaration is missing falls
/// back to bare row numbers.
fn render_pullback_violation(w: &PullbackWitness, schema: &Schema, label: &str) -> Vec<String> {
    let pullback = schema.pullbacks.get(w.pullback());
    let head = match pullback {
        Some(pullback) => format!(
            "violation: pullback-{} {}({} → {}){}",
            w.pullback(),
            pullback.apex.0,
            arrow_list(&pullback.legs),
            arrow_list(&pullback.cospan),
            label
        ),
        None => format!("violation: pullback-{}{}", w.pullback(), label),
    };
    vec![head, pullback_detail(w, pullback, schema)]
}

/// The pullback witness line: one shape per verdict, over the object
/// names the declaration supplies.
fn pullback_detail(w: &PullbackWitness, pullback: Option<&Pullback>, schema: &Schema) -> String {
    let apex = pullback.map(|pullback| pullback.apex.0.as_str());
    let (a, c_left) = cospan_ends(schema, pullback, 0);
    let (b, c_right) = cospan_ends(schema, pullback, 1);
    let c = c_left.or(c_right);
    match w {
        PullbackWitness::Commutes { row, lhs, rhs, .. } => format!(
            "  at row {}: lhs={} rhs={}",
            row_at(apex, *row),
            row_at(c, *lhs),
            row_at(c, *rhs),
        ),
        PullbackWitness::Covers { pair, .. } => format!(
            "  uncovered pair {}, {}",
            row_at(a, pair[0]),
            row_at(b, pair[1])
        ),
        PullbackWitness::Unique { rows, .. } => format!(
            "  at rows {}, {}: both project to one pair",
            row_at(apex, rows[0]),
            row_at(apex, rows[1])
        ),
    }
}

/// The `index`-th cospan arrow's source and target objects, read from the
/// declaration and the schema; absent when either lookup fails (the
/// totality fallback).
fn cospan_ends<'a>(
    schema: &'a Schema,
    pullback: Option<&Pullback>,
    index: usize,
) -> (Option<&'a str>, Option<&'a str>) {
    let arrow = pullback
        .and_then(|pullback| pullback.cospan.get(index))
        .and_then(|name| schema.arrows.get(name));
    match arrow {
        Some(arrow) => (Some(arrow.src.0.as_str()), Some(arrow.tgt.0.as_str())),
        None => (None, None),
    }
}

/// The naturality violation block: the arrow, the source row, and both
/// composed values in the transcript's `Object#n` spelling (the §5
/// `at row …: lhs=… rhs=…` line).
fn render_naturality_violation(w: &NaturalityWitness, schema: &Schema, label: &str) -> Vec<String> {
    let arrow = schema.arrows.get(&w.arrow);
    let (src_object, tgt_object) = match arrow {
        Some(arrow) => (Some(arrow.src.0.as_str()), Some(arrow.tgt.0.as_str())),
        None => (None, None),
    };
    vec![
        format!("violation: naturality of {}{}", w.arrow.0, label),
        format!(
            "  at row {}: lhs={} rhs={}",
            row_at(src_object, w.source_row),
            naturality_value(tgt_object, w.lhs),
            naturality_value(tgt_object, w.rhs),
        ),
    ]
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

#[cfg(test)]
mod tests;
