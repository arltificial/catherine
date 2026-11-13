//! Purpose: the `rules --project` bridge (openspec change
//! `add-rules-projection`, step 1) — project an object's instance rows
//! onto a rule table and feed the shipped rule lens, so a stored table
//! can be analyzed without hand-authoring the table it already holds as
//! data.
//!
//! Responsibilities: resolve `--project`/`--columns`/`--action` against
//! the schema (declared object, arrows sourced at it, no duplicates), one
//! rule per row in dense `RowId` order (predicate = the conjunction of
//! `column = target-row-name` equalities in the given order, column type
//! always `String`; action = the row's own name or the `--action` arrow's
//! target name), `policy: DeclaredDisjoint` fixed, and the exact-tuple
//! domain = the distinct observed column tuples unioned with any
//! `--values` product declaration. Every refusal is a labeled CLI error
//! (`bad_argument`, `projection_incomplete`, `empty_projection`) — a row
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
//! The exact input set is deduplicated and canonically sorted here, then
//! stored in that order by `RuleDomain::from_tuples`, so witness indices
//! are deterministic.

use crate::args::{bad_arg, parse_domain_values};
use crate::render::LabeledError;
use crate::rules::bad_values;
use catherine_core::model::{
    CmpOp, ColumnType, Ident, Literal, Model, Policy, Predicate, RowId, Rule, RulePresentation,
};
use catherine_core::parse::RowNames;
use catherine_core::rule_lens::{RuleDomain, RuleInput};
use std::collections::{BTreeMap, BTreeSet};

/// One projection invocation, as spelled on the command line.
pub struct Projection<'a> {
    /// The object whose instance rows become rules.
    pub object: &'a str,
    /// The comma-separated `--columns` arrow list (absent = refused).
    pub columns: Option<&'a str>,
    /// The optional `--action` arrow (identity when absent).
    pub action: Option<&'a str>,
}

/// One row's projected values, keyed by the column arrow.
struct RowProjection {
    name: Ident,
    values: BTreeMap<Ident, Ident>,
    action: String,
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
    let resolved = resolve_spec(model, projection)?;
    let Some(row_names) = rows.get(&resolved.object) else {
        return Err(empty_projection(&resolved.object));
    };
    if row_names.is_empty() {
        return Err(empty_projection(&resolved.object));
    }
    let projected = project_rows(
        model,
        rows,
        &resolved.columns,
        resolved.action.as_ref(),
        row_names,
    )?;
    let presentation = presentation(&resolved.object, &resolved.columns, &projected);
    let domain = exact_domain(&presentation, &projected, values)?;
    Ok((presentation, domain))
}

/// The resolved invocation: a declared object with arrows sourced at it.
struct Resolved {
    object: Ident,
    columns: Vec<Ident>,
    action: Option<Ident>,
}

/// Validate the invocation shape: one presentation source, a declared
/// object, a non-empty column list, and arrows sourced at the object.
fn resolve_spec(model: &Model, projection: &Projection<'_>) -> Result<Resolved, LabeledError> {
    refuse_second_source(model, projection.object)?;
    let object = Ident(projection.object.to_string());
    if !model.schema.objects.contains_key(&object) {
        return Err(bad_arg(
            format!(
                "`--project` names `{}`, which is not a declared schema object",
                projection.object
            ),
            "project an object declared in schema.objects",
        ));
    }
    let Some(column_spec) = projection.columns else {
        return Err(bad_arg(
            format!(
                "`--project {}` requires `--columns <arrow,...>`",
                projection.object
            ),
            "a projection with no columns names no rule predicate",
        ));
    };
    let columns = resolve_arrows(model, &object, column_spec, "--columns")?;
    let action = match projection.action {
        Some(spec) => {
            let mut arrows = resolve_arrows(model, &object, spec, "--action")?;
            if arrows.len() != 1 {
                return Err(bad_arg(
                    format!("`--action` takes exactly one arrow, got `{spec}`"),
                    "an action is one arrow: a second entry would be silently ignored",
                ));
            }
            Some(arrows.remove(0))
        }
        None => None,
    };
    Ok(Resolved {
        object,
        columns,
        action,
    })
}

/// `--project` on a document that already carries a `rule_presentation`
/// is two sources of the same presentation: never an implicit preference.
fn refuse_second_source(model: &Model, object: &str) -> Result<(), LabeledError> {
    if model.rule_presentation.is_none() {
        return Ok(());
    }
    Err(bad_arg(
        format!(
            "`--project {object}` was given, but the document already carries a `rule_presentation` — keep one presentation source"
        ),
        "drop the document's `rule_presentation`, or analyze it with `--values` instead of `--project`",
    ))
}

/// Resolve one arrow-list flag: a declared arrow per comma-separated
/// entry, sourced at the projected object, no empties, no duplicates.
fn resolve_arrows(
    model: &Model,
    object: &Ident,
    spec: &str,
    flag: &str,
) -> Result<Vec<Ident>, LabeledError> {
    let mut arrows: Vec<Ident> = Vec::new();
    for name in spec.split(',') {
        let arrow = resolve_arrow(model, object, name, flag)?;
        if arrows.contains(&arrow) {
            return Err(bad_arg(
                format!("`{flag}` names `{name}` twice — each column is one arrow"),
                "a duplicated column would project the same value twice",
            ));
        }
        arrows.push(arrow);
    }
    Ok(arrows)
}

/// One `--columns`/`--action` entry: declared, and sourced at the object.
fn resolve_arrow(
    model: &Model,
    object: &Ident,
    name: &str,
    flag: &str,
) -> Result<Ident, LabeledError> {
    if name.is_empty() {
        return Err(bad_arg(
            format!("empty column entry in `{flag}`"),
            "name one declared arrow per comma-separated entry",
        ));
    }
    let arrow = Ident(name.to_string());
    let Some(declared) = model.schema.arrows.get(&arrow) else {
        return Err(bad_arg(
            format!("`{flag}` names `{name}`, which is not a declared arrow"),
            "declare the arrow in schema.arrows, or fix the spelling",
        ));
    };
    if declared.src != *object {
        return Err(bad_arg(
            format!(
                "`{flag}` names `{name}`, which is not sourced at `{}`",
                object.0
            ),
            "every projected column and action arrow must start at the projected object",
        ));
    }
    Ok(arrow)
}

/// Project every row in dense id order; a null cell refuses the run.
fn project_rows(
    model: &Model,
    rows: &RowNames,
    columns: &[Ident],
    action: Option<&Ident>,
    row_names: &[Ident],
) -> Result<Vec<RowProjection>, LabeledError> {
    row_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let row = RowId(index);
            let mut values = BTreeMap::new();
            for column in columns {
                values.insert(column.clone(), cell_name(model, rows, column, row, name)?);
            }
            let action = match action {
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
fn presentation(
    object: &Ident,
    columns: &[Ident],
    projected: &[RowProjection],
) -> RulePresentation {
    let rules = projected
        .iter()
        .map(|row| Rule {
            name: row.name.clone(),
            predicate: Predicate::And(
                columns
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
        domain: object.clone(),
        columns: columns
            .iter()
            .map(|column| (column.clone(), ColumnType::String))
            .collect(),
        rules,
        policy: Some(Policy::DeclaredDisjoint),
    }
}

/// The exact domain: distinct observed tuples ∪ the `--values` product,
/// canonically sorted (the tuple order `from_tuples` preserves).
fn exact_domain(
    presentation: &RulePresentation,
    projected: &[RowProjection],
    values: &[String],
) -> Result<RuleDomain, LabeledError> {
    let mut inputs: BTreeSet<RuleInput> = projected
        .iter()
        .map(|row| {
            presentation
                .columns
                .keys()
                .map(|column| Literal::String(row.values[column].0.clone()))
                .collect()
        })
        .collect();
    if !values.is_empty() {
        let supplied = parse_domain_values(values, &presentation.columns)?;
        let product = RuleDomain::from_presentation(presentation, supplied)
            .map_err(|error| bad_values(&error))?;
        inputs.extend(product.inputs());
    }
    RuleDomain::from_tuples(presentation, inputs.into_iter().collect())
        .map_err(|error| bad_values(&error))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Two captures colliding on `(source, edition)`, plus one distinct
    /// row; every cell resolves.
    fn fixture() -> (Model, RowNames) {
        let doc = r#"{
            "format_version": "1.0",
            "schema": {
                "objects": ["capture", "source", "edition", "batch"],
                "arrows": [
                    {"name": "capture_source", "src": "capture", "tgt": "source"},
                    {"name": "capture_edition", "src": "capture", "tgt": "edition"},
                    {"name": "capture_batch", "src": "capture", "tgt": "batch"},
                    {"name": "source_edition", "src": "source", "tgt": "edition"}
                ],
                "equations": []
            },
            "instance": {
                "rows": {
                    "capture": ["cap_c", "cap_b", "cap_a"],
                    "source": ["src_a", "src_b"],
                    "edition": ["ed_1", "ed_2"],
                    "batch": ["b1", "b2"]
                },
                "tables": {
                    "capture_source": {"cap_a": "src_a", "cap_b": "src_a", "cap_c": "src_b"},
                    "capture_edition": {"cap_a": "ed_1", "cap_b": "ed_1", "cap_c": "ed_2"},
                    "capture_batch": {"cap_a": "b1", "cap_b": "b1", "cap_c": "b2"}
                }
            }
        }"#;
        catherine_core::parse::parse_with_rows(doc).expect("fixture parses")
    }

    fn projection<'a>(columns: &'a str, action: Option<&'a str>) -> Projection<'a> {
        Projection {
            object: "capture",
            columns: Some(columns),
            action,
        }
    }

    /// Rules follow dense row order; predicates are conjunctive
    /// equalities in the given column order; the action is identity by
    /// default and the arrow target otherwise.
    #[test]
    fn rows_become_rules_in_dense_order() {
        let (model, rows) = fixture();
        let (presentation, _) = project(
            &model,
            &rows,
            &projection("capture_source,capture_edition", None),
            &[],
        )
        .expect("projection succeeds");
        let names: Vec<&str> = presentation
            .rules
            .iter()
            .map(|r| r.name.0.as_str())
            .collect();
        assert_eq!(names, vec!["cap_a", "cap_b", "cap_c"], "dense row order");
        assert_eq!(
            presentation.columns,
            BTreeMap::from([
                (Ident("capture_source".to_string()), ColumnType::String),
                (Ident("capture_edition".to_string()), ColumnType::String),
            ])
        );
        assert_eq!(presentation.policy, Some(Policy::DeclaredDisjoint));
        assert_eq!(presentation.rules[0].action, "cap_a");
        assert_eq!(
            presentation.rules[0].predicate,
            Predicate::And(vec![
                Predicate::Cmp {
                    column: Ident("capture_source".to_string()),
                    op: CmpOp::Eq,
                    value: Literal::String("src_a".to_string()),
                },
                Predicate::Cmp {
                    column: Ident("capture_edition".to_string()),
                    op: CmpOp::Eq,
                    value: Literal::String("ed_1".to_string()),
                },
            ]),
            "predicate order follows --columns order"
        );
    }

    /// `--action` replaces the identity action with the arrow's target
    /// row name.
    #[test]
    fn the_action_arrow_projects_target_row_names() {
        let (model, rows) = fixture();
        let (presentation, _) = project(
            &model,
            &rows,
            &projection("capture_source", Some("capture_batch")),
            &[],
        )
        .expect("projection succeeds");
        let actions: Vec<&str> = presentation
            .rules
            .iter()
            .map(|rule| rule.action.as_str())
            .collect();
        assert_eq!(actions, vec!["b1", "b1", "b2"]);
    }

    /// The observed tuple domain is deduplicated and canonically sorted;
    /// a declared product input is unioned in.
    #[test]
    fn the_domain_is_the_observed_set_plus_any_declaration() {
        let (model, rows) = fixture();
        let (presentation, domain) = project(
            &model,
            &rows,
            &projection("capture_source,capture_edition", None),
            &[],
        )
        .expect("projection succeeds");
        let observed = vec![
            vec![
                Literal::String("ed_1".to_string()),
                Literal::String("src_a".to_string()),
            ],
            vec![
                Literal::String("ed_2".to_string()),
                Literal::String("src_b".to_string()),
            ],
        ];
        assert_eq!(domain.inputs(), observed, "deduped, in column-key order");

        let (_, declared) = project(
            &model,
            &rows,
            &projection("capture_source,capture_edition", None),
            &[
                "capture_source=src_a,src_b".to_string(),
                "capture_edition=ed_1,ed_2".to_string(),
            ],
        )
        .expect("projection succeeds");
        assert_eq!(
            declared.inputs().len(),
            4,
            "the declared product is unioned"
        );
        assert!(presentation
            .columns
            .contains_key(&Ident("capture_source".into())));
    }

    /// A schema arrow is not automatically a projection arrow: it must
    /// start at the projected object.
    #[test]
    fn an_arrow_sourced_elsewhere_is_refused() {
        let (model, rows) = fixture();
        let bad = Projection {
            object: "capture",
            columns: Some("source_edition"),
            action: None,
        };
        let error = project(&model, &rows, &bad, &[]).expect_err("not sourced at capture");
        assert_eq!(error.code, "bad_argument");
        assert!(error.message.contains("source_edition"));
    }
}
