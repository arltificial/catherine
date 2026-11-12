//! Purpose: the shared parse-error type — design Decision 3's
//! `{line, column, code, message}` shape and the stable, closed error-code
//! enum the golden fixtures pin.
//!
//! Responsibilities: define [`ErrorCode`], [`SemanticError`], and
//! [`ParseError`] for the parser and every future consumer (P1 checks
//! surface findings, not these — but format-level rejection reporting
//! stays uniform).
//!
//! Rationale: the set is closed at P0 (brief §10): extending it is a
//! follow-up ticket, never a silent addition. Structural malformations
//! outside the closed set (missing required fields, JSON type mismatches,
//! references to undeclared objects/rows) are `ParseError::Syntax` with
//! serde_json's own line/column and a human message.

/// The stable semantic error codes. Closed at P0; extended only by a
/// named ticket — `partial_schema_map` (openspec change
/// `add-delta-migration` phase A) and the rule-presentation codes
/// (openspec change `add-rule-lens` phase A) are the extensions so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    DuplicateIdent,
    UnknownArrow,
    IllTypedEquation,
    InvalidIdent,
    InvalidStatus,
    UnsupportedVersion,
    /// A schema map that fails to assign every declared source object or
    /// arrow — never repaired with a silent default.
    PartialSchemaMap,
    /// A rule presentation's `domain` names no declared schema object.
    UndeclaredRuleDomain,
    /// A rule presentation column's `type` is outside `int`/`bool`/`string`.
    InvalidColumnType,
    /// A predicate references a column the presentation does not declare.
    UnknownColumn,
    /// A predicate is malformed or ill-typed (unknown kind or operator, a
    /// literal of the wrong type, `mod` misuse, empty `and`/`or`).
    InvalidPredicate,
    /// A presentation policy is outside the three declared policies.
    InvalidPolicy,
    /// A rule assigns no action — the action must be a total function on
    /// the input domain.
    MissingAction,
}

impl ErrorCode {
    /// The stable spelling used by the fixture table and messages.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DuplicateIdent => "duplicate_ident",
            Self::UnknownArrow => "unknown_arrow",
            Self::IllTypedEquation => "ill_typed_equation",
            Self::InvalidIdent => "invalid_ident",
            Self::InvalidStatus => "invalid_status",
            Self::UnsupportedVersion => "unsupported_version",
            Self::PartialSchemaMap => "partial_schema_map",
            Self::UndeclaredRuleDomain => "undeclared_rule_domain",
            Self::InvalidColumnType => "invalid_column_type",
            Self::UnknownColumn => "unknown_column",
            Self::InvalidPredicate => "invalid_predicate",
            Self::InvalidPolicy => "invalid_policy",
            Self::MissingAction => "missing_action",
        }
    }
}

/// A semantic (closed-code-set) parse failure with the line/column of the
/// offending JSON node — 1-based; columns in characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub line: usize,
    pub column: usize,
    pub code: ErrorCode,
    pub message: String,
}

/// A parse failure: either JSON-structural (`Syntax` — serde_json's own
/// line/column and message) or semantic (`Semantic` — one of the stable
/// codes at the offending node's position).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    Semantic(SemanticError),
}
