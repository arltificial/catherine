//! Purpose: the CLI's flag-value parsing — `--rows`, `--observe`,
//! `--seed`, `--direction`, and the rule lens's `--values` — the only
//! places CLI-level input errors are minted.
//!
//! Responsibilities: parse each flag's syntax; for observations also
//! validate against the schema and the first instance's row space; every
//! failure is a labeled `bad_argument` error with a remediation hint.
//!
//! Rationale: only the syntax is CLI's — node validation is the op's own
//! contract (the parity gate's unknown-seeds-rejected behavior,
//! reported as `unknown_seed`); the observation spelling is the
//! transcripts' `Object#n` dense-row id.

use crate::render::LabeledError;
use catherine_analysis::distinguish::Observation;
use catherine_analysis::query::{Direction, NodeRef};
use catherine_core::model::{ColumnType, Ident, Literal, Path, RowId, Schema};
use std::collections::BTreeMap;

/// A CLI-level input error: malformed flag values or semantically empty
/// observation/seed sets. Not a kernel code and not clap's usage code —
/// the args parse; their content does not serve.
pub fn bad_arg(message: String, hint: &'static str) -> LabeledError {
    LabeledError {
        code: "bad_argument".to_string(),
        line: None,
        column: None,
        message,
        hint,
    }
}

/// Parse the `--rows` bounds: `object=count`, repeatable, later
/// spellings win per object.
pub fn parse_rows(specs: &[String]) -> Result<BTreeMap<Ident, usize>, LabeledError> {
    let mut max_rows = BTreeMap::new();
    for spec in specs {
        let Some((object, count)) = spec.split_once('=') else {
            return Err(bad_arg(
                format!("--rows expects object=count, got `{spec}`"),
                "spell the bound as object=count, e.g. --rows Assignment=4",
            ));
        };
        let count: usize = count.parse().map_err(|_| {
            bad_arg(
                format!("--rows expects a row count, got `{count}` for object `{object}`"),
                "the count is a non-negative integer",
            )
        })?;
        max_rows.insert(Ident(object.to_string()), count);
    }
    Ok(max_rows)
}

/// Parse and validate the `--observe` list — each spec as
/// `path@object#row`, the row as its dense id (the transcripts'
/// `Object#n` spelling): the arrows must all exist in the schema and
/// compose from the object; the row must be inside the object's declared
/// row space of the first instance (the second instance's shortfall at
/// that row is an honest "no value", a difference, not an input error).
pub fn parse_observations(
    specs: &[String],
    schema: &Schema,
    instance: &catherine_core::model::Instance,
) -> Result<Vec<Observation>, LabeledError> {
    let mut observations = Vec::new();
    for spec in specs {
        observations.push(parse_observation(spec, schema, instance)?);
    }
    Ok(observations)
}

/// Split one `path@object#row` spec into its three spellings; either
/// separator missing is an input error.
fn split_observation_spec(spec: &str) -> Result<(&str, &str, &str), LabeledError> {
    let Some((path_part, rest)) = spec.split_once('@') else {
        return Err(bad_arg(
            format!("--observe expects path@object#row, got `{spec}`"),
            "spell the observation as path@object#row, e.g. --observe staff_unit∘assigned_staff@Assignment#3",
        ));
    };
    let Some((object, row)) = rest.split_once('#') else {
        return Err(bad_arg(
            format!("--observe expects path@object#row, got `{spec}`"),
            "the row is the dense id, spelled object#row",
        ));
    };
    Ok((path_part, object, row))
}

/// Parse and validate one `path@object#row` observation spec.
fn parse_observation(
    spec: &str,
    schema: &Schema,
    instance: &catherine_core::model::Instance,
) -> Result<Observation, LabeledError> {
    let (path_part, object, row) = split_observation_spec(spec)?;
    let arrows = resolve_observation_path(path_part, object, spec, schema)?;
    let row = validated_row(object, row, instance)?;
    Ok(Observation {
        object: Ident(object.to_string()),
        row: RowId(row),
        path: Path(
            arrows
                .into_iter()
                .map(|name| Ident(name.to_string()))
                .collect(),
        ),
    })
}

/// Parse the dense row id and check it against the first instance's
/// declared row space: a row outside it is an input error, not an
/// observation difference.
fn validated_row(
    object: &str,
    row: &str,
    instance: &catherine_core::model::Instance,
) -> Result<usize, LabeledError> {
    let row: usize = row.parse().map_err(|_| {
        bad_arg(
            format!("--observe expects a row number, got `{row}`"),
            "the row is the dense id, a non-negative integer",
        )
    })?;
    let in_space = instance
        .rows
        .get(&Ident(object.to_string()))
        .is_some_and(|rows| row < rows.len());
    if !in_space {
        return Err(bad_arg(
            format!("row `{object}#{row}` is not a row of the first instance"),
            "observe from a row the first instance declares",
        ));
    }
    Ok(row)
}

/// An observation path's arrow steps: the `∘`-separated segments, empty
/// for an observation on the object itself.
fn split_path(path_part: &str) -> Vec<&str> {
    if path_part.is_empty() {
        Vec::new()
    } else {
        path_part.split('∘').collect()
    }
}

/// Resolve the path side of an observation: the `∘`-joined arrow names
/// must all exist in the schema and compose from the observation object.
fn resolve_observation_path<'a>(
    path_part: &'a str,
    object: &str,
    spec: &str,
    schema: &Schema,
) -> Result<Vec<&'a str>, LabeledError> {
    let arrows = split_path(path_part);
    let mut current = object.to_string();
    for name in &arrows {
        match schema.arrows.get(&Ident(name.to_string())) {
            Some(arrow) if arrow.src.0 == current => current = arrow.tgt.0.clone(),
            Some(arrow) => {
                return Err(non_composing_arrow_error(
                    path_part,
                    name,
                    &arrow.src.0,
                    &current,
                ))
            }
            None => {
                return Err(bad_arg(
                    format!("unknown arrow `{name}` in observation `{spec}`"),
                    "declare the arrow in schema.arrows, or fix the path spelling",
                ));
            }
        }
    }
    Ok(arrows)
}

/// A path that does not compose: the step's arrow starts somewhere other
/// than the object the previous step (or the observation) landed on —
/// the CLI never guesses across a vocabulary gap.
fn non_composing_arrow_error(
    path_part: &str,
    name: &str,
    starts_at: &str,
    current: &str,
) -> LabeledError {
    bad_arg(
        format!(
            "path `{path_part}` does not compose: arrow `{}` starts at `{}`, not `{current}`",
            starts_at, name
        ),
        "both sides of an observation must start at the observation object",
    )
}

/// Parse the `--seed` list: `object#row`, the row as its dense id (the
/// same spelling the transcripts use). Only the syntax is CLI's — node
/// validation is the op's own `seeds_exist` contract (the parity gate's
/// unknown-seeds-rejected behavior), reported as `unknown_seed`.
pub fn parse_seeds(specs: &[String]) -> Result<Vec<NodeRef>, LabeledError> {
    let mut seeds = Vec::new();
    for spec in specs {
        let Some((object, row)) = spec.split_once('#') else {
            return Err(bad_arg(
                format!("--seed expects object#row, got `{spec}`"),
                "spell the seed as object#row, e.g. --seed Assignment#3",
            ));
        };
        let row: usize = row.parse().map_err(|_| {
            bad_arg(
                format!("--seed expects a row number, got `{row}`"),
                "the row is the dense id, a non-negative integer",
            )
        })?;
        seeds.push(NodeRef {
            object: Ident(object.to_string()),
            row: RowId(row),
        });
    }
    Ok(seeds)
}

/// Parse the `--direction` value.
pub fn parse_direction(direction: &str) -> Result<Direction, LabeledError> {
    match direction {
        "forward" => Ok(Direction::Forward),
        "backward" => Ok(Direction::Backward),
        other => Err(bad_arg(
            format!("--direction is forward or backward, got `{other}`"),
            "forward follows src→tgt; backward follows tgt→src",
        )),
    }
}

/// Parse the `--values` specs of the rule lens: `column=v1,v2,...`,
/// repeatable (later spellings win per column). Every token is parsed
/// against the column's declared type; missing columns are left to
/// `RuleDomain::from_presentation`, which names the omission — the CLI
/// never invents a value list.
pub fn parse_domain_values(
    specs: &[String],
    columns: &BTreeMap<Ident, ColumnType>,
) -> Result<BTreeMap<Ident, Vec<Literal>>, LabeledError> {
    let mut supplied = BTreeMap::new();
    for spec in specs {
        let (column, values) = parse_value_spec(spec, columns)?;
        supplied.insert(column, values);
    }
    Ok(supplied)
}

/// Parse one `column=v1,v2,...` spec against the declared columns; the
/// CLI never invents a value list, so every failure is a labeled error.
fn parse_value_spec(
    spec: &str,
    columns: &BTreeMap<Ident, ColumnType>,
) -> Result<(Ident, Vec<Literal>), LabeledError> {
    let Some((name, list)) = spec.split_once('=') else {
        return Err(bad_arg(
            format!("--values expects column=value[,value...], got `{spec}`"),
            "spell the values as column=v1,v2,..., e.g. --values age=10,20,30",
        ));
    };
    let column = Ident(name.to_string());
    let Some(column_type) = columns.get(&column) else {
        return Err(bad_arg(
            format!("--values names `{name}`, which the rule presentation does not declare"),
            "use a column declared in rule_presentation.columns",
        ));
    };
    if list.is_empty() {
        return Err(bad_arg(
            format!("--values gives no values for column `{name}`"),
            "an empty value list would make the analysis vacuous; supply at least one value",
        ));
    }
    let values = list
        .split(',')
        .map(|token| parse_literal(token, *column_type, name, spec))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((column, values))
}

/// One typed value of a `--values` spec; strings stay verbatim (commas
/// are the list separator, so a string value cannot contain one).
fn parse_literal(
    token: &str,
    column_type: ColumnType,
    column: &str,
    spec: &str,
) -> Result<Literal, LabeledError> {
    match column_type {
        ColumnType::Int => token.parse::<i64>().map(Literal::Int).map_err(|_| {
            bad_arg(
                format!("`{token}` in `{spec}` is not an int for column `{column}`"),
                "int columns take decimal i64 values",
            )
        }),
        ColumnType::Bool => match token {
            "true" => Ok(Literal::Bool(true)),
            "false" => Ok(Literal::Bool(false)),
            _ => Err(bad_arg(
                format!("`{token}` in `{spec}` is not a bool for column `{column}`"),
                "bool columns take `true` or `false`",
            )),
        },
        ColumnType::String => Ok(Literal::String(token.to_string())),
    }
}
