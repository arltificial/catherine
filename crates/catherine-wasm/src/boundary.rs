//! Purpose: the boundary module itself — the document cap, the typed
//! error, the config surface, and the two JSON-in/JSON-out operations
//! (`check`, `analyze`). No logic lives here beyond decode-and-forward:
//! the kernel's parse/build/check walk and the analysis ops are called
//! exactly as the CLI calls them.
//!
//! Responsibilities: [`MAX_DOCUMENT_BYTES`] (the row-count/byte cap the
//! boundary enforces before the parser is invoked),
//! [`BoundaryError`] (the closed typed error with a stable `code`-tagged
//! JSON encoding), [`AnalysisConfig`] (the bounds/caps surface), and the
//! two reports (`check` → the kernel's findings verbatim under
//! `findings`; `analyze` → the combined bounded-analysis findings plus
//! the honest `unknown` scope when the search budget expires).
//!
//! Rationale: the cap is a length check on the input string, not a row
//! count computed by parsing — an oversized document never reaches the
//! kernel, so the cost of an oversize request is O(len), not the parse.
//! The error enum is closed and tagged like the kernel's own error
//! codes: the web UI matches on `code`, never on message text. The
//! findings encoding is the kernel's serde encoding, untouched — the
//! parity gate (task 7) pins terminal and browser to identical bytes.

use catherine_analysis::bounds::{ScopeCaps, SearchBounds};
use catherine_analysis::parallel;
use catherine_analysis::rewrite;
use catherine_analysis::search::{self, SearchOutcome};
use catherine_core::checks;
use catherine_core::error::ParseError;
use catherine_core::findings::Finding;
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, Model};
use catherine_core::parse;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The document cap enforced at the boundary: a document whose encoded
/// size exceeds this is rejected with [`BoundaryError::DocumentTooLarge`]
/// before the kernel is invoked.
pub const MAX_DOCUMENT_BYTES: usize = 512 * 1024;

/// The closed typed boundary error. The `code` tag is the stable
/// spelling the UI matches on; extending the enum is a ticket, never a
/// silent addition (mirrors the kernel's six-code discipline).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum BoundaryError {
    /// The document exceeded [`MAX_DOCUMENT_BYTES`] — the kernel was
    /// never invoked. `bytes` is the actual size, `max_bytes` the cap.
    DocumentTooLarge { bytes: usize, max_bytes: usize },
    /// The kernel's parser rejected the document: line/column (1-based)
    /// and the kernel's message; a semantic failure prefixes its stable
    /// six-code spelling.
    Parse {
        line: usize,
        column: usize,
        message: String,
    },
    /// The analysis config did not decode.
    AnalysisConfig { message: String },
}

impl BoundaryError {
    /// The error's stable JSON encoding — the exact string that crosses
    /// the wasm boundary's error side.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("boundary error encoding is infallible")
    }
}

/// The analysis bounds/caps surface. Every field optional with the
/// engine defaults (`ScopeCaps::default`); `max_rows` defaults to the
/// document's own row counts (the search then covers exactly the
/// declared instance's shape).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct AnalysisConfig {
    /// The search engine's candidate budget.
    pub max_candidates: Option<usize>,
    /// The parallel-path walk's maximum path length.
    pub max_path_length: Option<usize>,
    /// The rewriting kernel's maximum derivation steps.
    pub max_rewrite_steps: Option<usize>,
    /// Per-object row counts of the searched shape; objects omitted
    /// read as zero rows for the search.
    pub max_rows: Option<BTreeMap<Ident, usize>>,
}

/// The `check` report: the kernel's findings under `findings` — the same
/// encoding the CLI forwards in `data.findings`, no reshaping.
#[derive(Debug, Serialize)]
pub struct CheckReport<'a> {
    pub findings: &'a [Finding],
}

/// The scope a non-answer (`unknown`) explored — the honest first-class
/// non-result, never collapsed into clean or a finding.
#[derive(Debug, Serialize)]
pub struct UnknownScope {
    /// The effective nonzero per-object row counts explored.
    pub rows: BTreeMap<Ident, usize>,
    /// How many candidates the enumeration explored before stopping.
    pub candidates_explored: usize,
}

/// The `analyze` report: the combined bounded-analysis findings (check +
/// contradiction/redundancy/underconstraint, in the taxonomy's total
/// order), the search outcome as a machine-readable discriminator, and,
/// when the search budget expired before an answer, the exact scope
/// explored. v1 limit, stated here so it is never hidden: the witness
/// INSTANCE is not carried across the boundary (`"outcome": "witness"`
/// says one exists; the CLI's `search` op owns witness payloads).
#[derive(Debug, Serialize)]
pub struct AnalyzeReport {
    pub findings: Vec<Finding>,
    /// `"witness" | "contradiction" | "unknown"` — the search outcome,
    /// so a found witness is never indistinguishable from a vacuous run.
    pub outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown: Option<UnknownScope>,
}

/// Decode a document: the cap first (kernel not invoked on oversize),
/// then the kernel's parser. This is the only door any export uses.
fn decode(document: &str) -> Result<Model, BoundaryError> {
    let bytes = document.len();
    if bytes > MAX_DOCUMENT_BYTES {
        return Err(BoundaryError::DocumentTooLarge {
            bytes,
            max_bytes: MAX_DOCUMENT_BYTES,
        });
    }
    parse::parse(document).map_err(|error| match error {
        ParseError::Syntax {
            line,
            column,
            message,
        } => BoundaryError::Parse {
            line,
            column,
            message,
        },
        ParseError::Semantic(semantic) => BoundaryError::Parse {
            line: semantic.line,
            column: semantic.column,
            message: format!("{}: {}", semantic.code.as_str(), semantic.message),
        },
    })
}

/// The kernel's executed checks over a decoded model — the exact path
/// the CLI's `check`/`findings` subcommands drive.
pub fn check(document: &str) -> Result<String, BoundaryError> {
    let model = decode(document)?;
    let typed = TypedInstance::build(&model.schema, &model.instance);
    let findings = checks::check(&model.schema, &typed);
    serde_json::to_string(&CheckReport {
        findings: &findings,
    })
    .map_err(|_| BoundaryError::AnalysisConfig {
        message: "findings serialization failed".to_string(),
    })
}

/// The bounded analysis over a decoded model: executed checks, bounded
/// search (contradiction only when exhaustion completes), redundancy,
/// and underconstraint — every finding in the taxonomy's total order.
pub fn analyze(document: &str, config: &str) -> Result<String, BoundaryError> {
    let model = decode(document)?;
    let config: AnalysisConfig =
        serde_json::from_str(config).map_err(|error| BoundaryError::AnalysisConfig {
            message: error.to_string(),
        })?;
    let caps = analysis_caps(&config);
    let typed = TypedInstance::build(&model.schema, &model.instance);

    let mut findings = checks::check(&model.schema, &typed);
    findings.extend(parallel::underconstraint(&model.schema, &caps));
    findings.extend(rewrite::redundant(&model.schema, &caps));
    let (outcome, unknown) = fold_search(
        search::search(&model.schema, &search_bounds(&model, &config, &caps)),
        &mut findings,
    );

    findings.sort();
    serde_json::to_string(&AnalyzeReport {
        findings,
        outcome,
        unknown,
    })
    .map_err(|_| BoundaryError::AnalysisConfig {
        message: "analysis report serialization failed".to_string(),
    })
}

/// The effective analysis caps: explicit config values, else the engine
/// defaults (the single read of `ScopeCaps::default` this module needs).
fn analysis_caps(config: &AnalysisConfig) -> ScopeCaps {
    let defaults = ScopeCaps::default();
    ScopeCaps {
        max_candidates: config.max_candidates.unwrap_or(defaults.max_candidates),
        max_path_length: config.max_path_length.unwrap_or(defaults.max_path_length),
        max_rewrite_steps: config
            .max_rewrite_steps
            .unwrap_or(defaults.max_rewrite_steps),
    }
}

/// The effective search bounds: explicit config rows, else the
/// document's own row shape (the search then covers exactly the
/// declared instance).
fn search_bounds(model: &Model, config: &AnalysisConfig, caps: &ScopeCaps) -> SearchBounds {
    let mut max_rows = config.max_rows.clone().unwrap_or_default();
    if max_rows.is_empty() {
        for (object, rows) in &model.instance.rows {
            max_rows.insert(object.clone(), rows.len());
        }
    }
    SearchBounds {
        max_rows,
        max_candidates: caps.max_candidates,
    }
}

/// Fold the search report into the findings plus the outcome
/// discriminator and the honest unknown scope — the one place
/// `SearchOutcome` maps to report fields (a witness says "witness" and
/// carries no payload; the CLI's `search` op owns witness instances).
fn fold_search(
    report: search::SearchReport,
    findings: &mut Vec<Finding>,
) -> (&'static str, Option<UnknownScope>) {
    match report.outcome {
        SearchOutcome::Contradiction { finding } => {
            findings.push(finding);
            ("contradiction", None)
        }
        SearchOutcome::Unknown { scope } => (
            "unknown",
            Some(UnknownScope {
                rows: scope.rows,
                candidates_explored: scope.candidates_explored,
            }),
        ),
        SearchOutcome::Witness { .. } => ("witness", None),
    }
}
