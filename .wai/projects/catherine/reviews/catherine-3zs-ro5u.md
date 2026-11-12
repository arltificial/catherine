# ro5u review — catherine-3zs (variant-coverage ratchet + advisory diff-coverage)

Full-diff lead-side review. Diff: `crates/catherine-cli/tests/variant_coverage.rs`
(new ratchet test), `lefthook.yml` (advisory pre-push block, outside the
ddl-managed markers), plus the salvaged fixture
`crates/catherine-cli/tests/fixtures/proposed_equation.ct.json` from the
credit-dead first implementor (schema-verified: it parses and its
`status: "proposed"` equation exercises the hypothesized weight through
the real `check` path).

Orchestrator verification: `variant_coverage` green; `just ci` green;
pretender gate green; the failure-driven iteration (contradiction kind
initially missing → search envelopes now harvested) is itself evidence
the ratchet bites.

Five passes:

1. Correctness: coverage runs the REAL binary over the REAL corpus —
   `check` across all fixtures, `underconstraint`/`redundant` over their
   dedicated fixtures, `search` in three bound configurations
   (contradiction / unknown / witness) — and every harvested finding
   must deserialize into the contract `Finding` (drift fails loudly).
2. Contract drift (class B): the exhaustive `match` name functions (no
   wildcard arms) pin each enum's variant set at compile time — a new
   variant breaks this test until the inventory grows (the ratchet).
3. Tests: the ticket IS a test; structural assertions only; the
   initial-red (missing contradiction coverage) was fixed by widening
   harvesting, never by weakening an assertion.
4. Negative matrix: no flag/surface added — the advisory block is a
   gate, deliberately report-only this phase per the ticket.
5. Regression risk: runtime adds ~15 CLI spawns (~0.05s measured);
   lefthook pre-push grows only the advisory block; missing-tool-safe.

Findings + dispositions:

- [ADJUDICATED, recorded] `DomainQuestion::UncoveredInputs` and
  `EpistemicLabel::Unknown` have no producer in v1 (hard constraint /
  render-reserved, cited inline in the test + module docs). Direct
  construction covers them; implementing producers would be P8 scope
  growth — rejected.
- [DEFERRED, LOW, written reason] the ~80% changed-lines diff-coverage
  GATE phase (ticket's stronger tier step 2) is deliberately not
  activated: the ticket itself phases it ("advisory for one phase, then
  gate"), and gating now would measure the tool, not the discipline.

max_critical = 0. Verdict: PASS.
