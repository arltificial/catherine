//! Purpose: validate a `rules --project` invocation's shape before any row
//! is read — one presentation source, a declared object, a non-empty
//! column list, and arrows actually sourced at that object.
//!
//! Responsibilities: [`resolve_spec`] returns the [`Resolved`] object,
//! column arrows, and optional action arrow; each rejection is the labeled
//! `bad_argument` the CLI emits (an unknown object, a missing/empty/
//! duplicated column entry, an unknown or not-sourced arrow, and a
//! document that already carries a `rule_presentation`).
//!
//! Rationale: `--project` and the document's own `rule_presentation` are
//! two sources of the same table — never an implicit preference — and a
//! guessed arrow would analyze a column the caller never named. Splitting
//! the resolution out keeps the entry point small and the refusal sites
//! individually testable.

use crate::args::bad_arg;
use crate::projection::Projection;
use crate::render::LabeledError;
use catherine_core::model::{Ident, Model};

/// The resolved invocation: a declared object with arrows sourced at it.
pub(super) struct Resolved {
    pub(super) object: Ident,
    pub(super) columns: Vec<Ident>,
    pub(super) action: Option<Ident>,
}

/// Validate the invocation shape and resolve its arrow names.
pub(super) fn resolve_spec(
    model: &Model,
    projection: &Projection<'_>,
) -> Result<Resolved, LabeledError> {
    refuse_second_source(model, projection.object)?;
    let object = project_object(model, projection.object)?;
    let Some(column_spec) = projection.columns else {
        return Err(missing_columns(&object));
    };
    let columns = resolve_arrows(model, &object, column_spec, "--columns")?;
    let action = match projection.action {
        Some(spec) => Some(resolve_action(model, &object, spec)?),
        None => None,
    };
    Ok(Resolved {
        object,
        columns,
        action,
    })
}

/// The projected object must be a declared schema object.
fn project_object(model: &Model, object: &str) -> Result<Ident, LabeledError> {
    let object = Ident(object.to_string());
    if model.schema.objects.contains_key(&object) {
        return Ok(object);
    }
    Err(bad_arg(
        format!(
            "`--project` names `{}`, which is not a declared schema object",
            object.0
        ),
        "project an object declared in schema.objects",
    ))
}

/// A projection with no columns names no rule predicate.
fn missing_columns(object: &Ident) -> LabeledError {
    bad_arg(
        format!("`--project {}` requires `--columns <arrow,...>`", object.0),
        "a projection with no columns names no rule predicate",
    )
}

/// `--project` on a document that already carries a `rule_presentation`
/// is two sources of the same presentation: never an implicit preference.
fn refuse_second_source(model: &Model, object: &str) -> Result<(), LabeledError> {
    if model.rule_presentation.is_none() {
        return Ok(());
    }
    Err(bad_arg(
        format!(
            "`--project {object}` was given, but the document already carries a `rule_presentation` — keep one presentation source"
        ),
        "drop the document's `rule_presentation`, or analyze it with `--values` instead of `--project`",
    ))
}

/// `--action` takes exactly one arrow: an action is one target, so a
/// second entry would be silently ignored.
fn resolve_action(model: &Model, object: &Ident, spec: &str) -> Result<Ident, LabeledError> {
    let mut arrows = resolve_arrows(model, object, spec, "--action")?;
    if arrows.len() != 1 {
        return Err(bad_arg(
            format!("`--action` takes exactly one arrow, got `{spec}`"),
            "an action is one arrow: a second entry would be silently ignored",
        ));
    }
    Ok(arrows.remove(0))
}

/// Resolve one arrow-list flag: a declared arrow per comma-separated
/// entry, sourced at the projected object, no empties, no duplicates.
fn resolve_arrows(
    model: &Model,
    object: &Ident,
    spec: &str,
    flag: &str,
) -> Result<Vec<Ident>, LabeledError> {
    let mut arrows: Vec<Ident> = Vec::new();
    for name in spec.split(',') {
        let arrow = resolve_arrow(model, object, name, flag)?;
        if arrows.contains(&arrow) {
            return Err(bad_arg(
                format!("`{flag}` names `{name}` twice — each column is one arrow"),
                "a duplicated column would project the same value twice",
            ));
        }
        arrows.push(arrow);
    }
    Ok(arrows)
}

/// One `--columns`/`--action` entry: declared, and sourced at the object.
fn resolve_arrow(
    model: &Model,
    object: &Ident,
    name: &str,
    flag: &str,
) -> Result<Ident, LabeledError> {
    if name.is_empty() {
        return Err(bad_arg(
            format!("empty column entry in `{flag}`"),
            "name one declared arrow per comma-separated entry",
        ));
    }
    let arrow = Ident(name.to_string());
    let Some(declared) = model.schema.arrows.get(&arrow) else {
        return Err(bad_arg(
            format!("`{flag}` names `{name}`, which is not a declared arrow"),
            "declare the arrow in schema.arrows, or fix the spelling",
        ));
    };
    if declared.src != *object {
        return Err(bad_arg(
            format!(
                "`{flag}` names `{name}`, which is not sourced at `{}`",
                object.0
            ),
            "every projected column and action arrow must start at the projected object",
        ));
    }
    Ok(arrow)
}
