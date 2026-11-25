//! The lane flavor's construction: id keys only, no `String`
//! materialization, plain loops only — measured (catherine-6g2):
//! iterator shunts, String sorts, and byte loops each churn the
//! solver to the timeout on their own.

use super::EngineSchema;

#[cfg(kani)]
use super::{
    ident_key, insertion_sort_by, ArrowDirs, Bounded, CompiledCoproduct, CompiledEquation,
    CompiledKey, CompiledPullback, ConstraintStatus, Nid, Step, ARROWS_CAP,
};

/// Sorted (key, declaration position) entries for one id space — ids
/// are assigned in sorted-name order, so id order is name order.
#[cfg(kani)]
fn id_entries(names: &[&str]) -> Vec<(u64, Nid)> {
    let mut entries: Vec<(u64, Nid)> = Vec::with_capacity(names.len());
    for position in 0..names.len() {
        push_entry(&mut entries, (ident_key(names[position]), position));
    }
    insertion_sort_by(&mut entries, |a, b| a.0.cmp(&b.0));
    entries
}

/// Capacity-asserted push for the lane's concrete-bounded vectors (no
/// alloc growth branches reach the solver).
#[cfg(kani)]
fn push_entry(entries: &mut Vec<(u64, Nid)>, entry: (u64, Nid)) {
    assert!(entries.len() < entries.capacity());
    let len = entries.len();
    // SAFETY: the assert proves `len < capacity`; single-element write.
    unsafe {
        entries.as_mut_ptr().add(len).write(entry);
        entries.set_len(len + 1);
    }
}

/// Split sorted entries into the parallel key and order tables.
#[cfg(kani)]
fn split_entries(entries: &[(u64, Nid)]) -> (Vec<u64>, Vec<Nid>) {
    let mut keys = Vec::with_capacity(entries.len());
    let mut order = Vec::with_capacity(entries.len());
    for entry in entries {
        keys.push(entry.0);
        order.push(entry.1);
    }
    (keys, order)
}

/// Linear id-key lookup — the lane's plain-loop resolver (no maps).
#[cfg(kani)]
fn resolve(want: u64, keys: &[u64], order: &[Nid]) -> Nid {
    let mut i = 0;
    while i < keys.len() && keys[i] != want {
        i += 1;
    }
    assert!(i < keys.len());
    order[i]
}

/// The objects' and arrows' sorted key/order tables — ids in sorted-name
/// order, so id order is name order.
#[cfg(kani)]
fn id_tables(
    objects: &[&str],
    arrows: &[(&str, &str, &str)],
) -> (Vec<u64>, Vec<u64>, Vec<Nid>, Vec<Nid>) {
    let (object_keys, object_order) = split_entries(&id_entries(objects));
    let (arrow_keys, arrow_order) = split_entries(&id_entries(
        &arrows.iter().map(|(name, _, _)| *name).collect::<Vec<_>>(),
    ));
    (object_keys, arrow_keys, object_order, arrow_order)
}

/// Per arrow: the resolved source and target object ids, declaration order.
#[cfg(kani)]
fn arrow_endpoints(arrows: &[(&str, &str, &str)], object_of: impl Fn(u64) -> Nid) -> ArrowDirs {
    let src: Vec<Nid> = arrows
        .iter()
        .map(|(_, src, _)| object_of(ident_key(src)))
        .collect();
    let tgt: Vec<Nid> = arrows
        .iter()
        .map(|(_, _, tgt)| object_of(ident_key(tgt)))
        .collect();
    ArrowDirs { src, tgt }
}

/// One compiled path over the lane's id-key resolver.
#[cfg(kani)]
fn lane_path<F: Fn(u64) -> Nid>(names: &[&str], arrow_of: &F) -> Bounded<Step, ARROWS_CAP> {
    let mut steps = Bounded::<Step, ARROWS_CAP>::new();
    for name in names.iter() {
        steps.push(Some(arrow_of(ident_key(name))));
    }
    steps
}

/// The lane's compiled equations, in declaration order; the domain is
/// the lhs lead arrow's source (absent when the path is empty).
#[cfg(kani)]
fn compile_lane_equations<F: Fn(u64) -> Nid>(
    equations: &[(&[&str], &[&str], ConstraintStatus)],
    arrow_of: F,
    dirs: &ArrowDirs,
) -> Vec<CompiledEquation> {
    let mut compiled: Vec<CompiledEquation> = Vec::with_capacity(equations.len());
    for (lhs, rhs, status) in equations {
        let lhs_steps = lane_path(lhs, &arrow_of);
        let rhs_steps = lane_path(rhs, &arrow_of);
        let domain = lhs_steps
            .get(0)
            .and_then(|slot| slot.flatten())
            .map(|lead| dirs.src[lead]);
        compiled.push(CompiledEquation {
            lhs: lhs_steps,
            rhs: rhs_steps,
            domain,
            status: *status,
        });
    }
    compiled
}

/// One declaration's arrows, each resolved and checked against
/// `endpoint` at `object_nid`; `None` on the first arrow outside the
/// object (the walk's skip).
#[cfg(kani)]
fn resolve_arrows_at<F: Fn(u64) -> Nid>(
    names: &[&str],
    object_nid: Nid,
    arrow_of: &F,
    endpoint: &[Nid],
) -> Option<Vec<Nid>> {
    let mut resolved: Vec<Nid> = Vec::with_capacity(names.len());
    for name in names.iter() {
        let nid = arrow_of(ident_key(name));
        if endpoint[nid] != object_nid {
            return None;
        }
        resolved.push(nid);
    }
    Some(resolved)
}

/// The lane's compiled keys: each arrow declared and sourced at the key's
/// object, else a `None` entry with the walk's skip semantics.
#[cfg(kani)]
fn compile_lane_keys<F: Fn(u64) -> Nid, G: Fn(u64) -> Nid>(
    keys: &[(&str, &[&str], ConstraintStatus)],
    object_of: &F,
    arrow_of: &G,
    dirs: &ArrowDirs,
) -> Vec<Option<CompiledKey>> {
    keys.iter()
        .map(|(object, names, status)| {
            let object_nid = object_of(ident_key(object));
            resolve_arrows_at(names, object_nid, arrow_of, &dirs.src).map(|arrow_ids| CompiledKey {
                object: object_nid,
                arrows: arrow_ids,
                status: *status,
            })
        })
        .collect()
}

/// The lane's compiled coproducts: each injection declared and targeting
/// the coproduct object, else a `None` entry with the walk's skip
/// semantics.
#[cfg(kani)]
fn compile_lane_coproducts<F: Fn(u64) -> Nid, G: Fn(u64) -> Nid>(
    coproducts: &[(&str, &[&str], ConstraintStatus)],
    object_of: &F,
    arrow_of: &G,
    dirs: &ArrowDirs,
) -> Vec<Option<CompiledCoproduct>> {
    coproducts
        .iter()
        .map(|(object, names, status)| {
            let object_nid = object_of(ident_key(object));
            resolve_arrows_at(names, object_nid, arrow_of, &dirs.tgt).map(|arrow_ids| {
                CompiledCoproduct {
                    object: object_nid,
                    arrows: arrow_ids,
                    status: *status,
                }
            })
        })
        .collect()
}

/// Resolve two declared arrows through the lane's id-key resolver.
#[cfg(kani)]
fn resolve_two<F: Fn(u64) -> Nid>(names: [&str; 2], arrow_of: &F) -> [Nid; 2] {
    [arrow_of(ident_key(names[0])), arrow_of(ident_key(names[1]))]
}

/// The lane's compiled pullbacks with the square-resolution guards:
/// exactly two legs sourced at the apex, exactly two cospan arrows with
/// agreeing targets; anything else is a `None` entry.
#[cfg(kani)]
fn compile_lane_pullbacks<F: Fn(u64) -> Nid, G: Fn(u64) -> Nid>(
    pullbacks: &[(&str, &[&str], &[&str], ConstraintStatus)],
    object_of: &F,
    arrow_of: &G,
    dirs: &ArrowDirs,
) -> Vec<Option<CompiledPullback>> {
    pullbacks
        .iter()
        .map(|(apex, legs, cospan, status)| {
            if legs.len() != 2 || cospan.len() != 2 {
                return None;
            }
            let [leg1, leg2] = resolve_two([legs[0], legs[1]], arrow_of);
            let [f, g] = resolve_two([cospan[0], cospan[1]], arrow_of);
            let apex_nid = object_of(ident_key(apex));
            let square = dirs.src[leg1] == apex_nid
                && dirs.src[leg2] == apex_nid
                && dirs.tgt[f] == dirs.tgt[g];
            square.then_some(CompiledPullback {
                apex: apex_nid,
                legs: [leg1, leg2],
                cospan: [f, g],
                status: *status,
            })
        })
        .collect()
}

/// The lane presentation's declared shapes — objects, arrows, and the
/// four constraint lists. Field order and names match the old `lane`
/// parameter list.
#[cfg(kani)]
pub struct LaneSpec<'a> {
    pub objects: &'a [&'a str],
    pub arrows: &'a [(&'a str, &'a str, &'a str)],
    pub equations: &'a [(&'a [&'a str], &'a [&'a str], ConstraintStatus)],
    pub keys: &'a [(&'a str, &'a [&'a str], ConstraintStatus)],
    pub coproducts: &'a [(&'a str, &'a [&'a str], ConstraintStatus)],
    pub pullbacks: &'a [(&'a str, &'a [&'a str], &'a [&'a str], ConstraintStatus)],
}

impl EngineSchema {
    /// The lane flavor: id keys only, no `String` materialization. Names
    /// resolve through [`ident_key`] (pure-int identity); collection
    /// loops are plain `for`/`while` so no iterator shunt or comparison
    /// machinery reaches the solver. Constraint entries carry the same
    /// skip semantics as [`EngineSchema::compile`].
    #[cfg(kani)]
    pub fn lane(spec: LaneSpec) -> Self {
        let LaneSpec {
            objects,
            arrows,
            equations,
            keys,
            coproducts,
            pullbacks,
        } = spec;
        let (object_keys, arrow_keys, object_order, arrow_order) = id_tables(objects, arrows);
        let object_of = |want: u64| -> Nid { resolve(want, &object_keys, &object_order) };
        let arrow_of = |want: u64| -> Nid { resolve(want, &arrow_keys, &arrow_order) };
        let endpoints = arrow_endpoints(arrows, object_of);
        let equations = compile_lane_equations(equations, arrow_of, &endpoints);
        let keys = compile_lane_keys(keys, &object_of, &arrow_of, &endpoints);
        let coproducts = compile_lane_coproducts(coproducts, &object_of, &arrow_of, &endpoints);
        let pullbacks = compile_lane_pullbacks(pullbacks, &object_of, &arrow_of, &endpoints);
        Self {
            object_names: Vec::new(),
            arrow_names: Vec::new(),
            object_keys,
            arrow_keys,
            arrows: endpoints,
            equations,
            keys,
            coproducts,
            pullbacks,
        }
    }
}
