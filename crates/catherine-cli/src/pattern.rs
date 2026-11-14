//! Purpose: the `pattern` runner (openspec change `add-pattern-lenses`,
//! beads `catherine-mx1.2`) — `catherine pattern <model.ct.json> [--id
//! <pattern>] [--values ...] [--list]`, one execution path for the
//! pattern-lens registry and single-lens runs.
//!
//! Responsibilities: the registry listing (id, GoF name, abstraction, law
//! kind, scope; no model read, no lens run), the per-lens run through
//! [`catherine_analysis::pattern::run`], the same findings envelope and
//! min-over-class exit decision as `rules` for the taxonomy's checks, the
//! migration engine's lost-data findings rendered under `data.migrations` in
//! their own serde shape with the lost-data exit class (8), the `unknown`
//! outcome with its scope (a completed bounded run, exit 0), and the labeled
//! usage errors: an omitted or unknown id is `bad_argument` exit 2 naming
//! the id and the known ids; a malformed `--values` spelling is
//! `bad_argument` exit 1; a parse/build failure is the shared labeled error
//! before any lens runs.
//!
//! Rationale: no second pipeline — the shared loader, envelope
//! constructors, warning channel, and exit decision. The run's two finding
//! channels are the two kernel surfaces, so each keeps its own payload key
//! (`data.findings` checks, `data.migrations` lost data) and its own exit
//! contract: a migration finding is always the table's lost-data class,
//! which outranks the checks' minimum because the migration never had a
//! chance to be anything else. `--values` typing is the lens's (it alone
//! knows whether it has a rule surface), so the CLI checks only the
//! spelling; a syntactically valid declaration the lens cannot bind is the
//! lens's `Unknown { scope }`, never a silently ignored flag. Listing runs
//! no lens and prints every metadata field, so the scope string's honesty
//! boundary travels with the id.

use crate::args::bad_arg;
use crate::cli::{default_scope_caps, LOST_DATA_EXIT_CODE};
use crate::render::{self, LabeledError};
use crate::run::{emit_error, exit_code, load_model, write_envelope};
use catherine_analysis::pattern::{
    self, LensError, LensFindings, LensInput, LensOutcome, PatternLens,
};
use catherine_core::findings::Finding;
use catherine_core::migrate::{MigrationFinding, MigrationFindingKind};
use catherine_core::model::Schema;
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use serde::Serialize;
use std::io::Write;

mod command;

pub use command::command;

/// One registry entry as `--list` emits it.
#[derive(Serialize)]
pub struct LensView {
    /// Exact registry id.
    pub id: &'static str,
    /// GoF pattern name.
    pub name: &'static str,
    /// Universal abstraction.
    pub abstraction: &'static str,
    /// `identity | totality | coverage | order | delta`.
    pub law: &'static str,
    /// The disposition row's law scope, verbatim.
    pub scope: &'static str,
}

/// The list payload: the registry in its fixed order.
#[derive(Serialize)]
pub struct PatternListData {
    /// Every registered lens.
    pub lenses: Vec<LensView>,
}

/// The run payload: outcome, the unknown scope when present, the checks
/// under `findings`, and the migration engine's lost data under
/// `migrations`.
#[derive(Serialize)]
pub struct PatternData<'a> {
    /// `"analyzed"` or `"unknown"`.
    pub outcome: &'static str,
    /// The exact surface a lens could not decide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The kernel's taxonomy findings verbatim.
    pub findings: &'a [Finding],
    /// The migration engine's lost-data findings verbatim — the migrate
    /// surface's own `label` + `kind` + `witness` encoding.
    pub migrations: &'a [MigrationFinding],
}

/// Execute the named lens over `model_path`, or list the registry. Returns
/// the process exit code. `quiet` suppresses human lines only.
#[allow(clippy::too_many_arguments)]
pub fn run_pattern(
    model_path: &str,
    id: Option<&str>,
    values: &[String],
    list: bool,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if list {
        return list_registry(json, quiet, warnings, stdout, stderr);
    }
    let Some(id) = id else {
        let error = bad_arg(
            "`pattern` requires `--id <pattern>` or `--list`".to_string(),
            "run `catherine pattern <model.ct.json> --list` to see the registered ids",
        );
        return emit_usage_error(&error, json, warnings, stdout, stderr);
    };
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    if let Err(labeled) = check_values_syntax(values) {
        return emit_error(&labeled, json, warnings, stdout, stderr);
    }
    run_lens(id, &model, values, json, quiet, warnings, stdout, stderr)
}

/// Run one lens over the loaded model and render its outcome. Split from
/// [`run_pattern`] so the dispatch keeps the gate's function-length bound.
#[allow(clippy::too_many_arguments)]
fn run_lens(
    id: &str,
    model: &catherine_core::model::Model,
    values: &[String],
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let input = LensInput { model, values };
    match pattern::run(id, &input, &default_scope_caps()) {
        Ok(LensOutcome::Findings(findings)) => {
            report_findings(id, &findings, model, json, quiet, warnings, stdout, stderr)
        }
        Ok(LensOutcome::Unknown { scope }) => {
            report_unknown(id, &scope, json, quiet, warnings, stdout, stderr)
        }
        Err(error) => {
            let labeled = unknown_pattern(&error);
            emit_usage_error(&labeled, json, warnings, stdout, stderr)
        }
    }
}

/// `--list`: the registry in fixed order, no model read, no lens run.
fn list_registry(
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let lenses = pattern::registry();
    if json {
        let envelope = Envelope::success(
            render::CLI_VERSION,
            EnvelopeKind::List,
            PatternListData {
                lenses: lenses.iter().map(|lens| view(lens.meta())).collect(),
            },
            warnings,
            Vec::new(),
        );
        write_envelope(&envelope, stdout, stderr, "pattern");
    } else if !quiet {
        for line in list_lines(lenses) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    0
}

/// The human list: a count head, then one metadata line per lens.
fn list_lines(lenses: &[&'static dyn PatternLens]) -> Vec<String> {
    let mut lines = vec![format!("pattern: {} registered lens(es)", lenses.len())];
    for lens in lenses {
        let meta = lens.meta();
        lines.push(format!(
            "{}  {} — {} · {} · {}",
            meta.id,
            meta.name,
            meta.abstraction,
            meta.law.as_str(),
            meta.scope
        ));
    }
    lines
}

/// One lens's metadata as the list view.
fn view(meta: &catherine_analysis::pattern::PatternMeta) -> LensView {
    LensView {
        id: meta.id,
        name: meta.name,
        abstraction: meta.abstraction,
        law: meta.law.as_str(),
        scope: meta.scope,
    }
}

/// Render a completed lens run: the envelope or one head plus the check
/// renderer's witness lines and the migration renderer's lost-data lines,
/// then the exit decision. Any migration finding is the lost-data class
/// (8); otherwise the shared min-over-class decision applies to the checks.
#[allow(clippy::too_many_arguments)]
fn report_findings(
    id: &str,
    findings: &LensFindings,
    model: &catherine_core::model::Model,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    print_warnings(json, &warnings, stderr);
    if json {
        let envelope = Envelope::success(
            render::CLI_VERSION,
            EnvelopeKind::Check,
            PatternData {
                outcome: "analyzed",
                scope: None,
                findings: &findings.checks,
                migrations: &findings.migrations,
            },
            warnings,
            Vec::new(),
        );
        write_envelope(&envelope, stdout, stderr, "pattern");
    } else if !quiet {
        let total = findings.checks.len() + findings.migrations.len();
        let _ = writeln!(stdout, "pattern {id}: {total} finding(s)");
        for line in render::render_findings(&findings.checks, &model.schema) {
            let _ = writeln!(stdout, "{line}");
        }
        for line in render_migrations(&findings.migrations, &model.schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    outcome_exit(findings)
}

/// The exit decision over a completed run: a migration finding is the
/// lost-data class (the table's lowest severity, and never outranked here
/// because the lost data is the run's own result); otherwise the shared
/// min-over-class decision over the checks.
fn outcome_exit(findings: &LensFindings) -> i32 {
    if findings.migrations.is_empty() {
        exit_code(&findings.checks)
    } else {
        LOST_DATA_EXIT_CODE
    }
}

/// The lost-data human lines, the migrate renderer's spelling:
/// `lost-row: Object#n`, `lost-cell: arrow at Object#n` (the owner object
/// resolved from the source schema, the same totality fallback).
fn render_migrations(findings: &[MigrationFinding], schema: &Schema) -> Vec<String> {
    findings
        .iter()
        .map(|finding| match &finding.kind {
            MigrationFindingKind::LostRow { object, row } => {
                format!("lost-row: {}#{}", object.0, row.0)
            }
            MigrationFindingKind::LostCell { arrow, row } => {
                let owner = schema
                    .arrows
                    .get(arrow)
                    .map(|decl| format!("{}#{}", decl.src.0, row.0))
                    .unwrap_or_else(|| row.0.to_string());
                format!("lost-cell: {} at {}", arrow.0, owner)
            }
        })
        .collect()
}

/// Render a bounded non-answer: `outcome: "unknown"` plus the exact scope.
/// A completed bounded run exits 0 and is never rendered as clean.
fn report_unknown(
    id: &str,
    scope: &str,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    print_warnings(json, &warnings, stderr);
    if json {
        let envelope = Envelope::success(
            render::CLI_VERSION,
            EnvelopeKind::Check,
            PatternData {
                outcome: "unknown",
                scope: Some(scope.to_string()),
                findings: &[],
                migrations: &[],
            },
            warnings,
            Vec::new(),
        );
        write_envelope(&envelope, stdout, stderr, "pattern");
    } else if !quiet {
        let _ = writeln!(stdout, "pattern {id}: unknown — {scope}");
    }
    0
}

/// The unknown-id refusal as a labeled usage error (exit 2).
fn unknown_pattern(error: &LensError) -> LabeledError {
    bad_arg(
        error.to_string(),
        "run `catherine pattern <model.ct.json> --list` to see the registered ids",
    )
}

/// Only the `--values` spelling is the CLI's: each spec must be
/// `column=v1,v2,...`. Typing and column binding belong to the lens, whose
/// refusal is a labeled UNKNOWN naming the missing surface.
fn check_values_syntax(values: &[String]) -> Result<(), LabeledError> {
    for spec in values {
        let Some((name, list)) = spec.split_once('=') else {
            return Err(bad_arg(
                format!("--values expects column=value[,value...], got `{spec}`"),
                "spell the values as column=v1,v2,..., e.g. --values kind=lit,neg",
            ));
        };
        if name.is_empty() || list.is_empty() {
            return Err(bad_arg(
                format!("--values expects a column and at least one value, got `{spec}`"),
                "spell the values as column=v1,v2,..., e.g. --values kind=lit,neg",
            ));
        }
    }
    Ok(())
}

/// Emit a labeled error on stderr and return the given exit code (2 for the
/// usage class, which the shared [`emit_error`] fixed at 1).
fn emit_usage_error(
    labeled: &LabeledError,
    json: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let _ = stdout;
    if json {
        let envelope = render::error_envelope(labeled, warnings);
        match serde_json::to_string(&envelope) {
            Ok(text) => {
                let _ = writeln!(stderr, "{text}");
            }
            Err(error) => {
                let _ = writeln!(stderr, "catherine: cannot write error envelope: {error}");
            }
        }
    } else {
        let _ = writeln!(stderr, "{}", labeled.human_head());
        let _ = writeln!(stderr, "  hint: {}", labeled.hint);
    }
    2
}

/// Run-level warnings reach stderr in human mode too.
fn print_warnings(json: bool, warnings: &[Warning], stderr: &mut impl Write) {
    if json {
        return;
    }
    for warning in warnings {
        let _ = writeln!(stderr, "warning: {}", warning.message);
    }
}

#[cfg(test)]
mod tests;
