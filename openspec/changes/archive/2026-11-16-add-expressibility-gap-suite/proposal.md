# Change: Expressibility gap suite (executable dispositions for known limitations)

> Grounding: cotorreo `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4`
> (2026-11-15: "a suite of 15 runnable cases… the runner prints `GAP`
> while a limitation holds and `CLOSED` once it changes") and the
> corpus-review finding that every limitation conclusion from the 11-15
> chat investigations lived only in `~/cotorreo` — no spec, no ticket,
> no executable trace in the repo.

## Why

The chat investigations produced precise, reproducible limitation
findings — identity laws unparseable, cones inert, unknown top-level
fields dropped, `distinguish`'s observation scope, vacuous clean on
wrong-shaped instances — but none of them were tracked: they existed
only as conversation text with sandbox-referenced repro scripts that
were never committed. The pattern-kata suite solved the same problem
for pattern coverage (`docs/ref/pattern-kata-dispositions.md` rows with
executed dispositions); the gap suite applies that discipline to
*expressibility limitations* — each known limitation becomes an
executable case whose assertion is pinned, so fixing a limitation
without flipping its disposition fails CI, and claiming a limitation
that no longer holds also fails CI. The failure mode this closes:
conclusions about the current implementation that outlive the chat they
were made in.

## What Changes

- **A disposition table** (`docs/ref/expressibility-dispositions.md`):
  one row per known limitation — case id, the limitation claim, the
  source conversation (provider + conv id), and the disposition
  (`gap` | `closed`) with the sealing evidence.
- **An executable case per row**: a fixture document plus a structural
  assertion of the claimed behavior (e.g. "declaring `cones` exits 0
  and is ignored", "an equation path of length 0 fails to parse"),
  runnable as an integration test. A case whose observation contradicts
  its row's disposition fails the suite — in either direction.
- **Registered initial rows** (from the chat investigations, behavior
  verified against HEAD at RED time): identity/empty-path equations,
  optional references, unknown top-level fields, declared cones,
  non-functorial `schema_map` handling, dropped equations on migrate,
  vacuous-clean instances, `distinguish` observation scope,
  associativity scaffolding cost, joint-check absence (composition),
  naturality absence (instance morphism), coproduct absence. Rows whose
  behavior already changed at registration are recorded `closed` with
  the fixing commit — the suite starts truthful, not nostalgic.
- **CI wiring**: the suite runs under `just ci`; adding a limitation
  claim in docs, the CHANGELOG, or an openspec close note without a
  corresponding row is a gate failure. The gate is **new work** — the
  pattern-kata disposition table ships without one — implemented as a
  simple grep gate over known doc locations (machine files like
  `.beads/issues.jsonl` are out of scope).
- **Scope guardrail**: the suite records *expressibility* limitations
  (what the tool cannot state or see) — not correctness bugs (those are
  ordinary defect-loop issues) and not domain claims (those are
  probes).

## Impact

- Affected specs: new `gap-suite` capability.
- Affected code: `crates/catherine-cli/tests/expressibility_gaps/`
  (fixtures + harness), `docs/ref/expressibility-dispositions.md`,
  `justfile` (CI wiring), the docs gate.
- Evidence: every row's assertion runs and matches its disposition at
  registration; the pattern-lens row registers `closed` citing the
  pattern-lens commits; flipping a row requires the same PR to update
  the table and the case.
- Beads: feature ticket `catherine-j6g` (RED→GREEN); refactor ticket
  filed at apply.
- Conservative git profile: commits fine, no push/deploy.
