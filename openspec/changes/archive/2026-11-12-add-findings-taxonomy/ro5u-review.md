# ro5u review — rule-of-5-universal over the full catherine-1x7 diff (§8 gate)

**Work reviewed:** code — the complete uncommitted diff at review time:
`crates/catherine-core/src/{findings.rs (new),checks.rs,lib.rs,model.rs}`,
`tests/{findings_fixtures.rs (new),findings_properties.rs (new),check_fixtures.rs,checks_properties.rs,parity_gate.rs}`,
`tests/golden/{redundant_shape,contradiction_shape,underconstraint_shape}.json (new)`,
`openspec/changes/add-findings-taxonomy/{tasks.md,ts-types-spike.md}`.
**Mode:** `/skill:rule-of-5-universal`, single-model run. **Verification:**
`TYPESAFE_API_KEY` not set → findings `UNVERIFIED`, self-reported
validation in effect, backed by the fresh run outputs below.

```
BANNER: Findings NOT verified by TypeSafe (API unavailable or key absent).
Self-reported validation in effect; treat false-positive estimates as unmeasured.
```

## Fresh test-run output (validation evidence)

```
cargo test -p catherine-core        → TEST-EXIT=0
  lib 6 ok · check_fixtures 5 ok · checks_properties 5 ok
  findings_fixtures 7 ok · findings_properties 4 ok (256-case proptests)
  golden_fixtures 2 ok · parity_gate 1 ok · roundtrip 2 ok · doc 0 ok
just ci                             → CI-EXIT=0 (fmt --check clean,
  clippy --workspace --all-targets -D warnings clean, workspace tests green)
openspec validate add-findings-taxonomy --strict → valid (exit 0)
openspec validate --strict --all    → 10 passed / 0 failed (exit 0)
openspec list → add-findings-taxonomy ✓ Complete (9/9)
ah check --changes add-findings-taxonomy → exit 1, ONLY 4 no-toml
  structural findings (every-finding-kind-has-a-golden-fixture,
  label-matches-producer, repeated-runs-are-diff-stable,
  violation-finding-carries-row-and-values) — the criterion-6 expected
  state; zero substantive findings
```

## Stages

**Stage 1 — DRAFT.** Shape GOOD: one new module carrying the whole
taxonomy (`Finding` = label + kind; five witness-carrying kinds; three
constructible labels), `check` evolved to `Vec<Finding>` (no wrapper, no
parallel enum — `CheckOutcome` deleted), two new test targets (structural
fixtures + 256-case proptests), P1 targets migrated, three new golden
documents for the constructed shapes, scope locked to the brief's file
list, no new deps, no new `ErrorCode` variants, no query.rs, no
snapshot files. No findings.

**Stage 2 — CORRECTNESS.** Quality EXCELLENT. Verified against the code
and the suites: invalid states unrepresentable (every kind variant
carries its witness struct; no generic-witness constructor); labels are
exactly `executed`/`hypothesized`/`unknown` — `proved`/`illustrated`
absent from the type; the P1 `Hypothesis` kind re-shaped as a violation
with the `hypothesized` label (same witness, countable apart by label —
pinned by `proposed_failure_is_violation_shaped_and_hypothesized` on both
suites); the oracle proptest recomputes findings independently (labels
mirroring equation status) and `check` output sorts under the total
order; the parity gate's expected sets are unchanged on disk
(`fixtures/parity/**` untouched) and re-sorted under the P2 order before
comparing — same facts, stricter than set equality; round-trip and
two-runs-byte-identical hold per fixture and per 256-case property;
`check` remains panic-free on total input (`get`-based walk unchanged).
New CRITICAL: 0; new issues: 0. **CONVERGED** (procedure continued).

**Stage 3 — CLARITY.** One finding, fixed:

- [CLAR-001] MEDIUM — `src/findings.rs` `FindingKind::Violation` doc was
  garbled ("`executed` for accepted equations, `executed` vs
  `hypothesized` decided by…"): redundant and self-contradictory phrasing
  on the taxonomy's central kind. **Fixed:** rewritten — the kind names
  the shape, the label names the weight (`executed` accepted /
  `hypothesized` proposed). Doc-only.

**Stage 4 — EDGE CASES.** Two findings:

- [EDGE-001] MEDIUM — `src/findings.rs` `UnderconstraintWitness`
  declared `question` before `case`, so the free-form question text was
  both the JSON's first witness field and the leading order key inside
  the kind — contrary to the order rule (kind, then witness *ids*) and
  to the spec naming the tagged union as the witness with the question
  text appended. The JSON field order is a promise (P3 prints it
  verbatim), so it had to be pinned correctly before P3 exists.
  **Fixed:** fields reordered (`case` flattened first, `question` last);
  module doc + TS spike note updated; discriminating structural
  assertion added (`underconstraint_shape_pins_both_witness_arms`:
  `parallel_paths`/`"zzz"` sorts before `uncovered_inputs`/`"aaa"` —
  fails under the old order).
- [EDGE-002] LOW — `tests/parity_gate.rs` re-sorts the vendored expected
  outcome lists under the P2 order before comparing; a reviewer could
  misread the on-disk P1-era order as still the contract.
  **Fixed-by-documentation:** comment at the sort site states the
  expected files record P1-era facts, re-sorted under the findings
  taxonomy's total order; `fixtures/parity/**` untouched (facts are the
  contract, not their historical order). Test-only code.

**Stage 5 — EXCELLENCE.**

- [EXCL-001] LOW — the parse→build→check pipeline helper `check_parsed`
  is duplicated between `tests/check_fixtures.rs` and
  `tests/findings_fixtures.rs`. **Deferred with reason:** P1's
  established pattern is per-target local helpers; extracting a shared
  loader into `tests/common/mod.rs` would churn a P1 file beyond the
  reshape's needs for a four-line helper. (Advisory per
  test-abstraction-miner; no behavior impact.)
- [EXCL-002] LOW — the crate README still frames the findings taxonomy
  as downstream future work ("Everything downstream (P2 findings
  taxonomy, …) consumes the artifacts this crate publishes"). **Deferred
  with reason:** the README is not in the brief's exact-files list
  (scope lock); its statements remain true of the pipeline and the
  taxonomy's landing is recorded in `lib.rs` docs, the openspec change,
  and this review. A docs pass can update it with P3.

Excellence assessment: structure EXCELLENT · correctness EXCELLENT ·
clarity EXCELLENT (post-fix) · edge cases GOOD · overall EXCELLENT.
**Production Ready: YES.**

## Convergence checks

```
After Stage 2: new CRITICAL 0 · new issues 0 · Status: CONVERGED
After Stage 3: new CRITICAL 0 · new issues 1 (fixed) · Status: CONTINUE
After Stage 4: new CRITICAL 0 · new issues 2 (1 fixed, 1 fixed-by-doc) · Status: CONTINUE
Stage 5: new CRITICAL 0 · new issues 2 (both deferred with written reasons)
```

## Final tally and dispositions

| Severity | Count | Dispositions |
|---|---|---|
| CRITICAL | **0** | — (max_critical = 0 gate: **pass**) |
| HIGH | 0 | — |
| MEDIUM | 3 | all fixed ([CLAR-001] doc; [EDGE-001] field reorder + new test; [EDGE-002] loader-site doc) |
| LOW | 2 | deferred with written reasons ([EXCL-001], [EXCL-002]) |

Behavior-impacting fix: [EDGE-001] (changes the serialized witness field
order and the within-kind order key) — gate rule satisfied by the added
discriminating assertion in `tests/findings_fixtures.rs` plus the
arbitrary-witness proptests (`every_kind_round_trips_and_serializes_deterministically`,
`label_is_only_a_tiebreak`) re-run green after the fix; no fixture was
weakened (the full suite is green fresh, output above).

**Verdict: READY** — max_critical = 0, highs none, mediums fixed, lows
deferred with written reasons. (Orchestrator note: single-model run per
the brief §4 — the recorded verification verdict for the ticket remains
REVIEW_REQUIRED with lead-side checks.)
