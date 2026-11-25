//! Purpose: the `agree` runner (beads epic `catherine-kw9`, ticket 7) —
//! compare two instances over their shared sub-schema and report every
//! cell-level disagreement.
//!
//! Responsibilities: load both documents with their presentation names,
//! run [`catherine_analysis::agree::agree`], render the disagreements
//! through the shared envelope/§5 stream discipline, and exit nonzero
//! when any disagreement exists. Unlike the `naturality` runner, no
//! schema refusal: schemas need only *share* a sub-schema, and the
//! check compares exactly the shared names.
//!
//! Rationale: no second pipeline — the loader, the envelope constructor,
//! and the warning channel are the check path's; only the operation and
//! payload differ (the analysis layer is serde-free, so the runner owns
//! the disagreements' JSON encoding, as the CLI owns every envelope).
//!
//! Anti-goal: agreement reports disagreements; it never merges or
//! rewrites either document.

use super::print_run_warnings;
use crate::composition::warn_duplicate_inputs;
use crate::run::{emit_error, load_model_with_rows, write_envelope};
use catherine_analysis::agree::{self, Disagreement};
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use std::io::Write;

/// Run the agreement check. Partial overlap is the point — one-sided
/// rows are silence, mismatches are witnesses; any disagreement exits
/// through the violation class (3).
#[allow(clippy::too_many_arguments)]
pub fn run_agree(
    model_path: &str,
    second_path: &str,
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    warn_duplicate_inputs(&[model_path, second_path], json, &mut warnings, stderr);
    let (a, names_a) = match load_model_with_rows(model_path) {
        Ok(pair) => pair,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let (b, names_b) = match load_model_with_rows(second_path) {
        Ok(pair) => pair,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let disagreements = agree::agree(&a, &b, &names_a, &names_b);
    emit_agreement(
        &disagreements,
        model_path,
        second_path,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
    )
}

/// The emit/envelope tail of [`run_agree`]: the warning channel, the
/// `--json` envelope or the §5 human lines, and the exit decision —
/// any disagreement exits through the violation class (3).
#[allow(clippy::too_many_arguments)]
fn emit_agreement(
    disagreements: &[Disagreement],
    model_path: &str,
    second_path: &str,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let clean = disagreements.is_empty();
    print_run_warnings(json, &warnings, stderr);
    if json {
        let envelope = Envelope::success(
            crate::render::CLI_VERSION,
            EnvelopeKind::Check,
            AgreeData {
                disagreements: disagreements.iter().map(Into::into).collect(),
            },
            warnings,
            Vec::new(),
        );
        write_envelope(&envelope, stdout, stderr, "agree");
    } else if !quiet {
        for line in render_disagreements(disagreements, model_path, second_path) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    if clean {
        0
    } else {
        3
    }
}

/// The agree payload: `data.disagreements`, each disagreement encoded by
/// a CLI-owned DTO — the analysis layer stays serde-free, and the CLI
/// owns every envelope.
#[derive(Serialize)]
struct AgreeData<'a> {
    disagreements: Vec<DisagreementDto<'a>>,
}

/// The wire shape of one disagreement: the analysis witness's fields
/// verbatim (`entity`/`key`/`arrow` spelled, values as name or null).
#[derive(Serialize)]
struct DisagreementDto<'a> {
    entity: &'a str,
    key: &'a str,
    arrow: &'a str,
    value_a: Option<&'a str>,
    value_b: Option<&'a str>,
}

impl<'a> From<&'a Disagreement> for DisagreementDto<'a> {
    fn from(witness: &'a Disagreement) -> Self {
        DisagreementDto {
            entity: &witness.entity.0,
            key: &witness.key,
            arrow: &witness.arrow.0,
            value_a: witness.value_a.as_ref().map(|value| value.0.as_str()),
            value_b: witness.value_b.as_ref().map(|value| value.0.as_str()),
        }
    }
}

/// The §5 human lines: one line per disagreement, naming both documents.
fn render_disagreements(disagreements: &[Disagreement], a_path: &str, b_path: &str) -> Vec<String> {
    disagreements
        .iter()
        .map(|witness| {
            format!(
                "disagreement: {}.{} {} = {} (in {a_path}) ≠ {} (in {b_path})",
                witness.entity.0,
                witness.key,
                witness.arrow.0,
                value(witness.value_a.as_ref()),
                value(witness.value_b.as_ref()),
            )
        })
        .collect()
}

/// A projected value's name, or `undefined` for a dangling cell.
fn value(value: Option<&Ident>) -> &str {
    value.map(|ident| ident.0.as_str()).unwrap_or("undefined")
}

use catherine_core::model::Ident;
use serde::Serialize;
