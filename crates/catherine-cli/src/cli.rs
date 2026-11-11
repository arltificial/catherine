//! Purpose: the clap wiring — `check`/`findings` subcommands over a
//! positional model path, the genesis guide's verbosity/format structs,
//! and the exit-code table the help text carries verbatim.
//!
//! Responsibilities: define [`Cli`] (genesis `CliVerbosity` + `CliFormat`
//! flattened, so `-v`/`-q` and `--json`/`--human` behave exactly as in
//! every suite tool) and [`Commands`] (`check`, `findings`; positional
//! model path); publish [`EXIT_CODE_TABLE`] — the adjudicated exit-code
//! contract — in the root and both subcommands' help.
//!
//! Rationale: genesis's `CliFormat` auto-detects JSON when stdout is not
//! a TTY, but the cli spec mandates human-by-default ("human-readable by
//! default, a genesis-vibes JSON envelope under `--json`") — an
//! auto-flip under a CI pipe would break the DESIGN §5 walkthrough's
//! human transcript. The caller therefore keys off the `json` flag
//! directly; the structs themselves stay the genesis-standard surface.

use clap::{Parser, Subcommand};
use genesis::guide::{CliFormat, CliVerbosity};

/// The exit-code contract, verbatim from the change's adjudicated table
/// (`openspec/changes/add-check-cli`): numerals are stable once shipped.
pub const EXIT_CODE_TABLE: &str = "\
exit codes:
  0  clean — zero findings
  1  parse/build failure — labeled error (parse error or typing violation), never a finding
  2  usage error — clap's own code for bad args/unknown subcommand; never a finding exit
  3  violation (highest severity)
  4  dangling
  5  contradiction
  6  underconstraint
  7  redundant (lowest severity)
multi-class runs exit with the minimum present class code.";

#[derive(Debug, Parser)]
#[command(
    name = env!("CARGO_BIN_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    about = "Execute catherine model checks and print finding-typed diagnostics",
    after_help = EXIT_CODE_TABLE,
)]
pub struct Cli {
    /// Genesis-standard verbosity (-q / -v / -vv / -vvv).
    #[command(flatten)]
    pub verbose: CliVerbosity,

    /// Genesis-standard format flags (--json / --human).
    #[command(flatten)]
    pub format: CliFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Parse, build, and execute all checks; print every finding with
    /// its kind, epistemic label, and witness.
    #[command(after_help = EXIT_CODE_TABLE)]
    Check {
        /// Path to the .ct.json model document.
        model: String,
    },
    /// The findings-only view: same execution, same presentation, same
    /// exit contract — one execution path with `check`, never a re-run.
    #[command(after_help = EXIT_CODE_TABLE)]
    Findings {
        /// Path to the .ct.json model document.
        model: String,
    },
}
