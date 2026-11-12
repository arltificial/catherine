# ro5u review — catherine-1ml (symmetric-structure property ratchet)

Full-diff lead-side review. Diff: `crates/catherine-core/tests/roundtrip.rs`
plus its split-off `tests/roundtrip_symmetric.rs` (the ratchet file:
strategy + property + builders — the combined growth crossed the
pretender 400-line file cap, so the ratchet lives in its own file and
roundtrip.rs keeps the original round-trip properties)
only — new automorphism-rich strategy (`symmetric_model`: rotation cycles,
identity-pair involutions, twin rows, full collapse), a row-presentation
rotation helper, and the ratchet property asserting `model_hash` +
`canonical_bytes` invariance under row-permutation. No src/ changes
(ratchet, not fix). Budget note: the property lives in its own
`proptest!` block with an EXPLICIT 32-case envelope over sizes 2..=4 —
the automorphism-rich shapes are the canonicalizer's combinatorial worst
case (a 256-case × sizes≤8 draft took 428s), and the budget is documented
in-source as a trade, not a silent shrink.

Orchestrator verification: full 256-case draft GREEN (the assumption
holds at larger sizes too, evidence in session transcript); bounded
envelope green in the suite (`symmetric_ratchet_bounded_sizes ok`,
0.06s); `just ci` green after rebuild.

Five passes:

1. Correctness: rotation is a pure presentation-order permutation
   (tables reference row names); parse re-assigns dense RowIds in
   sorted-name order, so equal canonical forms are the correct
   expectation; both assertions (bytes + hash) are structural.
2. Contract drift: the ratcheted claim now matches its documentation —
   `canonical/label.rs:42-47` states the pruning assumption; the test
   cites it and exercises the pruning path (equal refined partitions
   recur in all four shapes).
3. Tests: this IS a test ticket; no snapshots (structural equality of
   canonical artifacts). Property deterministic given the strategy.
4. Negative matrix: no flag/surface. N/A.
5. Regression risk: none — test-only; no runtime code touched; budget
   bounded so `just ci` wall-clock stays dominated by compiles.

Findings + dispositions:

- [DEFERRED, LOW, written reason] CFI-family generator (the true
  counterexample class for 1-WL) not attempted within budget: a faithful
  CFI construction over base graphs exceeds this ticket's scope and the
  ratchet's value does not depend on it — the documented assumption is
  now guarded by executable evidence on the automorphism-rich shapes the
  ticket named (cycles/bipartite-pairs/twins/collapse). A CFI generator
  is a candidate follow-up ticket, not silently dropped.
- [NOTE] per-case cost grows combinatorially with row count; the
  documented envelope is load-bearing — future budget changes must
  re-measure `just ci` wall-clock.

max_critical = 0. Verdict: PASS.
