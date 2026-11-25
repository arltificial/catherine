//! Purpose: the `diff` runner (openspec change `add-schema-diff`, task
//! 7) — one execution path for the schema-diff surface: load both
//! documents, classify the v1→v2 edit under the caller's bounds,
//! render, and decide the exit.
//!
//! Responsibilities: the same stream discipline as every other runner
//! (payload to stdout, every error to stderr); the exit decision —
//! `weakened` is a finding-bearing run through the existing class table
//! (`run::exit_code`), every other completed classification exits 0
//! with the outcome in the payload (an `unknown` payload names its scope
//! and is never "clean"); `incomparable` documents are the labeled
//! `schema_incomparable` input error (exit 1), never a guess across
//! vocabularies.
//!
//! Rationale: no second pipeline — the shared loader, `--rows` parser,
//! undeclared-object warning, envelope constructors, and exit decision.
//! The bound is pruned against v1's schema only after the structural
//! check would have refused a vocabulary difference; an undeclared
//! `--rows` object is named on the warnings channel (catherine-0kw), as
//! on `search`.

use crate::args::parse_rows;
use crate::dispatch::Ctx;
use crate::render::{self, LabeledError};
use crate::run::{exit_code, load_model, write_envelope};
use crate::runners::prune_undeclared_rows;
use catherine_analysis::bounds::SearchBounds;
use catherine_analysis::schema_diff::{self, SchemaDiffOutcome};
use catherine_core::model::{Ident, Model, Schema};
use genesis::envelope::Warning;
use std::collections::BTreeMap;
use std::io::Write;

/// The diff invocation's own flags: the two documents and the scan
/// bounds (`--rows`, `--max-candidates`); the output plumbing travels
/// in the run context.
pub(crate) struct DiffRun<'a> {
    /// The v1 document, the edit's left side.
    pub(crate) model_path: &'a str,
    /// The v2 document, the edit's right side.
    pub(crate) second_path: &'a str,
    /// The `--rows` bounds, unparsed.
    pub(crate) rows: &'a [String],
    /// The `--max-candidates` budget.
    pub(crate) max_candidates: usize,
}

/// Execute the schema diff once and render. Returns the process exit
/// code. `quiet` suppresses the human lines (never the exit decision,
/// never errors, never the machine envelope).
pub fn run_diff(run: DiffRun<'_>, ctx: &mut Ctx<'_, impl Write, impl Write>) -> i32 {
    let (v1, v2) = match load_pair(run.model_path, run.second_path) {
        Ok(pair) => pair,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    let max_rows = match parse_rows(run.rows) {
        Ok(max_rows) => max_rows,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    let bound = prune_and_bound(&v1.schema, max_rows, run.max_candidates, &mut ctx.warnings);
    let report = schema_diff::diff(&v1.schema, &v2.schema, &bound);
    if matches!(report.outcome, SchemaDiffOutcome::Incomparable) {
        return ctx.emit_error(&incomparable(run.second_path));
    }
    render_diff_outcome(&report, &v1.schema, ctx)
}

/// The render tail of [`run_diff`]: run warnings, the `--json` envelope
/// or the §5 human lines, then the exit decision over the findings.
fn render_diff_outcome(
    report: &schema_diff::SchemaDiffReport,
    schema: &Schema,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    ctx.print_warnings();
    if ctx.json {
        let envelope = render::diff_envelope(report, ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "diff");
    } else if !ctx.quiet {
        for line in render::render_diff(report, schema) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    exit_code(&report.findings)
}

/// Both documents through the shared loader; the first failure is the
/// labeled error, never a half-loaded run.
fn load_pair(model_path: &str, second_path: &str) -> Result<(Model, Model), LabeledError> {
    Ok((load_model(model_path)?, load_model(second_path)?))
}

/// The bound the scan runs under: the `--rows` map pruned to v1's
/// declared objects, naming an undeclared object on the warnings channel
/// (catherine-0kw).
fn prune_and_bound(
    schema: &Schema,
    mut max_rows: BTreeMap<Ident, usize>,
    max_candidates: usize,
    warnings: &mut Vec<Warning>,
) -> SearchBounds {
    prune_undeclared_rows(schema, &mut max_rows, warnings);
    SearchBounds {
        max_rows,
        max_candidates,
    }
}

/// A vocabulary difference is a labeled input error: instance spaces
/// across different objects/arrows need a declared schema map, not a
/// guess (the `distinguish` schema-mismatch precedent).
fn incomparable(second_path: &str) -> LabeledError {
    LabeledError {
        code: "schema_incomparable".to_string(),
        line: None,
        column: None,
        message: format!("the schemas differ in objects or arrows (`{second_path}`)"),
        hint: "schema diff compares edits over one vocabulary; align the objects/arrows first, or use the Δ migration surface for a declared map",
    }
}
