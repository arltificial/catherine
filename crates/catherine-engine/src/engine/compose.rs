//! Purpose: path composition over the interned typed tables — the
//! parsed compose walk's solver-shaped resolution (openspec change
//! `add-kani-verification-lane`, beads `catherine-6g2`). Split out of
//! `mod.rs` under the complexity caps when the concrete-bound walk
//! landed (ledger `0007`).

use super::{Bounded, EngineTyped, Step, ARROWS_CAP};
use crate::model::RowId;

/// Compose a compiled path over the typed tables — the parsed compose
/// walk over ids: an unresolved cell (dangling, typing-violated, or a
/// step no arrow defines) makes the path unresolvable, never a hard
/// failure.
///
/// The resolution walk is concrete-bound and flat-guarded: slots are
/// read by array index against the path's concrete `len` (bounded
/// table — see [`super::CompiledEquation`]), and every read's failure
/// arm yields `None` explicitly instead of `?`-chaining through slice
/// machinery. Same partition as the old `get`-chain walk; the walk
/// itself is one `try_fold` over the step indices — the accumulator is
/// the composed row, and the first unresolved read ends the walk with
/// `None` (the old early `return`s).
pub(crate) fn engine_compose(
    typed: &EngineTyped,
    path: &Bounded<Step, ARROWS_CAP>,
    start: RowId,
) -> Option<RowId> {
    (0..path.len()).try_fold(start, |current, i| {
        let step = match path[i] {
            Some(Some(step)) => step,
            _ => return None,
        };
        if step >= typed.tables.len() {
            return None;
        }
        match typed.tables[step].get(current.0) {
            Some(Some(value)) => Some(*value),
            _ => None,
        }
    })
}
