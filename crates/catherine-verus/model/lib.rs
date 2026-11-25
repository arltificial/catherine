// The Verus lane's verified model (openspec change
// `add-verus-verification-lane`), promoted unchanged from the probe
// (openspec/changes/add-verus-verification-lane/probe-seed/lib.rs,
// archived verbatim under crates/catherine-verus/probe-seed/lib.rs).
//
// Probe origin: can Verus verify catherine-engine code?
//
// Part 1 (compose): the verified walk `engine_compose` copied VERBATIM from
// crates/catherine-engine/src/engine/compose.rs, over the engine's Vec
// flavor (store.rs:230), proven panic-free with the dense-id invariant.
//
// Part 2 (equation-family spike): the equation check family
// (families.rs:105 `equation_findings`, families.rs:134 `push_equation_finding`,
// render.rs:65 `render_equation`) characterized spec-level:
//   - full functional spec for engine_compose (== ghost `gcompose`),
//   - walk output = exactly the applicable disagreeing (equation, row)
//     emissions, ascending in family key order (Lemma A: sortedness +
//     validity + completeness),
//   - the family key order equals the rendered `Finding` order
//     (Lemma B: `key_lt` == `find_lt`), the order-bridge claim the
//     contract tests pin.
//
// Deviations from catherine source, all flagged inline:
//   - containers: `&[Option<Step>]`/Vec paths (engine's Vec flavor), and
//     `Bounded::get` spells out `Vec::get`'s semantics so the spec attaches;
//   - `for (index, equation) in schema.equations.iter().enumerate()` and
//     `equation.domain.and_then(...)` rewritten as index loop + match
//     (Verus doesn't take the iterator/`and_then` forms; identical control flow);
//   - one ghost hint assert inside compose (quantifier instantiation);
//   - FindingKind/ViolationWitness carry only the equation arm (the other
//     families' arms are out of this spike's scope; the equation arm is
//     verbatim shape).

#![verifier::loop_isolation(false)]

use vstd::prelude::*;

verus! {

// ===== catherine types, verbatim definitions =====
// crates/catherine-engine/src/model/mod.rs:40
#[derive(Clone, Copy, PartialEq, Eq, Structural)]
pub struct RowId(pub usize);
// crates/catherine-engine/src/engine/mod.rs:18,68
pub type Nid = usize;
pub type Step = Option<Nid>;
// crates/catherine-engine/src/engine/mod.rs:62
pub type SortKey = (u8, u8, u8, u64, u64, u64, u64, u8);
// crates/catherine-engine/src/engine/store.rs:27,41
pub const ROWS_CAP: usize = 8;
pub const ARROWS_CAP: usize = 8;
// crates/catherine-engine/src/model/mod.rs ConstraintStatus (2 arms)
#[derive(Clone, Copy, PartialEq, Eq, Structural)]
pub enum ConstraintStatus {
    Accepted,
    Proposed,
}
// crates/catherine-engine/src/findings.rs:16 — arm order is the Ord order
#[derive(Clone, Copy, PartialEq, Eq, Structural)]
pub enum EpistemicLabel {
    Executed,
    Hypothesized,
    Unknown,
}
// crates/catherine-engine/src/findings.rs:33
pub fn label_for(status: ConstraintStatus) -> (l: EpistemicLabel)
    ensures
        status == ConstraintStatus::Accepted ==> l == EpistemicLabel::Executed,
        status == ConstraintStatus::Proposed ==> l == EpistemicLabel::Hypothesized,
{
    match status {
        ConstraintStatus::Accepted => EpistemicLabel::Executed,
        ConstraintStatus::Proposed => EpistemicLabel::Hypothesized,
    }
}

// ===== engine Vec-flavor containers (store.rs:230 flavor) =====

/// The Vec-flavor `Bounded`: `Bounded<T, CAP>(Vec<Option<T>>)` — same
/// shape; `CAP` carried and ignored exactly as in the production flavor.
/// No derives: `Structural` on a const-generic struct ICEs the Verus
/// fork (param-env CAP lookup); spec equality on whole tables is not
/// needed — the specs read `slots` via the field view.
pub struct Bounded<T, const CAP: usize> {
    pub slots: Vec<Option<T>>,
}

impl<T, const CAP: usize> Bounded<T, CAP> {
    /// store.rs:250 `len`
    pub fn len(&self) -> (n: usize)
        ensures n == self.slots@.len(),
    {
        self.slots.len()
    }

    /// store.rs:269 `get`, semantics spelled out for the spec.
    pub fn get(&self, i: usize) -> (r: Option<&Option<T>>)
        ensures
            r is Some == (i < self.slots@.len()),
            r is Some ==> self.slots@[i as int] == *r.unwrap(),
    {
        if i < self.slots.len() {
            Some(&self.slots[i])
        } else {
            None
        }
    }
}

/// mod.rs:207 `EngineTyped` — the fields the equation family reads.
pub struct EngineTyped {
    pub row_counts: Vec<Option<usize>>,
    pub tables: Vec<Bounded<RowId, ROWS_CAP>>,
}

/// mod.rs:164 `CompiledEquation` — paths as the engine's Vec flavor.
pub struct CompiledEquation {
    pub lhs: Vec<Option<Step>>,
    pub rhs: Vec<Option<Step>>,
    pub domain: Option<Nid>,
    pub status: ConstraintStatus,
}

/// mod.rs:125 `EngineSchema` — only the equation list (spike scope).
pub struct EngineSchema {
    pub equations: Vec<CompiledEquation>,
}

// ===== compact findings (mod.rs:76-120 shapes; equation arm in scope) =====
// The Key/Coproduct/Pullback arms are the remaining families' scope; they
// are omitted here (a const-generic field inside a Structural derive ICEs
// the Verus fork, which the other families' proofs will need to dodge too).

pub type Emitted = (SortKey, EpistemicLabel, CompactFinding);

#[derive(Clone, PartialEq, Eq, Structural)]
pub enum CompactFinding {
    Violation(CompactViolation),
    Dangling { arrow: Nid, src: usize },
}

#[derive(Clone, PartialEq, Eq, Structural)]
pub enum CompactViolation {
    Equation {
        index: usize,
        row: usize,
        lhs: usize,
        rhs: usize,
    },
}

// ===== rendered findings (findings.rs:205-216, 307-320, 72-81; equation arm) =====

#[derive(Clone, PartialEq, Eq, Structural)]
pub struct EquationWitness {
    pub equation: usize,
    pub row: RowId,
    pub lhs: RowId,
    pub rhs: RowId,
}

#[derive(Clone, PartialEq, Eq, Structural)]
pub enum ViolationWitness {
    Equation(EquationWitness),
}

#[derive(Clone, PartialEq, Eq, Structural)]
pub enum FindingKind {
    Violation(ViolationWitness),
}

#[derive(Clone, PartialEq, Eq, Structural)]
pub struct Finding {
    pub label: EpistemicLabel,
    pub kind: FindingKind,
}

// ===== ghost semantics =====

/// The ghost meaning of `engine_compose`: fold the path over the tables.
/// Guards make every seq index in-bounds (total function).
spec fn gcompose(tables: Seq<Bounded<RowId, ROWS_CAP>>, path: Seq<Option<Step>>, start: RowId) -> Option<RowId>
    decreases path.len(),
{
    if path.len() == 0 {
        Some(start)
    } else {
        let head = path[0];
        match head {
            Some(Some(step)) =>
                if (step as int) < tables.len() && (start.0 as int) < tables[step as int].slots@.len() {
                    match tables[step as int].slots@[start.0 as int] {
                        Some(v) => gcompose(tables, path.skip(1), v),
                        _ => None,
                    }
                } else {
                    None
                },
            _ => None,
        }
    }
}

/// The ghost emission for equation `i`, row `r`: exactly what the walk
/// emits at that (i, r) — `None` when skipped (no domain, unresolved
/// path, agreeing paths).
#[verifier::opaque]
spec fn emission(schema: &EngineSchema, typed: &EngineTyped, i: usize, r: usize) -> Option<Emitted>
{
    if i < schema.equations@.len() {
        let eq = schema.equations@[i as int];
        let domain = match eq.domain {
            Some(d) =>
                if (d as int) < typed.row_counts@.len() {
                    typed.row_counts@[d as int]
                } else {
                    None
                },
            None => None,
        };
        if domain is Some && r < domain.unwrap() {
            let l = gcompose(typed.tables@, eq.lhs@, RowId(r));
            let rr = gcompose(typed.tables@, eq.rhs@, RowId(r));
            if l is Some && rr is Some && l != rr {
                Some((
                    (0u8, 0u8, 0u8, i as u64, r as u64, l.unwrap().0 as u64, rr.unwrap().0 as u64, 0u8),
                    label_for_spec(eq.status),
                    CompactFinding::Violation(CompactViolation::Equation {
                        index: i,
                        row: r,
                        lhs: l.unwrap().0,
                        rhs: rr.unwrap().0,
                    }),
                ))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

spec fn label_for_spec(status: ConstraintStatus) -> EpistemicLabel {
    match status {
        ConstraintStatus::Accepted => EpistemicLabel::Executed,
        ConstraintStatus::Proposed => EpistemicLabel::Hypothesized,
    }
}

spec fn emission_applicable(schema: &EngineSchema, typed: &EngineTyped, i: usize, r: usize) -> bool {
    emission(schema, typed, i, r) is Some
}

spec fn before(i1: usize, r1: usize, i2: usize, r2: usize) -> bool {
    i1 < i2 || (i1 == i2 && r1 < r2)
}

// ===== spec order: family key order and rendered Finding order =====

/// The family key's strict order — the lexicographic order the tuple's
/// derived `PartialOrd` gives (all fields machine ints).
spec fn key_lt(a: SortKey, b: SortKey) -> bool {
    a.0 < b.0
        || a.0 == b.0 && a.1 < b.1
        || a.0 == b.0 && a.1 == b.1 && a.2 < b.2
        || a.0 == b.0 && a.1 == b.1 && a.2 == b.2 && a.3 < b.3
        || a.0 == b.0 && a.1 == b.1 && a.2 == b.2 && a.3 == b.3 && a.4 < b.4
        || a.0 == b.0 && a.1 == b.1 && a.2 == b.2 && a.3 == b.3 && a.4 == b.4 && a.5 < b.5
        || a.0 == b.0 && a.1 == b.1 && a.2 == b.2 && a.3 == b.3 && a.4 == b.4 && a.5 == b.5
            && a.6 < b.6
        || a.0 == b.0 && a.1 == b.1 && a.2 == b.2 && a.3 == b.3 && a.4 == b.4 && a.5 == b.5
            && a.6 == b.6 && a.7 < b.7
}

/// Label order = declaration order (findings.rs:16).
spec fn label_lt(a: EpistemicLabel, b: EpistemicLabel) -> bool {
    (a == EpistemicLabel::Executed && b == EpistemicLabel::Hypothesized)
        || (a == EpistemicLabel::Executed && b == EpistemicLabel::Unknown)
        || (a == EpistemicLabel::Hypothesized && b == EpistemicLabel::Unknown)
}

/// Witness order = the derived Ord over (equation, row, lhs, rhs)
/// (findings.rs:72 field order).


// ===== the verified compose walk (compose.rs:28, verbatim) =====

pub(crate) fn engine_compose(
    typed: &EngineTyped,
    path: &[Option<Step>],
    start: RowId,
) -> (r: Option<RowId>)
    requires
        start.0 < ROWS_CAP,
        forall|ti: usize, j: usize|
            #![trigger typed.tables@[ti as int].slots@[j as int]]
            ti < typed.tables.len() && j < typed.tables@[ti as int].slots@.len()
                && typed.tables@[ti as int].slots@[j as int] is Some
                ==> #[trigger] typed.tables@[ti as int].slots@[j as int].unwrap().0 < ROWS_CAP,
    ensures
        r == gcompose(typed.tables@, path@, start),
        match r {
            Some(rid) => rid.0 < ROWS_CAP,
            None => true,
        },
{
    let mut current = start;
    let mut i = 0;
    assert(path@.subrange(0, path@.len() as int) == path@); // scaffolding: entry invariant
    while i < path.len()
        invariant
            i <= path.len(),
            current.0 < ROWS_CAP,
            gcompose(typed.tables@, path@.subrange(i as int, path@.len() as int), current)
                == gcompose(typed.tables@, path@, start),
            forall|ti: usize, j: usize|
                ti < typed.tables.len() && j < typed.tables@[ti as int].slots@.len()
                    && typed.tables@[ti as int].slots@[j as int] is Some
                    ==> #[trigger] typed.tables@[ti as int].slots@[j as int].unwrap().0 < ROWS_CAP,
        decreases path.len() - i,
    {
        let step = match path[i] {
            Some(Some(step)) => step,
            _ => {
                assert(match path@[i as int] {
                    Some(Some(_)) => false,
                    _ => true,
                }); // scaffolding
                assert(gcompose(typed.tables@, path@.subrange(i as int, path@.len() as int), current) is None); // scaffolding
                return None;
            },
        };
        if step >= typed.tables.len() {
            assert(gcompose(typed.tables@, path@.subrange(i as int, path@.len() as int), current) is None); // scaffolding
            return None;
        }
        let cell = match typed.tables[step].get(current.0) {
            Some(Some(value)) => *value,
            Some(None) => {
                assert(typed.tables@[step as int].slots@[current.0 as int] is None); // scaffolding
                assert(gcompose(typed.tables@, path@.subrange(i as int, path@.len() as int), current) is None); // scaffolding
                return None;
            },
            None => {
                assert(current.0 >= typed.tables@[step as int].slots@.len()); // scaffolding
                assert(gcompose(typed.tables@, path@.subrange(i as int, path@.len() as int), current) is None); // scaffolding
                return None;
            },
        };
        // Proof scaffolding (not in compose.rs): instantiate the cell-bound
        // quantifier for the cell just read, then unfold gcompose one step
        // before `current` and `i` are overwritten.
        assert(path@[i as int] == Some(Some(step))); // scaffolding
        assert(typed.tables@[step as int].slots@[current.0 as int] == Some(cell)); // scaffolding
        assert(path@.subrange(i as int, path@.len() as int) == path@.skip(i as int)); // scaffolding
        proof {
            gcompose_unfold(typed.tables@, path@.skip(i as int), current, step, cell); // scaffolding: unfold
            vstd::seq_lib::lemma_seq_skip_of_skip(path@, i as int, 1); // scaffolding: skip(i).skip(1) == skip(i+1)
        }
        assert(gcompose(typed.tables@, path@.skip(i as int).skip(1), cell)
            == gcompose(typed.tables@, path@.skip(i as int + 1), cell)); // scaffolding
        assert(gcompose(typed.tables@, path@.skip(i as int + 1), cell)
            == gcompose(typed.tables@, path@.subrange(i as int + 1, path@.len() as int), cell)); // scaffolding
        assert(gcompose(typed.tables@, path@.subrange(i as int + 1, path@.len() as int), cell)
            == gcompose(typed.tables@, path@, start)); // scaffolding: invariant at next i
        current = cell;
        i += 1;
    }
    Some(current)
}

/// One-step unfold of `gcompose` at a mapped cell — busts the fuel limit
/// by making the recursive call appear syntactically in `ensures`.
proof fn gcompose_unfold(
    tables: Seq<Bounded<RowId, ROWS_CAP>>,
    path: Seq<Option<Step>>,
    start: RowId,
    step: usize,
    next: RowId,
)
    requires
        path.len() > 0,
        path[0] == Some(Some(step)),
        (step as int) < tables.len(),
        (start.0 as int) < tables[step as int].slots@.len(),
        tables[step as int].slots@[start.0 as int] == Some(next),
    ensures
        gcompose(tables, path, start) == gcompose(tables, path.skip(1), next),
{
}


/// Small-context instances of the three table-shape requires clauses —
/// keeps quantifier firing reliable inside the walk's big body.
proof fn domain_in_range(schema: &EngineSchema, typed: &EngineTyped, i: usize)
    requires
        i < schema.equations@.len(),
        schema.equations@[i as int].domain is Some,
        forall|j: usize|
            #![trigger schema.equations@[j as int].domain]
            j < schema.equations@.len() && schema.equations@[j as int].domain is Some
                ==> schema.equations@[j as int].domain.unwrap() < typed.row_counts@.len(),
    ensures
        schema.equations@[i as int].domain.unwrap() < typed.row_counts@.len(),
{
}

proof fn row_count_bound(typed: &EngineTyped, d: usize)
    requires
        d < typed.row_counts@.len(),
        typed.row_counts@[d as int] is Some,
        forall|dd: usize|
            #![trigger typed.row_counts@[dd as int]]
            dd < typed.row_counts@.len() && typed.row_counts@[dd as int] is Some
                ==> typed.row_counts@[dd as int].unwrap() <= ROWS_CAP,
    ensures
        typed.row_counts@[d as int].unwrap() <= ROWS_CAP,
{
}

proof fn table_cell_bound(typed: &EngineTyped, ti: usize, j: usize)
    requires
        ti < typed.tables.len(),
        j < typed.tables@[ti as int].slots@.len(),
        typed.tables@[ti as int].slots@[j as int] is Some,
        forall|tii: usize, jj: usize|
            #![trigger typed.tables@[tii as int].slots@[jj as int]]
            tii < typed.tables.len() && jj < typed.tables@[tii as int].slots@.len()
                && typed.tables@[tii as int].slots@[jj as int] is Some
                ==> typed.tables@[tii as int].slots@[jj as int].unwrap().0 < ROWS_CAP,
    ensures
        typed.tables@[ti as int].slots@[j as int].unwrap().0 < ROWS_CAP,
{
}

// ===== the equation walk (families.rs:105,134 — Vec sink, index loops) =====

/// One equation-disagreement finding for a composed row pair —
/// families.rs:134 `push_equation_finding` verbatim (Vec emit).
fn push_equation_finding(
    out: &mut Vec<Emitted>,
    index: usize,
    row: usize,
    lhs: usize,
    rhs: usize,
    status: ConstraintStatus,
) -> (label: EpistemicLabel)
    ensures
        final(out)@ == old(out)@.push((
            (0u8, 0u8, 0u8, index as u64, row as u64, lhs as u64, rhs as u64, 0u8),
            label,
            CompactFinding::Violation(CompactViolation::Equation { index, row, lhs, rhs }),
        )),
        label == label_for_spec(status),
{
    let l = label_for(status);
    out.push((
        (0u8, 0u8, 0u8, index as u64, row as u64, lhs as u64, rhs as u64, 0u8),
        l,
        CompactFinding::Violation(CompactViolation::Equation {
            index,
            row,
            lhs,
            rhs,
        }),
    ));
    l
}

/// families.rs:105 `equation_findings` — every declared equation on every
/// row of its domain object. Iterator/`and_then` forms rewritten as index
/// loop + match (identical control flow); sink specialized to the parsed
/// flavor's heap `Vec`.
///
/// Proof strategy: the loop invariant is ONE quantifier-free equality
/// `out@ == collected_upto(schema, typed, ei, row)` against a recursive
/// spec mirror of the walk. Recursive spec fns stay uninterpreted symbols
/// in SMT (non-recursive ones get inlined, killing triggers), so all
/// quantifier reasoning happens in small induction lemmas over the mirror,
/// never inside the walk's own big VC.
fn equation_findings(
    schema: &EngineSchema,
    typed: &EngineTyped,
) -> (out: Vec<Emitted>)
    requires
        forall|ti: usize, j: usize|
            #![trigger typed.tables@[ti as int].slots@[j as int]]
            ti < typed.tables.len() && j < typed.tables@[ti as int].slots@.len()
                && typed.tables@[ti as int].slots@[j as int] is Some
                ==> #[trigger] typed.tables@[ti as int].slots@[j as int].unwrap().0 < ROWS_CAP,
        forall|i: usize|
            #![trigger schema.equations@[i as int].domain]
            i < schema.equations@.len() && schema.equations@[i as int].domain is Some
                ==> schema.equations@[i as int].domain.unwrap() < typed.row_counts@.len(),
        forall|d: usize|
            #![trigger typed.row_counts@[d as int]]
            d < typed.row_counts@.len() && typed.row_counts@[d as int] is Some
                ==> typed.row_counts@[d as int].unwrap() <= ROWS_CAP,
    ensures
        // the walk mirror, exactly
        out@ == collected_upto(schema, typed, schema.equations@.len() as usize, 0),
        // (a) family key order: output ascending, strictly
        forall|j: usize, k: usize| j < k < out@.len() ==> key_lt(
            out@[j as int].0,
            out@[k as int].0,
        ),
        // (b) validity: every emission is the ghost emission of a real
        // walk position
        forall|j: usize|
            #![trigger out@[j as int]]
            j < out@.len() ==> exists|i: usize, r: usize|
                i < schema.equations@.len() && emission(schema, typed, i, r) is Some
                    && out@[j as int] == emission(schema, typed, i, r).unwrap(),
        // (c) completeness: every applicable disagreeing (i, r) emitted
        forall|i: usize, r: usize|
            #![trigger emission(schema, typed, i, r)]
            emission_applicable(schema, typed, i, r) ==> exists|j: usize|
                j < out@.len() && emission(schema, typed, i, r) is Some
                    && out@[j as int] == emission(schema, typed, i, r).unwrap(),
{
    let mut out: Vec<Emitted> = Vec::new();
    let mut ei: usize = 0;
    while ei < schema.equations.len()
        invariant
            ei <= schema.equations@.len(),
            out@ == collected_upto(schema, typed, ei, 0),
            // requires restate + len bridges (heap-mutating loops kill fn requires)
            schema.equations.len() as int == schema.equations@.len(),
            typed.row_counts@.len() == typed.row_counts.len() as int,
            typed.tables@.len() == typed.tables.len() as int,
            forall|ti: usize, j: usize|
                #![trigger typed.tables@[ti as int].slots@[j as int]]
                ti < typed.tables.len() && j < typed.tables@[ti as int].slots@.len()
                    && typed.tables@[ti as int].slots@[j as int] is Some
                    ==> typed.tables@[ti as int].slots@[j as int].unwrap().0 < ROWS_CAP,
            forall|i: usize|
                #![trigger schema.equations@[i as int].domain]
                i < schema.equations@.len() && schema.equations@[i as int].domain is Some
                    ==> schema.equations@[i as int].domain.unwrap() < typed.row_counts@.len(),
            forall|d: usize|
                #![trigger typed.row_counts@[d as int]]
                d < typed.row_counts@.len() && typed.row_counts@[d as int] is Some
                    ==> typed.row_counts@[d as int].unwrap() <= ROWS_CAP,
        decreases schema.equations@.len() - ei,
    {
        let equation = &schema.equations[ei];
        let domain: Option<usize> = match equation.domain {
            Some(d) => {
                assert(schema.equations@[ei as int].domain == Some(d)); // borrow == seq elem
                proof {
                    domain_in_range(schema, typed, ei);
                }
                assert(d < typed.row_counts@.len());
                match typed.row_counts[d] {
                    Some(n) => {
                        assert(typed.row_counts@[d as int] == Some(n)); // ghost read
                        proof {
                            row_count_bound(typed, d);
                        }
                        assert(n <= ROWS_CAP);
                        Some(n)
                    },
                    None => None,
                }
            },
            None => None,
        };
        if let Some(domain) = domain {
            assert(domain == domain_count(schema, typed, ei)); // mirror agrees with the walk
            let mut row: usize = 0;
            while row < domain
                invariant
                    row <= domain,
                    ei < schema.equations@.len(),
                    domain <= ROWS_CAP,
                    out@ == collected_upto(schema, typed, ei, row),
                    schema.equations.len() as int == schema.equations@.len(),
                    typed.row_counts@.len() == typed.row_counts.len() as int,
                    typed.tables@.len() == typed.tables.len() as int,
                    forall|ti: usize, j: usize|
                        #![trigger typed.tables@[ti as int].slots@[j as int]]
                        ti < typed.tables.len() && j < typed.tables@[ti as int].slots@.len()
                            && typed.tables@[ti as int].slots@[j as int] is Some
                            ==> typed.tables@[ti as int].slots@[j as int].unwrap().0 < ROWS_CAP,
                    forall|i: usize|
                        #![trigger schema.equations@[i as int].domain]
                        i < schema.equations@.len() && schema.equations@[i as int].domain is Some
                            ==> schema.equations@[i as int].domain.unwrap() < typed.row_counts@.len(),
                    forall|d: usize|
                        #![trigger typed.row_counts@[d as int]]
                        d < typed.row_counts@.len() && typed.row_counts@[d as int] is Some
                            ==> typed.row_counts@[d as int].unwrap() <= ROWS_CAP,
                decreases domain - row,
            {
                assert(row < ROWS_CAP); // row <= domain <= ROWS_CAP
                let ol = engine_compose(typed, &equation.lhs, RowId(row));
                let or = engine_compose(typed, &equation.rhs, RowId(row));
                if let (Some(lhs), Some(rhs)) = (ol, or) {
                    if lhs != rhs {
                        let nk = (
                            0u8,
                            0u8,
                            0u8,
                            ei as u64,
                            row as u64,
                            lhs.0 as u64,
                            rhs.0 as u64,
                            0u8,
                        );
                        // emission at (ei, row) is exactly this key/label/finding
                        proof { reveal(emission); }
                        assert(emission(schema, typed, ei, row) == Some((nk, label_for_spec(equation.status), CompactFinding::Violation(CompactViolation::Equation {
                            index: ei,
                            row,
                            lhs: lhs.0,
                            rhs: rhs.0,
                        }))));
                        let l = push_equation_finding(&mut out, ei, row, lhs.0, rhs.0, equation.status);
                        proof {
                            reveal(emission);
                            // mirror step: append-case of collected_upto
                            assert(collected_upto(schema, typed, ei, (row + 1) as usize)
                                == collected_upto(schema, typed, ei, row).push(emission(schema, typed, ei, row).unwrap()));
                        }
                    } else {
                        proof {
                            reveal(emission);
                            assert(emission(schema, typed, ei, row) is None); // equal compose: no finding
                            // mirror step: skip-case
                            assert(collected_upto(schema, typed, ei, (row + 1) as usize)
                                == collected_upto(schema, typed, ei, row));
                        }
                    }
                } else {
                    proof {
                        reveal(emission);
                        assert(emission(schema, typed, ei, row) is None); // compose failed: no emission
                        assert(collected_upto(schema, typed, ei, (row + 1) as usize)
                            == collected_upto(schema, typed, ei, row));
                    }
                }
                row += 1;
            }
            // equation ei fully walked: mirror crosses to the next equation
            proof {
                assert(collected_upto(schema, typed, (ei + 1) as usize, 0)
                    == collected_upto(schema, typed, ei, domain));
            }
        }
        ei += 1;
    }
    // Final contract: (b)(a)(c) follow from the mirror equality by
    // induction lemmas over collected_upto.
    proof {
        let e_all = schema.equations@.len() as usize;
        assert(out@ == collected_upto(schema, typed, e_all, 0));
        // (b) validity
        assert forall|j: usize| #![trigger out@[j as int]] j < out@.len() implies (exists|i: usize, r: usize|
            i < schema.equations@.len() && emission(schema, typed, i, r) is Some
                && out@[j as int] == emission(schema, typed, i, r).unwrap())
        by {
            let w = collected_upto_pos_single(schema, typed, e_all, 0, j);
            assert(w.0 < schema.equations@.len()); // before(w, e_all, 0)
            assert(out@[j as int] == collected_upto(schema, typed, e_all, 0)[j as int]); // mirror equality
            assert(collected_upto(schema, typed, e_all, 0)[j as int] == emission(schema, typed, w.0, w.1).unwrap());
        }
        // (a) key order
        collected_upto_sorted(schema, typed, e_all, 0);
        assert forall|j: usize, k: usize| #![trigger out@[j as int], out@[k as int]]
            j < k < out@.len() implies key_lt(out@[j as int].0, out@[k as int].0)
        by {
            assert(out@[j as int] == collected_upto(schema, typed, e_all, 0)[j as int]); // grounds sorted's trigger
            assert(out@[k as int] == collected_upto(schema, typed, e_all, 0)[k as int]);
        }
        // (c) completeness
        assert forall|i: usize, r: usize| #![trigger emission(schema, typed, i, r)]
            emission_applicable(schema, typed, i, r) implies (exists|j: usize|
                j < out@.len() && emission(schema, typed, i, r) is Some
                    && out@[j as int] == emission(schema, typed, i, r).unwrap())
        by {
            reveal(emission);
            assert(i < schema.equations@.len()); // applicable => Some => in range
            let jw = collected_upto_complete(schema, typed, e_all, 0, i, r);
            assert(collected_upto(schema, typed, e_all, 0).len() == out@.len()); // mirror equality
            assert(out@.len() == out.len() as int); // Vec view
            let j = jw as usize; // exact: 0 <= jw < out@.len() <= usize::MAX
            assert(j as int == jw);
            assert(out@[j as int] == collected_upto(schema, typed, e_all, 0)[jw]); // mirror equality
        }
    }
    out
}

/// Keys of emissions at ordered (i, r) positions are themselves ordered —
/// the walk-order-is-key-order fact (key prefix (0,0,0,i,r,...)).
proof fn emission_key_order(
    schema: &EngineSchema,
    typed: &EngineTyped,
    i1: usize,
    r1: usize,
    i2: usize,
    r2: usize,
)
    requires
        before(i1, r1, i2, r2),
        emission(schema, typed, i1, r1) is Some,
        emission(schema, typed, i2, r2) is Some,
    ensures
        key_lt(
            emission(schema, typed, i1, r1).unwrap().0,
            emission(schema, typed, i2, r2).unwrap().0,
        ),
{
    reveal(emission);
    let k1 = emission(schema, typed, i1, r1).unwrap().0;
    let k2 = emission(schema, typed, i2, r2).unwrap().0;
    assert(k1.0 == 0u8 && k1.1 == 0u8 && k1.2 == 0u8 && k1.7 == 0u8); // scaffolding
    assert(k2.0 == 0u8 && k2.1 == 0u8 && k2.2 == 0u8 && k2.7 == 0u8); // scaffolding
    assert(k1.3 == i1 as u64 && k1.4 == r1 as u64); // scaffolding
    assert(k2.3 == i2 as u64 && k2.4 == r2 as u64); // scaffolding
}

// ===== collected_upto: recursive spec mirror of the walk =====
//
// The walk loop's invariant is ONE quantifier-free equality against this
// recursive mirror. Recursive spec fns stay uninterpreted symbols in SMT
// (non-recursive ones get inlined, killing triggers), so every quantified
// contract clause (a)/(b)/(c) is discharged by a small induction lemma
// over the mirror instead of inside the walk's own big VC.

/// Rows available for equation `ei` — `0` unless the domain object's row
/// count is in range. Total: no preconditions, matches `emission`'s guards.
spec fn domain_count(schema: &EngineSchema, typed: &EngineTyped, ei: usize) -> usize {
    if ei < schema.equations@.len() && schema.equations@[ei as int].domain is Some {
        let d = schema.equations@[ei as int].domain.unwrap();
        if d < typed.row_counts@.len() && typed.row_counts@[d as int] is Some {
            typed.row_counts@[d as int].unwrap()
        } else {
            0
        }
    } else {
        0
    }
}

/// Emissions the walk produces strictly before walk position `(ei, row)`:
/// equations `[0, ei)` in full plus rows `[0, row)` of equation `ei`.
spec fn collected_upto(schema: &EngineSchema, typed: &EngineTyped, ei: usize, row: usize) -> Seq<Emitted>
    decreases ei as int, row as int,
{
    if ei == 0 && row == 0 {
        Seq::<Emitted>::empty()
    } else if ei == 0 {
        let pre = collected_upto(schema, typed, 0, (row - 1) as usize);
        if emission(schema, typed, 0, (row - 1) as usize) is Some {
            pre.push(emission(schema, typed, 0, (row - 1) as usize).unwrap())
        } else {
            pre
        }
    } else if row == 0 {
        collected_upto(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize))
    } else {
        let pre = collected_upto(schema, typed, ei, (row - 1) as usize);
        if emission(schema, typed, ei, (row - 1) as usize) is Some {
            pre.push(emission(schema, typed, ei, (row - 1) as usize).unwrap())
        } else {
            pre
        }
    }
}

/// Walk position of the `j`-th emission (garbage when out of range).
/// Recursive spec fn: stays an uninterpreted SMT symbol, so triggers on
/// `walk_pos(...)` e-match instead of drowning the solver the way the
/// inlined `emission` did.
spec fn walk_pos(schema: &EngineSchema, typed: &EngineTyped, ei: usize, row: usize, j: int) -> (usize, usize)
    decreases ei as int, row as int,
{
    if ei == 0 && row == 0 {
        (0usize, 0usize)
    } else if ei == 0 {
        let pre_len = collected_upto(schema, typed, 0, (row - 1) as usize).len();
        if j == pre_len && emission(schema, typed, 0, (row - 1) as usize) is Some {
            (0usize, (row - 1) as usize)
        } else {
            walk_pos(schema, typed, 0, (row - 1) as usize, j)
        }
    } else if row == 0 {
        walk_pos(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize), j)
    } else {
        let pre_len = collected_upto(schema, typed, ei, (row - 1) as usize).len();
        if j == pre_len && emission(schema, typed, ei, (row - 1) as usize) is Some {
            (ei, (row - 1) as usize)
        } else {
            walk_pos(schema, typed, ei, (row - 1) as usize, j)
        }
    }
}

/// The `j`-th emission sits at a real, applicable walk position, is
/// emitted exactly, and lies before the current step `(ei, row)`.
proof fn collected_upto_pos_single(
    schema: &EngineSchema,
    typed: &EngineTyped,
    ei: usize,
    row: usize,
    j: usize,
) -> (w: (usize, usize))
    requires
        j < collected_upto(schema, typed, ei, row).len(),
    ensures
        w == walk_pos(schema, typed, ei, row, j as int),
        emission(schema, typed, w.0, w.1) is Some,
        collected_upto(schema, typed, ei, row)[j as int] == emission(schema, typed, w.0, w.1).unwrap(),
        before(w.0, w.1, ei, row),
    decreases ei as int, row as int,
{
    reveal(emission);
    if ei == 0 && row == 0 {
        assert(collected_upto(schema, typed, 0, 0).len() == 0); // contradicts requires
        return (0usize, 0usize);
    } else if ei == 0 {
        let pre = collected_upto(schema, typed, 0, (row - 1) as usize);
        if emission(schema, typed, 0, (row - 1) as usize) is Some {
            let whole = pre.push(emission(schema, typed, 0, (row - 1) as usize).unwrap());
            assert(collected_upto(schema, typed, 0, row) == whole);
            if j == pre.len() {
                assert(whole[j as int] == emission(schema, typed, 0, (row - 1) as usize).unwrap());
                return (0usize, (row - 1) as usize);
            } else {
                assert(j < pre.len());
                let w2 = collected_upto_pos_single(schema, typed, 0, (row - 1) as usize, j);
                assert(collected_upto(schema, typed, 0, row)[j as int] == pre[j as int]);
                assert(walk_pos(schema, typed, 0, row, j as int) == walk_pos(schema, typed, 0, (row - 1) as usize, j as int));
                return w2;
            }
        } else {
            let w2 = collected_upto_pos_single(schema, typed, 0, (row - 1) as usize, j);
            assert(collected_upto(schema, typed, 0, row) == pre);
            assert(walk_pos(schema, typed, 0, row, j as int) == walk_pos(schema, typed, 0, (row - 1) as usize, j as int));
            return w2;
        }
    } else if row == 0 {
        let w2 = collected_upto_pos_single(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize), j);
        assert(collected_upto(schema, typed, ei, 0) == collected_upto(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize)));
        return w2;
    } else {
        let pre = collected_upto(schema, typed, ei, (row - 1) as usize);
        if emission(schema, typed, ei, (row - 1) as usize) is Some {
            let whole = pre.push(emission(schema, typed, ei, (row - 1) as usize).unwrap());
            assert(collected_upto(schema, typed, ei, row) == whole);
            if j == pre.len() {
                assert(whole[j as int] == emission(schema, typed, ei, (row - 1) as usize).unwrap());
                return (ei, (row - 1) as usize);
            } else {
                assert(j < pre.len());
                let w2 = collected_upto_pos_single(schema, typed, ei, (row - 1) as usize, j);
                assert(collected_upto(schema, typed, ei, row)[j as int] == pre[j as int]);
                assert(walk_pos(schema, typed, ei, row, j as int) == walk_pos(schema, typed, ei, (row - 1) as usize, j as int));
                return w2;
            }
        } else {
            let w2 = collected_upto_pos_single(schema, typed, ei, (row - 1) as usize, j);
            assert(collected_upto(schema, typed, ei, row) == pre);
            assert(walk_pos(schema, typed, ei, row, j as int) == walk_pos(schema, typed, ei, (row - 1) as usize, j as int));
            return w2;
        }
    }
}

/// Emission positions are ordered: position of `j` precedes position of
/// `k` whenever `j < k`. Pure walk-order bookkeeping.
proof fn collected_upto_pos_pair(
    schema: &EngineSchema,
    typed: &EngineTyped,
    ei: usize,
    row: usize,
    j: usize,
    k: usize,
)
    requires
        j < k < collected_upto(schema, typed, ei, row).len(),
    ensures
        before(
            walk_pos(schema, typed, ei, row, j as int).0,
            walk_pos(schema, typed, ei, row, j as int).1,
            walk_pos(schema, typed, ei, row, k as int).0,
            walk_pos(schema, typed, ei, row, k as int).1,
        ),
    decreases ei as int, row as int,
{
    if ei == 0 && row == 0 {
        assert(collected_upto(schema, typed, 0, 0).len() == 0); // requires false
    } else if ei == 0 {
        let pre = collected_upto(schema, typed, 0, (row - 1) as usize);
        if emission(schema, typed, 0, (row - 1) as usize) is Some {
            assert(collected_upto(schema, typed, 0, row) == pre.push(emission(schema, typed, 0, (row - 1) as usize).unwrap()));
            if k == pre.len() {
                assert(j < pre.len());
                collected_upto_pos_single(schema, typed, 0, (row - 1) as usize, j);
                assert(walk_pos(schema, typed, 0, row, k as int) == (0usize, (row - 1) as usize));
                assert(walk_pos(schema, typed, 0, row, j as int) == walk_pos(schema, typed, 0, (row - 1) as usize, j as int));
            } else {
                collected_upto_pos_pair(schema, typed, 0, (row - 1) as usize, j, k);
                assert(walk_pos(schema, typed, 0, row, j as int) == walk_pos(schema, typed, 0, (row - 1) as usize, j as int));
                assert(walk_pos(schema, typed, 0, row, k as int) == walk_pos(schema, typed, 0, (row - 1) as usize, k as int));
            }
        } else {
            collected_upto_pos_pair(schema, typed, 0, (row - 1) as usize, j, k);
            assert(walk_pos(schema, typed, 0, row, j as int) == walk_pos(schema, typed, 0, (row - 1) as usize, j as int));
            assert(walk_pos(schema, typed, 0, row, k as int) == walk_pos(schema, typed, 0, (row - 1) as usize, k as int));
        }
    } else if row == 0 {
        collected_upto_pos_pair(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize), j, k);
        assert(walk_pos(schema, typed, ei, 0, j as int) == walk_pos(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize), j as int));
        assert(walk_pos(schema, typed, ei, 0, k as int) == walk_pos(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize), k as int));
    } else {
        let pre = collected_upto(schema, typed, ei, (row - 1) as usize);
        if emission(schema, typed, ei, (row - 1) as usize) is Some {
            assert(collected_upto(schema, typed, ei, row) == pre.push(emission(schema, typed, ei, (row - 1) as usize).unwrap()));
            if k == pre.len() {
                assert(j < pre.len());
                collected_upto_pos_single(schema, typed, ei, (row - 1) as usize, j);
                assert(walk_pos(schema, typed, ei, row, k as int) == (ei, (row - 1) as usize));
                assert(walk_pos(schema, typed, ei, row, j as int) == walk_pos(schema, typed, ei, (row - 1) as usize, j as int));
            } else {
                collected_upto_pos_pair(schema, typed, ei, (row - 1) as usize, j, k);
                assert(walk_pos(schema, typed, ei, row, j as int) == walk_pos(schema, typed, ei, (row - 1) as usize, j as int));
                assert(walk_pos(schema, typed, ei, row, k as int) == walk_pos(schema, typed, ei, (row - 1) as usize, k as int));
            }
        } else {
            collected_upto_pos_pair(schema, typed, ei, (row - 1) as usize, j, k);
            assert(walk_pos(schema, typed, ei, row, j as int) == walk_pos(schema, typed, ei, (row - 1) as usize, j as int));
            assert(walk_pos(schema, typed, ei, row, k as int) == walk_pos(schema, typed, ei, (row - 1) as usize, k as int));
        }
    }
}

/// (a) Output keys ascend along the walk order: elements at ordered walk
/// positions have ordered keys (`emission_key_order`).
proof fn collected_upto_sorted(
    schema: &EngineSchema,
    typed: &EngineTyped,
    ei: usize,
    row: usize,
)
    ensures
        forall|j: usize, k: usize| #![trigger collected_upto(schema, typed, ei, row)[j as int], collected_upto(schema, typed, ei, row)[k as int]]
            j < k < collected_upto(schema, typed, ei, row).len() ==> key_lt(
                collected_upto(schema, typed, ei, row)[j as int].0,
                collected_upto(schema, typed, ei, row)[k as int].0,
            ),
{
    assert forall|j: usize, k: usize| #![trigger collected_upto(schema, typed, ei, row)[j as int], collected_upto(schema, typed, ei, row)[k as int]]
        j < k < collected_upto(schema, typed, ei, row).len() ==> key_lt(
            collected_upto(schema, typed, ei, row)[j as int].0,
            collected_upto(schema, typed, ei, row)[k as int].0,
        )
    by {
        if j < k < collected_upto(schema, typed, ei, row).len() {
            let wj = collected_upto_pos_single(schema, typed, ei, row, j);
            let wk = collected_upto_pos_single(schema, typed, ei, row, k);
            collected_upto_pos_pair(schema, typed, ei, row, j, k);
            emission_key_order(schema, typed, wj.0, wj.1, wk.0, wk.1);
            assert(collected_upto(schema, typed, ei, row)[j as int] == emission(schema, typed, wj.0, wj.1).unwrap());
            assert(collected_upto(schema, typed, ei, row)[k as int] == emission(schema, typed, wk.0, wk.1).unwrap());
        }
    }
}

/// (c) Every applicable emission position appears in the mirror output;
/// returns its index in the mirror output. Return-style witness: no
/// existential to instantiate, each branch contributes concrete index
/// arithmetic.
proof fn collected_upto_complete(
    schema: &EngineSchema,
    typed: &EngineTyped,
    ei: usize,
    row: usize,
    i: usize,
    r: usize,
) -> (j: int)
    requires
        i < schema.equations@.len(),
        emission(schema, typed, i, r) is Some,
        before(i, r, ei, row),
    ensures
        0 <= j < collected_upto(schema, typed, ei, row).len(),
        collected_upto(schema, typed, ei, row)[j] == emission(schema, typed, i, r).unwrap(),
    decreases ei as int, row as int,
{
    reveal(emission);
    if ei == 0 {
        if row == 0 {
            assert(false); // before(i, r, 0, 0) is impossible for usize
            return 0;
        } else {
            let pre = collected_upto(schema, typed, 0, (row - 1) as usize);
            if emission(schema, typed, 0, (row - 1) as usize) is Some {
                assert(collected_upto(schema, typed, 0, row) == pre.push(emission(schema, typed, 0, (row - 1) as usize).unwrap()));
                if i == 0usize && r == row - 1 {
                    assert(r == (row - 1) as usize); // cast exactness below
                    assert(pre.len() < collected_upto(schema, typed, 0, row).len());
                    assert(collected_upto(schema, typed, 0, row)[pre.len() as int] == emission(schema, typed, i, r).unwrap());
                    return pre.len() as int;
                } else {
                    assert(i == 0usize && r < row - 1); // i < 0 impossible
                    let jw = collected_upto_complete(schema, typed, 0, (row - 1) as usize, i, r);
                    assert(jw < pre.len());
                    assert(collected_upto(schema, typed, 0, row)[jw] == pre[jw]);
                    return jw;
                }
            } else {
                let jw = collected_upto_complete(schema, typed, 0, (row - 1) as usize, i, r);
                assert(collected_upto(schema, typed, 0, row) == pre);
                return jw;
            }
        }
    } else if row == 0 {
        if i == ei - 1 {
            assert(r < domain_count(schema, typed, i)); // emission Some pins r below the row count
        }
        let jw = collected_upto_complete(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize), i, r);
        assert(collected_upto(schema, typed, ei, 0) == collected_upto(schema, typed, (ei - 1) as usize, domain_count(schema, typed, (ei - 1) as usize)));
        return jw;
    } else {
        let pre = collected_upto(schema, typed, ei, (row - 1) as usize);
        if emission(schema, typed, ei, (row - 1) as usize) is Some {
            assert(collected_upto(schema, typed, ei, row) == pre.push(emission(schema, typed, ei, (row - 1) as usize).unwrap()));
            if i == ei && r == row - 1 {
                assert(r == (row - 1) as usize); // cast exactness below
                assert(pre.len() < collected_upto(schema, typed, ei, row).len());
                assert(collected_upto(schema, typed, ei, row)[pre.len() as int] == emission(schema, typed, i, r).unwrap());
                return pre.len() as int;
            } else {
                if i == ei {
                    assert(r < row - 1);
                }
                let jw = collected_upto_complete(schema, typed, ei, (row - 1) as usize, i, r);
                assert(jw < pre.len());
                assert(collected_upto(schema, typed, ei, row)[jw] == pre[jw]);
                return jw;
            }
        } else {
            let jw = collected_upto_complete(schema, typed, ei, (row - 1) as usize, i, r);
            assert(collected_upto(schema, typed, ei, row) == pre);
            return jw;
        }
    }
}

} // verus!
