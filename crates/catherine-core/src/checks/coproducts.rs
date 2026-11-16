//! Purpose: the coproduct half of the executed checks — coverage and
//! disjointness verdicts over a declared coproduct's injection images,
//! split out of `checks.rs` at the module-file gate.
//!
//! Responsibilities: [`coproduct_findings`] walks every declared
//! coproduct and emits one finding per decidably failing row; the
//! complete-image rule keeps coverage undecidable while any injection
//! cell is unresolved but still decides disjointness among complete
//! images; witnesses name only the injections a verdict holds
//! responsible.
//!
//! Rationale: a partial image's unknown target could be any row, so an
//! uncovered-row finding cannot be proved while one exists; two complete
//! images covering the same row is a demonstrated failure independent of
//! the missing cell. The unresolved cell keeps its own dangling finding —
//! never double-reported.

use super::label_for;
use crate::findings::{CoproductWitness, EpistemicLabel, Finding, FindingKind, ViolationWitness};
use crate::instance::TypedInstance;
use crate::model::{Coproduct, Ident, RowId, Schema};
use std::collections::BTreeSet;

/// One injection's image: the coproduct-object rows it covers, named by
/// the injection that covers them.
type InjectionImage = (Ident, BTreeSet<RowId>);

/// Every declared injection's source table, in declared order.
type InjectionTable<'a> = Vec<(Ident, &'a Vec<Option<RowId>>)>;

/// Every declared coproduct: one coverage finding per uncovered row and
/// one disjointness finding per row in two complete injection images. A
/// declaration outside the parsed shape emits nothing. An injection with
/// a dangling/typing-violated cell has a partial image whose unknown
/// target could be any row, so coverage is undecidable for every row and
/// uncovered-row findings stay suppressed (the unresolved cell already
/// carries its own finding — never double-reported); a row in two
/// complete images is still a decidable disjointness violation.
pub(super) fn coproduct_findings(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, coproduct) in schema.coproducts.iter().enumerate() {
        findings.extend(coproduct_declaration(schema, instance, index, coproduct));
    }
    findings
}

/// The per-row verdict of one declaration: coverage when every image is
/// complete, disjointness among complete images otherwise.
type RowVerdict = fn(EpistemicLabel, usize, RowId, &[(Ident, BTreeSet<RowId>)]) -> Option<Finding>;

/// One declaration's findings over its object's rows. With complete
/// images a row covered by no image is uncovered and a row covered by
/// several names every covering injection; with any partial image only
/// the rows in at least two complete images are decidable.
fn coproduct_declaration(
    schema: &Schema,
    instance: &TypedInstance,
    index: usize,
    coproduct: &Coproduct,
) -> Vec<Finding> {
    let Some((images, complete)) = injection_images(schema, instance, coproduct) else {
        return Vec::new();
    };
    let Some(rows) = instance.rows().get(&coproduct.object) else {
        return Vec::new();
    };
    let verdict: RowVerdict = if complete {
        coproduct_row_finding
    } else {
        disjoint_row_finding
    };
    let label = label_for(coproduct.status);
    rows.iter()
        .filter_map(|&row| verdict(label, index, row, &images))
        .collect()
}

/// The declared injections' complete images — each the set of
/// coproduct-object rows an injection covers when every cell resolves —
/// plus whether every declared image is complete. Partial images are
/// absent from the list: their unknown targets could add any row, so they
/// can neither prove coverage nor join a disjointness witness. `None`
/// only when the declaration is outside the parsed presentation's shape.
fn injection_images(
    schema: &Schema,
    instance: &TypedInstance,
    coproduct: &Coproduct,
) -> Option<(Vec<InjectionImage>, bool)> {
    let tables = injection_tables(schema, instance, coproduct)?;
    let images: Vec<_> = tables
        .iter()
        .filter_map(|(arrow, table)| complete_image(arrow, table))
        .collect();
    let complete = images.len() == tables.len();
    Some((images, complete))
}

/// Each declared injection's source table, in declared order. `None` when
/// an arrow is undeclared, targets away from the coproduct object, or has
/// no table — outside the parsed presentation's shape.
fn injection_tables<'a>(
    schema: &Schema,
    instance: &'a TypedInstance,
    coproduct: &Coproduct,
) -> Option<InjectionTable<'a>> {
    let mut tables = Vec::with_capacity(coproduct.arrows.len());
    for arrow in &coproduct.arrows {
        let definition = schema.arrows.get(arrow)?;
        if definition.tgt != coproduct.object {
            return None;
        }
        tables.push((arrow.clone(), instance.tables().get(arrow)?));
    }
    Some(tables)
}

/// One injection's image, or `None` when any cell is unresolved (the
/// image is partial, so its coverage is unknowable).
fn complete_image(arrow: &Ident, table: &[Option<RowId>]) -> Option<(Ident, BTreeSet<RowId>)> {
    let rows: Option<Vec<RowId>> = table.iter().copied().collect();
    rows.map(|rows| (arrow.clone(), rows.into_iter().collect()))
}

/// The declared injections whose image contains `row`, in declared order.
fn covering_injections(images: &[(Ident, BTreeSet<RowId>)], row: RowId) -> Vec<Ident> {
    images
        .iter()
        .filter(|(_, image)| image.contains(&row))
        .map(|(arrow, _)| arrow.clone())
        .collect()
}

/// One row's coverage verdict against complete images: `None` when exactly
/// one image covers it, a witness otherwise (no image = uncovered; several
/// = the covering injections in declared order).
fn coproduct_row_finding(
    label: EpistemicLabel,
    index: usize,
    row: RowId,
    images: &[(Ident, BTreeSet<RowId>)],
) -> Option<Finding> {
    let injections = covering_injections(images, row);
    (injections.len() != 1).then(|| witness_finding(label, index, row, injections))
}

/// One row's disjointness verdict among complete images: a witness when at
/// least two cover it, `None` otherwise — with a partial sibling image
/// present, fewer than two complete covers decides nothing.
fn disjoint_row_finding(
    label: EpistemicLabel,
    index: usize,
    row: RowId,
    images: &[(Ident, BTreeSet<RowId>)],
) -> Option<Finding> {
    let injections = covering_injections(images, row);
    (injections.len() >= 2).then(|| witness_finding(label, index, row, injections))
}

/// The coproduct violation witness: the declaration's index, the row, and
/// every injection the verdict holds responsible.
fn witness_finding(
    label: EpistemicLabel,
    index: usize,
    row: RowId,
    injections: Vec<Ident>,
) -> Finding {
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: index,
            row,
            injections,
        })),
    }
}
