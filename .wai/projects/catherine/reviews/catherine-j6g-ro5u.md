# ro5u review — catherine-j6g (expressibility gap suite)

Change: `add-expressibility-gap-suite` (archived 2026-11-16). Base `79cef70`,
worktree uncommitted. Contract read: archived `proposal.md`, `design.md`,
`tasks.md`, `specs/gap-suite/spec.md`, and the merged
`openspec/specs/gap-suite/spec.md`.

Scope reviewed (j6g only, per assignment):

- `crates/catherine-cli/tests/expressibility_gaps/**` (`main.rs`, `runner.rs`,
  `cases/{mod,probe,schema,migration,analysis,future}.rs`) + 19 fixtures under
  `tests/fixtures/gap_suite/`
- `docs/ref/expressibility-dispositions.md`
- `scripts/check-expressibility-claims.sh`
- `justfile` (`check-gap-claims` recipe + its `ci` line)
- `README.md`, `CHANGELOG.md` (gap-suite hunks)
- `openspec/specs/gap-suite/spec.md`, the archived change, `.espectacular/gap-suite/*.toml`

Not reviewed as j6g (concurrent sibling work in the same worktree):
`crates/*/src/**` (0 staged `src` files in the j6g index), `add-instance-morphism`,
`add-model-composition`, `add-coproduct-exhaustiveness`, the
`negative_matrix.rs` / `roundtrip.rs` split. Where sibling edits change j6g's
verdict (the `naturality-absence` row and the `ledger/probes/naturality/` artifact)
the ordering hazard is recorded under pass 5, not as a j6g defect.

Gate: **max_critical = 0**. Critical/high must be fixed; medium fixed or
deferred with a written reason; low deferred only with a reason.

## Commands executed (all read-only; no repo file edited)

| # | Command | Result |
|---|---|---|
| 1 | `cargo test -p catherine-cli --test expressibility_gaps` | **6 passed**, 0 failed (0.08 s) |
| 2 | `just check-gap-claims` | exit 0 — `✅ … (0 claim line(s) scanned)` |
| 3 | 12 table rows' quoted invocations re-executed against `target/debug/catherine` from `tests/fixtures/gap_suite/` | all 12 observations reproduce (details in pass 2); associativity is exactly `18` findings, all `kind=violation`, all `witness.equation=7` |
| 4 | scratch copy `/tmp/j6g-probe` (rsync, `CARGO_TARGET_DIR=/tmp/j6g-target`): baseline suite | 6 passed |
| 5 | probe A: `naturality-absence` flipped to `gap` (0 closed rows left) → `regressed_limitation_must_flip_its_row_back` | **PASSED while checking 0 closed rows** |
| 6 | probe A2: same table → `fixed_limitation_must_flip_its_row` | FAILED naming `naturality-absence` with flip guidance (`runner.rs:85`) |
| 7 | probe B: `unknown-top-level-fields` set `closed` → `regressed…` test | FAILED naming `unknown-top-level-fields` (direction-2 mechanism works) |
| 8 | probe C: closed evidence `1a2b3c4` (7 hex chars, not a commit) | **PASSED** (evidence check accepts any 7–40 hex token) |
| 9 | probe D: closed evidence `ledger/probes/nope/does-not-exist.json` | FAILED (artifact existence IS checked) |
| 10 | probe T1/T2: corrupt `declared_cone.ct.json` (`{ not json`) | gap direction says "flip the row to `closed`"; after flipping the row to `closed` (citing an unrelated artifact) the closed-direction test **passes** |
| 11 | probe F/F2: short row `\| broken-row \|` appended / inserted | panics `malformed disposition row (expected six cells)` (`runner.rs:129`) |
| 12 | probe G: extra `\|` inside a cell | panics (same assert) |
| 13 | probe H: disposition `Gap` (capitalised) | panics `disposition must be bare gap or closed` (`runner.rs:141`) |
| 14 | probes I/J: extra row with no case; duplicate row id | both FAIL (runner.rs:297 / :285) |
| 15 | probe K/L: table missing / table with header+separator only | gate exit 1 both ways (fail-closed) |
| 16 | probes N/R: `Cones cannot be declared…` appended to `docs/USING.md` / `README.md` | gate exit 1, names `docs/USING.md:308` / `README.md:162` |
| 17 | probe M: `Unknown top-level fields are silently ignored.` in `docs/USING.md` | gate exit 0, **0 claim lines scanned** (slip-through) |
| 18 | probe O4: `Optional arrows cannot be declared.` inside a real `## Close note` section of `openspec/changes/archive/2026-11-16-add-expressibility-gap-suite/tasks.md` | gate exit 0 — close-note sections are not scanned |
| 19 | probe O3: same claim in a literal `openspec/changes/close-note-j6g.md` | gate exit 1 — the `close-note*.md` glob works, but no such file exists anywhere in the repo (`find openspec -name 'close-note*.md'` → 0) |
| 20 | probe P: `Identity laws cannot be declared; see also the \`coproduct-absence\` fixture.` | gate exit 0 — any row id anywhere on the line licenses the claim |
| 21 | probe Q: claim appended to `docs/generated/specs/kernel-checks.md` | gate exit 0 (excluded; see L/M below) |
| 22 | each of the 12 table rows' *Claimed limitation* cell matched against the gate's `CLAIM_PATTERN` | **4 caught / 8 missed** |
| 23 | `git ls-files docs/generated` / `git ls-files --error-unmatch ledger/probes/naturality/0002-46702d0f.json` | both empty/error → untracked |

Environment note: `TYPESAFE_API_KEY` is unset, so per the skill this review uses
self-reported validation; the false-positive estimate is therefore unmeasured.
Every finding below is grounded in an executed observation or a quoted line,
so I estimate FP risk as low.

---

## Pass 1 — DRAFT / shape

Assessment: the shape is right and matches the in-repo precedent
(`docs/ref/pattern-kata-dispositions.md` + an executing suite). One table is the
only source of dispositions; the Rust side holds cases with ids, and
`every_case_is_registered` enforces bidirectional coverage plus unique ids
(`runner.rs:277-303`). The 467-line original `cases.rs` is split into five
thematic modules plus `probe.rs`; every file is now < 400 lines
(`runner.rs` 397, `analysis.rs` 164, `main.rs` 119). No kernel/CLI/format code
is touched: `git diff --cached --name-only | grep -c '^crates/.*/src/'` = 0.
The gate is one script and one recipe; the phrase list is duplicated as prose in
three places (script header `scripts/check-expressibility-claims.sh:20-27`,
runner header `runner.rs:33-38`, justfile comment) with one enforcement site.

Shape quality: **GOOD**. Findings: [DRAFT-001] (low), [DRAFT-002] (low).

- **[DRAFT-001] low — `crates/.../expressibility_gaps/main.rs:8-10`**:
  header says every test is bound by a scenario contract under
  `.espectacular/changes/add-expressibility-gap-suite/gap-suite/`; that path does
  not exist — the contracts were archived to `.espectacular/gap-suite/*.toml`
  (six files, ids match the six test names). Disposition: **fix** (one path
  string), or a test asserting the contract files exist.
- **[DRAFT-002] low — `runner.rs:33-38` vs `scripts/…:20-27`**: the phrase list
  exists as enforced text in the script and as prose in the runner header with
  no test that the two agree. Disposition: **deferred** — the enforced list is
  the contract; a properties test (see [HIGH-003]) subsumes this.

## Pass 2 — CORRECTNESS (contract vs implementation)

I re-executed every row's quoted observation against the current binary. All 12
reproduce, so the table's *Verified HEAD observation* column is truthful for
everything executable here (exception: the `naturality-absence` cell, which a
sibling change already edited — see pass 5). Provenance citations
(`claude/b417f19b-…`, `kimi/1a1dee7a-…`) are present on all 12 rows as the spec
requires; the corpus itself is external, so provenance is unverifiable in-repo.

**Verified reproductions (row → observed):** identity ×2 → exit 1
`error[syntax] … equation path must be non-empty`; optional → exit 4, one
`dangling` at row 1, arrow `address`; unknown fields → exit 0 silent; cones →
exit 0 silent; non-functorial map → `migrate` exit 0 "0 lost data finding(s)";
dropped equation → `migrate` exit 0 "0 lost data finding(s)"; wrong-shape and
zero-row → exit 0 silent; distinguish → bare exit 2 (missing `--observe`),
`f@a#0` → `equivalent-in-scope`, `g@a#0` → `distinguishable`; associativity →
exit 3, 18×`violation` at equation 7, omitted triples exit 0 silent;
composition → joint `check a b c` exit 2 `unexpected argument`, each doc alone
exit 0; coproduct → exit 0 silent.

### [HIGH-001] `Observation::Cleared` is a catch-all, so a broken fixture launders a limitation into `closed`

Location: `cases/probe.rs:41-58` (`silently_accepted`, `all_silent`),
`cases/migration.rs:39-41` (`3 | 8 => Observation::Cleared`),
`cases/analysis.rs` (`naturality_absence` `else { Cleared }`,
`coproduct_absence` `0 | 1 | 3 | 4 | 6 => Cleared`), `cases/future.rs:51`.

Description: every case is written `if <claimed signature> { Limitation } else
{ Cleared }`. The `Cleared` arm does not assert the *fixed* behavior; it accepts
anything that is not the claimed signature — a labeled syntax error, an
unrelated exit 3/4/8, a usage error. `cases/mod.rs:15-18` promises the opposite
("a fixture that fails for an unrelated reason panics instead of being read as
a flip").

Evidence (executed, probe T1/T2): corrupting `declared_cone.ct.json` to
`{ not json` makes `fixed_limitation_must_flip_its_row` fail with "row
`declared-cones-ignored` … flip the row to `closed`"; flipping that row to
`closed` with an unrelated artifact as evidence makes
`regressed_limitation_must_flip_its_row_back` **pass**. The suite is therefore
green with a broken fixture and a false `closed` row — the exact failure
Decision 1 exists to prevent.

Minimal fix: in each case, assert the *cleared* signature explicitly (e.g.
cones: `code == 0 && findings.is_empty() && !warnings.is_empty()` or a labeled
`warnings` entry naming the declaration; coproduct: only the specific
coverage/disjointness finding kind), and `panic!` on everything else —
narrowing `coproduct_absence`'s `0 | 1 | 3 | 4 | 6` and `migrate_without_loss`'s
`3 | 8` first. Add one negative test per case family proving an unrelated
failure panics.

Disposition: **must fix (blocking; not fixed — read-only review).**

### [HIGH-002] the "openspec close notes" half of the gate scans zero files; real close notes are `## Close note` sections

Location: `scripts/check-expressibility-claims.sh:92`
(`find "$ROOT/openspec" -type f -name 'close-note*.md'`), header
`:16-19` ("the rest of `openspec/changes/**` (only close-note files are
scanned)").

Evidence: `find openspec -name 'close-note*.md'` → 0 files (this repo's close
notes are `## Close note` sections: `openspec/changes/archive/2026-11-16-add-declarative-keys/tasks.md:59`,
`…add-rules-projection/tasks.md:44`). Probe O4 appends
`Optional arrows cannot be declared.` under a `## Close note` in this change's
own `tasks.md` → gate exit 0, `0 claim line(s) scanned`. Probe O3 shows the glob
itself works when a file is literally named `close-note-*.md`, so the clause is
dead code against the repo's actual convention. Spec requirement
(`openspec/specs/gap-suite/spec.md`, "openspec close notes") is unmet; the
change's own task 11 writes a close note in a file the gate cannot see.

Minimal fix: scan the `## Close note` section of
`openspec/{changes,changes/archive}/**/{tasks,design}.md` (awk range from
`/^## Close note/` to the next `^## `), keeping the `close-note*.md` glob as a
fallback. Add a synthetic-file test: a temp copy of a `tasks.md` with a claim in
its close-note section must fail.

Disposition: **must fix (blocking).**

### [HIGH-003] the phrase list misses 8 of the 12 claims the change itself registers

Location: `scripts/check-expressibility-claims.sh:35-37` (list), `:60-66`
(substring acceptance), `runner.rs:306-330` (the only test of the gate).

Evidence (executed): matching each table row's *Claimed limitation* cell against
`CLAIM_PATTERN` gives **4 caught / 8 missed**. Missed, verbatim from the
registry: `unknown-top-level-fields`, `declared-cones-ignored`,
`non-functorial-schema-map`, `dropped-equations-on-migrate`,
`vacuous-clean-instances`, `distinguish-observation-scope`,
`associativity-scaffolding`, `composition-absence`. Probe M confirms a real
claim ("Unknown top-level fields are silently ignored.") in `docs/USING.md`
passes. The live corpus currently yields **0 claim lines scanned**, so the gate
has never caught a real claim; the only non-vacuity proof is a synthetic string
chosen *from* the list ("nothing can express a coproduct here"), which pins 1 of
9 phrases and cannot prove the list covers natural phrasing.

Minimal fix (two parts, both small): (a) in
`limitation_claims_name_a_disposition_row`, feed each row's *Claimed limitation*
cell through `CLAIM_GATE_SYNTHETIC` and require rejection — a non-tautological
coverage test that fails today for 8 rows; (b) extend the list to the phrasings
the registry actually uses (`silently ignored`, `is accepted and ignored`,
`no joint check`, `cannot be checked jointly`, `requires hand-picked`,
`only with hand-built`, `is never evaluated`, `cannot observe`). If the team
prefers to keep the list fixed, the spec sentence ("any sentence registering an
expressibility limitation names a disposition row id") must be softened to the
list — the spec is the contract, so (b) is the smaller diff.

Disposition: **must fix (blocking); (a) alone closes the review finding.**

### [MEDIUM-001] "fixing evidence" is shape-checked, not verified as a commit or a committed artifact

Location: `runner.rs:238-253` (`is_fixing_evidence`, `cites_artifact`,
`is_commit_hash`); doc claim at `docs/ref/expressibility-dispositions.md:37-40`
("require `closed` evidence to be a fixing commit hash or a committed artifact
path").

Evidence (executed): probe C — `closed` row with evidence `1a2b3c4` (7 hex
chars, not a commit) **passes**. Probe D — a non-existent path fails (good).
Probe 23 — `cites_artifact` only tests `Path::is_file()`, so the worktree's
`closed` row passes while its artifact
`ledger/probes/naturality/0002-46702d0f.json` is untracked
(`git ls-files --error-unmatch` → error). The doc says *committed*; nothing
checks git.

Minimal fix: `git cat-file -e HEAD:<path>` (or `git ls-files --error-unmatch`)
in `cites_artifact`, and for hashes `git merge-base --is-ancestor <hash> HEAD`;
if git is unavailable, `panic!` (fail closed). Alternatively reword the header to
"commit-shaped token or present file" and drop the word *committed*.

Disposition: **fix** (small).

### [MEDIUM-002] the table's cardinality is duplicated as six literals, and `Report.rows` is the table size, not what was checked

Location: `main.rs:42,56,70,84,98,112` (`report.rows, 12`);
`runner.rs:61-75` (`Report { rows, flipped }`), `:159-202`.

Description: adding a row requires six edits; and the anti-vacuity assertion is
satisfied by the table's size regardless of how many rows a direction visited,
which is why [MEDIUM-003] is invisible.

Minimal fix: derive the expected count (`report.rows == cases::all().len()`) or,
better, add `gap_checked`/`closed_checked` to `Report` and assert
`gap_checked + closed_checked == rows`.

Disposition: **fix** (small).

### [MEDIUM-003] the `closed` direction silently checks zero rows at registration

Location: `runner.rs:180-202`, `main.rs:52-66`.

Evidence (executed): the committed (index) table registers **12 `gap`, 0
`closed`**; probe A (all-gap table) → `regressed_limitation_must_flip_its_row_back`
**passes with 0 rows checked**. The runner header's claim "so the harness
asserts the run was not vacuous" (`:14-16`) is false for that direction. The
mechanism itself is sound: probe B (a `closed` row whose case regressed) fails
naming the row.

Minimal fix: a small `#[test]` exercising `Report::record`/`expect_no_flips`
with a synthetic `(Observation::Limitation, expected Cleared)` pair, i.e. test
the mechanism rather than only the data; plus the per-direction counts from
MEDIUM-002.

Disposition: **deferred** — reason: with 0 `closed` rows the spec scenario's
`WHEN` cannot fire, so nothing is falsely green about the registry; the
mechanism is proven by executed probes B and A2, and the hardening is a
test-infrastructure follow-up that does not change any disposition's truth.

### [MEDIUM-004] `unknown-top-level-fields` registers documented additive-minor policy as a `gap`

Location: table row `unknown-top-level-fields`; contradicts
`crates/catherine-core/README.md:57` ("Unknown top-level or section keys are
ignored…") and `:95-96`, and the negative matrix's explicit adjudication
`crates/catherine-cli/tests/negative_matrix.rs:27-40` ("Unknown keys are
therefore not a matrix row: ignoring them is documented policy (b), not a
fail-open seam").

Description: Decision 5 says the suite records *expressibility* limitations;
this row asks for a diagnostic the project has already decided not to emit, so
"closing" it would reverse a documented compatibility guarantee. Provenance is
also attached to `kimi/…` "unknown `instance_map` probe" — a section that is now
real, which muddies the claim further.

Minimal fix (wording, not scope): restate the claim as the actual expressibility
limit ("an unrecognized key cannot be reported *because* the additive-minor
policy ignores it") and cite the negative-matrix adjudication in the Source
column, so the registry and the policy agree.

Disposition: **deferred** — reason: the row is in the approved proposal's
registered-row list; correcting it is an owner adjudication between two
authoritative documents (registry vs core README + negative matrix), not a
mechanical edit. Recommend recording it in the change's close note.

### [MEDIUM-005] `wrong_shape_instance` is a class-A fail-open (negative-matrix jurisdiction)

Location: row `vacuous-clean-instances`; fixture
`tests/fixtures/gap_suite/wrong_shape_instance.ct.json:8` (`"instance": {"N": […]}`).

Description: the fixture puts rows under an unknown key, so the instance is
empty and the check is vacuous — a "context-mismatched input silently dropped"
seam, i.e. exactly the class `negative_matrix.rs` owns ("NEVER silence"), not an
expressibility limit. The zero-row half (`zero_rows_instance.ct.json`) is
legitimate expressibility (no vacuity diagnostic). Per Decision 5 the two halves
must not share a row.

Minimal fix: keep `zero_rows_instance` in the row and drop the wrong-shape
fixture; file the wrong-shape acceptance as a defect-loop issue / negative-matrix
row.

Disposition: **deferred** — reason: routing requires the owner's Decision-5
adjudication; the observation is truthful either way, so nothing false ships.

### [MEDIUM-006] the table's own summary counts are unverified and already false in the union worktree

Location: `docs/ref/expressibility-dispositions.md:59` ("Counts at registration:
12 rows, 12 `gap`, 0 `closed`"); `CHANGELOG.md:40` ("All twelve rows register
`gap` at this HEAD"); worktree state is 11 `gap` + 1 `closed`
(`git diff` of the naturality cell, sibling change).

Description: nothing derives or checks the summary line, so every flip silently
stales it (the flip protocol in the doc edits only the Disposition cell).

Minimal fix: one assertion in `closed_rows_cite_fixing_evidence` (or
`every_case_is_registered`) recomputing `rows/gap/closed` from the parsed table
and comparing with the `Counts at registration:` line; the CHANGELOG sentence
should be phrased as a point-in-time fact ("at registration") rather than
"at this HEAD".

Disposition: **fix** (small).

### [MEDIUM-007] the proposal's `closed`-row evidence is unfulfilled and unexplained

Location: archived `proposal.md` "Evidence" ("the pattern-lens row registers
`closed` citing the pattern-lens commits"), `design.md:25-26` Decision 2 example
("unknown-section handling was reworked into named additive-section passes; the
pattern lens shipped"), `tasks.md` item 7.

Description: the table registers 12 rows / 0 `closed`. There is no
pattern-lens row (and `docs/ref/pattern-kata-dispositions.md` contains no
`closed` row either), and Decision 2's example is contradicted by the verified
observation (unknown keys are still silently ignored, so `gap` is correct, not
`closed`). Task 7's "whichever the verification pass confirms" permits zero, but
nothing records that the two predicted `closed` rows were considered and
rejected.

Minimal fix: one sentence in the table (or the change's close note) recording
the adjudication: "the pattern-lens and additive-section claims named in the
design were verified to still hold as `gap` / are out of this table's scope".

Disposition: **deferred** — reason: archived contract text; the registry is
truth-verified and Decision 2's operative rule (verify at HEAD) was followed.
The gap is in the audit trail, not the behavior.

### [MEDIUM-008] the gate test never exercises the file-walk finding a real claim

Location: `runner.rs:306-330`; `main.rs:90-108`.

Description: `limitation_claims_name_a_disposition_row` asserts only that the
real run exits 0 and that one synthetic line is rejected/accepted. The real
corpus yields 0 matches, so a regression that drops `docs/**` (or the README/
CHANGELOG loop) from `claim_files` keeps the test green. Probes N and R show the
walk works today.

Minimal fix: fold into HIGH-003(a) — the per-row property test also proves the
walk catches real text; or add one hermetic `docs/`-shaped fixture dir behind a
`CLAIM_GATE_ROOT`-style override (the `CLAIM_GATE_SYNTHETIC` hook already exists).

Disposition: **fix** (fold into HIGH-003).

Correctness quality: **FAIR** — the registry's observations are truthful, but the
flip mechanism can be satisfied by unrelated failures and one gate location is
not implemented at all.

## Pass 3 — CLARITY

The table header states the machine-read contract (`:37-40`), the flip protocol,
and the two dispositions unambiguously; each case file states its row's claim in
its header. Three clarity defects:

- **[LOW-C1] `docs/ref/expressibility-dispositions.md:37-40`**: "parses this
  table's first column (case id) and fifth column (disposition…)" omits the sixth
  column (sealing evidence), which `runner.rs:143-152` parses and enforces
  (`Missing:` prefix, hash/path shape). Minimal fix: name the sixth column.
- **[LOW-C2] `runner.rs:129-133`**: `assert_eq!(cells.len(), 8, "malformed
  disposition row (expected six cells)")` — six *columns* parse as eight split
  parts; the message invites a wrong fix. Minimal fix: "expected 8 pipe-split
  parts for six columns".
- **[LOW-C3] `main.rs`** ends without a trailing newline (verified:
  last byte `}`). Minimal fix: newline.

Clarity quality: **GOOD**.

## Pass 4 — EDGE CASES / negative-input matrix rule

Adjudication of the ro5u checklist rule ("a new flag or surface without a
negative-matrix row is incomplete", catherine-348): **N/A, and complete.**

- No CLI flag, subcommand, envelope field, warning, or error code is added:
  `git diff --cached --name-only | grep -c '^crates/.*/src/'` = **0**. The new
  surface is a dev/CI script plus a `just` recipe; it emits no `catherine`
  output, so no malformed-input class applies to it.
- The gate's own malformed inputs are fail-closed by execution: missing table →
  exit 1; header-only table → exit 1 (`script:46-56`); claim with no row id →
  exit 1; claim naming a registered id → exit 0; malformed/short row → panic
  (`runner.rs:129`); extra `|` in a cell → panic; non-bare disposition → panic
  (`runner.rs:141`); duplicate row id → fail (`runner.rs:285`); row without a
  case → fail (`runner.rs:297`); case without a row → fail (`runner.rs:292`,
  executed: renaming a case id panics with
  `cases without a disposition row: ["coproduct-absence-renamed"]`).
- Table parser edge cases verified: the id column skips exactly the header
  (`Case id`) and separator (`-+`), so prose tables above/below would be parsed
  as rows (fail-closed today: the doc has one table); a row appended to the last
  line without a newline is silently skipped (probe, first attempt) — the file
  currently ends without a trailing newline, which is harmless for `lines()` but
  means `>>` edits can be swallowed; evidence cells with `` ` `` are trimmed;
  a cell containing `|` panics rather than mis-parsing (documented "cells carry
  no `|`").

Edge-case coverage: **GOOD** — the parser fails closed on every malformation I
could construct; the two real edge-case defects are behavioral, not parsing:
[HIGH-001] (`Cleared` catch-all) and [HIGH-002] (close-note location).

## Pass 5 — EXCELLENCE / regression risk

CI wiring verified: recipe `justfile:19-20`, invoked at `justfile:8`, before
`check-wasm` (`:9`); script mode `100755` in the index; the recipe is exercised
end-to-end (`just check-gap-claims` → exit 0). Both CI entry points run `just ci`
(`.github/workflows/ci.yml:41`, `release.yml:186`), so the gate really runs.

Clean-checkout dependency check: the gate needs only `bash`, `awk`, `grep`,
`find` and the committed table — no git, cargo, or generated files. `docs/`
trees it walks are committed; `docs/generated/` is **not** in git
(`git ls-files docs/generated` empty), so its exclusion
(`script:86`) has no effect on a clean checkout and is a local-mirror
convenience — the exclusion is therefore **justified** (it also prevents
double-reporting of regenerated text). The `docs/ref/pattern-kata-dispositions.md`
exclusion is unnecessary today (0 phrase hits) but defensible; the
`expressibility-dispositions.md` exclusion is **necessary** (4 of its own rows
match the phrase list). Remaining residual risk is HIGH-002/HIGH-003 above, not
ordering.

- **[MEDIUM-R1] ordering hazard with the sibling `instance-morphism` change**:
  the worktree's `closed` naturality row cites
  `ledger/probes/naturality/0002-46702d0f.json`, which is **untracked**
  (probe 23), and its cell now cites `naturality_warehouse_break.ct.json`, a file
  that does not exist under `tests/fixtures/gap_suite/` (the case runs
  `naturality_warehouse.ct.json`). The j6g index state (row `gap`, 0 `closed`) is
  unaffected, but if the two changes are committed in the wrong order the suite
  is red on a clean checkout (`is_file()` fails) and the table's cited invocation
  is not reproducible. Disposition: **deferred to the sibling change** — must add
  the ledger artifact in the same commit as the flip, and the `_break` fixture
  name must match what the case runs.
- **[LOW-R2] cross-row coupling**: `composition_seam_claim.ct.json` has
  `"rows": {}, "tables": {}` and the composition case asserts each document
  "checks clean alone" (`cases/future.rs:14-31`). When the `vacuous-clean`
  capability lands as a finding (exit 3), this case will panic rather than flip
  with row guidance. Deferred reason: the coupling is inherent to fixtures whose
  premise is another row's gap; the panic names the document, so diagnosis is
  possible.
- **[LOW-R3] commit hygiene**: `crates/…/expressibility_gaps/cases.rs` (467 lines)
  is staged but deleted on disk; the `cases/` split is untracked. If committed
  without `git add -A`, the index still contains the 467-line file that the
  pretender file-size gate rejects (`tasks.md` item 8). Note for the committer,
  not a code finding.
- **[LOW-R4] message coupling**: cases assert substrings
  (`equation path must be non-empty`, `unexpected argument`,
  `unrecognized subcommand`, `no disposition row id`). This is the same discipline
  as `negative_matrix.rs` and is acceptable; a wording change fails loudly.
- **[LOW-R5] `CLAIM_GATE_SYNTHETIC`** (`script:70-77`) is a bypass env var inside
  a gate; documented and used by the test, but any caller can short-circuit the
  scan. Deferred reason: the test needs a hermetic path and the var is inert
  unless set; a `--synthetic` argument would be marginally safer.
- **[LOW-R6] vendored docs** (`docs/ref/category_theoretic_modeling_methodology.md`)
  are scanned and could produce false positives on mathematical prose ("not
  expressible"); no current hit. Deferred: fixing means a curated exclusion list,
  which is worse than a rare false positive.

Excellence assessment: structure **GOOD**, correctness **FAIR**, clarity
**GOOD**, edge cases **GOOD**, regression risk **LOW-MODERATE** (one ordering
hazard with a sibling change).

---

## Findings summary

| ID | Severity | Location | Disposition |
|---|---|---|---|
| HIGH-001 | high | `cases/probe.rs:41-58`, `migration.rs:39-41`, `analysis.rs` (`naturality_absence`/`coproduct_absence`), `future.rs:51` | must fix — executed proof that a broken fixture + row flip = green suite |
| HIGH-002 | high | `scripts/check-expressibility-claims.sh:92` | must fix — `find openspec -name 'close-note*.md'` = 0 files; real `## Close note` sections unscanned |
| HIGH-003 | high | `scripts/…:35-37,60-66`; `runner.rs:306-330` | must fix — 8/12 registry claims evade the phrase list; 0 real claim lines ever scanned |
| MEDIUM-001 | medium | `runner.rs:238-253` | fix — `1a2b3c4` passes as "commit"; untracked artifact passes as "committed" |
| MEDIUM-002 | medium | `main.rs:42,56,70,84,98,112`; `runner.rs:61-75` | fix — six `12` literals; `rows` is table size, not checked count |
| MEDIUM-003 | medium | `runner.rs:180-202`; `main.rs:52-66` | deferred (reason in finding) — closed direction checks 0 rows at registration; mechanism proven sound |
| MEDIUM-004 | medium | table row `unknown-top-level-fields` vs `crates/catherine-core/README.md:57,95-96`; `negative_matrix.rs:27-40` | deferred (owner adjudication) |
| MEDIUM-005 | medium | row `vacuous-clean-instances`, `wrong_shape_instance.ct.json:8` | deferred (route per Decision 5) |
| MEDIUM-006 | medium | table `:59`; `CHANGELOG.md:40` | fix — counts line unverified, already stale in the union worktree |
| MEDIUM-007 | medium | archived `proposal.md` Evidence; `design.md:25-26` | deferred (audit-trail note) |
| MEDIUM-008 | medium | `runner.rs:306-330` | fix (folds into HIGH-003a) |
| LOW-C1 | low | table `:37-40` | fix (one clause) |
| LOW-C2 | low | `runner.rs:129-133` | fix (message) |
| LOW-C3 | low | `main.rs` EOF | fix (newline) |
| DRAFT-001 | low | `main.rs:8-10` | fix (stale `.espectacular` path) |
| DRAFT-002 | low | `runner.rs:33-38` | deferred (subsumed by HIGH-003a) |
| LOW-R2 | low | `cases/future.rs:14-31` | deferred (cross-row coupling) |
| LOW-R3 | low | staged `cases.rs` vs `cases/` | committer note |
| LOW-R4 | low | case message substrings | accepted |
| LOW-R5 | low | `script:70-77` | deferred (test hook) |
| LOW-R6 | low | `docs/ref/category_theoretic_modeling_methodology.md` | deferred (false-positive risk only) |
| LOW (tautologies) | low | `main.rs:66-76,80-90,94-104,108-118`; `runner.rs:204-235,277-303,364-397` | fix — `flipped.is_empty()` on Reports that are structurally always empty; the real signal is the panic |
| LOW (fixture metadata) | low | `cases/mod.rs:44-46`; `runner.rs:266-275` | fix — `Case.fixtures` duplicates names hardcoded in each `observe` fn and is only existence-checked, so the two can drift |

Counts: **critical 0, high 3, medium 8, low 12.** max_critical = 0 satisfied;
the high findings are unfixed (this review is read-only).

## Convergence

| Check | Stage 2 | Stage 3 | Stage 4 | Stage 5 |
|---|---|---|---|---|
| New CRITICAL | 0 | 0 | 0 | 0 |
| New issues | 12 | 3 | 0 (2 reclassified from pass 2) | 6 |
| vs previous | — | −75% | 0% | +100% (polish pass; no new class) |
| Status | CONTINUE | CONTINUE | CONVERGED on correctness | CONVERGED |

No new critical after pass 2; pass 4 produced no new defect classes (I re-ran
every parser malformation and all fail closed); pass 5's items are risk notes
plus polish. Self-estimated false-positive rate ≈ 5% (one low, LOW-R6, is
speculative about future vendor prose).

## Verdict

**Post-fix verdict: PASS** — all three HIGH findings were fixed before the
j6g commit and every medium is fixed or deferred with a reason (see *Lead
dispositions* at the end). `max_critical = 0` holds.

**As reviewed (pre-fix): FAIL** — 0 critical, but three HIGH findings were
open and had to be fixed before the ticket was done:

1. **HIGH-001** — `Observation::Cleared` accepts any non-claimed outcome, so a
   broken fixture plus a one-cell flip yields a green suite and a false `closed`
   row (executed proof). Fix: assert each cleared signature; panic otherwise.
2. **HIGH-002** — the gate's openspec close-note location matches no file in
   this repo; real close notes are `## Close note` sections in `tasks.md`
   (executed proof). Fix: scan that section.
3. **HIGH-003** — the gate's phrase list misses 8 of the 12 claims the change
   registers and the live corpus yields 0 scanned claim lines (executed proof).
   Fix: feed each row's claim cell through the synthetic path as a test, and
   extend the list.

Mediums are fix-or-defer with the reasons above; the deferred ones (003, 004,
005, 007) are audit-trail/scope adjudications, not behavior.

What I could not verify: the two cited research-corpus conversations (external
to the repo); the pre-sibling binary for the staged `naturality-absence`
observation (the sibling `naturality` subcommand is already in this worktree, so
the staged "exit 2 unrecognized subcommand" cell is consistent-with but not
re-executable at HEAD); and the archived change's `ah check --run-tests`
invocation (the change is archived).

## Lead dispositions (post-review fixes)

Fixed before the j6g commit:

- **HIGH-001** — every case now has an explicit `limitation` and `cleared`
  signature; any third outcome panics naming the case, exit code and stderr.
  Shared classification lives in `cases/probe.rs` (`CheckRun`), and the
  cleared signatures are pinned per case (see the case table in the commit).
  Corrupt-fixture probes panic rather than flipping.
- **HIGH-002** — the gate scans `## Close note` sections of `openspec/**/{tasks,design}.md`
  (level-aware extraction) plus the legacy glob, with a temp-root test.
- **HIGH-003** — the phrase list was extended until all 12 row claims are
  rejected when un-cited; `crates/catherine-cli/tests/expressibility_gate_rows.rs`
  feeds every row's claim cell through the gate in both directions.
- **M1** — evidence accepts a commit hash only when `git cat-file -e
  <hash>^{commit}` resolves, and an artifact path only when
  `git ls-files --error-unmatch` tracks it (index, so the flip commit's own
  artifact validates pre-commit); `evidence.rs` holds the check.
- **M2** — `Report{rows, checked, flipped}`; wrappers assert `checked == rows`
  and `rows > 0` (gap direction) with a single `EXPECTED_ROWS` constant
  cross-checked against `all()`.
- **M6** — the registration-count sentence and the CHANGELOG row-count claim
  now say rows flip with their capability instead of asserting 12/12.

Adjudicated / deferred:

- **M4** (unknown-top-level-fields) — kept as a `gap`, with the row now stating
  that ignoring unknown keys is deliberate additive-minor policy: the row
  registers agent invisibility, not a fail-open defect.
- **M5** (wrong-shaped instance vacuous clean) — correctness, not
  expressibility (Decision 5); filed as **catherine-3en** for the defect loop /
  negative-matrix adjudication. The gap row keeps both fixtures until that lands.
- **M7** (proposal Evidence text promising a `closed` pattern-lens row, and the
  design's registration wording) — the archived proposal is historical; the
  registered rows follow design Decision 2 (HEAD truth). Recorded here, not
  rewritten.
- **M3** (zero `closed` rows at registration) — honest at the registration
  commit; the direction becomes non-vacuous with the first flip
  (`add-instance-morphism`). The wrapper no longer demands `rows > 0` for the
  closed direction; gap direction keeps it. Discovered during commit-level
  verification (`36aa11d`).
- Low findings: accepted or deferred as recorded in the tables above; the
  tautological `flipped.is_empty()` assertions stay because the per-row panic
  carries the signal and the report shape is part of the runner contract.