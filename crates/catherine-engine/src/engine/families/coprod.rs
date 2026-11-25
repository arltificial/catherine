//! The coproduct family of the executed checks — coverage and
//! disjointness verdicts over a declared coproduct's injection images,
//! over the arena.

use super::super::{
    Bounded, CompactFinding, CompactViolation, EngineSchema, EngineTyped, Nid, ARROWS_CAP, ROWS_CAP,
};
use super::FindingSink;
use crate::findings::EpistemicLabel;
use crate::model::RowId;

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
        coproduct_row_findings(
            out,
            &images,
            CoproductRowScope {
                complete,
                index,
                rows,
                label,
            },
        );
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
    image.iter().any(|slot| *slot == Some(row))
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
    for &arrow in arrows {
        if let Some(image) = arrow_image(&typed.tables[arrow]) {
            images.push((arrow, image));
        }
    }
    // An arrow with any unresolved cell contributes no image, so a
    // complete image set is exactly one image per declared injection.
    let complete = images.len() == arrows.len();
    (images, complete)
}

/// One injection's resolved image, ascending target values each — `None`
/// when any of the arrow's cells is unresolved (the walk's partial
/// image). Plain resolve loop over concrete slots: the
/// `collect::<Option<Vec<_>>>()` form pulled the iterator/Option monad
/// machinery into the formula.
fn arrow_image(table: &Bounded<RowId, ROWS_CAP>) -> Option<Bounded<u64, ROWS_CAP>> {
    let mut values = Bounded::<u64, ROWS_CAP>::new();
    for idx in 0..table.len() {
        let value = table.get(idx).copied().flatten()?;
        values.push(value.0 as u64);
    }
    Some(values)
}

/// One coproduct's emission scope: the declaration index, its row space,
/// the completeness verdict, and the constraint's label.
struct CoproductRowScope {
    complete: bool,
    index: usize,
    rows: usize,
    label: EpistemicLabel,
}

/// Coverage and disjointness verdicts over one coproduct's rows: with a
/// complete image set, a row must be covered exactly once; with any
/// partial image, coverage is undecidable and only a collision of two or
/// more complete images is decidable.
fn coproduct_row_findings(
    out: &mut impl FindingSink,
    images: &CoproductImages,
    scope: CoproductRowScope,
) {
    let CoproductRowScope {
        complete,
        index,
        rows,
        label,
    } = scope;
    for row in 0..rows {
        let covering = row_covering(images, row);
        if coproduct_row_verdict(complete, covering.len()) {
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
    }
}

/// One row's coproduct verdict: with a complete image set, a row must be
/// covered exactly once; with any partial image, coverage is
/// undecidable and only a collision of two or more complete images is
/// decidable.
fn coproduct_row_verdict(complete: bool, covered: usize) -> bool {
    if complete {
        covered != 1
    } else {
        covered >= 2
    }
}

/// The complete images covering one row — the walk's coverage scan.
fn row_covering(images: &CoproductImages, row: usize) -> Bounded<Nid, ARROWS_CAP> {
    let mut covering = Bounded::<Nid, ARROWS_CAP>::new();
    for (arrow, image) in images.iter().flatten() {
        if image_has(image, row as u64) {
            covering.push(*arrow);
        }
    }
    covering
}
