# Brief: catherine-alr — P7 Δ migration axis, PHASE A (RED fixtures + schema maps in format)

Ticket: P1, openspec change `add-delta-migration` (read proposal.md + tasks.md + design.md first). This is phase A of a phased implementation: tasks 1–4 (+ the hash-stability guard). Tasks 5–10 are later phases — do NOT start them.

## Context

Δ migration is the flagship capability: schema evolution with LOUD lost-data proofs (kata-10 pattern — a migration lists what it drops, never silently drops). P6 survived; this unblocks nq5 (P8). Practical target: bogart raw corpus → serving projection (task 8, later phase).

## Phase A scope

1. **RED fixtures (tasks 1–3)** under `fixtures/migration/`: (a) map dropping an object → the shape that will exhibit one lost-data finding per row; (b) field-narrowing map; (c) non-invertible round-trip pair. The fixtures are documents + maps whose EXECUTION comes in phase B — pin them now as parse-valid documents with their maps total (task 4's totality), plus documented expected phase-B outcomes in a fixtures/README or per-fixture `_expected` notes.
2. **Schema maps in format (task 4)**: new additive section within `format_version` additive policy — total maps only (every source object and arrow assigned; a partial map is a LABELED build error naming the unmapped element, line/column per the kernel's error discipline). Touch points: `crates/catherine-core/src/model.rs` (SchemaMap types), `crates/catherine-core/src/parse/` (parse + totality validation), `crates/catherine-core/schema/ct-schema-v1.json` (additive), regenerate `web/src/types/ct-schema-v1.ts` via `web/scripts/generate-types.mjs`.
3. **KNOWN_MINOR**: additive semantics land → raise KNOWN_MINOR in `crates/catherine-core/src/parse/schema.rs` per its documented convention (format_version 1.1 accepted).
4. **Hash-stability guard**: existing documents must hash IDENTICALLY before/after this change — the schema-map section is optional (skip-serialized when absent) and old fixtures' `model_hash` values are pinned by the existing roundtrip/parity tests. Add one explicit test: parse a pre-existing corpus fixture, assert `model_hash` equals the pre-change value (compute it now, before your change, and pin it in the test).

## Contract invariants

- Invent nothing at this phase: maps are declared, not executed (pullback/reindex = phase B).
- Partial map → labeled error naming the unmapped element (never silent default).
- Hash neutrality for all existing documents (the guard test proves it).
- proptest/structural assertions, never snapshots; kernel genesis-free; findings taxonomy untouched (lost-data findings = phase B).
- Conventions: file-headers on new files; pretender caps (≤400/≤40/cyc≤3 app, ≤3 test fns, nesting ≤2); clippy -D warnings everywhere; CHANGELOG entry expected (orchestrator commits); ct-schema TS types regenerated, never hand-edited.

## Target paths (exclusive)

- `crates/catherine-core/src/{model.rs,parse/,error.rs}` — types + parse + totality
- `crates/catherine-core/schema/ct-schema-v1.json` — additive schema
- `crates/catherine-core/tests/` — parse/totality/hash-stability tests
- `fixtures/migration/` — the three RED fixtures
- `web/src/types/ct-schema-v1.ts` — REGENERATED only

## Discipline

Edit only. You MAY run: `cargo check`/`cargo test -p catherine-core`, `cargo run -p catherine-ledger --bin ledger-check` style tool runs if needed, `cd web && npm run generate:types && npx tsc --noEmit`. NEVER: fmt/clippy on other targets/workspace gates, `just ci`, ah/pretender/lefthook, bd anything, git commits (orchestrator verifies + commits per phase).

## Acceptance (report with evidence)

1. Three RED fixtures parse, maps total, documented phase-B expectations.
2. Total maps accepted; partial map → labeled error naming the unmapped element (test + verbatim output).
3. KNOWN_MINOR raised per convention; format_version 1.1 documents.
4. Hash-stability test: pre-existing fixture hash pinned pre-change, asserted post-change, green.
5. ct-schema-v1.json additive; TS types regenerated; `tsc --noEmit` green.
6. tasks.md boxes 1–4 ticked; orchestrator commands listed; "bd close pending orchestrator" noted.