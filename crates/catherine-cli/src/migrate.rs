//! Purpose: the `migrate` runner (add-delta-migration task 7) — one
//! execution path for the Δ migration surface: load, reindex along the
//! document's declared schema map, render the outcome, and decide the
//! exit.
//!
//! Responsibilities: the same stream discipline as [`crate::run::run`]
//! and the analysis runners (the envelope/payload to stdout, every
//! error to stderr); the migration exit decision — lost data is a
//! finding-bearing run (exit 8, [`crate::cli::LOST_DATA_EXIT_CODE`]), a
//! lossless migration is clean (exit 0); the labeled `no_schema_map`
//! error for a document without a declared map.
//!
//! Rationale: no second pipeline — the same loader, the same envelope
//! constructors, the same error emission as every other surface. A
//! document without a `schema_map` is not silently an empty migration
//! (that would be a fail-open seam): `reindex` returning `None` means
//! "no migration was declared", and it is reported as a labeled error,
//! the same honesty the negative matrix demands of every surface. Exit
//! codes are computed from the findings present, never from rendered
//! text.

use crate::cli::LOST_DATA_EXIT_CODE;
use crate::render::{self, LabeledError};
use crate::run::{emit_error, load_model, write_envelope};
use catherine_analysis::bounds::{ScopeCaps, SearchBounds};
use catherine_analysis::schema_diff;
use catherine_core::findings::EpistemicLabel;
use catherine_core::migrate::{self, MigrationFinding, MigrationFindingKind};
use catherine_core::model::{Ident, Model, SchemaMap};
use genesis::envelope::Warning;
use std::collections::BTreeMap;
use std::io::Write;

/// Execute `model_path`'s declared migration once and render. Returns
/// the process exit code. `quiet` suppresses the human lost-data lines
/// (never the exit decision, never errors, never the machine envelope).
#[allow(clippy::too_many_arguments)]
pub fn run_migrate(
    model_path: &str,
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
    let Some(outcome) = migrate::reindex(&model) else {
        return emit_error(&no_schema_map(model_path), json, warnings, stdout, stderr);
    };
    let mut outcome = outcome;
    weaken_constraints(&model, &mut outcome);
    print_warnings(json, &warnings, stderr);
    if json {
        let envelope = render::migrate_envelope(&outcome, warnings);
        write_envelope(&envelope, stdout, stderr, "migrate");
    } else if !quiet {
        for line in render::render_migrate(&outcome, &model.schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    migration_exit(&outcome.lost)
}

/// The migration exit decision: lost data is a finding-bearing run —
/// the table's lost-data class; a lossless migration is clean. The
/// migration taxonomy has exactly one finding class, so the
/// min-over-class convention collapses to this presence test. The
/// constraint-level `weakened` findings ride the same class: a
/// weakened migration is a finding-bearing run, not a new exit.
fn migration_exit(lost: &[MigrationFinding]) -> i32 {
    if lost.is_empty() {
        0
    } else {
        LOST_DATA_EXIT_CODE
    }
}

/// The constraint-level signal (catherine-kpn.8, option (a)): run the
/// shared diff classifier (`schema_diff::dropped_constraints`) over the
/// document's accepted constraints translated through the declared map
/// into the embedded target schema, and append one `weakened` finding
/// per dropped constraint — the witness row names the constraint.
/// Rows and cells migrate exactly as [`migrate::reindex`] migrated
/// them; this only adds findings, re-sorted into the deterministic
/// order the kernel enumerates in.
fn weaken_constraints(model: &Model, outcome: &mut migrate::MigrationOutcome) {
    let Some(map) = &model.schema_map else {
        return;
    };
    let bound = weaken_bound(&model.instance, map);
    for dropped in schema_diff::dropped_constraints(&model.schema, &model.instance, map, &bound) {
        outcome.lost.push(MigrationFinding {
            label: EpistemicLabel::Executed,
            kind: MigrationFindingKind::Weakened {
                constraint: render::diff::spelled(&dropped.constraint),
                object: dropped.object,
                row: dropped.row,
            },
        });
    }
    outcome.lost.sort();
}

/// The scan's bound: the target vocabulary's objects, each with room
/// for the migrated data (the source rows its map preimage carries)
/// plus two separating rows — enough for the smallest witness of any
/// single-arrow-path constraint violation, never grown by the engine.
/// The candidate budget is the analysis default, the same one the
/// `diff` surface starts from.
fn weaken_bound(
    instance: &catherine_core::model::Instance,
    map: &SchemaMap,
) -> SearchBounds {
    let mut max_rows: BTreeMap<Ident, usize> = BTreeMap::new();
    for target_object in map.target.objects.keys() {
        let source_rows = map
            .objects
            .values()
            .filter_map(|image| image.as_ref())
            .filter(|image| *image == target_object)
            .map(|image| instance.rows.get(image).map(Vec::len).unwrap_or(0))
            .sum::<usize>();
        max_rows.insert(target_object.clone(), source_rows + 2);
    }
    SearchBounds {
        max_rows,
        max_candidates: ScopeCaps::default().max_candidates,
    }
}

/// catherine-alr: a document without a `schema_map` is not a migration —
/// labeled, never an empty success (the fail-open-seam rule).
fn no_schema_map(model_path: &str) -> LabeledError {
    LabeledError {
        code: "no_schema_map".to_string(),
        line: None,
        column: None,
        message: format!("`{model_path}` carries no `schema_map` — there is no migration to run"),
        hint: "add a total `schema_map` section (format_version 1.1), or use `catherine check` for plain model checking",
    }
}

/// Run-level warnings reach stderr in human mode too — the same
/// `warning: <message>` line the other runners print (the
/// catherine-uh0 discipline); JSON mode keeps them on the envelope only.
fn print_warnings(json: bool, warnings: &[Warning], stderr: &mut impl Write) {
    if json {
        return;
    }
    for warning in warnings {
        let _ = writeln!(stderr, "warning: {}", warning.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::findings::EpistemicLabel;
    use catherine_core::migrate::MigrationFindingKind;
    use catherine_core::model::{Ident, RowId};

    #[test]
    fn lossless_migration_is_clean() {
        assert_eq!(migration_exit(&[]), 0);
    }

    #[test]
    fn lost_data_is_a_finding_bearing_run() {
        let findings = vec![MigrationFinding {
            label: EpistemicLabel::Executed,
            kind: MigrationFindingKind::LostRow {
                object: Ident("Scratch".to_string()),
                row: RowId(0),
            },
        }];
        assert_eq!(migration_exit(&findings), LOST_DATA_EXIT_CODE);
    }
}
