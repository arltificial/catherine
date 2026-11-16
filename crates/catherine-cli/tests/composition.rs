//! Purpose: RED acceptance for variadic `catherine check` — the joint
//! (composition) check over shared-subschema documents (openspec change
//! `add-model-composition`, beads `catherine-0qz`).
//!
//! Responsibilities: probe the real binary over the gap-suite
//! billing/shipping pair plus the seam-claim document, the disjoint and
//! parallel-path fixtures, and the captured single-document baseline; pin
//! the joint exit code, the empty-interface warning, source attribution on
//! `--json` findings, the unchanged exit-code precedence and single-document
//! bytes, and the post-glue underconstraint attribution.
//!
//! Rationale: design Decision 1 makes `check` (and the findings-bearing
//! analysis surface) variadic rather than a new `compose` subcommand, so
//! the joint run inherits the existing exit-code contract and envelope
//! shape. Assertions are structural (exit codes, envelope fields, finding
//! kinds) except the one mandated byte-identity check, whose baseline is
//! the envelope captured from the pre-change binary
//! (`fixtures/composition/single_document_baseline.json`).

mod common;

use common::{code, findings, parse_json, run, stderr, stdout};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
const GAP_SUITE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gap_suite");

fn fixture(path: &str) -> String {
    format!("{FIXTURES}/{path}")
}

/// The pre-change `--json` envelope for `bowling.ct.json` (captured from
/// the single-document path; the joint path must not perturb it).
fn baseline() -> String {
    let path = fixture("composition/single_document_baseline.json");
    std::fs::read_to_string(&path).expect("the captured single-document baseline")
}

/// The variable `meta.duration_ms` is not part of the contract; pin every
/// other byte of the envelope (the single-document output shape).
fn without_duration(text: &str) -> String {
    let marker = "\"duration_ms\":";
    let (before, rest) = text
        .split_once(marker)
        .expect("envelope carries meta.duration_ms");
    let digits = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    format!("{before}{marker}0{}", &rest[digits..])
}

fn gap(name: &str) -> String {
    format!("{GAP_SUITE}/{name}")
}

#[test]
fn joint_check_runs_over_billing_and_shipping() {
    let billing = gap("composition_billing.ct.json");
    let shipping = gap("composition_shipping.ct.json");
    for document in [&billing, &shipping] {
        let alone = run(&["check", document, "--json"]);
        assert_eq!(
            code(&alone),
            0,
            "each document checks clean alone: {}",
            stderr(&alone)
        );
    }
    let joint = run(&["check", &billing, &shipping, "--json"]);
    assert_eq!(
        code(&joint),
        0,
        "the glued pair is clean: {}",
        stderr(&joint)
    );
    assert_eq!(stderr(&joint), "", "a finding run writes no error stream");
    let value = parse_json(&stdout(&joint), "joint check");
    assert!(
        findings(&value).is_empty(),
        "no findings on the agreeing seam"
    );
}

#[test]
fn seam_violation_is_caught_across_documents() {
    let billing = gap("composition_billing.ct.json");
    let shipping = gap("composition_shipping.ct.json");
    let seam = gap("composition_seam_claim.ct.json");
    let out = run(&["check", &billing, &shipping, &seam, "--json"]);
    assert_eq!(
        code(&out),
        3,
        "the seam violation exits 3: {}",
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "a finding run writes no error stream");
    let value = parse_json(&stdout(&out), "seam check");
    let found = findings(&value);
    assert_eq!(found.len(), 1, "exactly the seam claim fails");
    assert_eq!(found[0]["kind"], "violation", "seam finding kind");
    assert_eq!(found[0]["label"], "executed", "accepted seam equation");
    assert_eq!(found[0]["witness"]["row"], 2, "order o3 is the seam row");
    let text = found[0].to_string();
    assert!(
        text.contains("composition_seam_claim"),
        "the declaring document is named: {text}"
    );
}

#[test]
fn disjoint_documents_warn_empty_composition_interface() {
    let a = fixture("composition/disjoint_a.ct.json");
    let b = fixture("composition/disjoint_b.ct.json");
    let out = run(&["check", &a, &b, "--json"]);
    assert_eq!(
        code(&out),
        0,
        "a disjoint union is clean, not an error: {}",
        stderr(&out)
    );
    let value = parse_json(&stdout(&out), "disjoint check");
    assert!(
        findings(&value).is_empty(),
        "no findings on a disjoint union"
    );
    let warnings = value["warnings"].to_string();
    assert!(
        warnings.contains("empty_composition_interface"),
        "the warning is labeled: {warnings}"
    );
    assert!(
        warnings.contains("disjoint_a.ct.json"),
        "names the first input: {warnings}"
    );
    assert!(
        warnings.contains("disjoint_b.ct.json"),
        "names the second input: {warnings}"
    );
}

#[test]
fn single_document_invocations_are_byte_identical() {
    let bowling = fixture("bowling.ct.json");
    let out = run(&["check", &bowling, "--json"]);
    assert_eq!(code(&out), 3, "the violation exit is unchanged");
    assert_eq!(
        without_duration(&stdout(&out)),
        without_duration(&baseline()),
        "the single-document --json envelope is byte-identical to the captured baseline"
    );
}

#[test]
fn joint_runs_keep_the_exit_code_precedence() {
    let multi = fixture("multi_class.ct.json");
    let bowling = fixture("bowling.ct.json");
    let out = run(&["check", &multi, &bowling, "--json"]);
    assert_eq!(
        code(&out),
        3,
        "violation outranks dangling: {}",
        stderr(&out)
    );
    let value = parse_json(&stdout(&out), "joint precedence");
    let kinds: Vec<&str> = findings(&value)
        .iter()
        .map(|finding| finding["kind"].as_str().unwrap_or_default())
        .collect();
    assert!(kinds.contains(&"violation"), "violation present: {kinds:?}");
    assert!(kinds.contains(&"dangling"), "dangling present: {kinds:?}");
}

#[test]
fn joint_findings_carry_source_attribution() {
    let billing = gap("composition_billing.ct.json");
    let shipping = gap("composition_shipping.ct.json");
    let seam = gap("composition_seam_claim.ct.json");
    let out = run(&["check", &billing, &shipping, &seam, "--json"]);
    let value = parse_json(&stdout(&out), "seam attribution");
    let text = findings(&value)[0].to_string();
    assert!(
        text.contains("composition_seam_claim"),
        "the declaring document is attributed: {text}"
    );
    assert!(
        text.contains("composition_billing"),
        "the lhs path's declaring document is attributed: {text}"
    );
    assert!(
        text.contains("composition_shipping"),
        "the rhs path's declaring document is attributed: {text}"
    );
}

#[test]
fn joint_coproduct_findings_carry_source_attribution() {
    let a = fixture("composition/coproduct_a.ct.json");
    let b = fixture("composition/coproduct_b.ct.json");
    let out = run(&["check", &a, &b, "--json"]);
    assert_eq!(
        code(&out),
        3,
        "the merged declarations are finding-bearing: {}",
        stderr(&out)
    );
    assert_eq!(stderr(&out), "", "a finding run writes no error stream");
    let value = parse_json(&stdout(&out), "joint coproduct check");
    let found = findings(&value);
    assert_eq!(
        found.len(),
        2,
        "one coverage finding per merged declaration"
    );
    assert_eq!(
        found[0]["witness"],
        serde_json::json!({"coproduct": 0, "row": 1}),
        "declaration a leaves the row only b's injection covers"
    );
    assert_eq!(
        found[1]["witness"],
        serde_json::json!({"coproduct": 1, "row": 0}),
        "declaration b leaves the row only a's injection covers"
    );
    let sources: Vec<Vec<String>> = found
        .iter()
        .map(|finding| {
            finding["source"]
                .as_array()
                .expect("joint findings carry source attribution")
                .iter()
                .map(|document| document.as_str().expect("document name").to_string())
                .collect()
        })
        .collect();
    assert!(
        sources[0][0].ends_with("coproduct_a.ct.json"),
        "the declaring document is attributed first: {sources:?}"
    );
    assert!(
        sources[1][0].ends_with("coproduct_b.ct.json"),
        "the second declaration keeps its own declaring document first: {sources:?}"
    );
}

#[test]
fn post_glue_parallel_paths_are_attributed() {
    let a = fixture("composition/parallel_a.ct.json");
    let b = fixture("composition/parallel_b.ct.json");
    for document in [&a, &b] {
        let alone = run(&["underconstraint", document, "--json"]);
        assert_eq!(
            code(&alone),
            0,
            "one path per document raises no question: {}",
            stderr(&alone)
        );
    }
    let joint = run(&["underconstraint", &a, &b, "--json"]);
    assert_eq!(
        code(&joint),
        6,
        "the glued parallel paths raise a hypothesis: {}",
        stderr(&joint)
    );
    let value = parse_json(&stdout(&joint), "joint underconstraint");
    let found = findings(&value);
    assert_eq!(found.len(), 1, "exactly the post-glue pair");
    assert_eq!(found[0]["kind"], "underconstraint", "the question kind");
    let text = found[0].to_string();
    assert!(
        text.contains("parallel_a.ct.json"),
        "first document attributed: {text}"
    );
    assert!(
        text.contains("parallel_b.ct.json"),
        "second document attributed: {text}"
    );
}

#[test]
fn single_document_unknown_arrow_error_is_unchanged() {
    let path = fixture("composition/unknown_arrow.ct.json");
    let out = run(&["check", &path, "--json"]);
    assert_eq!(code(&out), 1, "a labeled parse error exits 1");
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let value = parse_json(&stderr(&out), "unknown arrow");
    assert_eq!(value["data"]["code"], "unknown_arrow", "stable error code");
    let message = value["data"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("path [f, q] references undeclared arrow `q`"),
        "the kernel message shape is unchanged: {message}"
    );
    assert!(
        message.contains("(line 10, column 21)"),
        "the offending node position is unchanged: {message}"
    );
}

/// Whether the envelope's warnings channel carries `rule`.
fn warns(value: &serde_json::Value, rule: &str) -> bool {
    value["warnings"].to_string().contains(rule)
}

#[test]
fn repeated_document_paths_warn_without_deduplicating_findings() {
    let bowling = fixture("bowling.ct.json");
    let out = run(&["check", &bowling, &bowling, "--json"]);
    assert_eq!(code(&out), 3, "still exits 3");
    assert_eq!(stderr(&out), "", "JSON warnings stay on the envelope");
    let value = parse_json(&stdout(&out), "duplicated check");
    let found = findings(&value);
    assert_eq!(found.len(), 2, "both occurrences execute");
    for finding in found {
        assert_eq!(finding["kind"], "violation", "never deduplicated");
    }
    assert!(warns(&value, "duplicate_document_input"), "labeled");
    assert!(warns(&value, "bowling.ct.json"), "names the path");
    let human = run(&["check", &bowling, &bowling]);
    assert!(
        stderr(&human).contains("passed more than once"),
        "human warning: {}",
        stderr(&human)
    );
}

#[test]
fn only_repeated_document_paths_warn() {
    let bowling = fixture("bowling.ct.json");
    let single = run(&["check", &bowling, "--json"]);
    let value = parse_json(&stdout(&single), "single document");
    assert!(!warns(&value, "duplicate_document_input"), "no warning");
    let disguised = fixture("./bowling.ct.json");
    let repeated = run(&["check", &bowling, &disguised, "--json"]);
    let value = parse_json(&stdout(&repeated), "canonicalized repeat");
    assert!(warns(&value, "duplicate_document_input"), "canonicalized");
}

#[test]
fn findings_and_underconstraint_warn_on_repeated_documents() {
    let bowling = fixture("bowling.ct.json");
    let found = run(&["findings", &bowling, &bowling, "--json"]);
    let value = parse_json(&stdout(&found), "duplicated findings");
    assert!(warns(&value, "duplicate_document_input"), "findings");
    let parallel = fixture("composition/parallel_a.ct.json");
    let joint = run(&["underconstraint", &parallel, &parallel, "--json"]);
    assert_eq!(code(&joint), 0, "raises nothing");
    let value = parse_json(&stdout(&joint), "duplicated underconstraint");
    assert!(warns(&value, "duplicate_document_input"), "warns");
}

#[test]
fn naturality_warns_when_both_documents_are_identical() {
    let identity = fixture("naturality/identity_map.ct.json");
    let out = run(&["naturality", &identity, &identity, "--json"]);
    assert_eq!(code(&out), 0, "degenerate map is clean");
    let value = parse_json(&stdout(&out), "identical naturality paths");
    assert!(warns(&value, "duplicate_document_input"), "warned");
    assert!(findings(&value).is_empty(), "map holds");
}
