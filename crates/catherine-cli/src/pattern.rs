//! Purpose: the `pattern` runner (openspec change `add-pattern-lenses`,
//! beads `catherine-mx1.2`) — `catherine pattern <model.ct.json> [--id
//! <pattern>] [--values ...] [--list]`, one execution path for the
//! pattern-lens registry and single-lens runs.
//!
//! Responsibilities: the registry listing, the per-lens run through
//! [`catherine_analysis::pattern::run`], the same findings envelope and
//! min-over-class exit decision as `rules` for the taxonomy's checks, the
//! migration engine's lost-data findings rendered under `data.migrations` in
//! their own serde shape with the lost-data exit class (8), the `unknown`
//! outcome with its scope (a completed bounded run, exit 0), and the labeled
//! usage errors: an omitted or unknown id is `bad_argument` exit 2 naming
//! the id and the known ids; a malformed `--values` spelling is
//! `bad_argument` exit 1; a parse/build failure is the shared labeled error
//! before any lens runs. The payload structs and the listing/outcome
//! renderers live in the [`report`] submodule.
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
//! lens's `Unknown { scope }`, never a silently ignored flag.

use crate::args::bad_arg;
use crate::cli::default_scope_caps;
use crate::dispatch::Ctx;
use crate::render::{error_envelope, LabeledError};
use crate::run::load_model;
use catherine_analysis::pattern::{self, LensError, LensInput, LensOutcome};
use std::io::Write;

mod command;
mod report;

pub use command::command;

/// One pattern invocation's own flags; the output plumbing travels in
/// the run context.
pub(crate) struct PatternRun<'a> {
    /// The model document the lens runs over.
    pub(crate) model_path: &'a str,
    /// The `--id` lens, when one was named.
    pub(crate) id: Option<&'a str>,
    /// The `--values` specs, spelling-checked here and typed by the lens.
    pub(crate) values: &'a [String],
    /// The `--list` switch: the registry listing, no lens run.
    pub(crate) list: bool,
}

/// Execute the named lens over the model, or list the registry. Returns
/// the process exit code. `quiet` suppresses human lines only.
pub fn run_pattern(run: PatternRun<'_>, ctx: &mut Ctx<'_, impl Write, impl Write>) -> i32 {
    if run.list {
        return report::list_registry(ctx);
    }
    let Some(id) = run.id else {
        let error = bad_arg(
            "`pattern` requires `--id <pattern>` or `--list`".to_string(),
            "run `catherine pattern <model.ct.json> --list` to see the registered ids",
        );
        return emit_usage_error(&error, ctx);
    };
    let model = match load_model(run.model_path) {
        Ok(model) => model,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    if let Err(labeled) = check_values_syntax(run.values) {
        return ctx.emit_error(&labeled);
    }
    run_lens(id, &model, run.values, ctx)
}

/// Run one lens over the loaded model and render its outcome. Split from
/// [`run_pattern`] so the dispatch keeps the gate's function-length bound.
fn run_lens(
    id: &str,
    model: &catherine_core::model::Model,
    values: &[String],
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let input = LensInput { model, values };
    match pattern::run(id, &input, &default_scope_caps()) {
        Ok(LensOutcome::Findings(findings)) => report::report_findings(id, &findings, model, ctx),
        Ok(LensOutcome::Unknown { scope }) => report::report_unknown(id, &scope, ctx),
        Err(error) => {
            let labeled = unknown_pattern(&error);
            emit_usage_error(&labeled, ctx)
        }
    }
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
/// usage class, which the shared [`Ctx::emit_error`] fixed at 1).
fn emit_usage_error(labeled: &LabeledError, ctx: &mut Ctx<'_, impl Write, impl Write>) -> i32 {
    if ctx.json {
        let warnings = ctx.take_warnings();
        let envelope = error_envelope(labeled, warnings);
        match serde_json::to_string(&envelope) {
            Ok(text) => {
                let _ = writeln!(ctx.stderr, "{text}");
            }
            Err(error) => {
                let _ = writeln!(
                    ctx.stderr,
                    "catherine: cannot write error envelope: {error}"
                );
            }
        }
    } else {
        let _ = writeln!(ctx.stderr, "{}", labeled.human_head());
        let _ = writeln!(ctx.stderr, "  hint: {}", labeled.hint);
    }
    2
}

#[cfg(test)]
mod tests;
