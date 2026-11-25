//! Purpose: the `query` runner — the ported traversal over one loaded
//! model, with its flag parsing, the labeled `unknown_seed` error, and
//! the success rendering (envelope under `--json`, node lines otherwise).
//!
//! Responsibilities: parse the traversal's flags, prune undeclared
//! `--follow` names (catherine-348), run the query op, and render the
//! reached nodes under the shared stream discipline.
//!
//! Rationale: the same loader and envelope constructors as the other
//! runners — a divergence between the surfaces would be a
//! re-run-semantics bug.

use super::{print_run_warnings, prune_undeclared_follow};
use crate::args::{parse_direction, parse_seeds};
use crate::dispatch::Ctx;
use crate::render::{self, LabeledError};
use crate::run::{load_model, write_envelope};
use catherine_analysis::query::{self, Direction, NodeRef, QueryError};
use catherine_core::instance::TypedInstance;
use catherine_core::model::Ident;
use std::io::Write;

/// The traversal's flag inputs, grouped so [`run_query`] stays inside
/// the parameter cap: the parsed `--seed`s, the `--follow` names, and
/// the `--direction` spelling (re-emitted verbatim on the envelope).
pub struct QueryFlags<'a> {
    pub seeds: &'a [String],
    pub follow: &'a [String],
    pub direction: &'a str,
}

/// `query`: the ported traversal — exit 0 with the node list; an
/// unknown seed is a labeled error (`unknown_seed`), exit 1; a
/// `--follow` name matching no declared arrow is named on the warnings
/// channel (catherine-348), exit semantics unchanged.
pub fn run_query(
    model_path: &str,
    flags: QueryFlags<'_>,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    prune_undeclared_follow(&model.schema, flags.follow, &mut ctx.warnings);
    let (walk, seeds) = match parse_query_args(flags.direction, flags.seeds) {
        Ok(pair) => pair,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    let reached = query_nodes(&model, walk, seeds, flags.follow);
    let ordered: Vec<NodeRef> = match reached {
        Ok(nodes) => nodes.to_vec(),
        Err(errors) => return ctx.emit_error(&unknown_seed_error(errors)),
    };
    render_query_outcome(flags.direction, flags.follow, &ordered, ctx)
}

/// The traversal's flag inputs: the walk direction and the parsed seeds.
fn parse_query_args(
    direction: &str,
    seeds: &[String],
) -> Result<(Direction, Vec<NodeRef>), LabeledError> {
    parse_direction(direction).and_then(|walk| parse_seeds(seeds).map(|seeds| (walk, seeds)))
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
fn render_query_outcome(
    direction: &str,
    follow: &[String],
    ordered: &[NodeRef],
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let morphism_names: Vec<String> = follow.to_vec();
    print_run_warnings(ctx.json, &ctx.warnings, ctx.stderr);
    if ctx.json {
        let envelope =
            render::query_envelope(direction, &morphism_names, ordered, ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "query");
    } else if !ctx.quiet {
        for line in render::render_query(direction, ordered) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    0
}
