//! Purpose: the shared parse-error type — design Decision 3's
//! `{line, column, code, message}` shape and the stable, closed six-code
//! enum the golden fixtures pin.
//!
//! Responsibilities: define [`ErrorCode`], [`SemanticError`], and
//! [`ParseError`] for the parser and every future consumer (P1 checks
//! surface findings, not these — but format-level rejection reporting
//! stays uniform).
//!
//! Rationale: the set is closed at P0 (brief §10): extending it is a
//! follow-up ticket, never a silent addition. Structural malformations
//! outside the six codes (missing required fields, JSON type mismatches,
//! references to undeclared objects/rows) are `ParseError::Syntax` with
//! serde_json's own line/column and a human message.

/// The stable semantic error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    DuplicateIdent,
    UnknownArrow,
    IllTypedEquation,
    InvalidIdent,
    InvalidStatus,
    UnsupportedVersion,
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
        }
    }
}

/// A semantic (six-code) parse failure with the line/column of the
/// offending JSON node — 1-based; columns in characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub line: usize,
    pub column: usize,
    pub code: ErrorCode,
    pub message: String,
}

/// A parse failure: either JSON-structural (`Syntax` — serde_json's own
/// line/column and message) or semantic (`Semantic` — one of the six
/// stable codes at the offending node's position).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    Semantic(SemanticError),
}
