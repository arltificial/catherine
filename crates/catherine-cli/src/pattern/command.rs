//! Purpose: dispatch forwarding for `catherine pattern` (beads
//! `catherine-mx1.2`) — the wrapper that hands the parsed command and the
//! shared run context to [`super::run_pattern`].
//!
//! Responsibilities: destructure [`crate::cli::Commands::Pattern`] and call
//! the runner with the context's settings, warnings, and streams.
//!
//! Rationale: the wrapper is the same command seam, not a second pipeline;
//! it lives beside the runner so `main.rs` keeps its dispatch thin and
//! under the file-size gate. The `unreachable!` arm documents that the
//! dispatcher routes only the `Pattern` variant here.

use crate::cli;
use crate::Ctx;
use std::io::Write;

/// Forward [`super::run_pattern`] on the shared execution path.
pub fn command<O: Write, E: Write>(command: &cli::Commands, ctx: &mut Ctx<'_, O, E>) -> i32 {
    let cli::Commands::Pattern {
        model,
        id,
        values,
        list,
    } = command
    else {
        unreachable!("dispatch routes only Commands::Pattern here");
    };
    super::run_pattern(
        model,
        id.as_deref(),
        values,
        *list,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}
