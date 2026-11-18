# Change: Migrate property-based testing from proptest to hegel

> Beads epic: `catherine-7ko` (children `7ko.1`–`7ko.9`, issue-review verdict
> READY_TO_WORK) · Investigation provenance: docs.rs audit + scratch-repo probe,
> 2026-11-18. Pure test-tooling change: zero product requirements move, hence
> `skip_specs: true`.

## Why

Property suites are the project's soundness oracle — findings taxonomy,
check output ordering, roundtrip losslessness — but proptest gives us a
weaker engine than the state of the art: mediocre shrinking, no failure
database, no flakiness health checks, no suite-wide runtime controls.
`hegeltest` (lib `hegel`) is the Hypothesis-style engine for Rust by the
Hypothesis author (DRMacIver, Antithesis; MIT): Hypothesis-grade shrinking,
`reproduce_failure` blob replay, health checks, `HEGEL_TEST_CASES` /
`HEGEL_SEED` / `HEGEL_STATISTICS` env controls, and `hegel.toml` profiles
that separate development and CI runs. Verified on our pinned 1.98.1
toolchain: compiles clean, shrinks a failing `sum < 500` property to the
minimal `[500]`, replay blobs deterministic. Also discovered in the audit:
`catherine-ledger` declares a proptest dev-dep with zero usages — dead
weight to drop. Why now: the epic is filed and reviewed; the beads children
cite this change as their spec anchor (ALIGN-001), so it must exist before
`7ko.1` executes.

## What Changes

- Add `hegeltest =0.53.1` — **exact pin** (0.x shipping ~20 releases/month;
  subcrates exact-pinned upstream; upgrades are deliberate, never
  accidental) as dev-dependency to `catherine-core` and `catherine-analysis`
- Add workspace-root `hegel.toml` with `[profiles.development]` (local runs)
  and `[profiles.ci]` (auto-selected on CI servers)
- Migrate five proptest suites 1:1 to `#[hegel::test]` — every property
  preserved, none weakened or dropped:
  - `catherine-core`: `checks_properties.rs` (349 ln, the `arb_case`
    strategy machinery → shared `#[hegel::composite]` model generator),
    `findings_properties.rs` (294 ln), `roundtrip.rs`,
    `roundtrip_symmetric.rs`
  - `catherine-analysis`: `query_parity_properties.rs` (238 ln)
- Convert `catherine-core/tests/pullbacks_properties.rs` (hand-rolled
  permutation enumeration, framework-less) to hegel generators — randomized
  permutations reach order-stability violations the fixed enumeration
  missed
- Migration discipline per suite: **RED** — the ported harness must catch a
  seeded product-code mutation (a migration that only ever passes is not
  proven wired); **GREEN** — full property set passing under hegel;
  **REFACTOR** — old proptest file deleted, shared generators hoisted to
  `tests/common`
- Remove all proptest dev-deps — `catherine-core`, `catherine-analysis`,
  and `catherine-ledger`'s dead dep — plus `proptest-regressions` files and
  doc-comment mentions; `rg -i proptest` returns zero hits repo-wide
- Conventions updated: "proptest, never snapshots" house rule becomes
  "hegel, never snapshots" (structural assertions by parsed value only —
  unchanged rule, new framework)

Out of scope: the nine hand-rolled kata suites in `catherine-ledger`
(`state_discovery`, `wrapper_laws`, `memento_roundtrip`, …) — they use no
framework today, so converting them is new adoption, not migration. Revisit
only if hegel's `stateful` module proves out here (see design.md).

## Capabilities

### New Capabilities

(none — pure test-tooling refactor; `skip_specs: true`. The system's
observable behavior is unchanged: same properties, same verdicts, same
findings. No product spec requirement is added, modified, or removed, and
inventing a test-infrastructure capability would violate the template's
"do not invent a requirement just to satisfy validation" rule.)

### Modified Capabilities

(none — see above.)

## Impact

- **Dev-dependencies only**: `hegeltest =0.53.1` in `catherine-core` +
  `catherine-analysis`; proptest removed from all three crate manifests.
  ~41 transitive dev-deps added (rand, parking_lot, serde_json, macros) —
  zero runtime footprint in shipped artifacts.
- **Files**: new `hegel.toml` (workspace root); new `*_hegel.rs` test files
  (one per migrated suite, house `//! Purpose/Responsibilities/Rationale`
  headers); shared model generator hoisted into
  `crates/catherine-core/tests/common/`; five old suite files deleted.
- **CI**: hegel's `ci` profile activates automatically on CI runners
  (settings layer over `development`); nightly-style deeper runs available
  via `HEGEL_TEST_CASES` without source edits.
- **Not affected**: product code, public APIs, fixtures, ledger behavior,
  golden/parity oracles (`golden_fixtures`, `check_fixtures`, `parity_gate`
  must stay green throughout — they are the pass-to-pass safety net).
- **Risk**: hegel 0.x API churn between child executions → mitigated by the
  exact pin; if a minor bump is needed mid-epic it is a deliberate,
  reviewed change across all children at once.
