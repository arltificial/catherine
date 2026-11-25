//! Purpose: the per-command forwarding layer behind the entry point —
//! the command match, the per-run settings/output-stream context, and
//! the thin per-command wrappers, all funneling into the shared
//! execution path.
//!
//! Responsibilities: [`dispatch_command`] matches one arm per surface;
//! [`Ctx`] carries the settings resolved by the entry point's
//! `resolve_settings` plus the stdout/stderr pair under the stream
//! discipline; each `*_command` helper forwards to its runner.

use catherine_core::findings::EpistemicLabel;
use genesis::envelope::Warning;
use std::io::Write;

use crate::composition;
use crate::{agree, cli, diff, migrate, naturality, pattern, rules, run, runners};

/// The command match: one arm per surface, each on the shared path.
pub(crate) fn dispatch_command(
    command: &cli::Commands,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    use cli::Commands as C;
    match command {
        C::Check { models } => check_command(run::Surface::Check, models, ctx),
        C::Findings { models } => check_command(run::Surface::Findings, models, ctx),
        C::Search { .. } => search_command(command, ctx),
        C::Redundant { model } => redundant_command(model, ctx),
        C::Underconstraint { .. } => underconstraint_command(command, ctx),
        C::Distinguish { .. } => distinguish_command(command, ctx),
        C::Naturality { model, second } => naturality::run(model, second, ctx),
        C::Agree { model, second } => agree::run(model, second, ctx),
        C::Merge { left, right } => merge_command(&left, &right, ctx),
        C::Diff { .. } => diff_command(command, ctx),
        C::Query { .. } => query_command(command, ctx),
        C::Migrate { model } => migrate_command(model, ctx),
        C::Rules { .. } => rules_command(command, ctx),
        C::Pattern { .. } => pattern::command(command, ctx),
    }
}

/// The `merge` arm's rendering: UNKNOWN identity-conflict cells name both
/// conflicting sides; a completed pushout reports merged counts; the tool
/// never auto-resolves a conflict (exit 3 on any cell, 1 on labeled errors).
fn merge_command(left: &str, right: &str, ctx: &mut Ctx<'_, impl Write, impl Write>) -> i32 {
    match composition::merge::run(left, right) {
        Ok(outcome) => {
            for cell in &outcome.cells {
                let witness = &cell.witness;
                let label = match cell.label {
                    EpistemicLabel::Executed => "executed",
                    EpistemicLabel::Hypothesized => "hypothesized",
                    EpistemicLabel::Unknown => "unknown",
                };
                let _ = writeln!(
                    ctx.stdout,
                    "{label} merge cell: '{}' conflicts — left: {}; right: {}",
                    witness.entity, witness.left, witness.right
                );
            }
            if let Some(glued) = &outcome.glued {
                let _ = writeln!(
                    ctx.stdout,
                    "merged: {} objects, {} arrows, {} equations",
                    glued.schema.objects.len(),
                    glued.schema.arrows.len(),
                    glued.schema.equations.len()
                );
            }
            if outcome.cells.is_empty() {
                0
            } else {
                3
            }
        }
        Err(err) => {
            let _ = writeln!(ctx.stderr, "{}", err.positioned_message());
            1
        }
    }
}

/// The per-run settings and output streams threaded through every
/// command handler: settings as resolved by the entry point's
/// `resolve_settings`, the config warnings, and the stdout/stderr pair
/// under the stream discipline.
pub(crate) struct Ctx<'a, O: Write, E: Write> {
    pub(crate) json: bool,
    pub(crate) quiet: bool,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) stdout: &'a mut O,
    pub(crate) stderr: &'a mut E,
}

impl<'a, O: Write, E: Write> Ctx<'a, O, E> {
    pub(crate) fn new(
        json: bool,
        quiet: bool,
        warnings: Vec<Warning>,
        stdout: &'a mut O,
        stderr: &'a mut E,
    ) -> Self {
        Ctx {
            json,
            quiet,
            warnings,
            stdout,
            stderr,
        }
    }
}

/// Forward [`runners::run_search`] on the shared execution path. The arm
/// above binds no fields, so `dispatch_command` stays inside the
/// function-line cap; this helper re-destructures `Commands::Search`.
fn search_command<O: Write, E: Write>(command: &cli::Commands, ctx: &mut Ctx<'_, O, E>) -> i32 {
    let cli::Commands::Search {
        model,
        rows,
        max_candidates,
    } = command
    else {
        unreachable!("dispatch routes only Commands::Search here");
    };
    runners::run_search(
        model,
        rows,
        *max_candidates,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`runners::run_redundant`] on the shared execution path.
fn redundant_command<O: Write, E: Write>(model: &str, ctx: &mut Ctx<'_, O, E>) -> i32 {
    runners::run_redundant(
        model,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`runners::run_underconstraint`] on the shared execution path.
/// The arm above binds no fields, so `dispatch_command` stays inside the
/// function-line cap; this helper re-destructures `Commands::Underconstraint`.
fn underconstraint_command<O: Write, E: Write>(
    command: &cli::Commands,
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
    let cli::Commands::Underconstraint {
        models,
        max_path_length,
    } = command
    else {
        unreachable!("dispatch routes only Commands::Underconstraint here");
    };
    runners::run_underconstraint(
        models,
        *max_path_length,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`runners::run_distinguish`] on the shared execution path.
/// The arm above binds no fields, so `dispatch_command` stays inside the
/// function-line cap; this helper re-destructures `Commands::Distinguish`.
fn distinguish_command<O: Write, E: Write>(
    command: &cli::Commands,
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
    let cli::Commands::Distinguish {
        model,
        second,
        observe,
    } = command
    else {
        unreachable!("dispatch routes only Commands::Distinguish here");
    };
    runners::run_distinguish(
        model,
        second,
        observe,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`diff::run_diff`] on the shared execution path. The arm
/// above binds no fields, so `dispatch_command` stays inside the
/// function-line cap; this helper re-destructures `Commands::Diff`.
fn diff_command<O: Write, E: Write>(command: &cli::Commands, ctx: &mut Ctx<'_, O, E>) -> i32 {
    let cli::Commands::Diff {
        model,
        second,
        rows,
        max_candidates,
    } = command
    else {
        unreachable!("dispatch routes only Commands::Diff here");
    };
    diff::run_diff(
        model,
        second,
        rows,
        *max_candidates,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`runners::run_query`] on the shared execution path. The arm
/// above binds no fields, so `dispatch_command` stays inside the
/// function-line cap; this helper re-destructures `Commands::Query`.
fn query_command<O: Write, E: Write>(command: &cli::Commands, ctx: &mut Ctx<'_, O, E>) -> i32 {
    let cli::Commands::Query {
        model,
        seed,
        follow,
        direction,
    } = command
    else {
        unreachable!("dispatch routes only Commands::Query here");
    };
    runners::run_query(
        model,
        seed,
        follow,
        direction,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// The check/findings pair: one execution path under two names.
fn check_command<O: Write, E: Write>(
    surface: run::Surface,
    models: &[String],
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
    run::run(
        surface,
        models,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`migrate::run_migrate`] on the shared execution path.
fn migrate_command<O: Write, E: Write>(model: &str, ctx: &mut Ctx<'_, O, E>) -> i32 {
    migrate::run_migrate(
        model,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`rules::run_rules`] on the shared execution path. The arm
/// above binds no fields, so `dispatch_command` stays inside the
/// function-line cap; this helper is reached only for `Commands::Rules`.
fn rules_command<O: Write, E: Write>(command: &cli::Commands, ctx: &mut Ctx<'_, O, E>) -> i32 {
    let cli::Commands::Rules {
        model,
        values,
        max_candidates,
        project,
        columns,
        action,
    } = command
    else {
        unreachable!("dispatch routes only Commands::Rules here");
    };
    rules::run_rules(
        model,
        values,
        *max_candidates,
        project.as_deref(),
        columns.as_deref(),
        action.as_deref(),
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}
