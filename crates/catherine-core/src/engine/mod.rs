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

use crate::findings::{EpistemicLabel, Finding};
use crate::model::{ConstraintStatus, Ident, RowId};

/// A dense interned id. Object ids and arrow ids are two separate id
/// spaces, each assigned in sorted-name order, so id order is name order.
pub type Nid = usize;

mod compile;
mod families;
mod lane;
mod render;

#[cfg(kani)]
pub(crate) use compile::ident_key;
pub(crate) use families::{
    coproduct_findings, dangling_findings, equation_findings, key_findings, pullback_findings,
};
pub(crate) use render::render_compact;

/// A finding's pure-int sort key — exactly the comparison prefix
/// `Finding`'s derived order walks: kind, then violation variant, then a
/// per-variant discriminator, then the witness's int fields in field
/// order, then the label.
/// A path step resolved to its arrow id; `None` = a step no declared
/// arrow defines (compose can never resolve past it).
pub(crate) type Step = Option<Nid>;

pub(crate) type SortKey = (u8, u8, u8, u64, u64, u64, u64, u8);

/// The lane flavor's finding: the taxonomy's shapes with int payloads
/// only (ids stand in for names; id order is name order). Rendered to
/// the public [`Finding`] by [`render_compact`] on the parsed flavor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompactFinding {
    Violation(CompactViolation),
    Dangling { arrow: Nid, src: usize },
}

/// The violation shapes over ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompactViolation {
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
        injections: Vec<Nid>,
    },
    Pullback(CompactPullback),
}

/// The pullback verdict arms over ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompactPullback {
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
pub(crate) struct EngineSchema {
    /// Objects in canonical order; empty on the lane flavor.
    pub(crate) object_names: Vec<Ident>,
    /// Arrows in canonical order; empty on the lane flavor.
    pub(crate) arrow_names: Vec<Ident>,
    /// Objects' byte keys in canonical order — lane flavor only (the
    /// parsed flavor's names carry the identity); read by the harness
    /// instance builder.
    #[cfg(kani)]
    pub(crate) object_keys: Vec<u64>,
    /// Arrows' byte keys in canonical order — lane flavor only.
    #[cfg(kani)]
    pub(crate) arrow_keys: Vec<u64>,
    /// Per arrow: the source object's id.
    pub(crate) arrow_src: Vec<Nid>,
    /// Per arrow: the target object's id.
    pub(crate) arrow_tgt: Vec<Nid>,
    /// Equations in declaration order; `domain: None` marks the walk's
    /// skip cases (empty lhs, undeclared lead arrow).
    pub(crate) equations: Vec<CompiledEquation>,
    /// Keys in declaration order; `None` = the walk would skip the key.
    pub(crate) keys: Vec<Option<CompiledKey>>,
    /// Coproducts in declaration order; `None` = the walk would skip.
    pub(crate) coproducts: Vec<Option<CompiledCoproduct>>,
    /// Pullbacks in declaration order; `None` = the walk would skip.
    pub(crate) pullbacks: Vec<Option<CompiledPullback>>,
}

/// A compiled path equation.
pub(crate) struct CompiledEquation {
    pub(crate) lhs: Vec<Step>,
    pub(crate) rhs: Vec<Step>,
    /// The lhs lead arrow's source object, when the equation is
    /// applicable.
    pub(crate) domain: Option<Nid>,
    pub(crate) status: ConstraintStatus,
}

/// A compiled declarative key.
pub(crate) struct CompiledKey {
    pub(crate) object: Nid,
    pub(crate) arrows: Vec<Nid>,
    pub(crate) status: ConstraintStatus,
}

/// A compiled coproduct declaration.
pub(crate) struct CompiledCoproduct {
    pub(crate) object: Nid,
    pub(crate) arrows: Vec<Nid>,
    pub(crate) status: ConstraintStatus,
}

/// A compiled pullback declaration.
pub(crate) struct CompiledPullback {
    pub(crate) apex: Nid,
    pub(crate) legs: [Nid; 2],
    pub(crate) cospan: [Nid; 2],
    pub(crate) status: ConstraintStatus,
}

/// The interned instance: per-object row counts (dense id spaces) and one
/// partial cell vector per declared arrow, parallel to its source rows.
pub(crate) struct EngineInstance {
    /// `row_counts[object]` = the object's dense row space size; `None` =
    /// the parsed instance carried no row space for the object.
    pub(crate) row_counts: Vec<Option<usize>>,
    /// `cells[arrow]` = the parsed value vector, parallel to source rows.
    pub(crate) cells: Vec<Option<Vec<Option<RowId>>>>,
}

/// The interned typed instance: one partial morphism table per arrow plus
/// the build reports, all by id.
pub(crate) struct EngineTyped {
    /// Per-object dense row space sizes, parallel to
    /// [`EngineSchema::object_names`].
    pub(crate) row_counts: Vec<Option<usize>>,
    /// `tables[arrow]` = `vec![None; row_count(src)]`, filled where the
    /// value resolved inside the target space.
    pub(crate) tables: Vec<Vec<Option<RowId>>>,
    /// Typing violations, in build order (arrow id order, then cell):
    /// (arrow, source cell index, offending value).
    pub(crate) typing_violations: Vec<(Nid, usize, RowId)>,
    /// Dangling cells, in build order (arrow id order, then cell).
    pub(crate) dangling: Vec<(Nid, usize)>,
}

/// The build classification over ids — `TypedInstance::build`'s walk:
/// every parsed arrow value lands in a morphism table entry, a dangling
/// record, or a typing violation; cells beyond the source object's row
/// space are ignored; arrows outside the presentation are ignored.
pub(crate) fn engine_build(schema: &EngineSchema, instance: &EngineInstance) -> EngineTyped {
    let mut tables = Vec::with_capacity(schema.arrow_src.len());
    // The exact report bound: every taken cell lands in at most one
    // report, and no cell outside a table can.
    let mut report_bound = 0usize;
    for (nid, src) in schema.arrow_src.iter().enumerate() {
        let n_src = instance.row_counts[*src].unwrap_or(0);
        if let Some(cells) = &instance.cells[nid] {
            report_bound += cells.len().min(n_src);
        }
    }
    let mut typing_violations = Vec::with_capacity(report_bound);
    let mut dangling = Vec::with_capacity(report_bound);
    for (nid, src) in schema.arrow_src.iter().enumerate() {
        let n_src = instance.row_counts[*src].unwrap_or(0);
        let n_tgt = schema
            .arrow_tgt
            .get(nid)
            .and_then(|tgt| instance.row_counts[*tgt])
            .unwrap_or(0);
        let mut table = vec![None; n_src];
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

/// Compose a compiled path over the typed tables — the parsed compose
/// walk over ids: an unresolved cell (dangling, typing-violated, or a
/// step no arrow defines) makes the path unresolvable, never a hard
/// failure.
pub(crate) fn engine_compose(typed: &EngineTyped, path: &[Step], start: RowId) -> Option<RowId> {
    let mut current = start;
    for step in path {
        let table = typed.tables.get((*step)?)?;
        let cell = table.get(current.0)?;
        current = (*cell)?;
    }
    Some(current)
}

/// Execute every declared constraint over the interned tables, compact:
/// findings sorted under the taxonomy's total order (the int-key sort is
/// exactly `Finding::cmp`), names never materialized. The parsed flavor
/// renders with [`render_compact`]; the lane harnesses assert the
/// compact shapes directly.
pub(crate) fn engine_families(
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
    dangling_findings(schema, typed, &mut out);
    equation_findings(schema, typed, &mut out);
    key_findings(schema, typed, &mut out);
    coproduct_findings(schema, typed, &mut out);
    pullback_findings(schema, typed, &mut out);
    out.sort_unstable_by_key(|entry| entry.0);
    out
}

/// The parsed-surface check: compile, walk, render, return public
/// findings in the taxonomy order (byte-identical to the pre-engine
/// walk's sorted output — pinned by the contract tests).
pub(crate) fn engine_check(schema: &EngineSchema, typed: &EngineTyped) -> Vec<Finding> {
    engine_families(schema, typed)
        .into_iter()
        .map(|(key, label, compact)| {
            let _ = key;
            render_compact(schema, label, &compact)
        })
        .collect()
}
