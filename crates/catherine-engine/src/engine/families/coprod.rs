//! The coproduct family of the executed checks — coverage and
//! disjointness verdicts over a declared coproduct's injection images,
//! over the arena.

use super::super::{
    Bounded, CompactFinding, CompactViolation, EngineSchema, EngineTyped, Nid, ARROWS_CAP, ROWS_CAP,
};
use super::FindingSink;
use crate::findings::EpistemicLabel;

pub(crate) fn coproduct_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
    out: &mut impl FindingSink,
) {
    for (index, coproduct) in schema.coproducts.iter().enumerate() {
        let Some(coproduct) = coproduct else { continue };
        let Some(rows) = typed.row_counts[coproduct.object] else {
            continue;
        };
        let label = crate::findings::label_for(coproduct.status);
        let (images, complete) = coproduct_images(typed, &coproduct.arrows);
        coproduct_row_findings(out, &images, complete, index, rows, label);
    }
    let _ = schema;
}

/// The coproduct's complete injection images, ascending distinct target
/// values each — the walk's `BTreeSet` images; `partial` is true when
/// any injection image has an unresolved cell. Arena-backed: each image
/// is a fixed-cap table of its arrow's resolved cells.
type CoproductImages = Bounded<(Nid, Bounded<u64, ROWS_CAP>), ARROWS_CAP>;

/// Arena membership in one image — the walk's `BTreeSet`/`Vec` `contains`
/// as a plain slot scan.
fn image_has(image: &Bounded<u64, ROWS_CAP>, row: u64) -> bool {
    let mut i = 0;
    while i < image.len() {
        if image[i] == Some(row) {
            return true;
        }
        i += 1;
    }
    false
}

/// The coproduct's complete injection images, resolved target values in
/// walk order — `partial` is true when any injection image has an
/// unresolved cell. No sort, no dedup: the walk consumes the images by
/// membership only (`row_covering`), the merged cross-family sort fixes
/// the final order, and the parsed flavor's `BTreeSet` semantics were
/// the only reason for the sorted-distinct mirror — charging it to the
/// lane's symex without changing a single emission (measured
/// 2026-11-24: the mirror alone kept soundness_coproduct over the
/// 300 s wall at unwind 8 and 9).
fn coproduct_images(typed: &EngineTyped, arrows: &[Nid]) -> (CoproductImages, bool) {
    let mut images = CoproductImages::new();
    let mut complete = true;
    let mut a = 0;
    while a < arrows.len() {
        let table = &typed.tables[arrows[a]];
        // Plain resolve loop — the `collect::<Option<Vec<_>>>()` form
        // pulled the iterator/Option monad machinery into the formula.
        let mut values = Bounded::<u64, ROWS_CAP>::new();
        let mut resolved = true;
        let mut idx = 0;
        while idx < table.len() {
            match table.get(idx).copied().flatten() {
                Some(value) => values.push(value.0 as u64),
                None => {
                    resolved = false;
                    break;
                }
            }
            idx += 1;
        }
        if resolved {
            images.push((arrows[a], values));
        } else {
            complete = false;
        }
        a += 1;
    }
    (images, complete)
}

/// Coverage and disjointness verdicts over one coproduct's rows: with a
/// complete image set, a row must be covered exactly once; with any
/// partial image, coverage is undecidable and only a collision of two or
/// more complete images is decidable.
fn coproduct_row_findings(
    out: &mut impl FindingSink,
    images: &CoproductImages,
    complete: bool,
    index: usize,
    rows: usize,
    label: EpistemicLabel,
) {
    let mut row = 0;
    while row < rows {
        let covering = row_covering(images, row);
        let emit = if complete {
            covering.len() != 1
        } else {
            covering.len() >= 2
        };
        if emit {
            out.emit((
                (0, 3, 0, index as u64, row as u64, 0, 0, 0),
                label,
                CompactFinding::Violation(CompactViolation::Coproduct {
                    index,
                    row,
                    injections: covering,
                }),
            ));
        }
        row += 1;
    }
}

/// The complete images covering one row — the walk's coverage scan.
fn row_covering(images: &CoproductImages, row: usize) -> Bounded<Nid, ARROWS_CAP> {
    let mut covering = Bounded::<Nid, ARROWS_CAP>::new();
    let mut i = 0;
    while i < images.len() {
        if let Some(Some((arrow, image))) = images.get(i) {
            if image_has(image, row as u64) {
                covering.push(*arrow);
            }
        }
        i += 1;
    }
    covering
}
