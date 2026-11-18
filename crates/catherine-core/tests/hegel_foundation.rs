//! Purpose: foundation canary for the hegel property engine (epic
//! catherine-7ko, child 7ko.1) — proves the pinned dev-dependency, the
//! workspace-root `hegel.toml` and the `#[hegel::test]` harness run
//! under `cargo test`, so the migrated suites cannot silently stop
//! executing.
//!
//! Responsibilities: draw one integer and assert it survives a decimal
//! round trip. That is enough to exercise a real generator, a real
//! draw, and a real case loop (100 local / 200 CI cases) without
//! duplicating any migrated property.
//!
//! Rationale: a missing dev-dep, an undiscovered config or an engine
//! link failure must surface as a compile or test error here first
//! rather than as an empty-looking suite elsewhere. Keep this file tiny:
//! it is wiring evidence, not a semantic oracle.

use hegel::generators as gs;
use hegel::TestCase;

#[hegel::test]
fn drawn_integers_survive_a_decimal_round_trip(tc: TestCase) {
    let n = tc.draw(gs::integers::<i64>());
    tc.note(&format!("drawn n = {n}"));
    let reparsed: i64 = n
        .to_string()
        .parse()
        .expect("an integer's decimal form parses back");
    assert_eq!(reparsed, n);
}
