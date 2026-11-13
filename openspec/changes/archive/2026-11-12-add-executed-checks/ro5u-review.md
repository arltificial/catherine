# ro5u review — rule-of-5-universal over the full catherine-0gk diff (§8 gate)

**Work reviewed:** code — the complete uncommitted diff at review time:
`crates/catherine-core/src/{instance.rs,checks.rs,lib.rs}`, `README.md`,
`tests/{check_fixtures.rs,checks_properties.rs,parity_gate.rs,common/mod.rs}`,
`tests/golden/{violation_accepted,dangling_value,typing_violation,proposed_hypothesis,consistent}.json`,
`fixtures/parity/**` (7 fixtures × model+expected),
`openspec/changes/add-executed-checks/{tasks.md,parity-gate.md}`.
**Mode:** `/skill:rule-of-5-universal`, single-model run. **Verification:**
`TYPESAFE_API_KEY` not set → findings `UNVERIFIED`, self-reported
validation in effect, backed by the fresh run outputs below.

```
BANNER: Findings NOT verified by TypeSafe (API unavailable or key absent).
Self-reported validation in effect; treat false-positive estimates as unmeasured.
```

## Fresh test-run output (validation evidence)

```
cargo test -p catherine-core --no-fail-fast
  lib 6 ok · check_fixtures 5 ok · checks_properties 5 ok
  golden_fixtures 2 ok · parity_gate 1 ok · roundtrip 2 ok
just ci          → exit 0 (fmt --check + clippy -D warnings + workspace tests)
openspec validate --strict --all → exit 0, 10 passed / 0 failed
ah check --changes add-executed-checks → exit 1, ONLY 6 no-toml structural
  findings (contracts born at apply time) — the criterion-6 expected state
openspec list → add-executed-checks ✓ Complete (12/12)
```

## Stages

**Stage 1 — DRAFT.** Shape GOOD: two new modules with P0-style
purpose/responsibilities/rationale headers, four RED goldens + parity set
as data, one property target; scope locked to the brief's file list; no
new deps (Cargo.toml untouched), no new `ErrorCode` variants (error.rs
untouched), no query.rs, no scaffold changes. No findings.

**Stage 2 — CORRECTNESS.** Quality EXCELLENT. Verified against the code
and the suites: every declared equation executes on every row of its
domain (256-case proptest oracle equals an independent spec
recomputation); one outcome per violating row with full witnesses
(equation index, row, lhs, rhs); dangling cells recorded and excluded
from violations; typing-violated cells excluded from dangling outcomes
and never stored; status split emitted as distinct kinds; totality on
empty schema/instance; `get`-based walk — no panic path on any total
input (the donor's `intern_of` panic path was deliberately not ported).
New CRITICAL: 0; new issues: 0. **CONVERGED** (procedure continued).

**Stage 3 — CLARITY.** Two findings, both fixed:

- [CLAR-001] MEDIUM — `src/checks.rs` `check` doc: the defensive
  `continue` branches were undocumented; an auditor could misread a
  skipped equation as the E3 failure mode (unevaluated declaration).
  **Fixed:** doc paragraph stating the branches guard only
  programmatically-assembled out-of-shape presentations and skip an
  inapplicable equation, never a row of an applicable one. Doc-only.
- [CLAR-002] MEDIUM — `src/instance.rs` `build`: values keyed by arrows
  outside the schema were silently ignored without saying so.
  **Fixed:** doc sentence added. Doc-only.

**Stage 4 — EDGE CASES.** Two findings:

- [EDGE-001] MEDIUM — `tests/common/mod.rs` loader: a table value naming
  no row of the arrow's target object resolves in "any other object", in
  sorted-object order — a fixture-author-visible subtlety in shared
  test code. **Fixed-by-documentation:** the loader module doc states
  the exact resolution rule; fixtures never rely on ambiguous names (all
  expected sets match exactly). Test-only code.
- [EDGE-002] LOW — `tests/checks_properties.rs` candidate endpoint pairs
  span `min(n_arrows, n_objects)` objects, slightly narrowing equation
  coverage when there are fewer arrows than objects. **Deferred with
  reason:** generator-coverage softness only — the oracle remains sound;
  self-loops and two-hop paths are exercised by the deterministic
  self-loop test and the drawn shapes. No behavior-impacting gap; fixing
  it would widen coverage, not change behavior.

**Stage 5 — EXCELLENCE.**

- [EXCL-001] LOW — the `equation` witness is a `usize` index into
  `Schema::equations` (canonical order) rather than a richer identifier.
  Matches DESIGN §4's sketch, stable across re-parse. **Deferred with
  reason:** witness-taxonomy refinement is P2's `catherine-1x7` (brief
  anti-goal: no P2 taxonomy work in P1).

Excellence assessment: structure EXCELLENT · correctness EXCELLENT ·
clarity GOOD (post-fix) · edge cases GOOD · overall GOOD.
**Production Ready: YES.**

## Convergence checks

```
After Stage 2: new CRITICAL 0 · new issues 0 · Status: CONVERGED
After Stage 3: new CRITICAL 0 · new issues 2 (both fixed) · FP 0% · Status: CONTINUE
After Stage 4: new CRITICAL 0 · new issues 2 (1 fixed, 1 deferred+reason) · Status: CONTINUE
Stage 5: new CRITICAL 0 · new issues 1 (deferred+reason)
```

## Final tally and dispositions

| Severity | Count | Dispositions |
|---|---|---|
| CRITICAL | **0** | — (max_critical = 0 gate: **pass**) |
| HIGH | 0 | — |
| MEDIUM | 3 | all fixed ([CLAR-001], [CLAR-002] docs; [EDGE-001] loader doc) |
| LOW | 2 | deferred with written reasons ([EDGE-002], [EXCL-001]) |

Behavior-impacting fixes: none (all fixes documentation-only), so no
test additions were required by the gate rule; no fixture was weakened
(the full suite, including every golden and the parity gate, is green
after the fixes, fresh output above).

**Verdict: READY** — max_critical = 0, highs none, mediums fixed, lows
deferred with written reasons.
