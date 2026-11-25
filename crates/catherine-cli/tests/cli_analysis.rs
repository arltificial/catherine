//! Acceptance tests for the findings-bearing and traversal analysis
//! subcommands (`underconstraint`, `redundant`, `distinguish`, `query`)
//! — split out of `cli.rs` for the file-size gate; the spawn/parse
//! plumbing is the shared `tests/common` harness (`try_run`/`envelope`),
//! spelled for `?`.

mod common;

use common::{code, envelope, findings, stderr, stdout, try_run};
use std::error::Error;

const ANALYSIS_UNDERCONSTRAINT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_underconstraint.ct.json"
);
const ANALYSIS_REDUNDANT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_redundant.ct.json"
);
/// The same derivable equation declared twice (catherine-1lf).
const ANALYSIS_REDUNDANT_DUPLICATE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_redundant_duplicate.ct.json"
);
const ANALYSIS_DIST_A: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_dist_a.ct.json"
);
const ANALYSIS_DIST_B: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_dist_b.ct.json"
);
const ANALYSIS_QUERY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analysis_query.ct.json"
);

/// `underconstraint` on the parallel-paths fixture: exit 6 through the
/// existing decision; the hypothesized finding carries the flattened
/// domain-question witness with the question text intact.
#[test]
fn analysis_underconstraint_exits_6_with_hypothesized_payload() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["underconstraint", ANALYSIS_UNDERCONSTRAINT, "--json"])?;
    assert_eq!(code(&out), 6, "underconstraint exit code");
    let v = envelope("stdout", &stdout(&out))?;
    let found = findings(&v);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["kind"], "underconstraint");
    assert_eq!(found[0]["label"], "hypothesized");
    assert_eq!(found[0]["witness"]["case"], "parallel_paths");
    assert_eq!(
        found[0]["witness"]["detail"]["lhs"],
        serde_json::json!(["f", "g"])
    );
    assert_eq!(
        found[0]["witness"]["detail"]["rhs"],
        serde_json::json!(["h"])
    );
    assert_eq!(found[0]["witness"]["question"], "should these agree?");
    Ok(())
}

/// The human rendering keeps the label intact on the head line and the
/// question on the continuation line.
#[test]
fn analysis_underconstraint_human_lines_keep_the_label() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["underconstraint", ANALYSIS_UNDERCONSTRAINT])?;
    assert_eq!(code(&out), 6);
    let text = stdout(&out);
    assert!(
        text.contains("underconstraint: parallel paths f∘g / h (hypothesized)"),
        "label intact: {text}"
    );
    assert!(
        text.contains("should these agree?"),
        "question text: {text}"
    );
    Ok(())
}

/// `redundant` on the derivable-path fixture: exit 7; exactly the
/// route-spelling equation (`f,g,k`) proven derivable, with both
/// witness paths and the `executed` label.
#[test]
fn analysis_redundant_exits_7_with_both_witness_paths() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["redundant", ANALYSIS_REDUNDANT, "--json"])?;
    assert_eq!(code(&out), 7, "redundant exit code");
    let v = envelope("stdout", &stdout(&out))?;
    let found = findings(&v);
    assert_eq!(found.len(), 1, "only the derivable equation is claimed");
    assert_eq!(found[0]["kind"], "redundant");
    assert_eq!(found[0]["label"], "executed");
    assert_eq!(
        found[0]["witness"]["derivable"],
        serde_json::json!(["f", "g", "k"])
    );
    assert_eq!(found[0]["witness"]["reducing"], serde_json::json!(["n"]));
    Ok(())
}

/// `redundant` on a fixture declaring the same derivable equation twice:
/// one finding per derivable declaration — the duplicate declaration is
/// named twice on purpose (catherine-1lf), never collapsed. Exit stays 7.
#[test]
fn analysis_redundant_reports_duplicate_declarations_separately() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["redundant", ANALYSIS_REDUNDANT_DUPLICATE, "--json"])?;
    assert_eq!(code(&out), 7, "redundant exit code");
    let v = envelope("stdout", &stdout(&out))?;
    let found = findings(&v);
    assert_eq!(
        found.len(),
        2,
        "each of the two identical declarations is separately derivable: {found:?}"
    );
    for finding in found {
        assert_eq!(finding["kind"], "redundant");
        assert_eq!(
            finding["witness"]["derivable"],
            serde_json::json!(["f", "g", "k"])
        );
        assert_eq!(finding["witness"]["reducing"], serde_json::json!(["n"]));
    }
    Ok(())
}

/// `distinguish` over an explicit observation where the pair differs:
/// exit 0 with the observation set echoed and both values named.
#[test]
fn analysis_distinguish_names_the_difference_and_exits_0() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "distinguish",
        ANALYSIS_DIST_A,
        ANALYSIS_DIST_B,
        "--observe",
        "f@a#0",
        "--json",
    ])?;
    assert_eq!(code(&out), 0, "a completed distinguish run succeeds");
    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["envelope_kind"], "check");
    assert_eq!(v["data"]["outcome"], "distinguishable");
    assert_eq!(
        v["data"]["difference"]["observation"]["path"],
        serde_json::json!(["f"])
    );
    assert_eq!(v["data"]["difference"]["left"], 0);
    assert_eq!(v["data"]["difference"]["right"], 1);
    let observations = v["data"]["observations"]
        .as_array()
        .unwrap_or_else(|| panic!("observations"));
    assert_eq!(
        observations.len(),
        1,
        "the observation set is part of the result"
    );
    assert_eq!(observations[0]["object"], "a");
    Ok(())
}

/// catherine-18t: the second document carries its own schema, and a
/// different one is a labeled input error (`schema_mismatch`), exit 1 —
/// never a silent reinterpretation of B's tables under A's schema.
#[test]
fn analysis_distinguish_rejects_a_different_schema() -> Result<(), Box<dyn Error>> {
    const REV_SCHEMA: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/analysis_dist_revschema.ct.json"
    );
    let out = try_run(&[
        "distinguish",
        ANALYSIS_DIST_A,
        REV_SCHEMA,
        "--observe",
        "f@a#0",
        "--json",
    ])?;
    assert_eq!(code(&out), 1, "a differing schema is a labeled error");
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["ok"], false);
    assert_eq!(v["data"]["code"], "schema_mismatch");
    Ok(())
}

/// The same pair restricted to an observation where they agree:
/// `equivalent-in-scope` — in scope, and no claim beyond it.
#[test]
fn analysis_distinguish_restricted_set_is_equivalent_in_scope() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "distinguish",
        ANALYSIS_DIST_A,
        ANALYSIS_DIST_B,
        "--observe",
        "g@a#0",
        "--json",
    ])?;
    assert_eq!(code(&out), 0);
    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["data"]["outcome"], "equivalent-in-scope");
    assert!(
        v.get("data").and_then(|d| d.get("difference")).is_none(),
        "no difference is named on an in-scope equivalence"
    );
    assert_eq!(
        v["data"]["observations"]
            .as_array()
            .unwrap_or_else(|| panic!("observations"))
            .len(),
        1
    );
    Ok(())
}

/// A non-composing observation is a labeled input error, exit 1.
#[test]
fn analysis_distinguish_bad_observation_is_a_labeled_error() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "distinguish",
        ANALYSIS_DIST_A,
        ANALYSIS_DIST_B,
        "--observe",
        "g@b#0",
        "--json",
    ])?;
    assert_eq!(code(&out), 1);
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["data"]["code"], "bad_argument");
    assert!(v["data"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("message"))
        .contains("does not compose"));
    Ok(())
}

/// `query` forward closure over the query fixture: exit 0, sorted node
/// list under the genesis `list` kind; the dangling q1→? cell is never
/// followed.
#[test]
fn analysis_query_closure_exits_0_with_sorted_nodes() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "query",
        ANALYSIS_QUERY,
        "--seed",
        "p#0",
        "--follow",
        "f",
        "--follow",
        "g",
        "--json",
    ])?;
    assert_eq!(code(&out), 0);
    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["envelope_kind"], "list");
    assert_eq!(v["data"]["direction"], "forward");
    assert_eq!(
        v["data"]["nodes"],
        serde_json::json!([{"object": "p", "row": 0}, {"object": "q", "row": 1}]),
        "sorted closure; the dangling g(q1) cell is never followed"
    );
    Ok(())
}

/// A backward closure reaches the defined preimage only.
#[test]
fn analysis_query_backward_closure_reaches_defined_preimages() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "query",
        ANALYSIS_QUERY,
        "--seed",
        "r#0",
        "--follow",
        "g",
        "--direction",
        "backward",
        "--json",
    ])?;
    assert_eq!(code(&out), 0);
    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(v["data"]["direction"], "backward");
    assert_eq!(
        v["data"]["nodes"],
        serde_json::json!([{"object": "q", "row": 0}, {"object": "r", "row": 0}])
    );
    Ok(())
}

/// An unknown seed is a labeled error (`unknown_seed`), exit 1, error
/// envelope on stderr.
#[test]
fn analysis_query_unknown_seed_exits_1_with_error_envelope() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "query",
        ANALYSIS_QUERY,
        "--seed",
        "q#9",
        "--follow",
        "g",
        "--json",
    ])?;
    assert_eq!(code(&out), 1);
    assert_eq!(stdout(&out), "", "errors never write stdout");
    let v = envelope("stderr", &stderr(&out))?;
    assert_eq!(v["ok"], false);
    assert_eq!(v["envelope_kind"], "error");
    assert_eq!(v["data"]["code"], "unknown_seed");
    assert!(v["data"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("message"))
        .contains("q#9"));
    Ok(())
}

/// catherine-uh0: an undeclared `--follow` name is warned on stderr in
/// human mode too — `warning: <message>`, naming the typo'd arrow.
#[test]
fn analysis_query_human_mode_prints_follow_warning_to_stderr() -> Result<(), Box<dyn Error>> {
    let out = try_run(&[
        "query",
        ANALYSIS_QUERY,
        "--seed",
        "p#0",
        "--follow",
        "h",
        "--follow",
        "f",
    ])?;
    assert_eq!(code(&out), 0, "exit semantics unchanged by the warning");
    let err = stderr(&out);
    assert!(
        err.contains("warning: --follow names undeclared arrow `h`"),
        "human-mode stderr carries the warning line: {err:?}"
    );
    Ok(())
}
