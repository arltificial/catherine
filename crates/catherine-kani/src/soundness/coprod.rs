//! Purpose: the coproduct family's witness obligations, split out of
//! `soundness.rs` under the complexity caps (beads `catherine-6g2`,
//! ledger `0007`) — coverage and disjointness re-derived independently
//! from the interned tables, sharing no code with the kernel walk.

use catherine_engine::engine::{Bounded, EngineSchema, EngineTyped, Nid, ARROWS_CAP, ROWS_CAP};

/// The coproduct claim: coverage failure (`injections` empty) means the
/// row lies in no complete injection image; disjointness failure means
/// every named colliding injection covers the row, at least two do.
pub(super) fn assert_coproduct_sound(
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
        let covered = image_covers(image, row as u64);
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
