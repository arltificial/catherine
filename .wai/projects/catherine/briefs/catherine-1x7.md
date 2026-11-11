# Implementor brief — catherine-1x7 (P2: findings taxonomy — typed enum with witnesses and epistemic labels)

- **Ticket**: `catherine-1x7` — P2: findings taxonomy — enum + witnesses (beads: task, lane `afk`, blocked-by `catherine-0gk` ✓ done, blocks `catherine-86i`; complexity `s`; labels `ct-kernel`)
- **Openspec change**: `add-findings-taxonomy` (0/9 tasks at brief time)
- **Base**: `main` @ `809be72` (the P1 feat commit; only `01aa733`, a `chore(beads)` export, sits atop). The ticket's `base_commit` metadata `34da242` predates both the P0 and P1 merges and is superseded.
- **Scope**: `crates/catherine-core/**` + `openspec/changes/add-findings-taxonomy/**` (change-dir evidence files only — see §6)

## 1. Ticket ID and desired outcome

Desired outcome: **the public findings taxonomy** — the output contract every
later phase prints or consumes. On top of P1's executed checks, a new
`findings` surface in `catherine-core` that (a) defines the public
**`Finding` enum for v1 — violation, dangling, redundant, contradiction,
underconstraint — each variant carrying its concrete witness payload**
(equation id + row + both path values; arrow + source row; derivable path +
reducing path; explicit scope; the domain-question witness as a tagged union
`ParallelPaths{lhs, rhs}` | `UncoveredInputs{class, examples}` — the latter
reserved for P8's rule-lens gap mapping), (b) attaches an **epistemic label
to every finding** — `executed` (kernel-produced), `hypothesized`
(proposed-equation failures, underconstraint questions), `unknown`
(scope-bounded non-answers) — with `proved`/`illustrated` **not
representable** in v1 (reserved for later layers, never fabricated by the
kernel), (c) **re-shapes P1's check outcomes into findings** (see the
reconciliation below), (d) encodes findings as **stable machine-readable
JSON (serde)** that round-trips and orders **deterministically (by kind,
then witness ids)** so repeated checks of the same model are byte-identical,
and (e) lands **golden fixtures per finding kind**, asserted structurally —
violation/dangling from real kernel runs; redundant/contradiction/
underconstraint fixture *shapes* pinned for P4's producers (the taxonomy and
fixtures exist here; the producers do not — see §5).

**Finding vs `CheckOutcome` — the relationship, stated explicitly (per the
proposal's actual text, not invented):** the proposal says *check outcomes
re-shaped into findings* (Impact) and task 5 says *Reshape P1 check outcomes
into findings; update P1 fixtures*. That is an **evolve (replace), not
extend, not wrap**: the three-variant `CheckOutcome` enum in
`src/checks.rs` is **replaced** by the public `Finding` enum in a new
`findings` module, and `check(schema, instance) -> Vec<CheckOutcome>`
becomes `check(schema, instance) -> Vec<Finding>`. No second parallel enum
and no newtype wrapper survives. The mapping is fixed by the spec:

| P1 `CheckOutcome` | P2 `Finding` | witness | label |
|---|---|---|---|
| `Violation{equation, row, lhs, rhs}` | violation | same four fields | `executed` |
| `Hypothesis{equation, row, lhs, rhs}` | violation *(same shape, not a new kind)* | same four fields | `hypothesized` |
| `Dangling{arrow, src}` | dangling | same two fields | `executed` |
| *(new, no P1 producer)* | redundant | derivable path + reducing path | P4 labels at construction |
| *(new, no P1 producer)* | contradiction | explicit scope | P4 labels at construction |
| *(new, no P1 producer)* | underconstraint | tagged `ParallelPaths{lhs, rhs}` \| `UncoveredInputs{class, examples}` + domain-question text | `hypothesized` |

Key consequence: P1's separate `Hypothesis` *kind* disappears as a kind —
kind names the **shape** of the fact, the epistemic label names its
**weight**. "Distinguishable and countable separately" (the P1 status
decision) is preserved by counting on `(kind, label)`, never by merging
severity: an accepted-equation failure and a proposed-equation failure are
both violation-shaped findings with different labels, and a consumer can
still count them apart. The exact Rust shape is the implementor's call
within the spec's invariants (e.g. `struct Finding { label: EpistemicLabel,
kind: FindingKind }` with `FindingKind` the five-variant witness enum):
**no variant without its witness** (invalid states unrepresentable — task
3), **kernel constructors accept only `executed`/`hypothesized`/`unknown`**
(task 4), and the label must be independent data on the finding (violation
is the one kind that maps to two labels). P1's existing callers and fixtures
(`check_fixtures.rs`, `parity_gate.rs`, property targets) are migrated in
the same commit series; `TypedInstance::typing_violations` stays a **build
report, not a finding** — unchanged.

Everything downstream counts on this: P3's `catherine findings` prints the
JSON verbatim; P5's wasm boundary consumes it; P4 emits redundant/
contradiction/underconstraint into it. Nothing downstream re-derives a
finding's shape.

## 2. Why

Delegation rationale — why a subagent implements instead of the lead session:

- The epic-orchestrator canon forbids inline execution by the lead
  (Invariant 1: inline is the historical failure mode); the lead session
  orchestrates only (claim → brief → spawn → verify → ship).
- The ticket is fully specified without conversation context: openspec
  change with spec deltas (`specs/findings/spec.md`), 9 enumerated tasks,
  and the P1 surface (`checks.rs`, `check_fixtures.rs`, `tests/golden/`
  discipline) already landed and reviewed at `809be72`. A blank spawned
  session can execute it from repo artifacts alone.
- The RED→GREEN→REFACTOR cycle is long-running (cargo builds, a stability
  proptest, fixture iteration across five kinds); running it inline would
  stall the lead loop for the whole cycle.
- Isolation gives clean verification: the lead cross-checks the
  implementor's report against `git` reality and fresh test runs
  (Invariant 5), with no risk of the author explaining away its own
  failures. Single-model run → the degradation path of §9 applies and the
  recorded verdict is REVIEW_REQUIRED.

## 3. Completion criteria

Every criterion is a runnable command; **exit 0 = done** — with one named,
deliberate exception (criterion 6). All run from the repo root on the P1
base (workspace, justfile, P0+P1 targets in place):

1. `cargo test -p catherine-core` — full suite: the new findings targets
   (golden fixtures per kind, serde/stability property tests) green, with
   **all P0 and P1 suites unregressed** (parse goldens, parity gate,
   round-trip, P1 check fixtures reshaped per task 5 — no fixture weakened
   or deleted to make it pass).
2. `cargo test -p catherine-core --test check_fixtures` — the reshaped P1
   fixtures: same documents, same facts, now asserting `Finding` kind +
   witness + label structurally.
3. `cargo test -p catherine-core --test golden_fixtures` — P0's parse
   goldens stay green (the `tests/golden/` discipline continues for the new
   findings fixtures).
4. `cargo test -p catherine-core --test parity_gate` — the P1 parity gate
   stays green under the reshaped surface.
5. **JSON findings serialization validated by concrete runnable assertions**
   (task 6's stability property test + round-trip — these live in the suite
   and criterion 1 executes them; the assertions themselves are): (a)
   `serde_json::from_str::<Vec<Finding>>(&serde_json::to_string(&findings)?)`
   equals `findings` for every fixture and property case (Finding derives
   `Deserialize`); (b) checking the same model twice — in-process and in
   separate `check` invocations — yields **byte-identical**
   `serde_json::to_string` output; (c) serialized JSON is asserted
   **structurally** (field names/values via parsed `serde_json::Value` in
   the golden fixtures), never byte-compared to a snapshot file.
6. `ah check --changes add-findings-taxonomy` — **reports ONLY no-toml
   structural findings; 4 no-toml findings expected.** This criterion is
   deliberately NOT exit-0: the change's scenarios have no contract tomls
   yet (contracts are born at apply time), so `ah check` legitimately
   exits 1 with only those structural findings — **criterion = "only
   no-toml structural findings", never exit-0**. The implementor reports it
   exactly as "only no-toml structural findings (4), exit 1 expected" —
   never paper over substantive findings (missing requirements, orphan
   scenarios, spec/tasks mismatches) behind that exit code. Substantive
   findings = stop and report (§10).
7. `openspec validate add-findings-taxonomy --strict` — exit 0 (task 9's
   form), and `openspec validate --strict --all` — exit 0 (workspace gate).
8. `openspec/changes/add-findings-taxonomy/tasks.md` shows **9/9** ticks
   (verify: `openspec list` reports `9/9`).
9. **ro5u review artifact recorded in the change dir**:
   `openspec/changes/add-findings-taxonomy/ro5u-review.md` (matching P1's
   evidence-file discipline) — the rule-of-5-universal review of the full
   diff with **max_critical = 0**, cited in the implementor report (§8).
10. `just ci` — workspace gate (fmt + clippy `-D warnings` + tests).

A criterion that cannot be executed is not a completion criterion — if any
command above is impossible as specified, stop and report (§10), do not
substitute.

## 4. Spawn model

- **Implementor model**: `openrouter/z-ai/glm-5.3-flash`.
- **Context ceiling**: **≤ 60% of the implementor's configured context
  window, computed at spawn time** from the actual window (assume 1M for
  this model). No fixed token counts — if the configured window differs at
  spawn, recompute the absolute ceiling from that window. If the ticket
  cannot be executed within the ceiling, stop and split per §10. (Ticket
  complexity is `s`; the ceiling is still computed, not assumed small.)
- **Session**: named, resumable spawn `subagent:catherine-1x7:implement`;
  output streamed to a known log path recorded in
  `.wai/projects/catherine/runs/catherine-1x7.state` (Invariants 2, 7).
- **Wall-clock budget**: wrap the spawn in `timeout 5400` (90 min). Never a
  bare foreground call that blocks to the harness timeout; background +
  tail + poll (Invariant 8).
- **Retry**: fresh named session `subagent:catherine-1x7:implement-retry`;
  zero commits = clean retry, partial commits = manual resume. An
  interrupted-but-healthy spawn (commits or session record exist) resumes
  the existing session, never restarts from scratch.
- **Verification side**: no cross-family verifier is configured for this
  run → single-model degradation path applies (§9): lead-side verify, and
  the recorded verdict is **REVIEW_REQUIRED**.

## 5. Out of scope

From ticket `anti_goals`, the proposal's deliberate exclusions, and DESIGN's
phase gates — touching any of these is a scope violation; stop and file
follow-ups (§10) instead:

- **No snapshot tests** (`anti_goal`; DESIGN §3: insta-style snapshots are
  deliberately avoided). Findings carry witnesses and are asserted
  structurally — kinds, witness fields, labels, parsed-JSON field/value
  assertions. No `.snap` files, no byte-comparison goldens, no insta/derive
  snapshot crates.
- **No finding without a witness** (`anti_goal`): every variant carries its
  concrete witness payload; a finding that cannot name its witnesses is
  unrepresentable (task 3) — and in fixtures, a constructed finding always
  names real ids from its document.
- **No P3 CLI/rendering/exit codes.** `catherine check` / `catherine
  findings`, human formatting, exit-code mapping, and the genesis-vibes
  envelope are `catherine-86i`. P2 ships the *encoding* P3 prints verbatim,
  nothing that prints.
- **No P4 algorithm work — the split, stated precisely.** P2 owns the
  taxonomy **shape only**: the `Finding`/witness/label types, ordering, and
  serialization (tasks 3–7: task 7 says the underconstraint type is
  "defined here, emitted only by analysis (P4)"). P4 owns the **logic** —
  computing redundant (path-equivalence via normal-form rewriting),
  contradiction (bounded search admitting no instance), underconstraint
  detection (parallel-path analysis) — and emits into the taxonomy P2
  defines. Concretely forbidden in P2: no bounded search, no rewriting, no
  parallel-path *detection* at runtime, no **`query.rs` port** (specodelic
  `src/acset/query.rs` is P4's donor per DESIGN §3 reuse map). The
  redundant/contradiction/underconstraint *fixtures* construct `Finding`
  values directly to pin shape and serialization — they call no producer,
  because none exists yet.
- **Dependencies stay `serde`, `serde_json` (deps) + `proptest`
  (dev-dep).** No new crates; serde's derive feature already suffices for
  finding (de)serialization.
- **No new `ErrorCode` variants.** `error.rs`'s six-code enum is closed by
  design; findings are not parse errors and the serde story never routes
  through `ErrorCode`.
- **TS types spike is REFACTOR-phase only and carries no codegen
  dependency** (ticket REFACTOR: "TS types generation spike"): a
  hand-written TypeScript type sketch of the findings JSON, recorded as
  documentation (doc comments / change-dir note). No codegen pipeline, no
  build step, no `web/` or npm touches, no new tooling.
- No wasm (P5), no UI, no migration work (P7), no corpus/ledger (P6), no
  `genesis-vibes` anywhere in the kernel crate (carried from P0).

## 6. Exact files

Create/touch **only** (ticket metadata `files: ["crates/catherine-core"]`,
plus change-dir evidence files where tasks require recording):

- `crates/catherine-core/**`:
  `src/findings.rs` (new module: `Finding`, per-variant witness payloads,
  `EpistemicLabel` with exactly the three constructible v1 variants, the
  underconstraint tagged witness `ParallelPaths{lhs, rhs} |
  UncoveredInputs{class, examples}` + domain-question text, serde derives,
  deterministic ordering), `src/checks.rs` (re-shaped: `check` returns
  `Vec<Finding>`; the `CheckOutcome` enum and its doc's extension note are
  superseded — clean cutover, no deprecated alias kept), `src/lib.rs`
  (module wiring), and the migration of P1 callers/fixtures/tests:
  `tests/check_fixtures.rs` (task 5), `tests/parity_gate.rs` and property
  targets as the reshape touches them, new findings golden fixtures under
  `tests/golden/` plus their test target(s) under `tests/`, and the serde/
  stability property test target. `Cargo.toml` gets **no new dependencies**.
  New/edited files keep the crate's Purpose/Responsibilities/Rationale
  header discipline (as `checks.rs` does).
- `openspec/changes/add-findings-taxonomy/**`: tick `tasks.md` checkboxes;
  record `ro5u-review.md` (criterion 9); record the TS types spike note if
  doc comments are not its home; touch `specs/findings/spec.md` only if
  implementation exposes a genuine contract gap (say so in the report if
  so).

**Explicitly not this ticket**: task 9's `bd close catherine-1x7` (+ fresh
`bd` export) is **orchestrator-reserved** — the implementor must NOT run
`bd close` or any `bd` mutation; the orchestrator closes the ticket after
verification. Everything else in task 9 (`just ci` green,
`openspec validate add-findings-taxonomy --strict`) is the implementor's.
No `fixtures/parity/**` changes and no scaffold files: the taxonomy needs
no new repo-root plumbing.

## 7. TDD sequence — RED → GREEN → REFACTOR

Run one cycle, exactly as `tasks.md` orders it (RED = tasks 1–2, GREEN =
3–7, REFACTOR = 8, Validation = 9). State the transition evidence (command
+ observed failure mode) in the report at each transition.

### RED — fixtures for kinds that cannot exist yet (the unusual red)

Write the golden fixtures per finding kind (task 1) and the proposed-
equation `hypothesized` fixture (task 2) as executable tests under the
`tests/golden/` discipline — pinning, per kind, the expected `Finding`
kind + witness fields + label, asserted structurally. The RED here is
**deliberately a compile red**: the fixtures reference
`catherine_core::findings::Finding` and its analysis-kind variants
(`Contradiction`, `Underconstraint`, `Redundant`), which do not exist.
The **wrong-variant compile attempt is the legitimate observable red**:
`cargo test -p catherine-core` fails with errors naming the missing
type/variants — record that command + error in the report as the observed
red, once. **No trybuild, no compile-fail test target, no new dev
tooling** — the tasks do not mandate it and it would add a dependency and
a build cost for one transition; the durable red is the fixtures
themselves: once the enum exists (GREEN task 3), every fixture must fail
**behaviorally** — naming the kind, witness fields, and label it expects —
never on setup breakage, and a fixture passing for the wrong reason is a
failed RED. Fixtures to write:

| Fixture (task) | Constructed via | Expectation |
|---|---|---|
| violation on an accepted equation (1) | **real kernel run**: parse → build → `check` | violation finding, `executed` label, witness: equation id, row, both composed path values; one finding per violating row |
| dangling arrow value (1) | **real kernel run** | dangling finding, `executed` label, witness: arrow + source row; run continues; dangling never a violation |
| proposed equation failing (2) | **real kernel run** | violation-shaped finding with `hypothesized` label — distinguishable and countable apart from `executed` on the same row |
| redundant shape (1) | **constructed directly** (no P4 producer) | redundant finding: derivable path + reducing path; JSON shape pinned |
| contradiction shape (1) | **constructed directly** | contradiction finding: explicit scope witness; JSON shape pinned |
| underconstraint shape (1) + (7) | **constructed directly** | both witness arms pinned: `ParallelPaths{lhs, rhs}` and `UncoveredInputs{class, examples}` + domain-question text, `hypothesized` label; JSON shape pinned |

- **Narrow**: the findings fixtures target — red (compile) as described;
  after the enum lands, red per fixture on its named missing behavior.
- **Full**: `cargo test -p catherine-core` — red here is expected; do not
  fix forward.
- Transition when the compile red is recorded and every fixture names its
  expectation.

### GREEN — the taxonomy, labels, reshape, ordering, serialization

Minimal implementations, in tasks.md order:

- **`Finding` enum with witness payloads** (task 3): five variants, each
  carrying its witness (see §1 table); invalid states unrepresentable —
  no variant without its witness, no generic "any witness" constructor.
- **`EpistemicLabel`** (task 4): exactly `executed`/`hypothesized`/
  `unknown` constructible; **`proved`/`illustrated` do not exist in the
  type** — the kernel cannot fabricate them (spec: those belong to later
  layers, if ever). The label rides on the finding; kernel producers assign
  `executed` (accepted-failure violation, dangling) and `hypothesized`
  (proposed-equation failure, underconstraint) only.
- **Reshape P1** (task 5): `check(schema, instance) -> Vec<Finding>`;
  migrate `check_fixtures.rs`, `parity_gate.rs`, property targets; update
  `checks.rs` docs (the P1 "open for P2" extension note is now discharged —
  the taxonomy owns kinds; `CheckOutcome` is gone, not deprecated). The
  empty-schema/empty-instance totality pin (zero findings) stays.
- **Deterministic ordering + serde JSON + stability property** (task 6):
  a total, documented order (kind, then witness ids — kind order = variant
  declaration order; witness-id order = the canonical id order P1 already
  uses) so repeated checks are byte-identical; serde derives with stable
  field order (declaration order; ordered maps only — no `HashMap` in any
  serialized type); the stability proptest (two runs → identical
  `serde_json::to_string`) and the round-trip assertion per §3.5. This
  supersedes P1's dangling-first outcome order — update the `checks.rs`
  order doc to the P2 rule; the P1 spec's ordering requirement is restated
  by the findings spec and the P2 form is binding.
- **Underconstraint tagged witness** (task 7): `ParallelPaths{lhs, rhs}` |
  `UncoveredInputs{class, examples}` + the domain-question text; the type
  is complete here, `UncoveredInputs` is reserved for P8's rule-lens gap
  mapping — defined and fixture-pinned, **never produced in v1**.
- **Property test** (proptest, **never snapshots**): ordering stability,
  round-trip, label/kind soundness asserted structurally; golden fixtures
  assert parsed-JSON fields/values.

- **Narrow**: findings fixtures green, then the property target green.
- **Full**: `cargo test -p catherine-core` green.
- Transition when narrow targets are green and no fixture was weakened.

### REFACTOR — structure only, no behavior expansion

- **Dedupe witness construction** (task 8): shared witness types between
  `checks` and `findings` (e.g. one violation-witness struct) so the
  kernel walk constructs witnesses once; kinds, witnesses, labels, order,
  JSON — unchanged.
- **Finding docs into doc comments** (ticket REFACTOR): the taxonomy's
  contract (per-variant witness meaning, label semantics, ordering rule,
  serialization guarantees) lands as rustdoc on `findings.rs` — kept to the
  crate's header discipline.
- **TS types generation spike** (ticket REFACTOR; §5 constraints): the
  hand-written TS sketch of the findings JSON, recorded as documentation.
- **Narrow + full**: re-run both narrow targets and `cargo test -p
  catherine-core` — green, behavior-identical.

## 8. ro5u review completion gate

Before declaring done, run a rule-of-5-universal review of the **full
diff** (invoke explicitly: `/skill:rule-of-5-universal`). Gate:

- **max_critical = 0** — no exceptions.
- Critical and high findings: fixed.
- Medium findings: fixed, or deferred **with a written reason** in the
  report.
- Every behavior-impacting fix adds or updates a test (proptest or golden
  fixture — structural assertions, never snapshots).
- The review artifact is recorded at
  `openspec/changes/add-findings-taxonomy/ro5u-review.md` (findings +
  dispositions) and cited in the implementor report. ro5u is a completion
  criterion: without it the ticket is not done regardless of test status.

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

`tasks.md` enumerates exactly **9 tasks** — that is the whole change. Stop
and file follow-up beads tickets instead of expanding when:

- implementation demands more than the 9 tasks (new phases, extra fixture
  classes, a serialization format beyond the one serde JSON encoding);
- **P4 algorithm work starts leaking in** — bounded search, redundancy /
  normal-form rewriting, runtime parallel-path detection, or any `query.rs`
  pull-in: P2 defines the shape, P4 computes the findings; a need to
  *produce* redundant/contradiction/underconstraint inside this ticket is
  scope creep, file it;
- **P3 surface work starts leaking in** — human rendering, exit codes,
  envelope formatting, CLI flags;
- new dependencies beyond `serde`/`serde_json`/`proptest`, new
  `ErrorCode` variants, a codegen pipeline for the TS spike, or extra
  epistemic labels (`proved`/`illustrated`) start looking necessary;
- the context ceiling in §4 is exceeded, or a completion criterion in §3
  proves impossible as specified.

File the follow-ups with `bd`, note them in the report, and finish only the
9 tasks. Growing the change silently is a failure mode, not diligence.

## Hard constraints (binding)

1. **No snapshot tests.** All fixture and property assertions are
   structural — kind + witness fields + label, and parsed-JSON field/value
   checks. The JSON determinism contract is two-runs-byte-identical, not
   a stored golden byte-string.
2. **No finding without a witness.** Every variant carries its concrete
   witness payload; unrepresentable otherwise. Fixture findings name real
   ids from their documents.
3. **JSON findings round-trip and are deterministic** — serde
   (de)serialization equality, total documented ordering, no `HashMap` in
   serialized types: P3 prints this verbatim and P5's wasm boundary
   consumes it later; the encoding is a promise, not an implementation
   detail.
4. **P0/P1 suites stay unregressed** — parse goldens, parity gate,
   round-trip, and the reshaped check fixtures all green at the end; no
   fixture weakened to pass.
5. **Labels are epistemically honest** — the kernel constructs only
   `executed`/`hypothesized`/`unknown`; `proved`/`illustrated` are absent
   from the type, never fabricated. Proposed-equation failures remain
   countable apart from accepted violations (by label), never merged.
6. **The evolve cut is clean** — `CheckOutcome` is replaced by `Finding`
   per the proposal; no deprecated alias, no parallel enum, no shim.
