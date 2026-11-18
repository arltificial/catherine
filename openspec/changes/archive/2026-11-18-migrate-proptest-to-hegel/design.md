# Design

## Context

Five suites carry all framework-based property testing today (audit
2026-11-18, verified by `rg -c proptest`): core `checks_properties.rs` (349
ln) and `findings_properties.rs` (294 ln) share near-identical `arb_case`
strategy machinery built from `tuples` + `prop_flat_map` + `prop_map` over
(shapes, row_counts, cells, eq_seeds, statuses); core `roundtrip.rs` and
`roundtrip_symmetric.rs` assert serde losslessness; analysis
`query_parity_properties.rs` (238 ln) guards the bounded-search parity
invariants. `pullbacks_properties.rs` hand-rolls permutation loops with no
framework. `catherine-ledger` declares `proptest = "1"` with zero usages.
Constraints: toolchain pinned at 1.98.1 (fmt/clippy behavior is part of the
gate); house rule "never snapshots — structural assertions by parsed
value"; golden/parity suites are the byte oracles and must stay green
throughout; every migrated file already carries `//! Purpose` headers.

## Goals / Non-Goals

**Goals:**
- One engine, zero proptest references at close (`rg -i proptest` empty)
- Property-for-property preservation, proven by a quantified count gate
  (`grep -c '#\[test\]' @base ≥ grep -c '#\[hegel::test\]' @close` per file)
- Every migration proven *wired*: the ported harness must catch a seeded
  product-code mutation before the old file is deleted
- Suite-wide runtime knobs without source edits (`HEGEL_TEST_CASES`,
  `HEGEL_SEED`, `HEGEL_STATISTICS`) and clean dev/ci profile separation

**Non-Goals:**
- Converting the nine framework-less ledger kata suites (new adoption, not
  migration — deferred until hegel's `stateful` module proves out)
- Adopting hegel's `stateful` model-based testing (no stateful property
  exists today; ledger rollup sequences are the future candidate)
- Any product-code change beyond the transient seeded mutations each RED
  step requires (reverted before GREEN completes)
- Upgrading/loosening the hegel pin during the epic

## Decisions

- **Exact pin `hegeltest =0.53.1`**, not semver `"0.53"`. Hegel is 0.x
  shipping ~20 releases/month (0.49→0.53 in one month; `hegeltest-macros`
  exact-pinned upstream). A floating dev-dep would let an unrelated
  `cargo update` break the gate mid-epic. Alternative rejected: tracking
  latest — churn cost lands on every ticket in flight. Mid-epic bump, if
  ever needed, is one deliberate change across all children at once.
- **Full migration, not coexistence.** Two PBT idioms in one repo is a
  standing cognitive tax; the suites are small enough (5 files, ~1200 ln)
  that a clean cutover is cheaper than maintaining both. (User decision,
  epic scope; the alternative — hegel for new suites only — was the
  original investigation recommendation and was superseded.)
- **RED = seeded-mutation proof, per suite.** Each port starts by
  temporarily breaking a product invariant (e.g. skip dangling detection;
  drop one findings sort key) and showing the new harness FAILS. This is
  the only way to distinguish "migrated" from "silently not exercising
  anything". Mutation is reverted before GREEN; the failing proof is a
  session-local observation, not a committed state.
- **Shared model generator hoisted to `tests/common`.** `arb_case` is
  duplicated in checks/findings today; the hegel port writes it once as a
  `#[hegel::composite]` generator (draws shapes → row counts → cells → eq
  seeds → statuses, feeding earlier draws into later ones exactly as the
  proptest `prop_flat_map` chain does) and both suites reuse it. Hoist
  lands with the checks child (first consumer); findings depends on it.
- **Idiom map** (documented as `//!` doc in `tests/common/mod.rs`):
  `Strategy<X>` → `Generator<X>`; `prop_flat_map`/`prop_map` →
  `#[hegel::composite]` sequential draws / generator `.map`;
  `prop_assert_eq!` → plain `assert_eq!` + `tc.note(...)` for counterexample
  context; `proptest-regressions` persistence files → `#[hegel::reproduce_failure("…")]`
  blobs (captured on failure, committed only when a real failure needs
  pinning); `HEGEL_TEST_CASES` replaces proptest's `CASES` env var.
- **`skip_specs: true`** — no capability invented. Product requirements
  (kernel-checks, findings, analysis, …) are untouched; properties migrate
  1:1. A "test-infra" capability would be a requirement with no system
  behavior to own.
- **Profiles: `development` (default 100 cases, hegel's own default) and
  `ci` (200 cases)** — CI eats ~2× the compute of a local run but stays
  under gate time budget; nightly-scale runs (`HEGEL_TEST_CASES=10000`)
  remain an on-demand env override, not a profile, until a nightly lane
  exists to hang them on. `ci` self-selects on CI servers (hegel
  environment detection), so no CI-config edit is required.

## Risks / Trade-offs

- **0.x churn mid-epic** → exact pin; single coordinated bump procedure
  written into the cleanup task. Trade-off: we forgo bug fixes landing
  upstream until the epic closes.
- **Different engine, different failure sets.** Hegel's generator/shrinker
  is not proptest's; suites that pass under proptest may surface a
  counterexample proptest never reached (better generation, not a
  regression). Policy: treat any new counterexample as a real finding —
  triage against the property, fix product or correct the property; never
  tune generators to make failures disappear.
- **Gate runtime cost.** ~41 new transitive dev-deps to compile once;
  hegel adds per-case engine overhead. Mitigation: profiles keep CI at
  200 cases; the crates' check binaries are unaffected (dev-only).
- **Transition window** (foundation merged, cleanup pending): both
  frameworks present. Bounded by the dependency chain; no ticket may
  *write new* proptest properties during the window — new properties wait
  for hegel or block the epic.
