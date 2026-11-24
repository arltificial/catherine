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
/// arm returns `None` explicitly instead of `?`-chaining through slice
/// machinery. Same partition as the old `get`-chain walk; the shape is
/// the solver-shaped one (ledger `0006`: the compose-side wall's
/// recorded trigger).
pub(crate) fn engine_compose(
    typed: &EngineTyped,
    path: &Bounded<Step, ARROWS_CAP>,
    start: RowId,
) -> Option<RowId> {
    let mut current = start;
    let mut i = 0;
    while i < path.len() {
        let step = match path[i] {
            Some(Some(step)) => step,
            _ => return None,
        };
        if step >= typed.tables.len() {
            return None;
        }
        let cell = match typed.tables[step].get(current.0) {
            Some(Some(value)) => *value,
            _ => return None,
        };
        current = cell;
        i += 1;
    }
    Some(current)
}
