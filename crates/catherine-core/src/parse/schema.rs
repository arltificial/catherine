//! Raw document shapes and the schema-side validation passes (version
//! gate, objects, arrows, equations). Split out of `parse.rs` verbatim
//! during the mechanical lint refactor; no behavior change.

use super::spans::Spans;
use crate::error::ErrorCode;
use crate::model::{Arrow, ConstraintStatus, Equation, Ident, Path, Version};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The raw document as deserialized by serde_json: presentation strings,
/// no interpretation. Semantic validation runs against the span scan of
/// the same text, so every error can point at its node.
#[derive(Deserialize)]
pub(super) struct RawDoc {
    #[serde(default)]
    pub(super) format_version: Option<String>,
    #[serde(default)]
    pub(super) schema: Option<RawSchema>,
    #[serde(default)]
    pub(super) instance: Option<RawInstance>,
}

#[derive(Deserialize)]
pub(super) struct RawSchema {
    #[serde(default)]
    pub(super) objects: Vec<String>,
    #[serde(default)]
    pub(super) arrows: Vec<RawArrow>,
    #[serde(default)]
    pub(super) equations: Vec<RawEquation>,
}

#[derive(Deserialize)]
pub(super) struct RawArrow {
    pub(super) name: String,
    pub(super) src: String,
    pub(super) tgt: String,
}

#[derive(Deserialize)]
pub(super) struct RawEquation {
    pub(super) lhs: Vec<String>,
    pub(super) rhs: Vec<String>,
    #[serde(default)]
    pub(super) rationale: String,
    #[serde(default)]
    pub(super) status: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct RawInstance {
    #[serde(default)]
    pub(super) rows: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) tables: BTreeMap<String, BTreeMap<String, Option<String>>>,
}

/// The version gate: `MAJOR.MINOR`, present, well-formed, known major.
pub(super) fn check_version(
    doc: &RawDoc,
    spans: &Spans,
) -> Result<Version, crate::error::ParseError> {
    let Some(text) = &doc.format_version else {
        return Err(spans.semantic(
            "",
            ErrorCode::UnsupportedVersion,
            "missing `format_version`: expected `MAJOR.MINOR`".to_string(),
        ));
    };
    let Some((major, minor)) = parse_version(text) else {
        return Err(spans.semantic(
            "/format_version",
            ErrorCode::UnsupportedVersion,
            format!("malformed `format_version` `{text}`: expected `MAJOR.MINOR`"),
        ));
    };
    if major != 1 {
        return Err(spans.semantic(
            "/format_version",
            ErrorCode::UnsupportedVersion,
            format!("unsupported `format_version` `{text}`: known major is 1"),
        ));
    }
    Ok(Version { major, minor })
}

/// Schema objects: ident charset and duplicate idents (second occurrence).
pub(super) fn check_objects(
    schema: &RawSchema,
    spans: &Spans,
) -> Result<BTreeMap<Ident, ()>, crate::error::ParseError> {
    let mut objects = BTreeMap::new();
    for (i, name) in schema.objects.iter().enumerate() {
        check_ident(spans, &format!("/schema/objects/{i}"), name)?;
        let ident = Ident(name.clone());
        if objects.contains_key(&ident) {
            return Err(spans.semantic(
                &format!("/schema/objects/{i}"),
                ErrorCode::DuplicateIdent,
                format!("duplicate object ident `{name}`"),
            ));
        }
        objects.insert(ident, ());
    }
    Ok(objects)
}

/// Schema arrows: name charset + duplicates; src/tgt charset and
/// declaredness.
pub(super) fn check_arrows(
    schema: &RawSchema,
    objects: &BTreeMap<Ident, ()>,
    spans: &Spans,
) -> Result<BTreeMap<Ident, Arrow>, crate::error::ParseError> {
    let mut arrows = BTreeMap::new();
    for (i, raw) in schema.arrows.iter().enumerate() {
        check_ident(spans, &format!("/schema/arrows/{i}/name"), &raw.name)?;
        let name = Ident(raw.name.clone());
        if arrows.contains_key(&name) {
            return Err(spans.semantic(
                &format!("/schema/arrows/{i}/name"),
                ErrorCode::DuplicateIdent,
                format!("duplicate arrow ident `{}`", raw.name),
            ));
        }
        for (field, endpoint) in [("src", &raw.src), ("tgt", &raw.tgt)] {
            check_ident(spans, &format!("/schema/arrows/{i}/{field}"), endpoint)?;
            if !objects.contains_key(&Ident(endpoint.clone())) {
                return Err(spans.syntax(
                    &format!("/schema/arrows/{i}/{field}"),
                    format!(
                        "arrow `{}`: {field} object `{endpoint}` is not declared",
                        raw.name
                    ),
                ));
            }
        }
        arrows.insert(
            name,
            Arrow {
                src: Ident(raw.src.clone()),
                tgt: Ident(raw.tgt.clone()),
            },
        );
    }
    Ok(arrows)
}

/// Equations: path idents, declared arrows, endpoint typing, status.
pub(super) fn check_equations(
    schema: &RawSchema,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<Vec<Equation>, crate::error::ParseError> {
    let mut equations = Vec::new();
    for (i, raw) in schema.equations.iter().enumerate() {
        let base = format!("/schema/equations/{i}");
        let lhs = check_path(spans, &format!("{base}/lhs"), &raw.lhs, arrows)?;
        let rhs = check_path(spans, &format!("{base}/rhs"), &raw.rhs, arrows)?;
        check_endpoints(raw, &lhs, &rhs, &base, arrows, spans)?;
        let status = check_status(raw.status.as_deref(), &base, spans)?;
        equations.push(Equation {
            lhs,
            rhs,
            rationale: raw.rationale.clone(),
            status,
        });
    }
    // Canonical order: equation list order in the file is presentation
    // (design Decision 4).
    equations.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Ok(equations)
}

/// Endpoint typing: both paths must span the same src → tgt pair.
fn check_endpoints(
    raw: &RawEquation,
    lhs: &Path,
    rhs: &Path,
    base: &str,
    arrows: &BTreeMap<Ident, Arrow>,
    spans: &Spans,
) -> Result<(), crate::error::ParseError> {
    let lhs_end = path_endpoints(lhs, arrows);
    let rhs_end = path_endpoints(rhs, arrows);
    if lhs_end != rhs_end {
        return Err(spans.semantic(
            base,
            ErrorCode::IllTypedEquation,
            format!(
                "equation paths [{}] ({} -> {}) and [{}] ({} -> {}) have mismatched endpoints",
                raw.lhs.join(", "),
                lhs_end.0,
                lhs_end.1,
                raw.rhs.join(", "),
                rhs_end.0,
                rhs_end.1
            ),
        ));
    }
    Ok(())
}

/// Equation status: absent defaults to `accepted`; anything but
/// `proposed`/`accepted` is rejected.
fn check_status(
    status: Option<&str>,
    base: &str,
    spans: &Spans,
) -> Result<ConstraintStatus, crate::error::ParseError> {
    match status {
        None => Ok(ConstraintStatus::Accepted),
        Some("proposed") => Ok(ConstraintStatus::Proposed),
        Some("accepted") => Ok(ConstraintStatus::Accepted),
        Some(other) => Err(spans.semantic(
            &format!("{base}/status"),
            ErrorCode::InvalidStatus,
            format!("invalid equation status `{other}`: must be `proposed` or `accepted`"),
        )),
    }
}

/// Version spelling: `MAJOR.MINOR`, both all-digits, no sign or padding
/// rules beyond JSON string content.
fn parse_version(text: &str) -> Option<(u64, u64)> {
    let (major, minor) = text.split_once('.')?;
    if major.is_empty() || minor.is_empty() {
        return None;
    }
    if !major.bytes().all(|b| b.is_ascii_digit()) || !minor.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Ident charset `[A-Za-z_][A-Za-z0-9_]*` (DESIGN §4.1).
pub(super) fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub(super) fn check_ident(
    spans: &Spans,
    node: &str,
    name: &str,
) -> Result<(), crate::error::ParseError> {
    if is_ident(name) {
        Ok(())
    } else {
        Err(spans.semantic(
            node,
            ErrorCode::InvalidIdent,
            format!("invalid ident `{name}`: must match [A-Za-z_][A-Za-z0-9_]*"),
        ))
    }
}

/// Validate one equation path: non-empty, every element an ident naming a
/// declared arrow. The unknown-arrow error names the whole path and points
/// at the offending element.
fn check_path(
    spans: &Spans,
    base: &str,
    path: &[String],
    arrows: &BTreeMap<Ident, Arrow>,
) -> Result<Path, crate::error::ParseError> {
    if path.is_empty() {
        return Err(spans.syntax(base, "equation path must be non-empty"));
    }
    for (j, name) in path.iter().enumerate() {
        let node = format!("{base}/{j}");
        check_ident(spans, &node, name)?;
        if !arrows.contains_key(&Ident(name.clone())) {
            return Err(spans.semantic(
                &node,
                ErrorCode::UnknownArrow,
                format!(
                    "equation path [{}] references undeclared arrow `{name}`",
                    path.join(", ")
                ),
            ));
        }
    }
    Ok(Path(path.iter().map(|s| Ident(s.clone())).collect()))
}

/// Path endpoints: the src of the first arrow and the tgt of the last.
/// Every element is declared by [`check_path`], so the lookups are total.
fn path_endpoints(path: &Path, arrows: &BTreeMap<Ident, Arrow>) -> (String, String) {
    let first = &arrows[&path.0[0]];
    let last = &arrows[&path.0[path.0.len() - 1]];
    (first.src.0.clone(), last.tgt.0.clone())
}
