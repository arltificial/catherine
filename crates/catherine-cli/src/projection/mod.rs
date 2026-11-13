//! Purpose: the `rules --project` bridge (openspec change
//! `add-rules-projection`, step 1) — project an object's instance rows
//! onto a rule table and feed the shipped rule lens, so a stored table
//! can be analyzed without hand-authoring the table it already holds as
//! data.
//!
//! Responsibilities: [`project`] is the one entry point — it validates the
//! invocation ([`spec`]), projects every row and builds the exact-tuple
//! domain ([`build`]) — and every refusal is a labeled CLI error
//! (`bad_argument`, `projection_incomplete`, `empty_projection`): a row
//! that cannot be projected is never silently dropped and an empty
//! projection is never a clean run.
//!
//! Rationale: the projection is a pure translation `Model + RowNames ->
//! RulePresentation + RuleDomain`; no new analysis path and no new
//! finding kind exists. Names are read from the parser's side table, so
//! the model's hash, canonical bytes, and equality stay name-blind. The
//! projected claim is **not sealed evidence**: it is outside the document
//! hash and the ledger, and it describes this snapshot of rows projected
//! this way. `DeclaredDisjoint` is the only policy because a projection
//! asserts its rows are a function of the projected columns — resolving a
//! collision by order would silently pick one of the colliding rows.
//! `--values` keeps its product semantics (a declared combination no row
//! has is a genuine gap: the declared scope, not a claim about the data).
//! Layout: [`spec`] resolves/validates the flag shape, [`build`] turns
//! rows into rules and the domain, and this module owns the seam the
//! runner calls.

mod build;
mod spec;

#[cfg(test)]
mod tests;

use crate::render::LabeledError;
use catherine_core::model::{Ident, Model, RulePresentation};
use catherine_core::parse::RowNames;
use catherine_core::rule_lens::RuleDomain;

/// One projection invocation, as spelled on the command line.
pub struct Projection<'a> {
    /// The object whose instance rows become rules.
    pub object: &'a str,
    /// The comma-separated `--columns` arrow list (absent = refused).
    pub columns: Option<&'a str>,
    /// The optional `--action` arrow (identity when absent).
    pub action: Option<&'a str>,
}

/// Project `projection.object`'s rows onto a rule table and its exact
/// input domain. Returns the presentation (owned: it does not live in the
/// document) and the domain the analyzer enumerates.
pub fn project(
    model: &Model,
    rows: &RowNames,
    projection: &Projection<'_>,
    values: &[String],
) -> Result<(RulePresentation, RuleDomain), LabeledError> {
    let resolved = spec::resolve_spec(model, projection)?;
    let Some(row_names) = rows.get(&resolved.object) else {
        return Err(empty_projection(&resolved.object));
    };
    if row_names.is_empty() {
        return Err(empty_projection(&resolved.object));
    }
    let projected = build::project_rows(model, rows, &resolved, row_names)?;
    let presentation = build::presentation(&resolved, &projected);
    let domain = build::exact_domain(&presentation, &projected, values)?;
    Ok((presentation, domain))
}

/// The projected object has no rows: an empty projection supports no
/// claim, and a clean exit would be the vacuous-clean defect.
fn empty_projection(object: &Ident) -> LabeledError {
    LabeledError {
        code: "empty_projection".to_string(),
        line: None,
        column: None,
        message: format!(
            "`{}` has no instance rows — an empty projection supports no claim",
            object.0
        ),
        hint: "project an object whose instance carries the rows to analyze",
    }
}
