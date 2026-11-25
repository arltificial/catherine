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
//! The spawn/parse plumbing is the shared `tests/common` harness
//! (`try_run`/`envelope`), spelled for `?`.

mod common;

use common::{code, envelope, findings, stderr, stdout, try_run};
use std::error::Error;
use std::process::Output;

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

/// The weakened class is finding-bearing: exit 3, the `weakened`
/// outcome, the Node witness rows, and the violation finding's equation.
fn assert_weakened_witness(out: &Output) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        code(out),
        3,
        "weakened is a finding-bearing class (violation)"
    );
    let v = envelope("stdout", &stdout(out))?;
    assert_eq!(v["data"]["outcome"], "weakened");
    assert_eq!(
        v["data"]["witness"]["rows"]["Node"],
        serde_json::json!([0, 1])
    );
    assert_eq!(v["data"]["findings"][0]["kind"], "violation");
    assert_eq!(v["data"]["findings"][0]["label"], "executed");
    assert_eq!(v["data"]["findings"][0]["witness"]["equation"], 0);
    Ok(())
}

/// A budget smaller than the space is a completed run: exit 0, outcome
/// `unknown`, no witness, and the hint naming the explore flags.
fn assert_budget_expiry_is_unknown(out: &Output) -> Result<(), Box<dyn Error>> {
    assert_eq!(code(out), 0, "a completed bounded run exits 0");
    let v = envelope("stdout", &stdout(out))?;
    assert_eq!(v["data"]["outcome"], "unknown");
    assert_eq!(v["data"]["scope"]["candidates_explored"], 1);
    assert!(
        v["data"].get("witness").is_none(),
        "an unknown classification carries no witness"
    );
    assert_eq!(
        v["data"]["hint"],
        "bound exhausted; pass --rows OBJECT=N or --max-candidates=N to explore further",
        "the unknown classification names the flags that make it answerable"
    );
    Ok(())
}

/// No rows bound at all: `unknown` with the hint naming `--rows`.
fn assert_unbounded_is_unknown(out: &Output) -> Result<(), Box<dyn Error>> {
    assert_eq!(code(out), 0, "a completed bounded run exits 0");
    let v = envelope("stdout", &stdout(out))?;
    assert_eq!(v["data"]["outcome"], "unknown");
    assert_eq!(
        v["data"]["hint"], "no rows bound; pass --rows OBJECT=N",
        "an unknown classification with no rows bound names the --rows flag"
    );
    Ok(())
}

#[test]
fn weakened_edit_exits_3_with_witness_and_budget_expiry_is_unknown() -> Result<(), Box<dyn Error>> {
    let weakened = try_run(&[
        "diff",
        WEAKENED_V1,
        WEAKENED_V2,
        "--rows",
        "Node=2",
        "--json",
    ])?;
    assert_eq!(
        code(&weakened),
        3,
        "weakened is a finding-bearing class (violation)"
    );
    assert_weakened_witness(&weakened)?;
    assert_budget_expiry_is_unknown(&try_run(&[
        "diff",
        WEAKENED_V1,
        WEAKENED_V2,
        "--rows",
        "Node=2",
        "--max-candidates",
        "1",
        "--json",
    ])?)?;
    assert_unbounded_is_unknown(&try_run(&["diff", WEAKENED_V1, WEAKENED_V2, "--json"])?)?;
    Ok(())
}

#[test]
fn refactor_is_equivalent_in_scope_and_cosmetic_edit_is_silent() -> Result<(), Box<dyn Error>> {
    let refactor = try_run(&[
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
    ])?;
    assert_eq!(code(&refactor), 0);
    let v = envelope("stdout", &stdout(&refactor))?;
    assert_eq!(v["data"]["outcome"], "equivalent-in-scope");
    assert_eq!(v["data"]["scope"]["candidates_explored"], 18);
    assert_eq!(v["data"]["findings"], serde_json::json!([]));
    let cosmetic = try_run(&["diff", COSMETIC_V1, COSMETIC_V2, "--json"])?;
    assert_eq!(code(&cosmetic), 0);
    let c = envelope("stdout", &stdout(&cosmetic))?;
    assert_eq!(c["data"]["outcome"], "cosmetic");
    assert_eq!(
        c["data"]["findings"],
        serde_json::json!([]),
        "negative control is silent"
    );
    assert!(c["data"].get("scope").is_none(), "cosmetic needs no bounds");
    Ok(())
}

/// One labeled-error row of the diff surface: the run exits 1 through the
/// labeled-error path with `expected_code` on stderr.
fn assert_labeled_error(args: &[&str], expected_code: &str) -> Result<(), Box<dyn Error>> {
    let out = try_run(args)?;
    assert_eq!(code(&out), 1, "a labeled input error");
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["data"]["code"], expected_code, "the stable error code");
    Ok(())
}

/// An `--rows` object naming an undeclared object is ignored for the
/// bound but named on the warnings channel — exit unchanged.
fn assert_undeclared_bound_is_a_warning(out: &Output) -> Result<(), Box<dyn Error>> {
    assert_eq!(code(out), 3, "a warning never changes the exit decision");
    let w = envelope("stdout", &stdout(out))?;
    assert!(
        w["warnings"]
            .as_array()
            .unwrap_or_else(|| panic!("warnings array"))
            .iter()
            .any(|warning| warning["message"]
                .as_str()
                .map(|m| m.contains("Objet"))
                .unwrap_or(false)),
        "the ignored bound object is named: {w}"
    );
    Ok(())
}

#[test]
fn diff_surface_surfaces_malformed_input_honestly() -> Result<(), Box<dyn Error>> {
    assert_labeled_error(
        &["diff", WEAKENED_V1, INCOMPARABLE, "--json"],
        "schema_incomparable",
    )?;
    assert_labeled_error(
        &["diff", WEAKENED_V1, WEAKENED_V2, "--rows", "Node", "--json"],
        "bad_argument",
    )?;
    assert_labeled_error(
        &["diff", WEAKENED_V1, BAD_VERSION, "--json"],
        "unsupported_version",
    )?;
    let unknown_flag = try_run(&["diff", WEAKENED_V1, WEAKENED_V2, "--bogus", "--json"])?;
    assert_eq!(code(&unknown_flag), 2, "clap owns the usage code");
    assert_undeclared_bound_is_a_warning(&try_run(&[
        "diff",
        WEAKENED_V1,
        WEAKENED_V2,
        "--rows",
        "Node=2",
        "--rows",
        "Objet=1",
        "--json",
    ])?)?;
    Ok(())
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

// ── coproducts: the added-constraint payload and human line ─────────

const COPRODUCT_V1: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/coproduct/v1.ct.json"
);
const COPRODUCT_V2: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/schema_diff/coproduct/v2.ct.json"
);

/// An added accepted key is `accreted`, named in the payload with the
/// untagged key shape and in the human `added:` line — the equation shape
/// is unchanged.
#[test]
fn added_key_is_named_in_payload_and_human_output() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["diff", KEYS_V1, KEYS_V2, "--rows", "A=1", "--json"])?;
    assert_eq!(code(&out), 0, "accretion is not finding-bearing");
    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["data"]["outcome"], "accreted");
    assert_eq!(
        v["data"]["added"][0],
        serde_json::json!({"object": "A", "arrows": ["f"]}),
        "the key's untagged payload shape"
    );
    assert!(findings(&v).is_empty(), "accretion emits zero findings");

    let human = try_run(&["diff", KEYS_V1, KEYS_V2, "--rows", "A=1"])?;
    assert_eq!(code(&human), 0);
    assert!(
        stdout(&human).contains("  added: key A(f)"),
        "the human added line names the key: {}",
        stdout(&human)
    );
    Ok(())
}

/// A key collision renders with both rows in the human transcript and as
/// a key witness in the JSON payload.
#[test]
fn key_violation_renders_both_rows() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["check", KEY_VIOLATION])?;
    assert_eq!(code(&out), 3, "an accepted key violation exits 3");
    let human = stdout(&out);
    assert!(
        human.contains("violation: key-0 capture(capture_source∘capture_edition)"),
        "the key head names the object and tuple: {human}"
    );
    assert!(
        human.contains("  at rows capture#0, capture#1"),
        "the witness names both colliding rows: {human}"
    );

    let json = try_run(&["check", KEY_VIOLATION, "--json"])?;
    let v = envelope("stdout", &stdout(&json))?;
    assert_eq!(v["data"]["findings"][0]["kind"], "violation");
    assert_eq!(v["data"]["findings"][0]["label"], "executed");
    assert_eq!(v["data"]["findings"][0]["witness"]["key"], 0);
    assert_eq!(
        v["data"]["findings"][0]["witness"]["rows"],
        serde_json::json!([0, 1])
    );
    Ok(())
}

/// The human transcript names the accreted coproduct on the `added:`
/// line.
fn assert_human_line_names_the_coproduct() -> Result<(), Box<dyn Error>> {
    let human = try_run(&["diff", COPRODUCT_V1, COPRODUCT_V2, "--rows", "C=1"])?;
    assert_eq!(code(&human), 0);
    assert!(
        stdout(&human).contains("  added: coproduct C(i1∘i2)"),
        "the human added line names the coproduct: {}",
        stdout(&human)
    );
    Ok(())
}

/// An added accepted coproduct is `accreted`, named in the payload with
/// its `kind: "coproduct"` discriminator and in the human `added:` line —
/// the key payload shape stays unchanged.
#[test]
fn added_coproduct_is_named_in_payload_and_human_output() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "diff",
        COPRODUCT_V1,
        COPRODUCT_V2,
        "--rows",
        "C=1",
        "--json",
    ])?;
    assert_eq!(code(&out), 0, "accretion is not finding-bearing");
    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["data"]["outcome"], "accreted");
    assert_eq!(
        v["data"]["added"][0],
        serde_json::json!({"object": "C", "arrows": ["i1", "i2"], "kind": "coproduct"}),
        "the coproduct payload carries its discriminator"
    );
    assert!(findings(&v).is_empty(), "accretion emits zero findings");
    assert_human_line_names_the_coproduct()?;
    Ok(())
}
