# Brief: catherine-ga7 — bogart corpus emits .ct.json instances (grounded-instance harness)

Ticket: P9 adjunct, complexity m. Works in `../bogart` (own repo/beads/AGENTS.md — read FIRST; POC policy). Catherine repo: read-only EXCEPT the one adjudicated kernel delta below. Run AFTER catherine-9vv lands (shared catherine tree; orchestrator sequences).

## Adjudications (bind you)

1. **Adapters live in bogart** (`scripts/` or `src/` tooling), NEVER in catherine-core (C4/E2: kernel smallness dies in adapters; specodelic 3.3k-line crate is the cautionary tale). anti_goals: no parsers in catherine-core; no adapter maintenance in kernel; no live/repo-filesystem coupling in catherine.
2. **Catherine-core delta — default ZERO.** The ticket allows "at most a source provenance field on instances". Add it ONLY if a provenance field can ride the existing schema/format discipline WITHOUT changing `model_hash` for any existing document (new optional field must be hash-neutral or the parity/hashing properties break — verify against `fixtures/parity/` + the roundtrip properties). If it cannot be hash-neutral, keep the kernel untouched and carry provenance in bogart's emission metadata (sidecar JSON next to the instance or the generator script's output naming) — and say so in the report. If you DO change the kernel: RED-first with parity + roundtrip regression tests, and the change must keep `openspec validate` + the ct-schema-v1.json artifact consistent (regenerate TS types via `web/scripts/generate-types.mjs` if the schema artifact changes).
3. **Seeded-defect validation is a quantified Must gate** (hc6 NEXT-ACTIONS precondition): inject N known schema/data mismatches into the slice (record N and each injection), run `catherine check`, and the acceptance is **N/N detected WITH witnesses** — misses reported honestly per-injection (what the model cannot say), never averaged away. N ≥ 5.
4. **File-headers on every new bogart file** (hc6 NEXT-ACTIONS precondition; follow bogart's existing header style).

## Scope (RED→GREEN→REFACTOR)

- RED: today there is NO corpus-to-instance path — instances are hand-authored (the hc6 pilot models were). Demonstrate the absence, then build the deterministic path.
- GREEN: bogart emits `.ct.json` INSTANCES for a corpus slice (deterministic extraction — the hc6 pilot generator `scripts/pilot_ct_instances.py` is the precedent pattern, generalize it into the grounded-instance harness); `catherine check` runs UNCHANGED over the emitted instance + the agent-proposed schema; a contradiction-with-data finding carries the offending row as witness. Demonstrate one contradiction-with-data: a schema that says impossible, a row that exists (inject it — that is the seeded-defect protocol's executable form).
- REFACTOR: extraction contract documented in bogart README (the reader recipe pattern: schema proposed by agent → instances extracted deterministically → check → findings are reality-checks).
- Catherine-side: per adjudication 2.

## Discipline

Edit only; scoped cargo check/test for your own targets allowed; catherine check runs allowed. NEVER: fmt/clippy/workspace gates, `just ci`, ah/pretender/bd/lefthook, git commits in catherine (bogart commits per its conventions ARE yours; never push), bd mutations (ga7 close is orchestrator-reserved).

## Acceptance (report with evidence)

1. RED: the absent corpus-to-instance path demonstrated (before/after).
2. Bogart emits a grounded `.ct.json` instance for a corpus slice; `catherine check` runs unchanged over it (exit recorded).
3. One contradiction-with-data demonstrated with the offending row as witness.
4. Seeded-defect Must gate: N injections, N/N detected with witnesses, or honest per-miss accounting.
5. File-headers on new bogart files; README extraction-contract section.
6. Kernel delta: landed with regression tests OR adjudicated to zero with the provenance carried bogart-side (report which).
7. Bogart commit SHAs; nothing pushed; exact orchestrator commands.
