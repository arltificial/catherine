//! The rejection-table helper: one shape for every parse rejection.

use catherine_core::error::ParseError;
use catherine_core::parse::parse;

/// The stable code, `(line, column)`, and message of a rejected
/// document: the syntactic and semantic arms collapsed to one shape for
/// the rejection table.
pub(crate) fn rejection(text: &str, what: &str) -> (&'static str, (usize, usize), String) {
    let Err(error) = parse(text) else {
        panic!("{what}: the document parses, a rejection was required");
    };
    match error {
        ParseError::Syntax {
            line,
            column,
            message,
        } => ("syntax", (line, column), message),
        ParseError::Semantic(semantic) => (
            semantic.code.as_str(),
            (semantic.line, semantic.column),
            semantic.message,
        ),
    }
}
