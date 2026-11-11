//! The wild-normative canonical JSON writer. Split out of `canonical.rs`
//! verbatim (mechanical lint refactor); no behavior change.

use serde_json::Value;

/// Canonical bytes of an arbitrary JSON value per wild's normative rules.
/// serde_json's `Map` is a `BTreeMap`, so object iteration *is* the
/// recursively-sorted key order wild's `keys.sort()` produces.
pub fn write_value(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => write_escaped(s, out),
        Value::Array(items) => write_array(items, out),
        Value::Object(entries) => write_object(entries, out),
    }
}

fn write_array(items: &[Value], out: &mut String) {
    out.push('[');
    for (idx, item) in items.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        write_canonical(item, out);
    }
    out.push(']');
}

fn write_object(entries: &serde_json::Map<String, Value>, out: &mut String) {
    out.push('{');
    for (idx, (key, value)) in entries.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        write_escaped(key, out);
        out.push(':');
        write_canonical(value, out);
    }
    out.push('}');
}

fn write_escaped(text: &str, out: &mut String) {
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
}
