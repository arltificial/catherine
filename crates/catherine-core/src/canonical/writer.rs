//! The wild-normative canonical JSON writer. Split out of `canonical.rs`
//! (mechanical lint refactor); the output buffer is threaded through the
//! recursion by value, so serialization still allocates the string once.

use serde_json::Value;

/// Canonical bytes of an arbitrary JSON value per wild's normative rules.
/// serde_json's `Map` is a `BTreeMap`, so object iteration *is* the
/// recursively-sorted key order wild's `keys.sort()` produces.
pub fn write_value(value: &Value) -> String {
    write_canonical(value, String::new())
}

/// Append `value`'s canonical form to `out` and return it.
fn write_canonical(value: &Value, mut out: String) -> String {
    match value {
        Value::Null => {
            out.push_str("null");
            out
        }
        Value::Bool(true) => {
            out.push_str("true");
            out
        }
        Value::Bool(false) => {
            out.push_str("false");
            out
        }
        Value::Number(number) => {
            out.push_str(&number.to_string());
            out
        }
        Value::String(text) => write_escaped(text, out),
        Value::Array(items) => write_array(items, out),
        Value::Object(entries) => write_object(entries, out),
    }
}

fn write_array(items: &[Value], mut out: String) -> String {
    out.push('[');
    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out = write_canonical(item, out);
    }
    out.push(']');
    out
}

fn write_object(entries: &serde_json::Map<String, Value>, mut out: String) -> String {
    out.push('{');
    for (idx, (key, value)) in entries.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out = write_escaped(key, out);
        out.push(':');
        out = write_canonical(value, out);
    }
    out.push('}');
    out
}

/// Append `text` as a wild-normative JSON string: quotes and backslashes
/// escaped, control characters as lowercase `\u00xx`, literal UTF-8
/// otherwise.
fn write_escaped(text: &str, mut out: String) -> String {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
