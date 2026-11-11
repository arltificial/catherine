//! Purpose: the optional `catherine.toml` — genesis-standard knobs only
//! (format, verbosity), loaded best-effort.
//!
//! Responsibilities: define the config schema and its path; load it
//! best-effort (missing file = defaults; malformed file or invalid value
//! = a warning, never a failure); resolve the effective output format
//! (explicit flag > config > human default) and verbosity (explicit flag
//! > config > normal).
//!
//! Rationale: the config file carries only the genesis-standard knobs —
//! no catherine-specific surface. Config problems degrade to warnings
//! (carried in the JSON envelope's `warnings`, printed to stderr in
//! human mode) instead of masquerading as model failures, so exit 1
//! stays "parse/build failure of the model" and nothing else.

use crate::cli;
use genesis::config::ConfigFile;
use genesis::guide::{OutputFormat, Verbosity};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The optional `catherine.toml` — genesis-standard knobs only. Every
/// field is optional; absent file means defaults.
#[derive(Debug, Default, Deserialize)]
pub struct CatherineConfig {
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
pub fn load_config(repo_root: &Path) -> (CatherineConfig, Vec<String>) {
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
pub fn resolve_format(cli: &cli::Cli, config: &CatherineConfig) -> OutputFormat {
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
pub fn resolve_verbosity(cli: &cli::Cli, config: &CatherineConfig) -> Verbosity {
    if cli.verbose.raw_count() == 0 && !cli.verbose.quiet {
        return match config.verbosity.as_deref() {
            Some("quiet") => Verbosity::Quiet,
            Some("verbose") => Verbosity::Verbose,
            _ => Verbosity::Normal,
        };
    }
    cli.verbose.verbosity()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli;
    use clap::Parser;

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
}
