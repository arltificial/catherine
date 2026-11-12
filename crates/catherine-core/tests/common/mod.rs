//! Shared test-side fixture loading: the lenient presentation loader the
//! parity gate and the check goldens use for fixtures whose shapes P0's
//! parser gates structurally (a wrong-object table value is parse-level
//! `Syntax`). The loader mirrors parse's normalization — dense per-object
//! row ids in sorted-name order, total per-arrow vectors — but preserves
//! every table value: a value naming a row of another object lands in its
//! home object's id space, which is exactly the out-of-target-space shape
//! [`TypedInstance::build`] records as a typing violation.

// Each test binary compiles this module and uses a different subset of it.
#![allow(dead_code)]

pub mod fixture;
pub mod rule_lens;

use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Path, RowId, Schema,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A resolved per-object row-name → row-id index.
type RowIndex = BTreeMap<String, BTreeMap<String, RowId>>;

/// Load a `.ct.json` presentation leniently: schema + instance with names
/// interned per object in sorted order (parse's rule), values resolved
/// into their home object's id space. Fails only on loader-level bugs
/// (malformed fixture JSON, references to undeclared rows).
pub fn load_presentation(text: &str) -> Result<(Schema, Instance), String> {
    let doc: Value = serde_json::from_str(text).map_err(|e| format!("fixture is not JSON: {e}"))?;
    let schema = schema_of(&doc)?;
    let (rows_out, index) = rows_and_index(&doc, &schema.objects)?;
    let values = values_of(&doc, &schema.arrows, &index)?;
    Ok((
        schema,
        Instance {
            rows: rows_out,
            values,
        },
    ))
}

/// The document's schema: objects and arrows interned in sorted order
/// (parse's rule), equations in the parser's canonical order.
fn schema_of(doc: &Value) -> Result<Schema, String> {
    let mut objects = BTreeMap::new();
    for name in strings_at(doc, "/schema/objects")? {
        objects.insert(Ident(name), ());
    }

    let mut arrows = BTreeMap::new();
    for arrow in array_at(doc, "/schema/arrows")? {
        let name = Ident(str_at(&arrow, "/name")?);
        let src = Ident(str_at(&arrow, "/src")?);
        let tgt = Ident(str_at(&arrow, "/tgt")?);
        arrows.insert(name, Arrow { src, tgt });
    }

    Ok(Schema {
        objects,
        arrows,
        equations: equations_of(doc)?,
    })
}

/// The equations in canonical order, exactly as the parser stores them:
/// sorted by (lhs, rhs, rationale, status).
fn equations_of(doc: &Value) -> Result<Vec<Equation>, String> {
    let mut equations = Vec::new();
    for eq in array_at(doc, "/schema/equations")? {
        equations.push(equation_of(&eq)?);
    }
    equations.sort_by(|a, b| {
        (&a.lhs, &a.rhs, &a.rationale, a.status.as_str()).cmp(&(
            &b.lhs,
            &b.rhs,
            &b.rationale,
            b.status.as_str(),
        ))
    });
    Ok(equations)
}

/// One document equation, with its rationale defaulting to empty.
fn equation_of(eq: &Value) -> Result<Equation, String> {
    let rationale = eq
        .get("rationale")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    Ok(Equation {
        lhs: Path(path_at(eq, "/lhs")?),
        rhs: Path(path_at(eq, "/rhs")?),
        rationale,
        status: equation_status(eq)?,
    })
}

/// A drawn equation's epistemic status string, defaulted to accepted.
fn equation_status(eq: &Value) -> Result<ConstraintStatus, String> {
    match eq.get("status").and_then(Value::as_str) {
        None | Some("accepted") => Ok(ConstraintStatus::Accepted),
        Some("proposed") => Ok(ConstraintStatus::Proposed),
        Some(other) => Err(format!("unknown equation status `{other}`")),
    }
}

/// Rows: dense ids in sorted-name order per object — parse's rule — plus
/// the row-name → row-id index the table resolution needs.
fn rows_and_index(
    doc: &Value,
    objects: &BTreeMap<Ident, ()>,
) -> Result<(BTreeMap<Ident, Vec<RowId>>, RowIndex), String> {
    let mut rows_out: BTreeMap<Ident, Vec<RowId>> = BTreeMap::new();
    let mut index: RowIndex = BTreeMap::new();
    for obj in objects.keys() {
        let (row_ids, map) = object_rows(doc, obj)?;
        rows_out.insert(obj.clone(), row_ids);
        index.insert(obj.0.clone(), map);
    }
    Ok((rows_out, index))
}

/// One object's dense row ids and its row-name → row-id map.
fn object_rows(doc: &Value, obj: &Ident) -> Result<(Vec<RowId>, BTreeMap<String, RowId>), String> {
    let pointer = format!("/instance/rows/{}", obj.0);
    let names: BTreeSet<String> = match doc.pointer(&pointer) {
        Some(node) => strings_node(node, &pointer)?.into_iter().collect(),
        None => BTreeSet::new(),
    };
    let map: BTreeMap<String, RowId> = names
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), RowId(i)))
        .collect();
    Ok(((0..map.len()).map(RowId).collect(), map))
}

/// Tables: total vectors per arrow; every value resolved into its home
/// object's id space (target object first — the well-typed case).
fn values_of(
    doc: &Value,
    arrows: &BTreeMap<Ident, Arrow>,
    index: &RowIndex,
) -> Result<BTreeMap<Ident, Vec<Option<RowId>>>, String> {
    let mut values: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for (name, arrow) in arrows {
        values.insert(name.clone(), table_of(doc, name, arrow, index)?);
    }
    Ok(values)
}

/// One arrow's table: a total vector over the source row space.
fn table_of(
    doc: &Value,
    name: &Ident,
    arrow: &Arrow,
    index: &RowIndex,
) -> Result<Vec<Option<RowId>>, String> {
    let n_src = index.get(&arrow.src.0).map(BTreeMap::len).ok_or_else(|| {
        format!(
            "arrow `{}`: src object `{}` undeclared",
            name.0, arrow.src.0
        )
    })?;
    let mut table = vec![None; n_src];
    for (row_name, value) in table_entries(doc, name)? {
        let src_id = src_row_id(name, arrow, index, &row_name)?;
        table[src_id.0] = resolved_cell(name, arrow, &value, index)?;
    }
    Ok(table)
}

/// The document table's `(row name, cell value)` entries, empty when the
/// document declares no table for the arrow.
fn table_entries(doc: &Value, name: &Ident) -> Result<Vec<(String, Value)>, String> {
    let pointer = format!("/instance/tables/{}", name.0);
    match doc.pointer(&pointer) {
        Some(table_doc) => {
            let cells = table_doc
                .as_object()
                .ok_or_else(|| format!("`{pointer}` is not an object"))?;
            Ok(cells.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        }
        None => Ok(Vec::new()),
    }
}

/// The source row id a table key names.
fn src_row_id(
    name: &Ident,
    arrow: &Arrow,
    index: &RowIndex,
    row_name: &str,
) -> Result<RowId, String> {
    index
        .get(&arrow.src.0)
        .and_then(|m| m.get(row_name))
        .copied()
        .ok_or_else(|| {
            format!(
                "table `{}`: `{row_name}` is not a row of `{}`",
                name.0, arrow.src.0
            )
        })
}

/// One table cell: `null` is the dangling value; a row name resolves
/// into its home object's id space.
fn resolved_cell(
    name: &Ident,
    arrow: &Arrow,
    value: &Value,
    index: &RowIndex,
) -> Result<Option<RowId>, String> {
    match value {
        Value::Null => Ok(None),
        Value::String(target) => Ok(Some(resolved_target(name, arrow, target, index)?)),
        other => Err(format!(
            "table `{}`: value {other} is not a row name or null",
            name.0
        )),
    }
}

/// Resolve a row name: the arrow's target object first — the well-typed
/// case — then any declared object's id space.
fn resolved_target(
    name: &Ident,
    arrow: &Arrow,
    target: &str,
    index: &RowIndex,
) -> Result<RowId, String> {
    let tgt_map = index.get(&arrow.tgt.0);
    tgt_map
        .and_then(|m| m.get(target))
        .or_else(|| index.values().filter_map(|m| m.get(target)).next())
        .copied()
        .ok_or_else(|| format!("table `{}`: value `{target}` names no declared row", name.0))
}

fn array_at(doc: &Value, pointer: &str) -> Result<Vec<Value>, String> {
    let node = doc
        .pointer(pointer)
        .ok_or_else(|| format!("missing `{pointer}`"))?;
    node.as_array()
        .ok_or_else(|| format!("`{pointer}` is not an array"))
        .cloned()
}

fn strings_at(doc: &Value, pointer: &str) -> Result<Vec<String>, String> {
    let node = doc
        .pointer(pointer)
        .ok_or_else(|| format!("missing `{pointer}`"))?;
    strings_node(node, pointer)
}

fn strings_node(node: &Value, pointer: &str) -> Result<Vec<String>, String> {
    node.as_array()
        .ok_or_else(|| format!("`{pointer}` is not an array"))?
        .iter()
        .map(|v| v.as_str().map(str::to_string))
        .collect::<Option<_>>()
        .ok_or_else(|| format!("`{pointer}` has a non-string element"))
}

fn path_at(doc: &Value, pointer: &str) -> Result<Vec<Ident>, String> {
    Ok(strings_at(doc, pointer)?.into_iter().map(Ident).collect())
}

fn str_at(doc: &Value, pointer: &str) -> Result<String, String> {
    doc.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("missing string `{pointer}`"))
}
