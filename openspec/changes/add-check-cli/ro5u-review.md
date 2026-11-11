# ro5u review — rule-of-5-universal over the full catherine-86i diff (§8 gate)

**Work reviewed:** code — the complete P3 diff, committed as `b427810`
(`crates/catherine-cli/{Cargo.toml,src/main.rs,src/cli.rs,src/run.rs,src/render.rs,tests/cli.rs}`,
five fixtures under `tests/fixtures/`), plus the workspace member line
(`Cargo.toml`), `Cargo.lock`, README CLI section, CHANGELOG entry, and
`openspec/changes/add-check-cli/tasks.md`; reviewed against the binding
contracts: the implementor brief
(`.wai/projects/catherine/briefs/catherine-86i.md` §1–§10), the spec
delta (`specs/cli/spec.md`), DESIGN §5, and the P2 surface
(`crates/catherine-core/src/findings.rs`, `checks.rs`, `error.rs`).
**Mode:** `/skill:rule-of-5-universal`, single-model run.
**Verification:** `TYPESAFE_API_KEY` not set → findings `UNVERIFIED`,
self-reported validation in effect, backed by the fresh run outputs
below.

```
BANNER: Findings NOT verified by TypeSafe (API unavailable or key absent).
Self-reported validation in effect; treat false-positive estimates as unmeasured.
```

## Fresh test-run output (validation evidence)

```
just ci (TMPDIR redirected: /tmp tmpfs at its per-user quota, 25G/24.8G)
  → CI-EXIT=0 · fmt --check clean · clippy --workspace --all-targets
  -D warnings clean · 59 tests passed / 0 failed
cargo test -p catherine-cli → 17 unit ok · 10 integration ok
cargo test --workspace → kernel suites unregressed: lib 6, check_fixtures 5,
  checks_properties 5, findings_fixtures 7, findings_properties 4,
  golden_fixtures 2, parity_gate 1, roundtrip 2 — catherine-core diff-free
walkthrough (criterion 10), both commands run from the repo root:
  $ cargo run -q -p catherine-cli -- check crates/catherine-cli/tests/fixtures/bowling.ct.json
  violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item
    at row Assignment#3: lhs=Unit#1 rhs=Unit#2
  EXIT=3
  $ cargo run -q -p catherine-cli -- check crates/catherine-cli/tests/fixtures/bowling-fixed.ct.json
  (no output) EXIT=0
  — the §5 violation block is byte-for-byte; the transcript's
  `underconstraint` line is P4-gated (findings.rs emits nothing for it)
  and its absence is the honest state, never fabricated
per-class exits observed end-to-end (in addition to the asserted suite):
  clean (bowling-fixed --json) EXIT=0 · dangling EXIT=4 ·
  multi-class EXIT=3 (payload lists both kinds — asserted) ·
  malformed --json EXIT=1 (error envelope on stderr, code
  duplicate_ident, line 14 column 16, remediation present) ·
  usage (--bogus) EXIT=2 · findings --json on bowling EXIT=3 with
  data.findings structurally equal to check's (asserted)
openspec validate add-check-cli --strict → valid (exit 0)
openspec validate --strict --all    → exit 0
openspec list → add-check-cli ✓ Complete (9/9)
ah check --changes add-check-cli → exit 1, ONLY 5 no-toml structural
  findings (agent-loop-acceptance, ci-branches-without-parsing,
  envelope-shape-matches-suite-tools, malformed-document-is-a-labeled-error,
  multiple-classes-resolve-by-precedence) — the criterion-7 expected
  state; zero substantive findings
cargo tree -p catherine-core -i genesis-vibes → "did not match any
  packages" (kernel genesis-free); cargo tree -p catherine-cli -i
  genesis-vibes → resolves (v0.13.2); genesis-vibes appears in no
  kernel crate's Cargo.toml
```

## Stages

**Stage 1 — DRAFT.** Shape GOOD: four modules with single
responsibilities (`cli.rs` clap wiring + the verbatim exit table,
`run.rs` the one execution path + the pure exit decision,
`render.rs` human renderer + deduplicated envelope construction,
`main.rs` entry + genesis config/suggestions wiring), integration
tests asserting structurally (`serde_json::Value`) and numerically
(exit codes), the walkthrough scripted as the task-7 test, fixtures
engineered from real kernel documents, scope locked to the brief's
file list. No findings.

**Stage 2 — CORRECTNESS.** Quality EXCELLENT. Verified against the code
and the suites: the exit decision is exactly `min` over class codes
`3..=7 = 2 + rank` under the adjudicated precedence (unit-pinned over
directly constructed analysis-kind findings — 5/6/7 — since no P4
producer exists); findings JSON is P2's serialization verbatim (the
envelope serializes `&[Finding]` through the kernel's serde; the
observed payload `{"label":"executed","kind":"violation","witness":{"equation":0,"row":3,"lhs":1,"rhs":2}}`
matches the taxonomy's stable field order); envelope keys
`ok/envelope_kind/data/warnings/hints` present on success envelopes,
error envelopes carry `ok:false`, `kind=error`, stable code, position,
non-empty remediation (genesis Invariant 3.2.5); streams hold in both
modes (findings/stdout, errors/stderr, `--json` error envelope on
stderr — asserted per test); no text is parsed back for control flow
(the single human-text assertion is the mandated walkthrough); usage
errors stay clap's code 2; config problems degrade to warnings instead
of masquerading as model failures, so exit 1 remains parse/build-only;
`check`/`findings` share one execution path (the `Surface` enum exists
for intent and is documented as never-growing divergent semantics).
New CRITICAL: 0; new issues: 0. **CONVERGED** (procedure continued).

**Stage 3 — CLARITY.** One finding, fixed:

- [CLAR-001] LOW — `tests/cli.rs` walkthrough assertion built the
  transcript's two-space witness indent via a `\` string continuation
  plus `\x20`, correct but unobvious at the repo's mandated-contract
  test. **Fixed:** comment at the assertion states the continuation
  semantics and names the plain-literal unit twin
  (`render::tests::violation_renders_the_s5_block`). Comment-only; the
  suite re-ran green after the fix.

**Stage 4 — EDGE CASES.** Two findings:

- [EDGE-001] LOW — the empty-document happy path (empty schema +
  instance → exit 0) has no end-to-end CLI fixture. **Deferred with
  reason:** the mapping is pinned at both ends that own it — the
  kernel's `empty_ok` golden (parse/check totality, "silence after
  evaluation is real success") and the CLI unit test
  `exit_code(&[]) == 0` — and §10 closes the door on extra fixture
  classes beyond the brief's five.
- [EDGE-002] LOW — an unreadable/missing model path exits **1** as a
  labeled `read_error` (with hint), not strictly a *parse* failure.
  **Deferred with reason:** the exit table's bucket-1 intent is
  "labeled error, never a finding" and the alternative (a new exit
  numeral) would drift from the pinned table; the path is documented
  in `render.rs`/`run.rs` headers and was exercised live (observed
  exit 1 + labeled stderr during the walkthrough's first run from a
  non-repo-root cwd).

**Stage 5 — EXCELLENCE.**

- [EXCL-001] LOW — runtime `serde` (derive, for the config struct) and
  `serde_json` (envelope emission) sit beside the brief §6 pinned set.
  **Disposition:** unavoidable, not silent — the envelope contract
  ("print the genesis envelope, never a second serialization") requires
  a JSON serializer, genesis-vibes re-exports none, and every surveyed
  genesis consumer in the suite (whisper, dont, testaruda,
  dulce-de-leche, espectacular) carries `serde_json` beside
  `genesis-vibes`; versions unify with `catherine-core`'s (both "1").
  Flagged here and in the implementor report; `bd` is
  orchestrator-reserved, so the follow-up filing (if the orchestrator
  wants the pin list formally amended) is the lead's.
- [EXCL-002] LOW — README "Status & roadmap" still says
  "implementation not started" (stale since P0–P2 landed). **Deferred
  with reason:** the brief's README mandate is the CLI usage + exit
  table only (scope lock); a docs pass can refresh the status line
  with P4.

Excellence assessment: structure EXCELLENT · correctness EXCELLENT ·
clarity EXCELLENT (post-fix) · edge cases GOOD · overall EXCELLENT.
**Production Ready: YES.**

## Convergence checks

```
After Stage 2: new CRITICAL 0 · new issues 0 · Status: CONVERGED
After Stage 3: new CRITICAL 0 · new issues 1 (fixed) · Status: CONTINUE
After Stage 4: new CRITICAL 0 · new issues 2 (both deferred with written reasons) · Status: CONTINUE
Stage 5: new CRITICAL 0 · new issues 2 (both deferred with written reasons)
```

## Final tally and dispositions

| Severity | Count | Dispositions |
|---|---|---|
| CRITICAL | **0** | — (max_critical = 0 gate: **pass**) |
| HIGH | 0 | — |
| MEDIUM | 0 | — |
| LOW | 5 | 1 fixed ([CLAR-001] comment), 4 deferred with written reasons ([EDGE-001], [EDGE-002], [EXCL-001], [EXCL-002]) |

No behavior-impacting fixes were required (the single fix is
comment-only); the full suite is green fresh on the final tree (output
above), with every kernel fixture untouched and `catherine-core`
diff-free.

**Verdict: READY** — max_critical = 0, no highs or mediums, lows
deferred with written reasons. (Orchestrator note: single-model run per
the brief §4 — the recorded verification verdict for the ticket remains
REVIEW_REQUIRED with lead-side checks; criterion 10's paste above is
reproducible by re-running the two walkthrough commands from the repo
root.)
