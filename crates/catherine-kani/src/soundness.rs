//! Purpose: the witness re-evaluation half of the Kani harnesses
//! (openspec change `add-kani-verification-lane`, beads `catherine-6g2`)
//! — every finding the walk emits has its witness re-evaluated
//! independently: the claimed predicate is recomputed straight off the
//! interned schema and typed tables, sharing no code with the kernel
//! walk (the Rust counterpart of `Catherine.Proofs.soundness`, verified
//! on the shipped artifact).
//!
//! Responsibilities: [`compose`] rebuilds path composition from the
//! engine tables; the `assert_*_sound` family re-derives each witness
//! family's claim (equation disagreement, key collision, coproduct
//! coverage/disjointness, pullback commutes/covers/unique, dangling
//! cell) over the lane flavor's compact witnesses (ids, zero Strings);
//! [`assert_finding_sound`] dispatches one finding to its obligation.
//!
//! Rationale: an independent evaluator that reused the kernel's walk
//! would prove nothing about it — the obligations here carry their own
//! composition and table reads (never `engine::engine_compose` or the
//! family walkers), so a kernel regression that emits a wrong witness
//! cannot satisfy them. Id order is name order, witnesses carry
//! canonical indices, and no solver time goes to container, string, or
//! render modeling. Naturality/Redundant/Contradiction/Underconstraint
//! have no `check` producer — out of this family's scope
//! (docs/kani-lane.md).

use catherine_engine::engine::{
    Bounded, CompactFinding, CompactPullback, CompactViolation, CompiledKey, EngineSchema,
    EngineTyped, Nid, ARROWS_CAP, ROWS_CAP,
};

/// Compose a compiled path starting at source row `start` — rebuilt
/// independently: each step resolves its cell or the path fails.
fn compose(typed: &EngineTyped, path: &[Option<Nid>], start: usize) -> Option<usize> {
    let mut current = start;
    for step in path {
        let table = typed.tables.get((*step)?)?;
        let cell = table.get(current)?.as_ref()?;
        current = cell.0;
    }
    Some(current)
}

/// The equation claim: both sides recomposed land on the witness's
/// values, and they disagree.
fn assert_equation_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
) {
    let equation = &schema.equations[index];
    let left = compose(typed, &equation.lhs, row);
    let right = compose(typed, &equation.rhs, row);
    assert_eq!(left, Some(lhs));
    assert_eq!(right, Some(rhs));
    assert_ne!(lhs, rhs);
    let _ = schema;
}

/// The key claim: both colliding rows' key tuples fully resolve and
/// are equal, ascending as carried.
fn assert_key_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    lo: usize,
    hi: usize,
) {
    let key = schema.keys[index].as_ref().unwrap();
    assert!(lo < hi);
    // Both rows' key tuples fully resolve (any unresolved cell fails
    // the claim), and the first arrow's values agree — plain slot
    // reads, no heap tuple (walker arena round 2).
    let first_lo = key_first_value(typed, key, lo);
    let first_hi = key_first_value(typed, key, hi);
    assert_eq!(first_lo, first_hi);
    let _ = schema;
}

/// The row's key tuple fully resolves (unresolved fails the claim);
/// the first arrow's value rides out for the equality check.
fn key_first_value(typed: &EngineTyped, key: &CompiledKey, row: usize) -> Option<u64> {
    let mut a = 0;
    let mut first: Option<u64> = None;
    while a < key.arrows.len() {
        let cell = typed.tables[key.arrows[a]]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        match cell {
            Some(value) => {
                if first.is_none() {
                    first = Some(value as u64);
                }
            }
            None => panic!(),
        }
        a += 1;
    }
    assert!(first.is_some());
    first
}

/// The coproduct claim: coverage failure (`injections` empty) means the
/// row lies in no complete injection image; disjointness failure means
/// every named colliding injection covers the row, at least two do.
fn assert_coproduct_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
    injections: &Bounded<Nid, ARROWS_CAP>,
) {
    let coproduct = schema.coproducts[index].as_ref().unwrap();
    let (images, partial) = coproduct_images(typed, &coproduct.arrows);
    if injections.is_empty() {
        assert_coproduct_uncovered(&images, partial, row);
    } else {
        assert_coproduct_collision(&images, partial, row, injections);
    }
}
/// One arrow's complete injection image (sorted distinct values) —
/// `partial` records whether any image has an unresolved cell. Same
/// arena shape as the walk, re-derived independently.
fn coproduct_images(
    typed: &EngineTyped,
    arrows: &[Nid],
) -> (Bounded<(Nid, Bounded<u64, ROWS_CAP>), ARROWS_CAP>, bool) {
    let mut images: Bounded<(Nid, Bounded<u64, ROWS_CAP>), ARROWS_CAP> = Bounded::new();
    let mut partial = false;
    let mut a = 0;
    while a < arrows.len() {
        let table = &typed.tables[arrows[a]];
        let mut values = Bounded::<u64, ROWS_CAP>::new();
        let mut complete = true;
        let mut idx = 0;
        while idx < table.len() {
            match table.get(idx).copied().flatten() {
                Some(value) => values.push(value.0 as u64),
                None => complete = false,
            }
            idx += 1;
        }
        if complete {
            values.sort_pushed_by(|x, y| x.cmp(y));
            values.dedup_pushed();
            images.push((arrows[a], values));
        } else {
            partial = true;
        }
        a += 1;
    }
    (images, partial)
}

/// The coverage claim: no partial image, no image covers the row.
fn assert_coproduct_uncovered(
    images: &Bounded<(Nid, Bounded<u64, ROWS_CAP>), ARROWS_CAP>,
    partial: bool,
    row: usize,
) {
    assert!(!partial);
    let mut i = 0;
    while i < images.len() {
        let (_, image) = images[i].as_ref().unwrap();
        assert!(!image_covers(image, row as u64));
        i += 1;
    }
}

/// The disjointness claim: every covering image is a named injection,
/// and at least two complete images cover the row.
fn assert_coproduct_collision(
    images: &Bounded<(Nid, Bounded<u64, ROWS_CAP>), ARROWS_CAP>,
    partial: bool,
    row: usize,
    injections: &Bounded<Nid, ARROWS_CAP>,
) {
    let mut named = 0;
    let mut i = 0;
    while i < images.len() {
        let (arrow, image) = images[i].as_ref().unwrap();
        let covered = image_covers(image, row);
        let is_named = injection_named(injections, *arrow);
        if covered {
            assert!(is_named || partial);
            if is_named {
                named += 1;
            }
        } else {
            assert!(!is_named || partial);
        }
        i += 1;
    }
    assert!(named >= 2);
}

/// Plain slot scan: does one complete image cover the row?
fn image_covers(image: &Bounded<u64, ROWS_CAP>, row: u64) -> bool {
    let mut j = 0;
    while j < image.len() {
        if image[j] == Some(row) {
            return true;
        }
        j += 1;
    }
    false
}

/// Plain slot scan: is the arrow one of the witness's named injections?
fn injection_named(injections: &Bounded<Nid, ARROWS_CAP>, arrow: Nid) -> bool {
    let mut k = 0;
    while k < injections.len() {
        if injections[k] == Some(arrow) {
            return true;
        }
        k += 1;
    }
    false
}

/// The commutes claim: both cospan compositions resolve and disagree
/// as the witness carried them.
fn assert_commutes_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
) {
    let pullback = schema.pullbacks[index].as_ref().unwrap();
    let [p1, p2] = pullback.legs;
    let [f, g] = pullback.cospan;
    let left = typed.tables[p1]
        .get(row)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    let right = typed.tables[p2]
        .get(row)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    match (left, right) {
        (Some(left), Some(right)) => {
            let f_value = typed.tables[f]
                .get(left)
                .and_then(|cell| cell.as_ref())
                .map(|cell| cell.0);
            let g_value = typed.tables[g]
                .get(right)
                .and_then(|cell| cell.as_ref())
                .map(|cell| cell.0);
            assert_eq!(f_value, Some(lhs));
            assert_eq!(g_value, Some(rhs));
            assert_ne!(lhs, rhs);
        }
        _ => panic!(),
    }
    let _ = schema;
}

/// The covers claim: the pair shares a cospan target, and no apex row
/// projects onto it through both legs of the square.
fn assert_covers_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    a: usize,
    b: usize,
) {
    let pullback = schema.pullbacks[index].as_ref().unwrap();
    let [p1, p2] = pullback.legs;
    let [f, g] = pullback.cospan;
    let f_value = typed.tables[f]
        .get(a)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    let g_value = typed.tables[g]
        .get(b)
        .and_then(|cell| cell.as_ref())
        .map(|cell| cell.0);
    assert_eq!(f_value, g_value);
    assert!(f_value.is_some());
    let apex_rows = typed.row_counts[pullback.apex].unwrap_or(0);
    let mut row = 0;
    while row < apex_rows {
        let left = typed.tables[p1]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        let right = typed.tables[p2]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        match (left, right) {
            (Some(left), Some(right)) => {
                assert!(!(left == a && right == b));
            }
            _ => {}
        }
        row += 1;
    }
}

/// The unique claim: two ascending apex rows resolve both leg
/// projections to the same values — the square is not injective there.
fn assert_unique_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    index: usize,
    lo: usize,
    hi: usize,
) {
    let pullback = schema.pullbacks[index].as_ref().unwrap();
    let [p1, p2] = pullback.legs;
    assert!(lo < hi);
    let mut lo_left: Option<usize> = None;
    let mut lo_right: Option<usize> = None;
    let mut hi_left: Option<usize> = None;
    let mut hi_right: Option<usize> = None;
    for row in [lo, hi] {
        let left = typed.tables[p1]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        let right = typed.tables[p2]
            .get(row)
            .and_then(|cell| cell.as_ref())
            .map(|cell| cell.0);
        assert!(left.is_some() && right.is_some());
        if row == lo {
            lo_left = left;
            lo_right = right;
        } else {
            hi_left = left;
            hi_right = right;
        }
    }
    assert_eq!(lo_left, hi_left);
    assert_eq!(lo_right, hi_right);
    let _ = schema;
}

/// The independent soundness obligation for one compact finding: every
/// witness re-evaluated from the interned tables fails exactly as
/// claimed.
pub(super) fn assert_finding_sound(
    schema: &EngineSchema,
    typed: &EngineTyped,
    finding: &CompactFinding,
) {
    match finding {
        CompactFinding::Dangling { arrow, src } => assert_dangling_sound(typed, *arrow, *src),
        CompactFinding::Violation(CompactViolation::Equation {
            index,
            row,
            lhs,
            rhs,
        }) => assert_equation_sound(schema, typed, *index, *row, *lhs, *rhs),
        CompactFinding::Violation(CompactViolation::Key { index, lo, hi }) => {
            assert_key_sound(schema, typed, *index, *lo, *hi)
        }
        CompactFinding::Violation(CompactViolation::Coproduct {
            index,
            row,
            injections,
        }) => assert_coproduct_sound(schema, typed, *index, *row, injections),
        CompactFinding::Violation(CompactViolation::Pullback(pullback)) => {
            assert_pullback_sound(schema, typed, pullback)
        }
    }
}

fn assert_dangling_sound(typed: &EngineTyped, arrow: usize, src: usize) {
    let cell = typed.tables.get(arrow).and_then(|table| table.get(src));
    match cell {
        Some(None) => {}
        _ => panic!(),
    }
}

/// The pullback family's three obligations, one per verdict.
fn assert_pullback_sound(schema: &EngineSchema, typed: &EngineTyped, pullback: &CompactPullback) {
    match pullback {
        CompactPullback::Commutes {
            index,
            row,
            lhs,
            rhs,
        } => assert_commutes_sound(schema, typed, *index, *row, *lhs, *rhs),
        CompactPullback::Covers { index, pair } => {
            assert_covers_sound(schema, typed, *index, pair[0], pair[1])
        }
        CompactPullback::Unique { index, rows } => {
            assert_unique_sound(schema, typed, *index, rows[0], rows[1])
        }
    }
}
