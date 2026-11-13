//! Purpose: turn a resolved projection into the two artifacts the rule
//! lens consumes — the synthesized `RulePresentation` and its exact input
//! domain.
//!
//! Responsibilities: one rule per row in dense `RowId` order (predicate =
//! the conjunction of `column = target-row-name` equalities in the given
//! order, column type always `String`; action = the row's own name or the
//! `--action` arrow's target name), `policy: DeclaredDisjoint` fixed, and
//! the domain = the distinct observed column tuples unioned with any
//! `--values` product declaration, canonically sorted. A dangling (`null`)
//! projected cell refuses the run with `projection_incomplete` naming the
//! row and arrow.
//!
//! Rationale: a projection never invents an input — the observed tuples
//! are the domain, so a selector with no row contributes nothing — while
//! an explicit `--values` declaration names combinations no row has, which
//! the gap finding then reports as the declared scope. Deduplicating and
//! sorting here (then storing that order in `RuleDomain::from_tuples`)
//! keeps witness indices deterministic.

use crate::args::parse_domain_values;
use crate::projection::spec::Resolved;
use crate::render::LabeledError;
use crate::rules::bad_values;
use catherine_core::model::{
    CmpOp, ColumnType, Ident, Literal, Model, Policy, Predicate, RowId, Rule, RulePresentation,
};
use catherine_core::parse::RowNames;
use catherine_core::rule_lens::{RuleDomain, RuleInput};
use std::collections::{BTreeMap, BTreeSet};

/// One row's projected values, keyed by the column arrow.
pub(super) struct RowProjection {
    name: Ident,
    values: BTreeMap<Ident, Ident>,
    action: String,
}

/// Project every row in dense id order; a null cell refuses the run.
pub(super) fn project_rows(
    model: &Model,
    rows: &RowNames,
    resolved: &Resolved,
    row_names: &[Ident],
) -> Result<Vec<RowProjection>, LabeledError> {
    row_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let row = RowId(index);
            let mut values = BTreeMap::new();
            for column in &resolved.columns {
                values.insert(column.clone(), cell_name(model, rows, column, row, name)?);
            }
            let action = match resolved.action.as_ref() {
                Some(arrow) => cell_name(model, rows, arrow, row, name)?.0,
                None => name.0.clone(),
            };
            Ok(RowProjection {
                name: name.clone(),
                values,
                action,
            })
        })
        .collect()
}

/// One cell's target row name; a dangling (`null`) cell names the row and
/// the arrow — a row that cannot be projected is not silently dropped.
fn cell_name(
    model: &Model,
    rows: &RowNames,
    arrow_name: &Ident,
    row: RowId,
    row_name: &Ident,
) -> Result<Ident, LabeledError> {
    let target = model
        .instance
        .values
        .get(arrow_name)
        .and_then(|cells| cells.get(row.0))
        .copied()
        .flatten()
        .ok_or_else(|| incomplete(row_name, arrow_name))?;
    let arrow = &model.schema.arrows[arrow_name];
    rows.get(&arrow.tgt)
        .and_then(|names| names.get(target.0))
        .cloned()
        .ok_or_else(|| incomplete(row_name, arrow_name))
}

/// The synthesized presentation: one rule per row, conjunctive column
/// equalities in the given order, identity/`--action` action, and the
/// fixed `DeclaredDisjoint` policy.
pub(super) fn presentation(resolved: &Resolved, projected: &[RowProjection]) -> RulePresentation {
    let rules = projected
        .iter()
        .map(|row| Rule {
            name: row.name.clone(),
            predicate: Predicate::And(
                resolved
                    .columns
                    .iter()
                    .map(|column| Predicate::Cmp {
                        column: column.clone(),
                        op: CmpOp::Eq,
                        value: Literal::String(row.values[column].0.clone()),
                    })
                    .collect(),
            ),
            action: row.action.clone(),
        })
        .collect();
    RulePresentation {
        domain: resolved.object.clone(),
        columns: resolved
            .columns
            .iter()
            .map(|column| (column.clone(), ColumnType::String))
            .collect(),
        rules,
        policy: Some(Policy::DeclaredDisjoint),
    }
}

/// The exact domain: distinct observed tuples ∪ the `--values` product,
/// canonically sorted (the tuple order `from_tuples` preserves).
pub(super) fn exact_domain(
    presentation: &RulePresentation,
    projected: &[RowProjection],
    values: &[String],
) -> Result<RuleDomain, LabeledError> {
    let mut inputs: BTreeSet<RuleInput> = observed_tuples(presentation, projected);
    if !values.is_empty() {
        let supplied = parse_domain_values(values, &presentation.columns)?;
        let product = RuleDomain::from_presentation(presentation, supplied)
            .map_err(|error| bad_values(&error))?;
        inputs.extend(product.inputs());
    }
    RuleDomain::from_tuples(presentation, inputs.into_iter().collect())
        .map_err(|error| bad_values(&error))
}

/// One tuple per row, in the presentation's (sorted) column order.
fn observed_tuples(
    presentation: &RulePresentation,
    projected: &[RowProjection],
) -> BTreeSet<RuleInput> {
    projected
        .iter()
        .map(|row| {
            presentation
                .columns
                .keys()
                .map(|column| Literal::String(row.values[column].0.clone()))
                .collect()
        })
        .collect()
}

/// A projected cell is absent: the row cannot be spelled, so the run is
/// refused rather than dropping it from either rules or domain.
fn incomplete(row: &Ident, arrow: &Ident) -> LabeledError {
    LabeledError {
        code: "projection_incomplete".to_string(),
        line: None,
        column: None,
        message: format!(
            "row `{}` has no value on arrow `{}` — a projection cannot drop a row",
            row.0, arrow.0
        ),
        hint: "give every projected row a value on every `--columns`/`--action` arrow (no `null` cells)",
    }
}
