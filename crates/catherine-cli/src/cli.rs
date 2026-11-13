//! Purpose: the clap wiring — the `check`/`findings` subcommands over a
//! positional model path plus P4's one-subcommand-per-analysis-op set
//! (`search`, `redundant`, `underconstraint`, `distinguish`, `query`),
//! P7's `migrate`, and P8's `rules` (the rule lens, plus the
//! `add-rules-projection` bridge's `--project`/`--columns`/`--action`
//! flags), the genesis guide's verbosity/format structs, and the
//! exit-code table the help text carries verbatim.
//!
//! Responsibilities: define [`Cli`] (genesis `CliVerbosity` + `CliFormat`
//! flattened, so `-v`/`-q` and `--json`/`--human` behave exactly as in
//! every suite tool) and [`Commands`] (`check`, `findings`, the five
//! analysis commands, `migrate`, and `rules`; positional model paths);
//! publish
//! [`EXIT_CODE_TABLE`] — the adjudicated exit-code contract, its shipped
//! numerals unchanged — in the root and every subcommand's help, plus
//! [`LOST_DATA_EXIT_CODE`] for the one `add-delta-migration` class. The
//! projection help states its outside-the-hash/ledger boundary and the
//! product-declaration caveat, because a projection run is evidence about
//! a projection, never a sealed model claim.
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
//! payload. `migrate` follows the same convention one class over: lost
//! data is a finding-bearing run (exit 8), a lossless migration is
//! clean (exit 0) — the loss report lives in the payload either way.

use clap::{Parser, Subcommand};
use genesis::guide::{CliFormat, CliVerbosity};

/// The lost-data exit numeral (openspec change `add-delta-migration`):
/// the one class `migrate` can emit, and the table's lowest severity.
/// Migration findings are the kernel's `MigrationFinding` taxonomy, not
/// the check taxonomy — the numeral is defined beside the table so the
/// table, the help text, and the runner cannot drift apart.
pub const LOST_DATA_EXIT_CODE: i32 = 8;

/// The exit-code contract: `add-check-cli`'s adjudicated table, whose
/// numerals are stable once shipped, extended once by
/// `add-delta-migration` with the lost-data class (8) — an additive
/// numeral, every shipped code unchanged.
pub const EXIT_CODE_TABLE: &str = "\
exit codes:
  0  clean — zero findings
  1  parse/build failure — labeled error (parse error or typing violation), never a finding
  2  usage error — clap's own code for bad args/unknown subcommand; never a finding exit
  3  violation (highest severity)
  4  dangling
  5  contradiction
  6  underconstraint
  7  redundant
  8  lost data (lowest severity; migrate only)
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
    /// Schema diff: classify a v1→v2 schema edit on the *enforced*
    /// constraints — cosmetic (presentation only: zero findings),
    /// accreted (constraints only added), weakened (a constraint is no
    /// longer enforced: one violation finding with a witness instance v2
    /// accepts and v1 rejects), equivalent-in-scope (no separating
    /// instance within the bounds), or unknown (the budget expired).
    /// Weakened exits through the finding-class table (3); every other
    /// completed classification exits 0 with the outcome in the payload.
    #[command(after_help = EXIT_CODE_TABLE)]
    Diff {
        /// Path to the first (old, v1) model document.
        model: String,
        /// Path to the second (new, v2) model document.
        second: String,
        /// Per-object row counts of the searched shape, `object=count`
        /// (repeatable; both schemas share the shape — an omitted object
        /// reads as 0 rows).
        #[arg(long = "rows", value_name = "OBJECT=COUNT")]
        rows: Vec<String>,
        /// Maximum candidates the scan may enumerate before the
        /// outcome is UNKNOWN with the explored scope.
        #[arg(long = "max-candidates", default_value_t = default_scope_caps().max_candidates)]
        max_candidates: usize,
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
    /// Δ migration: reindex the document's instance along its declared
    /// (total) schema map — the migrated instance summary, the row
    /// correspondence, and the loud lost-data enumeration (one finding
    /// per dropped row or valued dropped cell, labeled `executed`). A
    /// document without a `schema_map` is a labeled error, never an
    /// empty migration; lost data is a finding-bearing run (exit 8).
    #[command(after_help = EXIT_CODE_TABLE)]
    Migrate {
        /// Path to the .ct.json model document carrying `schema_map`.
        model: String,
    },
    /// Rule lens: analyze the document's `rule_presentation` over the
    /// declared finite input domain — gap → underconstraint (6), overlap
    /// under `DeclaredDisjoint` → violation (3), shadow/derivable →
    /// redundant (7) — with each finding's Decision-1 taxonomy kind. A
    /// budget that expires is `outcome: "unknown"` with the exact scope
    /// (exit 0: a completed bounded run, never "clean"); a document
    /// without a `rule_presentation` is a labeled error.
    ///
    /// With `--project`, one rule per instance row of the named object is
    /// synthesized instead (predicate = the conjunction of the projected
    /// columns equal to their target row names, action = the row's own
    /// name or `--action`'s target, policy `DeclaredDisjoint`), and the
    /// input domain is the observed column tuples, optionally unioned
    /// with a `--values` product declaration. The projected claim is
    /// outside the model hash and the ledger — it is not sealed evidence:
    /// it reads *on this snapshot of rows, projected this way*. A product
    /// `--values` declaration can name combinations no row has: such an
    /// input is a genuine gap, and the gap reports the declared scope,
    /// not the data.
    #[command(after_help = EXIT_CODE_TABLE)]
    Rules {
        /// Path to the .ct.json model document carrying
        /// `rule_presentation`, or — with `--project` — the document
        /// whose instance rows are projected.
        model: String,
        /// Finite values per declared column, `column=v1,v2,...`
        /// (repeatable; every declared column needs one — the domain X
        /// is declared data, never inferred from instance rows). For a
        /// projection the declared product is unioned with the observed
        /// column tuples; an input no row covers is reported as a gap.
        #[arg(long = "values", value_name = "COLUMN=V1,V2,...")]
        values: Vec<String>,
        /// Maximum candidates the enumeration may materialize before
        /// the outcome is UNKNOWN with the explored scope.
        #[arg(long = "max-candidates", default_value_t = default_scope_caps().max_candidates)]
        max_candidates: usize,
        /// Project the named object's instance rows onto a rule table
        /// instead of reading the document's `rule_presentation` (the two
        /// are refused together). The claim is not part of the model hash
        /// or the ledger.
        #[arg(long = "project", value_name = "OBJECT")]
        project: Option<String>,
        /// Projected columns: arrows sourced at the `--project` object,
        /// comma-separated (each becomes a string-equality predicate on
        /// the arrow's target row name).
        #[arg(long = "columns", value_name = "ARROW,...")]
        columns: Option<String>,
        /// The projected action arrow (default: each row's own name).
        #[arg(long = "action", value_name = "ARROW")]
        action: Option<String>,
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

    /// The exit-code contract is pinned verbatim: shipped numerals are
    /// stable, and the one extension (lost data, 8) is additive — the
    /// analysis commands route findings-bearing results through the same
    /// numerals (5/6/7) and never drift the table.
    #[test]
    fn exit_code_table_is_pinned_verbatim() {
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
  7  redundant
  8  lost data (lowest severity; migrate only)
multi-class runs exit with the minimum present class code."
        );
    }

    /// The lost-data numeral is the table's 8: the constant and the
    /// table are one contract, asserted together so neither can drift.
    #[test]
    fn lost_data_exit_matches_the_table_numeral() {
        assert_eq!(LOST_DATA_EXIT_CODE, 8);
        assert!(EXIT_CODE_TABLE.contains("8  lost data"));
    }
}
