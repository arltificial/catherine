//! Purpose: the clap wiring — the `check`/`findings` subcommands over a
//! positional model path plus P4's one-subcommand-per-analysis-op set
//! (`search`, `redundant`, `underconstraint`, `distinguish`, `query`),
//! the genesis guide's verbosity/format structs, and the exit-code table
//! the help text carries verbatim.
//!
//! Responsibilities: define [`Cli`] (genesis `CliVerbosity` + `CliFormat`
//! flattened, so `-v`/`-q` and `--json`/`--human` behave exactly as in
//! every suite tool) and [`Commands`] (`check`, `findings`, and the five
//! analysis commands; positional model paths); publish
//! [`EXIT_CODE_TABLE`] — the adjudicated exit-code contract, verbatim
//! unchanged — in the root and every subcommand's help.
//!
//! Rationale: genesis's `CliFormat` auto-detects JSON when stdout is not
//! a TTY, but the cli spec mandates human-by-default ("human-readable by
//! default, a genesis-vibes JSON envelope under `--json`") — an
//! auto-flip under a CI pipe would break the DESIGN §5 walkthrough's
//! human transcript. The caller therefore keys off the `json` flag
//! directly; the structs themselves stay the genesis-standard surface.
//! The analysis commands carry the same exit table: findings-bearing ops
//! (search-contradiction, redundant, underconstraint) exit through the
//! existing min-over-class-codes decision (5/6/7 — numerals stable once
//! shipped), while a search/distinguish run completing within its bounds
//! is a successful tool execution (exit 0) with the outcome in the
//! payload.

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
    /// Bounded instance search: explicit bounds in, an honest outcome
    /// out — a witness instance, a contradiction finding (exhaustion
    /// completed within bounds), or UNKNOWN with the exact scope
    /// explored. Search/distinguish runs that complete within their
    /// bounds are successful tool executions (exit 0) with the outcome
    /// in the payload; an UNKNOWN payload names its epistemic state
    /// (`outcome: "unknown"` + the scope), which is not "clean".
    #[command(after_help = EXIT_CODE_TABLE)]
    Search {
        /// Path to the .ct.json model document.
        model: String,
        /// Per-object row counts of the searched shape, `object=count`
        /// (repeatable; an omitted object reads as 0 rows).
        #[arg(long = "rows", value_name = "OBJECT=COUNT")]
        rows: Vec<String>,
        /// Maximum candidates the engine may enumerate before the
        /// outcome is UNKNOWN with the explored scope.
        #[arg(long = "max-candidates", default_value_t = default_scope_caps().max_candidates)]
        max_candidates: usize,
    },
    /// Prove redundancy: every accepted equation whose two sides
    /// rewrite to the same normal form under the other accepted
    /// equations. Absent a proof, no redundancy claim is emitted.
    #[command(after_help = EXIT_CODE_TABLE)]
    Redundant {
        /// Path to the .ct.json model document.
        model: String,
    },
    /// Raise the parallel-path domain question: paths with identical
    /// endpoints and no declared equation relating them, hypothesized —
    /// never a violation.
    #[command(after_help = EXIT_CODE_TABLE)]
    Underconstraint {
        /// Path to the .ct.json model document.
        model: String,
        /// Maximum path length to enumerate (a cyclic schema generates
        /// unboundedly many paths; the cap keeps the op total).
        #[arg(long = "max-path-length", default_value_t = default_scope_caps().max_path_length)]
        max_path_length: usize,
    },
    /// Bounded behavioral distinguishability of two instances over an
    /// explicit observation set — `distinguishable` naming the differing
    /// observation and both values, or `equivalent-in-scope`. The
    /// observation set is part of the result; no schema-equivalence
    /// claim is ever made. Completes within its bounds → exit 0.
    #[command(after_help = EXIT_CODE_TABLE)]
    Distinguish {
        /// Path to the first model document (its schema names the
        /// observation arrows).
        model: String,
        /// Path to the second model document.
        second: String,
        /// Observations, `path@object#row` with the path's arrows joined
        /// by `∘` and the row as its dense id — the transcripts'
        /// `Object#n` spelling (repeatable; at least one — the explicit
        /// observation set is the scope of the verdict).
        #[arg(long = "observe", value_name = "PATH@OBJECT#ROW", required = true)]
        observe: Vec<String>,
    },
    /// Reachability over the typed instance: the closure from seeds
    /// along the named morphisms, forward or backward (the ported
    /// specodelic query primitive). Dangling values are never followed.
    #[command(after_help = EXIT_CODE_TABLE)]
    Query {
        /// Path to the .ct.json model document.
        model: String,
        /// Seeds, `object#row` with the row as its dense id (repeatable;
        /// at least one — unknown seeds are rejected, not ignored).
        #[arg(long = "seed", value_name = "OBJECT#ROW", required = true)]
        seed: Vec<String>,
        /// Morphisms to follow, by arrow name (repeatable; at least one).
        #[arg(long = "follow", value_name = "ARROW", required = true)]
        follow: Vec<String>,
        /// Which way the closure walks: forward (what seeds reach) or
        /// backward (what reaches them).
        #[arg(long = "direction", default_value = "forward")]
        direction: String,
    },
}

/// The analysis scope-cap defaults, sourced from the analysis crate's
/// one configured place (task 12).
pub fn default_scope_caps() -> catherine_analysis::bounds::ScopeCaps {
    catherine_analysis::bounds::ScopeCaps::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exit-code contract is frozen: the analysis commands route
    /// findings-bearing results through the same numerals (5/6/7) and
    /// never drift the table.
    #[test]
    fn exit_code_table_is_verbatim_unchanged() {
        assert_eq!(
            EXIT_CODE_TABLE,
            "\
exit codes:
  0  clean — zero findings
  1  parse/build failure — labeled error (parse error or typing violation), never a finding
  2  usage error — clap's own code for bad args/unknown subcommand; never a finding exit
  3  violation (highest severity)
  4  dangling
  5  contradiction
  6  underconstraint
  7  redundant (lowest severity)
multi-class runs exit with the minimum present class code."
        );
    }
}
