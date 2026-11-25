//! Purpose: the pattern run's reporting surface (beads `catherine-mx1.2`)
//! — the two payload structs (`--list`'s registry view, the run's
//! outcome view) and the renderers that print them.
//!
//! Responsibilities: the registry listing (id, GoF name, abstraction, law
//! kind, scope; no model read, no lens run), the completed-run report
//! (envelope or human lines, then the exit decision), and the bounded
//! UNKNOWN report (`outcome: "unknown"` with the exact scope, exit 0).
//!
//! Rationale: one rendering seam beside the runner — the run's two
//! finding channels are the two kernel surfaces, so each keeps its own
//! payload key (`data.findings` checks, `data.migrations` lost data) and
//! its own exit contract: a migration finding is always the table's
//! lost-data class, which outranks the checks' minimum because the
//! migration never had a chance to be anything else. Listing runs no lens
//! and prints every metadata field, so the scope string's honesty
//! boundary travels with the id.

use crate::cli::LOST_DATA_EXIT_CODE;
use crate::dispatch::Ctx;
use crate::render;
use crate::run::{exit_code, write_envelope};
use catherine_analysis::pattern::{self, LensFindings, PatternLens};
use catherine_core::findings::Finding;
use catherine_core::migrate::{MigrationFinding, MigrationFindingKind};
use catherine_core::model::Schema;
use genesis::envelope::{Envelope, EnvelopeKind, Warning};
use serde::Serialize;
use std::io::Write;

/// One registry entry as `--list` emits it.
#[derive(Serialize)]
pub(super) struct LensView {
    /// Exact registry id.
    pub id: &'static str,
    /// GoF pattern name.
    pub name: &'static str,
    /// Universal abstraction.
    pub abstraction: &'static str,
    /// `identity | totality | coverage | order | delta`.
    pub law: &'static str,
    /// The disposition row's law scope, verbatim.
    pub scope: &'static str,
}

/// The list payload: the registry in its fixed order.
#[derive(Serialize)]
pub(super) struct PatternListData {
    /// Every registered lens.
    pub lenses: Vec<LensView>,
}

/// The run payload: outcome, the unknown scope when present, the checks
/// under `findings`, and the migration engine's lost data under
/// `migrations`.
#[derive(Serialize)]
pub(super) struct PatternData<'a> {
    /// `"analyzed"` or `"unknown"`.
    pub outcome: &'static str,
    /// The exact surface a lens could not decide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The kernel's taxonomy findings verbatim.
    pub findings: &'a [Finding],
    /// The migration engine's lost-data findings verbatim — the migrate
    /// surface's own `label` + `kind` + `witness` encoding.
    pub migrations: &'a [MigrationFinding],
}

/// `--list`: the registry in fixed order, no model read, no lens run.
pub(super) fn list_registry(ctx: &mut Ctx<'_, impl Write, impl Write>) -> i32 {
    let lenses = pattern::registry();
    if ctx.json {
        let warnings = ctx.take_warnings();
        let envelope = Envelope::success(
            render::CLI_VERSION,
            EnvelopeKind::List,
            PatternListData {
                lenses: lenses.iter().map(|lens| view(lens.meta())).collect(),
            },
            warnings,
            Vec::new(),
        );
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "pattern");
    } else if !ctx.quiet {
        for line in list_lines(lenses) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    0
}

/// The human list: a count head, then one metadata line per lens.
pub(super) fn list_lines(lenses: &[&'static dyn PatternLens]) -> Vec<String> {
    let head = format!("pattern: {} registered lens(es)", lenses.len());
    let lines = lenses.iter().map(|lens| {
        let meta = lens.meta();
        format!(
            "{}  {} — {} · {} · {}",
            meta.id,
            meta.name,
            meta.abstraction,
            meta.law.as_str(),
            meta.scope
        )
    });
    std::iter::once(head).chain(lines).collect()
}

/// One lens's metadata as the list view.
fn view(meta: &catherine_analysis::pattern::PatternMeta) -> LensView {
    LensView {
        id: meta.id,
        name: meta.name,
        abstraction: meta.abstraction,
        law: meta.law.as_str(),
        scope: meta.scope,
    }
}

/// Render a completed lens run: the envelope or one head plus the check
/// renderer's witness lines and the migration renderer's lost-data lines,
/// then the exit decision. Any migration finding is the lost-data class
/// (8); otherwise the shared min-over-class decision applies to the checks.
pub(super) fn report_findings(
    id: &str,
    findings: &LensFindings,
    model: &catherine_core::model::Model,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    ctx.print_warnings();
    if ctx.json {
        let warnings = ctx.take_warnings();
        report_findings_json(findings, warnings, ctx);
    } else if !ctx.quiet {
        report_findings_human(id, findings, model, ctx.stdout);
    }
    outcome_exit(findings)
}

/// The `--json` branch: the findings envelope, `pattern` named as the
/// emitting surface.
fn report_findings_json(
    findings: &LensFindings,
    warnings: Vec<Warning>,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) {
    let envelope = Envelope::success(
        render::CLI_VERSION,
        EnvelopeKind::Check,
        PatternData {
            outcome: "analyzed",
            scope: None,
            findings: &findings.checks,
            migrations: &findings.migrations,
        },
        warnings,
        Vec::new(),
    );
    write_envelope(&envelope, ctx.stdout, ctx.stderr, "pattern");
}

/// The human branch: one head with the total, then the check renderer's
/// witness lines and the migration renderer's lost-data lines.
fn report_findings_human(
    id: &str,
    findings: &LensFindings,
    model: &catherine_core::model::Model,
    stdout: &mut impl Write,
) {
    let total = findings.checks.len() + findings.migrations.len();
    let _ = writeln!(stdout, "pattern {id}: {total} finding(s)");
    let checks = render::render_findings(&findings.checks, &model.schema);
    for line in checks {
        let _ = writeln!(stdout, "{line}");
    }
    let migrations = render_migrations(&findings.migrations, &model.schema);
    for line in migrations {
        let _ = writeln!(stdout, "{line}");
    }
}

/// The exit decision over a completed run: a migration finding is the
/// lost-data class (the table's lowest severity, and never outranked here
/// because the lost data is the run's own result); otherwise the shared
/// min-over-class decision over the checks.
pub(super) fn outcome_exit(findings: &LensFindings) -> i32 {
    if findings.migrations.is_empty() {
        exit_code(&findings.checks)
    } else {
        LOST_DATA_EXIT_CODE
    }
}

/// The lost-data human lines, the migrate renderer's spelling:
/// `lost-row: Object#n`, `lost-cell: arrow at Object#n` (the owner object
/// resolved from the source schema, the same totality fallback).
pub(super) fn render_migrations(findings: &[MigrationFinding], schema: &Schema) -> Vec<String> {
    findings
        .iter()
        .map(|finding| match &finding.kind {
            MigrationFindingKind::LostRow { object, row } => {
                format!("lost-row: {}#{}", object.0, row.0)
            }
            MigrationFindingKind::LostCell { arrow, row } => {
                let owner = schema
                    .arrows
                    .get(arrow)
                    .map(|decl| format!("{}#{}", decl.src.0, row.0))
                    .unwrap_or_else(|| row.0.to_string());
                format!("lost-cell: {} at {}", arrow.0, owner)
            }
            MigrationFindingKind::Weakened {
                constraint,
                object,
                row,
            } => format!("weakened: {constraint} — witness {}#{}", object.0, row.0),
        })
        .collect()
}

/// Render a bounded non-answer: `outcome: "unknown"` plus the exact scope.
/// A completed bounded run exits 0 and is never rendered as clean.
pub(super) fn report_unknown(
    id: &str,
    scope: &str,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    ctx.print_warnings();
    if ctx.json {
        let warnings = ctx.take_warnings();
        let envelope = Envelope::success(
            render::CLI_VERSION,
            EnvelopeKind::Check,
            PatternData {
                outcome: "unknown",
                scope: Some(scope.to_string()),
                findings: &[],
                migrations: &[],
            },
            warnings,
            Vec::new(),
        );
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "pattern");
    } else if !ctx.quiet {
        let _ = writeln!(ctx.stdout, "pattern {id}: unknown — {scope}");
    }
    0
}
