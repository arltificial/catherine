# Change: Add schema diff — accreted / weakened / refactor-equivalent classification
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Agents edit model documents; conventional diffs show textual change, not
whether an *enforced* constraint silently disappeared. Given two schemas
v1 and v2, the edit must be classified as **accreted** (constraints only
added — the instance space can only shrink), **weakened** (a path
equation is no longer enforced, caught with a witness instance v2
accepts and v1 rejects — the agent-drift detector), or
**refactor-equivalent** (no separating instance within the bounds — the
instance space is the same *in scope*). A rationale-only edit is the
negative-control silence: **cosmetic**, zero findings. This is the P10
verification surface on the E4 migration axis: the ticket is
`catherine-dvx`.

## What Changes

- `catherine-analysis::schema_diff::diff(v1, v2, bound)`: compare the
  **enforced** equation sets — the accepted equations' `(lhs, rhs)`
  pairs. Rationale, document order, and proposed equations are
  presentation and never classify an edit. The comparison needs no new
  epistemic machinery: it is a set diff plus the existing bounded
  `search::enumerate` engine and the kernel's own `check` predicate.
- Outcomes (reports beside the findings, not new finding kinds):
  - `cosmetic` — no enforced-constraint change (zero findings);
  - `accreted { added }` — v2's enforced set is a superset (zero
    findings; structural proof, no bounds needed);
  - `weakened { witness }` — a v1-enforced equation is gone and a
    witness instance valid in v2 / invalid in v1 exists; emits exactly
    one existing-taxonomy finding, `violation` / `executed`, with the
    violated v1 equation's canonical index and the composed values at
    its row;
  - `equivalent-in-scope { scope }` — no separating instance within the
    explored fully resolved bounded shape (the scope is part of the
    result; no schema-equivalence claim beyond it);
  - `strengthened-in-scope { added, scope }` — a mixed edit whose
    bounded scan found no weakening witness but did witness
    strengthening; never rounded into `accreted`;
  - `unknown { scope }` — the candidate budget expired; never a silent
    equivalence claim;
  - `incomparable` — objects/arrows differ: comparing instance spaces
    across vocabularies needs a declared schema map (the Δ axis), not a
    guess.
- Shared bounded enumeration primitive (`search::enumerate`, returning
  found / exhausted-with-scope / unknown-with-scope) so the diff and the
  search cannot drift on ordering or budget semantics.
- CLI `catherine diff <v1.ct.json> <v2.ct.json> [--rows OBJECT=COUNT]
  [--max-candidates N]`: envelope + human rendering; `weakened` exits
  through the existing finding-class table (3), every other completed
  classification exits 0 with the outcome in the payload,
  `incomparable` is a labeled error (exit 1).
- No finding-kind, format, or kernel change: the classification is an
  analysis-crate report; kernel delta is zero LOC.

## Impact

- Affected specs: `analysis` (additive requirements; consumes
  `findings`, `kernel-checks`)
- Affected code: `crates/catherine-analysis` (new `schema_diff` module,
  shared enumeration in `search`), `crates/catherine-cli` (new `diff`
  runner + renderer), `fixtures/schema_diff/` (three pairs), tests
- Beads: implements `catherine-dvx` (P10)
- Honest limits carried forward: "equivalent" is always
  *in scope*; the scope echoes the per-object row counts and candidates
  explored. Equivalence is decided over fully resolved instances of the
  bounded shape (the search witness contract); partially defined
  instances differ only by dangling cells, which are schema-independent
  when the arrows are equal. Bounds are inputs — a zero-size bound is
  `unknown` with the empty scope, never equivalence.