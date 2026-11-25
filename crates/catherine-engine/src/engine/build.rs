//! The build classification walk (openspec change
//! `add-kani-verification-lane`): every parsed arrow value lands in a
//! morphism table entry, a dangling record, or a typing violation over
//! dense `usize` ids. Split out of `engine/mod.rs` to keep both files
//! inside the file-size gate; the public surface stays on
//! `crate::engine` via re-export.

use super::{Bounded, EngineInstance, EngineSchema, EngineTyped, Nid, REPORT_CAP, ROWS_CAP};
use crate::model::RowId;

/// The exact report bound: every taken cell lands in at most one
/// report, and no cell outside a table can. The lane's array caps are
/// proof obligations discharged against this bound; production is
/// parsed-data-sized and carries no cap.
#[cfg(kani)]
fn report_bound(schema: &EngineSchema, instance: &EngineInstance) -> usize {
    schema
        .arrows
        .src
        .iter()
        .enumerate()
        .map(|(nid, src)| {
            let n_src = instance.row_counts[*src].unwrap_or(0);
            instance.cells[nid]
                .as_ref()
                .map_or(0, |cells| cells.len().min(n_src))
        })
        .sum()
}

/// The target object's row count — `0` when the arrow is undeclared or
/// the target carries no row space.
fn target_rows(schema: &EngineSchema, instance: &EngineInstance, nid: Nid) -> usize {
    schema
        .arrows
        .tgt
        .get(nid)
        .and_then(|tgt| instance.row_counts[*tgt])
        .unwrap_or(0)
}

/// The build's report accumulators — the classification output the
/// build walk fills cell by cell.
#[derive(Default)]
struct BuildReports {
    typing_violations: Bounded<(Nid, usize, RowId), REPORT_CAP>,
    dangling: Bounded<(Nid, usize), REPORT_CAP>,
}

/// One arrow's resolved morphism table; misresolved and dangling cells
/// land in `reports` (the walk's per-cell classification).
fn arrow_table(
    schema: &EngineSchema,
    instance: &EngineInstance,
    nid: Nid,
    reports: &mut BuildReports,
) -> Bounded<RowId, ROWS_CAP> {
    let source = schema.arrows.src[nid];
    let n_src = instance.row_counts[source].unwrap_or(0);
    let n_tgt = target_rows(schema, instance, nid);
    let mut table = Bounded::<RowId, ROWS_CAP>::with_len(n_src);
    if let Some(cells) = &instance.cells[nid] {
        for (idx, &cell) in cells.iter().enumerate().take(n_src) {
            match cell {
                None => reports.dangling.push((nid, idx)),
                Some(value) if value.0 < n_tgt => table[idx] = Some(value),
                Some(value) => reports.typing_violations.push((nid, idx, value)),
            }
        }
    }
    table
}

/// The build classification over ids — `TypedInstance::build`'s walk:
/// every parsed arrow value lands in a morphism table entry, a dangling
/// record, or a typing violation; cells beyond the source object's row
/// space are ignored; arrows outside the presentation are ignored.
pub fn engine_build(schema: &EngineSchema, instance: &EngineInstance) -> EngineTyped {
    // Under the lane, the array caps are proof obligations discharged
    // against the computed exact bound; production carries no cap.
    #[cfg(kani)]
    assert!(report_bound(schema, instance) <= REPORT_CAP);
    let mut reports = BuildReports::default();
    let tables: Vec<Bounded<RowId, ROWS_CAP>> = schema
        .arrows
        .src
        .iter()
        .enumerate()
        .map(|(nid, _)| arrow_table(schema, instance, nid, &mut reports))
        .collect();
    EngineTyped {
        row_counts: instance.row_counts.clone(),
        tables,
        typing_violations: reports.typing_violations,
        dangling: reports.dangling,
    }
}
