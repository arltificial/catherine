//! Purpose: the shared CLI plumbing every gap-suite observation runs on.
//! Responsibilities: locate a fixture, run the CLI and unwrap its JSON
//! envelope, expose findings/warnings, and classify a `check` run into the
//! only outcomes a row may name — the exact claimed silence, a labeled
//! error, a warning-bearing clean run, or the finding-bearing violation run.
//! Any other exit is a panic naming the case: a fixture that fails for an
//! unrelated reason can never be read as a disposition flip.
//! Rationale: split out of the former single-file `cases.rs` so each thematic
//! module states only its row's claim; the classifiers keep every case's
//! `Cleared` arm reachable only from its documented fixed signature.

use crate::common;
use serde_json::Value;
use std::process::Output;

/// Absolute path to one gap-suite fixture document.
pub(super) fn fixture(name: &str) -> String {
    crate::fixture(name)
}

/// The JSON envelope of a run, falling back to stderr for labeled errors.
pub(super) fn envelope(out: &Output, what: &str) -> Value {
    let stdout = common::stdout(out);
    let text = if stdout.trim().is_empty() {
        // Labeled build/parse errors are the error envelope on stderr.
        common::stderr(out)
    } else {
        stdout
    };
    common::parse_json(&text, what)
}

pub(super) fn findings(value: &Value) -> &Vec<Value> {
    common::findings(value)
}

pub(super) fn warnings(value: &Value) -> &Vec<Value> {
    value["warnings"].as_array().expect("warnings array")
}

/// The four outcomes a `check` run may have for the gap suite. Each is a
/// complete structural signature; nothing else is representable here.
pub(super) enum CheckRun {
    /// Exit 0 with no findings and no warnings: the claimed silent drop.
    Silent,
    /// Exit 1 carrying a labeled error.
    Labeled(String),
    /// Exit 0 with a warning on the envelope: a diagnostic, not a finding.
    Warning(String),
    /// Exit 3 carrying the finding array.
    Findings(Vec<Value>),
}

impl CheckRun {
    pub(super) fn is_silent(&self) -> bool {
        matches!(self, Self::Silent)
    }

    pub(super) fn is_labeled(&self) -> bool {
        matches!(self, Self::Labeled(_))
    }

    pub(super) fn is_warning(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    /// The finding array, `None` for every non-violation outcome.
    pub(super) fn findings(&self) -> Option<&[Value]> {
        match self {
            Self::Findings(found) => Some(found),
            _ => None,
        }
    }

    /// The diagnostic text, empty for the silent signature.
    pub(super) fn text(&self) -> String {
        match self {
            Self::Silent => String::new(),
            Self::Labeled(text) | Self::Warning(text) => text.clone(),
            Self::Findings(found) => joined(found),
        }
    }
}

/// Run `check <fixture>` and classify it into exactly one [`CheckRun`].
/// Exit codes outside `0 | 1 | 3` panic naming the fixture, code and stderr.
pub(super) fn check_run(name: &str) -> CheckRun {
    let path = fixture(name);
    let out = common::run(&["check", &path, "--json"]);
    let value = parsed_envelope(name, &out);
    classify(name, &out, &value)
}

fn parsed_envelope(name: &str, out: &Output) -> Value {
    if matches!(common::code(out), 0 | 1 | 3) {
        return envelope(out, name);
    }
    panic!(
        "{name}: unexpected check exit {}: {}",
        common::code(out),
        common::stderr(out)
    )
}

fn classify(name: &str, out: &Output, value: &Value) -> CheckRun {
    match common::code(out) {
        0 => classify_clean(value),
        1 => CheckRun::Labeled(labeled_error(name, value)),
        _ => classify_finding_run(name, out, value),
    }
}

fn classify_finding_run(name: &str, out: &Output, value: &Value) -> CheckRun {
    if common::code(out) == 3 {
        return CheckRun::Findings(findings(value).clone());
    }
    panic!(
        "{name}: unexpected check exit {}: {}",
        common::code(out),
        common::stderr(out)
    )
}

fn classify_clean(value: &Value) -> CheckRun {
    if findings(value).is_empty() {
        if warnings(value).is_empty() {
            return CheckRun::Silent;
        }
    }
    CheckRun::Warning(diagnostic_text(value))
}

fn labeled_error(name: &str, value: &Value) -> String {
    assert_eq!(value["ok"], false, "{name}: exit 1 is the error envelope");
    let code = value["data"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: exit 1 carries a labeled error code: {value}"));
    format!(
        "{code}: {}",
        value["data"]["message"].as_str().unwrap_or_default()
    )
}

fn joined(found: &[Value]) -> String {
    found
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn diagnostic_text(value: &Value) -> String {
    let mut parts: Vec<String> = warnings(value).iter().map(warning_text).collect();
    parts.extend(findings(value).iter().map(Value::to_string));
    parts.join("\n")
}

fn warning_text(warning: &Value) -> String {
    format!(
        "{}: {}",
        warning["rule_name"].as_str().unwrap_or_default(),
        warning["message"].as_str().unwrap_or_default()
    )
}

/// Assert that a diagnostic names its subject; the panic is the case's
/// failure when a fixture breaks for an unrelated reason.
pub(super) fn assert_diagnostic_names(what: &str, text: &str, needles: &[&str]) {
    let lowered = text.to_ascii_lowercase();
    assert!(
        needles.iter().any(|needle| lowered.contains(needle)),
        "{what}: the diagnostic must name one of {needles:?}, got `{text}`"
    );
}