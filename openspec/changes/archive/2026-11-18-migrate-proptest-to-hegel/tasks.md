# Tasks

Tracks beads epic `catherine-7ko`; each group maps 1:1 to a child ticket.
Every migration group follows RED (seeded mutation caught) → GREEN (full
property set under `#[hegel::test]`) → REFACTOR (old file deleted in its
own step). Gates are quantified; anti-goals live in the tickets.

## 1. Foundation — beads `catherine-7ko.1`

- [ ] 1.1 Add `hegeltest =0.53.1` (exact pin) as dev-dependency to `crates/catherine-core/Cargo.toml` and `crates/catherine-analysis/Cargo.toml`; `cargo test -p catherine-core` still green
- [ ] 1.2 Add workspace-root `hegel.toml`: `[profiles.development]` (default 100 cases), `[profiles.ci]` (200 cases); smoke `#[hegel::test]` self-equality property runs green in a scratch integration test
- [ ] 1.3 Document the proptest→hegel idiom map as `//!` module doc in `crates/catherine-core/tests/common/mod.rs` (Strategy→Generator, flat_map→composite, prop_assert→assert+note, regressions→reproduce_failure)

## 2. Core roundtrip suites — beads `catherine-7ko.4`

- [ ] 2.1 RED: port serde roundtrip strategy to hegel generators in new `crates/catherine-core/tests/roundtrip_hegel.rs`; seed one product-side serialization fault (e.g. skipped field) and show the harness catches it; revert seed
- [ ] 2.2 GREEN: both old files' properties passing under `#[hegel::test]` — lossless roundtrip over arbitrary well-typed models; byte-identical repeated serialization; `golden_fixtures` still green
- [ ] 2.3 REFACTOR: delete `roundtrip.rs` + `roundtrip_symmetric.rs` and their `proptest-regressions` entries; reusable generators hoisted to `tests/common`

## 3. Core checks properties — beads `catherine-7ko.5`

- [ ] 3.1 RED: port `arb_case` chain to `#[hegel::composite]` shared generator in `tests/common`; seed a check() mutation (skip dangling detection) and show the harness catches it; revert seed
- [ ] 3.2 GREEN: total order on check output, label/kind soundness, dangling-witness declared-arrow + source-space, equation-index validity — all under `#[hegel::test]`; `check_fixtures` still green
- [ ] 3.3 REFACTOR: shared model generator is canonical in `tests/common`; delete `checks_properties.rs` + regressions entry

## 4. Core findings properties — beads `catherine-7ko.6`

- [ ] 4.1 RED: reuse shared generator; seed a findings-order mutation (swap two sorted findings) and show the harness catches it; revert seed
- [ ] 4.2 GREEN: every finding kind roundtrips from arbitrary witnesses; total order holds; serialization deterministic across repeated checks; `findings_fixtures` still green
- [ ] 4.3 REFACTOR: dedupe generators with group 3 into one `tests/common` module; delete `findings_properties.rs` + regressions entry

## 5. Analysis query parity — beads `catherine-7ko.7`

- [ ] 5.1 RED: port parity strategies to hegel generators in new `crates/catherine-analysis/tests/query_parity_hegel.rs`; seed one perturbed query result and show the harness catches it; revert seed
- [ ] 5.2 GREEN: full parity property set under `#[hegel::test]`; analysis regression suites still green
- [ ] 5.3 REFACTOR: delete `query_parity_properties.rs` + regressions entry; scrub the proptest doc-comment mention in `query_parity.rs`

## 6. Pullbacks order-stability conversion — beads `catherine-7ko.8`

- [ ] 6.1 RED: replace hand-rolled permutation loops with hegel vec/shuffle generators in new `pullbacks_hegel.rs`; seed a canonicalization skip (row order leaks into canonical bytes) and show the harness catches it; revert seed
- [ ] 6.2 GREEN: permuted presentations (row-name/table-key order, pullback declaration order) yield identical canonical bytes and identical finding vectors; `parity_gate` still green
- [ ] 6.3 REFACTOR: delete `pullbacks_properties.rs`

## 7. Cleanup gate — beads `catherine-7ko.9`

- [ ] 7.1 Remove `proptest` dev-deps from all three crate manifests (catherine-ledger's is dead — zero usages — drop independently if reached first); delete any remaining `proptest-regressions` files
- [ ] 7.2 Update conventions: "proptest, never snapshots" → "hegel, never snapshots" in test-file header docs and AGENTS.md; CONTRIBUTING/README proptest mentions scrubbed
- [ ] 7.3 Final gate: `rg -i proptest` zero hits repo-wide; property-count preserved per file (`grep -c '#\[test\]' @epic-base ≥ grep -c '#\[hegel::test\]' @close`); `cargo test` green across all three crates; `hegeltest` pin still exactly `=0.53.1`
