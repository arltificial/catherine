//! Purpose: the presentation row names the parser normalizes away
//! (openspec change `add-rules-projection`, kernel seam 1) — a side table
//! beside the parsed model, never inside it.
//!
//! Responsibilities: [`RowNames`] (object → declared row names indexed by
//! dense [`RowId`]) and [`row_names`], which builds it from the parser's
//! name index; the unit tests pin the dense-id ordering, `parse` equality,
//! and the empty-object entry.
//!
//! Rationale: `Instance` equality, the canonical bytes, and the model
//! hash must stay name-blind (the canonical writer synthesizes `_<i>`
//! labels), while a caller that has to *spell* a value — the
//! `rules --project` bridge — needs the names the parser validated. One
//! side table serves the second need without teaching the kernel's data
//! model a name, and this module split keeps `parse/mod.rs` inside the
//! repo's file-size gate.

use crate::model::{Ident, RowId};
use std::collections::BTreeMap;

/// The presentation names the parser normalizes away, indexed by dense
/// `RowId` per object: `RowNames[object][row.0]` is that row's declared
/// name. Deliberately *outside* [`Model`]: `Instance` equality, the
/// canonical bytes, and the model hash stay name-independent (the
/// canonical writer synthesizes dense `_<i>` labels), so a caller that
/// needs to spell a value — the `rules --project` bridge — reads this
/// side table instead of teaching the kernel's data model a name.
pub type RowNames = BTreeMap<Ident, Vec<Ident>>;

/// The presentation names, indexed by dense [`RowId`]: every declared
/// object gets an entry (an object with no rows gets an empty one), and
/// each name vector is ordered by row id, i.e. sorted-name order.
pub(super) fn row_names(
    objects: &BTreeMap<Ident, ()>,
    row_index: &BTreeMap<String, BTreeMap<String, RowId>>,
) -> RowNames {
    objects
        .keys()
        .map(|object| {
            let mut pairs: Vec<(&String, RowId)> = row_index
                .get(&object.0)
                .map(|names| names.iter().map(|(name, row)| (name, *row)).collect())
                .unwrap_or_default();
            pairs.sort_by_key(|(_, row)| row.0);
            let names = pairs
                .into_iter()
                .map(|(name, _)| Ident(name.clone()))
                .collect();
            (object.clone(), names)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse, parse_with_rows};

    /// Rows declared out of order: the name table must follow the dense
    /// ids the model carries (sorted-name order), not the file order.
    const DOC: &str = r#"{
        "format_version": "1.0",
        "schema": {"objects": ["Thing"], "arrows": [], "equations": []},
        "instance": {
            "rows": {"Thing": ["zeta", "alpha", "mu"]},
            "tables": {}
        }
    }"#;

    #[test]
    fn row_names_follow_the_dense_row_ids() {
        let (model, names) = parse_with_rows(DOC).expect("the document parses");
        let thing = Ident("Thing".to_string());
        assert_eq!(
            names[&thing],
            vec![
                Ident("alpha".to_string()),
                Ident("mu".to_string()),
                Ident("zeta".to_string()),
            ],
            "names indexed by dense RowId, i.e. sorted-name order"
        );
        assert_eq!(
            model.instance.rows[&thing],
            vec![RowId(0), RowId(1), RowId(2)],
            "the model keeps only the dense ids"
        );
    }

    /// The side table is additive: `parse` is exactly `parse_with_rows().0`.
    #[test]
    fn parse_returns_the_same_model_without_the_names() {
        let (with_rows, _) = parse_with_rows(DOC).expect("the document parses");
        assert_eq!(parse(DOC).expect("the document parses"), with_rows);
    }

    /// Every declared object has an entry, including one with no rows.
    #[test]
    fn row_names_cover_objects_without_rows() {
        let (_, names) = parse_with_rows(
            r#"{
                "format_version": "1.0",
                "schema": {"objects": ["Empty"], "arrows": [], "equations": []},
                "instance": {"rows": {}, "tables": {}}
            }"#,
        )
        .expect("the document parses");
        assert_eq!(names[&Ident("Empty".to_string())], Vec::<Ident>::new());
    }
}
