# Change: Coproduct exhaustiveness (closed variants as declared structure)

> Grounding: cotorreo `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d`
> (gap **G3**, 2026-11-15: "`Method = card | transfer | cash` cannot be
> declared closed… a `coproducts` block → silently ignored; a rogue
> `voucher` variant added as a row → clean"; `claude/b417f19b` case g3a/
> g3b — vacuous clean on wrong-shaped and zero-row instances when the
> only structural claim is an undeclared variant set).

## Why

Categorical data modeling's most programmer-facing construct after
arrows is the closed alternative — a coproduct. catherine already ships
the dual, limit-flavored half: `schema.keys` declares injectivity (the
mono side, format 1.3). The colimit half is missing: the variant set of
a sum type lives in instance rows, so adding a whole alternative is
presentation-only to the tool and a payment table silently accepts a
rogue variant. The gap analysis scoped the fix to catherine's validated
home turf: "coproduct + exhaustiveness first; pullbacks only when a
fixture demands them; no general limit machinery" — and both failure
modes are existing finding kinds, so no new epistemic machinery.

## What Changes

- **Format** (next free additive minor at RED — 4 at proposal time, 5 if
  `add-instance-morphism` archives first): `schema.coproducts` — a list of
  `{"object": C, "arrows": [i1, i2, ...], "status": "accepted" |
  "proposed"}` declarations. The arrows are the injection arrows
  (`V_k → C`, declared in `schema.arrows` as usual); the declaration
  asserts **coverage** (every row of `C` lies in the image of some
  injection) and **disjointness** (no row of `C` lies in the image of
  two injections). Same declaration grammar as keys — object + arrows +
  status. Absent or empty → omitted from canonical bytes; every pre-
  existing document keeps its hash.
- **Kernel check**: accepted coproducts run like accepted equations —
  an uncovered `C` row fires one `violation`/`executed` (the declared
  coverage is falsified on a named row); a row in two injection images
  fires one `violation`/`executed` naming the row and both injections
  (the declared disjointness is falsified). Proposed coproducts fire
  `hypothesized`-labeled findings, same rule as keys. No new finding
  kinds.
- **Parsing**: one new stable code, `invalid_coproduct` (undeclared
  object, undeclared arrow, arrow not sourced at a variant object or not
  targeted at `C`, empty arrow list, duplicate arrow names), mirroring
  `invalid_key` node discipline.
- **`schema_diff`**: the enforced-constraint set becomes accepted
  equations ∪ accepted keys ∪ accepted coproducts; adding one is
  `accreted`, dropping one is caught by the same bounded weakening scan
  (a dropped coproduct is `weakened` with a witness instance that
  accepts a rogue variant the v1 schema rejects).
- **Scope guardrail (the G3 decision)**: coproducts + exhaustiveness
  only. No pullbacks, no general limit/cone machinery, no
  coverage-with-defaults — declared totality or nothing.

## Impact

- Affected specs: `model-document` (additive section + versioning
  minor), `kernel-checks` (execution), `analysis` (enforced set — its
  MODIFIED delta is authored at apply, after `add-declarative-keys`
  archives; shared-requirement rule, design Decision 6), and `findings`
  (no new kinds — the witness-shape requirement gains the coproduct arm
  over the existing violation payload, the keys precedent).
- Affected code: `crates/catherine-core/src/{model,parse,checks,
  canonical}`, `crates/catherine-analysis/src/schema_diff.rs`,
  `crates/catherine-cli/src/render`, `schema/ct-schema-v1.json` +
  regenerated web types.
- Evidence: the payments fixture from the gap suite — rogue `voucher`
  row under a declared coproduct → `violation`/`executed` (exit 3,
  today exit 0); clean three-variant table → zero findings; absent
  coproducts → hash-neutral; dropped declaration → `weakened`.
- Beads: feature ticket `catherine-iib` (RED→GREEN); refactor ticket
  filed at apply.
- Depends on: `add-expressibility-gap-suite` (its coproduct rows must
  exist before this change's Verification flips them) and archive order
  `add-declarative-keys` → this change (shared-requirement rebase, design
  Decision 6).
- Fixture provenance: the payments fixtures are re-authored from the
  corpus transcripts (`kimi/1a1dee7a`, `claude/b417f19b`) — the sandbox
  artifacts referenced there are provider-side and not recoverable.
- Conservative git profile: commits fine, no push/deploy.
