# Implementor brief — catherine-86i (P3: the `catherine` CLI — `check` and `findings`)

- **Ticket**: `catherine-86i` — P3: CLI — catherine check/findings (beads: task, lane `afk`, blocked-by `catherine-1x7` ✓ done, blocks `catherine-1ay` (P4) and co-blocks `catherine-hc6` (P9); complexity `m`; labels `ct-kernel`)
- **Openspec change**: `add-check-cli` (proposal-only contract — **no design.md by intent**; 0/9 tasks at brief time; spec delta: `specs/cli/spec.md`)
- **Base**: current `main` @ `72af9ff`. The P2 code landed at `dc4c95e`; exactly two non-code commits sit above it (`59cec22` chore(beads) export, `72af9ff` P10 filing + memories). The ticket's `base_commit` metadata `34da242` predates the P0–P2 merges and is superseded. `catherine-core` is reviewed and closed (1x7); implement atop the current tip.
- **Scope**: `crates/catherine-cli/**` (new crate) + workspace `Cargo.toml` member line + README/CHANGELOG entries (see §6 for the decision) + change-dir evidence files. **`catherine-core` is untouched** (proposal Impact: dependency direction `cli → core` via the findings/checks APIs).

## 1. Ticket ID and desired outcome

Desired outcome: **the first user-facing surface** — a binary named `catherine` (the DESIGN §8.3 naming question is closed by the proposal: binary is `catherine`, aliases later) that drives the kernel from a terminal, CI, or an agent tool-call:

- **`catherine check <model.ct.json>`**: parse → build → execute all checks; print **every finding with its kind, epistemic label, and witness** — human-readable by default, the genesis-vibes JSON envelope under `--json` with findings under `data.findings`.
- **`catherine findings <model.ct.json>`**: the findings-only view — *same execution, different presentation* (no re-run-semantics divergence; that sentence in the proposal is binding: the two commands share one execution path).
- **Exit codes mirror finding classes** with the spec's fixed precedence so CI and agents branch on exit status **without parsing any text** (ticket `anti_goal`: "no exit contracts requiring text parsing").
- **Errors are labeled, never stack traces**: parse errors and typing violations print with line/column, stable code, and a remediation hint (P0's `error.rs` codes + span already carry this — the CLI renders it).
- **Stream discipline**: errors to **stderr**, findings to **stdout** (ticket REFACTOR line).
- **The DESIGN §5 agent-loop walkthrough passes** — see the verbatim adjudication below (§3, criterion 10, and §7 GREEN task 7): check a bowling-style model → violation with witness printed → agent proposes the equation → re-check → clean.

**The §5 walkthrough, verbatim — what "verbatim" binds (adjudicated here):** DESIGN §5's transcript is:

```
$ catherine check bowling.ct.json
violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item
  at row Assignment#3: lhs=Unit#1 rhs=Unit#2
underconstraint: parallel paths Assignment→Unit via staff/equipment
  have no declared equation? (hypothesis, domain question)
$ # agent proposes equation, tool re-checks, ledger updated — repeat
```

The violation block must be reproduced **byte-for-byte** by the human renderer: kind label, `eq-<canonical index>`, the composed paths joined with `∘`/`≠`, then the two-space-indented witness line `at row <RowId>: lhs=<RowId> rhs=<RowId>`. The acceptance fixture (`bowling.ct.json`) is engineered so the kernel genuinely emits exactly that witness — equation index 0 over paths `staff_unit∘assigned_staff` vs `equipment_unit∘assigned_item`, failing at row `Assignment#3` with values `Unit#1`/`Unit#2` — so the transcript's first block is real output, not aspiration. **The `underconstraint` line is P4-gated**: P2's `findings.rs` is explicit that the analysis kinds "are defined and fixture-pinned here; their producers are P4's — nothing in this crate emits them", and the proposal's own restatement of the acceptance scenario drops that line deliberately ("check bowling-style model → violation with witness printed → agent proposes equation → re-check → clean"). Therefore: **verbatim = the violation block + the loop shape** (check → violation exit → fixed document → re-check exits 0 with zero findings). The underconstraint hypothesis line returns when P4 lands; do **not** hand-fabricate a finding to make the transcript byte-match — fabricating findings is exactly what the P2 discipline forbids, and a faked line would be a failed walkthrough, not a passed one. The report pastes the actual runs (violating + fixed) and notes the underconstraint line as P4-pending.

**The two surfaces and their one contract:** the CLI prints P2's stable encoding **verbatim** — `data.findings` is the `Vec<Finding>` exactly as `catherine-core` serializes it (`{"label": …, "kind": …, "witness": {…}}` per finding; `FindingKind` adjacently tagged `kind`/`witness`; underconstraint witness flattened as `{"case": …, "detail": {…}, "question": …}`). The CLI adds a renderer and an exit decision; it adds **no second serialization**, no renaming, no reshaping. Two orderings exist and must not be confused: the **payload order** is P2's total order (kind in declaration order, then witness ids, label last); the **exit precedence** is the spec's severity order — violation > dangling > contradiction > underconstraint > redundant — and that order, per the reviewed HIGH finding on exit-code precedence, is **already adjudicated in the change text** (spec: "the order is part of the contract"). Implement the change text exactly as written above; do not re-litigate or reorder from memory.

**Envelope contract (the charly-vibes shape):** under `--json`, every command emits the shared suite envelope — `ok`, `envelope_kind`, `data`, `warnings`, `hints` — via genesis-vibes stable modules, consumed **by `catherine-cli` only**. Errors under `--json` are the envelope's error form (task 2: "labeled error envelope") carrying line/column, stable code, remediation hint; consult `genesis::envelope`'s own docs for the error envelope's exact fields — the kernel's labeled error data goes under it unmodified.

**Exit-code table (binding — pinned here because the change fixes the semantics, distinctness, and precedence but not the numerals):**

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

Derivation, stated so the implementor doesn't re-derive: `3..=7` are `2 + rank` under the spec's precedence order, so multi-class runs exit with the **minimum** present class code — the precedence rule collapses to a `min` over class codes, which is the whole implementation of the adjudicated HIGH finding. `0` is the spec's clean; `1` is the spec's "parse/build failure distinct from any finding"; `2` stays clap's built-in so usage errors never masquerade as findings. This table goes in `--help` and README verbatim (task 6) — the table **is** the contract; the numerals are stable once shipped.

## 2. Why

Delegation rationale — why a subagent implements instead of the lead session:

- The epic-orchestrator canon forbids inline execution by the lead (Invariant 1: inline is the historical failure mode); the lead session orchestrates only (claim → brief → spawn → verify → ship).
- The ticket is fully specified without conversation context: an openspec change with a spec delta (`specs/cli/spec.md`), 9 enumerated tasks, the P2 surface (`crates/catherine-core/src/findings.rs` — reviewed and closed) to print verbatim, and a donor ecosystem for the envelope (genesis-vibes; dependency route resolved in §6). A blank spawned session can execute it from repo artifacts alone.
- The RED→GREEN→REFACTOR cycle is long-running (a new crate, integration tests that spawn the binary, fixture engineering to reproduce the §5 witness, genesis wiring); running it inline would stall the lead loop for the whole cycle.
- Isolation gives clean verification: the lead cross-checks the implementor's report against `git` reality and fresh test runs (Invariant 5), with no risk of the author explaining away its own failures. Single-model run → the §9 degradation applies and the recorded verdict is REVIEW_REQUIRED.
- This is the epic's first consumer-facing contract: the walkthrough (DESIGN §5) becomes real, and P9's bogart CI gate ("`catherine check` exits per finding class") hangs off this ticket.

## 3. Completion criteria

Every criterion is a runnable command; **exit 0 = done** — with the one named, deliberate exception (criterion 7). All run from the repo root on the base in the header (workspace, justfile, P0–P2 targets in place). Fixtures for these commands live under `crates/catherine-cli/tests/fixtures/` (§6).

1. `cargo test -p catherine-cli` — the new crate's suite green: exit-code table assertions, envelope-shape assertions, the §5 walkthrough integration test, the labeled-error assertions (line/column present, no stack trace), and the `findings` view. Assertions are **structural**: parse stdout JSON with `serde_json::Value` and assert field names/values (`data.findings[0].kind == "violation"`, `witness.equation == 0`, `witness.row`, both path values, envelope keys `ok`/`envelope_kind`/`data`/`warnings`/`hints`); assert exit codes numerically.
2. `cargo test --workspace` — all kernel suites unregressed (P0 parse goldens, round-trip + parity gate, P2 findings fixtures/properties, check fixtures) plus the new crate; **no kernel fixture weakened or deleted**.
3. `just ci` — workspace gate (fmt + clippy `-D warnings` + tests).
4. **Exit-code classes exercised end-to-end with concrete assertions**, per class — `cargo run -p catherine-cli -- check <fixture>` and assert (exit code, stream, payload):
   - clean fixture → exit **0**, stdout, `data.findings` empty;
   - malformed fixture (duplicate ident with line/column, the spec's own example) → exit **1**, stderr, labeled error with line/column + stable code + remediation hint, no stack trace; under `--json` the error envelope on stderr;
   - violating fixture (`bowling.ct.json`) → exit **3**, stdout, exactly one violation entry under `data.findings` carrying equation id, row id, both path values (the spec's Agent-loop-acceptance scenario);
   - dangling fixture → exit **4**, stdout, dangling entry with arrow + source row; the run continues past dangling (P1 semantics);
   - **multi-class precedence fixture** (dangling + violation in one document) → exit **3** (violation wins), and the payload **still lists both classes** (the spec's Multiple-classes scenario);
   - the three analysis classes (contradiction/underconstraint/redundant) have **no P4 producer yet** — they are asserted at the exit-decision unit level: the `min`-over-class-codes function fed **directly constructed** findings (P2's taxonomy allows constructing them; its own fixtures did) yields 5/6/7 and the human renderer emits their labeled lines. End-to-end fixtures for those three classes are **P4's**, not this ticket's; do not add producers to fake them (§5, §10).
5. `cargo run -p catherine-cli -- findings bowling.ct.json --json` — same findings payload as `check` (same execution), asserted structurally equal to check's `data.findings`.
6. `openspec validate add-check-cli --strict` — exit 0 (task 9's form), and `openspec validate --strict --all` — exit 0 (workspace gate).
7. `ah check --changes add-check-cli` — **reports ONLY no-toml findings; 5 no-toml findings expected.** This criterion is deliberately NOT exit-0: the change's scenarios have no contract tomls yet (contracts are born at apply time), so `ah check` legitimately exits 1 with only those structural findings — **criterion = "only no-toml findings", never exit-0**. The implementor reports it exactly as "only no-toml findings (5), exit 1 expected" — never paper over substantive findings (missing requirements, orphan scenarios, spec/tasks mismatches) behind that exit code. Substantive findings = stop and report (§10).
8. `openspec/changes/add-check-cli/tasks.md` shows **9/9** ticks (verify: `openspec list` reports 9/9).
9. **ro5u review artifact recorded in the change dir**: `openspec/changes/add-check-cli/ro5u-review.md` (matching P2's evidence-file discipline) — the rule-of-5-universal review of the full diff with **max_critical = 0**, cited in the implementor report (§8).
10. **The DESIGN §5 agent-loop walkthrough executed verbatim, output pasted in the report**: run `cargo run -p catherine-cli -- check tests/fixtures/bowling.ct.json` (human mode) — the violation block matches the §5 transcript byte-for-byte (§1's adjudication: violation block + loop; underconstraint line P4-pending) and exits **3**; then run `check tests/fixtures/bowling-fixed.ct.json` (the same document plus the agent's proposed equation) — exits **0**, zero findings. Both runs' output pasted, with the two commands shown. The same two-phase walkthrough is scripted as the task-7 integration test, so criterion 1 executes it forever after.
11. **Genesis stays out of the kernel**: `cargo tree -p catherine-core -i genesis-vibes` fails to resolve (genesis absent from the kernel's dependency tree) while `cargo tree -p catherine-cli -i genesis-vibes` resolves; `genesis-vibes` appears in **no** kernel crate's `Cargo.toml`.
12. **README + CHANGELOG carry the contract** (decision recorded in §6): README documents the CLI usage + the exit-code table verbatim from §1; CHANGELOG gains the Unreleased → Added entry for the CLI.

A criterion that cannot be executed is not a completion criterion — if any command above is impossible as specified, stop and report (§10), do not substitute.

## 4. Spawn model

- **Implementor model**: `openrouter/z-ai/glm-5.3-flash`.
- **Context ceiling**: **≤ 60% of the implementor's configured context window, computed at spawn time** from the actual window (assume 1M for this model). No fixed token counts — if the configured window differs at spawn, recompute the absolute ceiling from that window. If the ticket cannot be executed within the ceiling, stop and split per §10. (Ticket complexity is `m`; the ceiling is still computed, not assumed.)
- **Session**: named, resumable spawn `subagent:catherine-86i:implement`; output streamed to a known log path recorded in `.wai/projects/catherine/runs/catherine-86i.state` (Invariants 2, 7).
- **Wall-clock budget**: wrap the spawn in `timeout 5400` (90 min). Never a bare foreground call that blocks to the harness timeout; background + tail + poll (Invariant 8).
- **Retry**: fresh named session `subagent:catherine-86i:implement-retry`; zero commits = clean retry, partial commits = manual resume. An interrupted-but-healthy spawn (commits or session record exist) resumes the existing session, never restarts from scratch.
- **Verification side**: no cross-family verifier is configured for this run → single-model degradation path applies (§9): lead-side verify, and the recorded verdict is **REVIEW_REQUIRED**.

## 5. Out of scope

From ticket `anti_goals`, the proposal's deliberate exclusions, and DESIGN's phase gates — touching any of these is a scope violation; stop and file follow-ups (§10) instead:

- **No genesis-vibes in the kernel crates** (`anti_goal`, ticket notes): `catherine-core` (and the future wasm crate) gain **no** genesis dependency — the dependency check is criterion 11. genesis-vibes is consumed by `catherine-cli` **only**, and only its stable modules: **envelope** (the `ok`/`envelope_kind`/`data`/`warnings`/`hints` envelope), **guide** (the verbosity/format clap structs), **suggestions** (did-you-mean for unknown args), **config** (config file parsing). No other genesis module, no `update-check` feature (that pulls semver/ureq network machinery the ticket never asked for), no kernel convenience re-export of genesis.
- **No exit contracts requiring text parsing** (`anti_goal`): exit codes are computed from the finding classes present, never derived by parsing rendered output — not the human text, not the JSON text. The JSON is printed, not read back for control flow. The single place human text is *asserted* is the §5 walkthrough test (task 7), where the transcript fidelity is the mandated contract; even there the exit code is asserted numerically.
- **No P4 analysis.** No producers for redundant/contradiction/underconstraint — no bounded search, no normal-form rewriting, no parallel-path detection, no `query.rs` pull-in. Constructing analysis-kind findings *in unit tests* to pin the exit mapping and rendering is allowed (the taxonomy is constructible by design); producing them at runtime is P4's, file it if it starts looking necessary.
- **No wasm** (P5), no UI, no migration axis (P7), no corpus/ledger (P6) — and no **ledger entry** here: per the phased plan the ledger entry follows the phase's validation run and is the orchestrator's wrap-up, not the implementor's.
- **No human-formatting flourishes beyond the change's spec**: the human renderer implements the §5 transcript's format (kinds labeled, witnesses on the indented continuation line, `∘`/`≠`/`→` as in the transcript) and the labeled-error format — no colors, no alignment tables, no summary boxes, no extra verbosity levels beyond what genesis's guide structs already standardize. If a rendering idea isn't in the spec, the §5 transcript, or genesis's standard structs, it is out of scope.
- **Dependencies stay pinned** (§6): runtime `catherine-core` (path), `clap` (4, derive), `genesis-vibes = "0.13"`; dev `serde_json` (+ `tempfile` only if fixtures must be materialized on disk — prefer `include_str!` over writing temp files). Anything beyond this set is a §10 trigger.
- **No `ErrorCode` additions**: P0's six-code enum is closed; the CLI renders existing codes, it does not extend them.
- **No aliases/short bin** (proposal: binary is `catherine`; aliases, if any, later), no shell-completion generation, no config-file surface beyond what genesis's config module needs to be wired, no update checks.

## 6. Exact files

Create/touch **only** (ticket metadata `files: ["crates/catherine-cli"]`, plus the workspace line the proposal's Impact names, plus change-dir evidence files where tasks require recording):

- `crates/catherine-cli/**`:
  - `Cargo.toml` — package `catherine-cli`, bin `catherine`; deps: `catherine-core` (path), `clap = { version = "4", features = ["derive"] }`, `genesis-vibes = "0.13"`; dev-deps: `serde_json` (+ `tempfile` only if truly needed).
  - `src/main.rs` (entry, stream discipline: errors → stderr, findings → stdout), `src/cli.rs` (clap wiring: `check`/`findings` subcommands, positional model path, `--json`; genesis guide's verbosity/format structs), `src/run.rs` (one execution path both subcommands share; the exit decision — min over class codes — lives here as a pure, unit-testable function over `&[Finding]`), and the render module the REFACTOR task extracts (`src/render.rs`: human renderer + envelope construction, deduped so both subcommands build envelopes one way).
  - `tests/` integration targets: the exit-code/envelope suite, the §5 walkthrough test (task 7), and `tests/fixtures/`: `bowling.ct.json` (violating — engineered to reproduce the §5 witness exactly), `bowling-fixed.ct.json` (plus the proposed equation → clean), a dangling fixture, the multi-class fixture, and a malformed fixture (duplicate ident at a known line — the spec's own example). Fixture models are real kernel documents: parse → build → check must produce the named findings; a fixture whose findings are hand-edited to match is a failed fixture.
  - New/edited files keep the repo's Purpose/Responsibilities/Rationale header discipline (as `findings.rs` does).
- `Cargo.toml` (workspace root): `members` gains `"crates/catherine-cli"`. Nothing else changes.
- `README.md`: CLI usage + **the exit-code table from §1 verbatim** (task 6 mandates README; the table is the contract CI authors will copy).
- `CHANGELOG.md`: **decided — yes, an entry is required.** Task 6 names `--help` + README; it does not name CHANGELOG. But the standing epic decision is that **changelog entries start at P3**, and P3 is precisely this ticket — the first user-facing surface. So the CHANGELOG gains the Unreleased → Added entry describing `catherine check`/`catherine findings`, the exit-code contract, and the genesis-vibes adoption (the suite convention the existing Unreleased section already anticipates: "genesis-vibes adopted for CLI plumbing (P3)"). Stated plainly: README because tasks.md task 6 says so; CHANGELOG because the standing decision starts entries at this phase and this is the phase.
- `openspec/changes/add-check-cli/**`: tick `tasks.md` checkboxes; record `ro5u-review.md` (criterion 9); touch `specs/cli/spec.md` only if implementation exposes a genuine contract gap (say so in the report if so).

**Explicitly not this ticket**: task 9's `bd close catherine-86i` (+ fresh `bd` export) is **orchestrator-reserved** — the implementor must NOT run `bd close` or any `bd` mutation; the orchestrator closes the ticket after verification. Everything else in task 9 (`just ci` green, envelope validated against genesis-vibes expectations, `openspec validate add-check-cli --strict`) is the implementor's. **No changes under `crates/catherine-core/**`** — proposal Impact: `catherine-core` unchanged; any diff there is a §10 trigger, not a fix to make quietly. No new repo-root plumbing beyond the workspace member line.

**Genesis dependency route (resolved, per the donor ecosystem):** siblings consume genesis-vibes from **crates.io by version** — `genesis-vibes = "0.13"` (dulce-de-leche, espectacular, testaruda, specodelic), `"0.13.1"` (whisper, dont); the one git-tag route (`evallerina`, `tag = "v0.12.3"`) is the outdated outlier; feature-gated `update-check` appears only where a tool wants phone-home (specodelic, wai) — catherine does not. So: `genesis-vibes = "0.13"` from crates.io, default features (genesis's only feature is opt-in `update-check`), lib name `genesis`. Do not pin a git dep; do not vendor.

## 7. TDD sequence — RED → GREEN → REFACTOR

Run one cycle, exactly as `tasks.md` orders it (RED = tasks 1–2, GREEN = tasks 3–7, REFACTOR = task 8, Validation = task 9). State the transition evidence (command + observed failure mode) in the report at each transition.

### RED — the acceptance fixtures against a binary that does not exist

The ticket names the red plainly: **"no binary exists"**. Author the two acceptance fixtures and their tests first (tasks 1–2): the violating fixture pinned to the exact envelope JSON shape (`data.findings` = one violation with equation id, row id, both path values; envelope keys present) **and the violation exit code**; the malformed fixture pinned to the labeled-error envelope, the error exit code, and line/column presence. Both asserted structurally, never byte-compared. Because the tests spawn the binary, hosting them requires the minimal crate skeleton (`Cargo.toml` + an empty `main.rs`) — that skeleton is scaffolding for authoring the red, not the GREEN work: no clap wiring, no renderers, no exit logic. The **observed red**, recorded once in the report: `cargo run -p catherine-cli -- check tests/fixtures/bowling.ct.json` fails — no such binary/target exists — and the test target fails against the empty binary (non-zero exit where 3 is expected; no output where the envelope is expected). A red that passes for any reason is a failed red.

- **Narrow**: the two fixture tests — red as described.
- **Full**: `cargo test -p catherine-cli` — red here is expected; do not fix forward.
- Transition when both fixtures pin their expectations and the red is recorded.

### GREEN — wiring, renderers, exit codes, the walkthrough

Minimal implementations, in tasks.md order:

- **Crate + clap wiring** (task 3): `catherine-cli` with `check`/`findings` subcommands; genesis stable modules in — envelope/guide (verbosity/format structs) first; suggestions/config wired where they standardize unknown-arg hints and config loading. Kernel untouched.
- **Renderers over `findings`** (task 4): human renderer producing the §5 format (kind label, `eq-<n>`, `∘`/`≠` paths, indented witness line) and the labeled-error rendering (line/column, stable code, remediation hint — never a stack trace); `--json` emits the genesis envelope with `data.findings` = P2's serialization **verbatim**. One execution path; `findings` (task 5) is the presentation-only variant.
- **Exit-code table** (task 6): the min-over-class-codes function per §1's table; documented in `--help` and README **in the same commit** — an undocumented exit code is not done.
- **Walkthrough scripted** (task 7): the two-phase integration test — violating fixture → exit 3 + transcript-matching violation block; fixed fixture → exit 0, zero findings. This is the durable form of criterion 10.
- **Narrow**: the RED fixture tests green, then the walkthrough test green.
- **Full**: `cargo test -p catherine-cli` and `cargo test --workspace` green (kernel suites unregressed).
- Transition when narrow targets are green and no kernel fixture was touched.

### REFACTOR — structure only, no behavior expansion

- **Clap arg structure** (ticket REFACTOR): subcommand/flag layout tidied against genesis guide's structs — the public CLI surface unchanged.
- **Extract render/format module; dedupe envelope construction** (task 8): one envelope-construction path, one renderer module; kinds, witnesses, envelope keys, exit codes, streams — unchanged.
- **Errors to stderr, findings to stdout** (ticket REFACTOR — verify it holds everywhere after the extraction): the labeled-error path and usage errors write stderr; findings and clean output write stdout; the `--json` error envelope is on stderr too.
- **Narrow + full**: re-run the fixture tests, the walkthrough test, and `cargo test --workspace` — green, behavior-identical.

## 8. ro5u review completion gate

Before declaring done, run a rule-of-5-universal review of the **full diff** (invoke explicitly: `/skill:rule-of-5-universal`). Gate:

- **max_critical = 0** — no exceptions.
- Critical and high findings: fixed.
- Medium findings: fixed, or deferred **with a written reason** in the report.
- Every behavior-impacting fix adds or updates a test (structural JSON assertions, numeric exit-code assertions, or the walkthrough test — never snapshots; the repo has no snapshot discipline and P2's anti-snapshot rule carries forward).
- The review artifact is recorded at `openspec/changes/add-check-cli/ro5u-review.md` (findings + dispositions) and cited in the implementor report. ro5u is a completion criterion: without it the ticket is not done regardless of test status.

## 9. Verifier evidence contract (wai-px4g)

Applies to the verify step that follows this ticket:

- The review artifact may record **CONFIRM only when it cites at least one read artifact path** (e.g. `.wai/projects/catherine/research/…`) **and includes fresh test-run output** — the actual command plus its result, pasted. Lacking either: **REVIEW_REQUIRED**, never CONFIRM on claims alone.
- **Single-model rule**: no cross-family verifier is configured for this run, so the lead-side degradation path applies — skip the spawned verifier; the lead runs the lead-side checks instead (state checkpoint, `git log`/`git diff` vs the report, fresh test-suite run, beads state). This degradation is non-blocking, but the **recorded verdict is REVIEW_REQUIRED with a lead-side-only note — never CONFIRM**.
- A contradiction between report and reality fails the run: fix or file it, then re-verify fresh. For this ticket specifically, the walkthrough paste (criterion 10) is the highest-value cross-check: the pasted output must be reproducible by re-running the two commands from the report.

## 10. Follow-up threshold

`tasks.md` enumerates exactly **9 tasks** — that is the whole change. Stop and file follow-up beads tickets instead of expanding when:

- implementation demands more than the 9 tasks (new subcommands, extra flags beyond `--json` + the genesis-standard verbosity/format, extra fixture classes beyond §6's list);
- **kernel changes start leaking in** — any diff under `crates/catherine-core/**`: the proposal fixes the dependency direction (`cli → core`) and the Impact line says `catherine-core` unchanged; a needed core change (new API, new error code, serialization tweak) is a P-core follow-up, file it, never patch the kernel from the CLI ticket;
- **P4 analysis work starts leaking in** — a need to *produce* redundant/contradiction/underconstraint at runtime (bounded search, rewriting, parallel-path detection, `query.rs` pull-in): P3 renders and exits, P4 computes;
- P5/P6/P7/P8 surface leaks in — wasm, UI, migration, rule lens, ledger entries;
- genesis modules beyond **envelope/guide/suggestions/config** start looking necessary, or the `update-check` feature, extra dependencies beyond the §6 pinned set, a second serialization, or an alias/short bin name come up;
- the exit-code numerals drift from §1's table, or any exit contract starts needing text parsing;
- the context ceiling in §4 is exceeded, or a completion criterion in §3 proves impossible as specified.

File the follow-ups with `bd`, note them in the report, and finish only the 9 tasks. Growing the change silently is a failure mode, not diligence.

## Hard constraints (binding)

1. **Kernel crates stay genesis-free.** genesis-vibes appears in `catherine-cli` only, stable modules only (envelope/guide/suggestions/config); criterion 11's dependency-tree check is part of done.
2. **Exit codes exactly per the change**: clean = 0, parse/build failure distinct from any finding, precedence violation > dangling > contradiction > underconstraint > redundant as adjudicated in the change text — numerals pinned in §1's table, documented in `--help` + README, never derived from parsed text.
3. **Findings JSON is P2's encoding, printed verbatim** — `data.findings` carries `Vec<Finding>` exactly as `catherine-core` serializes it (stable field order, tagged kind/witness, byte-identical across repeated runs); the CLI adds the envelope, never a reshaping.
4. **No text-parsing exit contracts** (`anti_goal`): exit codes computed from findings; structural JSON assertions and numeric exit assertions in tests.
5. **Errors to stderr, findings to stdout**; labeled errors with line/column, stable code, remediation hint — never a stack trace, never a silent success.
6. **P0–P2 suites stay unregressed** — parse goldens, round-trip, parity gate, check/findings fixtures and properties all green at the end; `catherine-core` diff-free.
7. **The §5 walkthrough passes honestly**: the violation block verbatim from a real kernel run on a real fixture; the underconstraint line P4-pending and never fabricated; the loop demonstrated by the violating → fixed re-check pair.
