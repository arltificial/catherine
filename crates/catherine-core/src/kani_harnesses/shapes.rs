//! Purpose: the bounded presentation shapes for the partitioned Kani
//! harnesses (openspec change `add-kani-verification-lane`) — one
//! minimal schema per check family plus the all-families integration
//! shape, so each harness states its own bounds and keeps the symbolic
//! heap footprint at that family's size.
//!
//! Responsibilities: [`OBJECT_ROWS`] / [`ARROWS`] / [`any_cells`] — the
//! single space description (objects, arrows with source-row counts,
//! per-arrow entropy tier); [`rows_of`]; the per-family schema builders
//! (`schema_equation`, `schema_key`, `schema_coproduct`,
//! `schema_pullback`) and [`schema_all_families`]; [`instance_for`] —
//! the deterministic instance builder composing `kani::any()` scalar
//! cells into the domain tables (design Decision 2: never the
//! map-bearing types themselves, never serde strings).
//!
//! Rationale: bounds are per harness and non-vacuous (spec requirement):
//! each family's minimal space admits its failure shapes — equation
//! disagreement across two x rows (`e1` wide, `e2` narrow), key
//! collisions on equal tuples, coproduct coverage and disjointness over
//! two-row targets, pullback commutes/covers/unique against a two-row
//! `z` (a one-row cospan target could never disagree), the dangling
//! branch (`None` cells), and the build-time typing violation (`g1`
//! wide into the one-row `y`). The shared space description is the
//! single source the instance builder walks, so schema and instance
//! cannot drift apart.

use crate::model::{Arrow, ConstraintStatus, Coproduct, Equation, Ident, Instance, Key, Path, Pullback, RowId, Schema};

/// Per-object row bounds of the shared space.
pub const OBJECT_ROWS: [(&str, usize); 4] = [("a", 2), ("x", 2), ("y", 1), ("z", 2)];

/// The shared arrow space: (name, src, tgt, source row count). `g1` is
/// the typing-violation carrier (wide cell into the one-row `y`); it
/// belongs to no check family.
pub const ARROWS: [(&str, &str, &str, usize); 7] = [
    ("e1", "a", "x", 2),
    ("e2", "a", "x", 2),
    ("i_y", "y", "x", 1),
    ("i_z", "z", "x", 2),
    ("f1", "x", "z", 2),
    ("f2", "x", "z", 2),
    ("g1", "y", "y", 1),
];

/// The per-object row bound (`rows_of`).
pub fn rows_of(object: &str) -> usize {
    OBJECT_ROWS
        .iter()
        .find(|(name, _)| *name == object)
        .map_or(0, |&(_, rows)| rows)
}

/// One cell at the arrow's entropy tier: wide (dangling/row0/row1) on
/// `e1`, `f1`, `g1` — the arrows whose families need the extra reach —
/// narrow (dangling/row0) elsewhere.
pub fn any_cells(arrow: &str) -> Option<RowId> {
    match arrow {
        "e1" | "f1" | "g1" => match kani::any::<u8>() % 3 {
            0 => None,
            1 => Some(RowId(0)),
            _ => Some(RowId(1)),
        },
        _ => match kani::any::<u8>() % 2 {
            0 => None,
            _ => Some(RowId(0)),
        },
    }
}

/// The deterministic instance over `arrows`: dense rows per object,
/// tiered partial cells per (arrow, source row).
pub fn instance_for(arrows: &[(&str, &str, &str, usize)]) -> Instance {
    let mut instance = Instance::default();
    for (object, rows) in OBJECT_ROWS {
        instance
            .rows
            .insert(Ident(object.to_string()), (0..rows).map(RowId).collect());
    }
    for (name, _src, _tgt, rows) in arrows {
        let mut cells = Vec::new();
        for _ in 0..*rows {
            cells.push(any_cells(name));
        }
        instance.values.insert(Ident(name.to_string()), cells);
    }
    instance
}

/// The instance over the whole shared arrow space.
pub fn instance_all() -> Instance {
    instance_for(&ARROWS)
}

fn base_schema(objects: &[&str], arrows: &[(&str, &str, &str, usize)]) -> Schema {
    let mut schema = Schema::default();
    for object in objects {
        schema.objects.insert(Ident(object.to_string()), ());
    }
    for (name, src, tgt, _rows) in arrows {
        schema.arrows.insert(
            Ident(name.to_string()),
            Arrow {
                src: Ident(src.to_string()),
                tgt: Ident(tgt.to_string()),
            },
        );
    }
    schema
}

/// The equation family: parallel pair `[e1] = [e2]` over `a`.
pub fn schema_equation() -> Schema {
    let arrows = [("e1", "a", "x", 2), ("e2", "a", "x", 2)];
    let mut schema = base_schema(&["a", "x"], &arrows);
    schema.equations.push(Equation {
        lhs: Path(vec![Ident("e1".to_string())]),
        rhs: Path(vec![Ident("e2".to_string())]),
        rationale: String::new(),
        status: ConstraintStatus::Accepted,
    });
    schema
}

/// The key family: one two-arrow key over `a`'s two rows.
pub fn schema_key() -> Schema {
    let arrows = [("e1", "a", "x", 2), ("e2", "a", "x", 2)];
    let mut schema = base_schema(&["a", "x"], &arrows);
    schema.keys.push(Key {
        object: Ident("a".to_string()),
        arrows: vec![Ident("e1".to_string()), Ident("e2".to_string())],
        status: ConstraintStatus::Accepted,
    });
    schema
}

/// The coproduct family: two injections into the two-row `x`.
pub fn schema_coproduct() -> Schema {
    let arrows = [("i_y", "y", "x", 1), ("i_z", "z", "x", 2)];
    let mut schema = base_schema(&["x", "y", "z"], &arrows);
    schema.coproducts.push(Coproduct {
        object: Ident("x".to_string()),
        arrows: vec![Ident("i_y".to_string()), Ident("i_z".to_string())],
        status: ConstraintStatus::Accepted,
    });
    schema
}

/// The pullback family: apex `a` over legs `e1`, `e2` and cospan
/// `f1`, `f2` into the two-row `z`.
pub fn schema_pullback() -> Schema {
    let arrows = [
        ("e1", "a", "x", 2),
        ("e2", "a", "x", 2),
        ("f1", "x", "z", 2),
        ("f2", "x", "z", 2),
    ];
    let mut schema = base_schema(&["a", "x", "z"], &arrows);
    schema.pullbacks.push(Pullback {
        apex: Ident("a".to_string()),
        legs: vec![Ident("e1".to_string()), Ident("e2".to_string())],
        cospan: vec![Ident("f1".to_string()), Ident("f2".to_string())],
        status: ConstraintStatus::Accepted,
    });
    schema
}

/// All four families on one minimal presentation — the integration
/// shape the diff-stable sortedness obligation runs over.
pub fn schema_all_families() -> Schema {
    let mut schema = base_schema(&["a", "x", "y", "z"], &ARROWS);
    schema.equations.push(Equation {
        lhs: Path(vec![Ident("e1".to_string())]),
        rhs: Path(vec![Ident("e2".to_string())]),
        rationale: String::new(),
        status: ConstraintStatus::Accepted,
    });
    schema.keys.push(Key {
        object: Ident("a".to_string()),
        arrows: vec![Ident("e1".to_string()), Ident("e2".to_string())],
        status: ConstraintStatus::Accepted,
    });
    schema.coproducts.push(Coproduct {
        object: Ident("x".to_string()),
        arrows: vec![Ident("i_y".to_string()), Ident("i_z".to_string())],
        status: ConstraintStatus::Accepted,
    });
    schema.pullbacks.push(Pullback {
        apex: Ident("a".to_string()),
        legs: vec![Ident("e1".to_string()), Ident("e2".to_string())],
        cospan: vec![Ident("f1".to_string()), Ident("f2".to_string())],
        status: ConstraintStatus::Accepted,
    });
    schema
}
