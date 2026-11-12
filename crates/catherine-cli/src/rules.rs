//! Purpose: the `rules` runner (openspec change `add-rule-lens` phase C,
//! task 8) — one execution path for the rule-lens surface: load, bind
//! the declared finite domain, analyze under the candidate budget,
//! render, and decide the exit.
//!
//! Responsibilities: the same stream discipline as [`crate::run::run`]
//! and the analysis runners (the envelope/payload to stdout, every error
//! to stderr); the rule-lens exit decision — findings through the
//! existing min-over-class table (gap → 6, overlap under
//! `DeclaredDisjoint` → 3, redundant → 7; clean → 0), a bounded
//! non-answer is a completed tool execution (exit 0, `outcome: "unknown"`
//! and the exact scope in the payload — never collapsed into clean); the
//! labeled `no_rule_presentation` error for a document without the
//! additive section; `--values` parsing/validation errors are labeled
//! `bad_argument`.
//!
//! Rationale: no second pipeline — the same loader, envelope
//! constructors, warning channel, and exit decision as every other
//! surface. The input domain X is declared data the document does not
//! carry (the fixtures' README says bounded search supplies it), so
//! `--values` is the explicit bound-in flag; the CLI never invents a
//! value list or guesses a type, and a missing column is a labeled error
//! rather than a vacuous clean run.

use crate::args::parse_domain_values;
use crate::cli::default_scope_caps;
use crate::render::{self, LabeledError};
use crate::run::{emit_error, exit_code, load_model, write_envelope};
use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::rule_lens::{rule_lens, RuleLensOutcome};
use catherine_core::rule_lens::{RuleDomain, RuleDomainError};
use genesis::envelope::Warning;
use std::io::Write;

/// Execute the rule lens over `model_path`'s `rule_presentation` and
/// render. Returns the process exit code. `quiet` suppresses the human
/// finding lines (never the exit decision, never errors, never the
/// machine envelope).
#[allow(clippy::too_many_arguments)]
pub fn run_rules(
    model_path: &str,
    values: &[String],
    max_candidates: usize,
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let model = match load_model(model_path) {
        Ok(model) => model,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    let Some(presentation) = model.rule_presentation.as_ref() else {
        return no_presentation(model_path, json, warnings, stdout, stderr);
    };
    let Some(domain) = bind_or_emit(presentation, values, json, &mut warnings, stdout, stderr)
    else {
        return 1;
    };
    let caps = ScopeCaps {
        max_candidates,
        ..default_scope_caps()
    };
    report_outcome(
        rule_lens(presentation, &domain, &caps),
        presentation,
        json,
        quiet,
        warnings,
        stdout,
        stderr,
    )
}

/// Emit the labeled `no_rule_presentation` error (a document without
/// the additive 1.2 section is never a silent empty analysis).
fn no_presentation(
    model_path: &str,
    json: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    emit_error(
        &no_rule_presentation(model_path),
        json,
        warnings,
        stdout,
        stderr,
    )
}

/// Render one analysis outcome: findings envelope/human lines, or the
/// bounded UNKNOWN payload.
fn report_outcome(
    outcome: RuleLensOutcome,
    presentation: &catherine_core::model::RulePresentation,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    match outcome {
        RuleLensOutcome::Findings(findings) => report_findings(
            &findings,
            presentation,
            json,
            quiet,
            warnings,
            stdout,
            stderr,
        ),
        RuleLensOutcome::Unknown { scope } => {
            report_unknown(&scope, json, quiet, warnings, stdout, stderr)
        }
    }
}

/// Bind the `--values` domain, emitting the labeled error on failure.
fn bind_or_emit(
    presentation: &catherine_core::model::RulePresentation,
    values: &[String],
    json: bool,
    warnings: &mut Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Option<RuleDomain> {
    match bind_domain(presentation, values) {
        Ok(domain) => Some(domain),
        Err(labeled) => {
            emit_error(&labeled, json, std::mem::take(warnings), stdout, stderr);
            None
        }
    }
}

/// Bind the `--values` finite domain to the presentation, translating
/// either failure (flag syntax or a missing/ill-typed column) into a
/// labeled CLI input error.
fn bind_domain(
    presentation: &catherine_core::model::RulePresentation,
    values: &[String],
) -> Result<RuleDomain, LabeledError> {
    let supplied = parse_domain_values(values, &presentation.columns)?;
    RuleDomain::from_presentation(presentation, supplied).map_err(|error| bad_values(&error))
}

/// Render a completed analysis: the envelope or the human lines, then
/// the min-over-class exit decision.
fn report_findings(
    findings: &[catherine_core::findings::Finding],
    presentation: &catherine_core::model::RulePresentation,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    print_warnings(json, &warnings, stderr);
    if json {
        let envelope = render::rules_envelope(findings, None, warnings);
        write_envelope(&envelope, stdout, stderr, "rules");
    } else if !quiet {
        for line in render::render_rules(findings, presentation) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    exit_code(findings)
}

/// Render a bounded non-answer: `outcome: "unknown"` + the exact scope,
/// or the one honest human line. A completed bounded run exits 0.
fn report_unknown(
    scope: &catherine_core::rule_lens::RuleScope,
    json: bool,
    quiet: bool,
    warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    print_warnings(json, &warnings, stderr);
    if json {
        let envelope = render::rules_envelope(&[], Some(scope), warnings);
        write_envelope(&envelope, stdout, stderr, "rules");
    } else if !quiet {
        let _ = writeln!(
            stdout,
            "rules: unknown — explored {} candidate(s) within bounds",
            scope.candidates_explored
        );
    }
    0
}

/// A document without a `rule_presentation` is not a clean rule table —
/// labeled, never an empty success (the fail-open-seam rule).
fn no_rule_presentation(model_path: &str) -> LabeledError {
    LabeledError {
        code: "no_rule_presentation".to_string(),
        line: None,
        column: None,
        message: format!(
            "`{model_path}` carries no `rule_presentation` — there is no rule table to analyze"
        ),
        hint: "add a `rule_presentation` section (format_version 1.2), or use `catherine check` for plain model checking",
    }
}

/// The domain binder's rejection as a labeled CLI input error: the
/// document parsed, but the supplied finite values do not name its
/// declared columns with their declared types.
fn bad_values(error: &RuleDomainError) -> LabeledError {
    LabeledError {
        code: "bad_argument".to_string(),
        line: None,
        column: None,
        message: format!("cannot bind the rule presentation's input domain: {error}"),
        hint: "supply `--values column=v1,v2,...` for every declared column, with values of the column's type",
    }
}

/// Run-level warnings reach stderr in human mode too — the same
/// `warning: <message>` line the other runners print; JSON mode keeps
/// them on the envelope only.
fn print_warnings(json: bool, warnings: &[Warning], stderr: &mut impl Write) {
    if json {
        return;
    }
    for warning in warnings {
        let _ = writeln!(stderr, "warning: {}", warning.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_presentation_names_the_section() {
        let error = no_rule_presentation("model.ct.json");
        assert_eq!(error.code, "no_rule_presentation");
        assert!(error.message.contains("rule_presentation"));
        assert!(!error.hint.is_empty());
    }

    #[test]
    fn an_unbound_domain_is_a_bad_argument() {
        let error = bad_values(&RuleDomainError::MissingValues(
            catherine_core::model::Ident("age".to_string()),
        ));
        assert_eq!(error.code, "bad_argument");
        assert!(error.message.contains("age"));
        assert!(!error.hint.is_empty());
    }
}
