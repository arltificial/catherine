//! Standard-conformance gate for the pinned ecosystem toolchain
//! (beads `catherine-6p5`; Standardization §4 via versions.ddl.toml).
//!
//! Purpose: close the gap where versions.ddl.toml claimed "CI installs these
//! pinned (see ci.yml)" with nothing checking it — this test is the gate the
//! file's header points at. It asserts (1) versions.ddl.toml parses; (2)
//! every [tools] entry is a concrete pin — no `latest`, no bare major or
//! partial triple, only the full numeric x.y.z / calendar-date form each
//! tool uses (wai's 2026.10.19 included); (3) the seven tools CI installs
//! (.github/workflows/ci.yml → scripts/ci-install.sh) all carry pins; and
//! (4) .beads/issues.jsonl — the source of truth per the no-db invariant —
//! is intact: every line is a JSON object with an id, ids are unique, and
//! every dependency reference resolves inside the same file.
//!
//! Responsibilities: read the two repo files via CARGO_MANIFEST_DIR, assert
//! structurally (toml::Value / serde_json::Value), fail loudly naming the
//! offending tool or line. No second pin source exists — CHANGELOG.md,
//! docs/, and openspec/ record no tool versions and Cargo.lock covers only
//! workspace deps — so cross-source matching is NOT asserted here; syntax
//! and pin-shape only.
//!
//! Rationale: `bd export` reads bd's dolt database and CI never runs dolt,
//! so the bd-export freshness check is this conformance assertion on
//! issues.jsonl integrity instead (the ticket's prescribed fallback). The
//! red shape was the previously-absent gate itself: before this file,
//! nothing failed when a pin read `latest`.

use std::collections::BTreeSet;
use std::fs;

use serde_json::Value as Json;
use toml::Value as Toml;

const VERSIONS_TOML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../versions.ddl.toml");
const BEADS_JSONL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.beads/issues.jsonl");

/// The tools .github/workflows/ci.yml installs via scripts/ci-install.sh —
/// each must keep a pin in versions.ddl.toml or the install script aborts.
const CI_TOOLS: [&str; 7] = [
    "wai",
    "pretender",
    "dont",
    "dulce-de-leche",
    "openspec",
    "bd",
    "ah",
];

/// A concrete pin: exactly three dot-separated non-empty numeric segments.
/// Covers semver (`0.10.0`) and wai's calendar form (`2026.10.19`); rejects
/// `latest`, bare majors (`1`), partials (`1.3`), and ranges (`^1`, `1.*`).
fn is_concrete_pin(pin: &str) -> bool {
    if pin.is_empty() || pin.eq_ignore_ascii_case("latest") {
        return false;
    }
    let segments: Vec<&str> = pin.split('.').collect();
    segments.len() == 3
        && segments
            .iter()
            .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}

#[test]
fn versions_ddl_toml_parses_with_a_concrete_pin_per_ci_tool() {
    let raw =
        fs::read_to_string(VERSIONS_TOML).unwrap_or_else(|e| panic!("read {VERSIONS_TOML}: {e}"));
    let parsed: Toml = raw
        .parse()
        .unwrap_or_else(|e| panic!("versions.ddl.toml does not parse: {e}"));

    let tools = parsed
        .get("tools")
        .and_then(|t| t.as_table())
        .unwrap_or_else(|| panic!("versions.ddl.toml has no [tools] table"));
    assert!(!tools.is_empty(), "[tools] is empty — nothing is pinned");

    for tool in CI_TOOLS {
        let pin = tools.get(tool).and_then(|v| v.as_str()).unwrap_or_else(|| {
            panic!(
                "tool '{tool}' has no pin in [tools] — ci-install.sh would install nothing for it"
            )
        });
        assert!(
            is_concrete_pin(pin),
            "tool '{tool}' pin '{pin}' is not a concrete pin (no 'latest', no bare major)"
        );
    }

    for (name, value) in tools {
        let Some(pin) = value.as_str() else {
            panic!("tool '{name}' pin is not a string: {value}");
        };
        assert!(
            is_concrete_pin(pin),
            "tool '{name}' pin '{pin}' is not a concrete pin"
        );
    }
}

#[test]
fn beads_issues_jsonl_is_an_intact_source_of_truth() {
    let raw = fs::read_to_string(BEADS_JSONL).unwrap_or_else(|e| panic!("read {BEADS_JSONL}: {e}"));
    assert!(
        !raw.trim().is_empty(),
        ".beads/issues.jsonl is empty — the exported source of truth is gone"
    );

    let mut ids = BTreeSet::new();
    // (line number, dependency's issue_id, dependency's depends_on_id)
    let mut dependencies = Vec::new();

    for (i, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record: Json = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} of issues.jsonl is not valid JSON: {e}", i + 1));

        let id = record
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| panic!("line {} of issues.jsonl has no id", i + 1));
        assert!(
            ids.insert(id.to_string()),
            "duplicate issue id '{id}' at line {}",
            i + 1
        );

        if let Some(deps) = record.get("dependencies").and_then(|v| v.as_array()) {
            for dep in deps {
                let issue_id = dep.get("issue_id").and_then(|v| v.as_str());
                let depends_on_id = dep.get("depends_on_id").and_then(|v| v.as_str());
                match (issue_id, depends_on_id) {
                    (Some(from), Some(to)) => {
                        dependencies.push((i + 1, from.to_string(), to.to_string()));
                    }
                    _ => panic!(
                        "line {} dependency is missing issue_id/depends_on_id: {dep}",
                        i + 1
                    ),
                }
            }
        }
    }

    assert!(
        !ids.is_empty(),
        "issues.jsonl contains no issues — source of truth is degenerate"
    );

    for (line, from, to) in dependencies {
        assert!(
            ids.contains(&from),
            "line {line}: dependency issue_id '{from}' is not in issues.jsonl"
        );
        assert!(
            ids.contains(&to),
            "line {line}: dependency depends_on_id '{to}' is not in issues.jsonl"
        );
    }
}
