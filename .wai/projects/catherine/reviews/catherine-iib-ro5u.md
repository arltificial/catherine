# ro5u review — catherine-iib (coproduct exhaustiveness / `add-coproduct-exhaustiveness`)

Change: `add-coproduct-exhaustiveness` (ticket `catherine-iib`). Base `7aba364`,
worktree uncommitted (the test/fixture/contract half was committed inside
`0f4f741`/`9f315fe`; the kernel half and the GREEN ledger entries are in the
worktree). Contract read: `openspec/changes/add-coproduct-exhaustiveness/{proposal,design,tasks}.md`
+ all three spec deltas, the deployed `openspec/specs/{model-document,findings,analysis,kernel-checks}.md`,
the archived `add-declarative-keys` deltas (integration precedent), the `coproduct-absence`
row in `docs/ref/expressibility-dispositions.md`, and `docs/ro5u-checklist.md`.

Gate: **max_critical = 0**. Critical/high fixed before commit; medium fixed or
deferred with a written reason; low deferred only with a reason. All findings
below were reproduced either by a command in the table or by reading the cited
line; none is speculative.

## Scope reviewed

- Core: `model/{mod,relations}.rs`, `parse/{coproducts,rows,schema,mod,build,schema_map}.rs`,
  `error.rs`, `findings.rs`, `checks.rs`, `canonical/value.rs`,
  `composition{,.resolve,/instance}.rs`, `schema/ct-schema-v1.json`
- Analysis: `schema_diff.rs`, `search.rs` (satisfaction predicate)
- CLI/web: `render.rs`, `render/{diff,errors,rules,tests}.rs`, `composition.rs`,
  `web/src/{findings.ts,types/ct-schema-v1.ts}`, `web/test/**`
- Tests/fixtures: `crates/catherine-core/tests/coproducts.rs`,
  `crates/catherine-cli/tests/{coproducts,negative_matrix_coproduct}.rs`,
  `crates/catherine-analysis/tests/schema_diff_coproducts.rs`,
  `crates/catherine-cli/tests/fixtures/coproduct/**`, and every modified
  builder/property/fixture file
- Probe/ledger: `crates/catherine-ledger/src/catalog.rs`,
  `ledger/probes/coproduct/{0001,0002,0003}-*.json`, `fixtures/probes/coproduct/**`
- Specs/docs: the three deltas, the `.espectacular` contracts for this change,
  `docs/ref/expressibility-dispositions.md`, README/CHANGELOG/USING hunks

Not reviewed as iib: `.beads/`, `.whisper/`, the unrelated lint cleanups in
modified test files (R-04, informational).

## Commands executed (read-only; scratch files only in `/tmp`; only this artifact written)

| # | Command | Result |
|---|---|---|
| 1 | `cargo test -p catherine-core --test coproducts` | **10 passed**, 0 failed |
| 2 | `cargo test -p catherine-cli --test coproducts --test negative_matrix_coproduct` | **6 + 1 passed**, 0 failed |
| 3 | `cargo test -p catherine-analysis --test schema_diff_coproducts` | **3 passed**, 0 failed |
| 4 | `cargo test -p catherine-analysis --test schema_diff_keys` | **4 passed**, 0 failed |
| 5 | `cargo test -p catherine-analysis --test schema_diff` | **3 passed**, 0 failed |
| 6 | `cargo test -p catherine-cli --test cli_diff` | **5 passed**, 0 failed |
| 7 | `cargo test -p catherine-core --test checks_properties --test findings_properties` | **5 + 4 passed**, 0 failed |
| 8 | `cargo test -p catherine-core --test composition --test keys --test composition_agreement` | **10 + 7 + 10 passed**, 0 failed |
| 9 | `cargo test -p catherine-core --test editor_validation` | **4 passed** (shipped JSON Schema compiles under draft 2020-12) |
| 10 | `cargo test -p catherine-cli --test expressibility_gaps` | **6 passed** (closed-row direction includes the flipped `coproduct-absence` row) |
| 11 | `cd web && npx vitest run` | **6 passed** (2 files) |
| 12 | `cd web && npx tsc --noEmit` | exit 0 |
| 13 | `openspec validate add-coproduct-exhaustiveness --strict` | valid |
| 14 | `openspec validate --strict --all` | **16 passed, 0 failed** (includes `change/add-coproduct-exhaustiveness`) |
| 15 | `./target/debug/ledger-check ledger/probes/coproduct` | exit 0 — `ok: 3 entries (2 kata), rollup verdict: (no rollup)` |
| 16 | `node` generator run in a `/tmp` copy of `web/scripts/generate-types.mjs` + the schema, then `diff` against the checked-in type | **TS-CONSISTENT** (byte-identical) |
| 17 | `printf '%s' '<canonical bytes>' \| sha256sum` for `fixtures/parity/small/model.ct.json` | `fb033e3c…` = `SMALL_PRE_CHANGE_HASH`; constant also pinned by `instance_morphism.rs`, `rule_lens.rs`, `schema_maps.rs` |
| 18 | `/tmp/hashprobe2` (built against `libcatherine_core-af5986…rlib`, calls `parse` + `canonical_bytes` + `model_hash`) over the four probe fixtures | green hashes `ede09429…`(clean), `999d9c5e…`(rogue), `c953da59…`(dual), `0517667d…`(floor) — exactly seq 3's `model_hashes` and seq 2's floor hash; dropping the section yields `e7a06c0b…`, `2e10d916…`, `1a55c46f…`, `76dbedc2…` — exactly the pre-registered RED hashes |
| 19 | same probe over `fixtures/probes/naturality/{crm,warehouse_break}.ct.json` | `9bb77131…`, `ca16f215…` — exactly the sealed naturality ledger hashes (independent hash-neutrality regression) |
| 20 | `/tmp/verdictprobe` calls `harness::derive_probe_verdict("probe-coproduct", …)` | `empty-unique → Some(Killed)`, `non-empty + executed → Some(Survived)` — matches seq 2 / seq 3 |
| 21 | `/tmp/catalogcmp` calls `catalog::probe_expectations("probe-coproduct")` + `probe_catalog_hash` | 4 rows equal the README catalog block verbatim (whitespace-collapsed); hash `sha256:a4b0451c…` = the entries' `catalog_hash` |
| 22 | `catherine check` on `payments-{rogue,dual,three-variants}` + `floor-keys-caught` | findings identical to seq 3 (`{coproduct:0,row:3}`, `{coproduct:0,row:2,injections:[card_inj,transfer_inj]}`, `[]`, `{key:0,rows:[0,1]}`; exits 3/3/0/3) |
| 23 | `catherine check crates/catherine-cli/tests/fixtures/gap_suite/coproduct_payments.ct.json --json` | exit 3, `[{coproduct:0,row:2,…dual…},{coproduct:0,row:4}]` — matches the flipped disposition row |
| 24 | `catherine diff` probes: no-coproduct → one-coproduct, and reverse | `accreted` with `added:[{object:"C",arrows:["i1","i2"],kind:"coproduct"}]`; reverse `weakened`, exit 3, `{coproduct:0,row:0,injections:["i1","i2"]}`, witness v2-clean/v1-rejected |
| 25 | `catherine diff` duplicate-declaration probes (one copy ↔ two copies) | `accreted` (adding the copy) / `equivalent-in-scope` (removing it) — C-04 |
| 26 | `catherine check` scratch docs `/tmp/{selfloop,double,suppress,noarrows,targetbad,targetbadkey}.ct.json` | self-loop injection → `invalid_coproduct` at 10:47; duplicate declaration → two findings (`coproduct` 0 and 1); wholesale suppression → only the dangling finding, exit 4; missing `arrows` → `invalid_coproduct` at **1:1**; target-schema coproduct/key → `invalid_coproduct`/`invalid_key` at **1:1** |
| 27 | `catherine check /tmp/comp-a2.ct.json /tmp/comp-b2.ct.json` (joint form) | exit 3, `{coproduct:0,row:1}` with `source:[a,b]`; human block renders row + `uncovered` + source |
| 28 | `catherine check …/proposed-voucher.ct.json --human` | `violation: coproduct-0 Method(card_inj∘transfer_inj∘cash_inj) (hypothesized)`, row 3 `uncovered`, exit 3 |

## Pass 1 — Correctness

Runtime verification (commands 22–28) found **no wrong semantics**. The coverage/
disjointness image construction (`checks.rs:268-346`), status→label rule
(`checks.rs:169-178`), the sorted finding order (`findings.rs:280-292` +
`CoproductWitness` field order), the canonical writer (`canonical/value.rs:124-199`),
the parser's injection shape (`parse/coproducts.rs:140-170`), the schema-diff
enforced set (`schema_diff.rs:221-262`), and the probe arithmetic all behave as
the deltas say. Four defects remain:

- **C-01 [LOW — defer]** — `parse/coproducts.rs:50` hardcodes the pointer root
  `/schema/coproducts/<i>`; `schema_map::validated_target`
  (`parse/schema_map.rs:29-47`) validates the *target* schema through the same
  passes, so a malformed coproduct inside `schema_map.target_schema` reports
  `invalid_coproduct … (line 1, column 1)` (probe `/tmp/targetbad.ct.json`).
  Pre-existing pointer-root flaw shared with `invalid_key`
  (`/tmp/targetbadkey.ct.json` also reports 1:1); iib inherits it. Deferred:
  fixing it means threading a pointer root through every schema pass (a
  separate refactor), and the failure is still labeled and non-silent.
- **C-02 [LOW — fix]** — `parse/coproducts.rs:33-35` adds
  `#[serde(default)]` to `RawCoproduct.arrows`, unlike `RawKey.arrows`
  (`parse/keys.rs:26-27`). A declaration omitting `arrows` therefore becomes
  `invalid_coproduct` at **1:1** (`spans.semantic` fallback,
  `parse/spans.rs:34-41`) instead of keys' positioned serde syntax error
  (`missing field \`arrows\` (line 7, column 28)`). Fix: drop the `default` (the
  JSON Schema already requires the field) or point the empty-list error at
  `base` when `/arrows` has no span.
- **C-03 [MEDIUM — defer to a superseding spec change]** — suppression is per
  declaration, not per image/row: `checks.rs:301-321` returns `None` for the
  whole declaration if *any* injection cell is `None`. Probe
  `/tmp/suppress.ct.json` (i1 has a dangling cell; i2 and i3 both cover `c1`;
  `c2` uncovered): output is only `{dangling,i1,src:1}` and **exit 4**, so a
  definitely-decidable disjointness (i2∩i3) and a definite uncovered row are
  hidden and the exit class drops from 3 to 4. This is exactly what the
  kernel-checks delta mandates ("its coproduct findings SHALL be suppressed")
  and what `crates/catherine-core/tests/coproducts.rs:317-340` pins, so it is
  **not drift**; the rationale "the partial image cannot decide them" is only
  true for rows that could be the unresolved cell's value. Deferred with
  reason: changing it needs a superseding kernel-checks/coproducts spec change
  (per-image or per-affected-row suppression).
- **C-04 [LOW — defer]** — duplicate declarations are presentation in bytes but
  have per-copy semantics elsewhere: `canonical/value.rs:156` keeps both copies
  (correct), `checks.rs:268-276` emits one finding per copy (probe: one rogue
  row → `coproduct` 0 and 1), and `schema_diff.rs:221-258` counts copies in the
  enforced multiset (probe command 25: adding a duplicate classifies
  `accreted`; removing it classifies `equivalent-in-scope`, never `cosmetic`).
  This is the inherited equations/keys multiset design, and the model-document
  delta only says a repeated declaration "is presentation and is kept".
  Deferred: changing the enforced-set shape affects equations and keys too;
  duplicate declarations are pathological and explicitly allowed.

## Pass 2 — Contract drift

- **D-01 [HIGH — blocking, fix]** — **no `findings` spec delta.** The deployed
  requirement `openspec/specs/findings/spec.md:25-36` says *"Each kind's witness
  shape SHALL be fixed: a violation carries the equation witness … or the key
  witness …"* and the keys change's own delta
  (`openspec/changes/archive/2026-11-16-add-declarative-keys/specs/findings/spec.md`)
  rewrote it to *"A violation's witness SHALL be one of: …"*. iib adds a third
  serialized arm (`findings.rs:122-137`, `ViolationWitness::Coproduct` at
  `findings.rs:165`, encoding `{"coproduct":n,"row":r}` /
  `{"coproduct":n,"row":r,"injections":[i…]}`), and design Decision 3 says
  "the keys change's integration shape, reused verbatim" — but no findings
  delta was authored, and `proposal.md:70-74`'s "findings untouched" only holds
  for the *kind set*, not the witness-shape requirement. After archive the
  published requirement will forbid every coproduct violation the shipped code
  emits. `openspec validate --strict` cannot see an omission (command 13).
  Minimal fix: add `openspec/changes/add-coproduct-exhaustiveness/specs/findings/spec.md`
  with one MODIFIED requirement rebased on the deployed keys text — add the
  coproduct witness (coproduct index + row + covering injections, empty/omitted
  for coverage) and its encoding, plus one scenario ("Coproduct violation
  carries its row and injections"). No code change.
- **D-02 [MEDIUM — fix]** — `docs/USING.md:82` documents `violation` as
  "declared equation … or declared key …" only; `docs/USING.md:206-212` still
  says the step-3 constructions ("pullbacks …, monomorphisms …,
  coproducts for exhaustive alternatives") "are advisory only in v1 … Declaring
  a cone in this version checks nothing" — false for `schema.coproducts` since
  this change. `tasks.md:66-69` (REFACTOR 13) explicitly required a
  README/USING note and Decision 4's guardrail (no pullbacks/cones/defaults)
  is documented nowhere. Fix: extend the findings row and narrow the advisory
  bullet to cones/pullbacks, adding one sentence that a failed coverage claim
  is a violation, never a synthesized default row.
- **D-03 [LOW — fix]** — `crates/catherine-core/README.md:112-116`'s stable-code
  table omits `invalid_coproduct` (the keys precedent added `invalid_key` at
  :116), and :129-131's "Check order is the document order: version → objects →
  arrows → equations → keys → rows → tables → schema map → rule presentation"
  omits coproducts although `parse/mod.rs:69` cites this README as the authority.
- **D-04 [LOW — fix]** — `parse/mod.rs:104-110`: the step list numbers two
  entries "8." ("instance rows" and "instance tables") and leaves 9–12 stale
  after inserting coproducts as 7.
- **D-05 [LOW — fix]** — `fixtures/probes/coproduct/README.md:27-29,170,178-186`
  is frozen at the RED state and now contradicts the ledger: "Ledger entry … the
  only entry at RED" (three entries exist), "Probe entry | deferred: appended by
  GREEN task 11", "Kernel state | no executable `schema.coproducts`", "Probe
  entry — deferred to GREEN task 11". Fix prose only (Status/Recorded state →
  seq 3, the two unique detections, `survived`); the catalog rows and fixtures
  must not move.
- **D-06 [LOW — defer]** — the deployed shared requirement
  `openspec/specs/model-document/spec.md:31-60` ("Malformed presentations …")
  still enumerates "… and invalid keys" but not coproducts; the keys change
  modified this requirement, iib instead puts the malformed scenario in its own
  ADDED requirement. Behavior is fully specified either way; deferred as spec
  organization only.
- **D-07 [LOW — defer]** — `schema_diff.rs:12-25` (module responsibilities) and
  :84-86 (`Cosmetic` doc) still say the enforced set is "equations … together
  with … keys" and "proposed equations/keys differ". The analysis delta text is
  correct; the code docs lag. Defer (doc-only, no contract effect).
- **D-08 [LOW — defer]** — `findings.rs:13-14` (witness list), :153 ("equations
  before keys before naturality") and :247-250 (`FindingKind::Violation`
  "untagged union of the equation witness and the key witness") do not mention
  the coproduct (or, pre-existing, naturality) arm. Defer (doc-only).
- **D-09 [MEDIUM — fix]** — the pre-registered conventional baseline arm cites
  an artifact that does not resolve: `fixtures/probes/coproduct/README.md:114-115`
  and `fixtures/probes/coproduct/faults.json:112` name
  `crates/catherine-cli/tests/expressibility_gaps/cases.rs` with sha256
  `2630d4be…`. That path never existed in git (`git log --all -- …/cases.rs`
  empty) and the hash matches no blob of the gap-suite tree at any commit (19
  distinct blobs checked); the case actually lives at
  `crates/catherine-cli/tests/expressibility_gaps/cases/analysis.rs`
  (:294-316 `coproduct_absence`). The other two arms in the same citation do
  resolve (`coproduct_payments.ct.json` = `95fc2ac8…`; the RED-era dispositions
  blob = `a97df123…` at `bda3819`), and the incumbent-keys arm hashes resolve at
  `79cef70` (6044d0ac…/10f9fbe1…). The verdict itself is still sound — baseline
  `mode: "executed"`, unique set non-empty, arithmetic reproduced in command 20
  — but the evidence citation is untraceable. Fix: correct the path/hash in the
  README status prose and in seq 3's `discrepancies` (seq 3 is uncommitted, so
  an in-place correction before commit is not a goalpost move); do **not** edit
  the frozen `faults.json` (its digest `f18a5566…` is the catalog pin cited by
  seq 1/2). A resolvable RED-era citation is `cases/analysis.rs` at `bda3819`,
  sha256 `8dc2d525…`.

## Pass 3 — Tests

Structural honesty is good: every finding assertion is typed/JSON-witness
based (no snapshots), `equation_violation_json_is_unchanged`
(`tests/coproducts.rs:369-386`) pins the untouched encoding, the hash-neutrality
test pins real bytes + a hash that reproduces independently and is shared with
three pre-existing suites (command 17), and no test asserts wiring, copies, or
a stubbed value. Gaps:

- **T-01 [MEDIUM — fix]** — the **`src == C` injection disjunct is untested.**
  `crates/catherine-core/tests/coproducts.rs:141-173` pins six "classes" but
  both non-injection cases (`f: C→V1`, `j: V1→V2`) miss `C` as *target*, so
  they exercise only `parse/coproducts.rs:145-155`; the `definition.src == *object`
  branch (:157-166) is never executed by any test, and the
  `assert_eq!(cases.len(), 6, "every malformed-coproduct class is pinned")` at
  :170 overclaims. The branch is correct (probe `/tmp/selfloop.ct.json` →
  `invalid_coproduct … sourced at the coproduct object` at 10:47). Minimal fix:
  one `C→C` self-loop case + column.
- **T-02 [MEDIUM — fix]** — **no artifact test for coproducts.** The keys change
  added `keys_are_covered_by_the_artifact` (`crates/catherine-core/tests/keys.rs:267-306`)
  proving the published artifact accepts a well-formed key and rejects an empty
  and a duplicate arrow list; there is no coproduct analogue, so the new
  `$defs.coproduct` `minItems`/`uniqueItems` (`schema/ct-schema-v1.json:272-293`)
  are verified only by this review's generator diff (command 16), not by a
  test. Fix: mirror the keys test (`coproducts_are_covered_by_the_artifact`).
- **T-03 [MEDIUM — fix]** — **the CLI `diff` coproduct payload/branch is
  untested.** `crates/catherine-cli/src/render/diff.rs:54-61,76-80,254-262`
  adds the only discriminated `ConstraintView` variant (`kind: "coproduct"`)
  and the human `added: coproduct C(i1∘i2)` line; the keys precedent has
  `added_key_is_named_in_payload_and_human_output`
  (`crates/catherine-cli/tests/cli_diff.rs:227-251`). Behavior is verified only
  by command 24. Fix: fixtures pair + one CLI test asserting the payload
  discriminator and the human line.
- **T-04 [MEDIUM — fix]** — **composition coproduct merge/attribution is
  untested.** `composition/resolve.rs:99-123` (`merge_coproducts`),
  `composition.rs:190-225` (`CoproductSource`, `coproduct_sources`) and
  `crates/catherine-cli/src/composition.rs:116-119,152-158` (witness → source
  attribution) have no test; only `coproducts: Vec::new()` was added to the
  builders (`tests/composition.rs`, `composition_agreement.rs`). Command 27
  proves the path works (joint run: one witness with `source:[a,b]`). Fix: one
  joint-check case asserting the coproduct witness and its source attribution.
- **T-05 [MEDIUM — fix]** — the analysis delta's scenario
  "Accepted equations constrain, proposed equations question" now names
  coproducts, but no test exercises a *proposed coproduct* leaving search
  cleanliness true. The scenario contract
  `.espectacular/changes/add-coproduct-exhaustiveness/analysis/accepted-equations-constrain-proposed-equations-question.toml`
  maps to `is_clean_accepts_accepted_clean_proposed_dirty`,
  `search_returns_witness_despite_proposed_violations`,
  `proposed_key_does_not_block_search_cleanliness` and
  `dropped_coproduct_is_weakened_with_a_rogue_variant_witness` — none of the
  first three constructs a coproduct, and the fourth uses an **accepted** one.
  Fix: `proposed_coproduct_does_not_block_search_cleanliness` (the label test
  `proposed_coproduct_is_hypothesized` proves the label, not the predicate).
- **T-06 [LOW — defer]** — the new coproduct arms in
  `tests/checks_properties.rs:144-155` and `tests/findings_properties.rs:232-243`
  are unreachable: every assembler sets `coproducts: Vec::new()`
  (`tests/common/fixture.rs:185`, `tests/common/mod.rs:64`), so the generator
  never produces a coproduct. Same pre-existing pattern as the key arm; defer
  (or extend the generator once, which would also revive the dead key arm).
- **T-07 [LOW — fix]** — all ten CLI coproduct fixtures declare
  `"format_version": "1.4"` (e.g. `tests/fixtures/coproduct/clean.ct.json:2`)
  while using the 1.5 section; no CLI test parses a 1.5 document, so the
  `KNOWN_MINOR` bump is exercised only in core. The parser legitimately caps
  rather than gates per section, but the fixtures should carry the newest minor
  (repo convention: 44 fixtures at 1.4, keys fixtures at 1.4) or one 1.5 fixture
  should be added.
- **T-08 [INFO — pass]** — no tautologies found. The probe-side hash checks
  (commands 17–19) and the equation-bytes test are non-vacuous; the two
  `assert_eq!(cases.len(), 6, …)` overclaim is T-01's evidence, not an extra
  finding.

## Pass 4 — Negative-input matrix

The surface follows the repo's split-binary convention
(`negative_matrix_coproduct.rs`; `negative_matrix.rs:21-22` documents the
pattern), and command 2 verifies 7 rows: undeclared object, undeclared arrow,
wrong-direction arrow, empty arrow list, duplicate arrow, bad status
(`invalid_status`), bad flag (exit 2) — each with exit 1, empty stdout, a
`{ok:false, envelope_kind:"error", data.code}` envelope carrying `line` and
`column`, and no panic. Two gaps:

- **M-01 [MEDIUM — fix]** — the matrix's "wrong-direction arrow (not a
  V_k -> C injection)" row uses `malformed-wrong-direction.ct.json`
  (`f: C→V1`), which triggers only the *target* check; the "sourced at the
  coproduct object" class the delta names is absent from the matrix (same gap
  as T-01). Fix: add `malformed-self-injection.ct.json` (`loop: C→C`) + one row
  (the row name may then honestly cover both disjuncts).
- **M-02 [LOW — fix]** — no row covers an **absent** `arrows` field; today it
  is `invalid_coproduct` at 1:1 (C-02). Add the row once the field handling is
  aligned (or pin the 1:1 fallback explicitly as the accepted behavior).
- **M-03 [INFO — pass]** — no silent repair: each class returns a distinct,
  positioned message and nothing is dropped; the `invalid_coproduct` code is a
  single stable code as specified, and a bad status correctly reuses
  `invalid_status`.
- **M-04 [INFO — pass]** — usage row: exit 2, clap text on stderr, empty
  stdout, no `panicked`, verified by command 2.

## Pass 5 — Regression, edge cases, polish

- **R-01 [LOW — defer]** — the human transcript prints the injection list with
  the composition operator: `render_coproduct_violation`
  (`crates/catherine-cli/src/render.rs:144-168`) reuses `arrows_string`
  (:275-281), producing `coproduct-0 Method(card_inj∘transfer_inj∘cash_inj)`
  (command 28) and `covered by i1∘i2 (disjointness)` (command 24). Keys'
  tuple is a composition; a coproduct's injections are alternatives — `+` or
  `,` would not mislead. Defer: no spec pins the transcript spelling, and the
  tests assert facts, not the separator.
- **R-02 [LOW — defer]** — `web/src/findings.ts:56-64` mirrors the violation
  union as equation/key/coproduct and `:120-128`'s fallthrough labels *any*
  other arm as coproduct, so a `NaturalityWitness` (kernel arm since 1.4,
  `findings.rs:155-165`) would render `coproduct undefined…`. Unreachable
  through the web surface today (`web/src/wasm.ts` exposes `runCheck` only,
  whose kernel walk never emits naturality), and pre-existing; the change's
  new doc comment re-asserts a completeness that is not true of the kernel
  enum. Defer, or add the fourth arm (cheap) when the panel grows a naturality
  surface.
- **R-03 [INFO — pass] regression:** no sealed identity moved —
  `fixtures/parity/small` bytes/hash reproduce; the two sealed naturality model
  hashes reproduce under the new build (command 19); the only fixtures carrying
  a previously-ignored `coproducts` block are the four probe fixtures, whose
  moved hashes are exactly the ones seq 2/3 record (command 18); the
  coproduct-free corpus is untouched (canonical writer omits an empty list,
  `canonical/value.rs:156`). Equation/key search and diff behavior unchanged
  (commands 4–6); the gap-suite row flip is consistent with its observer and
  the live-split tests (commands 10, 23); JSON Schema still compiles
  (command 9); web type-checks and tests pass (11–12).
- **R-04 [INFO]** — the diff carries unrelated behavior-neutral lint cleanups
  (`glue(..).err().expect(..)` → `expect_err(..)` in `tests/composition*.rs`,
  `&mut Vec` → `&mut [Option<CellClaim>]` in `composition/instance.rs`). These
  follow task 12's gate-cap rule; no behavior change and no finding.

## Convergence

| After pass | New CRITICAL | New issues | vs previous | False positives found | Status |
|---|---|---|---|---|---|
| 1 (correctness) | 0 | 4 | — | 0 (all reproduced by command) | CONTINUE |
| 2 (contract drift) | 0 | 9 | — (separate axis: spec/doc text) | 0 | CONTINUE |
| 3 (tests) | 0 | 8 | −11 % | 0 | CONTINUE |
| 4 (negative matrix) | 0 | 2 | −75 % | 0 | CONVERGED |
| 5 (regression/polish) | 0 | 2 | 0 % | 0 | CONVERGED |

The five passes are contract-area partitions, not editorial refinements of one
text, so the pass-2 count does not decay; convergence is asserted on the
substantive criteria: zero CRITICAL, zero behavioral defects, zero false
positives, and the residual findings are documentation/test-coverage only.

## Summary

| Severity | Count | Ids |
|---|---|---|
| CRITICAL | **0** | — |
| HIGH | 1 | D-01 |
| MEDIUM | 9 | C-03, D-02, D-09, T-01, T-02, T-03, T-04, T-05, M-01 |
| LOW | 14 | C-01, C-02, C-04, D-03, D-04, D-05, D-06, D-07, D-08, T-06, T-07, M-02, R-01, R-02 |

## What the assignment asked to verify

| Item | Result |
|---|---|
| Canonical `(object, arrows, status)` order, repeated declarations kept, empty list omitted | **PASS** — `parse/coproducts.rs:64-67`, `canonical/value.rs:165-199`; tests pin order/bytes/fixed point |
| Hash neutrality vs a pre-existing fixture (real bytes/hash) | **PASS** — `fb033e3c…` reproduced from the canonical bytes; same constant pinned by three earlier suites; naturality hashes also reproduce |
| `KNOWN_MINOR = 5` + versioning MODIFIED delta owned by this change | **PASS** — `parse/schema.rs:79-87`; delta mirrors deployed text with 5/`schema.coproducts`; no other pending change owns it (`add-bogart-pilot` has no model-document delta); 1.6 rejected, hint says 1.5 |
| JSON Schema + generated TS consistency | **PASS** — generator rerun in `/tmp` is byte-identical; both `schema` and `schema_map.target_schema` carry `coproducts`; TS compiles. Artifact *test* gap: T-02 |
| `invalid_coproduct` node positions for every class, both `V_k → C` conditions | **PASS on behavior; gap on coverage** — every top-level class positioned at line 13 (tests) and message-level in the matrix; `src == C` correct (10:47) but untested (T-01/M-01); two positional fallbacks to 1:1 (C-01 nested target schema, C-02 missing field) |
| No silent repair | **PASS** — every malformed declaration is a labeled error; nothing dropped |
| Coverage + disjointness witness shapes | **PASS** — `{coproduct,row}` (field omitted when empty) and `{coproduct,row,injections:[…]}`; CLI/human/web all render both |
| Label rules accepted→executed / proposed→hypothesized | **PASS** — core + CLI tests, property arm (dead, T-06), human `(hypothesized)` |
| Dangling suppression: no double-report; does it hide an uncovered row? | **PASS per contract** (test pins only the dangling finding); it does hide an uncovered row **and** a decidably-failing disjointness, and lowers the exit class — recorded as C-03 |
| Equation-witness bytes unchanged | **PASS** — exact-JSON test |
| Canonical ordering of coproduct findings | **PASS** — kind order then `(coproduct,row,injections)`; gap-suite output rows 2 then 4 |
| Schema diff: enforced-set, `accreted`, `weakened` witness v2-clean/v1-rejected, proposed `cosmetic`, no keys/equations regression | **PASS** — tests assert `witness` re-checks clean under v2 and rejected under v1; CLI probe confirms both witness arms; keys/equation diff suites pass. Duplicate-copy edge: C-04 |
| Probe: catalog unchanged from RED | **PASS** — `faults.json` digest `f18a5566…`; README rows = all three entries = `catalog.rs` registration (command 21); RED model hashes reproduce (command 18) |
| Probe: entries derive verdicts | **PASS** — command 20; seq 2 control `killed`, seq 3 `survived` |
| Probe: `survived` justified by an executed baseline | **PASS on verdict**, citation broken (D-09): `mode: "executed"`, detections empty, RED silence recorded; the cited `cases.rs` does not resolve |
| Probe: unique detections real/re-derivable | **PASS** — command 22 reproduces both finding ids and the clean/floor arms |
| Probe: sequence/control order | **PASS** — `ledger-check` (command 15) |
| CLI/web: both shapes render, no `undefined`, exit codes | **PASS** for coproduct (tests 2/11, commands 22/27); exit 3 for violations, 3 for `weakened`, 0 clean, 1 labeled, 2 usage; web naturality arm absent but unreachable (R-02) |
| Tests structural only, matrix covers the malformed surface, no wiring/copies/tautologies | **PASS** on structure/tautologies; matrix class gap T-01/M-01; artifact test gap T-02 |

## Design-decision verdicts

| Decision | Verdict |
|---|---|
| 1 — `keys` grammar reused; canonical order; duplicates kept; empty omitted | **PASS** (duplicate *semantics* beyond bytes: C-04) |
| 2 — coverage/disjointness failures are `violation`/`executed`, witnessed | **PASS** |
| 3 — status participates like keys; diff integration verbatim | **PASS on behavior**; the "reused verbatim" integration shape omitted the keys change's `findings` delta (D-01) |
| 4 — no pullbacks/cones/defaults | **PASS in code/spec delta**; the guardrail is absent from user docs and USING still claims the opposite (D-02) |
| 5 — probe first, capability second; verdict recorded whatever it is | **PASS on honesty** — catalog frozen, control self-negated, baseline executed, two unique detections reproduced; baseline citation broken (D-09) |
| 6 — shared-requirement rebase + archive order + next free minor | **PASS** — analysis delta is rebased on the *deployed* keys text (verified line-by-line); this change owns the 1.5 versioning delta; `openspec validate --strict --all` passes |

## Blocking before this change is committed/archived

1. **D-01 (HIGH)** — author the `findings` MODIFIED delta (coproduct witness
   arm + encoding + scenario), rebased on the deployed keys requirement text.
   `proposal.md`'s "findings untouched" must be read as "no new kinds"; the
   witness-shape requirement changes. No code change.
2. **D-02 (MEDIUM, cheap)** — correct `docs/USING.md:82,206-212` (coproducts
   are executable; the guardrail is cones/pullbacks/defaults, not coproducts).
   Task 13 owns this.
3. **D-09 (MEDIUM, cheap while seq 3 is uncommitted)** — fix the conventional
   baseline citation to a resolvable artifact (`cases/analysis.rs` at `bda3819`,
   `8dc2d525…`) in the README status prose and seq 3's `discrepancies`; leave
   the frozen `faults.json` digest alone or record a superseding entry.
4. **T-01 + M-01 (MEDIUM)** — add the `C→C` self-injection case and its matrix
   fixture/row; strengthen the core case table.
5. **T-02/T-03/T-04/T-05 (MEDIUM)** — artifact test, CLI diff test, joint-check
   attribution test, and the proposed-coproduct-does-not-block-search test.

Fix in the same commit (LOW, free): D-03 (crate README code table + check
order), D-04 (parse/mod.rs numbering), D-05 (probe README status prose), and
C-02 together with T-01's new fixture.

Deferred with reason: C-03 (spec-mandated suppression; needs a superseding
kernel-checks/coproducts spec change), C-01 (shared pointer-root refactor,
pre-existing for keys/objects/equations), C-04 (inherited multiset design;
affects equations and keys), D-06/D-07/D-08 (spec organization and code-doc
text only), T-06 (unreachable property arms, same pattern as the key arm),
M-02 (depends on C-02), R-01 (transcript spelling not in any contract),
R-02 (web surface exposes `runCheck` only; pre-existing).

## Unverified

- `just ci`, `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --workspace`, `pretender` gate, `ah check --changes
  add-coproduct-exhaustiveness --run-tests`, and the wasm32 build: not run
  here (forbidden by the assignment / a workspace run is in flight). The
  implementation report's claim that they are green is **not** verified.
- The RED binary digest `819d798f…` (revision `79cef70`) cannot be re-verified:
  the on-disk binary is the current build (`34645c07…`). The revision exists
  (`git cat-file -t 79cef70` → commit), and the claimed RED silence is
  consistent with the pre-change parser (command 18's section-dropped hashes
  reproduce the pre-registered identities), but the binary itself is gone.
- The `cases.rs`/`2630d4be…` baseline citation (D-09) is unresolvable, hence
  the conventional arm's *file identity* is unverified; its state
  (silent at RED) is corroborated by the recorded observation and by the
  gap-suite observer's `Limitation` branch, not by a re-runnable artifact.
- Cross-change archive ordering: the review ran against the combined worktree
  (`7aba364` + uncommitted), not a clean checkout of iib alone; the analysis
  delta's rebase was checked textually against the deployed keys spec.
- TypeSafe verification: `TYPESAFE_API_KEY` absent.
  BANNER: findings not verified by TypeSafe (API unavailable or key absent);
  self-reported validation in effect, every finding reproduced by the commands
  above.

## Verdict

**PASS — gate max_critical = 0** (0 critical, 1 high, 9 medium, 14 low).
**Post-fix status: PASS — D-01, D-02, D-09, the medium test gaps and the
actionable lows were fixed before the commit (see Lead dispositions).**

Committed/archived only after D-01 (the missing `findings` MODIFIED delta) is
fixed; D-02/D-09 and the medium test/artifact gaps should land in the same
commit because their artifacts are still untracked or cheap to amend. The
shipped behavior is correct and the probe is honest: the format is canonical
and hash-neutral when absent, malformed declarations are one labeled code at
their nodes, accepted declarations execute coverage and disjointness as
`violation`/`executed` with declaration-shaped witnesses (proposed ones
`hypothesized`), the enforced-set diff accretes/weakens with a re-checked
witness, and the ledger's `survived` verdict re-derives from frozen fixtures
whose catalog and RED/GREEN model hashes all reproduce exactly.
## Lead dispositions (post-review fixes)

Fixed before the iib commit:

- **D-01 (HIGH)** — the `findings` MODIFIED delta now carries the coproduct
  witness arm (`{coproduct, row, injections?}`), byte-identical equation/key
  encodings, no new kind, plus the new scenario and its contract; the
  proposal's "findings untouched" line was corrected to "no new kinds".
- **D-02** — `docs/USING.md` states coproducts as declared totality claims
  whose failure is a `violation` (advisory limitation narrowed to
  cones/pullbacks/monomorphisms, Decision-4 guardrail included).
- **D-09** — the probe baseline citation was corrected to the real artifact
  (`crates/catherine-cli/tests/expressibility_gaps/cases/analysis.rs` at
  `bda3819`, digest `8dc2d525…`); the catalog digest moved, so seq 2 and
  seq 3 were re-sealed (`0002-d717d7fa.json`, `0003-2709a990.json`), the
  untraceable pre-registration citation is recorded in seq 3's
  `discrepancies`, the verdict/unique arithmetic is unchanged, and the
  disposition table cites the new id.
- **T-01/M-01** — the `src == C` self-injection condition is pinned by a core
  case and a matrix row; a missing-`arrows` row pins the `syntax` class.
- **T-02…T-05** — added coverage for the JSON Schema artifact, the CLI `diff`
  coproduct constraint view (payload + human line), coproducts enforced
  across both documents in a joint run with source attribution, and the
  proposed-coproduct-never-blocks-a-witness clause.
- Adjacent lows: `RawCoproduct.arrows` no longer uses `#[serde(default)]`
  (matches keys), core README documents `invalid_coproduct` and the check
  order, parse step numbering corrected.

Deferred (LOW, reasons in the review tables): prose polish and the
TypeSafe run (no API key in this environment); the known `ah
check --run-tests` executor misreport (`no-tests-ran` for deployed
contracts) is pre-existing tooling behaviour, not a contract failure.
