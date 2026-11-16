# Change: Instance morphism check (declared mappings between instances)

> Grounding: cotorreo `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d`
> (gap **G2**, 2026-11-15: "a naturality failure… three runs tell the
> story — declaring the mapping as a field → silently dropped; 
> `distinguish` fires but only by luck of row alignment and answers
> 'instances differ', never 'this mapping cell is incoherent'");
> `docs/THEORY.md` names "natural transformation: coherent mapping
> between instances" a step-3 construction; DESIGN.md methodology E6
> lists `unupdatable` as requiring instance morphisms.

## Why

Two internally-consistent instances of the same schema (a CRM and a
warehouse, say) can be connected by a sync/ETL mapping that violates the
structure: the mapping sends a row whose region is `north` to a row whose
region is `south`. Today no surface can state the mapping as a claim:
declaring it as an instance field is silently dropped, and `distinguish`
compares observationally without ever checking a claimed map. The check
is finite and decidable — commutation of a given row map with each arrow
— which is exactly the tool's validated home turf, and it generalizes
what Δ-migration's row correspondence already computes in the special
case.

## What Changes

- **Format** (next free additive minor at RED — 4 at proposal time): an additive
  `instance_map` section in the *source* document:
  `{"maps": [{"object": O, "rows": {"srcRow": "tgtRow"}}]}` — a claimed
  row mapping from this document's instance to another instance of the
  same schema. Absent or empty → omitted from canonical bytes, so every
  existing document keeps its hash and every sealed ledger
  `model_hashes` entry stays valid.
- **Totality is required at parse**: the map must cover every row of
  every object of the schema. Incomplete maps fail with
  `incomplete_instance_map` at the offending node — a partial map is a
  malformed claim, not a silent identity-on-the-rest.
- **New `naturality` operation**: `catherine naturality <src>
  <tgt>` — schemas must be identical (reusing `distinguish`'s existing
  schema check); each commutation square is executed, and a break is a
  `violation`/`executed` finding with a witness naming the arrow, the
  source row, and both composed values (the witness union pattern keys
  established). No new finding kinds.
- **Target resolution is construction-time**: a map cell naming a target
  row that does not exist fails as a labeled error before any check runs
  — invalid states are rejected at construction, not reported as
  findings.
- **Scope guardrail (the G2 decision)**: verify a *given* mapping only.
  No transformation search, no map synthesis, no generic functor
  machinery — the map is data, the check is its execution.
- **Probe**: `ledger/probes/naturality/` — floor control on a sync defect
  `distinguish` already catches (zero unique detections required there);
  baseline arm is `distinguish` itself, committed and executed, with its
  known wrong-question behavior recorded.

## Impact

- Affected specs: new `instance-morphism` capability; `model-document`
  (additive section + versioning minor). `findings` untouched (witness
  union, no new kinds — the keys precedent); `migration` untouched (the
  map is same-schema and claimed, not derived).
- Affected code: `crates/catherine-core/src/{model,parse,canonical}`,
  `crates/catherine-analysis/src/` (commutation check),
  `crates/catherine-cli/src/` (subcommand + render),
  `schema/ct-schema-v1.json` + regenerated web types.
- Evidence: the crm/warehouse fixture from the gap suite — declared map
  that commutes → clean (exit 0); the `a3→w3` mapping → one
  `violation`/`executed` naming the arrow and row; incomplete map →
  labeled parse error; absent section → hash-neutral.
- Beads: feature ticket `catherine-s60` (RED→GREEN); refactor ticket
  filed at apply.
- Depends on: `add-expressibility-gap-suite` (its naturality rows must
  exist before this change's Verification flips them).
- Fixture provenance: the crm/warehouse fixtures are re-authored from
  the corpus transcripts (`kimi/1a1dee7a` G2 scenario) — the sandbox
  artifacts referenced there are provider-side and not recoverable.
- Conservative git profile: commits fine, no push/deploy.
