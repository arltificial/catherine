//! Purpose: the State pattern lens (beads `catherine-mx1.12`) — the
//! transition table and its declared (state, event) cells read as a
//! totality law: every executed transition equation must hold on every
//! cell an instance declares, and every declared cell must resolve.
//!
//! Responsibilities: one `'static` [`PatternMeta`] matching the
//! disposition row for `state`, and `run` reporting the kernel surfaces
//! the kata executed — [`catherine_core::checks::check`] for equation
//! drift, unresolved links, and unfilled cells, plus the recorded
//! one-step `parallel::underconstraint` questions where the model is
//! above the control floor and its transition table has a totality gap.
//! The checks channel carries both; the migration channel stays empty
//! (this law has no Δ surface).
//!
//! Rationale: the sealed entries (`ledger/katas/state-discovery/`) ran
//! `check` on every fixture and `underconstraint --max-path-length 1`
//! only where discovery pressure was recorded, so the lens mirrors that
//! selection rather than raising questions unconditionally. Above the
//! two-state/two-event floor the control self-negates, and the questions
//! belong only to a totality gap — a declared cell with no transition or
//! a declared state no transition enters; a dangling link or an equation
//! drift is already an executed finding, not a schema question.
//! Reachability itself stays unexpressible: the lens observes only
//! whether a state has an incoming transition, emits no finding about
//! reachability, and never fabricates the recorded null result. No new
//! kind, witness, or detection claim exists here; the one-step path cap
//! is the recorded command's, not a tunable.

use std::collections::BTreeSet;

use crate::bounds::ScopeCaps;
use crate::parallel;
use crate::pattern::{LawKind, LensFindings, LensInput, LensOutcome, PatternLens, PatternMeta};
use catherine_core::checks;
use catherine_core::instance::TypedInstance;
use catherine_core::model::Ident;

/// The disposition row's metadata, verbatim.
pub static STATE_META: PatternMeta = PatternMeta {
    id: "state",
    name: "State",
    abstraction: "state shape",
    law: LawKind::Totality,
    scope: "transition laws and cell totality; unreachable-state unexpressible; one null kill",
};

/// The state shape's declared-cell link and the transition target link.
const CELL_TRANSITION: &str = "cell_transition";
const TARGET: &str = "to";
/// The two vocabularies whose joint floor is the kata's control.
const STATE: &str = "State";
const EVENT: &str = "Event";

/// The State lens: transition equations and declared-cell totality over
/// the model, plus the recorded discovery questions where totality is
/// undecided.
pub struct Lens;

impl PatternLens for Lens {
    fn meta(&self) -> &'static PatternMeta {
        &STATE_META
    }

    fn run(&self, input: &LensInput<'_>, caps: &ScopeCaps) -> LensOutcome {
        let schema = &input.model.schema;
        let typed = TypedInstance::build(schema, &input.model.instance);
        let mut checks = checks::check(schema, &typed);
        if asks_discovery_questions(&typed) {
            // The recorded discovery arm ran at `--max-path-length 1`;
            // keep that bound while honouring every other cap.
            let recorded = ScopeCaps {
                max_path_length: 1,
                ..*caps
            };
            checks.extend(parallel::underconstraint(schema, &recorded));
        }
        checks.sort();
        LensOutcome::Findings(LensFindings {
            checks,
            migrations: Vec::new(),
        })
    }
}

/// Whether the recorded schema questions belong to this model: it must
/// be above the two-state/two-event control floor, and its transition
/// table must leave a declared cell unfilled or a declared state without
/// an incoming transition. Both are totality gaps, not detection claims.
fn asks_discovery_questions(typed: &TypedInstance) -> bool {
    above_control_floor(typed) && has_totality_gap(typed)
}

/// The kata's near-floor control self-negates; questions are scoped above
/// its two-state, two-event vocabulary.
fn above_control_floor(typed: &TypedInstance) -> bool {
    row_count(typed, STATE) > 2 || row_count(typed, EVENT) > 2
}

/// Whether the transition table is not total: a declared cell with no
/// transition, or a declared state no transition resolves into.
fn has_totality_gap(typed: &TypedInstance) -> bool {
    let unfilled_cell = typed
        .dangling()
        .iter()
        .any(|cell| cell.arrow.0 == CELL_TRANSITION);
    unfilled_cell || has_unentered_state(typed)
}

/// Whether some declared state is the target of no transition.
fn has_unentered_state(typed: &TypedInstance) -> bool {
    let Some(states) = typed.rows().get(&Ident(STATE.to_string())) else {
        return false;
    };
    let Some(targets) = typed.tables().get(&Ident(TARGET.to_string())) else {
        return false;
    };
    let entered: BTreeSet<_> = targets.iter().flatten().copied().collect();
    states.iter().any(|state| !entered.contains(state))
}

/// The declared row count for one object, zero when the object is absent.
fn row_count(typed: &TypedInstance, object: &str) -> usize {
    typed
        .rows()
        .get(&Ident(object.to_string()))
        .map_or(0, Vec::len)
}
