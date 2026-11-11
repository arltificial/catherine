# Implementor brief — catherine-0gk (P1: instance + executed checks, the E3 bet)

- **Ticket**: `catherine-0gk` — P1: instance tables + executed checks (beads: task, lane `afk`, blocked-by `catherine-bpz` ✓, blocks `catherine-1x7`)
- **Openspec change**: `add-executed-checks` (0/12 tasks at brief time)
- **Base**: `main` @ `1f62581` (post-P0-merge; the ticket's `base_commit` metadata `34da242` predates the P0 merge and is superseded by this)
- **Scope**: `crates/catherine-core/**` + `fixtures/parity/**` + `openspec/changes/add-executed-checks/**` (no scaffold files this time — workspace + justfile exist from P0)

## 1. Ticket ID and desired outcome

Desired outcome: **executed path-equation checks** — the phase that justifies
the project's existence (E3: "declarations do not enforce; checks do … if
nothing else ships, this must", DESIGN §0/§6.2). On top of P0's parsed
`Model`, a new crate surface that (a) builds typed **instance tables** from
the parsed presentation (dense per-object row ids, one
`Vec<Option<RowId>>` partial morphism table per arrow — ported from
specodelic `acset/instance.rs`, 925 LOC, Apache-2.0), (b) exposes
`check(schema, instance) -> Vec<CheckOutcome>` that executes **every
declared equation on every applicable row**, emitting
`Violation{equation, row, lhs, rhs}` and `Dangling{arrow, src}` outcomes
**with witness rows** — one outcome per violating row, not one per
equation, (c) enforces arrow typing **at instance build** (an arrow value
naming a row of the wrong object is a build-time typing violation, never a
silently stored value), and (d) lands the `status` semantics deferred from
P0: `accepted` equation failures are violations; `proposed` equation
failures are **hypothesis-labeled** outcomes, distinguishable and countable
separately. Totality rules: empty schema or empty instance → **zero
outcomes, OK** (silence is real success, not an unevaluated declaration);
self-loop arrows are legal and evaluated; a dangling value short-circuits
path composition at that arrow — recorded, not crashed, and never reported
as a violation. Everything downstream (P2 findings taxonomy, P3 CLI) counts
these outcomes; nothing downstream re-derives them.

## 2. Why

Delegation rationale — why a subagent implements instead of the lead session:

- The epic-orchestrator canon forbids inline execution by the lead
  (Invariant 1: inline is the historical failure mode); the lead session
  orchestrates only (claim → brief → spawn → verify → ship).
- The ticket is fully specified without conversation context: openspec
  change with spec deltas (`specs/kernel-checks/spec.md`), 12 enumerated
  tasks, and a verified donor (`specodelic/src/acset/instance.rs` + its
  parity-harness precedent `tests/acset_parity.rs`). A blank spawned
  session can execute it from repo artifacts alone.
- The RED→GREEN→REFACTOR cycle is long-running (cargo builds, proptest
  shrinking, parity-fixture iteration against a foreign corpus); running it
  inline would stall the lead loop for the whole cycle.
- Isolation gives clean verification: the lead cross-checks the
  implementor's report against `git` reality and fresh test runs
  (Invariant 5), with no risk of the author explaining away its own
  failures. Single-model run → the degradation path of §9 applies and the
  recorded verdict is REVIEW_REQUIRED.

## 3. Completion criteria

Every criterion is a runnable command; **exit 0 = done** — with one named,
deliberate exception (criterion 6). All exist and can run from the current
base (workspace, justfile, and P0 targets are in place). From the repo root:

1. `cargo test -p catherine-core` — full suite, now covering the new
   parity gate target, the four RED golden fixtures, and the property
   targets, with all P0 tests unregressed.
2. `cargo test -p catherine-core --test parity_gate` — the **named parity
   target** the proposal requires: vendored fixtures under `fixtures/parity/`
   produce findings identical to specodelic's acset behavior on every
   shared case.
3. `cargo test -p catherine-core --test golden_fixtures` — P0's
   line-accurate parse goldens stay green; no fixture weakened.
4. `just ci` — workspace gate (fmt + clippy `-D warnings` + tests).
5. `openspec validate --strict --all` — exit 0.
6. `ah check --changes add-executed-checks` — **reports ONLY no-toml
   structural findings.** This criterion is deliberately NOT exit-0: the
   change's scenarios have no contract tomls yet because contracts are born
   at apply time (repo workflow), so `ah check` legitimately exits 1 with
   only those structural findings. The implementor must report it as such —
   "only no-toml structural findings, exit 1 expected" — never as a
   failure, and never paper over substantive findings (missing
   requirements, orphan scenarios, spec/tasks mismatches) behind that exit
   code. Substantive findings = stop and report (§10).
7. `openspec/changes/add-executed-checks/tasks.md` shows **12/12** ticks
   (verify: `openspec list` reports `12/12`).
8. **Parity gate result recorded in the change** (task 9): the run over
   `fixtures/parity/` and its result are recorded inside
   `openspec/changes/add-executed-checks/` **before** any rewrite of
   ported logic — visible in the change tree, cited in the report. This is
   a proposal requirement, not a convention.
9. ro5u review gate passed (§8): rule-of-5-universal review artifact of the
   full diff with **max_critical = 0**.

A criterion that cannot be executed is not a completion criterion — if any
command above is impossible as specified, stop and report (§10), do not
substitute.

## 4. Spawn model

- **Implementor model**: `openrouter/z-ai/glm-5.3-flash`.
- **Context ceiling**: **≤ 60% of the implementor's configured context
  window, computed at spawn time** from the actual window (assume 1M for
  this model). No fixed token counts — if the configured window differs at
  spawn, recompute the absolute ceiling from that window. If the ticket
  cannot be executed within the ceiling, stop and split per §10.
- **Session**: named, resumable spawn `subagent:catherine-0gk:implement`;
  output streamed to a known log path recorded in
  `.wai/projects/catherine/runs/catherine-0gk.state` (Invariants 2, 7).
- **Wall-clock budget**: wrap the spawn in `timeout 5400` (90 min). Never a
  bare foreground call that blocks to the harness timeout; background +
  tail + poll (Invariant 8).
- **Retry**: fresh named session `subagent:catherine-0gk:implement-retry`;
  zero commits = clean retry, partial commits = manual resume. An
  interrupted-but-healthy spawn (commits or session record exist) resumes
  the existing session, never restarts from scratch.
- **Verification side**: no cross-family verifier is configured for this
  run → single-model degradation path applies (§9): lead-side verify, and
  the recorded verdict is **REVIEW_REQUIRED**.

## 5. Out of scope

From ticket `anti_goals`, the proposal, and DESIGN's phase gates — touching
any of these is a scope violation; stop and file follow-ups (§10) instead:

- **No unevaluated declarations (E3).** Shipping equation representation
  without execution is the exact failure mode this ticket exists to kill
  (Catlab's ACSet path equations are documentation — smoke gate S3).
  Every declared `accepted` equation is executed on every applicable row;
  a consistent row emits nothing *because it was checked*, never because
  it was skipped.
- **Findings never without witnesses.** A violation carries the equation,
  the offending row, and both composed path values; a dangling outcome
  carries the arrow and the source row. An outcome that cannot name its
  witnesses is a bug, not a finding.
- **No P2 taxonomy work.** `catherine-1x7` owns the findings taxonomy
  (enum + witnesses + golden fixtures per finding type). P1's
  `CheckOutcome` stays a minimal, open/extensible enum — the kinds this
  ticket needs (violation, dangling, hypothesis-labeled; typing violation
  is a *build report*, not a check outcome) with a documented extension
  path. No severity merging, no new kinds beyond those, no taxonomy
  refinements.
- **No `query.rs` port.** Checked against the reuse map (DESIGN §3):
  specodelic `src/acset/query.rs` (264 LOC, reachability/fan-in) is the
  **P4 analysis** donor; no task in this change's 12 requires it. It stays
  untouched until P4.
- No CLI (`catherine check` / `catherine findings` are P3), no exit-code
  mapping, no wasm, no UI, no analysis/bounded search (P4), no migrations
  (P7), no corpus/ledger (P6).
- **Dependencies stay `serde`, `serde_json`, `proptest`.** No new crates.
- **No new `ErrorCode` variants.** `error.rs`'s six-code enum is closed at
  P0 by design; check findings are `CheckOutcome`s, not parse errors.
  Extending the enum is a follow-up, never a silent addition.
- No `genesis-vibes` anywhere in the kernel crate (P0 hard constraint 1
  carries forward).

## 6. Exact files

Create/touch **only**:

- `crates/catherine-core/**`:
  `src/instance.rs` (typed instance tables ported from specodelic
  `acset/instance.rs`: dense row ids, per-arrow `Vec<Option<RowId>>`,
  build-time typing + dangling reports),
  `src/checks.rs` (`check(schema, instance) -> Vec<CheckOutcome>`, path
  composition, status split), module wiring in `src/lib.rs`,
  `README.md` (attribution table extension, §7 REFACTOR),
  tests: `tests/parity_gate.rs` (the named parity target, criterion 2),
  RED golden fixtures + their test target(s) under `tests/` (P0's
  `tests/golden/` discipline continues for `.ct.json` goldens).
  `Cargo.toml` gets no new dependencies.
- `fixtures/parity/**` (new repo-root directory): parity fixtures vendored
  from specodelic's corpus — schema/instance exports as `.ct.json` plus an
  **expected finding set** per fixture. Follow specodelic's own parity
  discipline (`tests/acset_parity.rs`): each fixture is minimal and
  triggers exactly one named shape (clean, dangling, typing-allowed,
  typing-violation), mirroring its
  `tests/fixtures/{small,zero_file,single_intent,dangling,typing_allowed,typing_violations}`
  corpus layout. Include the degenerate empty cases (empty schema, empty
  instance) — they are boundaries, expected clean.
- `openspec/changes/add-executed-checks/**`: tick `tasks.md` checkboxes;
  record the parity gate result in the change (criterion 8) **before any
  rewrite of ported logic**; touch `specs/` only if execution exposes a
  genuine contract gap (say so in the report if so).

**Explicitly not this ticket**: task 12's `bd close catherine-0gk` (+ fresh
`bd` export) is **orchestrator-reserved** — the implementor must NOT run
`bd close` or any `bd` mutation; the orchestrator closes the ticket after
verification. Everything else in task 12 (`just ci` green,
`openspec validate add-executed-checks --strict`) is the implementor's. No
scaffold files are needed this time: root workspace `Cargo.toml` and
`justfile` exist from P0 and must not change beyond what a new module
requires (nothing — `just ci` already builds the workspace).

## 7. TDD sequence — RED → GREEN → REFACTOR

Run one cycle. State the transition evidence (command + observed failure
mode) in the report at each transition.

### RED — checks fail before they exist

Write the four golden fixtures (tasks 1–4) as executable tests pinning the
expected outcomes. A compile error on the not-yet-existing `check`/builder
API is an acceptable *observed-red once* (specodelic's parity precedent);
after the harness compiles, every fixture must fail **behaviorally** — each
test names the outcome kind and witnesses it expects — never on setup
breakage. Fixtures under `tests/golden/` as `.ct.json`, table-driven:

| Fixture | Expectation |
|---|---|
| one violating row on an **accepted** equation | `Violation` with witness: equation, row, both composed path values (`lhs`, `rhs`) |
| dangling arrow value | `Dangling` with witness: arrow, source row |
| arrow value naming a row of the wrong object | **build-time** typing violation naming row + arrow; value does **not** become a morphism table entry |
| **proposed** equation failing on a row | hypothesis-labeled outcome — a distinct kind from `Violation`, separately countable |

Also pin in RED: an all-consistent instance yields **zero** outcomes
(silence is real success), and a dangling on one arrow neither crashes the
run nor turns other rows' failures into violations.

- **Narrow**: the new golden target — must FAIL per-fixture on its named
  missing behavior.
- **Full**: `cargo test -p catherine-core` — red here is expected; do not
  fix forward.
- Transition when every fixture fails for its named reason.

### GREEN — port behind the parity gate, execute the equations

Minimal implementations, adapting the donor (do **not** author the instance
layer from scratch):

- **Instance tables** (`instance.rs`): adapt specodelic
  `src/acset/instance.rs` — dense, bidirectional interning per object;
  one partial `Vec<Option<RowId>>` morphism table per arrow (`None` =
  dangling, a representable value never dropped or repaired); typing
  violations and dangling values are **named reports**, never silent
  drops or hard failures. Kept donor conventions: violations never stored
  as morphism values; every parsed arrow value lands somewhere (morphism
  value, dangling record, or typing violation) — the builder is **total
  on parsed presentations** and typing-enforced at build. Dropped as
  specodelic-domain: `graph::build` parity machinery, walk-order edge
  recording, attribute cells, `Spec`/markdown parsing (P0's parser owns
  the input). Self-loops are legal (P0's `Arrow` already allows them).
- **Executed checks** (`checks.rs`): `check(schema, instance)` composes
  each equation's `lhs`/`rhs` path **per row of the equation's domain
  object** and compares — this part is **new code** (E10: specodelic has
  no path-equation execution; equations were already new in P0), with its
  own property tests. Status split lands here (the decision deferred from
  P0): `accepted` failure → `Violation`; `proposed` failure →
  hypothesis-labeled outcome; never merged into one severity. Dangling
  short-circuit: a path through a `None` cell cannot compose — the
  affected rows are recorded as not-evaluable-through-dangling, **never**
  emitted as violations, and the run continues (recorded, not crashed).
  Totality: empty schema or empty instance → zero outcomes.
- **Parity gate** (E8): `tests/parity_gate.rs` runs the vendored
  `fixtures/parity/` set and asserts findings match specodelic's acset
  behavior on every shared case (typing allowed/violated, dangling,
  clean, empty). **Run it and record the result in the change (task 9)
  BEFORE any rewrite of ported logic.** The gate covers the shared acset
  behavior — instance tables, typing, dangling — not equation semantics,
  which the donor does not have.
- **Property test** (proptest, **never insta-snapshots**): empty
  schema/instance → zero outcomes; self-loops legal and evaluated like
  any arrow; outcomes asserted structurally (kind + witnesses), never
  byte snapshots. The vendored expected-finding sets in `fixtures/parity/`
  are the parity contract, not snapshots.

- **Narrow**: the golden target green, then `cargo test -p
  catherine-core --test parity_gate` green.
- **Full**: `cargo test -p catherine-core` green.
- Transition when narrow targets are green and no fixture was weakened.

### REFACTOR — structure only, no behavior expansion

- Extract the **shared path-composition helper** (composition + dangling
  short-circuit in one place, used by `check` and available to future
  consumers); align the outcome/error shapes with P0's conventions
  (`error.rs` discipline: closed kinds, stable `as_str` spellings —
  outcomes carry witnesses where parse errors carry line/column).
  Findings, kinds, and witnesses must not change.
- **Attribution**: the donor ships Apache-2.0 **without** a NOTICE file,
  so each ported file carries an Apache-2.0 header naming the upstream
  repo + source path, and the crate README's attribution table gains the
  `instance.rs` row (extends P0's table; same discipline).
- **Narrow + full**: re-run both narrow targets and
  `cargo test -p catherine-core` — green, behavior-identical.

## 8. ro5u review completion gate

Before declaring done, run a rule-of-5-universal review of the **full
diff** (invoke explicitly: `/skill:rule-of-5-universal`). Gate:

- **max_critical = 0** — no exceptions.
- Critical and high findings: fixed.
- Medium findings: fixed, or deferred **with a written reason** in the
  report.
- Every behavior-impacting fix adds or updates a test (proptest or golden
  fixture — structural assertions, never snapshots).
- The review artifact (findings + dispositions) is cited in the implementor
  report. ro5u is a completion criterion: without it the ticket is not done
  regardless of test status.

## 9. Verifier evidence contract (wai-px4g)

Applies to the verify step that follows this ticket:

- The review artifact may record **CONFIRM only when it cites at least one
  read artifact path** (e.g. `.wai/projects/catherine/research/…`) **and
  includes fresh test-run output** — the actual command plus its result,
  pasted. Lacking either: **REVIEW_REQUIRED**, never CONFIRM on claims
  alone.
- **Single-model rule**: no cross-family verifier is configured for this
  run, so the lead-side degradation path applies — skip the spawned
  verifier; the lead runs the lead-side checks instead (state checkpoint,
  `git log`/`git diff` vs the report, fresh test-suite run, beads state).
  This degradation is non-blocking, but the **recorded verdict is
  REVIEW_REQUIRED with a lead-side-only note — never CONFIRM**.
- A contradiction between report and reality fails the run: fix or file it,
  then re-verify fresh.

## 10. Follow-up threshold

`tasks.md` enumerates exactly **12 tasks** — that is the whole change. Stop
and file follow-up beads tickets instead of expanding when:

- implementation demands more than the 12 tasks (new phases, extra fixture
  classes beyond the four RED + parity sets);
- a **`query.rs` pull-in** starts looking necessary (it is P4's donor —
  needing reachability/fan-in inside P1 means scope creep, file it);
- a **donor mismatch needs redesign rather than adaptation** — e.g.
  specodelic's `Dangling`/`Violation`/`Value` shapes or its
  `graph::build`-comparison machinery do not map onto catherine's
  `RowId`-keyed `BTreeMap` presentation and would require redesigning the
  donor's contract instead of adapting its conventions;
- new dependencies beyond `serde`/`serde_json`/`proptest`, new error-code
  variants, or new check-outcome kinds beyond the ticket's set start
  looking necessary (outcome-kind growth is P2's `catherine-1x7`);
- the context ceiling in §4 is exceeded, or a completion criterion in §3
  proves impossible as specified.

File the follow-ups with `bd`, note them in the report, and finish only the
12 tasks. Growing the change silently is a failure mode, not diligence.

## Hard constraints (binding)

1. **E3 — executed checks are the product.** Every declared `accepted`
   equation is executed on every applicable row; findings never without
   witnesses; silence after evaluation is real success, never an
   unevaluated declaration. Equations without execution = bug.
2. **Adapt specodelic `instance.rs`; do not author the instance layer from
   scratch.** Kept (adapted): dense bidirectional interning, partial
   morphism storage (`None` = dangling, never dropped), typing violations
   reported and never stored as morphism values, collisions/reports named
   never swallowed, total builder. Dropped as specodelic-domain:
   `graph::build` parity machinery, walk-order edge recording, attribute
   cells, markdown `Spec` parsing. The path-equation execution itself is
   new code (E10) with its own property tests.
3. **Parity gate mechanics per the proposal (E8):** fixtures vendored from
   specodelic's corpus under `fixtures/parity/` with expected finding sets,
   run as a **named test target** (`tests/parity_gate.rs`), result recorded
   in the change **before any rewrite of ported logic**.
4. **proptest, never insta-snapshots.** Outcomes asserted structurally
   (kinds + witnesses); the vendored expected-finding sets are the parity
   contract, not snapshots.
5. **Status semantics land HERE** — the decision deferred from P0:
   `accepted` → violation; `proposed` → hypothesis-labeled, distinguishable
   in the output, never merged into one severity. A proposed equation is a
   domain question under review, not a broken invariant.
6. **Apache-2.0 attribution preserved** on all ported code (per-file header
   + README attribution table; the donor has no NOTICE file).
7. **Kernel crates genesis-free** — no `genesis-vibes` in
   `catherine-core` (carried from P0).
8. **The outcome enum stays open for P2** (`catherine-1x7` owns the
   taxonomy) — P1 adds only the kinds it needs and documents the extension
   path; no taxonomy work now.
