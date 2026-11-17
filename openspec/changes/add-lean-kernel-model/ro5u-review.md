# Rule of 5 Review — `add-lean-kernel-model`

**Artifact reviewed:** the full uncommitted change on base `16e1492` (beads
`catherine-xvh`): the new `lean/` assurance twin (16 `.lean` files — 15 modules plus
`lakefile.lean` — with `lean-toolchain`, `lake-manifest.json`, `README.md`),
the ledger stop-gate schema (`crates/catherine-ledger/src/{entry,lib,validate,decision}.rs`,
`src/bin/ledger_record.rs`, `tests/{decision_entry,ledger_record_seal}.rs` and
seven mechanical `decision: None` test updates), `justfile`,
`.github/workflows/ci.yml`, `scripts/check-lean-axioms.sh`, `.gitignore`,
`ledger/lean/0001-cbef9219.json` (re-sealed from `0001-c1e3d39b.json` in the
fix wave), the change directory, and the implementor brief.

**Mode:** rule-of-5-universal (DRAFT → CORRECTNESS → CLARITY → EDGE CASES →
EXCELLENCE), convergence checks after stages 2, 3, and 4.
**Date:** 2026-11-17. **Reviewer:** ro5u gate (read-only).
**Re-verification:** after the fix-up wave, every check was re-run and every
finding re-checked (17:45–17:55 -03, including the EDGE-003 fix re-check); the five stages below are the original
review record, and the post-fix disposition of each finding is in
*Re-verification after fixes* near the end.

**Concurrent-write caveat (material to reproducibility).** Two fix-up waves
edited the tree while this review ran. The first (`proc://LedgerSplitModules`)
split `crates/catherine-ledger` (new `src/decision.rs`), added the missing final
newline to `bin/ledger_record.rs`, and brought the pretender gate from red to
green. The second wave (the dispositions in *Re-verification after fixes*)
rewrote the axiom audit, expanded `Axioms.lean`, strengthened the dangling
clause, and corrected the change record. The pre-fix snapshot below pins the
revisions reviewed in the original pass; the post-fix snapshot pins the
revisions re-verified in the second. Neither pass modified a repository file —
the only file this review writes is this report.

## Verification actually run

| # | Command | Observed result |
|---|---|---|
| 1 | `cd lean && lake build` | `Build completed successfully (24 jobs)`, exit 0 (`lake --version` = Lake 5.0.0-src, Lean 4.36.0, matching `lean/lean-toolchain`) |
| 2 | `lake exe lean-parity` | exit 0 — `total=190 rust_checkable=180 skipped_parse=10 lean_only_presentation=1 trivially_agreeing=102 findings_bearing_agreeing=79 divergences=0 build_mismatches=0` |
| 3 | `lake exe lean-parity --check-build` | exit 0 — same counts, `build_mismatches=0` |
| 4 | `just lean` | exit 0 — build + oracle + axiom audit |
| 5 | `lake env lean Catherine/Axioms.lean` | exit 0 — `soundness`, `equation_completeness`, `dangling_discipline`: `depends on axioms: [propext, Classical.choice, Quot.sound]`; `labelFor_agrees`: `does not depend on any axioms` |
| 6 | `bash scripts/check-lean-axioms.sh` | exit 0 — `✅ check-lean-axioms: no sorryAx and no axiom declarations in Catherine/Axioms.lean` |
| 7 | `cargo test -p catherine-ledger` | exit 0 — all suites green, incl. `decision_entry` (7 passed) and `ledger_record_seal` (5 passed) |
| 8 | `./target/debug/ledger-check ledger/lean` | exit 0 — `ok: 1 entries (0 kata), rollup verdict: (no rollup)` |
| 9 | `./target/debug/ledger-check ledger/probes/coproduct` | exit 0 — `ok: 3 entries (2 kata)` |
| 10 | `openspec validate add-lean-kernel-model --strict` | `Change 'add-lean-kernel-model' is valid`, exit 0 |
| 11 | `ah check` | `no issues — 0 structural, 0 execution`, exit 0 |
| 12 | `pretender check --mode gate` | **exit 1 at 17:22** (5 violations: `entry.rs` 423>400, `validate.rs` 444>400, `integrity_violations` cognitive 17>15 and 62>40 lines, `rollup_violations` 44>40); **exit 0 at 17:34:41** after the concurrent split (0 violating files). Recorded, not an open finding. |
| 13 | `rustfmt --check --edition 2021` on every changed ledger Rust file | All OK at 17:33. Before the concurrent write, `bin/ledger_record.rs` had no final newline and rustfmt reported `Diff in ...: +` (a `cargo fmt --check` failure); the concurrent task fixed it before this report. |
| 14 | Out-of-tree probe `lake env lean /tmp/axcheck.lean` on `axiom cheat : False` / `theorem t : False := cheat` / `#print axioms t` | exit 0, output exactly `'t' depends on axioms: [cheat]` — Lean echoed **no** declaration line. Evidence for CORR-001. |
| 15 | `git status --porcelain=v1 -uall` before/after | Same author-owned set; the only addition from this review is this report. |

**Legend.** **CONFIRMED** = reproduced by a command above or decided by direct
inspection of the cited lines (deterministic). **UNVERIFIED** = TypeSafe did not
verify it (below the skill's `VERIFY_SEVERITIES` cost cap) and it is not
reproducible in a read-only review environment.

### Post-fix re-verification (2026-11-17, 17:45–17:55 -03)

Every row was re-run from the tree after the fix-up wave; the counts did not
move.

| # | Command | Observed result |
|---|---|---|
| 16 | `cd lean && lake build` | `Build completed successfully (24 jobs)`, exit 0 |
| 17 | `lake exe lean-parity` | exit 0 — `total=190 rust_checkable=180 skipped_parse=10 lean_only_presentation=1 trivially_agreeing=102 findings_bearing_agreeing=79 divergences=0 build_mismatches=0` |
| 18 | `lake exe lean-parity --check-build` | exit 0 — same counts, `build_mismatches=0` |
| 19 | `just lean` | exit 0 — build + findings oracle + build-report oracle + hardened audit |
| 20 | `bash scripts/check-lean-axioms.sh` | exit 0 — “✅ check-lean-axioms: 18 audited declaration(s) use only {propext, Classical.choice, Quot.sound}; no `sorry` in the build; no axiom declarations under lean/Catherine” |
| 21 | `lake env lean Catherine/Axioms.lean` | exit 0 — 18 lines; every set ⊆ `{propext, Classical.choice, Quot.sound}`; five declarations axiom-free; `dangling_discipline` still `[propext, Classical.choice, Quot.sound]` |
| 22 | Self-test `--file /tmp/self_cheat.lean` (`axiom cheat` + use) | exit 1 — `'t' depends on disallowed axiom 'cheat'` **and** `axiom declaration at /tmp/self_cheat.lean:1` |
| 23 | Self-test `--file /tmp/self_sorry.lean` (`by sorry`) | exit 1 — `'t2' depends on disallowed axiom 'sorryAx'` |
| 24 | Self-test `--file /tmp/self_empty.lean` (no `#print axioms`) | exit 1 — `no '#print axioms' output … the audit asserts nothing` |
| 25 | Self-test `--file $PWD/lean/Catherine/Axioms.lean` | exit 0 — the hardened audit passes on the committed file |
| 26 | `grep -rn sorry lean/Catherine` | only `Axioms.lean` documentation lines (6, 20, 26, 28); no proof uses `sorry` |
| 27 | `grep -rn '^theorem' lean/Catherine/Proofs/*.lean` | 18 public theorems; every one has a `#print axioms` line in `Axioms.lean` |
| 28 | `cargo test -p catherine-ledger` | exit 0 |
| 29 | `cargo run -q -p catherine-ledger --bin ledger-check ledger/lean` | exit 0 — `ok: 1 entries (0 kata), rollup verdict: (no rollup)` |
| 30 | `pretender check --mode gate` | exit 0 |
| 31 | `openspec validate add-lean-kernel-model --strict` | `Change 'add-lean-kernel-model' is valid`, exit 0 |
| 32 | `ah check` | `no issues — 0 structural, 0 execution`, exit 0 |
| 33 | `elan --version`; `curl -I …/elan/v4.2.4/elan-init.sh` | `elan 4.2.4`; `HTTP=200` (the pinned installer tag exists) |
| 34 | `sha256sum openspec/specs/{kernel-checks,findings}/spec.md` | `4d0aab91…`, `754ea474…` — exactly the decision entry's `model_hashes` |
| 35 | From `/tmp`: `--file rel-audit.lean` (clean; imports `Catherine.Proofs`) | exit 0 — `1 audited declaration(s) use only {propext, Classical.choice, Quot.sound}` |
| 36 | From `/tmp`: `--file rel-cheat.lean` (`axiom cheat2`) | exit 1 — `'w' depends on disallowed axiom 'cheat2'` **and** `axiom declaration at /tmp/rel-cheat.lean:1:axiom cheat2 : False` |
| 37 | From `/tmp`: absolute `--file /tmp/rel-audit.lean`; `--file /tmp/rel-cheat.lean` | exit 0; exit 1 (`proofs are not sorry-free / assumption-free`) |
| 38 | From `/tmp`: `--file=rel-audit.lean`; `--file no-such.lean` | exit 0; exit 2 with `(--file was given explicitly; check the path.)` |
| 39 | Repo root, default (no `--file`); `--help` | exit 0 — `18 audited declaration(s) …`; exit 0 |
| 40 | After the EDGE-003 fix: `just lean`, `pretender check --mode gate`, `openspec validate … --strict`, `ah check` | all exit 0; `just lean` still reports `divergences=0`, `build_mismatches=0`, and 18 audited declarations |

## TypeSafe verification

`TYPESAFE_API_KEY` was set; one Stage-B request was sent
(`POST https://api.typesafe.ai/v1/systemone`, HTTP 200, model `jev-1.14.0`) for
the single HIGH finding (CORR-001): verdict **`verified`**, confidence **0.29**
— below the skill's `CONFIDENCE_GATE` of 0.8, so the finding is kept and marked
**REVIEW_REQUIRED** (routed to human review). MEDIUM/LOW findings are below
`VERIFY_SEVERITIES` and are **UNVERIFIED** by TypeSafe by design. The API was
reachable, so the skill's "API unavailable" banner does not apply. No
`[META-*]` finding was generated (Stage A locations all exist). After the
fix wave no CRITICAL/HIGH finding remains, so no additional Stage-B request was
made; the post-fix dispositions are command-backed instead.

## Snapshot (finding anchors)

**Pre-fix (original pass), `sha256` computed at 17:39 -03.** These are the
revisions the findings below were raised against; several have since changed.
(`ledger/lean/0001-c1e3d39b.json` has since been re-sealed as
`0001-cbef9219.json`.)

```
b6c699790ac12e750cc945e819e964dce8b897819c7395e22f0a779ef0cc7bcb  scripts/check-lean-axioms.sh
016c27130c9eab7152c40da82317c2f38ee63d4b8478410ad7fe39da95d3531a  lean/Catherine/Axioms.lean
89303a8d3ec3ff16159525566f6944c40b60d553b86382552ad226fe77c3cf1f  lean/Catherine/Proofs/DanglingDiscipline.lean
ddcbc5a0db8aa143eebdef8ceced2321b95c211680a2137f0c2e19d2387bcfce  lean/Catherine/Proofs/Soundness.lean
f8358b4dda227f4bfbdf1bcf8060a51f4008281b1f4bcb6acf8aa236fb266538  openspec/changes/add-lean-kernel-model/tasks.md
ea9ec69bad33de857704fd008ed1f361fb79a03b03623d3f4c25f051d7fffc7f  openspec/changes/add-lean-kernel-model/proposal.md
293e499f1da2ddfb6933fb8ade9332b082dcdd4a33e829fbca63b4d2969763ba  openspec/changes/add-lean-kernel-model/design.md
e5879e775f4728fd24ad8f99b4a10cc25bdf5853b15c28efde626bce06c6241e  .github/workflows/ci.yml
1eba98b0fdd2c1de006b73d325dd1bd82f1b430fea0101e34aa6bb385903c2bf  justfile
9b14c5455c4446af9db45fe4b3e62d656f08eccec18112b491a1061ea0492806  ledger/lean/0001-c1e3d39b.json
6aaca477aba201828de2eb2ccd96fbc9ba66f5cc1b16cf54c03e85f4258df27d  crates/catherine-ledger/src/decision.rs
a276ca6d6dae72f9c48c3168651ff27bbc9490050573a5aededaaf49c3aa905d  crates/catherine-ledger/src/entry.rs
43b805ce0e7e3e5f992832aa46aa3b220c8be1d92d892c4395e279e190d6df44  crates/catherine-ledger/src/validate.rs
```

**Post-fix (re-verification), `sha256` computed at 17:53 -03.**

```
92b08be595c473d400397b5bd028b1966cacb150757c90d5f2cbeef10eec2a05  scripts/check-lean-axioms.sh (EDGE-003 fix)
7b986780643830f6c9ccf57f4df65c60a31cfb77e6f9c8ff073251354e7fabb4  lean/Catherine/Axioms.lean
86da854f3bb868585660136c8b5594476e5eae748000afe4c6971d36f9a5b723  lean/Catherine/Proofs/DanglingDiscipline.lean
ddcbc5a0db8aa143eebdef8ceced2321b95c211680a2137f0c2e19d2387bcfce  lean/Catherine/Proofs/Soundness.lean (unchanged)
bb623ef0ae08f2f04fa731d69ae999b7c13d6dc90011062f87b779ca213b85df  openspec/changes/add-lean-kernel-model/tasks.md (task 14 ticked)
a87f0970af72f90cca3a2b607a0fd4564e5784b5a36f4580cce632cefda3a96f  openspec/changes/add-lean-kernel-model/proposal.md
65aaa5ccb14c9226a3056b5abb2bbb0bc86746cef23dae2af5568951bfd5768b  openspec/changes/add-lean-kernel-model/design.md
1392fc6696b5a51f62651ef0e149f3f9db9a0593dab1840965d9bf879f067cbf  .github/workflows/ci.yml
35cbe983592cb56497a10bac467841bc85fae7feb260b1f05696b563e12a98d1  justfile
ebe55f30ad531dff75c14268b842c28200b81b50adfc122e4c6eea5457b1b592  ledger/lean/0001-cbef9219.json
6aaca477aba201828de2eb2ccd96fbc9ba66f5cc1b16cf54c03e85f4258df27d  crates/catherine-ledger/src/decision.rs (unchanged)
a276ca6d6dae72f9c48c3168651ff27bbc9490050573a5aededaaf49c3aa905d  crates/catherine-ledger/src/entry.rs (unchanged)
43b805ce0e7e3e5f992832aa46aa3b220c8be1d92d892c4395e279e190d6df44  crates/catherine-ledger/src/validate.rs (unchanged)
8b35e6758b01349fbfa751c6017307a1e4bad042636b564beed537ec6f78c3e0  lean/README.md (gate paragraph updated)
```

File mtimes identify the exact revisions in each pass; the concurrent-write
caveat above explains the drift between them.

---

> **Note.** The five stages below are the original review pass (pre-fix). Their
> findings stand as the review trail; every disposition — FIXED, DEFERRED, or
> OPEN — is recorded in *Re-verification after fixes*, which follows the stage
> record.

---

## STAGE 1: DRAFT

**Assessment.** The change has the right shape for an assurance artifact: a
standard-library-only Lean twin that mirrors the Rust kernel 1:1, a parity
oracle that compares normalized finding *multisets* over a dynamically
enumerated corpus, three frozen theorems with a machine-checkable axiom audit, a
CI gate, and a recorded stop-gate decision on the ledger. The Rust footprint is
policy-shaped (a new closed `decision` entry kind), not kernel surgery: the
kernel sources are untouched, so the parity evidence is unchanged. The main
structural drift is in the change record itself (DRAFT-001/DRAFT-002), not in
the executable surfaces.

### DRAFT-001 (MEDIUM) — `tasks.md` still records the proof and CI phases as unfinished

**Location:** `openspec/changes/add-lean-kernel-model/tasks.md:81-90` (tasks
8–11), `:108` (task 13).

**Description.** Tasks 8, 9, 10 (the three proofs), 11 (CI wiring), and 13 (the
Lean README) are still `- [ ]`, although the verified state contradicts them:
`bash scripts/check-lean-axioms.sh` exits 0 with no `sorryAx` and
`lake env lean Catherine/Axioms.lean` prints only the three standard axioms
(proofs landed); `.github/workflows/ci.yml:48-59` plus `justfile:39-42` are the
CI wiring; `lean/README.md` exists. The brief's completion criterion 5 requires
"`tasks.md` 14/14 ticks", so the change record as committed would misstate its
own completion state (an archiver/human reads this file as the phase record).

**Evidence.** Tasks 8–11 and 13 are unchecked in the cited lines; the five
landed artifacts are cited above with their green commands. Task 14's `bd`
half is orchestrator-reserved, so it is legitimately unchecked.

**Recommendation.** Tick tasks 8, 9, 10, 11, and 13 with the evidence already
written for tasks 6/7 (the command and the counts), or add a one-line status
note to each. Leave task 14 for the close step.

### DRAFT-002 (LOW) — `proposal.md` Impact claims "new `lean/` tree only" while the change extends the ledger schema

**Location:** `openspec/changes/add-lean-kernel-model/proposal.md:56-58`;
`design.md:81`.

**Description.** The proposal says "Affected code: new `lean/` tree only; the
Cargo workspace gains no dependency on Lean and vice versa. Rust changes only if
parity finds kernel defects", and Decision 5 assumed a decision "entry under the
existing `ledger/` entry schema". The delivered change instead adds a new
`decision` kind, a new `decision.rs` module, `seal_parsed`/`declared_id`, a new
binary path, and two integration suites in `catherine-ledger`. The extension is
justified (the existing schema had no decision kind), but the Impact section and
Decision 5 are now wrong about the shape of the change and omit the ledger
contract from the declared blast radius.

**Evidence.** `git diff --stat` shows 11 modified Rust files plus two new test
suites and `src/decision.rs`; the proposal text above still asserts the
opposite. `ledger-check ledger/lean` validates the new entry kind.

**Recommendation.** Amend the Impact bullet to name the ledger schema extension
(`catherine-ledger`: new `decision` entry kind + authoring seam) and update
Decision 5's "existing entry schema" phrasing to "a new terminal `decision`
kind on the ledger schema".

**Shape Quality:** GOOD.

---

## STAGE 2: CORRECTNESS

### CORR-001 (HIGH) — The axiom audit cannot detect an unexpected axiom dependency

**Location:** `scripts/check-lean-axioms.sh:61-70` (the only `sorryAx` grep) and
`:76-92` (the `axiom`-declaration scan); claim repeated in `lean/README.md`
("so a proof claim cannot be accepted on trust") and required by the brief as
"`lake env lean Catherine/Axioms.lean` prints no `sorryAx` (and no unexpected
axioms)".

**Description.** The audit fails only when (a) a `#print axioms` line contains
the literal `sorryAx`, or (b) an `axiom NAME` *declaration line* appears in the
elaboration output or in `Axioms.lean`'s source. Lean does not echo source
declarations in elaboration output, and a `#print axioms` line names the axiom
bare (`[cheat]`), never with an `axiom` prefix. Therefore a proof that depends
on a user-declared axiom **in any other module** (e.g.
`Catherine/Proofs/Soundness.lean`) passes the gate: the offending name appears
in the axiom list, which neither check matches.

**Evidence (CONFIRMED, reproduced).** Out-of-tree probe
`/tmp/axcheck.lean` containing `axiom cheat : False`, `theorem t : False :=
cheat`, `#print axioms t`, elaborated with `lake env lean` from `lean/`, prints
exactly:

```
't' depends on axioms: [cheat]
```

and exits 0 — no declaration line is echoed. The two script loops
(`grep -F 'sorryAx'`; `awk '/^[[:space:]]*axiom[[:space:]]+[A-Za-z_]/'`) match
neither that line nor a source scan of `Axioms.lean`. The comments at
`scripts/check-lean-axioms.sh:44-45` and `:73-75` ("any declaration the source
carries is echoed by the compiler as a line starting with `axiom`") are false,
which is why the second loop is dead code. TypeSafe: `verified` at confidence 0.29 →
**REVIEW_REQUIRED**.

**Recommendation.** Make the gate assert the axiom *set*, not a substring:

- Parse every `depends on axioms: [..]` line and require each listed name to be
  in an allowlist `{propext, Classical.choice, Quot.sound}` (a `sorryAx`
  dependency then fails automatically, and an unexpected name is named).
- Scan the source of **all** `lean/Catherine/**/*.lean` for
  `^\s*(private\s+)?axiom\s` (not only `Axioms.lean`), so a declaration cannot
  hide in a proof module.
- Delete the false "declarations are echoed" rationale and state the real
  mechanism in the header.

### CORR-002 (MEDIUM) — The audit covers only four declarations, so other proofs can regress to `sorry`

**Location:** `lean/Catherine/Axioms.lean:25-28`;
`lean/Catherine/Proofs/Soundness.lean:670,692,713,735`.

**Description.** `Axioms.lean` issues `#print axioms` for `soundness`,
`equation_completeness`, `dangling_discipline`, and `labelFor_agrees` only. The
four exclusion theorems that the stop-gate entry itself lists as landed proofs
(`ledger/lean/0001-c1e3d39b.json`, `decision.proofs`: `check_no_naturality`,
`check_no_redundant`, `check_no_contradiction`, `check_no_underconstraint`) are
never audited, and neither is any other public theorem. `lake build` accepts a
`sorry` with a warning, and `just lean` does not treat warnings as failures, so
a `sorry` in one of those declarations leaves the gate green.

**Evidence (CONFIRMED, inspection).** The audit file names four declarations;
the four exclusion theorems are public and un-audited; the ledger entry cites
all four as proofs. No build-warning grep exists in `justfile:39-42` or the
script.

**Recommendation.** Add one `#print axioms` line per public theorem in
`Catherine/Proofs/*.lean` (or a generated audit list), or fail the gate on any
`declaration uses 'sorry'` line in `lake build` output.

### CORR-003 (MEDIUM) — `no_violation_through_unresolved` proves only one short-circuit orientation, not the documented contract

**Location:** `lean/Catherine/Proofs/DanglingDiscipline.lean:77-91` (docstring
and statement), `:760` (proof).

**Description.** The clause's docstring says "An unresolved equation path emits
no violation: when one composite short-circuits to `none`, `check` carries no
equation finding for that index and row", and `lean/README.md` repeats it ("an
equation path that short-circuits on a dangling or typing-violated cell emits no
violation"). The formal statement only covers `lhs` *resolving* and `rhs`
returning `none`:

```
EvalPath.compose ... equation.lhs row = some lhs →
EvalPath.compose ... equation.rhs row = none →
```

The mirror case (`lhs = none`, `rhs = some _`) and the both-`none` case are not
stated, so the bundle does not prove the spec scenario
(`kernel-checks/spec.md`, "Dangling arrow value": equations traversing the
dangling arrow "report the affected rows as not-evaluable-through-dangling
rather than as violations") for an unresolved *lhs* path. The kernel behaves
correctly in all three cases (`Check.equationFinding` returns `none` when either
composite is `none`); the gap is in what the theorem certifies.

**Evidence (CONFIRMED, inspection).** The cited hypothesis set fixes the `lhs`
composite to `some`; the docstring's "one composite" quantifies over both.

**Recommendation.** State the symmetric form —
`compose lhs row = none ∨ compose rhs row = none → no equation finding for
(index,row)` — or add the two missing orientations as separate clauses. The
existing proof is already the hard half.

**Correctness Quality:** GOOD — the executable Lean/Rust semantics agree on the
whole corpus (180 compared documents, 0 divergences, 0 build mismatches), and
the three theorems are sorry-free; the defects are in the *assurance gate* and
in one statement's coverage, not in `check`.

```
Convergence check (after Stage 2)
New CRITICAL issues: 0
Total new issues: 3
New issues vs Previous Stage: +50% (2 → 3)
Estimated false positive rate: 0%
Status: CONTINUE
```

---

## STAGE 3: CLARITY

### CLAR-001 (LOW) — `design.md` describes the mirror input as the `expected.json` shape, not the implemented `{rows, values}` shape

**Location:** `openspec/changes/add-lean-kernel-model/design.md:27-33`.

**Description.** Decision 2 says the Lean input interface is "a concrete format
that already exists in-repo: the normalized 'mirror' presentation of
`fixtures/parity/*/expected.json` (`tables`, `dangling`, `typing_violations` per
arrow) plus the schema section — the oracle emits exactly that JSON". The
implemented mirror is a different document: `Catherine/Mirror.lean` `toJson`
emits `{fixture, schema, instance: {rows: {object: count}, values: {arrow:
[option<rowId>]}}}` (the brief's §3.1 shape). A reader who trusts `design.md`
will look for `tables`/`dangling`/`typing_violations` in `--emit-mirror` output
and not find them; the `expected.json` files are the
`--check-build` comparison inputs and the presentation-route outcomes, not the
mirror the Lean twin consumes.

**Evidence (CONFIRMED, inspection).** `design.md:27-33` vs
`lean/Catherine/Mirror.lean` `toJson` (`instance` = `rows` counts + `values`
cells) and the brief §3.1 JSON example. `--emit-mirror` writes that shape.

**Recommendation.** Rewrite the sentence to name the actual mirror schema
(`{fixture, schema, instance:{rows, values}}`) and describe `expected.json` as
the `--check-build` oracle input, not the Lean interface.

**Clarity Quality:** GOOD.

```
Convergence check (after Stage 3)
New CRITICAL issues: 0
Total new issues: 1
New issues vs Previous Stage: -67% (3 → 1)
Estimated false positive rate: 0%
Status: CONTINUE (33% of the previous stage's issues are still new, above the 10% bar)
```

---

## STAGE 4: EDGE CASES

### EDGE-001 (LOW) — The elan install step is unpinned and its failure is masked by the shell

**Location:** `.github/workflows/ci.yml:48-55`.

**Description.** The step fetches `raw.githubusercontent.com/leanprover/elan/master/elan-init.sh`
— a moving branch — while the workflow header states every tool pin comes from
`versions.ddl.toml` and "never `latest`", and the change's own provisioning
rationale is reproducibility. Separately, the step uses the workflow's
*unspecified* default shell (`bash -e {0}`, no `pipefail`), so `curl … | sh`
returns `sh`'s status: a failed download yields a successful install step and
the failure surfaces later inside `just lean` as "lake not found", pointing at
the wrong cause.

**Evidence (CONFIRMED, inspection).** `ci.yml:53` is the `master` URL;
GitHub's documented default for a step without `shell:` on Linux/macOS is
`bash -e {0}` (pipefail only applies when `shell: bash` is named explicitly).
The step has no `shell:` and no `set -o pipefail`.

**Recommendation.** Pin the installer (a release tag/commit URL or an
`ELAN_VERSION`-style pin recorded with the other versions) and add
`set -o pipefail` (or `shell: bash`) so a failed bootstrap fails the step that
performed it.

### EDGE-002 (LOW) — The decision entry's `model_hashes` name no document and are unauditable

**Location:** `ledger/lean/0001-c1e3d39b.json` (`model_hashes`);
`crates/catherine-ledger/src/entry.rs:278-279` ("The model-document hashes this
entry validated (fixture + probes)").

**Description.** A `decision` entry validates no model document (it records a
gate), yet `integrity_violations` requires every entry to carry a non-empty
`model_hashes` list and the new kind adds no field naming what those hashes
address. The entry carries two bare hashes
(`sha256:4d0aab91…`, `sha256:754ea474…`) with no fixture path, corpus
reference, or rationale tying them to the parity evidence they are meant to
anchor (kata entries escape this by carrying `kata.fixture`/`fixture_hash`).
Anyone re-deriving the stop gate cannot tell which documents were hashed.

**Evidence (CONFIRMED for the structural gap; UNVERIFIED for provenance).** The
entry JSON has no document names; `MissingModelHashes` applies to every kind
(`tests/decision_entry.rs::decision_without_model_hashes_is_refused`); the
test's placeholder (`sha256:lean-coproduct-pair`) shows the field is
free-form. I scanned every `*.ct.json`/`expected.json` byte hash and found no
match, but canonical model hashes are not file hashes, so the cited values
remain untraceable rather than disproved.

**Recommendation.** Either cite the documents (`{"path": …, "hash": …}`) or say
in the decision record which corpus documents the hashes cover; alternatively
make `model_hashes` optional for terminal decision entries and put the evidence
anchors in the decision payload where they can be named.

**Edge Case Coverage:** GOOD.

```
Convergence check (after Stage 4)
New CRITICAL issues: 0
Total new issues: 2
New issues vs Previous Stage: +100% (1 → 2)
Estimated false positive rate: 0%
Status: CONTINUE
```

---

## STAGE 5: EXCELLENCE

### EXCL-001 (LOW) — Finding-membership bridge lemmas are duplicated across proof modules

**Location:** `lean/Catherine/Proofs/Soundness.lean:69,114,487,547` and
`lean/Catherine/Proofs/DanglingDiscipline.lean:716,724` (plus the grouping
invariants private to `Soundness.lean`).

**Description.** Both proof modules re-derive membership characterizations of
the same walk (`danglingFindings`, `equationFindings`,
`coproductDeclaration`/`coproductFindings`) as `private` lemmas; the ledger
entry's own rationale records this ("the finding-membership bridges are
duplicated as private lemmas across Soundness and DanglingDiscipline … recorded
for a consolidation pass"). It is maintainability debt, not a correctness
defect, and the brief's REFACTOR phase explicitly asked for deduped lemmas.

**Evidence (CONFIRMED, inspection).** The two files define parallel private
membership lemmas over the same `Check` functions.

**Recommendation.** Move the shared membership lemmas (and the
`GroupGood`/`insertRow`/`groupedRows` invariants) into a
`Catherine/Proofs/Membership.lean` imported by both, and keep only family-specific
proof bodies in each module.

### EXCL-002 (LOW) — CI runs the findings oracle but never the build-report oracle

**Location:** `justfile:39-42`; `.github/workflows/ci.yml:57-58`.

**Description.** `just lean` runs `lake build`, `lake exe lean-parity`, and the
axiom audit, but not `lake exe lean-parity --check-build`. The build-report half
of parity (Lean `Instance.build` tables, dangling cells, typing violations
against `expected.json`) is therefore exercised only by hand, even though it
exits 0 on the current tree (command 3) and the README/tasks report it as part
of parity green. A regression in the builder's three landing places that leaves
findings unchanged would not be gated.

**Evidence (CONFIRMED, inspection).** `justfile:39-42` lacks `--check-build`;
`lake exe lean-parity --check-build` exits 0 with `build_mismatches=0`.

**Recommendation.** Add `cd lean && lake exe lean-parity --check-build` to the
`lean` recipe (the run costs about a second and is already green).

**Excellence assessment:**

- Structure: GOOD
- Correctness: GOOD (one HIGH gate defect)
- Clarity: GOOD
- Edge Cases: GOOD
- Overall: GOOD

**Production Ready:** NO — CORR-001 must be fixed before this change is merged;
CORR-002 and CORR-003 should be fixed in the same pass.

```
Convergence check (after Stage 5)
New CRITICAL issues: 0
Total new issues: 2
New issues vs Previous Stage: 0% (2 → 2)
Estimated false positive rate: 0%
Status: CONVERGED at Stage 5 (no CRITICAL reached the report; the remaining
items are LOW polish)
```

---

# Re-verification after fixes

**Pass date:** 2026-11-17, 17:45–17:55 -03. **Trigger:** the fix-up wave
completed. Every claim below was re-derived from the tree with the commands in
*Post-fix re-verification* above; nothing was accepted from the implementation
report on trust. **Superseded verdict:** the original pass returned
`NEEDS_REVISION` (1 HIGH, 3 MEDIUM); the pass below supersedes it.

## Per-finding disposition

| ID | Severity | Status | Evidence (independently verified) |
|---|---|---|---|
| DRAFT-001 | MEDIUM | **FIXED** | `tasks.md:81,89,98,106,128` — tasks 8, 9, 10, 11, and 13 are `[x]` with per-task evidence (axiom sets, file names, CI wiring, README scope). Task 14 is now `[x]` too (`tasks.md:137` — `openspec validate --strict` valid, `bd` closed + export refreshed); `grep -c '^- [x]'` = 14 and the unchecked count is 0. |
| DRAFT-002 | LOW | **FIXED** | `proposal.md:56-62` now names "a `catherine-ledger` schema extension … a new terminal `decision` entry kind, its validator rules, and the `ledger-record` sealing path"; `design.md:86` says "a new terminal `decision` kind on the `ledger/` entry schema". |
| CORR-001 | HIGH | **FIXED** | `scripts/check-lean-axioms.sh:120-135` (build + `declaration uses …sorry…` scan), `:152-185` (per-declaration axiom-set assertion against `{propext, Classical.choice, Quot.sound}`), `:211-224` (source scan of every `lean/Catherine/**/*.lean` for `axiom`). Re-ran: the committed audit exits 0 with 18 declarations; the two former bypasses now fail — `--file /tmp/self_cheat.lean` exits 1 naming `disallowed axiom 'cheat'` **and** the declaration at `:1`; `--file /tmp/self_sorry.lean` exits 1 naming `disallowed axiom 'sorryAx'`; an empty audit exits 1. The `--file` seam it exposed is fixed and re-verified (EDGE-003). |
| CORR-002 | MEDIUM | **FIXED** | `lean/Catherine/Axioms.lean:35-61` issues 18 `#print axioms` lines (7 `Basic`, 5 `Soundness`, 5 `EquationCompleteness`, 1 `DanglingDiscipline`); `grep -rn '^theorem' lean/Catherine/Proofs/*.lean` returns exactly those 18 public theorems, and `lake env lean Catherine/Axioms.lean` prints 18 lines, all within the allowlist. |
| CORR-003 | MEDIUM | **FIXED** | `DanglingDiscipline.lean:84-93` is now `∀ row, (compose … equation.lhs row = none ∨ compose … equation.rhs row = none) → … → ¬ (witness.equation = index ∧ witness.row = row)`; the proof at `:762-789` case-splits the disjunction. `lake build` is green and `#print axioms Catherine.Proofs.dangling_discipline` is still `[propext, Classical.choice, Quot.sound]`. |
| CLAR-001 | LOW | **FIXED** | `design.md:21-40` now describes `{fixture, schema, instance: {rows, values}}` with the sorted-row-name/dense-cell rules and states "`fixtures/parity/*/expected.json` is not the mirror: it is the `--check-build` comparison input and the presentation-route outcome". |
| EDGE-001 | LOW | **FIXED** | `.github/workflows/ci.yml:48-63` names elan v4.2.4, uses `shell: bash` + `set -euo pipefail`, downloads to `/tmp` before `sh`, and pins the URL to `…/elan/v4.2.4/elan-init.sh`. Verified: `elan --version` = `elan 4.2.4`; `curl -I` on the pinned URL → `HTTP=200`. |
| EDGE-002 | LOW | **FIXED** | `ledger/lean/0001-cbef9219.json` `decision.scope` names the two documents; its `model_hashes` are exactly `sha256sum openspec/specs/kernel-checks/spec.md` (`4d0aab91…`) and `sha256sum openspec/specs/findings/spec.md` (`754ea474…`), verified; `ledger-check ledger/lean` exits 0; `lean/README.md:230` and `tasks.md` point at the new file. |
| EXCL-001 | LOW | **DEFERRED** | Not changed by design (see below). Reason recorded: private, single-file bridges; consolidating would touch three verified proof modules for no semantic gain; tracked as future proof-tree cleanup. |
| EXCL-002 | LOW | **FIXED** | `justfile:45` adds `cd lean && lake exe lean-parity --check-build`; `just lean` exits 0 and the run reports `build_mismatches=0`. |
| EDGE-003 | LOW | **FIXED** | `scripts/check-lean-axioms.sh:85-87` resolves an explicit relative `--file` against the invocation `$PWD` before `cd lean/`. Reproduced from `/tmp`: `--file rel-audit.lean` → exit 0 (1 audited declaration); `--file rel-cheat.lean` → exit 1 naming `disallowed axiom 'cheat2'` **and** `axiom declaration at /tmp/rel-cheat.lean:1`; absolute paths → 0/1; `--file=rel-audit.lean` → 0; `--file no-such.lean` → exit 2 with the explicit-path hint; the default repo-root run → exit 0 (18 audited). |

### Deferred: EXCL-001 (duplicated private membership lemmas)

**Reason recorded:** private, single-file bridges; consolidating would touch
three verified proof modules for no semantic gain; tracked as future proof-tree
cleanup.

**Candidate shared lemmas (the exact duplicates):**
`Soundness.lean:69` `mem_danglingFindings_iff`, `:114`
`mem_equationFindings_iff`, `:487` `mem_coproductDeclaration_iff`, `:547`
`mem_coproductFindings_forward`; `DanglingDiscipline.lean:716`
`mem_coproductFindings`, `:724` `mem_coproductFindings_of`; plus the
`GroupGood` / `insertRow` / `stepGroups` / `foldl_range_good` /
`groupedRows_spec` invariants private to `Soundness.lean` that
`DanglingDiscipline.lean` re-derives (`insertRow_resolved`, `keyStep_resolved`,
`foldl_resolved`, `groupedRows_eq`, `mem_groupedRows_resolved`). The
consolidation target is one `Catherine/Proofs/Membership.lean` imported by
both, with the per-family proof bodies left in place. Deferral is acceptable:
both modules are proved, sorry-free, and audited, and the duplication is proof
plumbing rather than semantics.

### Resolved finding: EDGE-003 (LOW, FIXED)

**Location (as found):** `scripts/check-lean-axioms.sh:137` (the elaboration ran
after `cd -- "$LEAN_DIR"`) together with the argument parsing at `:54-80`.

**Description:** `AUDIT_FILE` was stored exactly as given, and the script later
`cd`s into `lean/` to run `lake env lean "$AUDIT_FILE"`, so a relative `--file`
path was resolved against `lean/` instead of the invocation directory.

**Pre-fix evidence (CONFIRMED, reproduced):** from the repository root,
`bash scripts/check-lean-axioms.sh --file lean/Catherine/Axioms.lean` exited 1
with `no such file or directory (error code: 2)`; the same file passed as an
absolute path exited 0 with `18 audited declaration(s)`.

**Impact:** self-test/authoring ergonomics only; it failed safe (exit 1, never
a false green), and the gate path (`just lean`, CI) uses the default absolute
path.

**Resolution (verified):** `scripts/check-lean-axioms.sh:85-87` now rewrites an
explicit relative `--file` against the invocation directory
(`AUDIT_FILE="$PWD/$AUDIT_FILE"`) before anything `cd`s. Re-reproduced from
`/tmp` (commands 35–39 above):

- `--file rel-audit.lean` → exit 0, `1 audited declaration(s)`; the path is
  reflected as `/tmp/rel-audit.lean` in any error text.
- `--file rel-cheat.lean` → exit 1, naming `disallowed axiom 'cheat2'` and
  `axiom declaration at /tmp/rel-cheat.lean:1:axiom cheat2 : False`.
- Absolute `--file` (clean/cheat) → 0/1; `--file=rel-audit.lean` → 0;
  `--file no-such.lean` → exit 2 with the explicit-path hint; the default
  repo-root run → exit 0 with 18 audited declarations; `--help` → 0.

**Status:** FIXED. No recommendation remains.

### Residual notes (non-blocking, not findings)

- The build-warning scan is complete on a fresh build, which is what CI does
  (clean checkout, `lake build` compiles every module). On an incrementally
  built local tree, a pre-existing `sorry` in an already-up-to-date module is
  not re-warned; the audited axiom-set check still catches any `sorryAx`
  reachable from the 18 audited theorems, and a clean build closes the gap.
- `lean/README.md:77-84` now describes the gate as implemented (build
  `sorry` warnings, the axiom-set allowlist, and the all-module `axiom` source
  scan); verified against the script's behaviour.

## Post-fix convergence

```
CRITICAL ever raised: 0
HIGH   raised: 1  -> FIXED (CORR-001)
MEDIUM raised: 3  -> FIXED (DRAFT-001, CORR-002, CORR-003)
LOW    raised: 7 total (6 original + EDGE-003, found in the re-check)
           -> 6 FIXED, 1 DEFERRED with reason (EXCL-001)
Open CRITICAL/HIGH/MEDIUM: 0
Open LOW: 1 (EXCL-001, DEFERRED; EDGE-003 FIXED and re-verified)
Self-estimated false positive rate: 0% (every disposition is command- or
file:line-backed; the one UNVERIFIED item is EDGE-002's pre-fix provenance,
now resolved by the verified hash match)
Status: CONVERGED — the gate conditions hold; the only residual item is the
deferred proof-tree cleanup
```

## Regression check on the semantic claims

No semantic claim regressed. `lake build` is green (24 jobs); both oracles
report the identical corpus shape (`190/180/10/1/102/79`, `divergences=0`,
`build_mismatches=0`); the three frozen theorems plus the exclusion theorems
and the `Basic` lemmas are sorry-free with axiom sets inside
`{propext, Classical.choice, Quot.sound}`; the strengthened dangling clause is
proved; `cargo test -p catherine-ledger` is green; the decision ledger validates
and its hashes resolve to the two normative spec files; `pretender`, `openspec
--strict`, and `ah check` are all green. The `check`/`build`/mirror code was not
touched by the fix wave, so the original parity evidence still applies.

---

# Rule of 5 Review — Final Report (post-fix)

**Work reviewed:** openspec change `add-lean-kernel-model` (beads
`catherine-xvh`) — full diff on base `16e1492`, re-verified after the fix-up
wave at 17:45–17:55 -03.
**Convergence:** Stage 5 + re-verification pass.

## Summary

Total findings across both passes: 11.

- CRITICAL: 0 — never raised.
- HIGH: 1 — CORR-001, FIXED and re-verified (both former bypasses now fail the
  self-tests; the clean audit passes).
- MEDIUM: 3 — DRAFT-001, CORR-002, CORR-003, all FIXED.
- LOW: 7 — 6 FIXED (DRAFT-002, CLAR-001, EDGE-001, EDGE-002, EXCL-002, and
  EDGE-003, found during re-verification and fixed inline), 1 DEFERRED with a
  written reason (EXCL-001).

TypeSafe: the single CRITICAL/HIGH finding (CORR-001) was verified at
confidence 0.29 (`REVIEW_REQUIRED`) in the original pass; it is now FIXED and
its fix was verified by commands, not by TypeSafe. No new CRITICAL/HIGH finding
exists, so no additional Stage-B request was required; all post-fix MEDIUM/LOW
items are UNVERIFIED by TypeSafe per the skill's cost cap and are backed by the
commands above.

## Top findings (post-fix)

1. **EXCL-001** (LOW, DEFERRED) — `private` membership-bridge lemmas are
   duplicated between `Soundness.lean` and `DanglingDiscipline.lean`; the
   candidate shared lemmas are listed above. Deferred as proof-tree cleanup;
   the only item that remains open.
2. **EDGE-003** (LOW, FIXED) — the `--file` self-test seam resolved relative
   paths against `lean/`; now resolved against the invocation `$PWD` before the
   `cd`, re-reproduced from `/tmp` for clean, cheating, absolute, `--file=`,
   missing, and default invocations.
3. **CORR-001** (HIGH, CLOSED) — the audited gate could accept an axiomatized
   proof; now the axiom set is asserted against an allowlist, every module is
   scanned for `axiom` declarations, and the build's `sorry` warnings fail the
   gate.

## Stage-by-stage quality (post-fix)

- Stage 1 (Draft): EXCELLENT — the change record now matches the delivered
  shape and completion state.
- Stage 2 (Correctness): EXCELLENT — the gate defect and the proof-coverage gaps
  are closed and re-verified; the semantics still agree on every compared
  fixture.
- Stage 3 (Clarity): EXCELLENT — the design doc now describes the implemented
  mirror.
- Stage 4 (Edge Cases): EXCELLENT — the elan bootstrap is pinned and
  fail-fast, the ledger hashes are traceable, and the `--file` seam resolves
  relative paths correctly (re-reproduced from `/tmp`).
- Stage 5 (Excellence): GOOD — both parity oracles run in CI; the lemma
  duplication is a deferred, named cleanup.

## Recommended actions

1. Schedule the `Catherine/Proofs/Membership.lean` consolidation for the named
   duplicate lemmas (EXCL-001).
2. Nothing else blocking: EDGE-003 is fixed and re-verified, the README
   describes the hardened gate, and task 14 is closed with `bd` exported.

## Verdict

**READY**

**Rationale:** After the fix wave, no CRITICAL, HIGH, or MEDIUM finding remains
open: the HIGH axiom-audit defect is fixed and its two former bypasses are
reproduced as failures, the three theorems plus every public lemma are audited
sorry-free, the dangling clause is symmetrically proved, and the change record
matches the delivered shape. The only remaining LOW is EXCL-001, explicitly
deferred with a written reason and named targets; EDGE-003 was fixed inline and
re-reproduced from `/tmp` (relative, absolute, `--file=`, missing, and default
invocations) — nothing blocks merge.

---

## Gate

```
max_critical: 0
critical: 0, high: 0, medium: 0, low: 1
verdict: READY
```

Gate rule applied: `max_critical = 0` is met; `critical = 0`, `high = 0`, and
`medium = 0`; the `low: 1` is EXCL-001 (DEFERRED with the reason and candidate
lemmas recorded above). EDGE-003 is FIXED and re-reproduced (rows 35–39). No
finding was downgraded to reach the gate: CORR-001 was re-verified by making
both bypasses fail, and the pretender/rustfmt failures from the first pass were
re-verified green rather than waived.
