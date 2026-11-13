# Change: Declarative keys (injectivity constraints)

> Grounding: `.wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md`
> §3 "Design, step 2 — declarative keys" and §7 row 2. Step 1 (`rules
> --project`) landed in `catherine-x94`; this change makes the same claim a
> hashed, ledger-able constraint instead of a CLI-time projection.

## Why

The pilot's only two real findings were one functional dependency:
`bogart-4wt` — two captures for one `(source, edition_date)`. A `.ct.json`
document has no way to declare that dependency. It can only be approximated by
a query-time arrow equation, so it surfaces as a dangling cell rather than as
the constraint that actually failed. The 8v9 re-model then could not represent
the FD at all: it encoded `(document_id, version)` as a row-naming convention,
which no consumer can recover from the model (composite arrows are step 3).

The projection bridge (`rules --project`) closes the measured blind spot, but
its claim lives outside the document hash and the ledger — it is evidence
about a projection, never sealed model evidence. A declared key is the missing
sealed form.

## What Changes

- **Format 1.3**: `schema.keys` — a list of `{"object": O, "arrows": [a, b],
  "status": "accepted" | "proposed"}` declarations. The tuple of those arrows'
  values is injective over `O`'s rows. `KNOWN_MINOR` 2 → 3. Absent or empty →
  the section is omitted from canonical bytes, so every pre-1.3 document keeps
  its hash and every sealed ledger `model_hashes` entry stays valid.
- **Kernel check**: accepted keys run like accepted equations — one
  `violation`/`executed` per colliding pair, proposed keys one
  `violation`/`hypothesized`; rows with a dangling (or typing-violated) cell in
  any key arrow are excluded (already reported, never double-reported).
- **Findings**: the `violation` payload becomes a witness union
  (`EquationWitness` | `KeyWitness`). Five finding kinds unchanged; the JSON
  shape for equation violations is byte-identical. This is the CORR-002 call —
  an overloaded `EquationWitness` (key index in `equation`, rows in
  `lhs`/`rhs`, a sentinel duplicated field) would make the same finding list
  ambiguous for every consumer.
- **`schema_diff`**: the enforced-constraint set becomes accepted equations ∪
  accepted keys. Adding a key classifies `accreted`; dropping one is caught by
  the same bounded weakening scan (the kernel's satisfaction predicate already
  treats `executed` violations as unclean, so key enforcement in the scan is
  automatic). Proposed keys stay presentation. The CLI's `added` payload gains
  a key shape (equation shape unchanged); exit codes unchanged.
- **Parsing**: one new stable code, `invalid_key` (undeclared object,
  undeclared arrow, arrow not sourced at the key's object, empty arrow list,
  duplicate arrow names inside one key); `invalid_status` is reused. Keys are
  stored in canonical order `(object, arrows, status)`; duplicates are kept
  (presentation repetition, as for equations). The published JSON Schema
  artifact gains `schema.keys`; `schema_map.target_schema` validates keys by
  the same passes (no migration-rule change: keys are not mapped, and the
  object/arrow totality rule is untouched).
- **Scope limit (CORR-003)**, stated in the format and in the help/docs: keys
  range over **arrows**, so a column-level FD such as
  `(source, edition_date)` is expressible only under the value-object
  convention (one row per distinct value, one arrow per column — the pilot's
  model 3) or through `rules --project`. Attribute-column keys would need the
  attribute layer that format 1.4 / step 5 has not built; inventing it here
  would mix an instance-level constraint with a presentation-level table.

## Impact

- Affected specs: `model-document`, `kernel-checks`, `findings`, `analysis`.
  `migration` is untouched (keys are not migration elements); `cli` is
  untouched (no new subcommand or exit class; the `diff` payload change is
  additive and unreferenced by a cli requirement).
- Affected code: `crates/catherine-core/src/{model,findings,checks}.rs`,
  `parse/`, `canonical/`, `schema/ct-schema-v1.json`,
  `crates/catherine-analysis/src/schema_diff.rs`,
  `crates/catherine-cli/src/{render,render/diff}.rs`,
  `web/src/findings.ts` + generated `web/src/types/ct-schema-v1.ts`.
- Evidence: a fixture whose key has a duplicated tuple emits one
  `violation`/`executed` per colliding pair; a clean key emits nothing; absent
  keys hash identically to today; a dropped key is classified `weakened` with
  its key witness.
- Beads: one feature ticket (RED→GREEN) plus one refactor ticket.
- Conservative git profile: commits fine, no push/deploy.