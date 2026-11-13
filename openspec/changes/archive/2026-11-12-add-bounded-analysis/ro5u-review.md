# ro5u review — rule-of-5-universal over the full catherine-1ay diff (§8 gate)

**Work reviewed:** code — the complete P4 diff: new
`crates/catherine-analysis/` (`Cargo.toml`, `src/{lib,bounds,search,
rewrite,parallel,query,distinguish}.rs`, `README.md`,
`tests/{analysis_fixtures,query_parity}.rs`), the workspace member line
(`Cargo.toml`, `Cargo.lock`), the task-11 CLI touches
(`crates/catherine-cli/{Cargo.toml,src/cli.rs,src/main.rs,src/run.rs,
src/render.rs,tests/cli.rs}` + six analysis fixtures under
`tests/fixtures/`), and `openspec/changes/add-bounded-analysis/tasks.md`
— reviewed against the binding contracts: the implementor brief
(`.wai/projects/catherine/briefs/catherine-1ay.md` §1–§10, hard
constraints 1–10), the spec delta (`specs/analysis/spec.md`, 4
requirements / 6 scenarios), DESIGN §4.2, and the P2 surface
(`crates/catherine-core/src/findings.rs` — five-kind closed taxonomy,
total order, label docs).
**Mode:** `/skill:rule-of-5-universal`, single-model run.
**Verification:** `TYPESAFE_API_KEY` is set; `VERIFY_SEVERITIES` =
CRITICAL, HIGH — the review found **zero findings at or above those
severities**, so per the verification pass's cost-cap rule no API
request was issued and nothing was verified or dropped; all MEDIUM/LOW
findings below carry `UNVERIFIED` by rule (excluded from FP-rate
arithmetic). Findings were confirmed against the code and the fresh run
outputs below (no "potential" issues flagged without evidence).

## Fresh test-run output (validation evidence)

```
TMPDIR=$PWD/target/tmp just ci
  → CI-EXIT=0 · cargo fmt --check clean · clippy --workspace
  --all-targets -D warnings clean · workspace suites green
cargo test --workspace → PASSED=95 FAILED=0, by suite:
  catherine-analysis lib 0 · analysis_fixtures 12 · query_parity 9 ·
  catherine-cli unit 18 (incl. exit_code_table_is_verbatim_unchanged) ·
  cli integration 24 (10 P3 + 14 P4) · kernel P0–P3 unregressed:
  lib 6, check_fixtures 5, checks_properties 5, findings_fixtures 7,
  findings_properties 4, golden_fixtures 2, parity_gate 1, roundtrip 2
git diff -- crates/catherine-core/ → empty (kernel diff-free)
RED evidence (recorded once, before any GREEN existed):
  cargo test -p catherine-analysis → exit 101; E0432 unresolved imports:
  `could not find search/rewrite/parallel/distinguish/query in
  catherine_analysis` — the outcome type with its `unknown { scope }`
  variant does not exist (UNKNOWN not representable) and no op accepts
  bounds; the fixtures cannot compile
scope-honesty cross-checks on the real binary (anti_goals live here):
  $ catherine search analysis_contradiction.ct.json --json --rows a=1
    --rows b=0 --rows c=1 --max-candidates 2
  {"data":{"outcome":"unknown","bound":{...,"max_candidates":2},
   "scope":{"rows":{"a":1,"c":1},"candidates_explored":2},
   "findings":[]}} EXIT=0 — budget smaller than the space: UNKNOWN with
  the exact explored scope, findings empty, never "no contradiction
  found"
  $ same, --max-candidates 64 → {"data":{"outcome":"contradiction",
   "findings":[{"label":"executed","kind":"contradiction",
   "witness":{"scope":[0,1]}}]}} EXIT=5 — contradiction only after the
  enumeration wrapped (real exhaustion), bound restated
  $ same, --rows a=0 --rows b=0 --rows c=0 → {"data":{"outcome":
   "unknown","scope":{"rows":{},"candidates_explored":0}}} EXIT=0 —
  zero-size bound: UNKNOWN with the empty scope, never a contradiction
  $ catherine underconstraint analysis_underconstraint.ct.json →
  "underconstraint: parallel paths f∘g / h (hypothesized)\n  should
  these agree?" EXIT=6 — label intact, question raised as a hypothesis
  $ catherine redundant analysis_redundant.ct.json → "redundant:
  f∘g∘k\n  derivable as n" EXIT=7 — proven only by rewriting
openspec validate add-bounded-analysis --strict → valid (exit 0)
openspec validate --strict --all    → exit 0
openspec list → add-bounded-analysis ✓ Complete (13/13)
ah check --changes add-bounded-analysis → ONLY 5 no-toml structural
  findings (budget-exhaustion-is-unknown-not-success,
  contradiction-proven-within-bounds, derivable-path,
  parallel-paths-without-equation, witnessed-difference) — the
  criterion-7 expected state; zero substantive findings
```

## Stages

**Stage 1 — DRAFT.** Shape GOOD: six modules with single
responsibilities (`bounds.rs` the shared bound/scope types + caps +
deduped witness builders, `search.rs` bounded enumeration,
`rewrite.rs` oriented normal-form rewriting, `parallel.rs` endpoint-
grouped path pairs, `query.rs` the attributed donor port,
`distinguish.rs` the observation-set op), README carrying the
decidability boundary + parity-gate documentation, CLI surface exactly
one subcommand per op with the verbatim exit table, integration tests
asserting structurally (`serde_json::Value`) and numerically (exit
codes), scope locked to the brief's file list. No findings.

**Stage 2 — CORRECTNESS.** Quality GOOD after fixes; two real defects
found and fixed, each with a test:
- [CORR-001] MEDIUM — `src/parallel.rs`
  (`underconstraint`, level-1 seeding). The length-1 paths were
  inserted before the cap check, so `--max-path-length 0` still
  enumerated single arrows — a cap that did not bound the op from the
  start. Evidence: the seeding block ran unconditionally above the
  extension loop. Fix: gate the level-1 insertion on
  `max_path_length >= 1`; test `task3_zero_cap_enumerates_no_paths`
  asserts cap 0 → no findings. A cap hit is a bounded non-answer, never
  a quiet truncation.
- [CORR-002] MEDIUM — fixtures (`analysis_fixtures.rs`, two CLI
  `.ct.json` fixtures). The equation paths were originally spelled in
  diagram order (`["g","f"]` for f-then-g); the parser type-checks
  paths in *application order* (lead arrow first — the bowling
  fixture's convention), so the CLI end-to-end run rejected them
  `ill_typed_equation` (observed: `equation paths [g, f] (b -> b) and
  [h] (a -> c) have mismatched endpoints`). Fix: all equation paths
  respelled in application order (`["f","g"]`, `["h","k"]`,
  `["f","g","k"]`) with assertions updated; the convention is now
  documented in the rewrite-fixture comment. Caught by the CLI suite,
  never papered over.
Verified against the code and suites (confirmed, not presumed): the
budget check precedes each enumeration step and the mixed-radix wrap is
the only exhaustion signal (so a contradiction is never emitted from a
partial enumeration); a candidate witnesses iff `checks::check` on it
emits nothing; only accepted equations constrain the search and anchor
the contradiction scope; `related()` accepts either orientation and any
declared status; redundancy requires a completed derivation on both
sides (a rewrite-cap hit emits nothing); the parity properties compare
the port against an independent BFS on generated instances.
No CRITICAL, no HIGH.

**Stage 3 — CLARITY.** Quality EXCELLENT. Every file carries the
Purpose/Responsibilities/Rationale header; the honesty contract is
stated in `lib.rs`, per-module docs, the README, and the CLI help; the
seed-validation split (CLI owns `object#row` syntax, the op owns
`seeds_exist`) is documented at both sites; the witness builders'
label adjudication (`executed` for the two proven results,
`hypothesized` for the question) is written where it is implemented.
One LOW note, no change needed: `search.rs`'s outcome names
(`witness`/`contradiction`/`unknown`) are the payload's epistemic
states, spelled out in the `SearchData` doc.

**Stage 4 — EDGE CASES.** Coverage GOOD after fixes.
- [EDGE-001] MEDIUM — same defect as CORR-001 (zero cap), fixed there
  with its test.
- [EDGE-002] MEDIUM — budget 0 and zero-size bounds had no direct
  assertions at the engine level. Fix: `task2_zero_budget_
  yields_unknown_before_any_enumeration` (budget 0 on a *satisfiable*
  schema → UNKNOWN, zero explored — no witness smuggled from an
  enumeration that never ran) and the existing
  `task2_zero_size_bound_yields_unknown_with_the_empty_scope`
  (empty scope asserted); both also asserted end-to-end in the CLI
  suite.
- [EDGE-003] LOW — a bounds-forced contradiction with an empty accepted
  set yields `ContradictionWitness { scope: [] }` (honest: no fully
  resolved instance exists within the bounds; the enforced set is
  empty). Deferred with reason: reachable only via programmatically
  assembled schemas (parse-gated documents always carry well-typed
  equations), the finding text stays truthful, and inventing scope
  content would be dishonest. Documented by the witness contract in the
  README.
- [EDGE-004] LOW — `distinguish` against a second model with shorter
  row spaces resolves to `None` on the missing rows: an honest
  difference (`left=Some right=None`), never an input error, documented
  in `parse_observations`. Accepted as designed; the first instance's
  space gates observation validity.
Convergence after Stage 4: no new CRITICAL, new-issue rate below
threshold → CONVERGED (Stage 5 run for polish).

**Stage 5 — EXCELLENCE.** Final polish notes, all addressed in-review:
formatters/clippy clean (`just ci` exit 0), the exit table pinned
byte-for-byte by a unit test, determinism asserted by comparing two
fresh runs (no snapshots anywhere), README tables documenting the caps
and the parity mapping. Excellence Assessment: Structure EXCELLENT ·
Correctness EXCELLENT · Clarity EXCELLENT · Edge Cases EXCELLENT ·
Overall EXCELLENT. Production Ready: YES.

## Findings and dispositions (gate summary)

Total issues by severity:
- CRITICAL: 0 — **max_critical = 0** ✓
- HIGH: 0
- MEDIUM: 3 — CORR-001, CORR-002, EDGE-002: all fixed, each
  behavior-impacting fix landed with a structural test (never
  snapshots); EDGE-001 is CORR-001's edge-case restatement, fixed with
  it.
- LOW: 4 — CLAR-001 (no change needed), EDGE-003, EDGE-004 (deferred
  with written reasons above), EXCL-notes (addressed in-review).

Every behavior-impacting fix added or updated a test:
`task3_zero_cap_enumerates_no_paths`,
`task2_zero_budget_yields_unknown_before_any_enumeration`, and the
respelled fixtures pin the kernel's application-order path convention
through the five fixture tests + 14 CLI integration assertions.

## Convergence

```
Stage 2: New CRITICAL issues: 0 · Total new issues: 2 · CONTINUE
Stage 3: New CRITICAL issues: 0 · Total new issues: 1 (LOW) · CONVERGED
Stage 4: New CRITICAL issues: 0 · Total new issues: 4 (2 fixed, 2 LOW
         deferred with reasons) · CONVERGED (Stage 5 run for polish)
False positive rate: unmeasured (no findings at VERIFY_SEVERITIES; the
         cost-cap rule excludes MEDIUM/LOW from FP arithmetic; the two
         fixed MEDIUM defects were confirmed by execution —
         `ill_typed_equation` output and cap-0 behavior — not estimates)
Status: CONVERGED
```

## Verdict

**REVIEW_REQUIRED** (single-model run — no cross-family verifier is
configured; per the brief's §9 degradation this artifact never records
CONFIRM). The work itself is complete against the brief's criteria 1–9:
`just ci` exit 0 (TMPDIR quirk applied), `cargo test -p
catherine-analysis` and `cargo test --workspace` green (95 passed / 0
failed, P0–P3 unregressed), `openspec validate --strict` exit 0,
`ah check` reporting only the 5 expected no-toml findings, parity gate
documented, tasks.md 13/13 (`openspec list` ✓ Complete), `bd close
catherine-1ay` pending orchestrator. The lead-side verification
(state checkpoint, `git diff` vs the report, fresh suite run) remains
the orchestrator's step.
