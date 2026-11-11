//! Purpose: presentation — the human renderer (the DESIGN §5 finding
//! format and the labeled-error format), the deduplicated genesis-vibes
//! envelope construction, and the remediation-hint table for P0's
//! stable codes.
//!
//! Responsibilities: render each finding as its labeled line(s) — kind
//! label first, witness facts on the two-space-indented continuation
//! line, paths joined with `∘`, disagreement as `≠`, row ids as
//! `Object#n` — the §5 transcript's exact format for the violation
//! block; render parse/build failures as labeled errors carrying
//! line/column, the stable code, and a remediation hint (never a stack
//! trace); build the `--json` envelopes one way for both subcommands:
//! findings under `data.findings` exactly as `catherine-core`
//! serializes them (no second serialization, no reshaping), errors as
//! the envelope's `ErrorResult` form with the kernel's message and
//! position unmodified under it.
//!
//! Rationale: the epistemic label rides on every finding in the JSON
//! payload; in human text the kernel-computed default (`executed`)
//! stays off the line so the §5 transcript is byte-reproducible, while
//! `hypothesized`/`unknown` findings append their label. The three
//! analysis kinds have no producer until P4; their renderings are
//! pinned by unit tests over constructed findings and their phrasing
//! stays minimal — P4's producers own the richer domain wording.

use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::findings::{
    DanglingWitness, DomainQuestion, EpistemicLabel, EquationWitness, Finding, FindingKind,
};
use catherine_core::model::{Path, RowId, Schema};
use genesis::envelope::{Envelope, EnvelopeKind, ErrorResult, RemediationEntry, Warning};
use serde::Serialize;

/// The CLI version carried in every envelope's `cli_version`.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Labeled errors ───────────────────────────────────────────────────

/// A labeled CLI-level error: the stable code, the position when the
/// kernel or the filesystem pins one, the kernel's message verbatim,
/// and a remediation hint. Never a stack trace.
pub struct LabeledError {
    /// The stable code: one of P0's six codes spelled as the fixtures
    /// pin them, or `syntax` for JSON-structural malformations, or
    /// `read_error` for an unreadable input path.
    pub code: String,
    /// 1-based line of the offending node, when known.
    pub line: Option<usize>,
    /// 1-based character column of the offending node, when known.
    pub column: Option<usize>,
    /// The kernel's message, verbatim (or the filesystem's).
    pub message: String,
    /// The remediation hint from the closed per-code table.
    pub hint: &'static str,
}

impl LabeledError {
    /// The kernel's labeled error data under the CLI's rendering: the
    /// stable code string, the position, the message verbatim, and the
    /// hint from the closed table. The CLI renders existing codes; it
    /// never extends them (P0's six-code enum is closed).
    pub fn from_parse(err: &ParseError) -> Self {
        match err {
            ParseError::Syntax {
                line,
                column,
                message,
            } => Self {
                code: "syntax".to_string(),
                line: Some(*line),
                column: Some(*column),
                message: message.clone(),
                hint: "fix the JSON at the reported position — the document must be a .ct.json v1 model",
            },
            ParseError::Semantic(sem) => Self {
                code: sem.code.as_str().to_string(),
                line: Some(sem.line),
                column: Some(sem.column),
                message: sem.message.clone(),
                hint: hint_for_code(sem.code),
            },
        }
    }

    /// The message with its position suffix — the JSON error envelope's
    /// message field and the human head's core content.
    pub fn positioned_message(&self) -> String {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                format!("{} (line {}, column {})", self.message, line, column)
            }
            _ => self.message.clone(),
        }
    }

    /// The human error's first line: `error[<code>] at line L, column C:
    /// <message>` (or without the position when none is known).
    pub fn human_head(&self) -> String {
        match (self.line, self.column) {
            (Some(line), Some(column)) => format!(
                "error[{}] at line {}, column {}: {}",
                self.code, line, column, self.message
            ),
            _ => format!("error[{}]: {}", self.code, self.message),
        }
    }
}

/// The remediation hints, keyed by the closed six-code enum.
fn hint_for_code(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::DuplicateIdent => {
            "idents must be unique within their collection — rename one of the duplicates"
        }
        ErrorCode::UnknownArrow => "declare the arrow in schema.arrows, or fix the path spelling",
        ErrorCode::IllTypedEquation => {
            "both paths must start at the same object and end at the same object"
        }
        ErrorCode::InvalidIdent => "idents match [A-Za-z_][A-Za-z0-9_]*",
        ErrorCode::InvalidStatus => "status is one of: accepted, proposed",
        ErrorCode::UnsupportedVersion => {
            "set format_version to \"1.0\" — additive minors of major 1 are accepted"
        }
    }
}

// ── Human finding rendering (the §5 format) ──────────────────────────

/// Render the findings as the §5 human lines: one head line per finding
/// (kind label + witness head), witness rows as two-space-indented
/// continuation lines.
pub fn render_findings(findings: &[Finding], schema: &Schema) -> Vec<String> {
    findings
        .iter()
        .flat_map(|f| render_one(f, schema))
        .collect()
}

fn render_one(finding: &Finding, schema: &Schema) -> Vec<String> {
    let label = label_suffix(finding.label);
    match &finding.kind {
        FindingKind::Violation(w) => render_violation(w, schema, &label),
        FindingKind::Dangling(w) => render_dangling(w, schema, &label),
        FindingKind::Redundant(w) => vec![
            format!("redundant: {}{}", path_string(&w.derivable), label),
            format!("  derivable as {}", path_string(&w.reducing)),
        ],
        FindingKind::Contradiction(w) => {
            let scope = w
                .scope
                .iter()
                .map(|index| format!("eq-{index}"))
                .collect::<Vec<_>>()
                .join(", ");
            vec![format!("contradiction: {scope} admit no instance{label}")]
        }
        FindingKind::Underconstraint(w) => {
            let head = match &w.case {
                DomainQuestion::ParallelPaths { lhs, rhs } => format!(
                    "underconstraint: parallel paths {} / {}{}",
                    path_string(lhs),
                    path_string(rhs),
                    label
                ),
                DomainQuestion::UncoveredInputs { class, examples } => {
                    let rows = examples
                        .iter()
                        .map(|row| row.0.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("underconstraint: uncovered inputs {class} (rows {rows}){label}")
                }
            };
            vec![head, format!("  {}", w.question)]
        }
    }
}

/// The §5 violation block: `violation: eq-<canonical index> <lhs path>
/// ≠ <rhs path>` then the two-space-indented witness line `at row
/// <Object#n>: lhs=<Object#n> rhs=<Object#n>`. The equation's paths and
/// the row objects come from the same schema the kernel checked, so the
/// lookups are total for every kernel-emitted witness; the bare-number
/// fallbacks exist for totality under programmatically assembled input.
fn render_violation(w: &EquationWitness, schema: &Schema, label: &str) -> Vec<String> {
    let equation = schema.equations.get(w.equation);
    let head = match equation {
        Some(eq) => format!(
            "violation: eq-{} {} ≠ {}{}",
            w.equation,
            path_string(&eq.lhs),
            path_string(&eq.rhs),
            label
        ),
        None => format!("violation: eq-{}{}", w.equation, label),
    };
    let row_line = match equation {
        Some(eq) => format!(
            "  at row {}: lhs={} rhs={}",
            row_at(path_domain(schema, &eq.lhs), w.row),
            row_at(path_codomain(schema, &eq.lhs), w.lhs),
            row_at(path_codomain(schema, &eq.rhs), w.rhs),
        ),
        None => format!("  at row {}: lhs={} rhs={}", w.row.0, w.lhs.0, w.rhs.0),
    };
    vec![head, row_line]
}

/// The dangling line pair: the arrow and the source row the dangling
/// value sits on.
fn render_dangling(w: &DanglingWitness, schema: &Schema, label: &str) -> Vec<String> {
    let head = format!("dangling: arrow {}{}", w.arrow.0, label);
    let src_object = schema.arrows.get(&w.arrow).map(|a| a.src.0.as_str());
    vec![head, format!("  at row {}", row_at(src_object, w.src))]
}

/// The epistemic label as rendered text: the kernel-computed default
/// (`executed`) stays off the human line so the §5 transcript is
/// byte-reproducible; the other weights append their label.
fn label_suffix(label: EpistemicLabel) -> String {
    match label {
        EpistemicLabel::Executed => String::new(),
        EpistemicLabel::Hypothesized => " (hypothesized)".to_string(),
        EpistemicLabel::Unknown => " (unknown)".to_string(),
    }
}

/// A path as the transcript spells it: the arrow names in composition
/// order joined with `∘`.
pub fn path_string(path: &Path) -> String {
    path.0
        .iter()
        .map(|ident| ident.0.as_str())
        .collect::<Vec<_>>()
        .join("∘")
}

/// A row as the transcript spells it: `Object#n` when the object is
/// known from the schema, the bare number otherwise (totality).
fn row_at(object: Option<&str>, row: RowId) -> String {
    match object {
        Some(name) => format!("{name}#{}", row.0),
        None => row.0.to_string(),
    }
}

/// The composed path's domain: its lead arrow's src.
fn path_domain<'a>(schema: &'a Schema, path: &Path) -> Option<&'a str> {
    schema.arrows.get(path.0.first()?).map(|a| a.src.0.as_str())
}

/// The composed path's codomain: its last arrow's tgt.
fn path_codomain<'a>(schema: &'a Schema, path: &Path) -> Option<&'a str> {
    schema.arrows.get(path.0.last()?).map(|a| a.tgt.0.as_str())
}

// ── Envelope construction (one path for both subcommands) ────────────

/// The findings payload: P2's serialization verbatim — the kernel's
/// `Vec<Finding>` under `data.findings`, no reshaping.
#[derive(Serialize)]
pub struct FindingsData<'a> {
    pub findings: &'a [Finding],
}

/// The one findings-envelope path both subcommands share: kind `Check`,
/// empty hints, the caller's warnings carried alongside the data.
pub fn findings_envelope(
    findings: &[Finding],
    warnings: Vec<Warning>,
) -> Envelope<FindingsData<'_>> {
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        FindingsData { findings },
        warnings,
        Vec::new(),
    )
}

/// The one error-envelope path: the envelope's error form with the
/// kernel's labeled error data under it — stable code, message with
/// position, remediation hint (Invariant 3.2.5: remediation non-empty).
pub fn error_envelope(err: &LabeledError, warnings: Vec<Warning>) -> Envelope<ErrorResult> {
    let result = ErrorResult::new(
        &err.code,
        &err.positioned_message(),
        None,
        None,
        None,
        Vec::new(),
        vec![RemediationEntry {
            command: "catherine check <model.ct.json>".to_string(),
            description: err.hint.to_string(),
        }],
    )
    .expect("remediation is non-empty by construction");
    Envelope::error(CLI_VERSION, result, warnings)
}

/// A config-loading warning, ruled under `config`.
pub fn config_warning(message: &str) -> Warning {
    Warning {
        rule_name: "config".to_string(),
        entity_id: None,
        message: message.to_string(),
        suggested_remediation: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::findings::{
        ContradictionWitness, DanglingWitness, DomainQuestion, RedundantWitness,
        UnderconstraintWitness,
    };
    use catherine_core::model::{Arrow, ConstraintStatus, Equation, Ident};
    use std::collections::BTreeMap;

    fn finding(kind: FindingKind, label: EpistemicLabel) -> Finding {
        Finding { label, kind }
    }

    /// The bowling schema, minimally reconstructed for the transcript
    /// pin: the four arrows and the one canonical equation.
    fn bowling_schema() -> Schema {
        let mut arrows = BTreeMap::new();
        for (name, src, tgt) in [
            ("staff_unit", "Assignment", "Staff"),
            ("assigned_staff", "Staff", "Unit"),
            ("equipment_unit", "Assignment", "Equipment"),
            ("assigned_item", "Equipment", "Unit"),
        ] {
            arrows.insert(
                Ident(name.to_string()),
                Arrow {
                    src: Ident(src.to_string()),
                    tgt: Ident(tgt.to_string()),
                },
            );
        }
        Schema {
            objects: BTreeMap::new(),
            arrows,
            equations: vec![Equation {
                lhs: Path(vec![
                    Ident("staff_unit".to_string()),
                    Ident("assigned_staff".to_string()),
                ]),
                rhs: Path(vec![
                    Ident("equipment_unit".to_string()),
                    Ident("assigned_item".to_string()),
                ]),
                rationale: "a row's staff and its equipment land in the same unit".to_string(),
                status: ConstraintStatus::Accepted,
            }],
        }
    }

    #[test]
    fn violation_renders_the_s5_block() {
        let schema = bowling_schema();
        let lines = render_one(
            &finding(
                FindingKind::Violation(EquationWitness {
                    equation: 0,
                    row: RowId(3),
                    lhs: RowId(1),
                    rhs: RowId(2),
                }),
                EpistemicLabel::Executed,
            ),
            &schema,
        );
        assert_eq!(
            lines,
            vec![
                "violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item",
                "  at row Assignment#3: lhs=Unit#1 rhs=Unit#2",
            ]
        );
    }

    #[test]
    fn non_executed_labels_append_to_the_head_line() {
        let schema = bowling_schema();
        let lines = render_one(
            &finding(
                FindingKind::Violation(EquationWitness {
                    equation: 0,
                    row: RowId(3),
                    lhs: RowId(1),
                    rhs: RowId(2),
                }),
                EpistemicLabel::Hypothesized,
            ),
            &schema,
        );
        assert!(lines[0].ends_with(" (hypothesized)"), "{}", lines[0]);
        assert_eq!(lines[1], "  at row Assignment#3: lhs=Unit#1 rhs=Unit#2");
    }

    #[test]
    fn dangling_renders_arrow_and_source_row() {
        let schema = bowling_schema();
        let lines = render_one(
            &finding(
                FindingKind::Dangling(DanglingWitness {
                    arrow: Ident("assigned_staff".to_string()),
                    src: RowId(0),
                }),
                EpistemicLabel::Executed,
            ),
            &schema,
        );
        assert_eq!(
            lines,
            vec!["dangling: arrow assigned_staff", "  at row Staff#0"]
        );
    }

    #[test]
    fn analysis_kinds_render_labeled_lines() {
        // The three analysis classes have no P4 producer; constructed
        // findings pin their renderings at the unit level (brief §3
        // criterion 4). No schema anchor is needed.
        let schema = Schema::default();

        let redundant = render_one(
            &finding(
                FindingKind::Redundant(RedundantWitness {
                    derivable: Path(vec![Ident("f".to_string()), Ident("g".to_string())]),
                    reducing: Path(vec![Ident("h".to_string())]),
                }),
                EpistemicLabel::Unknown,
            ),
            &schema,
        );
        assert_eq!(
            redundant,
            vec!["redundant: f∘g (unknown)", "  derivable as h",]
        );

        let contradiction = render_one(
            &finding(
                FindingKind::Contradiction(ContradictionWitness { scope: vec![0, 2] }),
                EpistemicLabel::Unknown,
            ),
            &schema,
        );
        assert_eq!(
            contradiction,
            vec!["contradiction: eq-0, eq-2 admit no instance (unknown)"]
        );

        let underconstraint = render_one(
            &finding(
                FindingKind::Underconstraint(UnderconstraintWitness {
                    case: DomainQuestion::ParallelPaths {
                        lhs: Path(vec![Ident("staff_unit".to_string())]),
                        rhs: Path(vec![Ident("equipment_unit".to_string())]),
                    },
                    question: "have no declared equation?".to_string(),
                }),
                EpistemicLabel::Hypothesized,
            ),
            &schema,
        );
        assert_eq!(
            underconstraint,
            vec![
                "underconstraint: parallel paths staff_unit / equipment_unit (hypothesized)",
                "  have no declared equation?",
            ]
        );
    }

    #[test]
    fn findings_envelope_carries_the_suite_shape() {
        let findings = vec![finding(
            FindingKind::Violation(EquationWitness {
                equation: 0,
                row: RowId(3),
                lhs: RowId(1),
                rhs: RowId(2),
            }),
            EpistemicLabel::Executed,
        )];
        let envelope = findings_envelope(&findings, Vec::new());
        let v: serde_json::Value = serde_json::to_value(&envelope).expect("envelope serializes");
        for key in ["ok", "envelope_kind", "data", "warnings", "hints"] {
            assert!(v.get(key).is_some(), "envelope key `{key}` present");
        }
        assert_eq!(v["ok"], true);
        assert_eq!(v["data"]["findings"][0]["kind"], "violation");
        assert_eq!(v["data"]["findings"][0]["witness"]["equation"], 0);
    }

    #[test]
    fn error_envelope_carries_code_position_and_hint() {
        let err = LabeledError {
            code: "duplicate_ident".to_string(),
            line: Some(14),
            column: Some(16),
            message: "duplicate arrow ident `in_section`".to_string(),
            hint: "idents must be unique within their collection — rename one of the duplicates",
        };
        let v: serde_json::Value =
            serde_json::to_value(error_envelope(&err, Vec::new())).expect("envelope serializes");
        assert_eq!(v["ok"], false);
        assert_eq!(v["envelope_kind"], "error");
        assert_eq!(v["data"]["code"], "duplicate_ident");
        let message = v["data"]["message"].as_str().expect("message");
        assert!(
            message.contains("line 14") && message.contains("column 16"),
            "{message}"
        );
        assert!(!v["data"]["remediation"]
            .as_array()
            .expect("remediation")
            .is_empty());
    }
}
