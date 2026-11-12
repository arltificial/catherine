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
use crate::render::{self, LabeledError};
use crate::run::{emit_error, exit_code, load_model, write_envelope};
use crate::runners::{print_run_warnings, prune_undeclared_rows};
use catherine_analysis::bounds::SearchBounds;
use catherine_analysis::schema_diff::{self, SchemaDiffOutcome};
use genesis::envelope::Warning;
use std::io::Write;

/// Execute the schema diff once and render. Returns the process exit
/// code. `quiet` suppresses the human lines (never the exit decision,
/// never errors, never the machine envelope).
#[allow(clippy::too_many_arguments)]
pub fn run_diff(
    model_path: &str,
    second_path: &str,
    rows: &[String],
    max_candidates: usize,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let v1 = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let v2 = match load_model(second_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let mut max_rows = match parse_rows(rows) {
        Ok(max_rows) => max_rows,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let mut warnings = warnings;
    prune_undeclared_rows(&v1.schema, &mut max_rows, &mut warnings);
    let bound = SearchBounds {
        max_rows,
        max_candidates,
    };
    let report = schema_diff::diff(&v1.schema, &v2.schema, &bound);
    if matches!(report.outcome, SchemaDiffOutcome::Incomparable) {
        return emit_error(&incomparable(second_path), json, warnings, stdout, stderr);
    }
    print_run_warnings(json, &warnings, stderr);
    if json {
        let envelope = render::diff_envelope(&report, warnings);
        write_envelope(&envelope, stdout, stderr, "diff");
    } else if !quiet {
        for line in render::render_diff(&report, &v1.schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    exit_code(&report.findings)
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
