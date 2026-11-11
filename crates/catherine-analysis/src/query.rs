//! Purpose: the query primitive — forward/backward closure over a seed
//! set and a morphism set M, the mixed blast-radius closure, cycle
//! membership, and fan-in/fan-out (openspec change
//! `add-bounded-analysis`, task 9; DESIGN §3 reuse map, E8).
//!
//! Responsibilities: answer reachability questions over a typed
//! instance. A query names its seeds (validated: an unknown seed is
//! rejected, not ignored — `seeds_exist`) and the schema morphisms to
//! follow. Closure terminates on any finite instance including cyclic M
//! (`closure_terminates` — one worklist, a visited set, each node
//! visited at most once); an unresolved reference is `None` in the
//! morphism table and is never followed (`dangling_not_followed`);
//! results are a `BTreeSet` sorted by construction (`results_ordered`);
//! `fan_in` counts exactly the defined preimages of a node
//! (`fan_in_is_preimage_size`). The parity gate carries these donor
//! contract names as tests (`tests/query_parity.rs`).
//!
//! Rationale: adapted (not authored) from specodelic
//! (`charly/specodelic`, Apache-2.0, ships no NOTICE file)
//! `src/acset/query.rs`, whose contracts this port preserves under the
//! donor's names. What was adapted and why: specodelic's world is
//! string-keyed (one global id space per instance file, `BTreeSet`
//! seeds/`BTreeSet<String>` results, morphism vectors queried by name
//! with no schema present); catherine's kernel is dense per-object
//! `RowId` tables over a typed schema, so a node must be named by
//! `(object, row)` — [`NodeRef`] — and the walk needs the schema's
//! arrow typing (`src`/`tgt`) to interpret each morphism table. The
//! donor's `blast_radius` mixed-closure fixpoint (`X ↦ backward∘forward`
//! until closed) ports verbatim: finite instances make it terminate,
//! and its parity-pinned definition (backward of touched ∪ forward reach
//! of backward-reached dependents) is preserved exactly. `cyclic_nodes`
//! ports as the closure-based reading: a node is on a cycle through M
//! iff some defined morphism value `s → t` has `s` reachable forward
//! from `t` (trivial self-edges included); dangling values contribute
//! no edge.

use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, RowId, Schema};
use std::collections::{BTreeMap, BTreeSet};

/// Which way the closure walks the morphism values: forward follows
/// `src → tgt` (what a seed reaches), backward follows `tgt → src` (what
/// depends on it — the dependents half of a blast radius).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

/// A node of a typed instance: `(object, dense row id)`. Catherine's
/// rows are dense per object, so a bare `RowId` is not a node name —
/// the donor's globally unique string ids needed no such wrapper.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeRef {
    /// The object whose dense row space the row lives in.
    pub object: Ident,
    /// The dense row id within that object's space.
    pub row: RowId,
}

/// One reachability question: walk the instance from `seeds` along the
/// named schema morphisms, in `direction`. `run` rejects unknown seeds
/// (`seeds_exist` — rejected, not ignored).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub direction: Direction,
    pub seeds: BTreeSet<NodeRef>,
    /// The set M of schema morphisms to follow, by name. A name matching
    /// no declared arrow contributes no edges (the donor's contract:
    /// restricting M yields exactly that subgraph).
    pub morphisms: BTreeSet<Ident>,
}

/// Why a query was rejected: a seed that is not a node of the instance,
/// named — never silently dropped from the traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    UnknownSeed(NodeRef),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSeed(seed) => write!(
                f,
                "unknown seed `{}#{}` — not a node of this instance",
                seed.object.0, seed.row.0
            ),
        }
    }
}

/// Whether `node` is an id of the typed instance's row spaces.
fn exists(node: &NodeRef, typed: &TypedInstance) -> bool {
    typed
        .rows()
        .get(&node.object)
        .is_some_and(|rows| node.row.0 < rows.len())
}

/// Check a query against an instance without running it: every seed
/// must be a node of the instance (`seeds_exist`). All violations are
/// reported, sorted.
pub fn check(
    query: &Query,
    _schema: &Schema,
    typed: &TypedInstance,
) -> Result<(), Vec<QueryError>> {
    let unknown: Vec<QueryError> = query
        .seeds
        .iter()
        .filter(|seed| !exists(seed, typed))
        .map(|seed| QueryError::UnknownSeed(seed.clone()))
        .collect();
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(unknown)
    }
}

/// The adjacency over the named morphisms, in the query's direction —
/// built from the instance's stored tables; `None` (dangling) values
/// contribute no edge (`dangling_not_followed`). BTreeMap of BTreeSets:
/// sorted by construction.
fn adjacency(
    direction: Direction,
    schema: &Schema,
    typed: &TypedInstance,
    morphisms: &BTreeSet<Ident>,
) -> BTreeMap<NodeRef, BTreeSet<NodeRef>> {
    let mut adj: BTreeMap<NodeRef, BTreeSet<NodeRef>> = BTreeMap::new();
    for name in morphisms {
        let Some(arrow) = schema.arrows.get(name) else {
            continue;
        };
        let Some(cells) = typed.tables().get(name) else {
            continue;
        };
        for (index, cell) in cells.iter().enumerate() {
            let Some(to) = cell else {
                continue; // dangling: no edge
            };
            let from = NodeRef {
                object: arrow.src.clone(),
                row: RowId(index),
            };
            let to = NodeRef {
                object: arrow.tgt.clone(),
                row: *to,
            };
            let (a, b) = match direction {
                Direction::Forward => (from, to),
                Direction::Backward => (to, from),
            };
            adj.entry(a).or_default().insert(b);
        }
    }
    adj
}

/// Run the closure: the seeds plus everything reachable from them by
/// following the morphism set M (in the query's direction), each node
/// visited at most once (`closure_terminates`, including cyclic M),
/// dangling values never followed, results sorted by construction
/// (`results_ordered`).
pub fn run(
    query: &Query,
    schema: &Schema,
    typed: &TypedInstance,
) -> Result<BTreeSet<NodeRef>, Vec<QueryError>> {
    check(query, schema, typed)?;
    let adj = adjacency(query.direction, schema, typed, &query.morphisms);
    // Worklist closure over the node refs — each node enters the queue
    // at most once (`closure_terminates`).
    let mut seen: BTreeSet<NodeRef> = query.seeds.clone();
    let mut queue: Vec<NodeRef> = seen.iter().cloned().collect();
    while let Some(node) = queue.pop() {
        for next in adj.get(&node).into_iter().flatten().cloned() {
            if seen.insert(next.clone()) {
                queue.push(next);
            }
        }
    }
    Ok(seen)
}

/// Forward closure — what the seeds reach.
pub fn forward_closure(
    schema: &Schema,
    typed: &TypedInstance,
    seeds: &[NodeRef],
    morphisms: &[&str],
) -> Result<BTreeSet<NodeRef>, Vec<QueryError>> {
    run(
        &Query {
            direction: Direction::Forward,
            seeds: seeds.iter().cloned().collect(),
            morphisms: morphisms.iter().map(|m| Ident(m.to_string())).collect(),
        },
        schema,
        typed,
    )
}

/// Backward closure — what reaches the seeds (the dependents half of a
/// blast radius).
pub fn backward_closure(
    schema: &Schema,
    typed: &TypedInstance,
    seeds: &[NodeRef],
    morphisms: &[&str],
) -> Result<BTreeSet<NodeRef>, Vec<QueryError>> {
    run(
        &Query {
            direction: Direction::Backward,
            seeds: seeds.iter().cloned().collect(),
            morphisms: morphisms.iter().map(|m| Ident(m.to_string())).collect(),
        },
        schema,
        typed,
    )
}

/// Closure over an already-validated seed set — the internal step the
/// mixed blast radius iterates (`seeds_exist` is pre-checked by the
/// first `run`, so the seeds are all instance nodes).
fn closure(
    direction: Direction,
    seeds: &BTreeSet<NodeRef>,
    morphisms: &[&str],
    schema: &Schema,
    typed: &TypedInstance,
) -> Result<BTreeSet<NodeRef>, Vec<QueryError>> {
    run(
        &Query {
            direction,
            seeds: seeds.clone(),
            morphisms: morphisms.iter().map(|m| Ident(m.to_string())).collect(),
        },
        schema,
        typed,
    )
}

/// The blast radius of touching `touched`: the mixed transitive closure —
/// from every reached node, BOTH directions are followed, so a
/// dependent's own forward reach joins the radius. Expressed through the
/// two primitives alone: the fixpoint of
/// `X ↦ backward_closure(forward_closure(X))`, which is closed under
/// both directions and contains the seeds — hence exactly the mixed
/// closure. Finite instances make the (monotone) iteration terminate.
/// The donor's parity property pinned this definition: the naive reading
/// (backward of touched ∪ forward of touched) misses the forward reach
/// of backward-reached dependents.
pub fn blast_radius(
    schema: &Schema,
    typed: &TypedInstance,
    touched: &[NodeRef],
    morphisms: &[&str],
) -> Result<BTreeSet<NodeRef>, Vec<QueryError>> {
    let seeds: BTreeSet<NodeRef> = touched.iter().cloned().collect();
    let mut current = closure(Direction::Backward, &seeds, morphisms, schema, typed)?;
    loop {
        let reached = closure(Direction::Forward, &current, morphisms, schema, typed)?;
        let closed = closure(Direction::Backward, &reached, morphisms, schema, typed)?;
        if closed == current {
            return Ok(current);
        }
        current = closed;
    }
}

/// The nodes on at least one cycle through the morphism set M
/// (`closure_terminates`'s counterpart for cycle membership): a node is
/// on a cycle through M iff some defined morphism value `s → t` exists
/// with `s` reachable forward from `t` — including the trivial `s == t`
/// self-edge. Unresolved references are never followed.
pub fn cyclic_nodes(
    schema: &Schema,
    typed: &TypedInstance,
    morphisms: &[&str],
) -> Result<BTreeSet<NodeRef>, Vec<QueryError>> {
    let names: Vec<Ident> = morphisms.iter().map(|m| Ident(m.to_string())).collect();
    let mut cyclic: BTreeSet<NodeRef> = BTreeSet::new();
    for name in &names {
        let Some(arrow) = schema.arrows.get(name) else {
            continue;
        };
        let Some(cells) = typed.tables().get(name) else {
            continue;
        };
        for (index, cell) in cells.iter().enumerate() {
            let Some(to) = cell else {
                continue; // dangling: no edge, no cycle through it
            };
            let from = NodeRef {
                object: arrow.src.clone(),
                row: RowId(index),
            };
            let seed = BTreeSet::from([NodeRef {
                object: arrow.tgt.clone(),
                row: *to,
            }]);
            let reached = closure(Direction::Forward, &seed, morphisms, schema, typed)?;
            if reached.contains(&from) {
                cyclic.insert(from);
            }
        }
    }
    Ok(cyclic)
}

/// `fan_in` — the number of defined morphism values targeting each node
/// (`fan_in_is_preimage_size`), keyed by node. Nodes with no defined
/// preimages carry no entry.
pub fn fan_in(schema: &Schema, typed: &TypedInstance) -> BTreeMap<NodeRef, usize> {
    tally(edges(schema, typed).iter().map(|(_, to)| to.clone()))
}

/// `fan_out` — the number of defined morphism values leaving each node,
/// keyed by node.
pub fn fan_out(schema: &Schema, typed: &TypedInstance) -> BTreeMap<NodeRef, usize> {
    tally(edges(schema, typed).iter().map(|(from, _)| from.clone()))
}

/// Every defined edge of the instance: one pair per resolved morphism
/// cell, across all declared arrows (the donor's `instance.edges()`).
fn edges(schema: &Schema, typed: &TypedInstance) -> Vec<(NodeRef, NodeRef)> {
    let mut edges = Vec::new();
    for (name, arrow) in &schema.arrows {
        let Some(cells) = typed.tables().get(name) else {
            continue;
        };
        for (index, cell) in cells.iter().enumerate() {
            if let Some(to) = cell {
                edges.push((
                    NodeRef {
                        object: arrow.src.clone(),
                        row: RowId(index),
                    },
                    NodeRef {
                        object: arrow.tgt.clone(),
                        row: *to,
                    },
                ));
            }
        }
    }
    edges
}

/// Key occurrence counts in BTree order — the deterministic aggregation
/// behind fan-in/fan-out (no `let mut` accumulators).
fn tally<I: Iterator<Item = NodeRef>>(keys: I) -> BTreeMap<NodeRef, usize> {
    keys.fold(BTreeMap::new(), |mut acc, k| {
        *acc.entry(k).or_default() += 1;
        acc
    })
}
