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
use crate::render;
use crate::run::{emit_error, exit_code, load_model_with_rows, write_envelope};
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
#[allow(clippy::too_many_arguments)]
pub fn run_naturality(
    model_path: &str,
    second_path: &str,
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    crate::composition::warn_duplicate_inputs(
        &[model_path, second_path],
        json,
        &mut warnings,
        stderr,
    );
    let Ok((source, source_names, target, target_names, warnings)) =
        load_both_documents(model_path, second_path, json, warnings, stdout, stderr)
    else {
        return 1;
    };
    dispatch_mode(
        &source,
        &source_names,
        &target,
        &target_names,
        second_path,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
    )
}

/// The mode dispatch once both documents load, split out of
/// `run_naturality` verbatim (mechanical lint refactor; no behavior
/// change): a source with a declared `schema_map` is a Δ migration
/// (catherine-kw9.2) — the second document must match the map's declared
/// target schema, and the sweep commutes operations through the row
/// migration. Without a `schema_map`, the classic same-schema
/// instance-map naturality applies unchanged.
#[allow(clippy::too_many_arguments)]
fn dispatch_mode(
    source: &Model,
    source_names: &RowNames,
    target: &Model,
    target_names: &RowNames,
    second_path: &str,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if let Some(schema_map) = &source.schema_map {
        return run_migration_naturality(
            source,
            source_names,
            target,
            target_names,
            second_path,
            schema_map,
            json,
            quiet,
            warnings,
            stdout,
            stderr,
        );
    }
    naturality_outcome(
        source,
        source_names,
        target,
        target_names,
        second_path,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
    )
}

/// Loads both documents up front (the classic path's paired loads).
/// A load failure emits its labeled error through the shared
/// `emit_error` path — which consumes the warnings — and yields `Err`;
/// success hands the warnings back untouched (exit code identical:
/// `emit_error` always exits 1).
#[allow(clippy::type_complexity)]
fn load_both_documents(
    model_path: &str,
    second_path: &str,
    json: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<
    (
        catherine_core::model::Model,
        catherine_core::parse::RowNames,
        catherine_core::model::Model,
        catherine_core::parse::RowNames,
        Vec<Warning>,
    ),
    (),
> {
    let (source, source_names) = match load_model_with_rows(model_path) {
        Ok(pair) => pair,
        Err(labeled) => {
            emit_error(&labeled, json, warnings, stdout, stderr);
            return Err(());
        }
    };
    let (target, target_names) = match load_model_with_rows(second_path) {
        Ok(pair) => pair,
        Err(labeled) => {
            emit_error(&labeled, json, warnings, stdout, stderr);
            return Err(());
        }
    };
    Ok((source, source_names, target, target_names, warnings))
}

/// The migration dispatch branch (catherine-kw9.2): the second document
/// must match the declared `schema_map`'s target schema, then the sweep
/// commutes operations through the row migration.
#[allow(clippy::too_many_arguments)]
fn run_migration_naturality(
    source: &Model,
    source_names: &RowNames,
    target: &Model,
    target_names: &RowNames,
    second_path: &str,
    schema_map: &catherine_core::model::SchemaMap,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if let Some(labeled) = schema_mismatch(&target.schema, &schema_map.target, second_path) {
        return emit_error(&labeled, json, warnings, stdout, stderr);
    }
    migration_naturality_outcome(
        source,
        source_names,
        target,
        target_names,
        second_path,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
    )
}

/// The migration-mode post-load outcome (catherine-kw9.2): absent
/// instance map → warning, resolution failure → labeled error,
/// otherwise migration findings through the shared render path.
#[allow(clippy::too_many_arguments)]
fn migration_naturality_outcome(
    source: &Model,
    source_names: &RowNames,
    target: &Model,
    target_names: &RowNames,
    second_path: &str,
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if source.instance_map.is_none() {
        return missing_map(source, json, quiet, warnings, stdout, stderr);
    }
    match naturality::migration_naturality(source, target, source_names, target_names) {
        Ok(outcome) => {
            warn_undefined_squares(&outcome.undefined, &mut warnings);
            render_findings(
                &outcome.findings,
                &source.schema,
                json,
                quiet,
                warnings,
                stdout,
                stderr,
            )
        }
        Err(error) => {
            let labeled = migration_naturality_error(&error, second_path);
            emit_error(&labeled, json, warnings, stdout, stderr)
        }
    }
}

/// The post-load outcome: the `distinguish` schema refusal first (the
/// same-schema target must match before anything is classified — order
/// preserved from the dispatch site), then absent map → warning,
/// resolution failure → labeled error, otherwise findings through the
/// shared render path.
#[allow(clippy::too_many_arguments)]
fn naturality_outcome(
    source: &Model,
    source_names: &RowNames,
    target: &Model,
    target_names: &RowNames,
    second_path: &str,
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if let Some(labeled) = schema_mismatch(&target.schema, &source.schema, second_path) {
        return emit_error(&labeled, json, warnings, stdout, stderr);
    }
    if source.instance_map.is_none() {
        return missing_map(source, json, quiet, warnings, stdout, stderr);
    }
    match naturality::naturality(source, target, source_names, target_names) {
        Ok(outcome) => {
            warn_undefined_squares(&outcome.undefined, &mut warnings);
            render_findings(
                &outcome.findings,
                &source.schema,
                json,
                quiet,
                warnings,
                stdout,
                stderr,
            )
        }
        Err(NaturalityError::MissingInstanceMap) => {
            missing_map(source, json, quiet, warnings, stdout, stderr)
        }
        Err(error) => {
            let labeled = naturality_error(&error, second_path);
            emit_error(&labeled, json, warnings, stdout, stderr)
        }
    }
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
fn missing_map(
    source: &Model,
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    warnings.push(render::missing_instance_map_warning());
    render_findings(&[], &source.schema, json, quiet, warnings, stdout, stderr)
}

/// The success path: findings under the shared envelope (or the §5 human
/// lines), exit through the existing class decision.
#[allow(clippy::too_many_arguments)]
fn render_findings(
    findings: &[Finding],
    schema: &Schema,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    print_run_warnings(json, &warnings, stderr);
    if json {
        let envelope = render::findings_envelope(findings, None, warnings);
        write_envelope(&envelope, stdout, stderr, "naturality");
    } else if !quiet {
        for line in render::render_findings(findings, schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    exit_code(findings)
}
