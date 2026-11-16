//! Purpose: the `naturality` runner (openspec change
//! `add-instance-morphism`, task 7) — execute the source document's
//! declared `instance_map` against a same-schema target instance.
//!
//! Responsibilities: load both documents with their presentation names,
//! reuse `distinguish`'s schema refusal, report construction-time
//! resolution failures as labeled errors, warn on an absent map (never a
//! silent clean), render the sweep's findings through the shared
//! envelope/§5 renderers, and exit through the existing class decision.
//!
//! Rationale: no second pipeline — the loader, envelope constructors, and
//! exit decision are the check path's; only the operation differs.

use super::{print_run_warnings, schema_mismatch};
use crate::render::{self, LabeledError};
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
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let (source, source_names) = match load_model_with_rows(model_path) {
        Ok(pair) => pair,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let (target, target_names) = match load_model_with_rows(second_path) {
        Ok(pair) => pair,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    if let Some(labeled) = schema_mismatch(&target.schema, &source.schema, second_path) {
        return emit_error(&labeled, json, warnings, stdout, stderr);
    }
    naturality_outcome(
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

/// The post-load outcome: absent map → warning, resolution failure →
/// labeled error, otherwise findings through the shared render path.
#[allow(clippy::too_many_arguments)]
fn naturality_outcome(
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
    if source.instance_map.is_none() {
        return missing_map(source, json, quiet, warnings, stdout, stderr);
    }
    match naturality::naturality(source, target, source_names, target_names) {
        Ok(findings) => render_findings(
            &findings,
            &source.schema,
            json,
            quiet,
            warnings,
            stdout,
            stderr,
        ),
        Err(NaturalityError::MissingInstanceMap) => {
            missing_map(source, json, quiet, warnings, stdout, stderr)
        }
        Err(error) => {
            let labeled = naturality_error(&error, second_path);
            emit_error(&labeled, json, warnings, stdout, stderr)
        }
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
        let envelope = render::findings_envelope(findings, warnings);
        write_envelope(&envelope, stdout, stderr, "naturality");
    } else if !quiet {
        for line in render::render_findings(findings, schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    exit_code(findings)
}

/// A construction-time resolution failure as the CLI's labeled error:
/// `unknown_map_target` naming the object, the mapped source row, and the
/// missing target row.
fn naturality_error(error: &NaturalityError, second_path: &str) -> LabeledError {
    match error {
        NaturalityError::UnknownMapTarget {
            object,
            source_row,
            target_row,
        } => LabeledError {
            code: "unknown_map_target".to_string(),
            line: None,
            column: None,
            message: format!(
                "instance map cell for `{}` row `{source_row}` names target row `{target_row}`, which `{second_path}` does not declare",
                object.0
            ),
            hint: "map every source row to a row the target instance declares, or fix the target document",
        },
        NaturalityError::MissingInstanceMap => LabeledError {
            code: "missing_instance_map".to_string(),
            line: None,
            column: None,
            message: "the source document declares no `instance_map`".to_string(),
            hint: "add an `instance_map` section to the source document (format 1.4)",
        },
        NaturalityError::IncompleteInstanceMap { object, row } => LabeledError {
            code: "incomplete_instance_map".to_string(),
            line: None,
            column: None,
            message: format!(
                "instance map does not cover row #{} of `{}`",
                row.0, object.0
            ),
            hint: "map every row of every schema object; the map must be total",
        },
    }
}
