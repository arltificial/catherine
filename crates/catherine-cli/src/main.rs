//! Purpose: the `catherine` entry point — CLI parsing with genesis
//! suggestions for unknown subcommands, optional `catherine.toml`
//! config (genesis config module, [`config`]), and the stream
//! discipline: findings to stdout, every error to stderr, exit codes
//! from the finding classes.
//!
//! Responsibilities: parse [`cli::Cli`]; on usage errors keep clap's
//! own output and exit code 2, augmented with a genesis did-you-mean
//! line for unknown subcommands; resolve the effective format and
//! verbosity from flags over config; dispatch each subcommand through
//! the one execution path in [`run`] / [`runners`].
//!
//! Rationale: config problems degrade to warnings (carried in the JSON
//! envelope's `warnings`, printed to stderr in human mode) instead of
//! masquerading as model failures, so exit 1 stays "parse/build failure
//! of the model" and nothing else. The dispatch is thin per-command
//! forwarding wrappers — every command funnels into the shared
//! execution path, never a second pipeline.

mod args;
mod cli;
mod config;
mod migrate;
mod render;
mod run;
mod runners;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::Parser;
use genesis::envelope::Warning;
use genesis::guide::{OutputFormat, Verbosity};
use genesis::suggestions::{CommandRegistry, SuggestionEngine};
use std::io::Write;
use std::process::exit;

/// The genesis suggestions line for an unknown subcommand, if any.
fn did_you_mean(err: &clap::error::Error) -> Option<String> {
    if err.kind() != ErrorKind::InvalidSubcommand {
        return None;
    }
    let name = match err.get(ContextKind::InvalidSubcommand)? {
        ContextValue::String(name) => name.clone(),
        _ => return None,
    };
    let mut registry = CommandRegistry::new();
    registry.register(
        "catherine",
        vec![
            "check".to_string(),
            "findings".to_string(),
            "search".to_string(),
            "redundant".to_string(),
            "underconstraint".to_string(),
            "distinguish".to_string(),
            "query".to_string(),
            "migrate".to_string(),
        ],
    );
    SuggestionEngine::new()
        .suggest_typo(&name, &registry)
        .map(|suggestion| suggestion.message())
}

/// Resolve the effective settings: flags over config, plus the config
/// warnings on the genesis warning channel (stderr in human mode, the
/// envelope's warnings in JSON mode).
fn resolve_settings(cli: &cli::Cli) -> (bool, bool, Vec<Warning>) {
    let (config, mut config_warnings) = config::load_config(std::path::Path::new("."));
    let format = config::resolve_format(cli, &config);
    let verbosity = config::resolve_verbosity(cli, &config);
    let json = format == OutputFormat::Json;
    let quiet = verbosity == Verbosity::Quiet;
    let warnings: Vec<Warning> = config_warnings
        .drain(..)
        .map(|message| render::config_warning(&message))
        .collect();
    if !json {
        let mut stderr = std::io::stderr();
        for warning in &warnings {
            let _ = writeln!(stderr, "warning: {}", warning.message);
        }
    }
    (json, quiet, warnings)
}

fn main() {
    let cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Usage errors keep clap's own code (2) and output stream;
            // the genesis suggestion only augments the message.
            if let Some(suggestion) = did_you_mean(&err) {
                let _ = writeln!(std::io::stderr(), "{suggestion}");
            }
            err.exit();
        }
    };

    let (json, quiet, warnings) = resolve_settings(&cli);
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let code = dispatch(
        &cli.command,
        json,
        quiet,
        warnings,
        &mut stdout,
        &mut stderr,
    );
    exit(code);
}

/// The subcommand dispatch: one thin forwarding wrapper per command,
/// all funneling into the shared execution path.
fn dispatch(
    command: &cli::Commands,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let ctx = &mut Ctx::new(json, quiet, warnings, stdout, stderr);
    use cli::Commands as C;
    match command {
        C::Check { model } => check_command(run::Surface::Check, model, ctx),
        C::Findings { model } => check_command(run::Surface::Findings, model, ctx),
        C::Search {
            model,
            rows,
            max_candidates,
        } => search_command(model, rows, *max_candidates, ctx),
        C::Redundant { model } => redundant_command(model, ctx),
        C::Underconstraint {
            model,
            max_path_length,
        } => underconstraint_command(model, *max_path_length, ctx),
        C::Distinguish {
            model,
            second,
            observe,
        } => distinguish_command(model, second, observe, ctx),
        C::Query {
            model,
            seed,
            follow,
            direction,
        } => query_command(model, seed, follow, direction, ctx),
        C::Migrate { model } => migrate_command(model, ctx),
    }
}

/// The per-run settings and output streams threaded through every
/// command handler: settings as resolved by [`resolve_settings`], the
/// config warnings, and the stdout/stderr pair under the stream
/// discipline.
struct Ctx<'a, O: Write, E: Write> {
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &'a mut O,
    stderr: &'a mut E,
}

impl<'a, O: Write, E: Write> Ctx<'a, O, E> {
    fn new(
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

/// Forward [`runners::run_search`] on the shared execution path.
fn search_command<O: Write, E: Write>(
    model: &str,
    rows: &[String],
    max_candidates: usize,
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
    runners::run_search(
        model,
        rows,
        max_candidates,
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
fn underconstraint_command<O: Write, E: Write>(
    model: &str,
    max_path_length: usize,
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
    runners::run_underconstraint(
        model,
        max_path_length,
        ctx.json,
        ctx.quiet,
        std::mem::take(&mut ctx.warnings),
        ctx.stdout,
        ctx.stderr,
    )
}

/// Forward [`runners::run_distinguish`] on the shared execution path.
fn distinguish_command<O: Write, E: Write>(
    model: &str,
    second: &str,
    observe: &[String],
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
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

/// Forward [`runners::run_query`] on the shared execution path.
fn query_command<O: Write, E: Write>(
    model: &str,
    seed: &[String],
    follow: &[String],
    direction: &str,
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
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
    model: &str,
    ctx: &mut Ctx<'_, O, E>,
) -> i32 {
    run::run(
        surface,
        model,
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn unknown_subcommand_gets_a_did_you_mean() {
        let err = cli::Cli::try_parse_from(["catherine", "chek", "m.ct.json"])
            .expect_err("unknown subcommand rejected");
        let suggestion = did_you_mean(&err).expect("suggestion for `chek`");
        assert!(suggestion.contains("check"), "{suggestion}");
    }

    #[test]
    fn help_command_is_not_usage_error() {
        // `--help` is not a subcommand typo: no suggestion, clap exits 0.
        let err = cli::Cli::try_parse_from(["catherine", "--help"])
            .expect_err("help exits via the error path");
        assert!(did_you_mean(&err).is_none());
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
        let _ = cli::Cli::command();
    }
}
