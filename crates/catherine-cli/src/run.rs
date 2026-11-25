//! Purpose: the one execution path both subcommands share — load,
//! parse, build, execute all checks — plus the exit decision: the
//! minimum class code over the findings (the spec's fixed precedence
//! collapsed to a `min`).
//!
//! Responsibilities: [`exit_code`] is the pure, unit-testable exit
//! contract over `&[Finding]` (0 clean; 3..=7 = 2 + rank under
//! violation > dangling > contradiction > underconstraint > redundant);
//! [`run`] drives the kernel once per invocation and renders — human
//! findings to stdout, the `--json` findings envelope to stdout, every
//! error (human labeled error or the `--json` error envelope) to
//! stderr — and returns the process exit code.
//!
//! Rationale: exit codes are computed from the finding classes present,
//! never from rendered text (ticket anti_goal); parse/build failures
//! exit 1 — distinct from any finding; usage errors stay clap's own 2
//! (never handled here). `check` and `findings` are the same surface
//! under two names: same execution, same presentation, same exit — a
//! divergence between the surfaces would be a re-run-semantics bug, which the
//! proposal binds away ("same execution, different presentation" names
//! the entry point, not a second pipeline). The analysis runners live
//! in [`crate::runners`], the migrate runner in [`crate::migrate`], the
//! flag parsers in [`crate::args`].

use crate::render::{self, LabeledError};
use catherine_core::checks;
use catherine_core::findings::{Finding, FindingKind};
use catherine_core::instance::TypedInstance;
use catherine_core::model::Model;
use genesis::envelope::Warning;
use std::io::Write;

/// Which subcommand invoked the shared path. The execution is
/// identical; the enum exists so the caller's intent is explicit and
/// `findings` can never grow divergent re-run semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Check,
    Findings,
}

/// The exit decision: the minimum class code present — the spec's fixed
/// precedence (violation > dangling > contradiction > underconstraint >
/// redundant) collapses to a `min`; zero findings are clean (0).
pub fn exit_code(findings: &[Finding]) -> i32 {
    findings
        .iter()
        .map(|finding| class_code(&finding.kind))
        .min()
        .unwrap_or(0)
}

/// One class → one code: 3..=7 are `2 + rank` under the precedence
/// order. The numerals are the contract (brief §1's table).
pub fn class_code(kind: &FindingKind) -> i32 {
    match kind {
        FindingKind::Violation(_) => 3,
        FindingKind::Dangling(_) => 4,
        FindingKind::Contradiction(_) => 5,
        FindingKind::Underconstraint(_) => 6,
        FindingKind::Redundant(_) => 7,
        // Below every kernel class: the granularity lint questions the
        // model's shape (ticket `catherine-kw9.3`), never a row.
        FindingKind::Granularity(_) => 8,
    }
}

/// Execute `model_paths` once and render. One path is the unchanged
/// single-document run; two or more are glued by declared name and
/// checked jointly (design Decision 1). Returns the process exit code.
/// `quiet` suppresses the human finding lines (never the exit decision,
/// never errors, never the machine envelope).
#[allow(clippy::too_many_arguments)]
pub fn run(
    surface: Surface,
    model_paths: &[String],
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let _ = surface; // one path for both names — see the module docs.
    if model_paths.len() > 1 {
        return run_composition(model_paths, json, quiet, warnings, stdout, stderr);
    }
    let model_path = model_paths.first().map(String::as_str).unwrap_or_default();

    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };

    let typed = TypedInstance::build(&model.schema, &model.instance);
    let (mut findings, scope) = checks::check_with_scope(&model.schema, &typed);
    // The stored-granularity lint (ticket `catherine-kw9.3`) rides the
    // shared check path, sorted last by the taxonomy's kind order.
    findings.extend(crate::granularity::findings(&model));

    emit_findings(
        &findings,
        &model.schema,
        &scope,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
    );
    exit_code(&findings)
}

/// The render tail of [`run`]: the `--json` findings envelope or the §5
/// human lines to stdout, stderr reserved for errors.
fn emit_findings(
    findings: &[Finding],
    schema: &catherine_core::model::Schema,
    scope: &checks::ExecutionScope,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) {
    if json {
        let envelope = render::findings_envelope(findings, Some(scope), warnings);
        let line = serde_json::to_string(&envelope)
            .map_err(|error| error.to_string())
            .and_then(|text| writeln!(stdout, "{text}").map_err(|error| error.to_string()));
        if let Err(error) = line {
            let _ = writeln!(stderr, "catherine: cannot write findings envelope: {error}");
        }
    } else if !quiet {
        for line in render::render_findings(findings, schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
}

/// The joint (composition) check: load and glue every input, execute the
/// kernel checks over the merged model, warn on a disjoint union, and
/// render the findings with source attribution.
fn run_composition(
    model_paths: &[String],
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    crate::composition::warn_duplicate_inputs(model_paths, json, &mut warnings, stderr);
    let glued = match crate::composition::load_glued(model_paths) {
        Ok(glued) => glued,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    crate::composition::warn_empty_interface(&glued, json, &mut warnings, stderr);
    let typed = TypedInstance::build(&glued.schema, &glued.instance);
    let (findings, scope) = checks::check_with_scope(&glued.schema, &typed);
    crate::composition::emit_findings(
        &findings,
        Some(scope),
        &glued,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
    )
}

/// Emit a labeled error on the error stream — human two-liner, or the
/// JSON error envelope under `--json` — and return the error exit code.
/// Errors never write stdout, in either mode.
pub(crate) fn emit_error(
    labeled: &LabeledError,
    json: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let _ = stdout; // stream discipline: this path writes stderr only.
    if json {
        let envelope = render::error_envelope(labeled, warnings);
        match serde_json::to_string(&envelope) {
            Ok(text) => {
                let _ = writeln!(stderr, "{text}");
            }
            Err(error) => {
                let _ = writeln!(stderr, "catherine: cannot write error envelope: {error}");
                let _ = writeln!(stderr, "{}", labeled.human_head());
            }
        }
    } else {
        let _ = writeln!(stderr, "{}", labeled.human_head());
        let _ = writeln!(stderr, "  hint: {}", labeled.hint);
    }
    1
}

/// Load and parse a model document once — the single load both the
/// check path and every analysis runner share. Errors are labeled:
/// `read_error` for an unreadable path, the kernel's codes for a parse
/// failure.
pub(crate) fn load_model(model_path: &str) -> Result<Model, LabeledError> {
    load_model_with_rows(model_path).map(|(model, _)| model)
}

/// [`load_model`], additionally returning the parser's name side table
/// (presentation row names indexed by dense `RowId`) — the seam the
/// `rules --project` bridge needs to spell projected values. The model is
/// the same value either way: names never enter it.
pub(crate) fn load_model_with_rows(
    model_path: &str,
) -> Result<(Model, catherine_core::parse::RowNames), LabeledError> {
    let text = std::fs::read_to_string(model_path).map_err(|error| LabeledError {
        code: "read_error".to_string(),
        line: None,
        column: None,
        message: format!("cannot read model document `{model_path}`: {error}"),
        hint: "pass a readable .ct.json model document path",
    })?;
    catherine_core::parse::parse_with_rows(&text).map_err(|error| LabeledError::from_parse(&error))
}

/// Write a success envelope as one JSON line to stdout; a serialization
/// failure is reported on stderr (the stream discipline never breaks).
pub(crate) fn write_envelope<T: serde::Serialize>(
    envelope: &genesis::envelope::Envelope<T>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    what: &str,
) {
    match serde_json::to_string(envelope) {
        Ok(text) => {
            let _ = writeln!(stdout, "{text}");
        }
        Err(error) => {
            let _ = writeln!(stderr, "catherine: cannot write {what} envelope: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::error::{ErrorCode, ParseError, SemanticError};
    use catherine_core::findings::{
        ContradictionWitness, DanglingWitness, DomainQuestion, EpistemicLabel, EquationWitness,
        RedundantWitness, UnderconstraintWitness, ViolationWitness,
    };
    use catherine_core::model::{Ident, Path, RowId};

    fn violation() -> Finding {
        Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
                equation: 0,
                row: RowId(3),
                lhs: RowId(1),
                rhs: RowId(2),
            })),
        }
    }

    fn dangling() -> Finding {
        Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Dangling(DanglingWitness {
                arrow: Ident("in_section".to_string()),
                src: RowId(0),
            }),
        }
    }

    /// The three analysis classes have no producer until P4; the exit
    /// mapping is pinned over directly constructed findings (the
    /// taxonomy is constructible by design).
    fn contradiction() -> Finding {
        Finding {
            label: EpistemicLabel::Unknown,
            kind: FindingKind::Contradiction(ContradictionWitness {
                scope: vec![0, 1],
                keys: Vec::new(),
                coproducts: Vec::new(),
                pullbacks: Vec::new(),
            }),
        }
    }

    fn underconstraint() -> Finding {
        Finding {
            label: EpistemicLabel::Hypothesized,
            kind: FindingKind::Underconstraint(UnderconstraintWitness {
                case: DomainQuestion::ParallelPaths {
                    lhs: Path(vec![Ident("f".to_string())]),
                    rhs: Path(vec![Ident("g".to_string())]),
                },
                question: "have no declared equation?".to_string(),
            }),
        }
    }

    fn redundant() -> Finding {
        Finding {
            label: EpistemicLabel::Unknown,
            kind: FindingKind::Redundant(RedundantWitness {
                derivable: Path(vec![Ident("f".to_string())]),
                reducing: Path(vec![Ident("g".to_string())]),
            }),
        }
    }

    #[test]
    fn clean_is_exit_zero() {
        assert_eq!(exit_code(&[]), 0);
    }

    #[test]
    fn each_class_maps_to_its_table_code() {
        assert_eq!(exit_code(&[violation()]), 3);
        assert_eq!(exit_code(&[dangling()]), 4);
        assert_eq!(exit_code(&[contradiction()]), 5);
        assert_eq!(exit_code(&[underconstraint()]), 6);
        assert_eq!(exit_code(&[redundant()]), 7);
    }

    #[test]
    fn multi_class_runs_exit_with_the_minimum_class_code() {
        assert_eq!(exit_code(&[dangling(), violation()]), 3);
        assert_eq!(exit_code(&[contradiction(), underconstraint()]), 5);
        assert_eq!(
            exit_code(&[redundant(), underconstraint(), contradiction()]),
            5
        );
        assert_eq!(
            exit_code(&[
                redundant(),
                underconstraint(),
                contradiction(),
                dangling(),
                violation(),
            ]),
            3
        );
    }

    #[test]
    fn parse_errors_render_with_position_code_and_hint() {
        let err = LabeledError::from_parse(&ParseError::Semantic(SemanticError {
            line: 14,
            column: 16,
            code: ErrorCode::DuplicateIdent,
            message: "duplicate arrow ident `in_section`".to_string(),
        }));
        assert_eq!(err.code, "duplicate_ident");
        let head = err.human_head();
        assert!(
            head.contains("line 14") && head.contains("column 16"),
            "{head}"
        );
        assert!(!err.hint.is_empty());
        assert_eq!(
            err.positioned_message(),
            "duplicate arrow ident `in_section` (line 14, column 16)"
        );
    }
}
