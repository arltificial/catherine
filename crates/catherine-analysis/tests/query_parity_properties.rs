//! The E8 property tests: the ported query traversals against an
//! independent plain-BFS reference, on arbitrary small instances.
//!
//! Purpose: pin the parity discipline as a property, not a fixture —
//! two renderings of one walk must agree on every generated instance.
//!
//! Responsibilities: generate arbitrary small instances over the fixed
//! three-arrow schema, run the ported closure and `fan_in`, and compare
//! against a naive reference implementation defined here.
//!
//! Rationale: split from `query_parity.rs` so the reference walk and the
//! generators live beside their only callers (each test binary compiles
//! `common` separately, so shared-only-by-one-binary helpers cannot sit
//! in `common` without going dead in the other binary).

use catherine_analysis::query::{fan_in, run, Direction, NodeRef, Query};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Arrow, Ident, Instance, Path, RowId, Schema};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

mod common;
use common::ident;

/// Arbitrary small instance over the fixed three-arrow schema.
fn arbitrary_instance() -> impl Strategy<Value = Instance> {
    (
        (0usize..=3, 0usize..=3),
        prop::collection::vec(0usize..=3, 3),
    )
        .prop_map(|((n_p, n_q), targets)| {
            // targets: f(p_i) ∈ {dangling, 0..n_q}; g(q_j) ∈ {dangling, r0};
            // u(p_i) ∈ {dangling, 0..n_p} — encoded as small numbers with
            // 0 meaning dangling.
            let decode = |v: usize, space: usize| {
                if v == 0 {
                    None
                } else if v <= space {
                    Some(RowId(v - 1))
                } else {
                    None
                }
            };
            let f_cells: Vec<Option<RowId>> = (0..n_p).map(|i| decode(targets[i], n_q)).collect();
            let u_cells: Vec<Option<RowId>> =
                (0..n_p).map(|i| decode(targets[i] % 4, n_p)).collect();
            let mut values = BTreeMap::new();
            values.insert(ident("f"), f_cells);
            values.insert(ident("u"), u_cells);
            if n_q > 0 {
                values.insert(
                    ident("g"),
                    (0..n_q).map(|j| decode(targets[(j + 1) % 3], 1)).collect(),
                );
            }
            let mut rows = BTreeMap::new();
            rows.insert(ident("p"), (0..n_p).map(RowId).collect::<Vec<_>>());
            rows.insert(ident("q"), (0..n_q).map(RowId).collect::<Vec<_>>());
            rows.insert(ident("r"), vec![RowId(0)]);
            Instance { rows, values }
        })
}

fn schema_for_properties() -> Schema {
    let mut arrows = BTreeMap::new();
    for (name, src, tgt) in [("f", "p", "q"), ("g", "q", "r"), ("u", "p", "p")] {
        arrows.insert(
            ident(name),
            Arrow {
                src: ident(src),
                tgt: ident(tgt),
            },
        );
    }
    Schema {
        objects: ["p", "q", "r"]
            .iter()
            .map(|o| (ident(o), ()))
            .collect::<BTreeMap<_, _>>(),
        arrows,
        equations: vec![],
    }
}

// ── the independent reference walk ───────────────────────────────────

/// A reference closure: plain BFS over the schema's defined edges.
fn reference_closure(
    direction: Direction,
    schema: &Schema,
    typed: &TypedInstance,
    seeds: &BTreeSet<NodeRef>,
    morphisms: &[Ident],
) -> BTreeSet<NodeRef> {
    let edges = defined_edges(schema, typed, morphisms);
    let mut seen = seeds.clone();
    let mut queue: VecDeque<NodeRef> = seeds.iter().cloned().collect();
    while let Some(node) = queue.pop_front() {
        relax(direction, &edges, &node, &mut seen, &mut queue);
    }
    seen
}

/// The defined edges of the named morphisms: (source, target) pairs for
/// every resolved cell; unknown morphisms and dangling cells contribute
/// nothing.
fn defined_edges(
    schema: &Schema,
    typed: &TypedInstance,
    morphisms: &[Ident],
) -> Vec<(NodeRef, NodeRef)> {
    let mut edges = Vec::new();
    for name in morphisms {
        let Some(arrow) = schema.arrows.get(name) else {
            continue;
        };
        let Some(cells) = typed.tables().get(name) else {
            continue;
        };
        edges.extend(arrow_edges(arrow, cells));
    }
    edges
}

/// The edges of one arrow's table: (src row i) → (tgt cell) for every
/// defined cell.
fn arrow_edges(arrow: &Arrow, cells: &[Option<RowId>]) -> Vec<(NodeRef, NodeRef)> {
    cells
        .iter()
        .enumerate()
        .filter_map(|(i, cell)| {
            cell.map(|to| {
                (
                    NodeRef {
                        object: arrow.src.clone(),
                        row: RowId(i),
                    },
                    NodeRef {
                        object: arrow.tgt.clone(),
                        row: to,
                    },
                )
            })
        })
        .collect()
}

/// One BFS relaxation over every edge at `node`: enqueue each newly
/// seen neighbor.
fn relax(
    direction: Direction,
    edges: &[(NodeRef, NodeRef)],
    node: &NodeRef,
    seen: &mut BTreeSet<NodeRef>,
    queue: &mut VecDeque<NodeRef>,
) {
    for (from, to) in edges {
        let Some(next) = step(direction, from, to, node) else {
            continue;
        };
        if seen.insert(next.clone()) {
            queue.push_back(next);
        }
    }
}

/// The neighbor of `node` along one edge, in the walk's direction.
fn step(direction: Direction, from: &NodeRef, to: &NodeRef, node: &NodeRef) -> Option<NodeRef> {
    match direction {
        Direction::Forward => (from == node).then(|| to.clone()),
        Direction::Backward => (to == node).then(|| from.clone()),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The ported closure equals the reference BFS on arbitrary
    /// instances, contains its seeds, and stays inside the instance's
    /// node set — closure terminates and is exact.
    #[test]
    fn ported_closure_matches_the_reference_bfs(
        instance in arbitrary_instance(),
        forward in proptest::bool::ANY,
    ) {
        let schema = schema_for_properties();
        let typed = TypedInstance::build(&schema, &instance);
        let all_seeds: BTreeSet<NodeRef> = typed
            .rows()
            .iter()
            .flat_map(|(object, rows)| {
                rows.iter()
                    .map(move |&row| NodeRef { object: object.clone(), row })
            })
            .collect();
        let direction = if forward { Direction::Forward } else { Direction::Backward };
        let query = Query {
            direction,
            seeds: all_seeds.clone(),
            morphisms: ["f", "g", "u"].iter().map(|n| ident(n)).collect(),
        };
        let ported = run(&query, &schema, &typed).expect("all seeds exist");
        let reference = reference_closure(direction, &schema, &typed, &all_seeds, &[
            ident("f"),
            ident("g"),
            ident("u"),
        ]);
        prop_assert_eq!(ported.clone(), reference, "the port matches the reference walk");
        prop_assert!(all_seeds.is_subset(&ported), "the closure contains its seeds");
        prop_assert!(ported.is_subset(&all_seeds), "the closure stays inside the instance");
    }

    /// `fan_in` equals the independently recounted preimage size on
    /// arbitrary instances (`fan_in_is_preimage_size` as a property).
    #[test]
    fn fan_in_counts_preimages_on_arbitrary_instances(instance in arbitrary_instance()) {
        let schema = schema_for_properties();
        let typed = TypedInstance::build(&schema, &instance);
        let fan = fan_in(&schema, &typed);
        let mut expected: BTreeMap<NodeRef, usize> = BTreeMap::new();
        for (name, arrow) in &schema.arrows {
            if let Some(cells) = typed.tables().get(name) {
                for to in cells.iter().flatten() {
                    let target = NodeRef {
                        object: arrow.tgt.clone(),
                        row: *to,
                    };
                    *expected.entry(target).or_default() += 1;
                }
            }
        }
        let _ = Path(vec![]); // keep the import honest; paths are rewrite-world
        prop_assert_eq!(fan, expected, "fan_in is exactly the preimage tally");
    }
}
