//! Purpose: `ledger-record` seals an entry file that declares no id under
//! its computed content address (openspec change `add-lean-kernel-model`,
//! tasks.md task 12) — authoring the stop-gate decision entry by hand
//! stays practical without weakening the integrity contract: a declared
//! id is never rewritten, so a wrong id is still refused as
//! `IdMismatch`, and content tampered with after sealing no longer
//! matches the sealed id.
//!
//! Responsibilities: drive the real binaries (`ledger-record`,
//! `ledger-check`) plus the library seal helper and assert observable
//! behavior only — exit codes, the named violation, the ledger file's
//! shape, and the sealed id.
//!
//! Rationale: the id IS the digest of the canonical content bytes, so a
//! human writes the content and the tool computes the address. The tool
//! must not also accept a hand-written address that disagrees with those
//! bytes: sealing is computation, never a repair of a stated claim.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use catherine_ledger::entry::{entry_id, seal_parsed, DecisionRecord, EntryContent};
use catherine_ledger::validate::{validate_dir, Violation};
use serde_json::{json, Value};

use support::{present, TempLedger, TestError};

/// The staged candidate file: written OUTSIDE the ledger directory —
/// the binary moves it into the directory itself.
fn candidate(ledger: &TempLedger) -> PathBuf {
    ledger.root.join("candidate.json")
}

/// Write the authored entry JSON OUTSIDE the ledger directory — the
/// binary stages it into the directory itself.
fn stage(ledger: &TempLedger, value: &Value) -> Result<(), TestError> {
    fs::write(candidate(ledger), serde_json::to_string_pretty(value)?)?;
    Ok(())
}

/// The ledger directory's `.json` files, sorted.
fn json_files(dir: &Path) -> Result<Vec<PathBuf>, TestError> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect();
    files.sort();
    Ok(files)
}

/// The stop-gate decision record the gate is asked to write down.
fn decision_record() -> DecisionRecord {
    DecisionRecord {
        outcome: "hold".into(),
        scope: "the lean kernel model at landed scope".into(),
        rationale:
            "the proofs pinned the route-exhaustiveness surface without changing it, so the \
             model is maintained at the scope it landed at"
                .into(),
        proofs: vec![
            "coproduct_route_exhaustive".into(),
            "coproduct_injection_disjoint".into(),
        ],
        defects_found: vec![],
        holes_found: vec![
            "pair-semantics injectivity is not stated by the path-equation surface".into(),
        ],
        fixture_only_comparison:
            "the fixture corpus sampled the accepted coproduct shapes; the proofs pinned \
             exhaustiveness and injection disjointness for every accepted row"
                .into(),
    }
}

/// A decision entry with no kata or rollup payload and non-empty model
/// hashes — the shape task 12 records under `ledger/lean/`.
fn decision_content() -> EntryContent {
    EntryContent {
        seq: 1,
        kind: "decision".into(),
        recorded: "2026-11-17".into(),
        kata: None,
        rollup: None,
        model_hashes: vec!["sha256:lean-kernel-model".into()],
        supersedes: None,
        decision: Some(decision_record()),
    }
}

/// The authored entry JSON: the content alone, with NO `id` field — what
/// a human writes.
fn decision_json() -> Result<Value, TestError> {
    Ok(serde_json::to_value(decision_content())?)
}

/// Run the binary over the ledger directory and the staged candidate.
fn record(ledger: &TempLedger) -> Result<Output, TestError> {
    Ok(Command::new(env!("CARGO_BIN_EXE_ledger-record"))
        .arg(&ledger.dir)
        .arg(candidate(ledger))
        .output()?)
}

/// Run the validator binary over the ledger directory.
fn check(dir: &Path) -> Result<Output, TestError> {
    Ok(Command::new(env!("CARGO_BIN_EXE_ledger-check"))
        .arg(dir)
        .output()?)
}

/// The library helper: a parsed entry JSON without an id seals under the
/// computed content address and serializes as the flat entry shape the
/// validator reads back — the flattened content plus `id`, not a nested
/// `content` object.
#[test]
fn seal_parsed_seals_an_unsealed_value_into_the_entry_shape(
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = seal_parsed(decision_json()?)?;
    assert_eq!(entry.id, entry_id(&decision_content())?);
    assert!(
        entry.id.starts_with("sha256:"),
        "the id is content-addressed"
    );

    let value: Value = serde_json::to_value(&entry)?;
    assert!(
        value.get("content").is_none(),
        "the entry shape is the flattened content, never nested"
    );
    assert_eq!(value.get("kind").and_then(Value::as_str), Some("decision"));
    assert_eq!(
        value.get("id").and_then(Value::as_str),
        Some(entry.id.as_str())
    );
    Ok(())
}

/// An entry file without an `id` is recorded under its computed content
/// address, the directory validates, and `ledger-check` exits 0.
#[test]
fn ledger_record_seals_an_entry_file_without_an_id() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("missing-id")?;
    stage(&ledger, &decision_json()?)?;

    let output = record(&ledger)?;
    assert!(
        output.status.success(),
        "an unsealed entry is recorded; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let sealed = entry_id(&decision_content())?;
    let files = json_files(&ledger.dir)?;
    assert_eq!(files.len(), 1, "exactly one entry was written");
    let recorded: Value = serde_json::from_str(&fs::read_to_string(&files[0])?)?;
    assert_eq!(
        recorded.get("id").and_then(Value::as_str),
        Some(sealed.as_str()),
        "the recorded id is the computed content address"
    );
    assert!(
        recorded.get("content").is_none(),
        "the file is the flattened entry shape, not a nested content object"
    );

    let valid = validate_dir(&ledger.dir)
        .map_err(|v| format!("the sealed decision entry validates: {v:?}"))?;
    assert_eq!(valid.entries.len(), 1);
    assert_eq!(valid.entries[0].id, sealed);
    assert_eq!(valid.entries[0].content.kind, "decision");

    let checked = check(&ledger.dir)?;
    assert!(
        checked.status.success(),
        "ledger-check exits 0; stderr: {}",
        String::from_utf8_lossy(&checked.stderr)
    );
    Ok(())
}

/// The authoring placeholders say "not sealed yet": an empty id and the
/// literal `auto` are both sealed, exactly like a missing id.
#[test]
fn ledger_record_seals_the_empty_and_auto_id_placeholders() -> Result<(), Box<dyn std::error::Error>>
{
    let sealed = entry_id(&decision_content())?;
    for (tag, placeholder) in [("empty-id", ""), ("auto-id", "auto")] {
        let ledger = TempLedger::new(tag)?;
        let mut value = decision_json()?;
        value["id"] = json!(placeholder);
        stage(&ledger, &value)?;

        let output = record(&ledger)?;
        assert!(
            output.status.success(),
            "the `{placeholder}` placeholder is sealed; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let files = json_files(&ledger.dir)?;
        assert_eq!(files.len(), 1, "one entry written for `{placeholder}`");
        let recorded: Value = serde_json::from_str(&fs::read_to_string(&files[0])?)?;
        assert_eq!(
            recorded.get("id").and_then(Value::as_str),
            Some(sealed.as_str()),
            "the placeholder is replaced by the content address"
        );
        validate_dir(&ledger.dir)
            .map_err(|v| format!("the sealed placeholder entry validates: {v:?}"))?;
    }
    Ok(())
}

/// A declared id is a claim, not a placeholder: a wrong one is refused as
/// `IdMismatch` and nothing is appended.
#[test]
fn ledger_record_refuses_a_wrong_declared_id() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("wrong-id")?;
    let mut value = decision_json()?;
    value["id"] = json!(format!("sha256:{}", "0".repeat(64)));
    stage(&ledger, &value)?;

    let output = record(&ledger)?;
    assert_eq!(output.status.code(), Some(1), "a wrong id is refused");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("IdMismatch"),
        "the rejection is named: {stderr}"
    );
    assert!(
        json_files(&ledger.dir)?.is_empty(),
        "the refused entry was not appended"
    );
    Ok(())
}

/// Tampering with the decision content after sealing changes the computed
/// id, so the sealed file no longer validates — the append-only integrity
/// contract holds for decision entries too.
#[test]
fn tampered_decision_content_changes_the_id_and_is_refused(
) -> Result<(), Box<dyn std::error::Error>> {
    let ledger = TempLedger::new("tampered")?;
    stage(&ledger, &decision_json()?)?;
    assert!(
        record(&ledger)?.status.success(),
        "the unsealed entry is recorded first"
    );

    let files = json_files(&ledger.dir)?;
    assert_eq!(files.len(), 1);
    let mut recorded: Value = serde_json::from_str(&fs::read_to_string(&files[0])?)?;
    let sealed_id = present(
        recorded.get("id").and_then(Value::as_str),
        "the recorded entry's id",
    )?
    .to_string();

    recorded["decision"]["rationale"] = json!("a rationale written after the seal");
    let tampered_content: EntryContent = serde_json::from_value(recorded.clone())?;
    assert_ne!(
        entry_id(&tampered_content)?,
        sealed_id,
        "the tampered content hashes to a different address"
    );
    fs::write(&files[0], serde_json::to_string(&recorded)?)?;

    let violations = match validate_dir(&ledger.dir) {
        Ok(_) => return Err("the tampered ledger is refused".into()),
        Err(violations) => violations,
    };
    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, Violation::IdMismatch { .. })),
        "the refusal is the id mismatch: {violations:?}"
    );
    let checked = check(&ledger.dir)?;
    assert_eq!(checked.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&checked.stderr).contains("IdMismatch"),
        "ledger-check names the rejection"
    );
    Ok(())
}
