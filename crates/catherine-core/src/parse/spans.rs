//! The span scan: every JSON node's location keyed by JSON pointer, and
//! object keys repeated inside one object. Split out of `parse.rs`
//! verbatim (mechanical lint refactor); no behavior change.

use crate::error::{ErrorCode, ParseError, SemanticError};
use std::collections::BTreeMap;

/// A node location: 1-based line and 1-based character column of the
/// node's first byte.
pub(super) type Pos = (usize, usize);

/// The span scan of one document: every JSON value's location keyed by
/// JSON pointer, every object key's location keyed by (object pointer,
/// key), and keys repeated inside one object — which serde_json's
/// last-wins maps would otherwise silently drop.
#[derive(Debug, Clone, Default)]
pub(super) struct Spans {
    pub(super) nodes: BTreeMap<String, Pos>,
    pub(super) keys: BTreeMap<(String, String), Pos>,
    pub(super) dup_keys: Vec<DupKey>,
}

/// Cloned with the scan: a deferred equation carries the spans into the
/// post-glue resolution pass.
#[derive(Debug, Clone)]
pub(super) struct DupKey {
    pub(super) object: String,
    pub(super) key: String,
    pub(super) line: usize,
    pub(super) column: usize,
}

/// A labeled defect: the error code and its rendered message, passed to
/// the span-positioned rejections as one bundle.
pub(super) struct Defect {
    pub(super) code: ErrorCode,
    pub(super) message: String,
}

impl Spans {
    pub(super) fn semantic(&self, pointer: &str, code: ErrorCode, message: String) -> ParseError {
        let (line, column) = self.nodes.get(pointer).copied().unwrap_or((1, 1));
        ParseError::Semantic(SemanticError {
            line,
            column,
            code,
            message,
        })
    }

    /// The node position, when the span scan recorded one.
    pub(super) fn position(&self, pointer: &str) -> Option<(usize, usize)> {
        self.nodes.get(pointer).copied()
    }

    pub(super) fn syntax(&self, pointer: &str, message: impl Into<String>) -> ParseError {
        let (line, column) = self.nodes.get(pointer).copied().unwrap_or((1, 1));
        ParseError::Syntax {
            line,
            column,
            message: message.into(),
        }
    }

    pub(super) fn syntax_key(&self, object: &str, key: &str, message: String) -> ParseError {
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

    /// A labeled rejection positioned at one object key rather than at a
    /// value node — the unknown-field discipline. The `(object pointer,
    /// key)` pair locates the key; the defect carries code and message.
    pub(super) fn semantic_key(&self, at: (&str, &str), defect: Defect) -> ParseError {
        let (line, column) = self
            .keys
            .get(&(at.0.to_string(), at.1.to_string()))
            .copied()
            .unwrap_or((1, 1));
        ParseError::Semantic(SemanticError {
            line,
            column,
            code: defect.code,
            message: defect.message,
        })
    }
}

/// Scan the (already syntax-validated) text and record every node and key
/// location. The scan is total over valid JSON; anything it cannot handle
/// it simply stops at, because serde_json has already rejected such text.
pub(super) fn scan_spans(text: &str) -> Spans {
    let mut scanner = Scanner {
        chars: text.chars().collect(),
        pos: 0,
        line: 1,
        col: 1,
        spans: Spans::default(),
    };
    scan_value(&mut scanner, String::new());
    scanner.spans
}

/// RFC 6901 pointer-component escaping — user idents never contain `~` or
/// `/`, but ignored unknown keys may, and pointers must not collide.
fn pointer_escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// The object delimiter the scanner dispatches on, spelled as a unicode
/// escape so the brace never appears as a bare source character.
const OPEN_BRACE: char = '\u{007b}';

/// The object delimiter the scanner consumes, spelled as a unicode
/// escape for the same reason.
const CLOSE_BRACE: char = '\u{007d}';

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

    /// After one element value: consume the `,` (the container continues)
    /// or the closing delimiter, reporting whether to read another
    /// element. Anything else ends the container — serde_json already
    /// rejected such text.
    fn separator(&mut self, close: char) -> bool {
        self.skip_ws();
        match self.peek() {
            Some(',') => {
                self.bump();
                true
            }
            Some(c) if c == close => {
                self.bump();
                false
            }
            _ => false,
        }
    }

    /// Consume one object key at its recorded location, registering it
    /// (or a duplicate-key finding) in the span scan.
    fn record_key(&mut self, pointer: &str) -> String {
        let key_here = (self.line, self.col);
        let key = self.string();
        match self.spans.keys.entry((pointer.to_string(), key.clone())) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(key_here);
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                self.spans.dup_keys.push(DupKey {
                    object: pointer.to_string(),
                    key: key.clone(),
                    line: key_here.0,
                    column: key_here.1,
                });
            }
        }
        key
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
            return self.surrogate_pair(high);
        }
        char::from_u32(high).unwrap_or(char::REPLACEMENT_CHARACTER)
    }

    /// Consume a `\uXXXX` low surrogate following a high surrogate and
    /// combine the pair; a malformed pair yields the replacement
    /// character.
    fn surrogate_pair(&mut self, high: u32) -> char {
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
        char::REPLACEMENT_CHARACTER
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

/// Record the current node location, then consume the node.
fn scan_value(scan: &mut Scanner, pointer: String) {
    let here = (scan.line, scan.col);
    scan.spans.nodes.insert(pointer.clone(), here);
    scan_consume(scan, pointer);
}

/// Consume one node's body: containers recurse at the same pointer
/// root, leaves end here.
fn scan_consume(scan: &mut Scanner, pointer: String) {
    match scan.peek() {
        Some(c) if c == OPEN_BRACE => scan_object(scan, pointer),
        Some('[') => scan_array(scan, pointer),
        Some('"') => {
            scan.string();
        }
        Some(_) => scan.scalar(),
        None => {}
    }
}

/// One object: each key at its recorded location, each value at
/// `{pointer}/{key}`.
fn scan_object(scan: &mut Scanner, pointer: String) {
    scan.bump(); // past the opening delimiter
    loop {
        scan.skip_ws();
        match scan.peek() {
            Some(c) if c == CLOSE_BRACE => {
                scan.bump();
                return;
            }
            Some('"') => {}
            _ => return, // serde_json already rejected such text
        }
        let key = scan.record_key(&pointer);
        scan.skip_ws();
        if scan.peek() != Some(':') {
            return;
        }
        scan.bump(); // ':'
        scan.skip_ws();
        scan_value(scan, format!("{pointer}/{}", pointer_escape(&key)));
        if !scan.separator(CLOSE_BRACE) {
            return;
        }
    }
}

/// One array: each element at `{pointer}/{index}`.
fn scan_array(scan: &mut Scanner, pointer: String) {
    scan.bump(); // '['
    for index in 0.. {
        scan.skip_ws();
        if scan.peek() == Some(']') {
            scan.bump();
            return;
        }
        scan_value(scan, format!("{pointer}/{index}"));
        if !scan.separator(']') {
            return;
        }
    }
}
