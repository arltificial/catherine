//! Purpose: the `naturality` command's forwarding wrapper (openspec change
//! `add-instance-morphism`) — a thin bridge from the shared dispatch
//! context to the analysis runner, split out of `main.rs` to keep the
//! entry point inside the file-size gate.
//!
//! Responsibilities: forward the resolved settings and streams to
//! [`crate::runners::run_naturality`]; nothing else.
//!
//! Rationale: one execution path — the wrapper carries no policy, so the
//! exit decision, envelope shape, and stream discipline cannot diverge
//! from the other analysis subcommands.

use crate::runners;
use crate::Ctx;
use std::io::Write;

/// Forward [`runners::run_naturality`] on the shared execution path.
pub(crate) fn run<O: Write, E: Write>(model: &str, second: &str, ctx: &mut Ctx<'_, O, E>) -> i32 {
    runners::run_naturality(
        model,
        second,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}
