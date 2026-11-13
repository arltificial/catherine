//! Purpose: the CLI's labeled error type — the stable code, the known
//! position, the kernel's message verbatim, and a remediation hint.
//!
//! Responsibilities: convert a kernel [`ParseError`] into the CLI's
//! [`LabeledError`]; render the position suffix and the human head
//! line; keep the remediation-hint table for the kernel's closed,
//! named-ticket-extended code set.
//!
//! Rationale: the CLI renders existing codes and never extends them
//! (the kernel enum is closed; additive-section extensions arrive with
//! their own hints); errors are labeled two-liners, never a stack trace.

use catherine_core::error::{ErrorCode, ParseError};

/// A labeled CLI-level error: the stable code, the position when the
/// kernel or the filesystem pins one, the kernel's message verbatim,
/// and a remediation hint. Never a stack trace.
#[derive(Debug)]
pub struct LabeledError {
    /// The stable code: one of the kernel's closed semantic codes spelled
    /// as the fixtures pin it, or `syntax` for JSON-structural
    /// malformations, or `read_error` for an unreadable input path.
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
    /// never extends them (the kernel's closed enum gains codes only by
    /// named ticket).
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

/// The remediation hints, keyed by the closed error-code enum.
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
            "set format_version to \"1.3\" — the known version is 1.3; raise KNOWN_MINOR when new additive semantics land"
        }
        ErrorCode::PartialSchemaMap => {
            "a schema map must be total: assign every declared source object and arrow — map it to the target, or to null to drop it explicitly"
        }
        ErrorCode::UndeclaredRuleDomain => {
            "declare the rule presentation's domain in schema.objects, or fix the domain spelling"
        }
        ErrorCode::InvalidColumnType => "column types are one of: int, bool, string",
        ErrorCode::UnknownColumn => {
            "declare the column in rule_presentation.columns, or fix the predicate spelling"
        }
        ErrorCode::InvalidPredicate => {
            "predicates are comparisons, mod, and/or/not over declared columns; a comparison literal must match its column's type"
        }
        ErrorCode::InvalidPolicy => {
            "policy is one of: PriorityOrder, FirstMatch, DeclaredDisjoint (omit it to declare no precedence)"
        }
        ErrorCode::MissingAction => "every rule must assign a non-empty action",
        ErrorCode::InvalidKey => {
            "a key names a declared object and a non-empty, duplicate-free list of arrows sourced at that object"
        }
    }
}
