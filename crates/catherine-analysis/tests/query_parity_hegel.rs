//! Purpose: pin the E8 parity discipline as a property, not a fixture —
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
use catherine_core::model::{Arrow, Ident, Instance, RowId, Schema};
use hegel::generators as gs;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

mod common;
use common::ident;

/// The drawn row counts and shared target codes of one arbitrary
/// instance.
struct InstanceShape {
    n_p: usize,
    n_q: usize,
    targets: Vec<usize>,
}

/// Draw one instance's row counts and its three small target codes.
fn draw_instance_shape(tc: &hegel::TestCase) -> InstanceShape {
    let n_p = tc.draw(gs::integers::<usize>().min_value(0).max_value(3));
    let n_q = tc.draw(gs::integers::<usize>().min_value(0).max_value(3));
    let targets: Vec<usize> = tc.draw(
        gs::vecs(gs::integers::<usize>().min_value(0).max_value(3))
            .min_size(3)
            .max_size(3),
    );
    InstanceShape { n_p, n_q, targets }
}

// targets: f(p_i) ∈ {dangling, 0..n_q}; g(q_j) ∈ {dangling, r0};
// u(p_i) ∈ {dangling, 0..n_p} — encoded as small numbers with
// 0 meaning dangling.
/// Decode one target code: `0` is dangling, `1..=space` is a row.
fn decode_target(v: usize, space: usize) -> Option<RowId> {
    if v == 0 {
        None
    } else if v <= space {
        Some(RowId(v - 1))
    } else {
        None
    }
}

/// The `f`/`g`/`u` cell tables decoded from the drawn target codes.
fn instance_values(shape: &InstanceShape) -> BTreeMap<Ident, Vec<Option<RowId>>> {
    let f_cells: Vec<Option<RowId>> = (0..shape.n_p)
        .map(|i| decode_target(shape.targets[i], shape.n_q))
        .collect();
    let u_cells: Vec<Option<RowId>> = (0..shape.n_p)
        .map(|i| decode_target(shape.targets[i] % 4, shape.n_p))
        .collect();
    let mut values = BTreeMap::new();
    values.insert(ident("f"), f_cells);
    values.insert(ident("u"), u_cells);
    if shape.n_q > 0 {
        values.insert(
            ident("g"),
            (0..shape.n_q)
                .map(|j| decode_target(shape.targets[(j + 1) % 3], 1))
                .collect(),
        );
    }
    values
}

/// The `p`/`q`/`r` row spaces of one instance.
fn instance_rows(shape: &InstanceShape) -> BTreeMap<Ident, Vec<RowId>> {
    let mut rows = BTreeMap::new();
    rows.insert(ident("p"), (0..shape.n_p).map(RowId).collect::<Vec<_>>());
    rows.insert(ident("q"), (0..shape.n_q).map(RowId).collect::<Vec<_>>());
    rows.insert(ident("r"), vec![RowId(0)]);
    rows
}

/// Arbitrary small instance over the fixed three-arrow schema: up to
/// three `p` rows and three `q` rows, one `r` row, and three small
/// target codes shared by the `f`/`g`/`u` tables.
#[hegel::composite]
fn arbitrary_instance(tc: &hegel::TestCase) -> Instance {
    let shape = draw_instance_shape(tc);
    let values = instance_values(&shape);
    let rows = instance_rows(&shape);
    Instance { rows, values }
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
        keys: Vec::new(),
        coproducts: Vec::new(),
        pullbacks: Vec::new(),
        stored: BTreeMap::new(),
    }
}

// ── the independent reference walk ───────────────────────────────────

/// The instance view the reference walk traverses: the schema, its typed
/// instance, and the morphisms the walk may follow.
struct WalkView<'a> {
    /// The fixed three-arrow schema.
    schema: &'a Schema,
    /// The instance's cells resolved against the schema.
    typed: &'a TypedInstance,
    /// The morphisms the walk may traverse.
    morphisms: &'a [Ident],
}

/// Everything one reference BFS reads: the walk's direction, the instance
/// view it traverses, and its seed nodes.
struct ReferenceWalk<'a> {
    /// The direction the closure follows.
    direction: Direction,
    /// The schema/instance/morphisms view the walk reads.
    view: WalkView<'a>,
    /// The nodes the closure starts from.
    seeds: &'a BTreeSet<NodeRef>,
}

/// The BFS frontier: every node already reached, and the expansion queue.
struct Frontier {
    /// Every node the walk has reached, seeds included.
    seen: BTreeSet<NodeRef>,
    /// The nodes awaiting expansion, in reach order.
    queue: VecDeque<NodeRef>,
}

/// One expansion step: the walk's direction, its edge list, and the node
/// being expanded.
struct Walk<'a> {
    /// The direction the closure follows.
    direction: Direction,
    /// The defined `(source, target)` edges of the traversable morphisms.
    edges: &'a [(NodeRef, NodeRef)],
    /// The node being expanded.
    node: &'a NodeRef,
}

impl Walk<'_> {
    /// The neighbor of the walk's node along one edge, in the walk's
    /// direction.
    fn neighbor(&self, edge: &(NodeRef, NodeRef)) -> Option<NodeRef> {
        let (from, to) = edge;
        match self.direction {
            Direction::Forward => (from == self.node).then(|| to.clone()),
            Direction::Backward => (to == self.node).then(|| from.clone()),
        }
    }
}

/// A reference closure: plain BFS over the schema's defined edges.
fn reference_closure(walk: &ReferenceWalk<'_>) -> BTreeSet<NodeRef> {
    let edges = defined_edges(&walk.view);
    let mut frontier = Frontier {
        seen: walk.seeds.clone(),
        queue: walk.seeds.iter().cloned().collect(),
    };
    while let Some(node) = frontier.queue.pop_front() {
        relax(
            &Walk {
                direction: walk.direction,
                edges: &edges,
                node: &node,
            },
            &mut frontier,
        );
    }
    frontier.seen
}

/// The defined edges of the named morphisms: (source, target) pairs for
/// every resolved cell; unknown morphisms and dangling cells contribute
/// nothing.
fn defined_edges(view: &WalkView<'_>) -> Vec<(NodeRef, NodeRef)> {
    let mut edges = Vec::new();
    for name in view.morphisms {
        let Some(arrow) = view.schema.arrows.get(name) else {
            continue;
        };
        let Some(cells) = view.typed.tables().get(name) else {
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

/// One BFS relaxation over every edge at the walk's node: enqueue each
/// newly seen neighbor into the frontier.
fn relax(walk: &Walk<'_>, frontier: &mut Frontier) {
    for edge in walk.edges {
        let Some(next) = walk.neighbor(edge) else {
            continue;
        };
        if frontier.seen.insert(next.clone()) {
            frontier.queue.push_back(next);
        }
    }
}

/// The ported closure equals the reference BFS on arbitrary
/// instances, contains its seeds, and stays inside the instance's
/// node set — closure terminates and is exact.
#[hegel::test]
fn ported_closure_matches_the_reference_bfs(tc: hegel::TestCase) {
    let instance: Instance = tc.draw(arbitrary_instance());
    let forward = tc.draw(gs::booleans());
    tc.note(&format!("instance = {instance:?}, forward = {forward}"));
    let schema = schema_for_properties();
    let typed = TypedInstance::build(&schema, &instance);
    let all_seeds: BTreeSet<NodeRef> = typed
        .rows()
        .iter()
        .flat_map(|(object, rows)| {
            rows.iter().map(move |&row| NodeRef {
                object: object.clone(),
                row,
            })
        })
        .collect();
    let direction = if forward {
        Direction::Forward
    } else {
        Direction::Backward
    };
    let query = Query {
        direction,
        seeds: all_seeds.clone(),
        morphisms: ["f", "g", "u"].iter().map(|n| ident(n)).collect(),
    };
    let ported = run(&query, &schema, &typed).expect("all seeds exist");
    let reference = reference_closure(&ReferenceWalk {
        direction,
        view: WalkView {
            schema: &schema,
            typed: &typed,
            morphisms: &[ident("f"), ident("g"), ident("u")],
        },
        seeds: &all_seeds,
    });
    assert_eq!(
        ported.clone(),
        reference,
        "the port matches the reference walk"
    );
    assert!(
        all_seeds.is_subset(&ported),
        "the closure contains its seeds"
    );
    assert!(
        ported.is_subset(&all_seeds),
        "the closure stays inside the instance"
    );
}

/// The exact preimage tally `fan_in` is asserted against: every defined
/// cell of every arrow counts once toward its target node.
fn expected_preimages(schema: &Schema, typed: &TypedInstance) -> BTreeMap<NodeRef, usize> {
    let mut expected: BTreeMap<NodeRef, usize> = BTreeMap::new();
    for (name, arrow) in &schema.arrows {
        let Some(cells) = typed.tables().get(name) else {
            continue;
        };
        for to in cells.iter().flatten() {
            let target = NodeRef {
                object: arrow.tgt.clone(),
                row: *to,
            };
            *expected.entry(target).or_default() += 1;
        }
    }
    expected
}

/// `fan_in` equals the independently recounted preimage size on
/// arbitrary instances (`fan_in_is_preimage_size` as a property).
#[hegel::test]
fn fan_in_counts_preimages_on_arbitrary_instances(tc: hegel::TestCase) {
    let instance: Instance = tc.draw(arbitrary_instance());
    tc.note(&format!("instance = {instance:?}"));
    let schema = schema_for_properties();
    let typed = TypedInstance::build(&schema, &instance);
    let fan = fan_in(&schema, &typed);
    let expected = expected_preimages(&schema, &typed);
    assert_eq!(fan, expected, "fan_in is exactly the preimage tally");
}
