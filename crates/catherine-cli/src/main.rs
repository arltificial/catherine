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

mod agree;
mod args;
mod cli;
mod composition;
mod config;
mod diff;
mod dispatch;
mod granularity;
mod migrate;
mod naturality;
mod pattern;
mod projection;
mod render;
mod rules;
mod run;
mod runners;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::Parser;
use dispatch::{dispatch_command, Ctx};
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
            "naturality".to_string(),
            "query".to_string(),
            "migrate".to_string(),
            "rules".to_string(),
            "pattern".to_string(),
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
    dispatch_command(command, ctx)
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
