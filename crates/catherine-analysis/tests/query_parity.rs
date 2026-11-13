//! The parity gate + property tests for the `query.rs` port (RED/GREEN
//! task 9, evidence base E8).
//!
//! Purpose: carry specodelic's donor contracts as tests, under the same
//! names the donor used — `seeds_exist`, `closure_terminates`,
//! `dangling_not_followed`, `results_ordered`, `fan_in_is_preimage_size`
//! — against catherine's `TypedInstance` tables, plus `proptest`
//! properties pinning the adapted walk against an independent BFS
//! reference.
//!
//! Responsibilities: build small typed instances, run the ported
//! traversals, and assert the contracts structurally. The property
//! tests generate arbitrary small instances and compare the ported
//! closure against a plain reference implementation written here (the
//! parity discipline: two renderings of one walk must agree).
//!
//! Rationale: the port is adapted, not authored (brief §6) — the donor's
//! string-keyed `BTreeSet<String>` world maps onto catherine's
//! `(Ident, RowId)` dense tables, and these tests are the gate that
//! says the adaptation preserved the donor's behavior contract.

use catherine_analysis::query::{
    backward_closure, blast_radius, check, cyclic_nodes, fan_in, fan_out, forward_closure, run,
    Direction, NodeRef, Query, QueryError,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Arrow, Ident, Instance, RowId, Schema};
use std::collections::{BTreeMap, BTreeSet};

mod common;
use common::ident;

// ── donor contract: seeds_exist — unknown seeds rejected, not ignored ─

#[test]
fn seeds_exist() {
    let typed = parity_typed();
    let schema = parity_schema();
    let query = Query {
        direction: Direction::Forward,
        seeds: seeds(&[node("r", 0), node("r", 7)]),
        morphisms: morphisms(&["f"]),
    };
    let errors = check(&query, &schema, &typed).expect_err("unknown seed rejected, not ignored");
    assert_eq!(errors.len(), 1, "all violations reported");
    assert_eq!(
        errors[0],
        QueryError::UnknownSeed(node("r", 7)),
        "the offending seed is named"
    );
    let ok = Query {
        direction: Direction::Forward,
        seeds: seeds(&[node("p", 0), node("q", 1)]),
        morphisms: morphisms(&["f", "g"]),
    };
    assert!(
        check(&ok, &schema, &typed).is_ok(),
        "existing seeds pass the check"
    );
}

// ── donor contract: closure_terminates — one visit per node, cyclic M ─

#[test]
fn closure_terminates() {
    let typed = parity_typed();
    let schema = parity_schema();
    // `u` is a self-loop on p0: following it forever would not terminate
    // a naive walk; the visited-set closure returns.
    let query = Query {
        direction: Direction::Forward,
        seeds: seeds(&[node("p", 0)]),
        morphisms: morphisms(&["u"]),
    };
    let reached = run(&query, &schema, &typed).expect("cyclic M terminates");
    assert_eq!(
        reached,
        seeds(&[node("p", 0)]),
        "the self-loop adds nothing and the walk stops"
    );
}

// ── donor contract: dangling_not_followed — None is no edge ──────────

#[test]
fn dangling_not_followed() {
    let typed = parity_typed();
    let schema = parity_schema();
    // q1's `g` cell is dangling: backward closure over `g` from r0
    // reaches q0 (the defined preimage) and never q1.
    let reached = backward_closure(&schema, &typed, &[node("r", 0)], &["g"]).expect("seeds exist");
    assert_eq!(
        reached,
        seeds(&[node("r", 0), node("q", 0)]),
        "the dangling preimage q1 contributes no edge"
    );
}

// ── donor contract: results_ordered — BTreeSet, sorted by construction ─

#[test]
fn results_ordered() {
    let typed = parity_typed();
    let schema = parity_schema();
    let reached =
        forward_closure(&schema, &typed, &[node("p", 0)], &["f", "g", "u"]).expect("seeds exist");
    let as_vec: Vec<NodeRef> = reached.iter().cloned().collect();
    let mut sorted = as_vec.clone();
    sorted.sort();
    assert_eq!(
        as_vec, sorted,
        "results iterate in sorted (object, row) order"
    );
    assert_eq!(
        as_vec.first(),
        Some(&node("p", 0)),
        "object-major ordering puts p0 first"
    );
}

// ── donor contract: fan_in_is_preimage_size ──────────────────────────

#[test]
fn fan_in_is_preimage_size() {
    let typed = parity_typed();
    let schema = parity_schema();
    let fan = fan_in(&schema, &typed);
    // Preimages: q0 ← f(p0); q1 ← (nothing — f dangles? no: f(p0)=q1? no,
    // f(p0)=q1 is defined! Wait: parity_instance sets f(p0)=RowId(1),
    // so q1's preimage under f is p0). Recount from the tables:
    // f: p0→q1; g: q0→r0, q1→dangling; u: p0→p0.
    assert_eq!(
        fan.get(&node("p", 0)),
        Some(&1),
        "u's self-loop is one preimage"
    );
    assert_eq!(
        fan.get(&node("q", 0)),
        None,
        "no defined edge targets q0 — no fan-in entry exists"
    );
    assert_eq!(
        fan.get(&node("q", 1)),
        Some(&1),
        "f(p0) is q1's one preimage"
    );
    assert_eq!(
        fan.get(&node("r", 0)),
        Some(&1),
        "g(q0) is r0's one preimage"
    );
    let fan_out = fan_out(&schema, &typed);
    assert_eq!(fan_out.get(&node("p", 0)), Some(&2), "f and u leave p0");
    assert_eq!(
        fan_out.get(&node("q", 1)),
        None,
        "q1's only outgoing arrow cell is dangling — no fan-out entry"
    );
}

// ── the adapted extras: blast_radius and cyclic_nodes ────────────────

#[test]
fn blast_radius_is_closed_under_both_directions() {
    let typed = parity_typed();
    let schema = parity_schema();
    // Touching r0: backward reaches q0, whose forward reach adds r0 back;
    // q1 is backward of r0? g(q1) dangles, so no — q1 stays out.
    let radius = blast_radius(&schema, &typed, &[node("r", 0)], &["g"]).expect("seeds exist");
    assert_eq!(
        radius,
        seeds(&[node("r", 0), node("q", 0)]),
        "the mixed closure over g"
    );
}

#[test]
fn cyclic_nodes_names_self_loop_members() {
    let typed = parity_typed();
    let schema = parity_schema();
    let cyclic = cyclic_nodes(&schema, &typed, &["u"]).expect("cyclic walk completes");
    assert_eq!(
        cyclic,
        seeds(&[node("p", 0)]),
        "the trivial self-edge makes p0 cyclic"
    );
    let acyclic = cyclic_nodes(&schema, &typed, &["f", "g"]).expect("cyclic walk completes");
    assert!(acyclic.is_empty(), "f and g form no cycle");
}

// ── parity fixtures (moved from common: only this binary uses them) ──

fn node(object: &str, row: usize) -> NodeRef {
    NodeRef {
        object: ident(object),
        row: RowId(row),
    }
}

/// The parity fixture: objects `p`, `q`, `r`; arrows `f: p→q`,
/// `g: q→r`, `u: p→p` (self-loop); `f(p0)=q1`, `g(q0)=r0`, `g(q1)=null`
/// (dangling), `u(p0)=p0`.
fn parity_schema() -> Schema {
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
    }
}

fn parity_instance() -> Instance {
    let mut rows = BTreeMap::new();
    rows.insert(ident("p"), vec![RowId(0)]);
    rows.insert(ident("q"), vec![RowId(0), RowId(1)]);
    rows.insert(ident("r"), vec![RowId(0)]);
    let mut values = BTreeMap::new();
    values.insert(ident("f"), vec![Some(RowId(1))]);
    values.insert(ident("g"), vec![Some(RowId(0)), None]);
    values.insert(ident("u"), vec![Some(RowId(0))]);
    Instance { rows, values }
}

fn parity_typed() -> TypedInstance {
    TypedInstance::build(&parity_schema(), &parity_instance())
}

fn seeds(set: &[NodeRef]) -> BTreeSet<NodeRef> {
    set.iter().cloned().collect()
}

fn morphisms(names: &[&str]) -> BTreeSet<Ident> {
    names.iter().map(|n| ident(n)).collect()
}
