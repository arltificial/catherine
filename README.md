# catherine

> cat, from *categories* — the small cat, not the whole zoo.

A browser-runnable, agent-usable category-theory modeling workbench:
**a small deterministic kernel that takes a model document (schema +
instance as JSON), executes its declared constraints, and returns
finding-typed diagnostics with witnesses — callable from a CLI by agents
and openable in a browser by humans.**

**Value proposition (one sentence):** catherine turns "do these
declarations actually hold?" from a human-reviewed diagram into an
executed, witness-backed check that any agent or browser can run — and
tells you when modeling is a waste of time.

## What you get

| Capability | What it means |
|---|---|
| Executed declarations | path equations run on instances with witness rows (Catlab declares, doesn't enforce — we enforce) |
| Finding-typed diagnostics | violation / dangling / contradiction / underconstraint / redundant — each with the concrete row/path that proves it |
| Bounded counterexample search | smallest breaking instance, or honest `UNKNOWN {scope}` — never a naked "no violation found" |
| Discovery questions | parallel paths without equations surface as domain questions (spec-ambiguity mechanism) |
| Redundancy & equivalence | derivable paths flagged; models compared via `distinguish` with witness inputs |
| Migration safety (P7) | Δ reindexing with loud lost-data counterexamples — no conventional counterpart exists |
| Rule lens (P8, hypothesized) | rules as subobjects + policy: gap/overlap/shadow/derivable; simplifications proven compliant |
| Epistemic labels | PROVED / EXECUTED / HYPOTHESIZED / UNKNOWN on every answer |

**The honest boundary is part of the product:** on rule-based or trivial
problems the tool says *"model this conventionally"* (FizzBuzz negative
control is a release gate).

## The CLI (P3)

The binary is `catherine` (`crates/catherine-cli`). Two commands share
one execution path (parse → build → execute all checks); they differ in
name only — `findings` is the findings-only entry point for agents,
never a divergent re-run:

```
catherine check <model.ct.json>      # every finding with kind, label, witness
catherine check <model.ct.json> --json   # the genesis-vibes JSON envelope, findings under data.findings
catherine findings <model.ct.json> [--json]
```

Human output implements the DESIGN §5 transcript (kind label, `eq-<n>`,
paths joined with `∘`/`≠`, witness on the indented continuation line);
parse errors and typing violations print as labeled errors with
line/column, stable code, and a remediation hint — never a stack trace.
Errors go to stderr, findings to stdout. `catherine.toml` (optional,
genesis config module) defaults `format` and `verbosity`; explicit
flags win.

Exit codes mirror finding classes — CI and agents branch on exit
status without parsing any text:

| Exit | Meaning | Class |
|---|---|---|
| 0 | clean — zero findings | — |
| 1 | parse/build failure — labeled error (parse error or typing violation), never a finding | — |
| 2 | usage error — clap's own code for bad args/unknown subcommand; never a finding exit | — |
| 3 | violation | highest severity |
| 4 | dangling | |
| 5 | contradiction | |
| 6 | underconstraint | |
| 7 | redundant | lowest severity |

Multi-class runs exit with the **minimum** present class code; the full
class set is always in the payload. The three analysis classes
(contradiction/underconstraint/redundant) have no producer until P4 —
their exit codes and renderings are pinned by unit tests, never
fabricated end-to-end.

## Documentation map

| Doc | Contents |
|---|---|
| [`docs/PROPOSAL.md`](docs/PROPOSAL.md) | why & what: evidence base, conclusions, value proposition |
| [`docs/THEORY.md`](docs/THEORY.md) | the mathematics: each construction, what it buys, where validated, reading order, glossary |
| [`docs/DESIGN.md`](docs/DESIGN.md) | how: architecture, kernel spec, technology decisions, phased plan |
| [`docs/USING.md`](docs/USING.md) | usage: agent recipes, human flow, what it enables, what it does NOT do |
| [`docs/ref/category_theoretic_modeling_methodology.md`](docs/ref/category_theoretic_modeling_methodology.md) | the modeling methodology the tool serves (vendored) |
| [`docs/ref/ct_methodology_playbook.md`](docs/ref/ct_methodology_playbook.md) | the methodology by problem type — drills, witnesses, stop rules; **works with no tool at all** |

## Status & roadmap

Design converged (Rule-of-5); implementation not started. Phases are
beads tickets (`bd list`), each a red→green→refactor cycle:

```
P0 kernel skeleton → P1 executed checks → P2 findings → P3 CLI (genesis)
→ P4 analysis → P5 wasm+web → P6 kata ledger (STOP CONDITION)
→ P7 migration Δ → P8 rule lens (hypothesis, pre-registered validation)
→ P9 bogart pilot (first real-domain dogfood)
```

`bd ready` shows the single unblocked ticket. If P6's ledger verdict is
`killed`, the project stops and records the negative result — that is the
pre-committed stop condition, not a failure mode.

## Tooling (ddl-managed)

- **gates**: lefthook pre-commit (`ah check`, pretender, `spk lint`) and
  pre-push (`ah check`) — wired via `ddl init --gates`
- **issues**: beads no-db mode; `.beads/issues.jsonl` is the tracked source
  of truth — after any `bd` change: `bd export -o .beads/issues.jsonl &&
  git add -f .beads/issues.jsonl`
- **knowledge**: whisper/turu workspace; repo-scope tracked, `.whisper/private/`
  is machine-local and never pushed
- **CLI plumbing**: `genesis-vibes` (CLI crate only; kernel stays suite-dep-free)
- **Code reuse**: specodelic acset (instance/query, parity-gated) + wild
  canonical hashing — see the reuse map in `DESIGN.md` §3; repo is
  Apache-2.0 to match the donors
- **pilot corpus**: bogart (`at/bogart`) — its openspec prose invariants are
  the first real-world modeling targets (P9)

## License

Apache-2.0 — see [LICENSE](LICENSE). Matches the code donors
(specodelic, wild, genesis); switched from the initial MIT before any code
landed because Apache-2.0 code cannot be relicensed to MIT.
