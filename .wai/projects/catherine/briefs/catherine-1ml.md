# Brief: catherine-1ml — symmetric-structure property ratchet (WL-pruning assumption)

Ticket: P2, test-only ratchet unless a real kernel bug surfaces.

## Target paths

- `crates/catherine-core/tests/roundtrip.rs` — add the biased strategy + property here (reuse the existing `arb_model()` helpers/imports; keep file-cohesive)
- NO src/ changes expected. If the property FAILS on generated symmetric models, STOP: that is a kernel bug in the 1-WL partition pruning (`crates/catherine-core/src/canonical/label.rs:51` solve(), documented caveat at :42-47) — report it with the shrinking counterexample; do NOT fix, do NOT weaken the property.

## Contract

- The assumption being ratcheted (label.rs:42-47): branches whose refined partitions are equal are skipped as equivalent — sound only under the 1-WL automorphism assumption (CFI graphs = known counterexample class). Random models (current `arb_model()`) rarely generate the self-symmetric structures where pruning bites.
- Add a proptest strategy biased toward automorphism-rich models: complete bipartite row-structures, cycles (row i → row i+1 mod n), duplicated identical components (repeated identical table/value blocks), and random row-order permutations of those.
- Property: for every generated model M and any permutation of its row order/serialization order, `hash::model_hash` is invariant (the canonical form must not depend on row presentation order — pruning correctness depends on it). Include at least one CFI-family attempt if reachable with a small generator (e.g. the classic CFI construction over a base cycle with per-vertex twin-row swaps); if not reachable within ~60 lines of generator, say so honestly in the report instead of pretending.
- RED/GREEN discipline: this is a ratchet — write the property first, expect GREEN on current code; a RED result is a bug report, not a fix trigger.
- Structural assertions; proptest, never snapshots. Config: `ProptestConfig::with_cases(256)` matching the file.

## Discipline

Edit only. `cargo check -p catherine-core --tests` + `cargo test -p catherine-core --test roundtrip` allowed as compile/property feedback (test binary only). NEVER: fmt/clippy/other test targets/workspace gates, `just ci`, ah/pretender/bd/lefthook, any git commit, any bd mutation. Orchestrator verifies + commits.

## Acceptance (report with evidence)

1. Biased strategy + permutation-invariance property compile and pass (orchestrator reruns).
2. The strategy demonstrably generates automorphism-rich instances (say how: e.g. generator coverage counts in a comment or a debug assertion run).
3. CFI attempt: landed or honestly reported as out-of-reach-within-budget.
4. No src/ changes; if a bug was found: shrinking counterexample + exact property that failed, STOPPED before any fix.
