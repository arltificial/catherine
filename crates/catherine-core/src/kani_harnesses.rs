//! Purpose: the Kani bounded-model-checking harnesses over the shipped
//! kernel (openspec change `add-kani-verification-lane`, beads
//! `catherine-xdi`) — panic/overflow-freedom over the post-parse
//! execution path (`TypedInstance::build`, path composition inside the
//! `check` walk, equation/key/coprod/pullback families) and witness
//! soundness on emitted findings (re-evaluation half in the `soundness`
//! submodule).
//!
//! Responsibilities: harnesses are PARTITIONED PER CHECK FAMILY, each
//! with its own minimal schema and stated bounds (design Decision 4:
//! bounds are per harness; one family per harness keeps the symbolic
//! heap footprint small enough for CBMC while every family stays
//! exhaustively checked within its bounds). Synthesis is nondeterministic
//! `kani::any()` scalars composed into domain values by deterministic
//! insertion loops (Decision 2: never the map-bearing domain types
//! themselves, never serde strings). Bounds are non-vacuous per family:
//! each harness's space admits a document exercising its family's
//! failure shapes; the provocation pass (task 3) demonstrates each
//! harness can fail.
//!
//! Rationale: the checker sets `cfg(kani)`; this module compiles only
//! under verification (the module wiring in lib.rs is the whole surface —
//! the workspace gains no Kani dependency outside this file). Kani's
//! UNDETERMINED and any counterexample follow the counterexample protocol
//! (design Decision 5): attribution recorded, obligations never weakened.

use crate::instance::TypedInstance;
use crate::model::{Ident, Instance, RowId};

mod soundness;

mod shapes;

use shapes::{any_cells, ARROWS};

// ---------------------------------------------------------------------------
// Shared synthesis (single space description, per-arrow entropy tiers).
//
// Cell entropy is tiered: wide (dangling/row0/row1) where a family needs
// the extra reach, narrow (dangling/row0) elsewhere — the symbolic space
// stays small enough for CBMC while every family's failure shapes stay
// reachable. `rows_of` is the per-object row bound.
// ---------------------------------------------------------------------------

/// The cell of `arrow` at source row `row`, if the presentation's shape
/// carries it. `None` = no such cell; `Some(None)` = dangling;
/// `Some(Some(v))` = a value (possibly a typing violation, judged by the
/// caller against the target space).
fn cell(instance: &Instance, arrow: &str, row: usize) -> Option<Option<RowId>> {
    instance
        .values
        .get(&Ident(arrow.to_string()))
        .and_then(|cells| cells.get(row).copied())
}

/// Nondeterministic instance over the shared arrow space: dense rows per
/// object, tiered cells per (arrow, source row).
fn any_instance() -> Instance {
    let mut instance = Instance::default();
    for (object, rows) in shapes::OBJECT_ROWS {
        instance
            .rows
            .insert(Ident(object.to_string()), (0..rows).map(RowId).collect());
    }
    for (name, _src, _tgt, rows) in ARROWS {
        let mut cells = Vec::new();
        for _ in 0..rows {
            cells.push(any_cells(name));
        }
        instance.values.insert(Ident(name.to_string()), cells);
    }
    instance
}

/// The nondeterministic `check` + diff-stable-order obligation over the
/// whole walk: all four families declared on one minimal presentation,
/// findings asserted totally ordered (repeated checks sort identically).
#[kani::proof]
fn check_sortedness_panic_freedom() {
    let schema = shapes::schema_all_families();
    let instance = any_instance();
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    for window in findings.windows(2) {
        assert!(window[0] <= window[1]);
    }
}

/// Panic/overflow-freedom of `TypedInstance::build` alone over the full
/// arrow space, including dangling and typing-violation branches (build
/// is infallible by contract: every value lands somewhere, never a hard
/// failure).
#[kani::proof]
fn build_panic_freedom() {
    let schema = shapes::schema_all_families();
    let instance = any_instance();
    let _typed = TypedInstance::build(&schema, &instance);
}

/// Panic/overflow-freedom of the equation family over its minimal space
/// (parallel pair `[e1] = [e2]`; both sides must resolve or the row
/// emits nothing — recorded by dangling, never crashed).
#[kani::proof]
fn check_equation_panic_freedom() {
    let schema = shapes::schema_equation();
    let instance = shapes::instance_for(&[("e1", "a", "x", 2), ("e2", "a", "x", 2)]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    for finding in &findings {
        assert!(matches!(
            finding.kind,
            crate::findings::FindingKind::Violation(_)
        ));
    }
}

/// Panic/overflow-freedom of the key family over its minimal space (one
/// two-arrow key over a two-row object; unresolved cells exclude their
/// row from the collision scan — never double-reported).
#[kani::proof]
fn check_key_panic_freedom() {
    let schema = shapes::schema_key();
    let instance = shapes::instance_for(&[("e1", "a", "x", 2), ("e2", "a", "x", 2)]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    for finding in &findings {
        assert!(matches!(
            finding.kind,
            crate::findings::FindingKind::Violation(_)
                | crate::findings::FindingKind::Dangling(_)
        ));
    }
}

/// Panic/overflow-freedom of the coproduct family over its minimal space
/// (two injections into a two-row target; coverage and disjointness
/// decided only on resolvable cells).
#[kani::proof]
fn check_coproduct_panic_freedom() {
    let schema = shapes::schema_coproduct();
    let instance = shapes::instance_for(&[("i_y", "y", "x", 1), ("i_z", "z", "x", 2)]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    for finding in &findings {
        assert!(matches!(
            finding.kind,
            crate::findings::FindingKind::Violation(_)
                | crate::findings::FindingKind::Dangling(_)
        ));
    }
}

/// Panic/overflow-freedom of the pullback family over its minimal space
/// (apex with two legs, cospan over a two-row target so commutes can
/// disagree; dangling legs suspend a row's verdicts, dangling cospan
/// cells shrink the decidable fiber).
#[kani::proof]
fn check_pullback_panic_freedom() {
    let schema = shapes::schema_pullback();
    let instance = shapes::instance_for(&[
        ("e1", "a", "x", 2),
        ("e2", "a", "x", 2),
        ("f1", "x", "z", 2),
        ("f2", "x", "z", 2),
    ]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    for finding in &findings {
        assert!(matches!(
            finding.kind,
            crate::findings::FindingKind::Violation(_)
                | crate::findings::FindingKind::Dangling(_)
        ));
    }
}

/// Witness soundness, equation family: every emitted violation's witness
/// re-evaluated independently (soundness submodule) disagrees exactly as
/// claimed.
#[kani::proof]
fn soundness_equation() {
    let schema = shapes::schema_equation();
    let instance = shapes::instance_for(&[("e1", "a", "x", 2), ("e2", "a", "x", 2)]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    kani::assume(!findings.is_empty());
    let index = (kani::any::<u32>() as usize) % findings.len();
    soundness::assert_finding_sound(&schema, &instance, &findings[index]);
}

/// Witness soundness, key family: every emitted collision's witness
/// re-evaluated independently finds equal, fully resolved tuples on
/// ascending rows.
#[kani::proof]
fn soundness_key() {
    let schema = shapes::schema_key();
    let instance = shapes::instance_for(&[("e1", "a", "x", 2), ("e2", "a", "x", 2)]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    kani::assume(!findings.is_empty());
    let index = (kani::any::<u32>() as usize) % findings.len();
    soundness::assert_finding_sound(&schema, &instance, &findings[index]);
}

/// Witness soundness, coproduct family: coverage witnesses name rows in
/// no injection image; disjointness witnesses name colliding injections
/// that really cover their row.
#[kani::proof]
fn soundness_coproduct() {
    let schema = shapes::schema_coproduct();
    let instance = shapes::instance_for(&[("i_y", "y", "x", 1), ("i_z", "z", "x", 2)]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    kani::assume(!findings.is_empty());
    let index = (kani::any::<u32>() as usize) % findings.len();
    soundness::assert_finding_sound(&schema, &instance, &findings[index]);
}

/// Witness soundness, pullback family: commutes witnesses re-derive both
/// compositions and the disagreement; covers witnesses re-derive the
/// uncovered fiber pair; unique witnesses re-derive the shared
/// projections.
#[kani::proof]
fn soundness_pullback() {
    let schema = shapes::schema_pullback();
    let instance = shapes::instance_for(&[
        ("e1", "a", "x", 2),
        ("e2", "a", "x", 2),
        ("f1", "x", "z", 2),
        ("f2", "x", "z", 2),
    ]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    kani::assume(!findings.is_empty());
    let index = (kani::any::<u32>() as usize) % findings.len();
    soundness::assert_finding_sound(&schema, &instance, &findings[index]);
}

/// Witness soundness, dangling family: every dangling witness names a
/// cell that really holds no row.
#[kani::proof]
fn soundness_dangling() {
    let schema = shapes::schema_equation();
    let instance = shapes::instance_for(&[("e1", "a", "x", 2), ("e2", "a", "x", 2)]);
    let typed = TypedInstance::build(&schema, &instance);
    let findings = crate::checks::check(&schema, &typed);
    kani::assume(!findings.is_empty());
    let index = (kani::any::<u32>() as usize) % findings.len();
    soundness::assert_finding_sound(&schema, &instance, &findings[index]);
}
