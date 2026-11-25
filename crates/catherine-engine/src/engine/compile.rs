//! Schema interning: the parsed flavor (owned canonical names), the
//! lane flavor (id keys only), and the parsed adapters that copy
//! presentation data into id tables once per call.

use std::collections::HashMap;

use super::{
    ArrowDirs, Bounded, CompiledCoproduct, CompiledEquation, CompiledKey, CompiledPullback,
    EngineInstance, EngineSchema, Nid, Step, ARROWS_CAP,
};
use crate::model::{Ident, Instance, Path, Schema};

/// Assign dense ids to a parsed table's names, in canonical order.
fn intern_names(names: impl Iterator<Item = Ident>) -> HashMap<Ident, Nid> {
    names.enumerate().map(|(id, name)| (name, id)).collect()
}

/// Per arrow: the resolved source and target object ids, declaration order.
fn arrow_endpoints(schema: &Schema, objects: &HashMap<Ident, Nid>) -> ArrowDirs {
    let mut src = Vec::with_capacity(schema.arrows.len());
    let mut tgt = Vec::with_capacity(schema.arrows.len());
    for arrow in schema.arrows.values() {
        src.push(objects[&arrow.src]);
        tgt.push(objects[&arrow.tgt]);
    }
    ArrowDirs { src, tgt }
}

/// Every compiled constraint list, in declaration order.
pub(super) type Constraints = (
    Vec<CompiledEquation>,
    Vec<Option<CompiledKey>>,
    Vec<Option<CompiledCoproduct>>,
    Vec<Option<CompiledPullback>>,
);

impl EngineSchema {
    /// Intern a parsed schema (the parsed flavor, owned names). Ids are
    /// assigned by iterating the parsed `BTreeMap`s in their canonical
    /// order, so id order is name order. Concrete data only.
    pub fn compile(schema: &Schema) -> Self {
        let object_names: Vec<Ident> = schema.objects.keys().cloned().collect();
        let arrow_names: Vec<Ident> = schema.arrows.keys().cloned().collect();
        let objects = intern_names(object_names.iter().cloned());
        let arrows = intern_names(arrow_names.iter().cloned());
        let endpoints = arrow_endpoints(schema, &objects);
        let (equations, keys, coproducts, pullbacks) =
            compile_constraints(&objects, &arrows, &endpoints, schema);
        Self {
            object_names,
            arrow_names,
            #[cfg(kani)]
            object_keys: Vec::new(),
            #[cfg(kani)]
            arrow_keys: Vec::new(),
            arrows: endpoints,
            equations,
            keys,
            coproducts,
            pullbacks,
        }
    }

    /// The canonical id of a name; `None` outside the presentation.
    pub fn arrow_nid(&self, name: &Ident) -> Option<Nid> {
        self.arrow_names.iter().position(|n| n == name)
    }

    /// Intern an instance against this presentation: dense row counts per
    /// object and one cell vector per arrow, copied out of the parsed
    /// tables once.
    pub fn compile_instance(&self, instance: &Instance) -> EngineInstance {
        let row_counts = self
            .object_names
            .iter()
            .map(|name| instance.rows.get(name).map(Vec::len))
            .collect();
        let cells = self
            .arrow_names
            .iter()
            .map(|name| {
                instance
                    .values
                    .get(name)
                    .cloned()
                    .map(Bounded::from_option_vec)
            })
            .collect();
        EngineInstance { row_counts, cells }
    }
}

/// The name's byte order as a u64 key: up to 8 bytes big-endian,
/// zero-padded — byte order for names that fit, which every lane shape
/// does. Pure-int identity so no solver time goes to String compares or
/// byte loops (catherine-6g2: the memcmp model churns to the unwind cap
/// even on concrete bytes; a byte-copy loop unrolls every call site).
#[cfg(kani)]
pub const fn ident_key(name: &str) -> u64 {
    let bytes = name.as_bytes();
    assert!(bytes.len() <= 8);
    let b0 = if bytes.len() > 0 { bytes[0] } else { 0 } as u64;
    let b1 = if bytes.len() > 1 { bytes[1] } else { 0 } as u64;
    let b2 = if bytes.len() > 2 { bytes[2] } else { 0 } as u64;
    let b3 = if bytes.len() > 3 { bytes[3] } else { 0 } as u64;
    let b4 = if bytes.len() > 4 { bytes[4] } else { 0 } as u64;
    let b5 = if bytes.len() > 5 { bytes[5] } else { 0 } as u64;
    let b6 = if bytes.len() > 6 { bytes[6] } else { 0 } as u64;
    let b7 = if bytes.len() > 7 { bytes[7] } else { 0 } as u64;
    (b0 << 56) | (b1 << 48) | (b2 << 40) | (b3 << 32) | (b4 << 24) | (b5 << 16) | (b6 << 8) | b7
}

/// Resolve a parsed path to arrow ids; `None` steps stay unresolvable
/// (compose returns nothing past them), mirroring the walk's get-based
/// totality. The path is a bounded table ([`Bounded`]): array-backed
/// under `cfg(kani)` (`ARROWS_CAP` is a proof obligation discharged by
/// the declared shapes' path lengths), `Vec`-backed and arbitrary-sized
/// in production.
fn compile_path(arrows: &HashMap<Ident, Nid>, path: &Path) -> Bounded<Step, ARROWS_CAP> {
    let mut steps = Bounded::<Step, ARROWS_CAP>::new();
    for name in &path.0 {
        steps.push(arrows.get(name).copied());
    }
    steps
}

/// Compile one key with the walk's guards: every arrow declared and
/// sourced at the key's object.
fn compile_key(
    objects: &HashMap<Ident, Nid>,
    arrows: &HashMap<Ident, Nid>,
    dirs: &ArrowDirs,
    key: &crate::model::Key,
) -> Option<CompiledKey> {
    let object = objects.get(&key.object).copied()?;
    let mut arrow_ids: Vec<Nid> = Vec::with_capacity(key.arrows.len());
    for arrow in &key.arrows {
        let nid = arrows.get(arrow).copied()?;
        if dirs.src[nid] != object {
            return None;
        }
        arrow_ids.push(nid);
    }
    Some(CompiledKey {
        object,
        arrows: arrow_ids,
        status: key.status,
    })
}

/// Compile one coproduct with the walk's guards: every injection declared
/// and targeting the coproduct object.
fn compile_coproduct(
    objects: &HashMap<Ident, Nid>,
    arrows: &HashMap<Ident, Nid>,
    dirs: &ArrowDirs,
    coproduct: &crate::model::Coproduct,
) -> Option<CompiledCoproduct> {
    let object = objects.get(&coproduct.object).copied()?;
    let mut arrow_ids: Vec<Nid> = Vec::with_capacity(coproduct.arrows.len());
    for arrow in &coproduct.arrows {
        let nid = arrows.get(arrow).copied()?;
        if dirs.tgt[nid] != object {
            return None;
        }
        arrow_ids.push(nid);
    }
    Some(CompiledCoproduct {
        object,
        arrows: arrow_ids,
        status: coproduct.status,
    })
}

/// Resolve two declared arrows, in order; `None` when either is
/// undeclared.
fn resolve_pair(arrows: &HashMap<Ident, Nid>, a: &Ident, b: &Ident) -> Option<(Nid, Nid)> {
    Some((arrows.get(a).copied()?, arrows.get(b).copied()?))
}

/// Compile one pullback with the square-resolution guards: exactly two
/// legs sourced at the apex, exactly two cospan arrows with agreeing
/// targets, all declared.
fn compile_pullback(
    objects: &HashMap<Ident, Nid>,
    arrows: &HashMap<Ident, Nid>,
    dirs: &ArrowDirs,
    pullback: &crate::model::Pullback,
) -> Option<CompiledPullback> {
    let [p1, p2] = pullback.legs.as_slice() else {
        return None;
    };
    let [f, g] = pullback.cospan.as_slice() else {
        return None;
    };
    let apex = objects.get(&pullback.apex).copied()?;
    let (p1, p2) = resolve_pair(arrows, p1, p2)?;
    let (f, g) = resolve_pair(arrows, f, g)?;
    if dirs.src[p1] != apex || dirs.src[p2] != apex {
        return None;
    }
    if dirs.tgt[f] != dirs.tgt[g] {
        return None;
    }
    Some(CompiledPullback {
        apex,
        legs: [p1, p2],
        cospan: [f, g],
        status: pullback.status,
    })
}

/// The compiled equations, in declaration order; every lhs/rhs path
/// compiled with undeclared arrows carried as `None` steps, the domain
/// from the lhs lead arrow (absent when the path is empty).
fn compile_equations(
    arrows: &HashMap<Ident, Nid>,
    dirs: &ArrowDirs,
    schema: &Schema,
) -> Vec<CompiledEquation> {
    let mut equations = Vec::with_capacity(schema.equations.len());
    for equation in &schema.equations {
        let lhs = compile_path(arrows, &equation.lhs);
        let rhs = compile_path(arrows, &equation.rhs);
        let domain = lhs
            .get(0)
            .and_then(|slot| slot.flatten())
            .map(|lead| dirs.src[lead]);
        equations.push(CompiledEquation {
            lhs,
            rhs,
            domain,
            status: equation.status,
        });
    }
    equations
}

/// Compile every declared constraint list — equations, keys,
/// coproducts, pullbacks — with the walk's skip guards. Split out of
/// `compile` to keep both under the complexity caps.
fn compile_constraints(
    objects: &HashMap<Ident, Nid>,
    arrows: &HashMap<Ident, Nid>,
    dirs: &ArrowDirs,
    schema: &Schema,
) -> Constraints {
    let equations = compile_equations(arrows, dirs, schema);
    let mut keys = Vec::with_capacity(schema.keys.len());
    for key in &schema.keys {
        keys.push(compile_key(objects, arrows, dirs, key));
    }
    let mut coproducts = Vec::with_capacity(schema.coproducts.len());
    for coproduct in &schema.coproducts {
        coproducts.push(compile_coproduct(objects, arrows, dirs, coproduct));
    }
    let mut pullbacks = Vec::with_capacity(schema.pullbacks.len());
    for pullback in &schema.pullbacks {
        pullbacks.push(compile_pullback(objects, arrows, dirs, pullback));
    }
    (equations, keys, coproducts, pullbacks)
}
