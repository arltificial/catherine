# Implementor brief — catherine-bpz (P0: kernel skeleton)

- **Ticket**: `catherine-bpz` — P0: kernel skeleton, model document format + parser (beads: task, lane `afk`, blocks `catherine-0gk`)
- **Openspec change**: `add-model-document-kernel` (0/17 tasks at brief time)
- **Base**: `main` @ `9e52e78a49aa6bfcafdd8825eebf43b4d065bf2f`
- **Scope**: `crates/catherine-core/**` + `openspec/changes/add-model-document-kernel/**` (+ two scaffold files, §6)

## 1. Ticket ID and desired outcome

Desired outcome: Catherine's first executable code. A new crate
`crates/catherine-core` that (a) parses `.ct.json` v1 model documents
(schema + instance) rejecting malformed presentations with **line-accurate
errors**, (b) serializes them canonically (byte-stable regardless of input
order), (c) computes a canonical-JSON + sha256 model hash ported from wild
`formats.rs`, (d) proves round-trip via proptest, and (e) publishes
`ct-schema-v1.json` + a documented `format_version` policy. Everything
downstream (P1 executed checks, P6 ledger content-addressing) consumes this
artifact. Executed checks are the product (E3) — P0 builds the substrate
they consume, nothing more.

## 2. Why

Delegation rationale — why a subagent implements instead of the lead session:

- The epic-orchestrator canon forbids inline execution by the lead (Invariant
  1: inline is the historical failure mode); the lead session orchestrates
  only (claim → brief → spawn → verify → ship).
- The ticket is fully specified without conversation context: openspec change
  with spec deltas, design decisions 1–5, and 17 enumerated tasks. A blank
  spawned session can execute it from repo artifacts alone.
- The RED→GREEN→REFACTOR cycle is long-running (cargo builds, proptest
  shrinking, fixture iteration); running it inline would stall the lead loop
  for the whole cycle.
- Isolation gives clean verification: the lead cross-checks the implementor's
  report against `git` reality and fresh test runs (Invariant 5), with no
  risk of the author explaining away its own failures.

## 3. Completion criteria

Every criterion is a runnable command; **exit 0 = done**. Commands naming
`catherine-core` / `just` cannot run before this ticket's task 1 creates
them — that is expected; they are completion criteria, not baseline
criteria. From the repo root:

1. `cargo test -p catherine-core` — full suite: unit + golden fixtures +
   property tests.
2. `cargo test -p catherine-core --test golden_fixtures` — all malformed-
   presentation fixtures pass with exact error code + line/column.
3. `cargo test -p catherine-core --test roundtrip` — round-trip property +
   row-id-canonicalization property pass (default proptest cases).
4. `just ci` — workspace gate wired by task 1 (fmt + clippy `-D warnings` +
   tests).
5. `openspec validate --strict --all` — exit 0.
6. `ah check --changes add-model-document-kernel` — exit 0.
7. `openspec/changes/add-model-document-kernel/tasks.md` shows 17/17
   checkboxes complete (verify: `openspec list` reports `17/17`).
8. ro5u review gate passed (§8): rule-of-5-universal review artifact of the
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
- **Session**: named, resumable spawn `subagent:catherine-bpz:implement`;
  output streamed to a known log path recorded in
  `.wai/projects/catherine/runs/catherine-bpz.state` (Invariants 2, 7).
- **Wall-clock budget**: wrap the spawn in `timeout 5400` (90 min). Never a
  bare foreground call that blocks to the harness timeout; background +
  tail + poll (Invariant 8).
- **Retry**: fresh named session `subagent:catherine-bpz:implement-retry`;
  zero commits = clean retry, partial commits = manual resume. An
  interrupted-but-healthy spawn (commits or session record exist) resumes
  the existing session, never restarts from scratch.
- **Verification side**: no cross-family verifier is configured for this
  run → single-model degradation path applies (§9).

## 5. Out of scope

From ticket `anti_goals` and proposal §Impact — touching any of these is a
scope violation; stop and file follow-ups (§10) instead:

- **No lossless text writer.** JSON model docs, not markdown. specodelic's
  `writer.rs` (631 LOC) is deliberately dropped.
- **`format_version` is never mutated in place.** Additive change = minor
  version; breaking change = new extension. v1 is never redefined. The
  parser accepts additive minors of the current major and rejects unknown
  majors / absent field with `unsupported_version`.
- **No schema-equivalence claims.** Behavioral `distinguish` over an
  explicit observation set is the only decidability-safe comparison, and it
  is not P0.
- **`ConstraintStatus` semantics deferred to P1** (`add-executed-checks`).
  P0 fixes the *format field only*: required per equation, enum
  `{proposed, accepted}`, defaults to `accepted` when absent, validated at
  parse to `invalid_status` otherwise. No per-status finding behavior.
- No executed checks, findings taxonomy, analysis, migrations, CLI, wasm,
  UI. No `genesis-vibes` anywhere in P0 (kernel crates stay genesis-free;
  it is allowed in `catherine-cli` only, P3, stable modules only).

## 6. Exact files

Create/touch **only**:

- `crates/catherine-core/**` (new):
  `Cargo.toml` (deps: serde, serde_json, proptest — nothing else),
  `src/lib.rs`, `src/model.rs`, `src/parse.rs`, `src/canonical.rs`,
  `src/hash.rs`, `src/error.rs`, `tests/golden_fixtures.rs`,
  `tests/roundtrip.rs`, `tests/golden/*.json` fixtures,
  `schema/ct-schema-v1.json` (published JSON Schema artifact),
  `README.md` (`format_version` policy + upstream attribution).
- `openspec/changes/add-model-document-kernel/**`: tick `tasks.md`
  checkboxes; touch `specs/` only if execution exposes a genuine contract
  gap (say so in the report if so).
- Scaffold exceptions required by task 1 (part of creating the crate in a
  repo with no workspace yet; nothing else outside the two trees): root
  `Cargo.toml` (workspace, single member) and `justfile` (check gate:
  fmt/clippy/test).

## 7. TDD sequence — RED → GREEN → REFACTOR

Run one cycle. State the transition evidence (command + observed failure
mode) in the report at each transition.

### RED — malformed presentations rejected with line-accurate errors

First scaffold so failure is behavioral, never setup breakage: workspace +
`catherine-core` compiles (`cargo build -p catherine-core` exit 0), error
type `{line, column, code, message}` with stable `code` enum
(`duplicate_ident`, `unknown_arrow`, `ill_typed_equation`, `invalid_ident`,
`invalid_status`, `unsupported_version`) exists as the fixture harness's
expected shape. Then write golden fixtures under `tests/golden/` with a
table in `tests/golden_fixtures.rs` pinning, per fixture: file, expected
`code`, expected line/column of the offending JSON node (serde_json span
tracking):

| Fixture | Expectation |
|---|---|
| duplicate object ident | `duplicate_ident` @ second occurrence |
| duplicate arrow ident | `duplicate_ident` |
| equation references undeclared arrow | `unknown_arrow`, naming the path |
| ill-typed equation (path endpoints mismatch, e.g. `a∘b = c`) | `ill_typed_equation`, naming the path |
| ident charset violations (unicode, empty, leading digit) | `invalid_ident` |
| equation `status` ∉ {proposed, accepted} | `invalid_status` |
| `format_version` absent / unknown major | `unsupported_version` @ the node |
| empty schema + empty instance | parses cleanly (no findings, no error) |

- **Narrow**: `cargo test -p catherine-core --test golden_fixtures` — must
  FAIL per-fixture on the expected missing behavior (each test names the
  code/position it expects), not on compile or harness errors.
- **Full**: `cargo test -p catherine-core` — red here is expected; do not
  fix forward.
- Transition when every fixture fails for its named reason.

### GREEN — parse, canonicalize, hash, round-trip

Minimal implementations, adapting donors (do **not** author from scratch):

- Parser (`parse.rs`): serde_json with span tracking for line/column;
  `Schema`/`Instance` as `BTreeMap` keyed by `Ident`; equations with
  rationale + status (default `accepted`); dangling `null` references
  stored as `None` — never dropped, repaired, or rejected at parse time.
- Canonical serialization (`canonical.rs`): adapt specodelic acset-core
  `canonical_order` + `build_deterministic` — sorted enumeration, byte-
  stable across input orders; dense per-object row ids in sorted
  first-reference order. Unit test: two input orderings → identical bytes.
- Hashing (`hash.rs`): port wild `formats.rs` canonical-JSON + sha256 —
  byte-for-byte normative, including wild's own normative tests.
- `tests/roundtrip.rs` (proptest, **never insta-snapshots**):
  `parse(serialize(arbitrary_model)) == model`; plus the row-id
  canonicalization property: same structure under different presentation
  row names → equal canonical bytes and equal hash.
- Publish `schema/ct-schema-v1.json` + document the `format_version`
  policy in the crate README.

**Donor sources (verified paths)**:

- specodelic: `/var/home/sasha/para/areas/dev/gh/charly/specodelic` —
  `src/acset/instance.rs` (typing conventions, dangling preservation),
  `src/acset/schema.rs` (partial — it has **no** equations; path equations
  are new code with new property tests), openspec contracts
  `openspec/changes/archive/2026-10-04-add-acset-core` (+ `acset-writer`
  alongside) as the spec base.
- wild: `/var/home/sasha/para/areas/dev/gh/charly/wild` — `src/formats.rs`
  (canonical-JSON + sha256, byte-for-byte normative).
- `src/acset/query.rs` is a **P1/P4** donor — do not port it now.

- **Narrow**: `cargo test -p catherine-core --test golden_fixtures` green,
  then `cargo test -p catherine-core --test roundtrip` green.
- **Full**: `cargo test -p catherine-core` green.
- Transition when narrow targets are green and no fixture was weakened.

### REFACTOR — structure only, no behavior expansion

- Extract the shared parse-error type into `error.rs`; dedupe the fixture
  harness. Error codes and positions must not change.
- NOTICE attribution: donors ship Apache-2.0 **without** a NOTICE file, so
  each ported file carries an Apache-2.0 header naming the upstream repo +
  source path, and the crate README records the attribution table (task 14).
- **Narrow + full**: re-run both narrow commands and
  `cargo test -p catherine-core` — green, byte-identical behavior.

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

`tasks.md` enumerates exactly **17 tasks** — that is the whole change. Stop
and file follow-up beads tickets instead of expanding when:

- implementation demands new error codes, modules, crates, or dependencies
  beyond serde/serde_json/proptest;
- any P1+ concern (executed checks, findings, semantics of `status`,
  migrations, corpus parity gates) starts leaking into P0;
- the context ceiling in §4 is exceeded, or a completion criterion in §3
  proves impossible as specified;
- a donor contract turns out not to fit and needs redesign rather than
  adaptation.

File the follow-ups with `bd`, note them in the report, and finish only the
17 tasks. Growing the change silently is a failure mode, not diligence.

## Hard constraints (binding)

1. **Kernel crates genesis-free.** No `genesis-vibes` in `catherine-core`
   (or any kernel/wasm crate) — it is allowed in `catherine-cli` only, and
   only stable modules.
2. **Adapt specodelic acset-core contracts; do not author from scratch.**
   Kept (adapted): morphisms typed with exactly one src/tgt; `(source,
   name)` uniqueness; canonical sorted enumeration; byte-stable builds;
   interned dense ids with bidirectional maps; partial morphism storage
   (`None` = unresolved, never dropped). Dropped as specodelic-domain:
   parity/refinement/typing-table predicates.
3. **proptest, never insta-snapshots.** Errors and findings are asserted
   structurally; byte comparison only in wild-port normative hash tests,
   where bytes ARE the contract.
4. **Apache-2.0 + NOTICE attribution preserved** on all ported code (§7
   REFACTOR).
5. **Executed checks are the product (E3)** — but they are P1. P0 ships the
   document format, parser, canonical form, and hash that P1's
   `check(schema, instance) -> Vec<Finding>` consumes.
6. Canonical order is semantic (design Decision 4): `BTreeMap` keyed by
   `Ident`, sorted serialization, dense row ids — equal models hash equal
   regardless of input key order or presentation row names.
