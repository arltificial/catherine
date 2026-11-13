//! Purpose: the `diff` CLI surface acceptance rows (openspec change
//! `add-schema-diff`, task 7 + validation 9–13) — the three fixture
//! pairs through the real binary, the exit table, and the surface's
//! malformed-input rows (the negative-matrix discipline: every seam
//! surfaces honestly, never silence).
//!
//! Responsibilities: assert the weakened class exits 3 through the
//! existing finding table and carries the witness; the bounded classes
//! exit 0 with their outcome and scope; the cosmetic negative control is
//! silent; incomparable documents are the labeled `schema_incomparable`
//! error; malformed flags and a bad second document are labeled or
//! usage errors; an undeclared `--rows` object is a warning, not a
//! silent drop.
//!
//! Rationale: probes the live binary (`CARGO_BIN_EXE_catherine`) and
//! asserts structurally (`serde_json::Value`, exit numerals) — no
//! snapshots. The `negative_matrix.rs` table is at its file-size gate;
//! this file carries the new surface's rows beside its acceptance tests.

use std::process::{Command, Output};

const WEAKENED_V1: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/weakened/v1.ct.json"
);
const WEAKENED_V2: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/weakened/v2.ct.json"
);
const REFACTOR_V1: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/refactor/v1.ct.json"
);
const REFACTOR_V2: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/refactor/v2.ct.json"
);
const COSMETIC_V1: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/cosmetic/v1.ct.json"
);
const COSMETIC_V2: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/cosmetic/v2.ct.json"
);
const BAD_VERSION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bad_version.ct.json"
);
const INCOMPARABLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/katas/bowling.ct.json"
);

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_catherine"))
        .args(args)
        .output()
        .expect("spawn catherine binary")
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn envelope(out: &Output) -> serde_json::Value {
    serde_json::from_str(&text(out))
        .unwrap_or_else(|e| panic!("stdout is one JSON envelope: {e}\n{}", text(out)))
}

#[test]
fn weakened_edit_exits_3_with_witness_and_budget_expiry_is_unknown() {
    let out = run(&[
        "diff",
        WEAKENED_V1,
        WEAKENED_V2,
        "--rows",
        "Node=2",
        "--json",
    ]);
    assert_eq!(
        code(&out),
        3,
        "weakened is a finding-bearing class (violation)"
    );
    let v = envelope(&out);
    assert_eq!(v["data"]["outcome"], "weakened");
    assert_eq!(
        v["data"]["witness"]["rows"]["Node"],
        serde_json::json!([0, 1])
    );
    assert_eq!(v["data"]["findings"][0]["kind"], "violation");
    assert_eq!(v["data"]["findings"][0]["label"], "executed");
    assert_eq!(v["data"]["findings"][0]["witness"]["equation"], 0);

    let expired = run(&[
        "diff",
        WEAKENED_V1,
        WEAKENED_V2,
        "--rows",
        "Node=2",
        "--max-candidates",
        "1",
        "--json",
    ]);
    assert_eq!(code(&expired), 0, "a completed bounded run exits 0");
    let next = envelope(&expired);
    assert_eq!(next["data"]["outcome"], "unknown");
    assert_eq!(next["data"]["scope"]["candidates_explored"], 1);
    assert!(
        next["data"].get("witness").is_none(),
        "an unknown classification carries no witness"
    );
}

#[test]
fn refactor_is_equivalent_in_scope_and_cosmetic_edit_is_silent() {
    let refactor = run(&[
        "diff",
        REFACTOR_V1,
        REFACTOR_V2,
        "--rows",
        "A=1",
        "--rows",
        "B=1",
        "--rows",
        "C=2",
        "--json",
    ]);
    assert_eq!(code(&refactor), 0);
    let v = envelope(&refactor);
    assert_eq!(v["data"]["outcome"], "equivalent-in-scope");
    assert_eq!(v["data"]["scope"]["candidates_explored"], 18);
    assert_eq!(v["data"]["findings"], serde_json::json!([]));

    let cosmetic = run(&["diff", COSMETIC_V1, COSMETIC_V2, "--json"]);
    assert_eq!(code(&cosmetic), 0);
    let c = envelope(&cosmetic);
    assert_eq!(c["data"]["outcome"], "cosmetic");
    assert_eq!(
        c["data"]["findings"],
        serde_json::json!([]),
        "negative control is silent"
    );
    assert!(c["data"].get("scope").is_none(), "cosmetic needs no bounds");
}

#[test]
fn diff_surface_surfaces_malformed_input_honestly() {
    let incomparable = run(&["diff", WEAKENED_V1, INCOMPARABLE, "--json"]);
    assert_eq!(
        code(&incomparable),
        1,
        "a vocabulary difference is a labeled error"
    );
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&incomparable.stderr))
        .expect("error envelope");
    assert_eq!(v["data"]["code"], "schema_incomparable");

    let bad_rows = run(&["diff", WEAKENED_V1, WEAKENED_V2, "--rows", "Node", "--json"]);
    assert_eq!(code(&bad_rows), 1);
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&bad_rows.stderr)).expect("error envelope");
    assert_eq!(v["data"]["code"], "bad_argument");

    let bad_doc = run(&["diff", WEAKENED_V1, BAD_VERSION, "--json"]);
    assert_eq!(code(&bad_doc), 1);
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&bad_doc.stderr)).expect("error envelope");
    assert_eq!(v["data"]["code"], "unsupported_version");

    let unknown_flag = run(&["diff", WEAKENED_V1, WEAKENED_V2, "--bogus", "--json"]);
    assert_eq!(code(&unknown_flag), 2, "clap owns the usage code");

    let warning = run(&[
        "diff",
        WEAKENED_V1,
        WEAKENED_V2,
        "--rows",
        "Node=2",
        "--rows",
        "Objet=1",
        "--json",
    ]);
    assert_eq!(
        code(&warning),
        3,
        "a warning never changes the exit decision"
    );
    let w = envelope(&warning);
    assert!(
        w["warnings"]
            .as_array()
            .expect("warnings array")
            .iter()
            .any(|warning| warning["message"]
                .as_str()
                .map(|m| m.contains("Objet"))
                .unwrap_or(false)),
        "the ignored bound object is named: {w}"
    );
}

// ── declarative keys: the added-key payload and human line ──────────

const KEYS_V1: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/keys/v1.ct.json"
);
const KEYS_V2: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/keys/v2.ct.json"
);
const KEY_VIOLATION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/key_violation.ct.json"
);

/// An added accepted key is `accreted`, named in the payload with the
/// untagged key shape and in the human `added:` line — the equation shape
/// is unchanged.
#[test]
fn added_key_is_named_in_payload_and_human_output() {
    let out = run(&["diff", KEYS_V1, KEYS_V2, "--rows", "A=1", "--json"]);
    assert_eq!(code(&out), 0, "accretion is not finding-bearing");
    let v = envelope(&out);
    assert_eq!(v["data"]["outcome"], "accreted");
    assert_eq!(
        v["data"]["added"][0],
        serde_json::json!({"object": "A", "arrows": ["f"]}),
        "the key's untagged payload shape"
    );
    assert!(
        v["data"]["findings"]
            .as_array()
            .expect("findings")
            .is_empty(),
        "accretion emits zero findings"
    );

    let human = run(&["diff", KEYS_V1, KEYS_V2, "--rows", "A=1"]);
    assert_eq!(code(&human), 0);
    assert!(
        text(&human).contains("  added: key A(f)"),
        "the human added line names the key: {}",
        text(&human)
    );
}

/// A key collision renders with both rows in the human transcript and as
/// a key witness in the JSON payload.
#[test]
fn key_violation_renders_both_rows() {
    let out = run(&["check", KEY_VIOLATION]);
    assert_eq!(code(&out), 3, "an accepted key violation exits 3");
    let human = text(&out);
    assert!(
        human.contains("violation: key-0 capture(capture_source∘capture_edition)"),
        "the key head names the object and tuple: {human}"
    );
    assert!(
        human.contains("  at rows capture#0, capture#1"),
        "the witness names both colliding rows: {human}"
    );

    let json = run(&["check", KEY_VIOLATION, "--json"]);
    let v = envelope(&json);
    assert_eq!(v["data"]["findings"][0]["kind"], "violation");
    assert_eq!(v["data"]["findings"][0]["label"], "executed");
    assert_eq!(v["data"]["findings"][0]["witness"]["key"], 0);
    assert_eq!(
        v["data"]["findings"][0]["witness"]["rows"],
        serde_json::json!([0, 1])
    );
}
