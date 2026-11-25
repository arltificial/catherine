//! Purpose: equation checking — path elements as declared arrows, shared
//! endpoint typing, constraint status, and the canonical equation order.
//! Split out of `parse/schema.rs` so both files stay under the size gate;
//! bodies moved verbatim, no behavior change.
//!
//! Responsibilities: the equation list check (sorted by sort key,
//! presentation order preserved as design Decision 4), the per-equation
//! path/endpoint/status checks, and the path walker behind them.
//!
//! Rationale: an equation is well typed exactly when both composed paths
//! map the same source object to the same target object; the endpoint
//! comparison is the whole typing rule.

use super::schema::{check_ident, check_status, ArrowTable, RawEquation, StatusCheck};
use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Arrow, Equation, Ident, Path};
use std::collections::BTreeMap;

/// One equation path under validation: its raw elements, its pointer
/// base, and the declared arrow table.
struct EqCheck<'a> {
    base: &'a str,
    path: &'a [String],
    table: &'a ArrowTable<'a>,
}

/// Equations: path idents, declared arrows, endpoint typing, status.
pub(super) fn check_equations(
    raw_equations: &[RawEquation],
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Vec<Equation>, crate::error::ParseError> {
    let table = ArrowTable { arrows, spans };
    let mut equations = Vec::with_capacity(raw_equations.len());
    for (i, raw) in raw_equations.iter().enumerate() {
        let base = format!("/schema/equations/{i}");
        equations.push(check_equation(raw, &base, &table)?);
    }
    // Canonical order: equation list order in the file is presentation
    // (design Decision 4).
    equations.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(equations)
}

/// One equation: path idents, endpoint typing, status.
fn check_equation(
    raw: &RawEquation,
    base: &str,
    table: &ArrowTable,
) -> Result<Equation, crate::error::ParseError> {
    let lhs = check_path(&EqCheck {
        base: &format!("{base}/lhs"),
        path: &raw.lhs,
        table,
    })?;
    let rhs = check_path(&EqCheck {
        base: &format!("{base}/rhs"),
        path: &raw.rhs,
        table,
    })?;
    check_same_endpoints(&lhs, &rhs, &EquationRef { raw, base, table })?;
    let status = check_status(&StatusCheck {
        status: raw.status.as_deref(),
        base,
        spans: table.spans,
        subject: "equation",
    })?;
    Ok(Equation {
        lhs,
        rhs,
        rationale: raw.rationale.clone(),
        status,
    })
}

/// The equation a check reports on: its raw declaration, pointer base,
/// and the declared arrow table.
struct EquationRef<'a> {
    raw: &'a RawEquation,
    base: &'a str,
    table: &'a ArrowTable<'a>,
}

/// The equation's two composed paths share both endpoints: they map the
/// same source object to the same target object (otherwise
/// `mismatched_endpoints` at the equation's node).
fn check_same_endpoints(
    lhs: &Path,
    rhs: &Path,
    equation: &EquationRef<'_>,
) -> Result<(), crate::error::ParseError> {
    let lhs_end = path_endpoints(lhs, equation.table.arrows);
    let rhs_end = path_endpoints(rhs, equation.table.arrows);
    if lhs_end == rhs_end {
        return Ok(());
    }
    let endpoints = EndpointMismatch {
        base: equation.base,
        lhs: &lhs_end,
        rhs: &rhs_end,
    };
    Err(mismatched_endpoints(
        equation.raw,
        &endpoints,
        equation.table.spans,
    ))
}

/// Two equation paths' endpoints, plus the pointer base the rejection
/// reports at.
struct EndpointMismatch<'a> {
    base: &'a str,
    lhs: &'a (String, String),
    rhs: &'a (String, String),
}

/// The labeled rejection of two paths spanning different src → tgt pairs.
fn mismatched_endpoints(
    raw: &RawEquation,
    ends: &EndpointMismatch,
    spans: &Spans,
) -> crate::error::ParseError {
    spans.semantic(
        ends.base,
        ErrorCode::IllTypedEquation,
        format!(
            "equation paths [{}] ({} -> {}) and [{}] ({} -> {}) have mismatched endpoints",
            raw.lhs.join(", "),
            ends.lhs.0,
            ends.lhs.1,
            raw.rhs.join(", "),
            ends.rhs.0,
            ends.rhs.1
        ),
    )
}

/// Validate one equation path: non-empty, every element an ident naming a
/// declared arrow. The unknown-arrow error names the whole path and points
/// at the offending element.
fn check_path(path: &EqCheck<'_>) -> Result<Path, crate::error::ParseError> {
    if path.path.is_empty() {
        return Err(path
            .table
            .spans
            .syntax(path.base, "equation path must be non-empty"));
    }
    for (j, name) in path.path.iter().enumerate() {
        let node = format!("{}/{j}", path.base);
        check_ident(path.table.spans, &node, name)?;
        if !path.table.arrows.contains_key(&Ident(name.clone())) {
            return Err(path.table.spans.semantic(
                &node,
                ErrorCode::UnknownArrow,
                format!(
                    "equation path [{}] references undeclared arrow `{name}`",
                    path.path.join(", ")
                ),
            ));
        }
    }
    Ok(Path(path.path.iter().map(|s| Ident(s.clone())).collect()))
}

/// Path endpoints: the src of the first arrow and the tgt of the last.
/// Every element is declared by [`check_path`], so the lookups are total.
fn path_endpoints(path: &Path, arrows: &BTreeMap<Ident, Arrow>) -> (String, String) {
    let first = &arrows[&path.0[0]];
    let last = &arrows[&path.0[path.0.len() - 1]];
    (first.src.0.clone(), last.tgt.0.clone())
}
