//! Purpose: the analysis runners (P4, task 11) — every runner drives
//! its analysis op once per invocation and renders.
//!
//! Responsibilities: the same stream discipline as [`crate::run::run`]
//! (payloads to stdout, every error to stderr, exits from the finding
//! classes or 0 for completed bounded runs); the shared findings-bearing
//! runner; the query traversal's labeled `unknown_seed` error.
//!
//! Rationale: no second pipeline — the same loader, the same envelope
//! constructors, the same exit decision as the check path; a divergence
//! between the surfaces would be a re-run-semantics bug.

use crate::args::{parse_direction, parse_observations, parse_rows, parse_seeds};
use crate::render::{self, LabeledError};
use crate::run::{emit_error, exit_code, load_model, write_envelope};
use catherine_analysis::bounds::{ScopeCaps, SearchBounds};
use catherine_analysis::distinguish;
use catherine_analysis::query::{self, Direction, NodeRef, QueryError};
use catherine_analysis::rewrite;
use catherine_analysis::search::{self, SearchOutcome};
use catherine_core::findings::Finding;
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Schema};
use genesis::envelope::Warning;
use std::collections::BTreeMap;
use std::io::Write;

/// `search`: bounds are flag inputs; the outcome decides the exit — a
/// contradiction is a finding-bearing run (exit 5 through the existing
/// decision), a witness or UNKNOWN completes within its bounds (exit 0,
/// outcome in the payload; the UNKNOWN payload names its epistemic
/// state, which is not "clean").
#[allow(clippy::too_many_arguments)]
pub fn run_search(
    model_path: &str,
    rows: &[String],
    max_candidates: usize,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let mut max_rows = match parse_rows(rows) {
        Ok(max_rows) => max_rows,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let mut warnings = warnings;
    prune_undeclared_rows(&model.schema, &mut max_rows, &mut warnings);
    let bound = SearchBounds {
        max_rows,
        max_candidates,
    };
    let report = search::search(&model.schema, &bound);
    if json {
        let envelope = render::search_envelope(&report, warnings);
        write_envelope(&envelope, stdout, stderr, "search");
    } else if !quiet {
        for line in render::render_search(&report, &model.schema) {
            let _ = writeln!(stdout, "{line}");
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
fn prune_undeclared_rows(
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

/// `redundant`: findings-bearing op — exits through the existing
/// min-over-class decision (7 when any redundancy is proven, 0 when
/// none is).
#[allow(clippy::too_many_arguments)]
pub fn run_redundant(
    model_path: &str,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    findings_runner(
        model_path,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
        |schema| rewrite::redundant(schema, &ScopeCaps::default()),
    )
}

/// `underconstraint`: findings-bearing op — exits 6 when a hypothesis is
/// raised, 0 when the schema has no unconstrained parallel paths within
/// the cap.
#[allow(clippy::too_many_arguments)]
pub fn run_underconstraint(
    model_path: &str,
    max_path_length: usize,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    findings_runner(
        model_path,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
        |schema| {
            catherine_analysis::parallel::underconstraint(
                schema,
                &ScopeCaps {
                    max_path_length,
                    ..ScopeCaps::default()
                },
            )
        },
    )
}

/// The shared findings-bearing analysis runner: load once, run the op
/// over the schema, render findings with the §5 renderer (labels
/// intact), exit through the pure decision.
#[allow(clippy::too_many_arguments)]
fn findings_runner(
    model_path: &str,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    op: impl Fn(&Schema) -> Vec<Finding>,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let findings = op(&model.schema);
    if json {
        let envelope = render::findings_envelope(&findings, warnings);
        write_envelope(&envelope, stdout, stderr, "findings");
    } else if !quiet {
        for line in render::render_findings(&findings, &model.schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    exit_code(&findings)
}

/// `distinguish`: completes within its bounds → exit 0 with the outcome
/// payload; input errors (malformed or non-composing observations,
/// out-of-range rows) are labeled errors, exit 1.
#[allow(clippy::too_many_arguments)]
pub fn run_distinguish(
    model_path: &str,
    second_path: &str,
    observe: &[String],
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let second = match load_model(second_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    if let Some(labeled) = schema_mismatch(&second.schema, &model.schema, second_path) {
        return emit_error(&labeled, json, warnings, stdout, stderr);
    }
    let observations = match parse_observations(observe, &model.schema, &model.instance) {
        Ok(observations) => observations,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let typed_a = TypedInstance::build(&model.schema, &model.instance);
    let typed_b = TypedInstance::build(&model.schema, &second.instance);
    let report = distinguish::distinguish(&typed_a, &typed_b, &observations);
    if json {
        let envelope = render::distinguish_envelope(&report, warnings);
        write_envelope(&envelope, stdout, stderr, "distinguish");
    } else if !quiet {
        for line in render::render_distinguish(&report, &model.schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    0
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

/// `query`: the ported traversal — exit 0 with the node list; an
/// unknown seed is a labeled error (`unknown_seed`), exit 1.
#[allow(clippy::too_many_arguments)]
pub fn run_query(
    model_path: &str,
    seeds: &[String],
    follow: &[String],
    direction: &str,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let (walk, seeds) = match parse_direction(direction)
        .and_then(|walk| parse_seeds(seeds).map(|seeds| (walk, seeds)))
    {
        Ok(pair) => pair,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let reached = query_nodes(&model, walk, seeds, follow);
    let ordered: Vec<NodeRef> = match reached {
        Ok(nodes) => nodes.to_vec(),
        Err(errors) => {
            return emit_error(&unknown_seed_error(errors), json, warnings, stdout, stderr)
        }
    };
    render_query_outcome(
        direction, follow, &ordered, json, quiet, warnings, stdout, stderr,
    )
}

/// The ported traversal over the loaded model: seeds and follow morphisms
/// into the query op; op-level errors (unknown seeds) come back labeled.
fn query_nodes(
    model: &catherine_core::model::Model,
    walk: Direction,
    seeds: Vec<NodeRef>,
    follow: &[String],
) -> Result<Vec<NodeRef>, Vec<QueryError>> {
    let morphisms: Vec<Ident> = follow.iter().map(|name| Ident(name.clone())).collect();
    let typed = TypedInstance::build(&model.schema, &model.instance);
    let reached = query::run(
        &query::Query {
            direction: walk,
            seeds: seeds.into_iter().collect(),
            morphisms: morphisms.into_iter().collect(),
        },
        &model.schema,
        &typed,
    );
    reached.map(|nodes| nodes.into_iter().collect())
}

/// The query op's error as a labeled CLI error: `unknown_seed`, the op's
/// messages joined, and the seed-spelling hint.
fn unknown_seed_error(errors: Vec<QueryError>) -> LabeledError {
    LabeledError {
        code: "unknown_seed".to_string(),
        line: None,
        column: None,
        message: errors
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; "),
        hint: "name seeds as object#row with rows that exist in the instance",
    }
}

/// The query success path: the envelope under `--json`, the node lines
/// in human mode; a completed traversal exits 0.
#[allow(clippy::too_many_arguments)]
fn render_query_outcome(
    direction: &str,
    follow: &[String],
    ordered: &[NodeRef],
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let morphism_names: Vec<String> = follow.to_vec();
    if json {
        let envelope = render::query_envelope(direction, &morphism_names, ordered, warnings);
        write_envelope(&envelope, stdout, stderr, "query");
    } else if !quiet {
        for line in render::render_query(direction, ordered) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    0
}
