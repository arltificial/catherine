//! Purpose: the migration payload (add-delta-migration task 7) — the
//! `migrate` envelope's `data` and the human rendering of a migration
//! outcome.
//!
//! Responsibilities: build the one migrate-envelope path (target rows
//! per object, the kernel's lost findings verbatim, the kernel's row
//! correspondence verbatim — no reshaping, the same discipline as
//! `data.findings`) and render the human lines (a summary head, then
//! one line per lost finding with its witness).
//!
//! Rationale: the kernel `MigrationFinding`/`row_map` encodings are the
//! contract — the CLI prints them, it never re-serializes them (the
//! `findings_envelope` precedent); a second encoding would drift from
//! the kernel the same way a second pipeline would. In human text the
//! `executed` label stays off the line (the check renderer's
//! byte-reproducibility rule); the witness facts name each lost element
//! countably — `object#row` for a dropped row, `arrow at object#row`
//! for a dropped cell, the owner object looked up from the source
//! schema the same way the §5 renderer resolves witnesses, with the
//! same totality fallbacks.

use super::CLI_VERSION;
use catherine_core::migrate::{MigrationFindingKind, MigrationOutcome};
use catherine_core::model::{Ident, RowId, Schema};
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use serde::Serialize;
use std::collections::BTreeMap;

/// The migrate payload: the migrated instance summary (target rows per
/// object), the loud lost-data enumeration, and the row correspondence —
/// `lost` and `row_map` are the kernel's own serializations verbatim.
#[derive(Serialize)]
pub struct MigrateData<'a> {
    /// Target rows per object (the migrated instance's row counts).
    pub rows: BTreeMap<String, usize>,
    /// One finding per dropped row or valued dropped cell, labeled
    /// `executed` — the kernel's `MigrationFinding` encoding verbatim.
    pub lost: &'a [catherine_core::migrate::MigrationFinding],
    /// Kept source rows → target rows, per target object, the kernel's
    /// correspondence verbatim (dropped objects are absent — their loss
    /// is in `lost`).
    pub row_map: &'a BTreeMap<Ident, BTreeMap<RowId, RowId>>,
}

/// The one migrate-envelope path: kind `Check` (the payload carries
/// findings, like every findings-bearing surface), empty hints, the
/// caller's warnings carried alongside the data.
pub fn migrate_envelope(
    outcome: &MigrationOutcome,
    warnings: Vec<Warning>,
) -> Envelope<MigrateData<'_>> {
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        MigrateData {
            rows: outcome
                .instance
                .rows
                .iter()
                .map(|(object, rows)| (object.0.clone(), rows.len()))
                .collect(),
            lost: &outcome.lost,
            row_map: &outcome.row_map,
        },
        warnings,
        Vec::new(),
    )
}

/// The migrate human lines: a summary head, then one line per lost
/// finding — `lost-row: Object#n` or `lost-cell: arrow at Object#n` (a
/// lossless migration reports zero findings, never silence about the
/// count).
pub fn render_migrate(outcome: &MigrationOutcome, source: &Schema) -> Vec<String> {
    let target_rows: usize = outcome.instance.rows.values().map(|rows| rows.len()).sum();
    let mut lines = vec![format!(
        "migrate: {} target row(s) across {} object(s), {} lost data finding(s)",
        target_rows,
        outcome.instance.rows.len(),
        outcome.lost.len()
    )];
    for finding in &outcome.lost {
        match &finding.kind {
            MigrationFindingKind::LostRow { object, row } => {
                lines.push(format!("lost-row: {}#{}", object.0, row.0));
            }
            MigrationFindingKind::LostCell { arrow, row } => {
                let owner = source
                    .arrows
                    .get(arrow)
                    .map(|decl| format!("{}#{}", decl.src.0, row.0))
                    .unwrap_or_else(|| row.0.to_string());
                lines.push(format!("lost-cell: {} at {}", arrow.0, owner));
            }
        }
    }
    lines
}
