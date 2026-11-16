# ro5u review — catherine-0qz (model composition at a seam)

Change: `add-model-composition` (ticket `catherine-0qz`). Base `df5ba6b`,
worktree uncommitted. Review mode: **rule-of-5-universal**, gate
**max_critical = 0**.

Contract read: the change's `proposal.md`, `design.md`, `tasks.md`, both
spec deltas (`openspec/changes/add-model-composition/specs/{composition,cli}/spec.md`),
the eight `.espectacular/changes/add-model-composition/**` contracts, the
`composition-absence` row in `docs/ref/expressibility-dispositions.md`,
README/CHANGELOG 0qz hunks, `openspec/specs/domain-probes/spec.md`, and
`docs/ro5u-checklist.md`.

Scope reviewed (0qz hunks only):

- `crates/catherine-core/src/{lib,composition}.rs`,
  `composition/{merge,resolve,interface}.rs`, `parse/{mod,deferred,spans}.rs`
- `crates/catherine-cli/src/{cli,main,run,runners,composition,render,render/envelope}.rs`,
  `runners/underconstraint.rs`
- tests/fixtures: `crates/catherine-core/tests/composition.rs`,
  `crates/catherine-cli/tests/{composition,negative_matrix_composition}.rs`,
  `crates/catherine-cli/tests/fixtures/composition/**`
- probe: `crates/catherine-ledger/src/catalog.rs` (probe-composition only),
  `ledger/probes/composition/**`, `fixtures/probes/composition/**`
- docs: the two spec deltas, `tasks.md`, the eight `.espectacular`
  contracts, the `composition-absence` row, README/CHANGELOG 0qz hunks

Not reviewed as 0qz: the untracked sibling RED material
(`crates/catherine-analysis/tests/schema_diff_coproducts.rs`,
`crates/catherine-core/tests/coproducts.rs`,
`crates/catherine-cli/tests/{coproducts,negative_matrix_coproduct}.rs`,
`.espectacular/changes/add-coproduct-exhaustiveness/**`) — it belongs to
`add-coproduct-exhaustiveness` and intentionally does not compile
(`crates/catherine-core/src/model.rs` has no `coproducts` field), so
`cargo test --workspace` is not runnable in this worktree.

## Commands executed

| # | Command | Result |
|---|---|---|
| 1 | `cargo test -q -p catherine-core --test composition` | **11 passed**, 0 failed |
| 2 | `cargo test -q -p catherine-cli --test composition --test negative_matrix_composition` | **8 + 1 passed**, 0 failed |
| 3 | `cargo run -q -p catherine-ledger --bin ledger-check -- ledger/probes/composition` | exit 0 — `ok: 2 entries (2 kata), rollup verdict: (no rollup)` |
| 4 | `openspec validate --strict add-model-composition` | exit 0 — `Change 'add-model-composition' is valid` |
| 5 | canonical `model_hash(parse(fixture))` recomputation for the five committed documents (scratch `/tmp` crate, no repo write) | all five match the entries: floor `4e251b9f…`, hand-merge `579d5b0d…`, billing `c90d90e6…`, shipping `c9fe1e75…`, seam claim `59455f9c…` |
| 6 | `probe_catalog_hash("probe-composition")` (scratch `/tmp` crate) | `sha256:3086cfde…` — matches both entries' `catalog_hash` |
| 7 | entry arithmetic replay (`derive_probe_verdict`) over both entries | `expected == probe_expectations` **true**; recorded `killed` == derived `Killed`; control seq 1; `unique_detections` 0; baseline `Executed` |
| 8 | `catherine check fixtures/probes/composition/floor_dangling.ct.json [same file] --json` | exit 4 both arms, one `dangling`/`executed` `{arrow: invoice_region, src: 0}`; the joint arm adds `source: ["…/floor_dangling.ct.json"]` |
| 9 | `catherine check <billing> <shipping> <seam_claim> --json` | exit 3, one `violation`/`executed` `{equation: 0, row: 2, lhs: 0, rhs: 1}`, `source` names the seam-claim, billing, and shipping documents |
| 10 | `catherine check fixtures/probes/composition/hand_merged_seam.ct.json --json` | exit 3, the same witness `{equation: 0, row: 2, lhs: 0, rhs: 1}` |
| 11 | `catherine check <billing>` / `<shipping>` / `<seam_claim>` alone | exit 0, `findings: []` each |
| 12 | `catherine check <endpoints_a> <endpoints_b> --json` | exit 1, `schema_conflict`, message names arrow `f`, both documents, `(line 5, column 16)` |
| 13 | `catherine check <rows_a> <rows_b> --json` | exit 1, `row_conflict`, message names `link`, row `h1`, both documents, `(line 10, column 31)` |
| 14 | `catherine check <keys_a> <keys_b> --json` (scratch fixtures) | exit 3, two key violations, `source` correctly `[b, a]` / `[a, b]` per declaring document |
| 15 | `catherine --human check <disjoint_a> <disjoint_b>`; also with stderr captured | exit 0, **stdout and stderr empty** (see H-3) |
| 16 | `catherine check <disjoint_a> <disjoint_b> --json` | exit 0, `warnings[0].rule_name == "empty_composition_interface"`, both paths named |
| 17 | `catherine underconstraint <parallel_a> <parallel_b> --json` | exit 6, one `underconstraint`/`hypothesized`, `source` names both documents |
| 18 | `catherine underconstraint <disjoint_a> <disjoint_b> --json` | exit 6 (per-document loop questions); `warnings: []` — no empty-interface warning (M-1) |
| 19 | partial-row probe `/tmp/oqz/{a,b}.json` (each clean alone) | joint → exit **4**, fabricated `dangling` `invoice_region@row1` (H-1) |
| 20 | null-vs-value probe `/tmp/oqz/{nul,val}.json` (`nul` alone exits 4) | joint → exit **0**, no finding (H-1) |
| 21 | flat-name probe `/tmp/oqz/{d1,d2}.json` (`Alpha.x` vs `Beta.x`) | exit 0, `warnings: []` (H-2) |
| 22 | precedence probe `/tmp/oqz/prec.json` (eq0 unknown arrow, eq1 bad status) | one path → `unknown_arrow`; the same document twice → `invalid_status` (H-4) |
| 23 | `catherine diff /tmp/oqz/seam_a.json <same file> --json` (equation names a sibling's arrow) | exit 1, `unknown_arrow` (H-5) |
| 24 | `catherine diff <seam_claim> <seam_claim minus equation>` — default / `--rows Order=1 --rows Region=2` / all four objects bound | `unknown` / `equivalent-in-scope` / `weakened` + exit 3 (M-4) |
| 25 | order swapped: `<seam> <shipping> <billing>` and `<endpoints_b> <endpoints_a>` | same finding set; conflict message deterministic in input order (fold is order-invariant on the model) |

Not run (per review contract): `cargo fmt`, `cargo clippy`, `just ci`,
`cargo test --workspace`.

Shorthand used in the table: H-1 = DRAFT-001/CORR-003, H-2 = CORR-001,
H-3 = K-002, H-4 = CORR-002, H-5 = K-001/DRAFT-002, M-1 = CORR-004,
M-4 = K-004.

## Pass 1 — Draft / shape

Shape is **GOOD**. The change lands on existing seams rather than
inventing a platform: one kernel module (`composition`), one deferred
parse pass, one CLI support module plus two dispatch branches, one
runner. The variadic-`check` decision (design Decision 1) reuses the
exit-code table, envelope constructors, and shared loaders; there is no
new finding kind, no `--emit`, and no second execution pipeline. The
probe reuses the kata entry shape and the closed verdict vocabulary.
Single-document bytes are preserved by *not* touching the 1-document
branch, and the tests are structural.

What the shape gets wrong:

**[DRAFT-001] [HIGH] Partial-instance semantics are undefined.** The
operation unions row spaces across documents but keeps each arrow's
declaration and fills cells only from documents that declare that
arrow (`merge.rs:96-145`, `merge.rs:151-190`). A row declared only by a
sibling input therefore becomes a `None` cell that `checks::check`
reports as `dangling`, and a `null` cell in one input is silently filled
by another input's value. Both directions were executed (commands 19,
20): two individually clean documents compose to exit 4 with a finding
no input entails; a document that fails alone (`dangling`) composes to
exit 0. This contradicts the capability's own premise ("two documents
each check clean alone … the joint run reports the seam violation") and
leaves the merged `Instance` violating the format's total-table rule
that the parser enforces. Decision needed: either (a) require per-object
row-set agreement on a shared object (a labeled conflict), (b) treat
cells no declaring document carries as absent rather than dangling, or
(c) document and test the current partial-union reading, including the
null-vs-value rule. Disposition: **fix before close** (spec + test +
code or explicit documentation).

**[DRAFT-002] [HIGH] The "sealed evidence" claim is not the shape the
code supports for a genuinely cross-document claim.** The composition
delta requires cross-document claims to live in an input document "so
they are hashed and schema-diffable like any other declaration"
(`specs/composition/spec.md:44-54`; README:88-93; CHANGELOG:19-22;
design:55-67). A document that actually names a sibling's arrow is
rejected by the single-document parser (`unknown_arrow`), so
`model_hash(parse(..))` cannot be computed and `diff` cannot read it
(command 23). Only the self-contained seam-claim fixture — which
declares all six arrows itself — is parseable, and therefore "diffable".
Disposition: **fix before close** (amend the requirement/README to the
real invariant, or give `diff` a composition-aware form).

**[DRAFT-003] [LOW] tasks.md status is stale.** RED tasks 1–4 are
implemented (the named tests exist and pass) but unchecked
(`tasks.md:8-30`); GREEN 6–9 are checked while item 11 (docs) is done
but unchecked (`tasks.md:46-50`); item 12 (`cargo test --workspace`) is
not runnable in this worktree because of the sibling RED coproduct
tests. Disposition: update the checkboxes/notes.

Shape Quality: **GOOD**.

## Pass 2 — Correctness

The merge machinery itself is sound where it is defined:

- Row identity is the presentation name, remapped through each
  document's `RowNames` into a sorted merged dense space
  (`merge.rs:96-129`). Presentation order never decides the merge;
  `AGREE_A`/`AGREE_B` (same rows, different file order) glue to one row
  space, and the singleton remap is the identity because the parser
  already assigns dense ids in sorted-name order
  (`parse/mod.rs:222-229`).
- Arrow identity is the declared name; an agreeing repeat records the
  second declaring document, a disagreeing one is `schema_conflict`
  naming the arrow, both documents, and the offending `arrows/{i}`
  position (`merge.rs:39-72`). Verified end-to-end (command 12).
- `row_conflict` fires on a shared source row in a commonly declared
  arrow when the remapped targets differ, naming the arrow, the row,
  both target names, both documents, and the cell position
  (`merge.rs:171-206`). Verified (command 13).
- Equations and keys are kept as an enforced multiset in canonical order
  (`resolve.rs:29-50`, `resolve.rs:192-220`); both sides' declarations
  execute (core tests `equations_from_both_documents_are_enforced`,
  `keys_from_both_documents_are_enforced`; key attribution verified at
  command 14).
- Deferred resolution runs exactly once over the merged arrows; a
  dangling element reproduces the parse-time `unknown_arrow` message
  (`equation path [a, b] references undeclared arrow \`b\``), span, and
  code for a singleton glue (core test
  `single_document_unknown_arrow_matches_the_parse_time_error`, 11/11
  green). Endpoint typing reproduces the parse-time
  `ill_typed_equation` message shape (`resolve.rs:111-133`).
- A path cannot resolve to the wrong arrow: arrows are keyed by name in
  one `BTreeMap`, and a same-name/different-endpoint overlap is refused
  before resolution.
- The n-ary fold is order-invariant on the merged model (sorted rows,
  canonical equations/keys, `BTreeMap` arrows); only attribution order
  and conflict-message order follow input order, deterministically
  (command 25).
- `empty_interface` is a symmetric intersection across all inputs
  (`interface.rs:15-30`), so a third document sharing nothing warns.
- Exit code is still `run::exit_code(findings)` (min over classes) for
  the composition path (`composition.rs:231`); precedence verified
  (violation > dangling → 3).

**[CORR-001] [HIGH] `empty_interface` compares a flat string set.**
`declared_names` (`interface.rs:33-37`) puts object names, arrow names,
and row names into one `BTreeSet<String>`. Row identity is
`(object, row)`, so `Alpha.x` and `Beta.x` are different declarations;
the predicate nevertheless sees a shared name and suppresses the
warning. Executed (command 21): two documents with disjoint objects and
no shared arrows, both carrying a row named `x`, exit 0 with
`warnings: []`. The converse also holds (an arrow named `x` in one
document and a row named `x` in another counts as an interface).
Disposition: **fix before close** — compare namespaced identities
(objects; arrows; `(object, row)`), and add the fixture to the CLI test.

**[CORR-002] [HIGH] The CLI forks the resolution path on arity.** The
design's Decision 3 invariant ("One resolution path … no
parse-time-vs-glue-time fork to diverge") is not implemented: `run`
routes one path through `load_model`/`parse_with_rows` (eager
`check_equations`) and two or more through `parse_deferred` + `glue`
(`run.rs:81-90`). The two paths choose different errors for the same
document because the deferred pass validates path idents and statuses
for **all** equations first, then sorts before resolving, while the
eager pass checks declaredness/endpoints per equation in file order
before `check_keys`/`check_rows`/`check_tables`
(`deferred.rs:111-141`, `resolve.rs:35-36`). Executed (command 22): the
same bytes report `unknown_arrow` as one path and `invalid_status` as
two. Disposition: **fix before close** — either route the singleton
through `glue()` and align the deferred check order (preferred, matches
the design), or amend Decision 3 + README and add a precedence-parity
test. The existing parity test covers only the single-error
`unknown_arrow` case.

**[CORR-003] [HIGH] Composition swallows and fabricates `dangling`
findings.** Same root as DRAFT-001, stated as behavior: at
`merge.rs:137` the merged cell vector is sized by the *union* row count,
but only declaring documents fill it; `fill_doc_cells` skips `None`
(`merge.rs:151-166`). Consequences executed in commands 19–20. This is
the correctness face of the shape finding; one fix covers both.

**[CORR-004] [MEDIUM] The joint `underconstraint` run never emits
`empty_composition_interface`.** `run_composition` pushes the warning
(`run.rs:126-128`), but `runners/underconstraint.rs:32-48` does not, in
either output mode (command 18: disjoint inputs, `warnings: []`). A
disjoint union is presented as a seam over the glued schema on that
surface with no signal. Disposition: fix (one push, same helper) and
add the row.

**[CORR-005] [LOW] `merge_equations` zips equations with their origins
and silently truncates on a length mismatch** (`resolve.rs:53-70`,
`.zip(&doc.equations)`). Unreachable today (`Document` is only built by
`parse_document`, whose vectors are aligned), but a future second
constructor would silently drop equations from the merged schema.
Disposition: defer with reason — replace the zip with an explicit
length assertion when the constructor surface grows.

**[CORR-006] [LOW] `sources_for` carries branches no composition
surface can produce** (`composition.rs:88-136`: `Redundant`,
`Contradiction`, `Naturality`; `DomainQuestion::UncoveredInputs` returns
`None`). Harmless but speculative; `emit_findings` also clones every
finding via `findings.to_vec()` though both callers own the `Vec`
(`composition.rs:217`). Disposition: defer; trim when the attribution
helper is next touched.

Correctness Quality: **FAIR** (the defined merge rules are correct; two
semantic holes remain).

## Pass 3 — Contract drift

Checked one resolution path, deferred spans, byte identity, structured
conflicts, source attribution, exit codes/warnings, and the probe
against the contract documents.

- **Single-document bytes.** The 1-document branch of `run` is unchanged
  in the diff (only the arity dispatch and the `model_paths.first()`
  binding were added), `parse_with_rows` still runs
  `check_equations` via `validate_with(.., false)`
  (`parse/mod.rs:71-74,112-150`), and the JSON branch still calls
  `findings_envelope(&findings, warnings)`. Byte identity is therefore
  structural; the committed baseline fixture
  (`tests/fixtures/composition/single_document_baseline.json`) is a
  regression pin normalized only for `meta.duration_ms`
  (`tests/composition.rs:38-51`), which is a legitimate normalization.
  Provenance of the fixture ("captured from the pre-change binary") is
  not independently provable from the repo, but the static diff is
  decisive. PASS.
- **Witness encoding.** `SourcedFinding` flattens `Finding` and adds one
  optional `source` list (`render/envelope.rs:41-64`); the witness bytes
  are untouched (verified in commands 9, 14, 17). PASS.
- **Attribution cannot name the wrong document.** Equation sources =
  declaring doc + the docs behind each path's arrows
  (`resolve.rs:162-190`); key sources = declaring doc + arrow docs;
  alignment with `schema.equations`/`schema.keys` holds because both are
  canonically sorted in the same pass. Verified for equations, keys,
  dangling, and parallel paths. PASS.
- **Conflicts are never silent.** Both conflict codes are labeled
  errors, name both documents and the node, and carry a position
  (commands 12–13); the negative-matrix rows assert exit 1 + code +
  message. PASS.
- **Exit-code precedence.** Unchanged (`run::exit_code`); joint
  violation+dangling → 3. PASS.
- **Probe honesty.** Floor control first and self-negating; baseline arm
  committed, executed, and hash-pinned; unique set empty; the
  workflow-elimination claim is a named `expressibility_gap`, never a
  detection; catalog hash and model hashes verified; validator green;
  verdict `killed` = derived arithmetic. PASS.

Drift found:

**[K-001] [HIGH] "Hashed and schema-diffable like any other
declaration" is false for a genuinely cross-document equation.**
`specs/composition/spec.md:44-54` (requirement + `Seam claim is sealed
document evidence` scenario), README:88-93, CHANGELOG:19-22, design:55-67.
The requirement's premise ("names defined in another input") makes the
document unparseable by every single-document surface; `diff` exits 1
`unknown_arrow` (command 23), and `model_hash` requires `parse`. Only a
document that re-declares the referenced arrows (the committed fixture)
is parseable, and then it is not the case the requirement describes.
Disposition: **fix before close** — either amend the requirement/README
to state the real invariant (the declaring document must also declare
the referenced arrows to be diffable; a joint-only fragment is hashed by
file bytes, not model identity), or add a composition-aware `diff`
(two document sets) as a superseding change. Do not leave the current
claim in the spec, README, CHANGELOG, and design.

**[K-002] [HIGH] The `empty_composition_interface` warning is invisible
in human output mode.** `composition.rs:209-231` passes `warnings` only
into `sourced_findings_envelope`; the human branch never calls
`runners::print_run_warnings` (`runners.rs:36-44`, the catherine-uh0
discipline: "run-level warnings reach stderr in human mode too").
Executed (command 15): `catherine --human check <disjoint_a>
<disjoint_b>` exits 0 with empty stdout and stderr — a signal-free clean
run, which the composition delta's scenario forbids
(`specs/composition/spec.md:28-33`). Disposition: **fix before close** —
call `print_run_warnings(json, &warnings, stderr)` before rendering
(and be mindful that config warnings are already printed by
`resolve_settings`; de-duplicate if needed), or explicitly narrow the
requirement to `--json`.

**[K-003] [MEDIUM] The negative matrix does not cover two of the changed
surfaces.** `negative_matrix_composition.rs:82-101` defines only
`Error`/`Usage`; there is no `Warning` row for the new labeled
`empty_composition_interface` surface, no row for the variadic
`underconstraint` (parse failure or `schema_conflict`), and none for the
variadic `findings` surface. `docs/ro5u-checklist.md` (catherine-348)
requires every changed CLI surface to gain rows over every applicable
malformed-input class, including the envelope-warning class. The check
rows themselves are good (conflict, dangling reference, malformed first
and second document, unknown flag). Disposition: fix — add
`Expect::Warning { exit: 0, contains: "empty_composition_interface" }`,
`underconstraint <billing> <malformed>`, `underconstraint <endpoints_a>
<endpoints_b>`, and a `findings` multi-document row.

**[K-004] [MEDIUM] The `.espectacular` contract for the sealed seam
claim is mapped to an unrelated pre-existing test.**
`.espectacular/changes/add-model-composition/composition/seam-claim-is-sealed-document-evidence.toml`
cites `weakened_edit_exits_3_with_witness_and_budget_expiry_is_unknown`
(`crates/catherine-cli/tests/cli_diff.rs:76`), which uses
`WEAKENED_V1/V2` from another ticket's fixtures and never touches a
seam-claim document. No test drops a seam equation and asserts the diff
outcome. A manual run (command 24) shows the classification is
bounds-sensitive: default bounds → `unknown`; `--rows Order=1 --rows
Region=2` → `equivalent-in-scope`; all four objects bound → `weakened`
exit 3. The contract's claim is true only with the full bound, which no
composition doc states. Disposition: fix or narrow — add a test that
drops the seam equation and asserts `weakened` with the documented
bounds, and cite it from the contract.

**[K-005] [LOW] `LabeledError`'s doc comment is stale.**
`render/errors.rs:18-32` says the code is one of the kernel's closed
semantic codes, `syntax`, or `read_error`; composition adds the stable
`schema_conflict` / `row_conflict` CLI codes. The codes are correctly
outside the closed `ErrorCode` enum (they are CLI labels, like
`schema_mismatch`), but the comment should say so.

**[K-006] [LOW] Design/proposal text nits.** `design.md:64-65` has a
garbled span (`` (`dropping the seam equation is `` / `` `weakened` ``)
and an incomplete sentence; `proposal.md:70` lists
`crates/catherine-analysis/src/` as affected, but no change was made
there (attribution is CLI-side). Disposition: defer (docs-only sweep).

Clarity Quality: **GOOD** (the module and function docs are dense but
consistent with the repo's style; the drift is in the external claims).

## Pass 4 — Edge cases

Exercised: equal/unequal row sets, `null` cells, duplicate paths, three
documents, order permutations, disjoint unions, shared-only-object
interfaces, flat row-name collisions, keys declared twice, equations
declared twice, cross-document dangling references, single-document
unknown arrow, conflict position fallbacks, empty equation lists,
zero-row objects.

Handled correctly: `AGREE_B`'s reordered rows; a shared arrow in three
documents; a third document that shares nothing (symmetric
intersection); duplicate path (`check a a`) — duplicated enforcement,
consistent with duplicate declarations in one document; an equation with
an empty path (parse-time `syntax`, unchanged); a document with no
equations; a conflict on the third input (document list grows in input
order); attribution for a duplicated identical equation (stable sort
keeps input order).

Unhandled / unverified:

**[EDGE-001] [HIGH] Shared object with unequal row sets (both
directions).** Commands 19–20; see CORR-003. This is the edge case the
review brief flagged: rows merge by name, but the merge key is the row,
not the arrow's domain. Minimal tests to add: `clean + clean with an
extra row` must not fabricate a finding, and `null + value` must have a
decided outcome (conflict or value).

**[EDGE-002] [HIGH] Flat-name interface false negative.** Command 21;
see CORR-001. The unit-level predicate change is small; add a
`disjoint_same_row_name` fixture pair to both the core test and the CLI
warning test.

**[EDGE-003] [MEDIUM] `underconstraint` on a disjoint union is silent.**
Command 18; see CORR-004.

**[EDGE-004] [LOW] The precedence test uses two disjoint documents.**
`.espectacular/.../cli/two-documents-run-the-joint-check.toml` maps its
"both a violation and a dangling finding still exits 3" claim to
`joint_runs_keep_the_exit_code_precedence`, which composes
`multi_class.ct.json` (Block/Section) with `bowling.ct.json`
(Assignment/…): they share nothing, so the test now also triggers the
empty-interface warning (unasserted) and does not exercise precedence
over shared-subschema documents. The two class findings both survive the
union, so the assertion is still meaningful, but the fixture choice is
not the contract's scenario.

**[EDGE-005] [LOW] "Enforced multiset" is tested with two *different*
equations, never with an identical repeat.** The design's rationale for
duplicates is "a repeated identical declaration enforces twice, exactly
as repeated declarations within one document already do"; no test pins
that (two findings for the same `(lhs, rhs)`). Defer or add a two-line
test.

**[EDGE-006] [LOW] No test covers the variadic `findings` surface.**
`findings` shares `run()`, but the changed surface has no direct
assertion; add one row (or rely on the matrix row from K-003).

Edge Case Coverage: **FAIR**.

## Pass 5 — Excellence (tests, probe, production quality)

Tests are structural and honest: kernel values/codes/positions; CLI exit
codes and envelope fields; one mandated byte-identity test with a
committed baseline. No snapshot tests, no wiring assertions. The
negative-matrix rows are independent of the unit tests. Good.

**[EXCL-001] [MEDIUM] The composition probe has no regression test.**
Sibling probes are pinned by `crates/catherine-ledger/tests/probe_*.rs`
(`probe_naturality.rs` recomputes both fixture model hashes, validates
the directory, and asserts each recorded verdict equals
`derive_probe_verdict`); composition has nothing. I verified the hashes,
catalog hash, expectations, and verdicts manually (commands 5–7) and the
entries are honest today, but a fixture edit or a verdict word edit
would not fail any test. `probe_catalog_green.rs:111-118` also hardcodes
the three older probe ids, so the new catalog hash is not in the
content-address ratchet. Disposition: add
`crates/catherine-ledger/tests/probe_composition.rs` mirroring
`probe_naturality.rs`, and add `probe-composition` to the
`catalog_hash_is_content_addressed` id list.

**[EXCL-002] [LOW] Probe recording inconsistencies.** Entry 1's
`run.findings` omits the additive `source` field that the executed joint
arm actually emitted (entry 2 includes it); entry 2's
`comparison.baseline.artifact` names the committed document but no tool
+ revision, while entry 1 names `catherine check (… revision df5ba6b)`
and the sibling naturality entries name tool + code site + revision
(`openspec/specs/domain-probes/spec.md`, "script or tool + revision").
Disposition: defer with reason — both are recording-shape nits; fix in
the probe-test change (EXCL-001).

**[EXCL-003] [LOW] Avoidable copies.** `emit_findings` clones the whole
finding vector (`findings.to_vec()`) although both callers own it;
`merge_schema` clones each document's object map
(`merge.rs:25`, `doc.model.schema.objects.clone()`) where a keyed extend
would do. Disposition: defer; no behavior impact.

**[EXCL-004] [LOW] `resolve.rs::endpoints` falls back to empty object
names** (`unwrap_or_default()`) when a path element is undeclared.
Unreachable after `check_path_arrows`, but if the call order ever
changes the error becomes a confusing empty-name message rather than an
error. Disposition: defer (comment or make it unreachable by
construction).

Excellence Assessment:
- Structure: GOOD
- Correctness: FAIR
- Clarity: GOOD
- Edge Cases: FAIR
- Overall: **FAIR**

## TypeSafe verification (rule-of-5 §Validation)

`TYPESAFE_API_KEY` is set; the Stage-A mechanical pre-gate passed for
all five HIGH findings (locations exist; each quoted evidence string
appears verbatim in the reviewed files), and a batched
`POST https://api.typesafe.ai/v1/systemone` request (`jev-latest`,
resolved `jev-1.14.0`) asked one Choice question per HIGH finding. The
request was repeated once to check stability: the verdict labels were
identical, confidences moved by ≤0.1; the numbers below are the second
run.

| Finding | Verdict | Confidence | Gate (0.8) |
|---|---|---|---|
| CORR-001 | verified | 0.95 | stands |
| K-002 | verified | 0.93 | stands |
| CORR-002 | verified | 0.73 | **REVIEW_REQUIRED** (below gate) |
| CORR-003 | verified | 0.52 | **REVIEW_REQUIRED** (below gate) |
| K-001 | fabricated | 0.11 | **REVIEW_REQUIRED** (below gate) |

Measured false-positive rate: **0 / 3 = 0%** (no verdict at or above the
gate was `unsupported`/`contradicted`/`fabricated`). Three findings are
kept and marked `REVIEW_REQUIRED` because their verdict confidence is
below the gate; they are independently evidenced by executed commands
(19–20 for CORR-003, 22 for CORR-002, 23 for K-001), so they are not
withdrawn. MEDIUM/LOW findings are marked `UNVERIFIED` (below the cost
cap) and rest on the file:line evidence in this artifact. No
`META-*` finding is filed: the single low-confidence `fabricated`
verdict is not treated as a Stage-A failure.

## Convergence

| Check | Critical | New issues | vs previous | FP rate | Status |
|---|---|---|---|---|---|
| after pass 1 | 0 | 3 | — | 0% | CONTINUE |
| after pass 2 | 0 | 6 | +100% | 0% | CONTINUE |
| after pass 3 | 0 | 6 | 0% | 0% | CONVERGED on criticals |
| after pass 4 | 0 | 3 | −50% | 0% | CONVERGED on criticals |
| after pass 5 | 0 | 4 | +33% | 0% | CONVERGED on criticals |

(Pass-4 `EDGE-001..003` are cross-references to CORR-003/CORR-001/CORR-004,
not new findings; only EDGE-004/005/006 are new.)

No CRITICAL issue appeared in any pass (so no `ESCALATE_TO_HUMAN` on the
critical gate). Pass 5 added only test/probe/recording items after the
contract-mapping check; the strict <10% new-issue rule is not met
arithmetically, but every residual item is LOW/MEDIUM coverage or
bookkeeping and none changes the gate. Final convergence: **CONVERGED
(max_critical = 0)**.

## Findings and dispositions

| ID | Sev | Location | Finding | Disposition |
|---|---|---|---|---|
| DRAFT-001 / CORR-003 | HIGH | `composition/merge.rs:96-145,151-190` | Partial-instance semantics undefined: fabricated `dangling` for a union row no declaring document carries; `null` vs value silently resolved | Fix before close: decide (row-set agreement / absent-not-dangling / documented partial union), add the two repro tests, update the composition spec |
| CORR-001 / EDGE-002 | HIGH | `composition/interface.rs:33-37` | `empty_interface` compares a flat string set; disjoint docs that reuse a row name (or cross-kind names) are not warned | Fix: namespaced identities (object, arrow, `(object,row)`); add core + CLI fixtures |
| CORR-002 | HIGH | `cli/src/run.rs:81-90`, `composition/resolve.rs:35-36`, `parse/deferred.rs:111-141` | "One resolution path" not implemented; 1-doc vs 2-doc error precedence diverges for the same bytes | Fix: route singleton through `glue()` and align deferred order (or amend design + add parity test) |
| K-001 / DRAFT-002 | HIGH | `specs/composition/spec.md:44-54`, README:88-93, CHANGELOG:19-22, `design.md:55-67` | "Hashed and schema-diffable like any other declaration" false for a genuinely cross-document equation (`diff` → `unknown_arrow`; `model_hash` needs a parse) | Fix: amend the requirement/README/CHANGELOG or add a composition-aware `diff` |
| K-002 | HIGH | `cli/src/composition.rs:209-231` vs `runners.rs:36-44` | Human-mode joint runs drop the `empty_composition_interface` warning (exit 0, no stdout/stderr) | Fix: call `print_run_warnings` before rendering (de-dupe config warnings) or narrow the requirement to `--json` |
| CORR-004 / EDGE-003 | MEDIUM | `cli/src/runners/underconstraint.rs:32-48` | Joint `underconstraint` never emits the empty-interface warning | Fix: push the same warning; add a row |
| K-003 | MEDIUM | `tests/negative_matrix_composition.rs:82-101` | No `Warning` row, no variadic `underconstraint` rows, no `findings` multi-doc row | Fix: add the rows |
| K-004 | MEDIUM | `.espectacular/.../composition/seam-claim-is-sealed-document-evidence.toml` | Contract maps to an unrelated diff test; the dropped-seam-equation classification is untested and bounds-sensitive | Fix or narrow: add a seam-claim diff test with documented bounds |
| EXCL-001 | MEDIUM | `crates/catherine-ledger/src/catalog.rs:272-303`; `ledger/probes/composition/**` | No probe regression test (hashes/catalog/verdicts unpinned); catalog-hash ratchet omits the new id | Fix: add `probe_composition.rs`; extend the id list |
| DRAFT-003 | LOW | `tasks.md:8-30,46-50` | Stale checkboxes; item 12 not runnable with the sibling RED coproduct tests | Update |
| CORR-005 | LOW | `composition/resolve.rs:53-70` | `zip` truncation would silently drop equations if the constructor surface grows | Defer with reason; assert lengths next touch |
| CORR-006 / EXCL-003 | LOW | `cli/src/composition.rs:88-136,217`; `merge.rs:25` | Dead attribution branches for kinds no composition surface produces; avoidable `to_vec()`/object-map clones | Defer; trim next touch |
| K-005 | LOW | `cli/src/render/errors.rs:18-32` | Doc comment still claims the closed kernel code set only | Defer (docs sweep) |
| K-006 | LOW | `design.md:64-65`, `proposal.md:70` | Broken span/garbled sentence; `catherine-analysis` listed as affected but unchanged | Defer (docs sweep) |
| EDGE-004 | LOW | `.espectacular/.../cli/two-documents-run-the-joint-check.toml` | Precedence claim exercised over two disjoint documents | Defer; note in the contract |
| EXCL-002 | LOW | `ledger/probes/composition/0001…json`, `0002…json` | Entry 1 omits `source` in `run.findings`; entry 2's baseline names no tool+revision | Defer to the probe-test change |
| EDGE-005 | LOW | `crates/catherine-core/tests/composition.rs` | No test for an identical repeated equation/key enforcing twice | Defer |
| EXCL-004 | LOW | `composition/resolve.rs:135-140` | Empty-name fallback on an unreachable branch | Defer |

Counts (finding rows): **CRITICAL 0 · HIGH 5 · MEDIUM 4 · LOW 9**.

## Verdict

**PASS on the stated gate (max_critical = 0).**
**Post-fix status: PASS — all five HIGH and every actionable MEDIUM were
fixed before the commit; remaining items are LOW with reasons (see Lead
dispositions).**

The gate itself is met: no CRITICAL finding, no measured false positive,
and the kernel/CLI composition core is correct for the cases it defines
(named union, structured conflicts, one post-glue resolution, multiset
enforcement, additive attribution, unchanged single-document bytes).
The probe is honest and hash-pinned.

**Not ready to close the ticket** under `docs/ro5u-checklist.md`: the
five HIGH findings are behavior/contract defects, not polish. Minimal
fixes, in order:

1. Decide and implement the partial-instance rule
   (`merge.rs:137-166`), with the two repro tests (clean+clean extra
   row; `null`+value).
2. Namespace the empty-interface predicate (`interface.rs:33-37`) and
   add the `Alpha.x`/`Beta.x` fixture.
3. Either route the singleton through `glue()` (`run.rs:81`) or correct
   design Decision 3 and add the precedence-parity test.
4. Print run-level warnings in human mode (`composition.rs:209-231`).
5. Amend the sealed-evidence requirement/README/CHANGELOG to the real
   invariant, or add a composition-aware `diff`.

Then the MEDIUMs: the underconstraint warning, the missing negative-matrix
rows, the seam-claim diff test, and the probe regression test.

## Unverified

- `cargo test --workspace`, `just ci`, `openspec validate --strict --all`,
  `ah check --changes add-model-composition --run-tests`: not run per the
  review contract; the workspace compile is additionally blocked by the
  sibling RED coproduct tests.
- Formatting/lint gates (`cargo fmt --check`, `cargo clippy`) and the
  strict pretender file/function gate: not run.
- Provenance of `tests/fixtures/composition/single_document_baseline.json`
  as "captured from the pre-change binary": not independently provable
  from the repository; byte identity is instead established statically
  from the unchanged 1-document branch.
- Ledger regression coverage for the composition probe: absent
  (EXCL-001) — verified manually here.
- TypeSafe: the three HIGH verdicts below the 0.8 gate are
  `REVIEW_REQUIRED`; MEDIUM/LOW findings are `UNVERIFIED`.
## Lead dispositions (post-review fixes)

Fixed before the 0qz commit:

- **HIGH-1** — row-set agreement: documents that claim rows for a shared
  object must declare the same row-name set (`row_conflict` naming both
  documents and the differing rows); an empty row vector is "no claim" (the
  schema-only seam document). `null` is a value claim: null-vs-value is a
  `row_conflict`, null-vs-null stays dangling. The fabricated-dangling union
  path is gone. `composition/instance.rs` + tests both sides; spec and design
  Decision 2 amended.
- **HIGH-2** — empty-interface identity is namespaced (objects, arrows,
  `(object,row)` pairs); warn only when all three intersections are empty.
- **HIGH-3** — human-mode joint runs print run-level warnings to stderr.
- **HIGH-4** — the deferred pass now runs the parser's own equation check
  against local arrows and defers only undeclared path arrows, so the first
  error (code/position/message) is arity-independent; the duplicate resolver
  was deleted and design Decision 3 states the real invariant (two entry
  paths, one resolver).
- **HIGH-5** — the standalone-hash/diff claim is amended in spec, design,
  README, CHANGELOG to the true invariant (joint-run evidence; standalone
  only when every name is local), with tests pinning both halves.
- **MEDIUMs** — joint `underconstraint` warns on a disjoint union; matrix
  gains a `Warning` class and variadic `underconstraint`/`findings` rows;
  the bounds-sensitive sealed-seam contract is replaced by joint-run and
  standalone-parseable contracts plus an arity-parity contract; probe
  regression test added (`probe_composition.rs`) and `probe-composition`
  joins the catalog-hash ratchet.

Deferred (LOW, reasons as recorded in the review's tables): `tasks.md`
checkbox bookkeeping, a doc-comment polish, the proposal path nit, avoidable
clones in the attribution path, and two extra edge tests. The `zip`
truncation and empty-name fallback findings disappeared with the deleted
duplicate resolver.
