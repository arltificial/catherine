//! Purpose: parse `.ct.json` v1 model documents into [`Model`] values,
//! rejecting malformed presentations with line-accurate errors.
//!
//! Responsibilities: gate JSON syntax through serde_json (its own
//! line/column), locate every JSON node by pointer with a span scan so
//! semantic errors carry the line/column of the *offending node*, and
//! enforce the six stable error codes (`duplicate_ident`, `unknown_arrow`,
//! `ill_typed_equation`, `invalid_ident`, `invalid_status`,
//! `unsupported_version`) — the fixture harness's expected shape. Parse
//! normalizes presentation away: row names become dense per-object ids in
//! sorted order, equations are stored in canonical sorted order, and
//! every declared arrow carries a total value vector (`None` = dangling).
//!
//! Rationale: design Decision 3 — "line-accurate" means the line/column of
//! the offending JSON node, not of a bespoke text syntax. Structural
//! malformations outside the six stable codes (missing required fields,
//! JSON type mismatches, references to undeclared objects/arrows/rows)
//! surface as `ParseError::Syntax` with a human message; the six codes are
//! the stable semantic contract the golden fixtures pin. Dangling `null`
//! references parse as `None` — a representable value, never dropped or
//! repaired (acset-core `dangling_is_a_value`). Check order is the
//! document order documented in the crate README, so an error is
//! predictable, never order-dependent on the walk.

use crate::error::{ErrorCode, ParseError, SemanticError};
use crate::model::{
    Arrow, ConstraintStatus, Equation, Ident, Instance, Model, Path, RowId, Schema, Version,
};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The raw document as deserialized by serde_json: presentation strings,
/// no interpretation. Semantic validation runs against the span scan of
/// the same text, so every error can point at its node.
#[derive(Deserialize)]
struct RawDoc {
    #[serde(default)]
    format_version: Option<String>,
    #[serde(default)]
    schema: Option<RawSchema>,
    #[serde(default)]
    instance: Option<RawInstance>,
}

#[derive(Deserialize)]
struct RawSchema {
    #[serde(default)]
    objects: Vec<String>,
    #[serde(default)]
    arrows: Vec<RawArrow>,
    #[serde(default)]
    equations: Vec<RawEquation>,
}

#[derive(Deserialize)]
struct RawArrow {
    name: String,
    src: String,
    tgt: String,
}

#[derive(Deserialize)]
struct RawEquation {
    lhs: Vec<String>,
    rhs: Vec<String>,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Deserialize)]
struct RawInstance {
    #[serde(default)]
    rows: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    tables: BTreeMap<String, BTreeMap<String, Option<String>>>,
}

/// Parse a `.ct.json` v1 model document. Accepted additive minors of the
/// current major (1); malformed presentations rejected with line-accurate
/// errors. Check order (documented in the crate README): version →
/// schema objects → arrows → equations → instance rows → tables.
pub fn parse(text: &str) -> Result<Model, ParseError> {
    let doc: RawDoc = serde_json::from_str(text).map_err(syntax_err)?;
    let spans = scan_spans(text);
    validate(doc, &spans)
}

fn validate(doc: RawDoc, spans: &Spans) -> Result<Model, ParseError> {
    // 1. The version gate: content under an unknown major is never
    // interpreted — not a partial or guessed parse.
    let version = match &doc.format_version {
        None => {
            return Err(spans.semantic(
                "",
                ErrorCode::UnsupportedVersion,
                "missing `format_version`: expected `MAJOR.MINOR`".to_string(),
            ));
        }
        Some(text) => {
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
            Version { major, minor }
        }
    };

    // 2. Required top-level sections — structural.
    let Some(schema) = doc.schema else {
        return Err(spans.syntax("", "missing required field `schema`"));
    };
    let Some(instance) = doc.instance else {
        return Err(spans.syntax("", "missing required field `instance`"));
    };

    // 3. Schema objects: charset and duplicate idents (second occurrence).
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

    // 4. Schema arrows: name charset + duplicates; src/tgt charset and
    // declaredness (structural — the six-code `unknown_arrow` is about
    // equation paths).
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

    // 5. Equations: path idents, declared arrows, endpoint typing, status.
    let mut equations = Vec::new();
    for (i, raw) in schema.equations.iter().enumerate() {
        let base = format!("/schema/equations/{i}");
        let lhs = check_path(spans, &format!("{base}/lhs"), &raw.lhs, &arrows)?;
        let rhs = check_path(spans, &format!("{base}/rhs"), &raw.rhs, &arrows)?;
        let lhs_end = path_endpoints(&lhs, &arrows);
        let rhs_end = path_endpoints(&rhs, &arrows);
        if lhs_end != rhs_end {
            return Err(spans.semantic(
                &base,
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
        let status = match raw.status.as_deref() {
            None => ConstraintStatus::Accepted,
            Some("proposed") => ConstraintStatus::Proposed,
            Some("accepted") => ConstraintStatus::Accepted,
            Some(other) => {
                return Err(spans.semantic(
                    &format!("{base}/status"),
                    ErrorCode::InvalidStatus,
                    format!("invalid equation status `{other}`: must be `proposed` or `accepted`"),
                ));
            }
        };
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

    // 6. Instance rows: repeated JSON keys (dropped silently by serde's
    // last-wins map) are caught from the span scan before they can be
    // lost; object keys must be declared; row idents checked per list.
    check_dup_keys(spans, "/instance/rows", "in instance rows")?;
    for obj_name in instance.rows.keys() {
        if !objects.contains_key(&Ident(obj_name.clone())) {
            return Err(spans.syntax_key(
                "/instance/rows",
                obj_name,
                format!("instance rows name undeclared object `{obj_name}`"),
            ));
        }
    }
    let mut row_index: BTreeMap<String, BTreeMap<String, RowId>> = BTreeMap::new();
    for (obj_name, rows) in &instance.rows {
        let mut seen = BTreeMap::new();
        for (j, row) in rows.iter().enumerate() {
            let node = format!("/instance/rows/{obj_name}/{j}");
            check_ident(spans, &node, row)?;
            if seen.contains_key(row) {
                return Err(spans.semantic(
                    &node,
                    ErrorCode::DuplicateIdent,
                    format!("duplicate row ident `{row}` on object `{obj_name}`"),
                ));
            }
            seen.insert(row.clone(), ());
        }
        // Dense ids in sorted-name order — the sorted enumeration is the
        // first-reference order (design Decision 4).
        let index: BTreeMap<String, RowId> = seen
            .keys()
            .enumerate()
            .map(|(i, name)| (name.clone(), RowId(i)))
            .collect();
        row_index.insert(obj_name.clone(), index);
    }

    // 7. Instance tables: repeated keys, declared arrows, declared src
    // rows, and values that are declared target rows or `null`.
    if let Some(dup) = spans
        .dup_keys
        .iter()
        .find(|d| d.object.starts_with("/instance/tables/"))
    {
        let arrow = dup.object.trim_start_matches("/instance/tables/");
        return Err(ParseError::Semantic(SemanticError {
            line: dup.line,
            column: dup.column,
            code: ErrorCode::DuplicateIdent,
            message: format!("duplicate row ident `{}` in table `{arrow}`", dup.key),
        }));
    }
    for arrow_name in instance.tables.keys() {
        if !arrows.contains_key(&Ident(arrow_name.clone())) {
            return Err(spans.syntax_key(
                "/instance/tables",
                arrow_name,
                format!("instance table names undeclared arrow `{arrow_name}`"),
            ));
        }
    }
    // Every declared arrow carries a total vector over its source object's
    // rows; an unspecified cell parses as `None` (dangling).
    let mut values: BTreeMap<Ident, Vec<Option<RowId>>> = BTreeMap::new();
    for (name, arrow) in &arrows {
        let n = row_index.get(&arrow.src.0).map_or(0, BTreeMap::len);
        values.insert(name.clone(), vec![None; n]);
    }
    for (arrow_name, table) in &instance.tables {
        let arrow = &arrows[&Ident(arrow_name.clone())];
        let src_rows = row_index.get(&arrow.src.0);
        let tgt_rows = row_index.get(&arrow.tgt.0);
        for (row_name, value) in table {
            let key_at = format!("/instance/tables/{arrow_name}");
            let Some(src_map) = src_rows else {
                return Err(spans.syntax_key(
                    &key_at,
                    row_name,
                    format!(
                        "table `{arrow_name}` keys row `{row_name}`, but `{}` has no declared rows",
                        arrow.src.0
                    ),
                ));
            };
            let Some(src_id) = src_map.get(row_name) else {
                return Err(spans.syntax_key(
                    &key_at,
                    row_name,
                    format!(
                        "table `{arrow_name}` keys row `{row_name}`, which is not a row of `{}`",
                        arrow.src.0
                    ),
                ));
            };
            let Some(target) = value else {
                continue; // explicit `null` — the same dangling value as an absent cell
            };
            let Some(tgt_id) = tgt_rows.and_then(|m| m.get(target)) else {
                return Err(spans.syntax(
                    &format!("{key_at}/{row_name}"),
                    format!(
                        "table `{arrow_name}` maps `{row_name}` to `{target}`, which is not a row of `{}`",
                        arrow.tgt.0
                    ),
                ));
            };
            values.get_mut(&Ident(arrow_name.clone())).unwrap()[src_id.0] = Some(*tgt_id);
        }
    }

    // 8. The normalized model: dense rows per declared object, total
    // value vectors, canonical equation order. Presentation names never
    // reach it.
    let rows_out = objects
        .keys()
        .map(|o| {
            let n = row_index.get(&o.0).map_or(0, BTreeMap::len);
            (o.clone(), (0..n).map(RowId).collect())
        })
        .collect();
    let schema = Schema {
        objects,
        arrows,
        equations,
    };
    Ok(Model {
        format_version: version,
        schema,
        instance: Instance {
            rows: rows_out,
            values,
        },
    })
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
fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn check_ident(spans: &Spans, node: &str, name: &str) -> Result<(), ParseError> {
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
) -> Result<Path, ParseError> {
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

fn check_dup_keys(spans: &Spans, pointer: &str, where_: &str) -> Result<(), ParseError> {
    if let Some(dup) = spans.dup_keys.iter().find(|d| d.object == pointer) {
        return Err(ParseError::Semantic(SemanticError {
            line: dup.line,
            column: dup.column,
            code: ErrorCode::DuplicateIdent,
            message: format!("duplicate object ident `{}` {where_}", dup.key),
        }));
    }
    Ok(())
}

/// Convert a serde_json error into the structural `Syntax` shape, keeping
/// serde_json's own line/column and stripping its location suffix from the
/// message (the position travels in the fields, not the text).
fn syntax_err(err: serde_json::Error) -> ParseError {
    let message = err.to_string();
    let message = message
        .split(" at line ")
        .next()
        .unwrap_or(&message)
        .to_string();
    ParseError::Syntax {
        line: err.line(),
        column: err.column(),
        message,
    }
}

/// A node location: 1-based line and 1-based character column of the
/// node's first byte.
type Pos = (usize, usize);

/// The span scan of one document: every JSON value's location keyed by
/// JSON pointer, every object key's location keyed by (object pointer,
/// key), and keys repeated inside one object — which serde_json's
/// last-wins maps would otherwise silently drop.
#[derive(Default)]
struct Spans {
    nodes: BTreeMap<String, Pos>,
    keys: BTreeMap<(String, String), Pos>,
    dup_keys: Vec<DupKey>,
}

struct DupKey {
    object: String,
    key: String,
    line: usize,
    column: usize,
}

impl Spans {
    fn semantic(&self, pointer: &str, code: ErrorCode, message: String) -> ParseError {
        let (line, column) = self.nodes.get(pointer).copied().unwrap_or((1, 1));
        ParseError::Semantic(SemanticError {
            line,
            column,
            code,
            message,
        })
    }

    fn syntax(&self, pointer: &str, message: impl Into<String>) -> ParseError {
        let (line, column) = self.nodes.get(pointer).copied().unwrap_or((1, 1));
        ParseError::Syntax {
            line,
            column,
            message: message.into(),
        }
    }

    fn syntax_key(&self, object: &str, key: &str, message: String) -> ParseError {
        let (line, column) = self
            .keys
            .get(&(object.to_string(), key.to_string()))
            .copied()
            .unwrap_or((1, 1));
        ParseError::Syntax {
            line,
            column,
            message,
        }
    }
}

/// Scan the (already syntax-validated) text and record every node and key
/// location. The scan is total over valid JSON; anything it cannot handle
/// it simply stops at, because serde_json has already rejected such text.
fn scan_spans(text: &str) -> Spans {
    let mut scanner = Scanner {
        chars: text.chars().collect(),
        pos: 0,
        line: 1,
        col: 1,
        spans: Spans::default(),
    };
    scanner.value(String::new());
    scanner.spans
}

/// RFC 6901 pointer-component escaping — user idents never contain `~` or
/// `/`, but ignored unknown keys may, and pointers must not collide.
fn pointer_escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

struct Scanner {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    spans: Spans,
}

impl Scanner {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.bump();
        }
    }

    /// Record the current node location, then consume the node.
    fn value(&mut self, pointer: String) {
        let here = (self.line, self.col);
        self.spans.nodes.insert(pointer.clone(), here);
        match self.peek() {
            Some('{') => self.object(pointer),
            Some('[') => self.array(pointer),
            Some('"') => {
                self.string();
            }
            Some(_) => self.scalar(),
            None => {}
        }
    }

    fn object(&mut self, pointer: String) {
        self.bump(); // '{'
        loop {
            self.skip_ws();
            match self.peek() {
                Some('}') => {
                    self.bump();
                    return;
                }
                Some('"') => {}
                _ => return, // serde_json already rejected such text
            }
            let key_here = (self.line, self.col);
            let key = self.string();
            match self.spans.keys.entry((pointer.clone(), key.clone())) {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(key_here);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    self.spans.dup_keys.push(DupKey {
                        object: pointer.clone(),
                        key: key.clone(),
                        line: key_here.0,
                        column: key_here.1,
                    });
                }
            }
            self.skip_ws();
            if self.peek() != Some(':') {
                return;
            }
            self.bump(); // ':'
            self.skip_ws();
            self.value(format!("{pointer}/{}", pointer_escape(&key)));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some('}') => {
                    self.bump();
                    return;
                }
                _ => return,
            }
        }
    }

    fn array(&mut self, pointer: String) {
        self.bump(); // '['
        let mut index = 0usize;
        loop {
            self.skip_ws();
            if self.peek() == Some(']') {
                self.bump();
                return;
            }
            self.value(format!("{pointer}/{index}"));
            index += 1;
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some(']') => {
                    self.bump();
                    return;
                }
                _ => return,
            }
        }
    }

    /// Consume a string node (positioned at the opening quote), returning
    /// its decoded content.
    fn string(&mut self) -> String {
        self.bump(); // opening quote
        let mut out = String::new();
        while let Some(c) = self.bump() {
            match c {
                '"' => return out,
                '\\' => match self.bump() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('b') => out.push('\u{0008}'),
                    Some('f') => out.push('\u{000c}'),
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('u') => out.push(self.unicode_escape()),
                    _ => return out, // serde_json already rejected the text
                },
                c => out.push(c),
            }
        }
        out
    }

    fn unicode_escape(&mut self) -> char {
        let Some(high) = self.hex4() else {
            return char::REPLACEMENT_CHARACTER;
        };
        if (0xd800..=0xdbff).contains(&high) && self.peek() == Some('\\') {
            self.bump();
            if self.peek() == Some('u') {
                self.bump();
                if let Some(low) = self.hex4() {
                    if (0xdc00..=0xdfff).contains(&low) {
                        let combined = 0x10000 + ((high - 0xd800) << 10) + (low - 0xdc00);
                        return char::from_u32(combined).unwrap_or(char::REPLACEMENT_CHARACTER);
                    }
                }
            }
        }
        char::from_u32(high).unwrap_or(char::REPLACEMENT_CHARACTER)
    }

    fn hex4(&mut self) -> Option<u32> {
        let mut value = 0u32;
        for _ in 0..4 {
            let digit = self.peek()?.to_digit(16)?;
            self.bump();
            value = value * 16 + digit;
        }
        Some(value)
    }

    /// Consume a number or literal node (its extent ends at a delimiter).
    fn scalar(&mut self) {
        while let Some(c) = self.peek() {
            if matches!(c, ',' | '}' | ']' | ' ' | '\t' | '\n' | '\r') {
                break;
            }
            self.bump();
        }
    }
}
