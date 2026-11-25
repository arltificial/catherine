//! Purpose: the migrate surface's constraint-level signal (bead
//! `catherine-kpn.8`, option (a)) — the dropped-constraint classifier
//! the migrate runner runs over the declared schema map's embedded
//! target schema, split out of the parent [`super`] module's file-line
//! budget.
//!
//! Responsibilities: [`dropped_constraints`] translates the source
//! schema's accepted constraints through the map into the target
//! vocabulary and classifies which of them the target no longer
//! enforces, reusing the parent module's enforced-set comparison
//! ([`super::enforced`], [`super::surplus`]) and bounded separating
//! scan ([`super::diff`]) — the classifier is shared, never
//! duplicated. A dropped constraint carries its witness row: the scan's
//! separating row when the constraint is expressible in the target,
//! the object's first surviving row when the map cannot even carry it
//! over.
//!
//! Rationale: decision catherine-kpn.8 option (a) — migrate reuses the
//! diff classifier and emits the weakened finding with its witness.
//! The machinery lives beside the classifier it calls, so the
//! translate-compare-scan pipeline stays one implementation.

use super::{diff, enforced, EnforcedConstraint};
use crate::bounds::SearchBounds;
use catherine_core::findings::{FindingKind, ViolationWitness};
use catherine_core::model::{
    Arrow, ConstraintStatus, Coproduct, Equation, Ident, Instance, Key, Path, Pullback, RowId,
    Schema, SchemaMap,
};
use std::collections::BTreeMap;

/// One dropped constraint with its migrate witness: the constraint in
/// the source document's own spelling and a row of its domain object
/// that exhibits the drop — the bounded scan's separating row when the
/// constraint is expressible in the target vocabulary, the first
/// surviving row of the object when the map cannot even carry it over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DroppedConstraint {
    /// The dropped constraint, source form.
    pub constraint: EnforcedConstraint,
    /// The constraint's source domain object.
    pub object: Ident,
    /// The witness row on that object.
    pub row: RowId,
}

/// The map's vocabulary translation: the object and arrow images the
/// declared map carries into the target (single-arrow path images only
/// — a `null` assignment or a longer path leaves an arrow, and every
/// constraint through it, untranslatable), plus the source schema's
/// translatable part in the target vocabulary and whether that
/// vocabulary coincides with the target's.
struct Translation<'a> {
    objects: BTreeMap<&'a Ident, &'a Ident>,
    arrows: BTreeMap<&'a Ident, &'a Ident>,
    schema: Schema,
    vocabularies_match: bool,
}

impl<'a> Translation<'a> {
    /// Build the translation from the source schema and its declared
    /// map, comparing the translated vocabulary against the target's.
    fn new(schema: &'a Schema, map: &'a SchemaMap) -> Self {
        let objects = object_images(&map.objects);
        let arrows = arrow_images(&map.arrows);
        let mut translated = Schema::default();
        for image in objects.values() {
            translated.objects.insert((*image).clone(), ());
        }
        for (name, declaration) in &schema.arrows {
            let Some(arrow) = rename_arrow(&objects, &arrows, name, declaration) else {
                continue;
            };
            translated.arrows.insert(arrow.0, arrow.1);
        }
        let target = &map.target;
        let vocabularies_match =
            translated.objects == target.objects && translated.arrows == target.arrows;
        Self {
            objects,
            arrows,
            schema: translated,
            vocabularies_match,
        }
    }

    /// One arrow path's image: `None` when any arrow lacks a single-arrow
    /// image.
    fn path(&self, path: &Path) -> Option<Path> {
        let mut image = Vec::new();
        for arrow in &path.0 {
            image.push((*self.arrows.get(arrow)?).clone());
        }
        Some(Path(image))
    }

    /// Every named arrow's image, in order; `None` when any is missing.
    fn arrows_all(&self, names: &[Ident]) -> Option<Vec<Ident>> {
        names
            .iter()
            .map(|name| self.arrows.get(name).map(|image| (*image).clone()))
            .collect()
    }

    /// A constraint through the map: `Some` when every arrow it names
    /// has a single-arrow image and its object is carried over.
    fn constraint(&self, constraint: &EnforcedConstraint) -> Option<EnforcedConstraint> {
        match constraint {
            EnforcedConstraint::Equation { lhs, rhs } => Some(EnforcedConstraint::Equation {
                lhs: self.path(lhs)?,
                rhs: self.path(rhs)?,
            }),
            EnforcedConstraint::Key { object, arrows } => Some(EnforcedConstraint::Key {
                object: (*self.objects.get(object)?).clone(),
                arrows: self.arrows_all(arrows)?,
            }),
            EnforcedConstraint::Coproduct { object, arrows } => {
                Some(EnforcedConstraint::Coproduct {
                    object: (*self.objects.get(object)?).clone(),
                    arrows: self.arrows_all(arrows)?,
                })
            }
            EnforcedConstraint::Pullback { apex, legs, cospan } => {
                Some(EnforcedConstraint::Pullback {
                    apex: (*self.objects.get(apex)?).clone(),
                    legs: self.arrows_all(legs)?,
                    cospan: self.arrows_all(cospan)?,
                })
            }
        }
    }
}

/// The non-null object assignments of the map as an image table.
fn object_images(side: &BTreeMap<Ident, Option<Ident>>) -> BTreeMap<&Ident, &Ident> {
    side.iter()
        .filter_map(|(source, image)| image.as_ref().map(|image| (source, image)))
        .collect()
}

/// The single-arrow arrow images of the map as an image table: a `null`
/// assignment or a longer path leaves the arrow untranslatable.
fn arrow_images(side: &BTreeMap<Ident, Option<Path>>) -> BTreeMap<&Ident, &Ident> {
    side.iter()
        .filter_map(|(source, path)| match path {
            Some(path) if path.0.len() == 1 => Some((source, &path.0[0])),
            _ => None,
        })
        .collect()
}

/// One renamed arrow declaration: its image name and its retyped
/// `src`/`tgt`, or `None` when an endpoint or the arrow itself is not
/// carried over.
fn rename_arrow<'a>(
    objects: &BTreeMap<&'a Ident, &'a Ident>,
    arrows: &BTreeMap<&'a Ident, &'a Ident>,
    name: &'a Ident,
    declaration: &'a Arrow,
) -> Option<(Ident, Arrow)> {
    let image = (*arrows.get(name)?).clone();
    let src = (*objects.get(&declaration.src)?).clone();
    let tgt = (*objects.get(&declaration.tgt)?).clone();
    Some((image, Arrow { src, tgt }))
}

/// The migrate surface's constraint-level weakening report (decision
/// catherine-kpn.8, option (a)): translate the source schema's accepted
/// constraints through the declared map into the target vocabulary and
/// classify which of them the embedded target schema no longer
/// enforces — the same enforced-set comparison and bounded separating
/// scan [`super::diff`] runs, never a reimplementation. A constraint
/// whose arrows or object the map does not carry into the target (a
/// `null` assignment or a multi-arrow path image) is dropped by
/// structure; a translatable constraint missing from the target's
/// enforced set is dropped with the scan's separating row as witness.
/// A constraint whose domain object carries no source rows stays
/// silent: the constraint is vacuous over the migrated data.
pub fn dropped_constraints(
    schema: &Schema,
    instance: &Instance,
    map: &SchemaMap,
    bound: &SearchBounds,
) -> Vec<DroppedConstraint> {
    let translation = Translation::new(schema, map);
    let enforced_target = enforced(&map.target);
    let inputs = ClassifyInputs {
        enforced_target: &enforced_target,
        instance,
        target: &map.target,
        bound,
    };
    accepted_with_owner(schema)
        .into_iter()
        .filter_map(|(constraint, owner)| classify(&translation, constraint, owner, &inputs))
        .collect()
}

/// The target-side comparison inputs one constraint classification
/// reads: the target's enforced set, the migrated instance, the target
/// schema, and the scan bounds — bundled so the per-constraint helper
/// stays within the parameter budget.
struct ClassifyInputs<'a> {
    enforced_target: &'a BTreeMap<EnforcedConstraint, usize>,
    instance: &'a Instance,
    target: &'a Schema,
    bound: &'a SearchBounds,
}

/// One accepted constraint's verdict: `None` when the target still
/// enforces the translated constraint or no witness row exists; `Some`
/// when the target dropped it — structurally (untranslatable, or the
/// scan finds nothing within bounds) with the object's first row as
/// witness, or through the bounded scan's separating row.
fn classify(
    translation: &Translation<'_>,
    constraint: EnforcedConstraint,
    owner: Ident,
    inputs: &ClassifyInputs<'_>,
) -> Option<DroppedConstraint> {
    let translated = translation.constraint(&constraint);
    let still_enforced = translated
        .as_ref()
        .is_some_and(|c| inputs.enforced_target.get(c).copied().unwrap_or(0) > 0);
    if still_enforced {
        return None;
    }
    let separating = match (&translated, translation.vocabularies_match) {
        (Some(c), true) => separating_row(c, &translation.schema, inputs.target, inputs.bound),
        _ => None,
    };
    let row = separating.or_else(|| first_row(inputs.instance, &owner))?;
    Some(DroppedConstraint {
        constraint,
        object: owner,
        row,
    })
}

/// The bounded scan's separating row for one dropped constraint: add
/// the constraint back to the otherwise-translated v1 and run the same
/// [`super::diff`] classification — the only dropped constraint is the
/// scan's own, so its finding's witness row is the separating witness.
fn separating_row(
    constraint: &EnforcedConstraint,
    translated: &Schema,
    target: &Schema,
    bound: &SearchBounds,
) -> Option<RowId> {
    let mut v1 = translated.clone();
    add_constraint(&mut v1, constraint);
    let finding = diff(&v1, target, bound).findings.into_iter().next()?;
    witness_row(&finding)
}

/// The witness row of a violation finding, per witness variant: an
/// equation's failed row, a key's first colliding row, a coproduct's
/// failed row; other witness shapes name no single row.
fn witness_row(finding: &catherine_core::findings::Finding) -> Option<RowId> {
    match &finding.kind {
        FindingKind::Violation(ViolationWitness::Equation(witness)) => Some(witness.row),
        FindingKind::Violation(ViolationWitness::Key(witness)) => Some(witness.rows[0]),
        FindingKind::Violation(ViolationWitness::Coproduct(witness)) => Some(witness.row),
        _ => None,
    }
}

/// Declare one enforced constraint on a schema (the scan's v1 side).
fn add_constraint(schema: &mut Schema, constraint: &EnforcedConstraint) {
    let accepted = ConstraintStatus::Accepted;
    match constraint {
        EnforcedConstraint::Equation { lhs, rhs } => schema.equations.push(Equation {
            lhs: lhs.clone(),
            rhs: rhs.clone(),
            rationale: String::new(),
            status: accepted,
        }),
        EnforcedConstraint::Key { object, arrows } => schema.keys.push(Key {
            object: object.clone(),
            arrows: arrows.clone(),
            status: accepted,
        }),
        EnforcedConstraint::Coproduct { object, arrows } => schema.coproducts.push(Coproduct {
            object: object.clone(),
            arrows: arrows.clone(),
            status: accepted,
        }),
        EnforcedConstraint::Pullback { apex, legs, cospan } => {
            schema.pullbacks.push(Pullback {
                apex: apex.clone(),
                legs: legs.clone(),
                cospan: cospan.clone(),
                status: accepted,
            });
        }
    }
}

/// Every accepted constraint in canonical order with its source domain
/// object (an equation's left path's source, a key's or coproduct's
/// object, a pullback's apex) — the owner the witness row sits on.
fn accepted_with_owner(schema: &Schema) -> Vec<(EnforcedConstraint, Ident)> {
    let mut accepted = Vec::new();
    for equation in &schema.equations {
        if equation.status == ConstraintStatus::Accepted {
            let owner = equation
                .lhs
                .0
                .first()
                .and_then(|arrow| schema.arrows.get(arrow))
                .map(|declaration| declaration.src.clone());
            if let Some(owner) = owner {
                accepted.push((
                    EnforcedConstraint::Equation {
                        lhs: equation.lhs.clone(),
                        rhs: equation.rhs.clone(),
                    },
                    owner,
                ));
            }
        }
    }
    accepted.extend(relation_constraints(schema));
    accepted
}

/// The accepted pullback constraints, each owned by its apex.
fn pullback_constraints(schema: &Schema) -> Vec<(EnforcedConstraint, Ident)> {
    let mut accepted = Vec::new();
    for pullback in &schema.pullbacks {
        if pullback.status == ConstraintStatus::Accepted {
            accepted.push((
                EnforcedConstraint::Pullback {
                    apex: pullback.apex.clone(),
                    legs: pullback.legs.clone(),
                    cospan: pullback.cospan.clone(),
                },
                pullback.apex.clone(),
            ));
        }
    }
    accepted
}

/// The accepted key, coproduct, and pullback constraints — each with
/// its object (the owner) carried verbatim from the declaration.
fn relation_constraints(schema: &Schema) -> Vec<(EnforcedConstraint, Ident)> {
    let mut accepted = Vec::new();
    for key in &schema.keys {
        if key.status == ConstraintStatus::Accepted {
            accepted.push((
                EnforcedConstraint::Key {
                    object: key.object.clone(),
                    arrows: key.arrows.clone(),
                },
                key.object.clone(),
            ));
        }
    }
    for coproduct in &schema.coproducts {
        if coproduct.status == ConstraintStatus::Accepted {
            accepted.push((
                EnforcedConstraint::Coproduct {
                    object: coproduct.object.clone(),
                    arrows: coproduct.arrows.clone(),
                },
                coproduct.object.clone(),
            ));
        }
    }
    accepted.extend(pullback_constraints(schema));
    accepted
}

/// The first source row of an object, in dense order — the structural
/// witness for a constraint the map cannot express in the target.
fn first_row(instance: &Instance, object: &Ident) -> Option<RowId> {
    instance.rows.get(object)?.first().copied()
}
