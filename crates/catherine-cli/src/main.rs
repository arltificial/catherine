//! Purpose: the `catherine` entry point — CLI parsing with genesis
//! suggestions for unknown subcommands, optional `catherine.toml`
//! config (genesis config module), and the stream discipline: findings
//! to stdout, every error to stderr, exit codes from the finding
//! classes.
//!
//! Responsibilities: parse [`cli::Cli`]; on usage errors keep clap's
//! own output and exit code 2, augmented with a genesis did-you-mean
//! line for unknown subcommands; load the config best-effort (missing
//! file = defaults; malformed file or invalid value = a warning, never
//! a failure); resolve the effective format (explicit flag > config >
//! human default) and verbosity; dispatch through the one execution
//! path in [`run`].
//!
//! Rationale: the config file carries only the genesis-standard knobs
//! (format, verbosity) — no catherine-specific surface. Config problems
//! degrade to warnings (carried in the JSON envelope's `warnings`,
//! printed to stderr in human mode) instead of masquerading as model
//! failures, so exit 1 stays "parse/build failure of the model" and
//! nothing else.

mod cli;
mod render;
mod run;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::Parser;
use genesis::config::ConfigFile;
use genesis::guide::{OutputFormat, Verbosity};
use genesis::suggestions::{CommandRegistry, SuggestionEngine};
use serde::Deserialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

/// The optional `catherine.toml` — genesis-standard knobs only. Every
/// field is optional; absent file means defaults.
#[derive(Debug, Default, Deserialize)]
struct CatherineConfig {
    /// Default output format: "human" (default) or "json".
    format: Option<String>,
    /// Default verbosity: "quiet", "normal" (default), or "verbose".
    verbosity: Option<String>,
}

impl ConfigFile for CatherineConfig {
    fn path(repo_root: &Path) -> PathBuf {
        repo_root.join("catherine.toml")
    }
}

/// Load the config best-effort: defaults when the file is absent; a
/// warning (collected for the caller) when it exists but cannot serve.
fn load_config(repo_root: &Path) -> (CatherineConfig, Vec<String>) {
    match CatherineConfig::read(repo_root) {
        Ok(config) => (config, Vec::new()),
        Err(genesis::config::ConfigError::MissingFile { .. }) => {
            (CatherineConfig::default(), Vec::new())
        }
        Err(error) => (
            CatherineConfig::default(),
            vec![format!(
                "ignoring catherine.toml: {} — {}",
                error,
                error.to_suggestion().message()
            )],
        ),
    }
}

/// Resolve the output format: explicit flag > config > human default.
/// `--human` and the default both mean human — the spec pins
/// human-by-default regardless of whether stdout is a TTY.
fn resolve_format(cli: &cli::Cli, config: &CatherineConfig) -> OutputFormat {
    if cli.format.json {
        OutputFormat::Json
    } else if cli.format.human {
        OutputFormat::Human
    } else if config.format.as_deref() == Some("json") {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    }
}

/// Resolve the verbosity: explicit flag > config > normal.
fn resolve_verbosity(cli: &cli::Cli, config: &CatherineConfig) -> Verbosity {
    if cli.verbose.raw_count() == 0 && !cli.verbose.quiet {
        return match config.verbosity.as_deref() {
            Some("quiet") => Verbosity::Quiet,
            Some("verbose") => Verbosity::Verbose,
            _ => Verbosity::Normal,
        };
    }
    cli.verbose.verbosity()
}

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
        vec!["check".to_string(), "findings".to_string()],
    );
    SuggestionEngine::new()
        .suggest_typo(&name, &registry)
        .map(|suggestion| suggestion.message())
}

fn main() {
    let parsed = cli::Cli::try_parse();
    let cli = match parsed {
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

    let (config, mut config_warnings) = load_config(Path::new("."));
    let format = resolve_format(&cli, &config);
    let verbosity = resolve_verbosity(&cli, &config);
    let json = format == OutputFormat::Json;
    let quiet = verbosity == Verbosity::Quiet;

    // Config problems ride the warning channel: stderr in human mode,
    // the envelope's warnings in JSON mode.
    let envelope_warnings: Vec<genesis::envelope::Warning> = config_warnings
        .drain(..)
        .map(|message| render::config_warning(&message))
        .collect();
    if !json {
        let mut stderr = std::io::stderr();
        for warning in &envelope_warnings {
            let _ = writeln!(stderr, "warning: {}", warning.message);
        }
    }

    let (model, surface) = match cli.command {
        cli::Commands::Check { model } => (model, run::Surface::Check),
        cli::Commands::Findings { model } => (model, run::Surface::Findings),
    };

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let code = run::run(
        surface,
        &model,
        json,
        quiet,
        envelope_warnings,
        &mut stdout,
        &mut stderr,
    );
    exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn cli_args(args: &[&str]) -> cli::Cli {
        cli::Cli::try_parse_from(args).expect("args parse")
    }

    #[test]
    fn format_flag_wins_over_config() {
        let cli = cli_args(&["catherine", "check", "m.ct.json", "--json"]);
        let config = CatherineConfig {
            format: Some("human".to_string()),
            verbosity: None,
        };
        assert_eq!(resolve_format(&cli, &config), OutputFormat::Json);
    }

    #[test]
    fn human_is_the_default_regardless_of_tty() {
        let cli = cli_args(&["catherine", "check", "m.ct.json"]);
        let config = CatherineConfig::default();
        assert_eq!(resolve_format(&cli, &config), OutputFormat::Human);
    }

    #[test]
    fn config_can_default_to_json_without_a_flag() {
        let cli = cli_args(&["catherine", "check", "m.ct.json"]);
        let config = CatherineConfig {
            format: Some("json".to_string()),
            verbosity: None,
        };
        assert_eq!(resolve_format(&cli, &config), OutputFormat::Json);
    }

    #[test]
    fn verbosity_flag_wins_over_config() {
        let cli = cli_args(&["catherine", "check", "m.ct.json", "-q"]);
        let config = CatherineConfig {
            format: None,
            verbosity: Some("verbose".to_string()),
        };
        assert_eq!(resolve_verbosity(&cli, &config), Verbosity::Quiet);
    }

    #[test]
    fn config_defaults_verbosity_when_flags_are_absent() {
        let cli = cli_args(&["catherine", "check", "m.ct.json"]);
        let config = CatherineConfig {
            format: None,
            verbosity: Some("quiet".to_string()),
        };
        assert_eq!(resolve_verbosity(&cli, &config), Verbosity::Quiet);
    }

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
