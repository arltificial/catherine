# ro5u review — catherine-s60 (instance morphism / `naturality`)

Change: `add-instance-morphism` (ticket `catherine-s60`). Base `79cef70`,
worktree uncommitted. Contract read: the change's `proposal.md`,
`design.md`, `tasks.md`, both spec deltas (`instance-morphism`,
`model-document`), the deployed `openspec/specs/model-document/spec.md`
and `openspec/specs/domain-probes/spec.md`, plus
`docs/ro5u-checklist.md`.

Scope reviewed (s60 hunks only; the archived gap-suite change's own files
are reviewed in `catherine-j6g-ro5u.md` and only the s60-authored cells of
`docs/ref/expressibility-dispositions.md` are assessed here):

- `crates/catherine-core/src/{model,error,findings}.rs`,
  `canonical/{label,value}.rs`,
  `parse/{mod,build,schema,instance_map}.rs`,
  `schema/ct-schema-v1.json`
- `crates/catherine-analysis/src/{naturality,lib}.rs`
- `crates/catherine-cli/src/{cli,main,naturality,runners,runners/naturality,render,render/*}.rs`
- tests/fixtures: `crates/catherine-cli/tests/{naturality,negative_matrix,negative_matrix_naturality}.rs`,
  `tests/fixtures/naturality/**`, `crates/catherine-core/tests/{instance_morphism,keys,rule_lens,roundtrip,roundtrip/model_fixture,roundtrip_symmetric,golden_fixtures,checks_properties,findings_properties,editor_validation}.rs`
- ledger/probes: `crates/catherine-ledger/src/catalog.rs`,
  `ledger/probes/naturality/**`, `fixtures/probes/naturality/**`
- docs/web: `web/src/types/ct-schema-v1.ts`, README/CHANGELOG s60 hunks,
  the `naturality-absence` row, `openspec/changes/add-instance-morphism/tasks.md`,
  the eight `.espectacular/changes/add-instance-morphism/**` contracts

Not reviewed as s60: `.beads/`, `justfile`/`scripts/check-expressibility-claims.sh`,
`.espectacular/gap-suite/*.toml`, the `negative_matrix.rs`/`roundtrip.rs`
splits (gap-suite refactor, no s60 behavior).

Gate: **max_critical = 0**. Critical/high fixed; medium fixed or deferred
with a written reason; low deferred only with a reason.

## Commands executed (read-only; no repo file edited, scratch only in `/tmp`)

| # | Command | Result |
|---|---|---|
| 1 | `cargo test -p catherine-core --test instance_morphism` | **4 passed**, 0 failed |
| 2 | `cargo test -p catherine-core --test instance_morphism --test keys --test roundtrip --test roundtrip_symmetric` | **4 + 7 + 2 + 1 passed**, 0 failed (roundtrip 83.3 s) |
| 3 | `cargo test -p catherine-cli --test naturality --test negative_matrix_naturality --test expressibility_gaps` | **8 + 1 + 6 passed**, 0 failed |
| 4 | `cargo test -p catherine-core --test editor_validation` | **4 passed** (the shipped JSON Schema compiles under draft 2020-12) |
| 5 | `cargo run -q -p catherine-ledger --bin ledger-check -- ledger/probes/naturality` | exit 0 — `ok: 2 entries (2 kata), rollup verdict: (no rollup)` |
| 6 | `openspec validate --strict --all` | **16 passed, 0 failed** |
| 7 | `bash scripts/check-expressibility-claims.sh` | exit 0 — `✅ every limitation claim names a disposition row id (0 claim line(s) scanned)` |
| 8 | `catherine distinguish fixtures/probes/naturality/{crm,warehouse_break}.ct.json --observe 'account_owner@account#2' --json` | exit 0, `outcome: distinguishable`, `left: 1`, `right: 0` |
| 9 | `catherine naturality fixtures/probes/naturality/{crm,warehouse_break}.ct.json --json` | exit 3, one `violation`/`executed`, `witness {arrow: "account_owner", source_row: 2, lhs: 1, rhs: 0}` |
| 10 | `catherine naturality fixtures/probes/naturality/crm.ct.json <same file> --json` | exit 1, `unknown_map_target` (see C-03) |
| 11 | `catherine naturality tests/fixtures/naturality/crm_no_map.ct.json .../warehouse.ct.json` | exit 0, stdout empty, stderr `warning: the source document declares no instance_map; …`; `--json` puts it in `warnings[]` |
| 12 | `catherine check tests/fixtures/naturality/crm.ct.json --json` | exit 0, `findings: []`, `warnings: []` (the map is inert on `check`) |
| 13 | probe fixtures for the five untested `instance_map` rejections (scratch `/tmp/nat_{dup,ghost,badtgt,missing_obj,badkey}.json`) | correct codes + positions: `duplicate_ident` (5:79), `invalid_ident` (5:40), `invalid_ident` (5:60), `incomplete_instance_map` (5:28), `syntax` (5:54) |
| 14 | dangling-square probes (`/tmp/nat_dangling*.json`) | both sides undefined → exit 0, `findings: []`; one side undefined → exit 3, `witness.lhs: null`, human `lhs=unresolved` |
| 15 | `diff` probe fixtures vs CLI fixtures (`crm`, `warehouse_break`) | byte-identical |
| 16 | `git show HEAD:crates/catherine-cli/tests/negative_matrix.rs \| grep -c 'Row { name'` vs worktree | 45 vs 45 (split kept the main matrix intact) |

## Pass 1 — Draft / shape

Shape is **GOOD**. The change lands on the existing seams: one new parse
pass (`parse/instance_map.rs`), one analysis module
(`analysis/naturality.rs`), one runner on the shared load/envelope/exit
path (`cli/src/runners/naturality.rs` behind the thin `cli/src/naturality.rs`
wrapper), witness shape added as a third untagged arm rather than a new
finding kind, and the canonical section appended after the 1.2 section.
No new abstraction, no second pipeline, no synthesis machinery — the G2
scope guardrail holds in the code (the only executor of a map is
`naturality`).

- **[D-01, MEDIUM]** Probe evidence is registered but not wired into any
  test. `crates/catherine-ledger/src/catalog.rs:238-269` registers
  `probe-naturality`, and `ledger/probes/naturality/{0001,0002}.json` are
  committed-adjacent, but no test validates that directory (the migration
  probe has `tests/probe_ledger_entries.rs:53` +
  `tests/probe_regression_red.rs`; rule-lens has
  `tests/probe_rule_lens_extracted.rs:88`), and
  `tests/probe_catalog_green.rs:111` still loops over
  `["probe-migration", "probe-rule-lens-extracted"]`, so
  `probe_catalog_hash("probe-naturality")` is never exercised by a test.
  `ledger-check` passes (command 5) but nothing runs it in CI.
  **Disposition: fix.** Minimal: add `probe-naturality` to the
  `probe_catalog_green.rs:111` loop and add a
  `tests/probe_naturality_entries.rs` mirroring the migration test
  (`validate_dir("ledger/probes/naturality")`, 2 kata, control first and
  `killed`, `entry.expected == probe_expectations("probe-naturality")`,
  `kata.fixture_hash == model_hash(parse(fixture))`).
- **[D-02, LOW]** `fixtures/probes/naturality/` has no `README.md`
  (the migration probe carries one); provenance lives only in the sealed
  `series_report`/`procedure` fields. Deferred: not a spec requirement,
  and the entry text is self-describing.
- **[D-03, LOW]** The canonical-labeling change widens the search with a
  rationale that does not hold; see C-04. Structural, no behavior finding.

## Pass 2 — Correctness

Verified correct (evidence in the commands table and read-through):

- **Resolution orientation.** `square()` computes `lhs = map(a_S(x))`
  (`analysis/naturality.rs:162-169`) and `rhs = a_T(map(x))`
  (`:170`); the witness docs in `findings.rs:116-121` match. The 4/4/1 and
  4/4/0 computation on the crm/warehouse fixtures reproduces the
  hand-computed witness (command 9) and the CLI test asserts the same
  numbers.
- **Totality at parse.** `check_coverage`
  (`parse/instance_map.rs:149-165`) names the first uncovered row in
  sorted-name order at the per-object `rows` node; the pinned position
  `(14, 37)` is asserted by `instance_morphism.rs` and passes.
- **Target resolution at construction.** `resolve_map`
  (`analysis/naturality.rs:82-121`) resolves every source row before
  `sweep`, so `unknown_map_target` is a labeled error with empty stdout,
  never a finding (command 3 covers it; matrix row at
  `negative_matrix_naturality.rs:119`).
- **Hash neutrality.** `canonical/value.rs:46-52` emits the section only
  when present and non-empty; `label.rs:85-90` keeps the pre-1.4 labeling
  path when absent. The pinned `SMALL_PRE_CHANGE_HASH`
  (`instance_morphism.rs:42-43`) is the *same* constant independently
  pinned by `tests/rule_lens.rs:39` and `tests/schema_maps.rs:34`,
  unchanged by this diff — so the guard is not self-referential.
- **Name-blindness.** `apply_labeling` relabels cells and map source ids
  through one `cls` (`label.rs:246-274`); target names stay verbatim.
  `MAPPED_RENAMED` is a genuine test: `account`'s two rows are
  structurally symmetric (one arrow to two indistinguishable owners), so
  the branch choice is decided by the map cells themselves, and the
  rename (`omega/alpha`) still yields identical canonical bytes and hash.
  `serialize→parse` is asserted as a fixed point.
- **Schema identity.** `schema_mismatch` is reused verbatim
  (`runners.rs:256-268`), before map handling.

- **[C-01, MEDIUM] Undefined squares are silently clean.** When both
  compositions are undefined (`values[cells]` dangling on the source side
  *and* on the target side at the mapped row), `square()` sees
  `lhs == rhs == None` and returns `None`
  (`analysis/naturality.rs:171`), so the run reports a clean envelope
  with no warning. Evidence: command 14 — a source document whose
  `f(a1)` is `null` and a target whose `f(map(a1))` is `null` exits 0
  with `findings: []`. This contradicts the witness's own contract
  ("rendered as a non-answer, never silently skipped",
  `findings.rs:108-111`) and the analysis layer's stated rule that
  "`UNKNOWN` is never silently converted into clean"
  (`crates/catherine-analysis/README.md:22-24`); `check` separately
  reports the source cell as `dangling`, but `naturality` never runs
  `check`, so the envelope is the only signal. **Disposition: fix (cheap)
  or specify.** Minimal: emit the existing warning channel entry when any
  square is undefined (same shape as `missing_instance_map_warning`), or
  add a delta scenario fixing the rule "an undefined side on both sides
  counts as commutation" so the behavior is contractual. Either way add
  the one-sided/both-undefined cases to `tests/naturality.rs`.
- **[C-02, MEDIUM] Probe verdict word contradicts the spec's rule.**
  `ledger/probes/naturality/0002-46702d0f.json` records
  `verdict: "inconclusive"` while `comparison.unique_detections` is empty
  and the baseline arm is executed. `openspec/specs/domain-probes/spec.md:91-95`
  says *"WHEN the baseline arm detects everything the probe fired … THEN
  the entry is `killed`"*, and `harness::derive_probe_verdict`
  (`crates/catherine-ledger/src/harness.rs:92-101`) returns `Killed` for
  an empty unique set. `inconclusive` is the cited-only/missing-arm
  ceiling. No test compares recorded against derived verdicts, so
  `ledger-check` cannot catch it. `fixtures/probes/naturality/faults.json`
  repeats `"verdict_expected": "inconclusive"` and
  `crates/catherine-ledger/src/catalog.rs:262-266` states the same.
  **Disposition: fix before the entries are committed** (they are still
  untracked, so this is a free edit): set the non-control verdict to
  `killed`, keep the wrong-question text in `expressibility_gap` (which
  already carries it), and add the derivation assertion from D-01.
  Over-claiming is not involved — the null result is stated — but the
  closed vocabulary is a contract.
- **[C-03, MEDIUM] "Same document twice is trivially clean" is not true in
  general.** The delta scenario
  (`openspec/changes/add-instance-morphism/specs/instance-morphism/spec.md:32-35`)
  and the bound contract
  (`.espectacular/changes/add-instance-morphism/instance-morphism/same-document-twice-is-trivially-clean.toml:2-3`)
  assert that when both paths name the same document "every commutation
  square holds by construction … accepted, never an error". Evidence:
  command 10 — `catherine naturality crm.ct.json crm.ct.json` exits **1**
  with `unknown_map_target`, because the only document whose map is total
  onto itself *and* commutes is the identity map; the test
  (`tests/naturality.rs:87-88`) exercises only the identity fixture.
  **Disposition: fix the contract text** (qualify: "a document whose
  declared map is the identity, run against itself"), and either keep the
  test as the identity-map case or add the honest non-identity assertion.
  The implementation needs no change: rejecting an unresolvable cell is
  the mandated construction-time behavior.
- **[C-04, LOW] The prune gate in `canonical/label.rs:63-72` rests on a
  false premise.** The comment says two branches with equal partitions
  "can still serialize differently once map cells are relabeled". After
  `while refine(…)`, class values are per-object ranks recomputed from
  signatures (`apply_ranks`, `label.rs:226-241`), so an equal
  `partition_key` implies an equal `cls` for every row — and
  `apply_labeling` (cells + map relabeling, `:246-274`) is a pure
  function of `(model, cls)`, so the two branches produce identical
  bytes, map included. Disabling the dedup is therefore conservative, not
  required; it costs redundant exploration on mapped models but adds no
  behavior. **Disposition: defer** (no correctness impact) — either
  restore the unconditioned prune with a test proving the map is
  relabeled inside the leaf labeling (`MAPPED_RENAMED` already covers
  it), or reword the comment to state the real reason (cheap insurance at
  a known-small cost).
- **[C-05, LOW] Design Decision 2 names a field that cannot exist.**
  `design.md:28` says the `unknown_map_target` error names "object, row,
  and arrow"; a per-object row map has no arrow dimension. The shipped
  error names object, source row, and declared target row
  (`runners/naturality.rs:143-176`), which is what the delta requires
  ("naming the object, row, and map cell",
  `specs/instance-morphism/spec.md:42-47`). **Disposition: fix the
  design text** (one line) so the contract set is self-consistent.

## Pass 3 — Tests

Strong where it counts: structural assertions (no snapshots), a genuine
rename-invariance test, a distinct pre-change hash constant, the
disjointness guard in both property suites (`checks_properties.rs:140-142`,
`findings_properties.rs:215-217` assert `check` never emits naturality
witnesses), and the matrix split leaving the main binary's 45 rows intact.

- **[T-01, MEDIUM] Five new rejection branches have zero tests
  anywhere.** `parse/instance_map.rs` — duplicate entry object
  (`:55-61`), undeclared object (`:84-90`), source key that is not a
  declared row (`:125-134`), target that is not an ident (`:135-141`),
  and a missing per-object entry (`reject_uncovered_objects`, `:189-198`).
  Command 13 shows all five behave correctly (right code, right
  line/column), so this is coverage, not correctness. **Disposition:
  fix.** Minimal: five assertions in
  `crates/catherine-core/tests/instance_morphism.rs` using the scratch
  documents from command 13.
- **[T-02, MEDIUM] The `Option` witness arms and the `unresolved`
  rendering are untested.** Both witness fields are `Option<RowId>`
  (`findings.rs:116-121`) and the human renderer branches to
  `"unresolved"` (`render.rs:159-165`), but no fixture reaches either:
  the only committed naturality pairs have both sides resolved. Combined
  with C-01 this leaves the whole third state of the check unpinned.
  **Disposition: fix** with the C-01 test (one-sided and both-sided
  undefined).
- **[T-03, MEDIUM] The `naturality-absence` case is too weak for a
  `closed` row.** `crates/catherine-cli/tests/expressibility_gaps/cases/analysis.rs:107-131`
  runs `naturality` on the *commuting* pair and treats the row as cleared
  as soon as the exit code is "not 2, and if 2 then not
  'unrecognized subcommand'". A `naturality` that returned exit 1 (or a
  wrong witness) on every input would keep the `closed` row green. The
  case asserts neither the exit 3 the row's observation quotes nor the
  witness. **Disposition: fix** — assert exit 3 + one
  `violation`/`executed` with `{arrow: account_owner, source_row: 2,
  lhs: 1, rhs: 0}` (and keep the `check`-silence half), using a
  break-pair fixture under `gap_suite/`.
- **[T-04, LOW] Probe entry invariants are not asserted.**
  `entry.expected == probe_expectations("probe-naturality")`,
  `catalog_hash`, and `fixture_hash == model_hash(fixture)` are checked
  by no test (the migration analogue asserts all three,
  `probe_ledger_entries.rs:78-97`). Folded into D-01's fix; the fixture
  hashes themselves are unverified here (no surface exposes `model_hash`
  — see Unverified).

## Pass 4 — Negative-input matrix

- **[N-01, MEDIUM] The 7-row matrix misses the new section's structural
  rejection classes.** `negative_matrix_naturality.rs:116-136` covers:
  incomplete claim, unknown target, absent claim (warning), second-doc
  schema mismatch, second-doc parse error, second-doc version, unknown
  flag. It does not cover the three map-structure rejections this change
  introduces (undeclared map object, ill-formed target ident, duplicate
  map entry) nor a source-document parse failure — and those three are
  also untested at the unit level (T-01), so the classes are covered
  nowhere. The master matrix carries an analogue for the neighbouring
  surface (`migrate × garbage instance rows`), so the omission is
  inconsistent with the standing convention. **Disposition: fix** — add
  three `Expect::Error` rows (`invalid_ident`, `invalid_ident`,
  `duplicate_ident`); the fixtures already exist as command 13's scratch
  documents.
- Verified good: the absent-claim row is genuinely non-silent in both
  modes (command 11: envelope `warnings[0].message` contains
  `instance_map`, stderr empty; human mode prints
  `warning: …` on stderr with empty stdout, exit 0); the split binary
  keeps the main matrix intact (45 rows before and after, command 16);
  the violation exit class is asserted at both the CLI and matrix level.
- **[N-02, LOW]** The missing-per-object-entry error points at
  `/instance_map/maps` rather than a per-object node
  (`parse/instance_map.rs:189-198`). Documented in the code and consistent
  with the delta's "offending map node"; recorded so it is not "fixed"
  later by accident.

## Pass 5 — Regression risk / excellence

- **[E-01, HIGH] The versioning `MODIFIED` delta is missing — the deployed
  spec now contradicts the parser.** `openspec/specs/model-document/spec.md:18-21`
  still says `KNOWN_MINOR`, *currently 3*: 1.1 `schema_map`, 1.2
  `rule_presentation`, 1.3 `schema.keys`. The code accepts **1.4**
  (`parse/schema.rs:81`, `KNOWN_MINOR = 4`), the error hint says 1.4
  (`render/errors.rs:100`), and `crates/catherine-core/README.md:78-81`
  lists 1.4. Design Decision 6 (`design.md:64-75`) and `tasks.md:32` both
  make the delta conditional on "no other pending change owns it"; that
  condition is now met — `add-declarative-keys` is archived and
  `add-coproduct-exhaustiveness/design.md:86-91` explicitly defers
  ownership to *this* change ("keys today; `add-instance-morphism`
  next"). `openspec validate --strict --all` still passes (command 6)
  precisely because an ADDED-only delta is legal, which is why this
  escapes every gate. **Disposition: must fix before archive** —
  add a `MODIFIED` `model-document` requirement delta updating the
  `currently N` list to 1.4 (`instance_map`) and drop the "illustrative
  only" escape in `tasks.md:32`; `tasks.md` item 5 is checked while its
  conditional clause is unfulfilled.
- **[E-02, MEDIUM] The flipped disposition row's evidence is not
  reproducible and overstates the fix.**
  `docs/ref/expressibility-dispositions.md:56` quotes
  `catherine naturality naturality_crm.ct.json naturality_warehouse_break.ct.json`
  → exit 3. There is **no** `naturality_warehouse_break.ct.json` anywhere
  in the repo (`find . -name 'naturality_warehouse_break*'` → nothing;
  `gap_suite/` holds only `naturality_crm.ct.json` +
  `naturality_warehouse.ct.json`), and the registered case runs the
  commuting pair (exit 0) — the very mismatch the j6g review deferred to
  this change (`catherine-j6g-ro5u.md:430-439`: "the `_break` fixture
  name must match what the case runs"). The same cell says the map "is
  parsed, hashed, and no longer silent", but `check` on a mapped document
  still returns `findings: []`, `warnings: []` (command 12); nonsilence
  belongs to `naturality`, not to `check`. The table's own contract
  ("produced by the exact invocation it quotes") is therefore violated
  for this row. **Disposition: fix** — either commit
  `gap_suite/naturality_warehouse_break.ct.json` and quote it (matching
  T-03's assertion), or quote the real pair (`…/crm.ct.json
  …/warehouse_break.ct.json` from `tests/fixtures/naturality/`) and
  qualify "no longer silent" as "the `naturality` surface executes the
  map; `check` still ignores it".
- **[E-03, LOW]** `docs/ref/expressibility-dispositions.md:59` still reads
  "Counts at registration: 12 rows, 12 `gap`, 0 `closed`" while the table
  now has 1 `closed`; the file also has no final newline. **Defer or
  fix** when E-02 is edited (the j6g review already asks for the count to
  be derived, `catherine-j6g-ro5u.md:305-311`).
- **[E-04, LOW]** Stale module docs in tests the change edited:
  `crates/catherine-core/tests/keys.rs:9` and
  `crates/catherine-core/tests/rule_lens.rs:13` still say "1.3 known, 1.4
  rejected". **Fix** (two phrases) — the assertions around them already
  say 1.4/1.5.
- **[E-05, LOW]** The gap-suite naturality fixtures declare
  `"format_version": "1.3"`
  (`crates/catherine-cli/tests/fixtures/gap_suite/naturality_crm.ct.json:2`)
  while carrying a 1.4 section, and the row's prose claims "(format 1.4)
  is parsed". The parser deliberately does not gate sections by declared
  minor (pre-existing), so this is honest behavior but weak evidence.
  **Fix** with E-02: bump both fixtures to 1.4.
- **[E-06, LOW]** The JSON Schema artifact is stricter than the parser for
  the new section: `ct-schema-v1.json:191-216` requires `maps` and sets
  `additionalProperties: false` on map entries, while
  `parse/instance_map.rs:29-42` has `#[serde(default)] maps` (so
  `"instance_map": {}` parses and is normalized away) and serde ignores
  unknown keys. Consistent with the registered
  `unknown-top-level-fields` limitation and with `rule_presentation`, so
  **defer**: the artifact remains a filter, the kernel the authority
  (`editor_validation.rs:…the_artifact_is_not_the_version_authority`).
  Recorded so the divergence is not mistaken for parser-enforced
  strictness.
- **[E-07, LOW]** The absent-claim outcome (warn + exit 0) is not in the
  `instance-morphism` delta; it exists only in help text
  (`cli.rs:167-171`), README, tests, and the matrix. It is also the
  opposite of the `migrate` precedent for the same "no claim to execute"
  shape (`no_schema_map` is a labeled error, exit 1,
  `cli/src/migrate.rs:74-84`). **Disposition: fix the delta** (one
  scenario: "Absent claim is a warned non-answer, exit 0") so the
  shipped surface is contractual; changing the CLI instead would need a
  stated reason, since CI consumers currently read exit 0 here.
- **[E-08, LOW]** Subcommand inventories are not updated:
  `openspec/specs/cli/spec.md:2-6` (and the identical generated
  `docs/generated/specs/cli.md:5`) still enumerate
  `check, findings, search, redundant, underconstraint, distinguish, diff,
  query, migrate, rules`. The change declares no `cli` delta, so this is
  docs drift rather than a broken requirement. **Defer** to the next
  archive that touches the `cli` spec, or fix the Purpose line.
- **[E-09, LOW]** The shared error envelope's `remediation[0].command` is
  the generic `catherine check <model.ct.json>` for `unknown_map_target`
  (`render/envelope.rs:48-58`). Pre-existing for every labeled error
  (`schema_mismatch`, `no_schema_map`), so **defer**; noted only because
  the new error's hint is surface-specific while its command is not.

Regression areas checked with no finding:

- **Pre-1.4 documents**: absent field → `None` → section omitted and
  labeling path unchanged; three independent pre-change hash pins stay
  green (command 2). Sealed `model_hashes` refer to committed fixtures
  with no map, so no sealed artifact moves.
- **migrate**: `reindex` builds an `Instance`, not a `Model`
  (`core/migrate.rs:163-178`); `round_trip` builds `Model::empty()` and
  assigns `schema`/`instance`, so no source map can leak into a migrated
  document (`:192-203`).
- **diff**: classifies schema edits only; the map is same-schema data and
  correctly inert.
- **wasm**: `catherine-wasm/src/boundary.rs` decodes via `parse` and never
  constructs a `Model` literal, so the added field is compile-checked;
  the web type is regenerated and matches the artifact
  (`web/src/types/ct-schema-v1.ts:92-98`, `schema.instance_map` shape
  identical). `editor_validation` recompiles the artifact (command 4).
- **`did_you_mean`**: `main.rs:60` adds `naturality`; no enumeration test
  pins the list, so nothing else to update.
- **Struct-literal breakage**: the only non-test `Model { … }` is
  `parse/build.rs:38`; property/fixture builders were updated
  (`tests/roundtrip/model_fixture.rs:88-100`, `roundtrip_symmetric.rs`,
  `checks_properties`, `findings_properties`).

## Convergence

| After pass | New CRITICAL | New issues | vs previous | False positives found | Status |
|---|---|---|---|---|---|
| 2 (correctness) | 0 | 5 | — | 0 (each reproduced by command) | CONTINUE |
| 3 (tests) | 0 | 4 | −20 % | 0 | CONTINUE |
| 4 (negative matrix) | 0 | 2 | −50 % | 0 | CONVERGED |
| 5 (regression/excellence) | 0 | 10 | — (release/contract sweep, not editorial) | 0 | CONVERGED |

No finding below was asserted without either a read-through of the cited
line or a command in the table above; verified-by-execution findings are
C-01, C-02 (derivation), C-03, C-04 (code path), T-01, T-02, D-01,
N-01, E-02, E-05, E-07. Findings not verified by TypeSafe (API absent):
all — self-reported validation in effect.

## Summary

| Severity | Count | Ids |
|---|---|---|
| CRITICAL | **0** | — |
| HIGH | 1 | E-01 |
| MEDIUM | 9 | D-01, C-01, C-02, C-03, T-01, T-02, T-03, N-01, E-02 |
| LOW | 12 | D-02, D-03/C-04 (one item), C-05, T-04, N-02, E-03, E-04, E-05, E-06, E-07, E-08, E-09 |

Design-decision verdicts:

| Decision | Verdict |
|---|---|
| 1 — map in the source doc, hashed, absent/empty hash-neutral | **PASS** (writer omits; three independent pre-change pins unchanged) |
| 2 — totality at parse, target resolution at construction | **PASS on behavior** (line/column + labeled error before findings); design text names an "arrow" that a row map cannot have (C-05) |
| 3 — break is one `violation`/`executed` with `{arrow, source_row, lhs, rhs}`; untagged union disjoint; equation JSON unchanged | **PASS**, except the third state: both sides undefined is reported clean (C-01) and its witness shape is unspecified/untested (T-02) |
| 4 — identical schemas only, `schema_mismatch` reused | **PASS**; "same document twice" as written overclaims (C-03) |
| 5 — probe scoped, floor self-negates, baseline executed and cited, no unique claims | **PASS on honesty** (verified baseline + probe arms, empty unique sets); the non-control verdict word contradicts the spec (C-02); the directory has no validating test (D-01) |
| 6 — `KNOWN_MINOR` bump + versioning handling | **FAIL** — bump landed, `MODIFIED` versioning delta did not (E-01) |
| Name-blind canonicalization (dense ids + side table) | **PASS** — source rename invariant (hash + meaning), `serialize/parse` fixed point; `MAPPED_RENAMED` exercises a structurally symmetric tie broken only by map cells |
| Negative matrix (7 rows, split binary, absent-map non-silent) | **PASS** on the rows present; three new section classes uncovered (N-01) |
| Regression (pre-1.4 docs, migrate/diff, wasm, did_you_mean) | **PASS** — no sealed hash moves, no map leak path |

## Blocking before this change is committed/archived

1. **E-01 (HIGH)** — add the `MODIFIED` `model-document` versioning delta
   (currently 1.4 `instance_map`) so
   `openspec/specs/model-document/spec.md:18-21` matches
   `parse/schema.rs:81`; uncheck or amend `tasks.md:32`'s conditional
   clause. Minimal fix: one requirement delta with the updated
   `currently N` list plus the 1.4 bullet; no code change.
2. **C-02 (MEDIUM, free now)** — flip the non-control probe verdict to
   `killed` in `ledger/probes/naturality/0002-46702d0f.json`,
   `fixtures/probes/naturality/faults.json`, and
   `crates/catherine-ledger/src/catalog.rs:262-266` before those files are
   committed (they are untracked, so no superseding entry is needed).
3. **C-03 (MEDIUM)** — qualify the same-document scenario in the delta and
   in the `.espectacular` contract, or the shipped spec asserts an
   accepted invocation that exits 1.
4. **E-02 + T-03 (MEDIUM)** — make the flipped row's quoted invocation
   reproducible and strengthen the case assertion to the exit-3 witness.

Fix-or-defer with reason (MEDIUM): C-01 (fix cheaply, or specify the
undefined-square rule), D-01 (add the probe test; ledger-check alone is
not a gate), T-01/T-02 (unit tests, five + two cases), N-01 (three matrix
rows).

Deferred with reason: C-04 (conservative gate, no behavior; rationale
comment wrong), D-02 (README parity), E-03/E-04/E-05/E-08 (text),
E-06 (artifact-vs-parser leniency is the registered unknown-fields
policy), E-07 (should be fixed as a delta scenario, but the shipped
behavior is non-silent and documented), E-09 (shared envelope
convention).

## Unverified

- `just ci`, `cargo fmt --check`, `cargo clippy -- -D warnings`,
  `cargo test --workspace`, `pretender check --mode gate`,
  `cargo check -p catherine-wasm --target wasm32-unknown-unknown`,
  `ah check --changes add-instance-morphism --run-tests` — out of scope
  for this review (forbidden or not run); `tasks.md` item 11 claims them
  green and that claim is **not** verified here. A workspace run was in
  flight during this review.
- `model_hash` of `fixtures/probes/naturality/{crm,warehouse_break}.ct.json`:
  not recomputed (no CLI/wasm surface exposes `model_hash`, and
  `ledger-check` only checks hash presence), so the entries'
  `fixture_hash` values are unverified until D-01's test exists.
  `ledger-check` did verify seq order, control-first, self-negation,
  `catalog_hash`, and the verdict vocabulary.
- Reproducibility of the generated web type: `npm run generate:types` was
  not re-run; `web/src/types/ct-schema-v1.ts` was compared by hand against
  the artifact and matches.
- Cross-change ordering: the s60 rows/artifacts depend on the archived
  gap-suite commit; this review ran on the combined worktree, not on a
  clean checkout of either change alone.

## Verdict

**PASS** — gate **max_critical = 0** (0 critical, 1 high, 9 medium, 12 low).
**Post-fix status: PASS with all non-deferred findings fixed; C-01/T-02
deferred to `catherine-ke1` (see Lead dispositions).**
The feature does what the delta claims on the shipped fixtures: a total
declared map is hashed and hash-neutral when absent, malformed claims are
labeled errors at their nodes and before findings, a break is exactly one
`violation`/`executed` with the arrow, source row, and both composed
values, schemas must be identical, and canonicalization stays name-blind.
The HIGH (E-01) is an apply-time spec obligation the change itself
scheduled, not a runtime defect; it must land with this change, and the
three MEDIUM entries that are free right now (C-02, C-03, E-02/T-03)
should land in the same commit because their artifacts are still
untracked.

## Lead dispositions (post-review fixes)

Fixed before the s60 commit:

- **E-01** — the change's `model-document` delta now carries the MODIFIED
  versioning requirement (`KNOWN_MINOR = 4`, list extended with `1.4 adds
  instance_map`), byte-compatible with the deployed requirement except the
  illustrative minor list; a staged `model-document/current-major-parses`
  contract keeps the overlay conflict-free. `tasks.md` records that this
  change owns the delta (Decision 6).
- **C-02** — the probe entry verdict is `killed` (parity with an executed
  baseline and an empty unique set, wrong-question gap recorded). Both
  entries were re-sealed because the catalog prose changed:
  `0001-44e5c4c8.json` and `0002-42f47295.json`; `faults.json` and the
  table's evidence path were updated.
- **C-03** — the same-document scenario is narrowed to an identity map;
  a same-document run whose map targets another instance is
  `unknown_map_target`.
- **E-02 / T-03** — the `naturality-absence` row quotes a runnable
  committed-fixture invocation, and the case's cleared signature requires
  the commuting pair quiet *and* the break pair exit 3 with exactly the
  quoted witness; anything else panics.
- **D-01 / T-04** — `probe-naturality` is registered in
  `probe_catalog_green`, and `tests/probe_naturality.rs` pins fixture
  hashes, catalog hash, control self-negation, and recorded == derived
  verdict per entry.
- **T-01** — the five untested `instance_map` rejections are pinned with
  their codes and node positions.
- **N-01** — the naturality matrix gains undeclared-object,
  ill-formed-target, duplicate-entry, and source-parse-failure rows.
- Deployed version contract: the stale `current_major_parses_a_1_3_document`
  flag (and a flag-name typo) were corrected to the renamed tests.

Deferred with reason:

- **C-01 / T-02** — a fully undefined commutation square reports a silent
  clean. Fixing it requires a spec decision on the undefined-square
  outcome (warning vs a `unknown`-labeled shape; the taxonomy is closed),
  so it is filed as **catherine-ke1**; the one-sided `lhs/rhs = null`
  rendering remains pinned by the existing witness type.
- Lows: accepted or deferred as recorded in the tables above (fixture-hash
  verification is now covered by `probe_naturality.rs`; the generate-types
  run is a release-time check).