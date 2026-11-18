# Change: Pullback checks (declared pullback squares over finite instances)

> Grounding: beads `catherine-4vs` · aleandro `docs/synthesis.org` §3
> ("Pullbacks/cones **check nothing yet** — pullback-shaped ideas
> (reference resolution) stay HYPOTHESIZED prose") · aleandro
> `docs/relation-inventory.org` (Identity/aliases: "resolution is a
> *relation object*, not an equation") · aleandro `docs/findings.org`
> (first falsifiable experiment: reconstruct `R293 Art.3 → R909 Art.5
> as of 2025-07-26` across the 26/2019 + D406/2019 + D273/2025 carriers).

## Why

The limit half of catherine's declared-structure pair is missing: keys
(format 1.3) declared the mono side, coproducts (format 1.5) declared
the colimit side, and both execute. A declared pullback cannot even be
expressed — `schema.cones` is rejected with `unknown_field` (exit 1)
since the unknown-field rejection (`67afba1`; disposition
`declared-cones-ignored`, closed), so a pullback-shaped claim over an
instance is unstatable, and the vacuous-pass failure mode it once had
cannot return. The archived coproduct decision deferred pullbacks
"only when a fixture demands them": aleandro-2bc is that fixture — its
temporal reference-resolution experiment is HYPOTHESIZED-only because
catherine can verify none of it, and its inventory pins the shape:
co-reference resolution is a *relation object* (rows = the agreeing
pairs), not an equation over existing rows.

## What Changes

- **Format** (additive 1.6): `schema.pullbacks` — a list of
  `{"apex": P, "legs": [p1, p2], "cospan": [f, g], "status":
  "accepted" | "proposed"}` declarations, where `p1: P → A`, `p2: P →
  B` are the legs and `f: A → C`, `g: B → C` the cospan (`tgt f = tgt
  g`). The declaration asserts the pullback universal property over
  the finite instance as **three executed checks**:
  **commutes** — `f∘p1 = g∘p2` on every apex row; **covers** — every
  fiber pair `(a, b)` with `f(a) = g(b)` is projected by some apex
  row; **unique** — no two apex rows project to the same pair (the
  legs are jointly injective on `P`). Absent or empty → omitted from
  canonical bytes; every pre-existing document keeps its hash.
- **Kernel check**: accepted pullbacks run like accepted equations —
  a non-commuting apex row fires one `violation`/`executed` (witness:
  the square, the row, both composed values); a missing fiber pair
  fires one `violation`/`executed` (witness: the square and the
  uncovered pair); a duplicated projection fires one
  `violation`/`executed` (witness: the square and both rows).
  Proposed pullbacks fire `hypothesized`-labeled findings — the
  equation/key/coproduct status rule, verbatim. Status defaults to
  accepted when absent, as in every declared constraint. No new epistemic
  labels, no new exit codes. Decidability discipline mirrors
  coproducts: an apex row with a dangling leg cell keeps commutativity
  undecidable (the dangling findings carry it, never double-reported);
  a fiber pair touching a dangling cospan cell is excluded from
  decidable coverage — silence is real success, never an unevaluated
  claim.
- **Parsing**: one new stable code, `invalid_pullback` (undeclared
  apex or cospan object, undeclared arrow, leg not sourced at the
  apex, cospan target mismatch, repeated arrow names), mirroring the
  `invalid_coproduct` node discipline. `cones` continues to be
  rejected as `unknown_field` — no cone namespace is added.
- **`schema_diff`**: the enforced-constraint set becomes accepted
  equations ∪ keys ∪ coproducts ∪ pullbacks; adding one is `accreted`,
  dropping one is caught by the bounded weakening scan.
- **Bounded search**: accepted pullbacks judge satisfiability exactly
  like accepted equations, keys, and coproducts.
- **Scope guardrail (Decision 4 reaffirmed)**: one shape only — the
  pullback over a declared cospan. No general limit/cone machinery, no
  cones field, no defaults. Migration transport parity with coproducts
  (declarations carried through schema maps, a dropped declaration
  never silent) is an implementation task; anything beyond goes to the
  findings ledger, not new machinery.

## Impact

- **Specs**: `model-document` (+2 requirements: pullback declarations
  format 1.6; `invalid_pullback` node rejection), `kernel-checks`
  (+1 requirement: executed pullback checks with the three verdicts
  and the decidability discipline), `findings` (witness-shape
  requirement extended with the pullback witness), `analysis`
  (bounded-search accepted-constraint set extended).
- **Code**: `catherine-core` model/parse/checks/hash/schema_diff;
  `catherine-analysis` bounded search; `catherine-cli` envelope,
  docs; `ct-schema-v1.json` + `web` types.
- **Demand side**: aleandro-2bc's reference-resolution square becomes
  declarable and executable; the HYPOTHESIZED-only limitation in
  aleandro `synthesis.org` §3 lifts when this archives.
