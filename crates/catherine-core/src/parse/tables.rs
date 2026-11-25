//! Purpose: the instance-table parse pass (check order step 10) — every
//! declared arrow carries a total table over its source object's rows.
//! Split out of `parse/mod.rs` verbatim (mechanical lint refactor; no
//! behavior change) so the parse composition module stays inside the
//! repo's file-size gate.
//!
//! Responsibilities: duplicate row idents inside one table (from the span
//! scan, before serde's last-wins map can hide them), declared-arrow
//! table names, source rows that exist, and target values that are
//! declared target rows or explicit `null` (the dangling value).
//!
//! Rationale: check order is the document order of the crate README, so
//! an error is predictable and never order-dependent on the walk; the
//! total-vector initialization happens before cell assignment so an
//! unspecified cell is `None`, never missing.

use super::schema::RawInstance;
use super::spans::Spans;
use crate::error::{ErrorCode, ParseError, SemanticError};
use crate::model::{Arrow, Ident, RowId};
use std::collections::BTreeMap;

/// Instance tables: repeated keys, declared arrows, declared src rows,
/// and values that are declared target rows or `null`. Every declared
/// arrow carries a total vector over its source object's rows; an
/// unspecified cell parses as `None` (dangling).
pub(super) fn check_tables(
    instance: &RawInstance,
    arrows: &BTreeMap<Ident, Arrow>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    spans: &Spans,
) -> Result<BTreeMap<Ident, Vec<Option<RowId>>>, ParseError> {
    check_table_dup_keys(spans)?;
    for arrow_name in instance.tables.keys() {
        if !arrows.contains_key(&Ident(arrow_name.clone())) {
            return Err(spans.syntax_key(
                "/instance/tables",
                arrow_name,
                format!("instance table names undeclared arrow `{arrow_name}`"),
            ));
        }
    }
    let mut values: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for (name, arrow) in arrows {
        let n = row_index.get(&arrow.src.0).map_or(0, BTreeMap::len);
        values.insert(name.clone(), vec![None; n]);
    }
    for (arrow_name, table) in &instance.tables {
        let arrow = &arrows[&Ident(arrow_name.clone())];
        check_table_cells(arrow_name, arrow, table, row_index, &mut values, spans)?;
    }
    Ok(values)
}

/// Repeated keys inside one table are a duplicate row ident.
fn check_table_dup_keys(spans: &Spans) -> Result<(), ParseError> {
    if let Some(dup) = spans
        .dup_keys
        .iter()
        .find(|d| d.object.starts_with("/instance/tables/"))
    {
        let arrow = dup.object.trim_start_matches("/instance/tables/");
        return Err(ParseError::Semantic(SemanticError {
            line: dup.line,
            column: dup.column,
            code: ErrorCode::DuplicateIdent,
            message: format!("duplicate row ident `{}` in table `{arrow}`", dup.key),
        }));
    }
    Ok(())
}

/// One table's cells: src row must exist, target row must exist (or the
/// cell is an explicit `null` — the same dangling value as an absent
/// cell).
fn check_table_cells(
    arrow_name: &str,
    arrow: &Arrow,
    table: &BTreeMap<String, Option<String>>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
    values: &mut BTreeMap<Ident, Vec<Option<RowId>>>,
    spans: &Spans,
) -> Result<(), ParseError> {
    let src_rows = row_index.get(&arrow.src.0);
    let tgt_rows = row_index.get(&arrow.tgt.0);
    for (row_name, value) in table {
        let key_at = format!("/instance/tables/{arrow_name}");
        let src_id = check_table_src(arrow_name, arrow, row_name, src_rows, &key_at, spans)?;
        let Some(target) = value else {
            continue; // explicit `null` — the same dangling value as an absent cell
        };
        let Some(tgt_id) = tgt_rows.and_then(|m| m.get(target)) else {
            return Err(spans.syntax(
                &format!("{key_at}/{row_name}"),
                format!(
                    "table `{arrow_name}` maps `{row_name}` to `{target}`, which is not a row of `{}`",
                    arrow.tgt.0
                ),
            ));
        };
        values.get_mut(&Ident(arrow_name.to_string())).unwrap()[src_id.0] = Some(*tgt_id);
    }
    Ok(())
}

/// A table key must be a declared row of the arrow's source object.
fn check_table_src(
    arrow_name: &str,
    arrow: &Arrow,
    row_name: &str,
    src_rows: Option<&BTreeMap<String, RowId>>,
    key_at: &str,
    spans: &Spans,
) -> Result<RowId, ParseError> {
    let Some(src_map) = src_rows else {
        return Err(spans.syntax_key(
            key_at,
            row_name,
            format!(
                "table `{arrow_name}` keys row `{row_name}`, but `{}` has no declared rows",
                arrow.src.0
            ),
        ));
    };
    let Some(src_id) = src_map.get(row_name) else {
        return Err(spans.syntax_key(
            key_at,
            row_name,
            format!(
                "table `{arrow_name}` keys row `{row_name}`, which is not a row of `{}`",
                arrow.src.0
            ),
        ));
    };
    Ok(*src_id)
}
