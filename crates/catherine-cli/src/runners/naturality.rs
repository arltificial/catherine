//! Purpose: the `naturality` runner (openspec change
//! `add-instance-morphism`, task 7) — execute the source document's
//! declared `instance_map` against a same-schema target instance.
//!
//! Responsibilities: load both documents with their presentation names,
//! reuse `distinguish`'s schema refusal, report construction-time
//! resolution failures as labeled errors, warn on an absent map (never a
//! silent clean), warn per square undefined on both sides (an unexecuted
//! square is a non-answer, not a clean), render the sweep's findings
//! through the shared envelope/§5 renderers, and exit through the
//! existing class decision.
//!
//! Rationale: no second pipeline — the loader, envelope constructors, and
//! exit decision are the check path's; only the operation differs.

use super::naturality_errors::{migration_naturality_error, naturality_error};
use super::{print_run_warnings, schema_mismatch};
use crate::dispatch::Ctx;
use crate::render;
use crate::run::{exit_code, load_model_with_rows, write_envelope};
use catherine_analysis::naturality::{self, NaturalityError};
use catherine_core::findings::Finding;
use catherine_core::model::{Model, Schema};
use catherine_core::parse::RowNames;
use genesis::envelope::Warning;
use std::io::Write;

/// Run the naturality check. Identical schemas reuse the `distinguish`
/// refusal; target resolution is construction-time (`unknown_map_target`);
/// the sweep emits the existing `violation`/`executed` findings and exits
/// through the pure decision. An absent map warns (exit 0), never a
/// silent clean.
pub fn run_naturality(
    model_path: &str,
    second_path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    crate::composition::warn_duplicate_inputs(&[model_path, second_path], ctx);
    let Ok((source, source_names, target, target_names)) =
        load_both_documents(model_path, second_path, ctx)
    else {
        return 1;
    };
    dispatch_mode(
        (&source, &source_names, &target, &target_names),
        second_path,
        ctx,
    )
}

/// Both loaded documents, borrowed, for the mode dispatch and its
/// outcome paths.
type Docs<'a> = (&'a Model, &'a RowNames, &'a Model, &'a RowNames);

/// The mode dispatch once both documents load, split out of
/// `run_naturality` verbatim (mechanical lint refactor; no behavior
/// change): a source with a declared `schema_map` is a Δ migration
/// (catherine-kw9.2) — the second document must match the map's declared
/// target schema, and the sweep commutes operations through the row
/// migration. Without a `schema_map`, the classic same-schema
/// instance-map naturality applies unchanged.
fn dispatch_mode(
    docs: Docs<'_>,
    second_path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let (source, ..) = docs;
    if let Some(schema_map) = &source.schema_map {
        return run_migration_naturality(docs, second_path, schema_map, ctx);
    }
    naturality_outcome(docs, second_path, ctx)
}

/// Loads both documents up front (the classic path's paired loads).
/// A load failure emits its labeled error through the shared
/// `emit_error` path — which consumes the warnings — and yields `Err`;
/// success leaves the warnings untouched (exit code identical:
/// `emit_error` always exits 1).
fn load_both_documents(
    model_path: &str,
    second_path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> Result<(Model, RowNames, Model, RowNames), ()> {
    let (source, source_names) = load_or_emit(model_path, ctx)?;
    let (target, target_names) = load_or_emit(second_path, ctx)?;
    Ok((source, source_names, target, target_names))
}

/// One document through the shared loader with row names; a load failure
/// emits its labeled error (which consumes the warnings) and fails.
fn load_or_emit(
    path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> Result<(Model, RowNames), ()> {
    match load_model_with_rows(path) {
        Ok(pair) => Ok(pair),
        Err(labeled) => {
            ctx.emit_error(&labeled);
            Err(())
        }
    }
}

/// The migration dispatch branch (catherine-kw9.2): the second document
/// must match the declared `schema_map`'s target schema, then the sweep
/// commutes operations through the row migration.
fn run_migration_naturality(
    docs: Docs<'_>,
    second_path: &str,
    schema_map: &catherine_core::model::SchemaMap,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let (_, _, target, _) = docs;
    if let Some(labeled) = schema_mismatch(&target.schema, &schema_map.target, second_path) {
        return ctx.emit_error(&labeled);
    }
    migration_naturality_outcome(docs, second_path, ctx)
}

/// The migration-mode post-load outcome (catherine-kw9.2): absent
/// instance map → warning, resolution failure → labeled error,
/// otherwise migration findings through the shared render path.
fn migration_naturality_outcome(
    docs: Docs<'_>,
    second_path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let (source, source_names, target, target_names) = docs;
    if source.instance_map.is_none() {
        return missing_map(source, ctx);
    }
    match naturality::migration_naturality(source, target, source_names, target_names) {
        Ok(outcome) => emit_outcome(&outcome.findings, &outcome.undefined, &source.schema, ctx),
        Err(error) => ctx.emit_error(&migration_naturality_error(&error, second_path)),
    }
}

/// The post-load outcome: the `distinguish` schema refusal first (the
/// same-schema target must match before anything is classified — order
/// preserved from the dispatch site), then absent map → warning,
/// resolution failure → labeled error, otherwise findings through the
/// shared render path.
fn naturality_outcome(
    docs: Docs<'_>,
    second_path: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let (source, source_names, target, target_names) = docs;
    if let Some(labeled) = schema_mismatch(&target.schema, &source.schema, second_path) {
        return ctx.emit_error(&labeled);
    }
    if source.instance_map.is_none() {
        return missing_map(source, ctx);
    }
    match naturality::naturality(source, target, source_names, target_names) {
        Ok(outcome) => emit_outcome(&outcome.findings, &outcome.undefined, &source.schema, ctx),
        Err(NaturalityError::MissingInstanceMap) => missing_map(source, ctx),
        Err(error) => ctx.emit_error(&naturality_error(&error, second_path)),
    }
}

/// The success path shared by both modes: name every square undefined on
/// both sides on the warnings channel (the warning never changes the
/// exit class), then findings through the shared render path.
fn emit_outcome(
    findings: &[Finding],
    undefined: &[naturality::UndefinedSquare],
    schema: &Schema,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    warn_undefined_squares(undefined, &mut ctx.warnings);
    render_findings(findings, schema, ctx)
}

/// Name every square undefined on both sides on the warnings channel; the
/// warning never changes the exit class (findings alone decide it).
fn warn_undefined_squares(undefined: &[naturality::UndefinedSquare], warnings: &mut Vec<Warning>) {
    for square in undefined {
        warnings.push(render::unresolved_naturality_square_warning(
            &square.arrow.0,
            square.source_row.0,
        ));
    }
}

/// The absent-map path: name it on the warnings channel (exit 0), never
/// return a silent clean.
fn missing_map(source: &Model, ctx: &mut Ctx<'_, impl Write, impl Write>) -> i32 {
    ctx.warnings.push(render::missing_instance_map_warning());
    render_findings(&[], &source.schema, ctx)
}

/// The success path: findings under the shared envelope (or the §5 human
/// lines), exit through the existing class decision.
fn render_findings(
    findings: &[Finding],
    schema: &Schema,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    print_run_warnings(ctx.json, &ctx.warnings, ctx.stderr);
    if ctx.json {
        let envelope = render::findings_envelope(findings, None, ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "naturality");
    } else if !ctx.quiet {
        for line in render::render_findings(findings, schema) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    exit_code(findings)
}
