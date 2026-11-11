# Implementor brief — catherine-1ay (P4: bounded analysis — search, counterexamples, redundancy, underconstraint)

- **Ticket**: `catherine-1ay` — P4: analysis — bounded search, counterexamples, redundancy, underconstraint (beads: task, lane `afk`, blocked-by `catherine-86i` ✓ done, blocks `catherine-9b5` (P5) and co-blocks `catherine-hc6` (P9); complexity `l` — the epic's largest ticket; labels `ct-kernel`)
- **Openspec change**: `add-bounded-analysis` (proposal-only contract — **no design.md by intent**; 0/13 tasks at brief time; spec delta: `specs/analysis/spec.md`, 4 requirements / 6 scenarios)
- **Base**: current `main` @ `1b29031`. P3 landed at `b427810` with exactly two non-code commits above it (`0a5ec98` P3 ro5u artifact, `1b29031` chore(beads) export). The ticket's `base_commit` metadata `34da242` predates the P0–P3 merges and is **superseded** — same pattern as the P3 brief; implement atop the current tip.
- **Scope**: `crates/catherine-analysis/**` (new crate; ticket metadata `files: ["crates/catherine-analysis"]`) + workspace `Cargo.toml` member line + `catherine-cli` touches **as task 11 demands** (the proposal's Impact is explicit: "CLI gains analysis commands" — task 11: "CLI commands for each analysis op") + change-dir evidence files. **`catherine-core` is untouched**: dependency direction `analysis → core`, and the construction/composition surfaces P4 needs are already public (`Instance`/`Path`/`Model` pub fields, `TypedInstance::compose` pub — see §6).

## 1. Ticket ID and desired outcome

Desired outcome: **the analysis layer** — the deterministic question-answerer agents ask mid-conversation (DESIGN §4.2, proposal §Why). The kernel executes declarations on a *given* instance; analysis answers: *can* these constraints be satisfied at all (contradiction), is this declaration derivable (redundancy), which parallel paths were never reconciled (underconstraint), and where do two instances differ (distinguish). Every op is deterministic, bounded, and **scope-honest**:

- **Bounded instance search** (task 6): explicit caller-supplied bounds in → outcome out. Outcome is one of: a **witness instance**, a **proven-unsatisfiable `contradiction` finding** (only when exhaustion *completes* within bounds; the finding restates the bound and carries `ContradictionWitness { scope: Vec<usize> }` — the canonical equation indices), or **`UNKNOWN { scope }`** labeled `unknown` with the exact scope explored. A **zero-size bound yields `UNKNOWN` with the empty scope — never a contradiction claim** (spec scenario, ticket `anti_goal`). "No violation found" is never a naked answer.
- **Counterexample minimality**: folded into the search engine's enumeration discipline, **not a separate op** — enumeration is deterministic and smallest-first within bounds, so the returned witness is the smallest one (proposal bullet 2; tasks.md has no separate minimality task — do not invent one).
- **Redundancy** (task 7): path-equivalence via **normal-form rewriting under the declared equations** → `redundant` findings carrying `RedundantWitness { derivable: Path, reducing: Path }` — a finding *only* when rewriting proves it; absent a proof, no redundancy claim (spec: "Derivable path"). Also flags declared equations derivable from the others (proposal bullet 3). Rewriting is scoped to the declared equation set; confluence limits are **documented**, not hand-waved.
- **Underconstraint** (task 8): parallel-path detection over the schema graph — identical endpoints, **no declared equation relating them** → `underconstraint` findings labeled **`hypothesized`**, phrased as the domain question "should these agree?" (the witness carries `DomainQuestion::ParallelPaths { lhs, rhs }` + the question text). Never a violation, never a failure.
- **`distinguish`** (task 10): bounded behavioral distinguishability of two instances over an **explicit observation set** → `distinguishable` naming the differing observation and both values, or `equivalent-in-scope`. **The observation set is part of the result.** Schema equivalence is undecidable and **out of scope by name** — no schema-equivalence claim is ever emitted. This is an analysis *result type*, not a `Finding` — the P2 taxonomy has no distinguish kind and stays closed.
- **`query.rs` port** (task 9): specodelic's traversal primitive (264 LOC: `Direction`/`Query`/`check`/`run`, forward/backward closure, `blast_radius`, `cyclic_nodes`, `fan_in`/`fan_out`) ported **behind the parity gate + property tests** (E8). Adapt, don't author (§3 reuse map).
- **CLI commands for each analysis op** (task 11): one subcommand per op under the existing `Commands` enum (names: implementor's choice within the existing clap surface), genesis envelope output, **labels intact**, findings-bearing ops exiting through the **existing** min-over-class-codes decision (5 = contradiction, 6 = underconstraint, 7 = redundant — numerals stable once shipped). A search/distinguish run that completes within its bounds is a **successful tool execution → exit 0** with the outcome in the payload; an `UNKNOWN` outcome must be **machine-visible in the payload** (`data` carries `outcome: "unknown"` + the scope object) — exit 0 here is *not* "clean", because the payload names the epistemic state; a test asserts the UNKNOWN payload shape and that the scope names the explored bounds. Exit numerals never drift.
- **Emits into the P2 taxonomy, no new finding kinds.** P2's `findings.rs` defines the three analysis kinds and their witnesses, fixture-pinned, with the docs sentence "their producers are P4's (bounded analysis) — nothing in this crate emits them". P4 is the producers. The five kinds, the total order (kind in declaration order, then witness ids, label last), and the stable serde encoding are **not touched**.

**Epistemic labels (adjudicated here — the change text is silent on contradiction/redundant):** `unknown` for scope-bounded non-answers (`UNKNOWN { scope }`), `hypothesized` for underconstraint questions, `executed` for violations — and, per P2's own label docs ("`executed` (kernel-computed results)"), **`executed` also for the proven results**: contradiction-by-completed-exhaustion and redundant-by-derivation are deterministic proofs the kernel computed, so they carry `executed`. The change's honest-limits line ("`UNKNOWN` is a first-class result (label `unknown` + scope)") pins the unknown case; this brief pins the other two where the change is silent.

## 2. Why

Delegation rationale — why a subagent implements instead of the lead session:

- The epic-orchestrator canon forbids inline execution by the lead (Invariant 1); the lead claims, briefs, spawns, verifies, ships.
- The ticket is fully specified without conversation context: an openspec change with a spec delta, 13 enumerated tasks, the P2 taxonomy to emit into (reviewed and closed, shapes + labels fixed), the P1 `TypedInstance::compose` helper to build on, the P3 CLI's one-execution-path/envelope/exit machinery to extend, and a named donor (`specodelic src/acset/query.rs`) for the port. A blank spawned session can execute it from repo artifacts alone.
- The RED→GREEN→REFACTOR cycle is the epic's longest (a new crate, a bounded search engine, a rewriting kernel, a graph traversal port with parity properties, CLI wiring); running it inline would stall the lead loop for the whole cycle.
- Isolation gives clean verification (Invariant 5): the lead cross-checks the report against `git` reality and fresh runs. Single-model run → the §9 degradation applies; the recorded verdict is **REVIEW_REQUIRED**.
- This ticket unblocks P5 (wasm) and co-unblocks P9 (bogart pilot — "gated on P3 CLI + P4 findings"): P9's evidence ("which findings fired on the live corpus") needs real producers for the three analysis kinds.

## 3. Completion criteria

Every criterion is a runnable command; **exit 0 = done**, with the one named deliberate exception (criterion 7). All run from the repo root on the base in the header (workspace, justfile, P0–P3 targets in place). Analysis fixtures live as Rust fixture builders under `crates/catherine-analysis/tests/` (the RED tasks' five fixtures); CLI analysis-command tests extend `crates/catherine-cli/tests/cli.rs` with their own fixtures.

1. `cargo test -p catherine-analysis` — the new crate's suite green, asserting **the concrete analysis outcomes tasks.md names**:
   - over-constrained fixture (task 1: mutually exclusive equations) → search **exhausts within bounds** → `contradiction` finding, witness `scope` = the equation indices, bound restated, label `executed`;
   - small-budget fixture (task 2: budget smaller than the search space) → **`UNKNOWN` with the explored scope**, label `unknown` — never "no contradiction found"; **zero-size bounds → `UNKNOWN` with empty scope**, never a contradiction claim (assert both);
   - parallel-paths fixture (task 3: same endpoints, no equation) → `underconstraint` finding, label `hypothesized`, witness `{"case": "parallel_paths", "detail": {"lhs": …, "rhs": …}}` + question text;
   - derivable-path fixture (task 4: path `p` rewrites to declared path `q`) → `redundant` finding naming `p` with `by: q` (`derivable`/`reducing`), label `executed`;
   - distinguish fixtures (task 5: two instances differing on one observation) → `distinguishable` naming the observation + both values; the same pair restricted to an observation set where they agree → `equivalent-in-scope`, observation set in the result;
   - task 9's **parity + property tests green**: the donor's contract names carried as tests (`seeds_exist`, `closure_terminates`, `dangling_not_followed`, `results_ordered`, `fan_in_is_preimage_size`) against catherine's `TypedInstance` tables;
   - emitted findings serialize **byte-identically across repeated runs** (P2's total order holds through the new producers — structural assertions, no snapshots).
   Assertions are structural (`PartialEq` on the crate's outcome/finding values, or `serde_json::Value` field assertions at the CLI layer) — never byte-compared snapshots (DESIGN: insta-style deliberately avoided).
2. `cargo test -p catherine-cli` — analysis subcommands green: envelope shape (`ok`/`envelope_kind`/`data`/`warnings`/`hints`), finding labels printed/serialized intact, UNKNOWN payload explicit (`outcome` + scope), findings-bearing ops exit 5/6/7 through the existing decision, search/distinguish complete → exit 0 with the outcome payload. `EXIT_CODE_TABLE` unchanged — assert that too.
3. `cargo test --workspace` — all kernel suites unregressed (P0 parse goldens + round-trip, P1 check fixtures/properties, P2 findings fixtures, P3 CLI suite) plus the new crate; **no kernel fixture weakened or deleted**.
4. `TMPDIR=$PWD/target/tmp just ci` — the workspace gate (fmt + clippy `-D warnings` + tests). **The TMPDIR quirk is named**: the sandbox `/tmp` breaks the suite's temp-file writes — always run the gate with `TMPDIR=$PWD/target/tmp` (workspace-local temp dir); a bare `just ci` failing on `/tmp` is the known environment artifact, not a regression — rerun with the quirk before diagnosing.
5. `openspec validate add-bounded-analysis --strict` — exit 0 (task 13's exact form).
6. **Parity gate documented** (task 13): the donor-contract → catherine-test mapping recorded in `crates/catherine-analysis/README.md` (which donor contracts the port carries, what was adapted and why — specodelic's string-keyed instance vs catherine's dense `RowId` tables).
7. `ah check --changes add-bounded-analysis` — **reports ONLY no-toml findings; 5 expected** at brief time. This criterion is deliberately NOT exit-0: the change's scenarios have no contract tomls yet (contracts are born at apply time), so `ah check` legitimately exits 1 with only those structural findings — **criterion = "only no-toml findings", never exit-0**. The implementor reports it exactly as "only no-toml findings (5), exit 1 expected". Substantive findings (missing requirements, orphan scenarios, spec/tasks mismatches) = stop and report (§10), never papered over behind the exit code.
8. `openspec/changes/add-bounded-analysis/tasks.md` shows **13/13 ticks** (verify: `openspec list` reports 13/13). Task 13's `bd close catherine-1ay` + fresh export is **orchestrator-reserved** — the implementor ticks the box once its own task-13 parts (ci green, parity gate documented, `openspec validate`) are green and records "bd close pending orchestrator" in the report.
9. **ro5u review artifact recorded in the change dir**: `openspec/changes/add-bounded-analysis/ro5u-review.md` (matching the P2/P3 evidence-file discipline) — the rule-of-5-universal review of the full diff with **max_critical = 0**, cited in the implementor report (§8).

A criterion that cannot be executed is not a completion criterion — if any command above is impossible as specified, stop and report (§10), do not substitute.

## 4. Spawn model

- **Implementor model**: `openrouter/z-ai/glm-5.3-flash`.
- **Context ceiling**: **≤ 60% of the implementor's configured context window, computed at spawn time** from the actual window (assume 1M for this model). No fixed token counts — if the configured window differs at spawn, recompute the absolute ceiling from that window. **Complexity `l` discipline**: this is the epic's largest ticket; the implementor must keep the ceiling by leaning on repo artifacts (this brief, the change dir, P2's `findings.rs` docs, the donor's own doc-comments) instead of re-deriving design in-session, and the **§10 stop-and-file applies early** — at the *first* sign of scope growth (a solver temptation, a core-API gap, a task count creeping past 13), stop and file rather than absorb.
- **Session**: named, resumable spawn `subagent:catherine-1ay:implement`; output streamed to a known log path recorded in `.wai/projects/catherine/runs/catherine-1ay.state` (Invariants 2, 7).
- **Wall-clock budget**: wrap the spawn in `timeout 7200` (120 min — the `l` ticket gets double the `m` budget). Never a bare foreground call that blocks to the harness timeout; background + tail + poll (Invariant 8).
- **Retry**: fresh named session `subagent:catherine-1ay:implement-retry`; zero commits = clean retry, partial commits = manual resume. An interrupted-but-healthy spawn (commits or session record exist) resumes the existing session, never restarts from scratch.
- **Verification side**: no cross-family verifier is configured for this run → single-model degradation path applies (§9): lead-side verify, and the recorded verdict is **REVIEW_REQUIRED**.

## 5. Out of scope

From ticket `anti_goals` ("no unbounded claims; UNKNOWN scope never collapsed into clean"), the proposal's deliberate exclusions, and DESIGN's phase gates — touching any of these is a scope violation; stop and file follow-ups (§10) instead:

- **No unbounded claims** (`anti_goal`): every analysis answer is either a proof within stated bounds or `UNKNOWN { scope }`. No code path, doc line, or help text may imply exhaustion that didn't happen; "no violation found" is never an output.
- **UNKNOWN never collapsed** (`anti_goal`): never silently converted into clean, into a finding, or into an error. Zero-size bounds → `UNKNOWN` with empty scope, never contradiction (spec scenario).
- **No P5/P6/P7/P8/P9 leaks**: no wasm/UI (P5), no ledger or corpus — and no **ledger entry** here (the ledger entry follows the phase's validation run and is the orchestrator's wrap-up, P6) (P6), no migration axis (P7), no rule lens — specifically **`DomainQuestion::UncoveredInputs` is never produced** (defined in P2, reserved for P8: "defined here, never produced in v1"), no bogart pilot fixtures (P9).
- **No inquiry-layer ops** (DESIGN §4.3): `consequences`, `impact`, `alternatives`, `questions`, `compare` and the heuristic scaffolds **stay spec-only until P6 validates the deterministic ops**. The deterministic v1 set is exactly what tasks.md names: search, rewriting, parallel paths, distinguish + the query traversals.
- **No schema-equivalence claims** — out of scope by name in the spec; `distinguish` decides bounded behavioral equivalence over an explicit observation set and says nothing beyond it.
- **No solver dependency**: bounded search is **plain hand-rolled enumeration over the finite row spaces** (`Instance`/`Path`/`Model` are pub; `TypedInstance::compose` is the composition primitive). No SAT/SMT/constraint crates, no external search libraries. Anything beyond the §6 pinned set is a §10 trigger.
- **No P3 surface drift**: exit numerals never drift from the shipped table; no colors, no alignment tables, no verbosity beyond the genesis-standard structs already wired; no second envelope shape; the kernel crates stay **genesis-free** (envelope lives in `catherine-cli` only).
- **No `FindingKind` additions**: P2's five-kind enum is closed; analysis emits existing kinds with existing witnesses, nothing new.
- **Dependencies stay pinned** (§6): runtime `catherine-core` (path) only; dev `proptest` (DESIGN's named testing tooling — task 9 mandates property tests) and `serde_json` where CLI tests assert envelopes. Beyond this set = §10 trigger, stated explicitly in the report.

## 6. Exact files

Create/touch **only** (ticket metadata `files: ["crates/catherine-analysis"]`, the workspace member line, the CLI touches task 11 mandates, and change-dir evidence files):

- `crates/catherine-analysis/**`:
  - `Cargo.toml` — package `catherine-analysis`, lib; deps: `catherine-core` (path); dev-deps: `proptest` (property tests, task 9), `serde_json` only if outcome-serialization assertions need it.
  - `src/lib.rs` — crate root with the **Purpose/Responsibilities/Rationale header** (repo discipline, as `findings.rs`/`instance.rs` carry) naming the dependency direction (`analysis → core`) and the honest-limits contract (bounds are inputs; `UNKNOWN` first-class).
  - Module layout (implementor's final call, this is the expected shape): `search.rs` (task 6), `rewrite.rs` (task 7), `parallel.rs` (task 8), `distinguish.rs` (task 10), `query.rs` (task 9's port — keep the donor's module name), and the shared types module task 12 extracts (`bounds.rs`: bound/scope types across ops + deduped witness builders).
  - `README.md` — **decidability boundary** (what the tool refuses to claim: no unbounded claims, zero-size bounds → UNKNOWN, no schema equivalence, confluence scoped to declared equations + their documented limits) **and the parity-gate documentation** (criterion 6).
  - `tests/` — the five RED fixtures as Rust fixture builders + assertions, the parity/property suite, determinism (byte-identical serialization) assertions.
  - The ported `query.rs` carries the **Apache-2.0 attribution** in its header, mirroring `instance.rs`'s precedent: "adapted (not authored) from specodelic (`charly/specodelic`, Apache-2.0, ships no NOTICE file)" — plus what was adapted and why (string-keyed `BTreeSet<String>` world → catherine's `Ident`/`RowId` dense tables).
- `Cargo.toml` (workspace root): `members` gains `"crates/catherine-analysis"`. Nothing else changes.
- `crates/catherine-cli/**` — **only what task 11 demands**: `src/cli.rs` (`Commands` gains one subcommand per analysis op; `EXIT_CODE_TABLE` verbatim-unchanged), `src/run.rs` (analysis runners reusing the single-execution-path/envelope machinery; findings-bearing ops feed the existing min-over-class-codes exit), `src/render.rs` (envelope construction reused, labels intact, UNKNOWN outcome as explicit payload data).
- `openspec/changes/add-bounded-analysis/**`: tick `tasks.md` checkboxes; record `ro5u-review.md` (criterion 9); touch `specs/analysis/spec.md` only if implementation exposes a genuine contract gap (say so in the report if so).

**Explicitly not this ticket**: task 13's `bd close catherine-1ay` (+ fresh `bd` export) is **orchestrator-reserved** — the implementor must NOT run `bd close` or any `bd` mutation; the orchestrator closes after verification. **No changes under `crates/catherine-core/**` are expected**: the analysis crate constructs candidates against pub surfaces (`Instance { rows, values }`, `Path(pub Vec<Ident>)`, pub `Model`, `TypedInstance::compose(&Path, RowId) -> Option<RowId>` — P1's REFACTOR exposed it precisely for this). If a task *forces* a core diff, it is minimal, justified in the report, and still a §10 signal — never patched quietly. No new repo-root plumbing beyond the workspace member line.

## 7. TDD sequence — RED → GREEN → REFACTOR

Run one cycle, exactly as `tasks.md` orders it (RED = tasks 1–5, GREEN = tasks 6–11, REFACTOR = task 12, Validation = task 13). State the transition evidence (command + observed failure mode) in the report at each transition.

### RED — the five fixtures against an analysis crate that does not exist

The ticket names the red: **"UNKNOWN not representable; search has no scope input"**. Author the five fixture tests first (tasks 1–5): each fixture builds its model/schema, calls the analysis op with explicit bounds, and asserts the outcome structurally — contradiction within bounds (task 1), `UNKNOWN { scope }` (task 2), underconstraint hypothesis (task 3), redundant with `by` witness (task 4), distinguishable + witness (task 5). Hosting them requires the minimal crate skeleton (`Cargo.toml` + empty `lib.rs`) — scaffolding for authoring the red, not GREEN work: no search, no rewriting, no port. The **observed red** is recorded once in the report: the outcome type with its `unknown { scope }` variant does not exist (UNKNOWN not representable), and no op accepts bounds (search has no scope input) — the fixtures cannot compile/pass.

- **Narrow**: the five fixture tests — red as described.
- **Full**: `cargo test -p catherine-analysis` — red here is expected; do not fix forward.
- Transition when all five fixtures pin their expectations and the red is recorded.

### GREEN — search, rewriting, parallel paths, port, distinguish, CLI

Minimal implementations, in tasks.md order:

- **Bounded search engine** (task 6): explicit bounds in → witness / `UNKNOWN { scope }` out; completed exhaustion → `contradiction` finding; zero-size bounds → `UNKNOWN` with empty scope, never contradiction; deterministic smallest-first enumeration (counterexample minimality lives here).
- **Normal-form rewriting** (task 7): path equivalence under the declared equations → `redundant` findings with both witness paths; confluence scoped to the declared equation set, limits documented (the README section starts here).
- **Parallel-path detection** (task 8): same endpoints, no declared equation → `underconstraint`, label `hypothesized`, question text phrased as the domain question.
- **`query.rs` port behind the parity gate** (task 9): adapt the donor's traversals onto `TypedInstance`'s tables; carry the donor contract names as tests (`seeds_exist`, `closure_terminates`, `dangling_not_followed`, `results_ordered`, `fan_in_is_preimage_size`); property tests green. If the port starts needing a *core* redesign to fit (string-id world vs dense tables), that is the §10 donor-mismatch trigger — stop, don't reshape the kernel.
- **`distinguish`** (task 10): observation set in, `distinguishable` + witness or `equivalent-in-scope` out; observation set part of the result.
- **CLI commands** (task 11): one subcommand per analysis op; genesis envelope; labels intact; findings-bearing ops exit through the existing decision; UNKNOWN payload explicit; exit 0 for completed bounded runs.
- **Narrow**: the five RED fixtures green one by one.
- **Full**: `cargo test -p catherine-analysis` and `cargo test --workspace` green (P0–P3 suites unregressed).
- Transition when narrow targets are green and no kernel fixture was touched.

### REFACTOR — structure + the honesty documentation, no behavior expansion

- **Share bound/scope types; dedupe witness builders** (task 12): one bounds/scope module across the analysis ops; witness construction deduped; all outcomes byte-identical before and after.
- **Scope caps as config** (ticket REFACTOR): the search/distinguish caps become configured values, not magic numbers scattered through the engine; defaults documented in the README's decidability-boundary section.
- **Document the decidability boundary** (ticket REFACTOR): what the tool refuses to claim — no unbounded claims, `UNKNOWN` semantics, zero-size-bounds rule, no schema equivalence, rewriting limits — written down in `crates/catherine-analysis/README.md`.
- **Narrow + full**: re-run the fixture suite and `cargo test --workspace` — green, behavior-identical.

## 8. ro5u review completion gate

Before declaring done, run a rule-of-5-universal review of the **full diff** (invoke explicitly: `/skill:rule-of-5-universal`). Gate:

- **max_critical = 0** — no exceptions.
- Critical and high findings: fixed.
- Medium findings: fixed, or deferred **with a written reason** in the report.
- Every behavior-impacting fix adds or updates a test (structural outcome assertions, parity/property tests, or CLI envelope/exit assertions — never snapshots; the repo has no snapshot discipline and P2's anti-snapshot rule carries forward).
- The review artifact is recorded at `openspec/changes/add-bounded-analysis/ro5u-review.md` (findings + dispositions) and cited in the implementor report. ro5u is a completion criterion: without it the ticket is not done regardless of test status.

## 9. Verifier evidence contract (wai-px4g)

Applies to the verify step that follows this ticket:

- The review artifact may record **CONFIRM only when it cites at least one read artifact path** (e.g. `.wai/projects/catherine/research/…`) **and includes fresh test-run output** — the actual command plus its result, pasted. Lacking either: **REVIEW_REQUIRED**, never CONFIRM on claims alone.
- **Single-model rule**: no cross-family verifier is configured for this run, so the lead-side degradation path applies — skip the spawned verifier; the lead runs the lead-side checks instead (state checkpoint, `git log`/`git diff` vs the report, fresh test-suite run with the TMPDIR quirk, beads state). This degradation is non-blocking, but the **recorded verdict is REVIEW_REQUIRED with a lead-side-only note — never CONFIRM**.
- A contradiction between report and reality fails the run: fix or file it, then re-verify fresh. For this ticket specifically, the highest-value cross-checks are the **scope-honesty outcomes**: re-run the search on the task-2 fixture (UNKNOWN + scope must appear in the payload, never "no contradiction found") and the task-1 fixture (contradiction only after real exhaustion) — the anti_goals live or die here.

## 10. Follow-up threshold

`tasks.md` enumerates exactly **13 tasks** — that is the whole change. Complexity `l`: **prefer filing over absorbing.** Stop and file follow-up beads tickets instead of expanding when:

- implementation demands more than the 13 tasks (extra analysis ops beyond search/rewriting/parallel/distinguish/queries, extra CLI surface beyond task 11's one-subcommand-per-op, extra fixture classes);
- **the donor mismatches**: the `query.rs` port starts needing a redesign of *core* types (its string-keyed instance world genuinely doesn't fit the dense `RowId` tables) — adapt the port, never reshape the kernel; a core redesign is a filed follow-up;
- **kernel changes start leaking in** — any diff under `crates/catherine-core/**` beyond what a task demonstrably forces: file it, never patch the kernel from the analysis ticket;
- **the search design explodes**: a temptation toward SAT/SMT solvers, heuristics, memoized infinite domains, or unbounded fixpoints — the bounds-in/UNKNOWN-out contract is the whole design; growth is a follow-up, not diligence;
- P5/P6/P7/P8/P9 surface leaks in — wasm/UI, ledger/corpus entries or the ledger write itself, migration axis, rule lens (`UncoveredInputs` production!), bogart fixtures;
- genesis modules start looking necessary in the analysis crate (the kernel crates stay genesis-free), or dependencies beyond the §6 pinned set (especially any solver) come up;
- exit numerals drift, or a second envelope shape appears;
- the context ceiling in §4 is exceeded, or a completion criterion in §3 proves impossible as specified.

File the follow-ups with `bd`, note them in the report, and finish only the 13 tasks. Growing the change silently is a failure mode, not diligence.

## Hard constraints (binding)

1. **Findings never without witnesses** (P2 shapes): `RedundantWitness { derivable, reducing }`, `ContradictionWitness { scope: Vec<usize> }` (canonical equation indices, in order), `UnderconstraintWitness { case: DomainQuestion, question }` (flattened `{"case": …, "detail": {…}, "question": …}`). Analysis emits P2's kinds with P2's witnesses; **no new finding kinds**.
2. **`UNKNOWN { scope }` is a first-class result** — never silently converted to clean, to a finding, or to an error; the scope names the exact bounds explored; a zero-size bound yields `UNKNOWN` with empty scope and is **never reported as a proven contradiction**; "no violation found" is never an output.
3. **Epistemic labels per P2**: `unknown` for scope-bounded non-answers, `hypothesized` for underconstraint questions, `executed` for violations — and for the proven analysis results (contradiction-by-completed-exhaustion, redundant-by-derivation) per §1's adjudication of P2's label docs.
4. **Determinism**: repeated runs serialize byte-identically (P2's total order: kind in declaration order, then witness ids, label last; donor traversals ordered by construction — `BTreeSet`/`BTreeMap` discipline).
5. **Bounds are inputs; no unbounded claims** (`anti_goal`); confluence of rewriting is scoped to the declared equations with documented limits; **no schema-equivalence claims, ever**.
6. **No new deps beyond the pinned set** (`catherine-core` path; dev `proptest`, `serde_json` where needed) — **no solver crates**; any deviation is stated explicitly in the report and is a §10 trigger.
7. **P0–P3 suites unregressed**; `catherine-core` diff-free (any forced diff: minimal, justified, reported); CLI `EXIT_CODE_TABLE` numerals stable; kernel crates genesis-free.
8. **Port discipline**: `query.rs` is adapted, not authored — Apache-2.0 attribution in the header per the `instance.rs` precedent; the parity gate carries the donor's contract names as tests; new files carry the Purpose/Responsibilities/Rationale header.
9. **`DomainQuestion::UncoveredInputs` is never produced in v1** (reserved for P8); the parallel-paths arm is the only underconstraint producer this phase.
10. **`bd close catherine-1ay` + fresh export are orchestrator-reserved** — the implementor never runs `bd` mutations.
