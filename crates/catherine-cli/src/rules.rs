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
//! `bad_argument`; and the `--project` branch, which swaps the document's
//! presentation for the one [`crate::projection`] synthesizes from
//! instance rows (its refusals arrive pre-labeled from that module).
//!
//! Rationale: no second pipeline — the same loader, envelope
//! constructors, warning channel, and exit decision as every other
//! surface. The input domain X is declared data the document does not
//! carry (the fixtures' README says bounded search supplies it), so
//! `--values` is the explicit bound-in flag; the CLI never invents a
//! value list or guesses a type, and a missing column is a labeled error
//! rather than a vacuous clean run. A projection substitutes its own
//! presentation and exact-tuple domain but then takes this very path —
//! the same analyzer, envelope, and exit decision: no second pipeline.

use crate::args::{bad_arg, parse_domain_values};
use crate::cli::default_scope_caps;
use crate::dispatch::Ctx;
use crate::projection::{self, Projection};
use crate::render::{self, LabeledError};
use crate::run::{exit_code, load_model_with_rows, write_envelope};
use catherine_analysis::bounds::ScopeCaps;
use catherine_analysis::rule_lens::{rule_lens, RuleLensOutcome};
use catherine_core::model::{Model, RulePresentation};
use catherine_core::parse::RowNames;
use catherine_core::rule_lens::{RuleDomain, RuleDomainError};
use std::borrow::Cow;
use std::io::Write;

/// The rule-lens invocation's own flags; the model document and the
/// output plumbing are separate arguments.
pub(crate) struct LensOptions<'a> {
    /// The `--values` specs, unparsed.
    pub(crate) values: &'a [String],
    /// The `--max-candidates` budget.
    pub(crate) max_candidates: usize,
    /// The `--project` object, when the presentation is synthesized.
    pub(crate) project: Option<&'a str>,
    /// The `--columns` arrows of a projected presentation.
    pub(crate) columns: Option<&'a str>,
    /// The `--action` arrow of a projected presentation.
    pub(crate) action: Option<&'a str>,
}

/// Execute the rule lens over `model_path`'s `rule_presentation` — or,
/// with `project`, over a rule table synthesized from that object's
/// instance rows (see [`crate::projection`]) — and render. Returns the
/// process exit code. `quiet` suppresses the human finding lines (never
/// the exit decision, never errors, never the machine envelope).
pub fn run_rules(
    model_path: &str,
    lens: &LensOptions<'_>,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let (model, rows) = match load_model_with_rows(model_path) {
        Ok(loaded) => loaded,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    let (presentation, domain) = match bind_presentation(model_path, &model, &rows, lens) {
        Ok(bound) => bound,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    report_bound(presentation.as_ref(), &domain, lens.max_candidates, ctx)
}

/// The bounded outcome tail: the scan runs under the `--max-candidates`
/// cap with the defaults elsewhere.
fn report_bound(
    presentation: &RulePresentation,
    domain: &RuleDomain,
    max_candidates: usize,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let caps = ScopeCaps {
        max_candidates,
        ..default_scope_caps()
    };
    report_outcome(rule_lens(presentation, domain, &caps), presentation, ctx)
}

/// Choose the presentation and domain: the document's declared table
/// bound through `--values`, or the synthetic projection. Every refusal is
/// a labeled error (the caller emits it on the error stream).
fn bind_presentation<'a>(
    model_path: &str,
    model: &'a Model,
    rows: &RowNames,
    lens: &LensOptions<'_>,
) -> Result<(Cow<'a, RulePresentation>, RuleDomain), LabeledError> {
    if lens.project.is_none() && (lens.columns.is_some() || lens.action.is_some()) {
        return Err(bad_arg(
            "`--columns`/`--action` require `--project <Object>`".to_string(),
            "project an object's instance rows with `--project <Object>`, or analyze the document's `rule_presentation` with `--values`",
        ));
    }
    if let Some(object) = lens.project {
        let request = Projection {
            object,
            columns: lens.columns,
            action: lens.action,
        };
        let (presentation, domain) = projection::project(model, rows, &request, lens.values)?;
        return Ok((Cow::Owned(presentation), domain));
    }
    let Some(presentation) = model.rule_presentation.as_ref() else {
        return Err(no_rule_presentation(model_path));
    };
    let domain = bind_domain(presentation, lens.values)?;
    Ok((Cow::Borrowed(presentation), domain))
}

/// Render one analysis outcome: findings envelope/human lines, or the
/// bounded UNKNOWN payload.
fn report_outcome(
    outcome: RuleLensOutcome,
    presentation: &catherine_core::model::RulePresentation,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    match outcome {
        RuleLensOutcome::Findings(findings) => report_findings(&findings, presentation, ctx),
        RuleLensOutcome::Unknown { scope } => report_unknown(&scope, ctx),
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
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    ctx.print_warnings();
    if ctx.json {
        let envelope = render::rules_envelope(findings, None, ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "rules");
    } else if !ctx.quiet {
        for line in render::render_rules(findings, presentation) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    exit_code(findings)
}

/// Render a bounded non-answer: `outcome: "unknown"` + the exact scope,
/// or the one honest human line. A completed bounded run exits 0.
fn report_unknown(
    scope: &catherine_core::rule_lens::RuleScope,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    ctx.print_warnings();
    if ctx.json {
        let envelope = render::rules_envelope(&[], Some(scope), ctx.take_warnings());
        write_envelope(&envelope, ctx.stdout, ctx.stderr, "rules");
    } else if !ctx.quiet {
        let _ = writeln!(
            ctx.stdout,
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
/// declared columns with their declared types. Shared with the projection
/// bridge, whose `--values` product declaration binds the same way.
pub(crate) fn bad_values(error: &RuleDomainError) -> LabeledError {
    LabeledError {
        code: "bad_argument".to_string(),
        line: None,
        column: None,
        message: format!("cannot bind the rule presentation's input domain: {error}"),
        hint: "supply `--values column=v1,v2,...` for every declared column, with values of the column's type",
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
