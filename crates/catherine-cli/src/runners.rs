//! Purpose: the analysis runners (P4, task 11) — every runner drives
//! its analysis op once per invocation and renders.
//!
//! Responsibilities: the same stream discipline as [`crate::run::run`]
//! (payloads to stdout, every error to stderr, exits from the finding
//! classes or 0 for completed bounded runs); the shared findings-bearing
//! runner; the per-run warning printer shared with `diff` and the
//! composition paths.
//!
//! Rationale: no second pipeline — the same loader, the same envelope
//! constructors, the same exit decision as the check path; a divergence
//! between the surfaces would be a re-run-semantics bug. The per-run
//! settings and streams travel in one place, [`crate::dispatch::Ctx`],
//! so the runners' flag inputs stay inside the parameter cap.

use crate::args::{parse_observations, parse_rows};
use crate::dispatch::Ctx;
use crate::render::{self, LabeledError};
use crate::run::{exit_code, load_model, write_envelope};
use catherine_analysis::bounds::{ScopeCaps, SearchBounds};
use catherine_analysis::distinguish;
use catherine_analysis::rewrite;
use catherine_analysis::search::{self, SearchOutcome, SearchReport};
use catherine_core::findings::Finding;
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Schema};
use genesis::envelope::Warning;
use std::collections::BTreeMap;
use std::io::Write;

mod agree;
mod naturality;
mod naturality_errors;
mod query;
mod underconstraint;
pub use agree::run_agree;
pub use naturality::run_naturality;
pub use query::{run_query, QueryFlags};

/// catherine-uh0: run-level warnings reach stderr in human mode too —
/// the same `warning: <message>` line `main.rs` prints for config
/// warnings; JSON mode keeps them on the envelope only (pinned by
/// `negative_matrix.rs`).
pub(crate) fn print_run_warnings(json: bool, warnings: &[Warning], stderr: &mut impl Write) {
    if json {
        return;
    }
    for warning in warnings {
        let _ = writeln!(stderr, "warning: {}", warning.message);
    }
}

/// `search`: bounds are flag inputs; the outcome decides the exit — a
/// contradiction is a finding-bearing run (exit 5 through the existing
/// decision), a witness or UNKNOWN completes within its bounds (exit 0,
/// outcome in the payload; the UNKNOWN payload names its epistemic
/// state, which is not "clean").
pub fn run_search(
    model_path: &str,
    rows: &[String],
    max_candidates: usize,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    let mut max_rows = match parse_rows(rows) {
        Ok(max_rows) => max_rows,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    prune_undeclared_rows(&model.schema, &mut max_rows, &mut ctx.warnings);
    let bound = SearchBounds {
        max_rows,
        max_candidates,
    };
    let report = search::search(&model.schema, &bound);
    render_search_outcome(&report, &model.schema, ctx)
}

/// The render tail of [`run_search`]: run warnings, the `--json`
/// envelope or the §5 human lines, then the outcome decides the exit —
/// a contradiction is finding-bearing (5), anything else completes
/// within its bounds (0).
fn render_search_outcome(
    report: &SearchReport,
    schema: &Schema,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    print_run_warnings(ctx.json, &ctx.warnings, ctx.stderr);
    if ctx.json {
        let envelope = render::search_envelope(report, ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "search");
    } else if !ctx.quiet {
        for line in render::render_search(report, schema) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    match &report.outcome {
        SearchOutcome::Contradiction { finding } => exit_code(std::slice::from_ref(finding)),
        _ => 0,
    }
}

/// catherine-0kw: a bound over an undeclared object would be silently
/// dropped downstream (zero rows → UNKNOWN with empty scope); name each
/// ignored object on the warnings channel and drop it from the bound.
/// Exit semantics unchanged.
pub(crate) fn prune_undeclared_rows(
    schema: &Schema,
    max_rows: &mut BTreeMap<Ident, usize>,
    warnings: &mut Vec<Warning>,
) {
    let undeclared: Vec<Ident> = max_rows
        .keys()
        .filter(|object| !schema.objects.contains_key(*object))
        .cloned()
        .collect();
    for object in undeclared {
        warnings.push(render::unknown_rows_warning(&object.0));
        max_rows.remove(&object);
    }
}

/// catherine-348: a `--follow` name matching no declared arrow silently
/// contributes no edges downstream (the donor's subgraph-restriction
/// semantics); name each ignored arrow on the warnings channel — once
/// per distinct name. Traversal semantics and exit codes unchanged.
pub(super) fn prune_undeclared_follow(
    schema: &Schema,
    follow: &[String],
    warnings: &mut Vec<Warning>,
) {
    let mut named: Vec<&str> = Vec::new();
    for name in follow {
        let declared = schema.arrows.contains_key(&Ident(name.clone()));
        if !declared && !named.contains(&name.as_str()) {
            named.push(name);
            warnings.push(render::unknown_follow_warning(name));
        }
    }
}

/// `redundant`: findings-bearing op — exits through the existing
/// min-over-class decision (7 when any redundancy is proven, 0 when
/// none is).
pub fn run_redundant(model_path: &str, ctx: &mut Ctx<'_, impl Write, impl Write>) -> i32 {
    findings_runner(model_path, ctx, |schema| {
        rewrite::redundant(schema, &ScopeCaps::default())
    })
}

/// `underconstraint`: findings-bearing op — exits 6 when a hypothesis is
/// raised, 0 when the schema has no unconstrained parallel paths within
/// the cap. One path is the unchanged single-document run; two or more
/// are glued first, so a question that exists only at the seam carries
/// source attribution.
pub fn run_underconstraint(
    model_paths: &[String],
    max_path_length: usize,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    if model_paths.len() > 1 {
        return underconstraint::composition(model_paths, max_path_length, ctx);
    }
    let model_path = model_paths.first().map(String::as_str).unwrap_or_default();
    findings_runner(model_path, ctx, underconstraint_op(max_path_length))
}

/// The `--max-path-length` flag bound into the op `findings_runner` runs.
fn underconstraint_op(max_path_length: usize) -> impl Fn(&Schema) -> Vec<Finding> {
    move |schema| {
        catherine_analysis::parallel::underconstraint(
            schema,
            &underconstraint::path_caps(max_path_length),
        )
    }
}

/// The shared findings-bearing analysis runner: load once, run the op
/// over the schema, render findings with the §5 renderer (labels
/// intact), exit through the pure decision.
fn findings_runner(
    model_path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
    op: impl Fn(&Schema) -> Vec<Finding>,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    let findings = op(&model.schema);
    if ctx.json {
        let envelope = render::findings_envelope(&findings, None, ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "findings");
    } else if !ctx.quiet {
        for line in render::render_findings(&findings, &model.schema) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    exit_code(&findings)
}

/// `distinguish`: completes within its bounds → exit 0 with the outcome
/// payload; input errors (malformed or non-composing observations,
/// out-of-range rows) are labeled errors, exit 1.
pub fn run_distinguish(
    model_path: &str,
    second_path: &str,
    observe: &[String],
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let (model, second) = match load_two(model_path, second_path, ctx) {
        Ok(pair) => pair,
        Err(exit) => return exit,
    };
    if let Some(labeled) = schema_mismatch(&second.schema, &model.schema, second_path) {
        return ctx.emit_error(&labeled);
    }
    let observations = match parse_observations(observe, &model.schema, &model.instance) {
        Ok(observations) => observations,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    let typed_a = TypedInstance::build(&model.schema, &model.instance);
    let typed_b = TypedInstance::build(&model.schema, &second.instance);
    let report = distinguish::distinguish(&typed_a, &typed_b, &observations);
    render_distinguish_outcome(&report, &model.schema, ctx)
}

/// The render tail of [`run_distinguish`]: the `--json` outcome payload
/// or the §5 human lines; a completed bounded run exits 0.
fn render_distinguish_outcome(
    report: &distinguish::DistinguishReport,
    schema: &Schema,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    if ctx.json {
        let envelope = render::distinguish_envelope(report, ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "distinguish");
    } else if !ctx.quiet {
        for line in render::render_distinguish(report, schema) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    0
}

/// Both documents through the shared loader; the first failure emits its
/// labeled error (which consumes the warnings) and returns its exit code.
#[allow(clippy::type_complexity)]
fn load_two(
    model_path: &str,
    second_path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> Result<(catherine_core::model::Model, catherine_core::model::Model), i32> {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return Err(ctx.emit_error(&labeled)),
    };
    let second = match load_model(second_path) {
        Ok(model) => model,
        Err(labeled) => return Err(ctx.emit_error(&labeled)),
    };
    Ok((model, second))
}

/// catherine-18t: the second document's schema was previously parsed and
/// dropped, silently reinterpreting its tables under the first document's
/// schema. A differing schema is a labeled input error, not a
/// schema-equivalence claim (that boundary stays intact).
fn schema_mismatch(
    second_schema: &Schema,
    model_schema: &Schema,
    second_path: &str,
) -> Option<LabeledError> {
    (second_schema != model_schema).then(|| LabeledError {
        code: "schema_mismatch".to_string(),
        line: None,
        column: None,
        message: format!("the second document's schema differs from the first (`{second_path}`)"),
        hint: "distinguish compares two instances of the same schema; give two documents with identical schemas",
    })
}
