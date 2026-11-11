//! Shared test fixture builder for the analysis integration tests.
//!
//! Purpose: one home for the tiny name/identifier builder both test
//! binaries use (`ident`); each binary keeps the fixtures it alone
//! exercises (the analysis fixtures in `analysis_fixtures.rs`, the
//! parity fixtures and the reference walk in `query_parity.rs`).
//!
//! Responsibilities: expose `ident` so both binaries spell object and
//! arrow names the same way.
//!
//! Rationale: `common` is compiled into *each* test binary, so any
//! helper only one binary calls is dead code in the other under
//! `-D warnings`. Only the truly shared builder lives here.

use catherine_core::model::Ident;

pub fn ident(name: &str) -> Ident {
    Ident(name.to_string())
}
