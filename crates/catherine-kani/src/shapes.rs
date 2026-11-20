//! Purpose: the bounded presentation shapes for the partitioned Kani
//! harnesses (openspec change `add-kani-verification-lane`, beads
//! `catherine-6g2`) — one minimal schema per check family plus the
//! all-families integration shape, built directly as interned engine
//! structures so no `BTreeMap`/`String` op is charged to the solver
//! (beads catherine-amh: container modeling is the wall).
//!
//! Responsibilities: [`OBJECT_ROWS`] / [`ARROWS`] / [`any_cells`] — the
//! single space description (objects, arrows with source-row counts,
//! per-arrow entropy tier); the per-family schema builders
//! (`schema_equation`, `schema_key`, `schema_coproduct`,
//! `schema_pullback`, `schema_all_families`) over
//! [`EngineSchema::literal`]; [`instance_for`] — the deterministic
//! instance builder composing `kani::any()` scalar cells into dense id
//! tables (design Decision 2: scalar synthesis by insertion loops, never
//! the map-bearing domain types, never serde strings).
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
//! cannot drift apart. Canonical id order comes from
//! [`EngineSchema::literal`]'s sort, and [`instance_for`] addresses
//! cells by id, so a shape list out of sorted order cannot corrupt the
//! tables.

use catherine_engine::engine::{ident_key, push_bounded, EngineInstance, EngineSchema};
use catherine_engine::model::{ConstraintStatus, RowId};

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

/// One cell at the arrow's entropy tier: wide (dangling/row0/row1) on
/// `e1`, `f1`, `g1` — the arrows whose families need the extra reach —
/// narrow (dangling/row0) elsewhere. The tier lookup is by byte key:
/// pure int, no String comparison reaches the solver.
pub fn any_cells(arrow: u64) -> Option<RowId> {
    let wide = arrow == ident_key("e1") || arrow == ident_key("f1") || arrow == ident_key("g1");
    if wide {
        match kani::any::<u8>() % 3 {
            0 => None,
            1 => Some(RowId(0)),
            _ => Some(RowId(1)),
        }
    } else {
        match kani::any::<u8>() % 2 {
            0 => None,
            _ => Some(RowId(0)),
        }
    }
}

/// The equation family: parallel pair `[e1] = [e2]` over `a`.
pub fn schema_equation() -> EngineSchema {
    EngineSchema::lane(
        &["a", "x"],
        &[("e1", "a", "x"), ("e2", "a", "x")],
        &[(&["e1"], &["e2"], ConstraintStatus::Accepted)],
        &[],
        &[],
        &[],
    )
}

/// The key family: one two-arrow key over `a`'s two rows.
pub fn schema_key() -> EngineSchema {
    EngineSchema::lane(
        &["a", "x"],
        &[("e1", "a", "x"), ("e2", "a", "x")],
        &[],
        &[("a", &["e1", "e2"], ConstraintStatus::Accepted)],
        &[],
        &[],
    )
}

/// The coproduct family: two injections into the two-row `x`.
pub fn schema_coproduct() -> EngineSchema {
    EngineSchema::lane(
        &["x", "y", "z"],
        &[("i_y", "y", "x"), ("i_z", "z", "x")],
        &[],
        &[],
        &[("x", &["i_y", "i_z"], ConstraintStatus::Accepted)],
        &[],
    )
}

/// The pullback family: apex `a` over legs `e1`, `e2` and cospan
/// `f1`, `f2` into the two-row `z`.
pub fn schema_pullback() -> EngineSchema {
    EngineSchema::lane(
        &["a", "x", "z"],
        &[
            ("e1", "a", "x"),
            ("e2", "a", "x"),
            ("f1", "x", "z"),
            ("f2", "x", "z"),
        ],
        &[],
        &[],
        &[],
        &[(
            "a",
            &["e1", "e2"],
            &["f1", "f2"],
            ConstraintStatus::Accepted,
        )],
    )
}

/// All four families on one minimal presentation — the integration
/// shape the diff-stable sortedness obligation runs over.
pub fn schema_all_families() -> EngineSchema {
    let arrows: Vec<(&str, &str, &str)> = ARROWS.iter().map(|(n, s, t, _)| (*n, *s, *t)).collect();
    EngineSchema::lane(
        &["a", "x", "y", "z"],
        &arrows,
        &[(&["e1"], &["e2"], ConstraintStatus::Accepted)],
        &[("a", &["e1", "e2"], ConstraintStatus::Accepted)],
        &[("x", &["i_y", "i_z"], ConstraintStatus::Accepted)],
        &[(
            "a",
            &["e1", "e2"],
            &["f1", "f2"],
            ConstraintStatus::Accepted,
        )],
    )
}

/// The deterministic instance over `schema`'s shapes: dense rows per
/// object and tiered partial cells per source row, resolved by byte key
/// (pure int lookups, no String ops in the solver path). Plain loops
/// only — collect closures over symbolic cells churn the solver
/// (catherine-6g2, measured).
pub fn instance_for(schema: &EngineSchema) -> EngineInstance {
    EngineInstance {
        row_counts: row_counts_for(schema),
        cells: cells_for(schema),
    }
}

/// Dense row counts per object, in the schema's canonical order — each
/// key resolved against the shared space description by byte key.
fn row_counts_for(schema: &EngineSchema) -> Vec<Option<usize>> {
    let mut row_counts: Vec<Option<usize>> = Vec::with_capacity(schema.object_keys.len());
    let mut oi = 0;
    while oi < schema.object_keys.len() {
        let want = schema.object_keys[oi];
        let mut found = None;
        let mut si = 0;
        while si < OBJECT_ROWS.len() {
            if ident_key(OBJECT_ROWS[si].0) == want {
                found = Some(OBJECT_ROWS[si].1);
                break;
            }
            si += 1;
        }
        push_bounded(&mut row_counts, found);
        oi += 1;
    }
    row_counts
}

/// One tiered partial cell vector per declared arrow, parallel to the
/// source rows; arrows outside the shared space carry no cells.
fn cells_for(schema: &EngineSchema) -> Vec<Option<Vec<Option<RowId>>>> {
    let mut cells: Vec<Option<Vec<Option<RowId>>>> = Vec::with_capacity(schema.arrow_keys.len());
    let mut ai = 0;
    while ai < schema.arrow_keys.len() {
        let want = schema.arrow_keys[ai];
        let mut found: Option<(usize, u64)> = None;
        let mut si = 0;
        while si < ARROWS.len() {
            let key = ident_key(ARROWS[si].0);
            if key == want {
                found = Some((ARROWS[si].3, key));
                break;
            }
            si += 1;
        }
        match found {
            Some((rows, key)) => {
                let mut table: Vec<Option<RowId>> = Vec::with_capacity(rows);
                let mut i = 0;
                while i < rows {
                    push_bounded(&mut table, any_cells(key));
                    i += 1;
                }
                push_bounded(&mut cells, Some(table));
            }
            None => push_bounded(&mut cells, None),
        }
        ai += 1;
    }
    cells
}
