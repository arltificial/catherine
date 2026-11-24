//! Purpose: the interned execution core of the kernel (openspec change
//! `add-kani-verification-lane`, beads `catherine-6g2`) — schema,
//! instance, build classification, path composition, and every executed
//! check family over dense `usize` ids instead of `Ident`-keyed
//! `BTreeMap`s, so the Kani lane can prove the kernel's post-parse
//! execution path within budget.
//!
//! Two flavors share one walker set:
//!
//! - **Parsed flavor** ([`EngineSchema::compile`]): owned canonical
//!   names, findings rendered to the public [`Finding`] taxonomy inline.
//!   Concrete data only (parse output) — never run under Kani, so its
//!   map and String ops cost nothing there. This is what
//!   [`crate::checks::check`] and [`crate::instance::TypedInstance::build`]
//!   serve.
//! - **Lane flavor** ([`EngineSchema::lane`]): id keys only, zero
//!   `String` values anywhere; findings stay compact (ints) and sorted.
//!   Measured (probe crates `target/probe-intern`, `target/probe-enginelite`,
//!   Kani 0.68 / CBMC 6.11, unwind 16): `BTreeMap` ops over symbolic
//!   content are fatal at any key type; a `&str` in a sort comparator
//!   churns memcmp to the unwind cap; and — the last wall — `String`
//!   allocation/drop/swap in the harness path is fatal even for a dozen
//!   concrete names, while the identical shape string-free proves in
//!   seconds. Hence the lane flavor never materializes a name.
//!
//! Canonical order: ids are assigned in sorted-name order, so id order is
//! name order everywhere; the int [`SortKey`] is exactly the comparison
//! prefix of `Finding`'s derived order (kind, violation variant,
//! per-variant discriminator, witness int fields, label), making the
//! lane's compact sort and the parsed flavor's `Finding::cmp` sort the
//! same total order. The parsed render and the contract tests pin that
//! equivalence.

use crate::findings::EpistemicLabel;
use crate::model::{ConstraintStatus, Ident, RowId};
use std::cmp::Ordering;

/// A dense interned id. Object ids and arrow ids are two separate id
/// spaces, each assigned in sorted-name order, so id order is name order.
pub type Nid = usize;

mod compile;
mod compose;
mod families;
mod lane;
mod store;

#[cfg(kani)]
pub use compile::ident_key;
pub(crate) use families::{
    coproduct_findings, dangling_findings, equation_findings, key_findings, pullback_findings,
    violation_flags,
};
pub use families::{
    engine_coproduct_findings, engine_dangling_findings, engine_equation_findings,
    engine_key_findings, engine_pullback_findings,
};
pub use store::{Bounded, ARROWS_CAP, REPORT_CAP, ROWS_CAP};

pub(crate) use compose::engine_compose;

/// A finding's pure-int sort key — exactly the comparison prefix
/// `Finding`'s derived order walks: kind, then violation variant, then a
/// per-variant discriminator, then the witness's int fields in field
/// order, then the label.
/// A path step resolved to its arrow id; `None` = a step no declared
/// arrow defines (compose can never resolve past it).
pub type Step = Option<Nid>;

pub type SortKey = (u8, u8, u8, u64, u64, u64, u64, u8);

/// The lane flavor's finding: the taxonomy's shapes with int payloads
/// only (ids stand in for names; id order is name order). Rendered to
/// the public [`Finding`] by [`render_compact`] on the parsed flavor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompactFinding {
    Violation(CompactViolation),
    Dangling { arrow: Nid, src: usize },
}

/// The violation shapes over ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompactViolation {
    Equation {
        index: usize,
        row: usize,
        lhs: usize,
        rhs: usize,
    },
    Key {
        index: usize,
        lo: usize,
        hi: usize,
    },
    Coproduct {
        index: usize,
        row: usize,
        injections: Bounded<Nid, ARROWS_CAP>,
    },
    Pullback(CompactPullback),
}

/// The pullback verdict arms over ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompactPullback {
    Commutes {
        index: usize,
        row: usize,
        lhs: usize,
        rhs: usize,
    },
    Covers {
        index: usize,
        pair: [usize; 2],
    },
    Unique {
        index: usize,
        rows: [usize; 2],
    },
}

/// The interned presentation. Names are owned only on the parsed flavor
/// (built by [`EngineSchema::compile`]); the lane flavor ([`EngineSchema::lane`])
/// leaves them empty and carries nothing but ids.
pub struct EngineSchema {
    /// Objects in canonical order; empty on the lane flavor.
    pub object_names: Vec<Ident>,
    /// Arrows in canonical order; empty on the lane flavor.
    pub arrow_names: Vec<Ident>,
    /// Objects' byte keys in canonical order — lane flavor only (the
    /// parsed flavor's names carry the identity); read by the harness
    /// instance builder.
    #[cfg(kani)]
    pub object_keys: Vec<u64>,
    /// Arrows' byte keys in canonical order — lane flavor only.
    #[cfg(kani)]
    pub arrow_keys: Vec<u64>,
    /// Per arrow: the source object's id.
    pub(crate) arrow_src: Vec<Nid>,
    /// Per arrow: the target object's id.
    pub(crate) arrow_tgt: Vec<Nid>,
    /// Equations in declaration order; `domain: None` marks the walk's
    /// skip cases (empty lhs, undeclared lead arrow).
    pub equations: Vec<CompiledEquation>,
    /// Keys in declaration order; `None` = the walk would skip the key.
    pub keys: Vec<Option<CompiledKey>>,
    /// Coproducts in declaration order; `None` = the walk would skip.
    pub coproducts: Vec<Option<CompiledCoproduct>>,
    /// Pullbacks in declaration order; `None` = the walk would skip.
    pub pullbacks: Vec<Option<CompiledPullback>>,
}

/// A compiled path equation.
pub struct CompiledEquation {
    /// The lhs/rhs paths as bounded tables — concrete-bounded slot
    /// arrays under the lane (`cfg(kani)`; the cap is a proof
    /// obligation discharged by the declared shapes' path lengths),
    /// `Vec`-backed and arbitrary-sized in production. The compose walk
    /// iterates slots against a concrete `len` and reads each step by
    /// array index — the pointer-validity charges of walking a
    /// heap-`Vec` path (selected at a symbolic equation index) were the
    /// measured compose-side wall (ledger `0006`; beads `catherine-6g2`).
    pub lhs: Bounded<Step, ARROWS_CAP>,
    pub rhs: Bounded<Step, ARROWS_CAP>,
    /// The lhs lead arrow's source object, when the equation is
    /// applicable.
    pub domain: Option<Nid>,
    pub status: ConstraintStatus,
}

/// A compiled declarative key.
pub struct CompiledKey {
    pub object: Nid,
    pub arrows: Vec<Nid>,
    pub status: ConstraintStatus,
}

/// A compiled coproduct declaration.
pub struct CompiledCoproduct {
    pub object: Nid,
    pub arrows: Vec<Nid>,
    pub status: ConstraintStatus,
}

/// A compiled pullback declaration.
pub struct CompiledPullback {
    pub apex: Nid,
    pub legs: [Nid; 2],
    pub cospan: [Nid; 2],
    pub status: ConstraintStatus,
}

/// The interned instance: per-object row counts (dense id spaces) and one
/// partial cell vector per declared arrow, parallel to its source rows.
pub struct EngineInstance {
    /// `row_counts[object]` = the object's dense row space size; `None` =
    /// the parsed instance carried no row space for the object.
    pub row_counts: Vec<Option<usize>>,
    /// `cells[arrow]` = the parsed value vector, parallel to source rows.
    /// Array-backed under `cfg(kani)` (see [`store`]); a plain owned
    /// vector on the parsed flavor.
    pub cells: Vec<Option<Bounded<RowId, ROWS_CAP>>>,
}

/// The interned typed instance: one partial morphism table per arrow plus
/// the build reports, all by id.
pub struct EngineTyped {
    /// Per-object dense row space sizes, parallel to
    /// [`EngineSchema::object_names`].
    pub row_counts: Vec<Option<usize>>,
    /// `tables[arrow]` = an unmapped table of `row_count(src)` rows,
    /// filled where the value resolved inside the target space.
    /// Array-backed under `cfg(kani)` (see [`store`]).
    pub tables: Vec<Bounded<RowId, ROWS_CAP>>,
    /// Typing violations, in build order (arrow id order, then cell):
    /// (arrow, source cell index, offending value). Capacity-asserted
    /// under `cfg(kani)`.
    pub typing_violations: Bounded<(Nid, usize, RowId), REPORT_CAP>,
    /// Dangling cells, in build order (arrow id order, then cell).
    /// Capacity-asserted under `cfg(kani)`.
    pub dangling: Bounded<(Nid, usize), REPORT_CAP>,
}

/// Capacity-asserted append — the walkers' and build's report vectors
/// never touch Kani's `Vec::push` alloc model (measured: the alloc model
/// explores growth branches under symbolic branches and dominates the
/// solver; the asserted raw write is free). The capacity is an exact
/// bound at every call site, so the assert is proof obligation, never
/// a reachable failure.
pub fn push_bounded<T>(v: &mut Vec<T>, x: T) {
    assert!(v.len() < v.capacity());
    let len = v.len();
    // SAFETY: the assert proves `len < capacity`, so the write lands in
    // the buffer and `set_len` grows the initialized prefix by one.
    unsafe {
        v.as_mut_ptr().add(len).write(x);
        v.set_len(len + 1);
    }
}

/// The exact report bound: every taken cell lands in at most one
/// report, and no cell outside a table can. The lane's array caps are
/// proof obligations discharged against this bound; production is
/// parsed-data-sized and carries no cap.
#[cfg(kani)]
fn report_bound(schema: &EngineSchema, instance: &EngineInstance) -> usize {
    let mut bound = 0usize;
    for (nid, src) in schema.arrow_src.iter().enumerate() {
        let n_src = instance.row_counts[*src].unwrap_or(0);
        if let Some(cells) = &instance.cells[nid] {
            bound += cells.len().min(n_src);
        }
    }
    bound
}

/// The build classification over ids — `TypedInstance::build`'s walk:
/// every parsed arrow value lands in a morphism table entry, a dangling
/// record, or a typing violation; cells beyond the source object's row
/// space are ignored; arrows outside the presentation are ignored.
pub fn engine_build(schema: &EngineSchema, instance: &EngineInstance) -> EngineTyped {
    let mut tables: Vec<Bounded<RowId, ROWS_CAP>> = Vec::with_capacity(schema.arrow_src.len());
    // Under the lane, the array caps are proof obligations discharged
    // against the computed exact bound (the shapes declare it well
    // under `REPORT_CAP`). Production is parsed-data-sized and carries
    // no cap.
    #[cfg(kani)]
    assert!(report_bound(schema, instance) <= REPORT_CAP);
    let mut typing_violations = Bounded::new();
    let mut dangling = Bounded::new();
    for (nid, src) in schema.arrow_src.iter().enumerate() {
        let n_src = instance.row_counts[*src].unwrap_or(0);
        let n_tgt = schema
            .arrow_tgt
            .get(nid)
            .and_then(|tgt| instance.row_counts[*tgt])
            .unwrap_or(0);
        let mut table = Bounded::<RowId, ROWS_CAP>::with_len(n_src);
        if let Some(cells) = &instance.cells[nid] {
            for (idx, &cell) in cells.iter().enumerate().take(n_src) {
                match cell {
                    None => dangling.push((nid, idx)),
                    Some(value) if value.0 < n_tgt => table[idx] = Some(value),
                    Some(value) => typing_violations.push((nid, idx, value)),
                }
            }
        }
        tables.push(table);
    }
    EngineTyped {
        row_counts: instance.row_counts.clone(),
        tables,
        typing_violations,
        dangling,
    }
}

/// Bounded insertion sort by comparator — the solver-shaped
/// replacement for `sort_unstable_by_key`/`sort_unstable` (beads catherine-6g2, measured: slice-sort
/// internals — `smallsort::insert_tail`, `swap_nonoverlapping_chunks`
/// — unwind to the cap under symbolic length and dominate every
/// harness; a plain insertion loop with single-element moves proves in
/// seconds). Sort domains are concretely bounded (report bounds, row
/// counts) and every family's keys are distinct within its domain
/// (kind + variant + per-variant discriminator), so the emitted order
/// equals the pdqsort order the parsed walk produced. `pub` because the
/// witness-soundness harness crate re-derives sorts independently.
pub fn insertion_sort_by<T>(v: &mut [T], cmp: impl Fn(&T, &T) -> Ordering) {
    let len = v.len();
    let mut i = 1;
    while i < len {
        // Hole technique: the element at `i` is read out once, later
        // elements shift by single-element reads/writes, and the held
        // element is written back at the final hole.
        let mut hole = i;
        // SAFETY: `hole < len` always; the element is read exactly once
        // here and written back exactly once below.
        let tmp: T = unsafe { std::ptr::read(v.as_ptr().add(hole)) };
        while hole > 0 && cmp(unsafe { &*v.as_ptr().add(hole - 1) }, &tmp) == Ordering::Greater {
            unsafe {
                let element = std::ptr::read(v.as_ptr().add(hole - 1));
                std::ptr::write(v.as_mut_ptr().add(hole), element);
            }
            hole -= 1;
        }
        unsafe { std::ptr::write(v.as_mut_ptr().add(hole), tmp) };
        i += 1;
    }
}

/// Drop duplicate adjacent equal values from a sorted vector — the
/// solver-shaped `Vec::dedup` (its tail memmove unrolls the same way).
pub fn dedup_sorted<T: Clone + PartialEq>(v: &mut Vec<T>) {
    let len = v.len();
    if len <= 1 {
        return;
    }
    let mut read_index = 1;
    let mut write_index = 1;
    while read_index < len {
        if v[read_index] != v[write_index - 1] {
            v[write_index] = v[read_index].clone();
            write_index += 1;
        }
        read_index += 1;
    }
    v.truncate(write_index);
}

/// Execute every declared constraint over the interned tables, compact:
/// findings sorted under the taxonomy's total order (the int-key sort is
/// exactly `Finding::cmp`), names never materialized. The parsed flavor
/// renders with [`render_compact`]; the lane harnesses assert the
/// compact shapes directly.
pub fn engine_families(
    schema: &EngineSchema,
    typed: &EngineTyped,
) -> Vec<(SortKey, EpistemicLabel, CompactFinding)> {
    // An upper bound on emitted findings — every family's worst case is
    // a function of the row spaces and declaration counts (concrete
    // inputs), so the walk never reallocates and never branches into the
    // allocator.
    let max_rows: usize = typed
        .row_counts
        .iter()
        .map(|rows| rows.unwrap_or(0))
        .max()
        .unwrap_or(0);
    let (eqs, keys, cops, pbs) = (
        schema.equations.len(),
        schema.keys.len(),
        schema.coproducts.len(),
        schema.pullbacks.len(),
    );
    let mut out: Vec<(SortKey, EpistemicLabel, CompactFinding)> = Vec::with_capacity(
        schema.arrow_src.len() * max_rows
            + max_rows * max_rows * (eqs + keys + cops + 3 * pbs)
            + max_rows * (eqs + cops + pbs)
            + 1,
    );
    let violated = violation_flags(typed);
    dangling_findings(schema, typed, &violated, &mut out);
    equation_findings(schema, typed, &mut out);
    key_findings(schema, typed, &mut out);
    coproduct_findings(schema, typed, &mut out);
    pullback_findings(schema, typed, &mut out);
    insertion_sort_by(&mut out, |a, b| a.0.cmp(&b.0));
    out
}
