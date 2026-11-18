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
| Migration safety (P7) | Δ reindexing with loud lost-data counterexamples — measured against a committed conventional baseline (statement lint + row-count/checksum dry run) on three pilot-scale fixtures: 10 named cell losses and 1 round-trip counterexample the baseline cannot identify ([probe ledger](fixtures/probes/migration/README.md), entries `ledger/probes/migration/`, `catherine-1od`) |
| Schema diff (P10) | v1→v2 edits classified accreted / weakened-with-witness (an instance v2 accepts and v1 rejects) / equivalent-in-scope / cosmetic (silent) |
| Rule lens (P8, executed; scoped to extracted tables) | rules as subobjects + policy: gap/overlap/shadow/derivable; simplifications proven compliant. Scoped to **extracted** prose/PDF tables no DMN analyzer ingests — authored decision tables are commodity (Calvanese et al., [arXiv:1603.07466](https://arxiv.org/abs/1603.07466); Signavio, OpenRules); extracted-table evidence ([probe ledger](fixtures/probes/rule_lens/README.md), entries `ledger/probes/rule-lens-extracted/`, `catherine-1od`) |
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
catherine check <model.ct.json>      # execution findings — violation (equation,
                                     # key, coproduct or pullback), dangling — with
                                     # kind, label, witness
catherine check <model.ct.json> --json   # the genesis-vibes JSON envelope, findings under data.findings
catherine findings <model.ct.json> [--json]
```

`check` executes the checks that need no search. The analysis classes are
separate subcommands — `search` (contradiction), `underconstraint`,
`redundant`, `distinguish` — and return their own exit codes; the exit
table below lists them.

Human output implements the DESIGN §5 transcript (kind label, `eq-<n>`,
paths joined with `∘`/`≠`, witness on the indented continuation line);
parse errors and typing violations print as labeled errors with
line/column, stable code, and a remediation hint — never a stack trace.
Errors go to stderr, findings to stdout. `catherine.toml` (optional,
genesis config module) defaults `format` and `verbosity`; explicit
flags win.

The `pattern` surface runs one registered design-pattern lens by its exact
id — `catherine pattern <model.ct.json> [--id <pattern>] [--values ...]
[--list]` — with the same findings envelope and exit table as `rules`, an
`unknown` outcome plus its law scope for a bounded non-answer, and a
labeled usage error (exit 2) for an omitted or unknown id. `--list` prints
each registered lens's id, name, abstraction, law kind, and scope and runs
nothing; the registry ships the twelve executed pattern lenses, so `--list`
is the installed inventory:

```
catherine pattern fixtures/katas/visitor-sum-types/visitor.ct.json --list
```

The `naturality` surface verifies a *given* row mapping between two
instances of identical schema — `catherine naturality <src> <tgt>
[--json]`. The map is declared in the source document's additive
`instance_map` section (`{"maps": [{"object": O, "rows": {"src": "tgt"}}]}`,
format 1.4), so the claim is hashed, diffable, and sealed model evidence.
Every schema arrow is executed against every mapped row: `map∘a` and
`a∘map` must agree, and a break is one `violation`/`executed` naming the
arrow, the source row, and both composed values. Totality is enforced at
parse (`incomplete_instance_map`); a cell naming a missing target row is
a construction-time `unknown_map_target` error; schemas must be
identical (the `distinguish` check). The surface verifies a mapping —
it never synthesizes or searches for one, and it emits no merged
document.

`check` and `findings` are variadic over documents — `catherine check
a.ct.json b.ct.json …` glues the inputs by shared declared name and runs
every input's equations and keys over the merged instance, with findings
carrying an additive `source` attribution naming the declaring documents.
Overlapping declarations must agree: disagreeing arrow endpoints are a
labeled `schema_conflict`; a shared object's differing row sets, or a
commonly-declared cell valued differently (including `null` against a
value), a `row_conflict`; and a glue sharing no object, no arrow, and no
`(object, row)` pair emits the `empty_composition_interface` warning
(stderr in human mode) rather than passing silently. A cross-document
equation lives in an input document, so the seam claim is versioned,
reviewed, and enforced by the joint run with source attribution — and the
same document alone is rejected with `unknown_arrow` unless it also
declares every arrow it names, so only that self-contained form carries an
ordinary model hash and the `diff` surface. A document whose equations
resolve alone reports the same first error whether it is run alone or
jointly, and single-document runs keep today's bytes and exit codes
exactly. The surface reports findings — it never emits a merged document.
`underconstraint` accepts the same joint form for seam-only parallel-path
questions and labels a disjoint union the same way.

Closed variants are declared, not inferred: the additive
`schema.coproducts` section (format 1.5) lists
`{"object": C, "arrows": ["V_k → C", …], "status": "accepted" | "proposed"}`
— the `keys` grammar with coverage/disjointness semantics. An accepted
declaration is executed by `check`: a row of `C` outside every injection
image is one `violation`/`executed` naming the row, a row in two images one
naming the row and both injections; `proposed` declarations fire
`hypothesized`. A malformed declaration is `invalid_coproduct` at its node,
and documents without the section keep their exact bytes and hash.

Pullback squares are declared, not inferred: the additive
`schema.pullbacks` section (format 1.6) lists
`{"apex": P, "legs": [p1, p2], "cospan": [f, g], "status": "accepted" | "proposed"}`
— the `keys` grammar with universal-property semantics. An accepted
declaration is executed by `check` on three separately decidable
verdicts: `commutes` (an apex row's two leg compositions disagree),
`covers` (a fiber pair over a common cospan target has no apex row
projecting to it), and `unique` (two apex rows project to the same pair);
`proposed` declarations fire `hypothesized`. A malformed declaration —
undeclared object or arrow, a leg not sourced at the apex, cospan targets
that differ, wrong arity, or a repeated arrow — is `invalid_pullback` at
its node, and documents without the section keep their exact bytes and
hash.

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
class set is always in the payload. The three analysis classes have real
producers: `search` (contradiction), `underconstraint` (parallel paths),
and `redundant` (derivable paths), with `distinguish` and the pattern/
rule lenses alongside them.

## Documentation map

| Doc | Contents |
|---|---|
| [`docs/PROPOSAL.md`](docs/PROPOSAL.md) | why & what: evidence base, conclusions, value proposition |
| [`docs/THEORY.md`](docs/THEORY.md) | the mathematics: each construction, what it buys, where validated, reading order, glossary |
| [`docs/DESIGN.md`](docs/DESIGN.md) | how: architecture, kernel spec, technology decisions, phased plan |
| [`docs/USING.md`](docs/USING.md) | usage: agent recipes, human flow, what it enables, what it does NOT do |
| [`docs/ref/category_theoretic_modeling_methodology.md`](docs/ref/category_theoretic_modeling_methodology.md) | the modeling methodology the tool serves (vendored) |
| [`docs/ref/ct_methodology_playbook.md`](docs/ref/ct_methodology_playbook.md) | the methodology by problem type — drills, witnesses, stop rules; **works with no tool at all** |
| [`docs/ref/pattern-kata-dispositions.md`](docs/ref/pattern-kata-dispositions.md) | GoF pattern → abstraction disposition table: which equivalences became executable katas, which were killed, and which patterns have no law on this surface |
| [`docs/ref/expressibility-dispositions.md`](docs/ref/expressibility-dispositions.md) | the registry of known expressibility limitations: each row pairs a limitation claim with an executable CLI case and a `gap`/`closed` disposition, checked in both directions by the gap suite |

Known expressibility limitations live **only** in that disposition table.
A claim that the tool cannot state or see something without naming a row
id fails `just check-gap-claims`, which runs as part of `just ci`; fixing a
limitation flips its row in the same change that lands the capability.

## Status & roadmap

P0–P11 are closed (each phase ran its red→green→refactor cycle as a beads
ticket), P11's domain-probe validation is recorded in `ledger/probes/`, and
the corpus-derived capability backlog (P12) landed the four G1–G3 changes:
the expressibility gap suite, instance morphisms (`naturality`), joint
composition, and coproduct exhaustiveness. The phase map, for orientation:

```
P0 kernel skeleton → P1 executed checks → P2 findings → P3 CLI (genesis)
→ P4 analysis → P5 wasm+web → P6 kata ledger (STOP CONDITION: survived,
scoped) → P7 migration Δ → P8 rule lens (pre-registered validation,
survived scoped) → P9 bogart pilot (first real-domain dogfood)
→ P10 schema diff → P11 domain probes (baseline-armed, recorded)
→ P12 gap suite + composition + instance morphisms + coproducts
```

`bd ready` lists the remaining work (seam/naturality recipes in USING,
positioning notes, and the two filed defects `catherine-3en` and
`catherine-ke1`). P6's ledger verdict gates P7/P8 — it came
back `survived (scoped)`, so both proceeded by the recorded procedure; had
it been `killed`, the project would have stopped and recorded the negative
result.

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
