//! Shared test-side fixture loading: the lenient presentation loader the
//! parity gate and the check goldens use for fixtures whose shapes P0's
//! parser gates structurally (a wrong-object table value is parse-level
//! `Syntax`). The loader mirrors parse's normalization — dense per-object
//! row ids in sorted-name order, total per-arrow vectors — but preserves
//! every table value: a value naming a row of another object lands in its
//! home object's id space, which is exactly the out-of-target-space shape
//! [`TypedInstance::build`] records as a typing violation.

use catherine_core::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Path, RowId, Schema,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Load a `.ct.json` presentation leniently: schema + instance with names
/// interned per object in sorted order (parse's rule), values resolved
/// into their home object's id space. Fails only on loader-level bugs
/// (malformed fixture JSON, references to undeclared rows).
pub fn load_presentation(text: &str) -> Result<(Schema, Instance), String> {
    let doc: Value = serde_json::from_str(text).map_err(|e| format!("fixture is not JSON: {e}"))?;

    let mut objects = BTreeMap::new();
    for name in strings_at(&doc, "/schema/objects")? {
        objects.insert(Ident(name), ());
    }

    let mut arrows = BTreeMap::new();
    for arrow in array_at(&doc, "/schema/arrows")? {
        let name = Ident(str_at(&arrow, "/name")?);
        let src = Ident(str_at(&arrow, "/src")?);
        let tgt = Ident(str_at(&arrow, "/tgt")?);
        arrows.insert(name, Arrow { src, tgt });
    }

    // Equations in canonical order, exactly as the parser stores them:
    // sorted by (lhs, rhs, rationale, status).
    let mut equations = Vec::new();
    for eq in array_at(&doc, "/schema/equations")? {
        let lhs = Path(path_at(&eq, "/lhs")?);
        let rhs = Path(path_at(&eq, "/rhs")?);
        let rationale = eq
            .get("rationale")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let status = match eq.get("status").and_then(Value::as_str) {
            None | Some("accepted") => ConstraintStatus::Accepted,
            Some("proposed") => ConstraintStatus::Proposed,
            Some(other) => return Err(format!("unknown equation status `{other}`")),
        };
        equations.push(Equation {
            lhs,
            rhs,
            rationale,
            status,
        });
    }
    equations.sort_by(|a, b| {
        (&a.lhs, &a.rhs, &a.rationale, a.status.as_str()).cmp(&(
            &b.lhs,
            &b.rhs,
            &b.rationale,
            b.status.as_str(),
        ))
    });

    // Rows: dense ids in sorted-name order per object — parse's rule.
    let mut rows_out: BTreeMap<Ident, Vec<RowId>> = BTreeMap::new();
    let mut index: BTreeMap<String, BTreeMap<String, RowId>> = BTreeMap::new();
    for obj in objects.keys() {
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
        rows_out.insert(obj.clone(), (0..map.len()).map(RowId).collect());
        index.insert(obj.0.clone(), map);
    }

    // Tables: total vectors per arrow; every value resolved into its home
    // object's id space (target object first — the well-typed case).
    let mut values: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for (name, arrow) in &arrows {
        let n_src = index.get(&arrow.src.0).map(BTreeMap::len).ok_or_else(|| {
            format!(
                "arrow `{}`: src object `{}` undeclared",
                name.0, arrow.src.0
            )
        })?;
        let mut table = vec![None; n_src];
        let tgt_map = index.get(&arrow.tgt.0);
        let pointer = format!("/instance/tables/{}", name.0);
        if let Some(table_doc) = doc.pointer(&pointer) {
            let cells = table_doc
                .as_object()
                .ok_or_else(|| format!("`{pointer}` is not an object"))?;
            for (row_name, value) in cells {
                let src_id = index
                    .get(&arrow.src.0)
                    .and_then(|m| m.get(row_name))
                    .ok_or_else(|| {
                        format!(
                            "table `{}`: `{row_name}` is not a row of `{}`",
                            name.0, arrow.src.0
                        )
                    })?;
                let cell = match value {
                    Value::Null => None,
                    Value::String(target) => {
                        let resolved = tgt_map
                            .and_then(|m| m.get(target))
                            .or_else(|| index.values().filter_map(|m| m.get(target)).next())
                            .ok_or_else(|| {
                                format!(
                                    "table `{}`: value `{target}` names no declared row",
                                    name.0
                                )
                            })?;
                        Some(*resolved)
                    }
                    other => {
                        return Err(format!(
                            "table `{}`: value {other} is not a row name or null",
                            name.0
                        ))
                    }
                };
                table[src_id.0] = cell;
            }
        }
        values.insert(name.clone(), table);
    }

    Ok((
        Schema {
            objects,
            arrows,
            equations,
        },
        Instance {
            rows: rows_out,
            values,
        },
    ))
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
