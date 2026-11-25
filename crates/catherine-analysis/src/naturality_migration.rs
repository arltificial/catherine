//! Purpose: the declared Δ-migration commutation check (catherine-kw9.2) —
//! verify that migrate-then-operate equals operate-then-migrate: for
//! every source operation `o` and its Δ target path `m`, `migrate ∘ o`
//! must equal `m ∘ migrate` on every migrated row
//! ([`migration_naturality`]), each breaking square reported as a
//! witness-bearing finding. Split out of `naturality.rs` to keep both
//! modules inside the file-size gate; the public surface stays on
//! `crate::naturality` via re-exports.
//!
//! Responsibilities: require the source Δ schema map and instance map (an
//! absent one is a caller concern, reported as [`MigrationError`]),
//! resolve every migrated row against the migrated instance at
//! construction, then sweep every operation arrow and every source row
//! comparing `migrate ∘ o` with `m ∘ migrate`. The sweep emits the
//! additive `MigrationNaturality` violation witness, naming the failing
//! operation pair (the source operation and its Δ path) and both
//! composed result rows; two unresolved sides are an [`UndefinedSquare`],
//! never a clean.

use catherine_core::findings::{
    EpistemicLabel, Finding, FindingKind, MigrationNaturalityWitness, ViolationWitness,
};
use catherine_core::model::{Arrow, Ident, InstanceMap, Model, RowId, SchemaMap};
use catherine_core::parse::RowNames;
use std::collections::BTreeMap;

use crate::naturality::{arrow_image, source_row_name, UndefinedSquare};

/// Why a migration-naturality run could not construct its check. The
/// same labeled-input-defect discipline as [`crate::naturality::NaturalityError`]: a
/// malformed claim is rejected before the sweep, never surfaced as a
/// finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// The source document declares no `schema_map`; there is no Δ
    /// migration to commute against.
    MissingSchemaMap,
    /// The source document declares no `instance_map`; there is no row
    /// migration to compose the operations through.
    MissingInstanceMap,
    /// The instance map does not cover one of the source instance's
    /// rows of a migrated object (programmatic models only; parse
    /// rejects the shape).
    IncompleteInstanceMap {
        /// The source object whose row is uncovered.
        object: Ident,
        /// The uncovered dense row id.
        row: RowId,
    },
    /// A map cell names a target row the migrated instance does not
    /// declare.
    UnknownMapTarget {
        /// The source object the cell belongs to.
        object: Ident,
        /// The mapped source row's presentation name.
        source_row: String,
        /// The target row name the cell declares.
        target_row: String,
    },
}

/// The migration-naturality sweep result (catherine-kw9.2): witnessed
/// breaks (`findings`) and unverifiable squares (`undefined`), disjoint,
/// exactly like [`crate::naturality::NaturalityOutcome`]. A square undefined on both sides
/// is reported in `undefined` so the caller warns instead of claiming a
/// verified clean.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationOutcome {
    /// One `violation`/`executed` finding per breaking square, sorted.
    pub findings: Vec<Finding>,
    /// One entry per square undefined on both sides, sorted.
    pub undefined: Vec<UndefinedSquare>,
}

/// The migration-naturality check: for every operation arrow `o` of the
/// v1 schema and its Δ target path `m`, compare operate-then-migrate
/// (`migrate ∘ o`, the witness `lhs`) with migrate-then-operate
/// (`m ∘ migrate`, the witness `rhs`) row by row. Operations Δ drops
/// (`m = None`) have no v2 counterpart and are skipped; a dropped
/// object's rows cannot migrate and are skipped with it. A mismatch —
/// including one unresolved side — emits one witnessed finding naming
/// the operation pair and both composed result rows; two unresolved
/// sides are an [`UndefinedSquare`], never a clean.
pub fn migration_naturality(
    source: &Model,
    target: &Model,
    source_names: &RowNames,
    target_names: &RowNames,
) -> Result<MigrationOutcome, MigrationError> {
    let schema_map = source
        .schema_map
        .as_ref()
        .ok_or(MigrationError::MissingSchemaMap)?;
    let instance_map = source
        .instance_map
        .as_ref()
        .ok_or(MigrationError::MissingInstanceMap)?;
    let ctx = MigrationResolution {
        schema_map,
        instance_map,
        source,
        source_names,
        target_names,
    };
    let resolved = resolve_migration_map(&ctx)?;
    Ok(migration_sweep(source, target, schema_map, &resolved))
}

/// The maps, documents, and names one migration resolution pass reads:
/// bundled so the per-object helper stays within the parameter budget.
struct MigrationResolution<'a> {
    schema_map: &'a SchemaMap,
    instance_map: &'a InstanceMap,
    source: &'a Model,
    source_names: &'a RowNames,
    target_names: &'a RowNames,
}

impl MigrationResolution<'_> {
    /// One migrated object's source rows resolved to their target rows
    /// in the target object's name table.
    fn object_rows(
        &self,
        object: &Ident,
        target_object: &Ident,
        rows: &[RowId],
    ) -> Result<Vec<RowId>, MigrationError> {
        let mut targets = Vec::with_capacity(rows.len());
        let cells = self.instance_map.maps.get(object);
        for row in rows {
            let Some(target_name) = cells.and_then(|cells| cells.get(row)) else {
                return Err(MigrationError::IncompleteInstanceMap {
                    object: object.clone(),
                    row: *row,
                });
            };
            let target = self
                .target_names
                .get(target_object)
                .and_then(|names| names.iter().position(|name| name == target_name))
                .map(RowId)
                .ok_or_else(|| MigrationError::UnknownMapTarget {
                    object: object.clone(),
                    source_row: source_row_name(self.source_names, object, *row),
                    target_row: target_name.0.clone(),
                })?;
            targets.push(target);
        }
        Ok(targets)
    }
}

/// The construction-time row migration: every source row of every
/// migrated object gets the target row its instance-map cell names,
/// resolved against the migrated instance's name table *under the
/// object Δ assigns* (objects may rename). A dropped object is skipped
/// with its rows; an unknown cell is a labeled error naming the object,
/// the source row, and the missing target row.
fn resolve_migration_map(
    ctx: &MigrationResolution<'_>,
) -> Result<BTreeMap<Ident, Vec<RowId>>, MigrationError> {
    let mut resolved: BTreeMap<Ident, Vec<RowId>> = BTreeMap::new();
    for (object, target_object) in &ctx.schema_map.objects {
        let Some(target_object) = target_object else {
            continue;
        };
        let Some(rows) = ctx.source.instance.rows.get(object) else {
            continue;
        };
        let targets = ctx.object_rows(object, target_object, rows)?;
        resolved.insert(object.clone(), targets);
    }
    Ok(resolved)
}

/// The documents and resolution one migration commutation square reads:
/// bundled so the per-square helpers stay within the parameter budget.
struct SquareInputs<'a> {
    source: &'a Model,
    target: &'a Model,
    schema_map: &'a SchemaMap,
    resolved: &'a BTreeMap<Ident, Vec<RowId>>,
}

/// The migration commutation sweep: for every operation arrow with a Δ
/// target path and every source row of its migrated source object,
/// compare `migrate ∘ o` (lhs) with the path applied after migration
/// (rhs). Two unequal values — including one unresolved side — are a
/// witnessed break; two `None`s are an undefined square.
fn migration_sweep(
    source: &Model,
    target: &Model,
    schema_map: &SchemaMap,
    resolved: &BTreeMap<Ident, Vec<RowId>>,
) -> MigrationOutcome {
    let mut outcome = MigrationOutcome::default();
    let inputs = SquareInputs {
        source,
        target,
        schema_map,
        resolved,
    };
    for (operation, arrow) in &source.schema.arrows {
        sweep_operation(&inputs, &mut outcome, operation, arrow);
    }
    outcome.findings.sort();
    outcome.undefined.sort();
    outcome
}

/// One operation arrow's commutation squares: every source row of its
/// migrated source object, classified and accumulated into the sweep
/// outcome. An arrow without a Δ target path, or whose source object's
/// rows did not migrate (a dropped object), contributes nothing.
fn sweep_operation(
    inputs: &SquareInputs<'_>,
    outcome: &mut MigrationOutcome,
    operation: &Ident,
    arrow: &Arrow,
) {
    let Some(Some(path)) = inputs.schema_map.arrows.get(operation) else {
        return;
    };
    let Some(rows) = inputs.resolved.get(&arrow.src) else {
        return;
    };
    for row in rows {
        match migration_square(inputs, operation, &path.0, *row) {
            MigrationSquare::Undefined => outcome.undefined.push(UndefinedSquare {
                arrow: operation.clone(),
                source_row: *row,
            }),
            MigrationSquare::Clean => {}
            MigrationSquare::Violation(finding) => outcome.findings.push(finding),
        }
    }
}

/// One migration commutation square's classification.
enum MigrationSquare {
    /// Both compositions resolve to the same row.
    Clean,
    /// The compositions differ (one side may be unresolved).
    Violation(Finding),
    /// Both compositions are unresolved: nothing was verified.
    Undefined,
}

/// One migration commutation square: `migrate ∘ o` (lhs) against `m ∘
/// migrate` (rhs) at one source row. Two equal values are clean, two
/// `None`s are undefined (never clean), and every other pair — including
/// one unresolved side — is a witnessed break.
fn migration_square(
    inputs: &SquareInputs<'_>,
    operation: &Ident,
    path: &[Ident],
    row: RowId,
) -> MigrationSquare {
    let lhs = operate_then_migrate(inputs, operation, row);
    let rhs = migrate_then_operate(inputs, operation, path, row);
    match (lhs, rhs) {
        (None, None) => MigrationSquare::Undefined,
        (lhs, rhs) if lhs == rhs => MigrationSquare::Clean,
        (lhs, rhs) => MigrationSquare::Violation(Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(ViolationWitness::MigrationNaturality(
                MigrationNaturalityWitness {
                    operation: operation.clone(),
                    path: path.to_vec(),
                    source_row: row,
                    lhs,
                    rhs,
                },
            )),
        }),
    }
}

/// `migrate ∘ o`: the operation's image on the source row, then the
/// target row its source-object Δ assign names. `None` when either side
/// is unresolved.
fn operate_then_migrate(inputs: &SquareInputs<'_>, operation: &Ident, row: RowId) -> Option<RowId> {
    let arrow = inputs.source.schema.arrows.get(operation)?;
    inputs
        .source
        .instance
        .values
        .get(operation)
        .and_then(|cells| cells.get(row.0))
        .copied()
        .flatten()
        .and_then(|image| {
            inputs
                .resolved
                .get(&arrow.tgt)
                .and_then(|targets| targets.get(image.0))
                .copied()
        })
}

/// `m ∘ migrate`: the source row's migrated image, then the Δ target
/// path composed in order over the target instance. `None` when either
/// side is unresolved.
fn migrate_then_operate(
    inputs: &SquareInputs<'_>,
    operation: &Ident,
    path: &[Ident],
    row: RowId,
) -> Option<RowId> {
    let arrow = inputs.source.schema.arrows.get(operation)?;
    let migrated = inputs
        .resolved
        .get(&arrow.src)
        .and_then(|rows| rows.get(row.0))
        .copied();
    path_image(inputs.target, path, migrated)
}

/// One Δ target path's image at a migrated row: the path's arrows
/// compose in order over the target instance; a dangling or
/// out-of-space cell anywhere on the path leaves the whole composition
/// unresolved.
fn path_image(target: &Model, path: &[Ident], start: Option<RowId>) -> Option<RowId> {
    let mut current = start;
    for arrow in path {
        current = arrow_image(target, arrow, current);
        current?;
    }
    current
}
