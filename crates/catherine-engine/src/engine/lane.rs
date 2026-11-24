//! The lane flavor's construction: id keys only, no `String`
//! materialization, plain loops only — measured (catherine-6g2):
//! iterator shunts, String sorts, and byte loops each churn the
//! solver to the timeout on their own.

use super::EngineSchema;

#[cfg(kani)]
use super::{
    ident_key, insertion_sort_by, Bounded, CompiledCoproduct, CompiledEquation, CompiledKey,
    CompiledPullback, ConstraintStatus, Nid, Step, ARROWS_CAP,
};

/// Sorted (key, declaration position) entries for one id space — ids
/// are assigned in sorted-name order, so id order is name order.
#[cfg(kani)]
fn id_entries(names: &[&str]) -> Vec<(u64, Nid)> {
    let mut entries: Vec<(u64, Nid)> = Vec::with_capacity(names.len());
    let mut position = 0;
    while position < names.len() {
        push_entry(&mut entries, (ident_key(names[position]), position));
        position += 1;
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

/// The lane's compiled equations, in declaration order; the domain is
/// the lhs lead arrow's source (absent when the path is empty).
#[cfg(kani)]
fn compile_lane_equations(
    equations: &[(&[&str], &[&str], ConstraintStatus)],
    arrow_of: impl Fn(u64) -> Nid,
    arrow_src: &[Nid],
) -> Vec<CompiledEquation> {
    let mut compiled: Vec<CompiledEquation> = Vec::with_capacity(equations.len());
    for (lhs, rhs, status) in equations {
        let mut lhs_steps = Bounded::<Step, ARROWS_CAP>::new();
        for name in lhs.iter() {
            lhs_steps.push(Some(arrow_of(ident_key(name))));
        }
        let mut rhs_steps = Bounded::<Step, ARROWS_CAP>::new();
        for name in rhs.iter() {
            rhs_steps.push(Some(arrow_of(ident_key(name))));
        }
        let mut domain: Option<Nid> = None;
        if let Some(lead) = lhs_steps.get(0).and_then(|slot| slot.flatten()) {
            domain = Some(arrow_src[lead]);
        }
        compiled.push(CompiledEquation {
            lhs: lhs_steps,
            rhs: rhs_steps,
            domain,
            status: *status,
        });
    }
    compiled
}

/// The lane's compiled keys: each arrow declared and sourced at the key's
/// object, else a `None` entry with the walk's skip semantics.
#[cfg(kani)]
fn compile_lane_keys(
    keys: &[(&str, &[&str], ConstraintStatus)],
    object_of: impl Fn(u64) -> Nid,
    arrow_of: impl Fn(u64) -> Nid,
    arrow_src: &[Nid],
) -> Vec<Option<CompiledKey>> {
    let mut compiled: Vec<Option<CompiledKey>> = Vec::with_capacity(keys.len());
    for (object, names, status) in keys {
        let object_nid = object_of(ident_key(object));
        let mut resolved: Vec<Nid> = Vec::with_capacity(names.len());
        let mut applicable = true;
        for name in names.iter() {
            let nid = arrow_of(ident_key(name));
            if arrow_src[nid] == object_nid {
                resolved.push(nid);
            } else {
                applicable = false;
                break;
            }
        }
        if applicable {
            compiled.push(Some(CompiledKey {
                object: object_nid,
                arrows: resolved,
                status: *status,
            }));
        } else {
            compiled.push(None);
        }
    }
    compiled
}

/// The lane's compiled coproducts: each injection declared and targeting
/// the coproduct object, else a `None` entry with the walk's skip
/// semantics.
#[cfg(kani)]
fn compile_lane_coproducts(
    coproducts: &[(&str, &[&str], ConstraintStatus)],
    object_of: impl Fn(u64) -> Nid,
    arrow_of: impl Fn(u64) -> Nid,
    arrow_tgt: &[Nid],
) -> Vec<Option<CompiledCoproduct>> {
    let mut compiled: Vec<Option<CompiledCoproduct>> = Vec::with_capacity(coproducts.len());
    for (object, names, status) in coproducts {
        let object_nid = object_of(ident_key(object));
        let mut resolved: Vec<Nid> = Vec::with_capacity(names.len());
        let mut applicable = true;
        for name in names.iter() {
            let nid = arrow_of(ident_key(name));
            if arrow_tgt[nid] == object_nid {
                resolved.push(nid);
            } else {
                applicable = false;
                break;
            }
        }
        if applicable {
            compiled.push(Some(CompiledCoproduct {
                object: object_nid,
                arrows: resolved,
                status: *status,
            }));
        } else {
            compiled.push(None);
        }
    }
    compiled
}

/// The lane's compiled pullbacks with the square-resolution guards:
/// exactly two legs sourced at the apex, exactly two cospan arrows with
/// agreeing targets; anything else is a `None` entry.
#[cfg(kani)]
fn compile_lane_pullbacks(
    pullbacks: &[(&str, &[&str], &[&str], ConstraintStatus)],
    object_of: impl Fn(u64) -> Nid,
    arrow_of: impl Fn(u64) -> Nid,
    arrow_src: &[Nid],
    arrow_tgt: &[Nid],
) -> Vec<Option<CompiledPullback>> {
    let mut compiled: Vec<Option<CompiledPullback>> = Vec::with_capacity(pullbacks.len());
    for (apex, legs, cospan, status) in pullbacks {
        let apex_nid = object_of(ident_key(apex));
        if legs.len() != 2 || cospan.len() != 2 {
            compiled.push(None);
            continue;
        }
        let leg1 = arrow_of(ident_key(legs[0]));
        let leg2 = arrow_of(ident_key(legs[1]));
        let f = arrow_of(ident_key(cospan[0]));
        let g = arrow_of(ident_key(cospan[1]));
        if arrow_src[leg1] != apex_nid
            || arrow_src[leg2] != apex_nid
            || arrow_tgt[f] != arrow_tgt[g]
        {
            compiled.push(None);
            continue;
        }
        compiled.push(Some(CompiledPullback {
            apex: apex_nid,
            legs: [leg1, leg2],
            cospan: [f, g],
            status: *status,
        }));
    }
    compiled
}

impl EngineSchema {
    /// The lane flavor: id keys only, no `String` materialization. Names
    /// resolve through [`ident_key`] (pure-int identity); collection
    /// loops are plain `for`/`while` so no iterator shunt or comparison
    /// machinery reaches the solver. Constraint entries carry the same
    /// skip semantics as [`EngineSchema::compile`].
    #[cfg(kani)]
    pub fn lane(
        objects: &[&str],
        arrows: &[(&str, &str, &str)],
        equations: &[(&[&str], &[&str], ConstraintStatus)],
        keys: &[(&str, &[&str], ConstraintStatus)],
        coproducts: &[(&str, &[&str], ConstraintStatus)],
        pullbacks: &[(&str, &[&str], &[&str], ConstraintStatus)],
    ) -> Self {
        let (object_keys, object_order) = split_entries(&id_entries(objects));
        let (arrow_keys, arrow_order) = split_entries(&id_entries(
            &arrows.iter().map(|(name, _, _)| *name).collect::<Vec<_>>(),
        ));
        let object_of = |want: u64| -> Nid { resolve(want, &object_keys, &object_order) };
        let arrow_of = |want: u64| -> Nid { resolve(want, &arrow_keys, &arrow_order) };
        let mut arrow_src: Vec<Nid> = Vec::with_capacity(arrow_keys.len());
        let mut arrow_tgt: Vec<Nid> = Vec::with_capacity(arrow_keys.len());
        for (name, src, tgt) in arrows {
            let _ = name;
            arrow_src.push(object_of(ident_key(src)));
            arrow_tgt.push(object_of(ident_key(tgt)));
        }
        let compiled_equations = compile_lane_equations(equations, arrow_of, &arrow_src);
        let compiled_keys = compile_lane_keys(keys, object_of, arrow_of, &arrow_src);
        let compiled_coproducts =
            compile_lane_coproducts(coproducts, object_of, arrow_of, &arrow_tgt);
        let compiled_pullbacks =
            compile_lane_pullbacks(pullbacks, object_of, arrow_of, &arrow_src, &arrow_tgt);
        Self {
            object_names: Vec::new(),
            arrow_names: Vec::new(),
            object_keys,
            arrow_keys,
            arrow_src,
            arrow_tgt,
            equations: compiled_equations,
            keys: compiled_keys,
            coproducts: compiled_coproducts,
            pullbacks: compiled_pullbacks,
        }
    }
}
