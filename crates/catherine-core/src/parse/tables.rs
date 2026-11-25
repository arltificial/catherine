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

/// What the instance tables are checked against: the declared arrows, the
/// declared rows per object, and the span index.
pub(super) struct TableCtx<'a> {
    pub(super) arrows: &'a BTreeMap<Ident, Arrow>,
    pub(super) row_index: &'a BTreeMap<String, BTreeMap<String, RowId>>,
    pub(super) spans: &'a Spans,
}

/// One table's cell checks: the declaring arrow's name and definition, its
/// source/target row maps, the value vectors being filled, and the span
/// index.
struct TableCellCheck<'a> {
    name: &'a str,
    arrow: &'a Arrow,
    src_rows: Option<&'a BTreeMap<String, RowId>>,
    tgt_rows: Option<&'a BTreeMap<String, RowId>>,
    values: &'a mut BTreeMap<Ident, Vec<Option<RowId>>>,
    spans: &'a Spans,
}

/// Instance tables: repeated keys, declared arrows, declared src rows,
/// and values that are declared target rows or `null`. Every declared
/// arrow carries a total vector over its source object's rows; an
/// unspecified cell parses as `None` (dangling).
pub(super) fn check_tables(
    instance: &RawInstance,
    ctx: TableCtx,
) -> Result<BTreeMap<Ident, Vec<Option<RowId>>>, ParseError> {
    check_table_dup_keys(ctx.spans)?;
    check_table_arrows(instance, &ctx)?;
    let mut values = init_tables(&ctx);
    for (arrow_name, table) in &instance.tables {
        let arrow = &ctx.arrows[&Ident(arrow_name.clone())];
        let mut cells = TableCellCheck {
            name: arrow_name,
            arrow,
            src_rows: ctx.row_index.get(&arrow.src.0),
            tgt_rows: ctx.row_index.get(&arrow.tgt.0),
            values: &mut values,
            spans: ctx.spans,
        };
        check_table_cells(&mut cells, table)?;
    }
    Ok(values)
}

/// Every table names a declared arrow.
fn check_table_arrows(instance: &RawInstance, ctx: &TableCtx) -> Result<(), ParseError> {
    for arrow_name in instance.tables.keys() {
        if !ctx.arrows.contains_key(&Ident(arrow_name.clone())) {
            return Err(ctx.spans.syntax_key(
                "/instance/tables",
                arrow_name,
                format!("instance table names undeclared arrow `{arrow_name}`"),
            ));
        }
    }
    Ok(())
}

/// An empty total table per declared arrow, sized by its source object's
/// row count.
fn init_tables(ctx: &TableCtx) -> BTreeMap<Ident, Vec<Option<RowId>>> {
    let mut values: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for (name, arrow) in ctx.arrows.iter() {
        let n = ctx.row_index.get(&arrow.src.0).map_or(0, BTreeMap::len);
        values.insert(name.clone(), vec![None; n]);
    }
    values
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
    check: &mut TableCellCheck,
    table: &BTreeMap<String, Option<String>>,
) -> Result<(), ParseError> {
    for (row_name, value) in table {
        let key_at = format!("/instance/tables/{}", check.name);
        let src_id = check_table_src(check, row_name, &key_at)?;
        let Some(target) = value else {
            continue; // explicit `null` — the same dangling value as an absent cell
        };
        let Some(tgt_id) = check.tgt_rows.and_then(|m| m.get(target)) else {
            return Err(check.spans.syntax(
                &format!("{key_at}/{row_name}"),
                format!(
                    "table `{}` maps `{row_name}` to `{target}`, which is not a row of `{}`",
                    check.name, check.arrow.tgt.0
                ),
            ));
        };
        check
            .values
            .get_mut(&Ident(check.name.to_string()))
            .unwrap()[src_id.0] = Some(*tgt_id);
    }
    Ok(())
}

/// A table key must be a declared row of the arrow's source object.
fn check_table_src(
    check: &TableCellCheck,
    row_name: &str,
    key_at: &str,
) -> Result<RowId, ParseError> {
    let Some(src_map) = check.src_rows else {
        return Err(check.spans.syntax_key(
            key_at,
            row_name,
            format!(
                "table `{}` keys row `{row_name}`, but `{}` has no declared rows",
                check.name, check.arrow.src.0
            ),
        ));
    };
    let Some(src_id) = src_map.get(row_name) else {
        return Err(check.spans.syntax_key(
            key_at,
            row_name,
            format!(
                "table `{}` keys row `{row_name}`, which is not a row of `{}`",
                check.name, check.arrow.src.0
            ),
        ));
    };
    Ok(*src_id)
}
