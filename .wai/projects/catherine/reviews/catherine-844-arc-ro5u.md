# Rule of 5 Review — Final Report

**Work reviewed:** implementation arc `79cef70..08bf44f` (8 commits, 267 files, +16 488/−811): tickets `catherine-bvt`, `catherine-j6g`, `catherine-s60`, `catherine-0qz`, `catherine-iib` — archive chore, expressibility gap suite, `instance_map`/`naturality`, variadic joint check, `schema.coproducts`.
**Convergence:** Stage 5 (no critical after Stage 2; new items shrink to polish only).
**TypeSafe:** `TYPESAFE_API_KEY` is set. Stage A mechanical pre-gate found **no finding at or above `VERIFY_SEVERITIES` (CRITICAL, HIGH)**; therefore no verification request was issued, and MEDIUM/LOW findings are marked `UNVERIFIED` per the procedure's cost cap. Every finding's location and quoted evidence below were re-checked to exist in the reviewed tree.

## STAGE 1: DRAFT

Assessment: the arc has one change per capability, each with RED→GREEN→REFACTOR, a pre-registered probe, a per-ticket rule-of-5 artifact, and an archive commit that deploys the delta and migrates its espectacular contracts. The layering is right: kernel format passes in `catherine-core`, execution in `checks.rs`, analysis in its own crate, one CLI seam, and no second pipeline.

Issues:

[DRAFT-001] [LOW] — Cross-capability integration coverage
Description: every capability is tested in its own binaries; no committed test exercises one document carrying `schema.keys` + `schema.coproducts` + `instance_map` through `check`, nor a joint run that mixes all three. The combination was verified manually during this review (see CORR evidence commands) but is not ratcheted.
Recommendation: add one `crates/catherine-cli/tests/integration_sections.rs` fixture with all additive sections plus a two-document joint run; assert exit code, finding kinds, and canonical stability.

[DRAFT-002] [LOW] — Surface documentation placement is uneven
Description: `README.md` documents `naturality`, variadic `check`, and coproducts; `docs/USING.md` only gains a coproduct violation row. The agent playbook's seam recipe (Recipe D) still presents the seam claim as a single self-contained document.
Recommendation: add a USING recipe step for `check a b` (`source` attribution, `empty_composition_interface`) and a one-paragraph `naturality` recipe.

Shape Quality: EXCELLENT

## STAGE 2: CORRECTNESS

Issues Found:

[CORR-001] [MEDIUM] — Coproduct suppression is whole-declaration, not per undecidable row; spec says "undecidable" where a subset is decidable
Location: `crates/catherine-core/src/checks.rs:285-318` (`coproduct_declaration` early return; `injection_images` returns `None` on any unresolved cell); deployed `openspec/specs/kernel-checks/spec.md:122-129`.
Evidence: `/tmp/dual_hidden.ct.json` (Order o1 and o2 each covered by two *complete* injection cells while two unrelated cells are `null`) yields only the two dangling findings:
`{"findings":[{"kind":"dangling","witness":{"arrow":"card_inj","src":1}},{"kind":"dangling","witness":{"arrow":"transfer_inj","src":0}}]}` — no `coproduct` violation, although o1 ∈ card_inj(card) ∩ transfer_inj(transfer) and o2 ∈ both voucher cells. The deployed requirement states the declaration's findings "SHALL be suppressed" because coverage/disjointness are "undecidable"; disjointness of a row already in two *complete* images is decidable regardless of other unresolved cells.
Impact: a provable disjointness violation is withheld while any one injection cell is dangling; the run still exits 4 (dangling), so it is never a green run, but the declared claim is not executed.
Recommendation: either (a) narrow suppression to genuinely undecidable questions — report dual coverage when the row lies in ≥2 complete images, and report uncovered only when no complete image and no unresolved cell could reach the row; or (b) keep the rule and reword the requirement from "undecidable for that declaration" to "suppressed by design while a cell is unresolved". Add the dual-hidden fixture as a test either way.

[CORR-002] [MEDIUM] — Core README version policy is stale
Location: `crates/catherine-core/README.md:79-80` vs `crates/catherine-core/src/parse/schema.rs:88`.
Evidence: README says "currently 4, i.e. `1.4` … 1.4 added `instance_map`"; the parser sets `KNOWN_MINOR = 5` and accepts 1.5 (verified: `unsupported_version` names "known version is 1.5" for 1.6). This is the exact documentation-drift class the gap suite exists to prevent, in the file the iib change edited for other reasons.
Recommendation: extend the list to 1.5 (`schema.coproducts`); ideally keep the version list in one place and reference the spec from the crate README.

[CORR-003] [MEDIUM] — Coproduct probe is not ratcheted by any test
Location: `crates/catherine-ledger/tests/probe_catalog_green.rs:110-116`; `ledger/probes/coproduct/**`.
Evidence: the catalog-hash list covers migration, rule-lens-extracted, naturality, composition — not `probe-coproduct`; `grep -rn 'coproduct' crates/catherine-ledger/tests/` returns nothing, while `probe_naturality.rs` and `probe_composition.rs` pin hashes/derived verdicts for their probes. `ledger-check ledger/probes/coproduct` is manual-only (`just ci` does not run it).
Impact: the surviving probe entry's fixture hashes and `survived` verdict can drift silently.
Recommendation: add `probe_coproduct.rs` mirroring `probe_naturality.rs` (fixture hashes, catalog hash, control supersession/self-negation, recorded == derived verdict), and add `probe-coproduct` to the ratchet list.

[CORR-004] [LOW] — Handoff records a stale HEAD and ahead-count
Location: `.whisper/branches/main/notes.md` (arc handoff entry).
Evidence: the entry says `HEAD: 83651a9` and "~25 ahead"; actual after the handoff commit is `08bf44f` and `git rev-list --count origin/main..HEAD` = 26.
Recommendation: cite the pre-mirror commit explicitly or note that the mirror commit follows.

[CORR-005] [LOW] — Disposition table registration counts read as current
Location: `docs/ref/expressibility-dispositions.md:59-60`.
Evidence: "Counts at registration: 12 rows, 12 `gap`, 0 `closed`" while three rows are now `closed`; the following sentence does scope it to registration and the suite asserts the live split.
Recommendation: drop the numeric sentence or restate as "at registration (see git history)".

[CORR-006] [LOW] — `naturality` absent-map warning has no requirement
Location: `crates/catherine-cli/src/runners/naturality.rs` (warning), `openspec/specs/instance-morphism/spec.md`.
Evidence: the warning is pinned only by `negative_matrix_naturality`; no deployed scenario/requirement mentions an absent `instance_map` run.
Recommendation: add one sentence to the instance-morphism requirement (no claim → warning, exit 0) or record it as an unreferenced additive surface.

Correctness Quality: GOOD

### Convergence (after Stage 2)
```
New CRITICAL issues: 0
Total new issues: 6
New issues vs Previous Stage: — (baseline)
Estimated false positive rate: 0% (all six re-executed/verified; TypeSafe gate not required below HIGH)
Status: CONTINUE
```

## STAGE 3: CLARITY

Issues Found:

[CLAR-001] [MEDIUM] — Docs-gate phrase list contains generic phrases
Location: `scripts/check-expressibility-claims.sh:33-43`.
Description: the list includes `is not checked`, `checks clean`, `is never evaluated`, `hand-picked`, `accepted and ignored`. These were needed to make all 12 registered claim cells trip the gate, but they match ordinary prose ("the run checks clean", "hand-picked observation") in README/CHANGELOG/`docs/**`/close notes.
Impact: future documentation will fail `just ci` and the only remedy — naming a row id — may be wrong for prose that is not a limitation claim; the gate becomes noise and gets weakened.
Evidence: current tree scans 0 claim lines, so no immediate failure; the risk is prospective.
Recommendation: require a limitation context alongside the generic phrase (tool subject + modal/negation), or keep generic phrases only in the table cells by feeding claim cells to the synthetic path (already tested) and scanning docs with the specific vocabulary only.

[CLAR-002] [LOW] — USING seam recipe predates the joint check
Location: `docs/USING.md:128-151`.
Description: Recipe D demonstrates the seam claim as one document; it does not mention `check a b`, `source` attribution, or `empty_composition_interface`, which are the G1 value proposition.
Recommendation: add a "when the claim spans documents" paragraph with the joint invocation and the warning.

Clarity Quality: GOOD

### Convergence (after Stage 3)
```
New CRITICAL issues: 0
Total new issues: 2
New issues vs Previous Stage: −67%
Estimated false positive rate: 0%
Status: CONTINUE (rate above 10%, but severity dropped to one medium + one low)
```

## STAGE 4: EDGE CASES

Issues Found:

[EDGE-001] [LOW] — Duplicate input paths duplicate findings without a warning
Location: `crates/catherine-cli/src/composition.rs` (glue), `run.rs` (variadic dispatch).
Scenario: `catherine check bowling.ct.json bowling.ct.json` (a typo or a scripted glob).
Evidence: one violation single-doc; two identical violations when the same path is passed twice; `source` lists the document once. This follows the documented repeated-declaration-multiset rule, but nothing tells the user the input was duplicated.
Recommendation: emit a labeled warning (`duplicate_document`) when an identical path appears twice, or dedupe paths after canonicalization.

[EDGE-002] [LOW] — Zero-row coproduct object is clean and silent
Location: `crates/catherine-core/src/checks.rs:288-290`.
Scenario: declared coproduct whose object has no rows, injections empty.
Evidence: `check` exits 0 with no findings. This is vacuous-but-legitimate (empty sum), and it is the one clean path the feature should keep; recorded so the gap-suite `vacuous-clean` row does not later absorb it.
Recommendation: no change; note the intent in the kernel-checks requirement ("an empty coproduct object is vacuously covered").

[EDGE-003] [LOW] — Empty `coproducts: []` accepted as omitted
Location: `crates/catherine-core/src/parse/coproducts.rs`.
Evidence: `{"coproducts": []}` parses and is omitted from canonical bytes (consistent with keys). No double-report or hash drift.
Recommendation: none; already documented by the omitted-when-empty requirement.

Edge Case Coverage: GOOD

### Convergence (after Stage 4)
```
New CRITICAL issues: 0
Total new issues: 3
New issues vs Previous Stage: +50% (all LOW; different class — input hygiene and vacuity)
Estimated false positive rate: 0%
Status: CONTINUE (no critical; polish class only)
```

## STAGE 5: EXCELLENCE

Final Polish Issues:

[EXCL-001] [LOW] — Dead parameter in gap-suite runner API
Location: `crates/catherine-cli/tests/expressibility_gaps/runner.rs` (`closed_rows_cite_fixing_evidence(_cases)`, and sibling entry points that ignore the case slice).
Description: the parameter is kept for a uniform signature; it invites readers to believe the check is case-dependent.
Recommendation: either use it (assert every closed row has a case) or drop it and let `every_case_is_registered` own that invariant.

[EXCL-002] [LOW] — Version list duplicated between spec and crate README
Location: `openspec/specs/model-document/spec.md:20`, `crates/catherine-core/README.md:79`.
Description: two hand-maintained lists of the additive minors; CORR-002 is the first drift and will recur.
Recommendation: make the spec the single list and have the README point at it (or generate the line at build time).

[EXCL-003] [LOW] — Probe fixture hashes pinned in both tests and entries
Location: `crates/catherine-ledger/tests/probe_{naturality,composition}.rs`, `ledger/probes/*/**`.
Description: intentional content-addressing ratchet; noted because three probes now repeat the pattern and a helper would remove about 40 duplicated lines.
Recommendation: extract a shared `probe_assertions` test module when the fourth probe lands.

Excellence Assessment:
- Structure: EXCELLENT
- Correctness: GOOD
- Clarity: GOOD
- Edge Cases: GOOD
- Overall: GOOD

Production Ready: WITH_NOTES

## Final Report

**Total Issues by Severity**
- CRITICAL: 0 — must fix before proceeding
- HIGH: 0
- MEDIUM: 4 (CORR-001, CORR-002, CORR-003, CLAR-001)
- LOW: 12 (DRAFT-001/002, CORR-004/005/006, CLAR-002, EDGE-001/002/003, EXCL-001/002/003)

**Top 3 findings**

1. CORR-001 — Coproduct suppression withholds decidable disjointness violations.
   Impact: a declared claim is not executed while any injection cell dangles; the spec justifies it with an "undecidable" statement that is false for rows already in two complete images.
   Fix: narrow suppression (dual coverage over complete images; uncovered only when no unresolved cell could reach the row), or reword and accept the deferral — plus a test for the dual-hidden fixture.

2. CORR-002 — Core README says `KNOWN_MINOR` is 4/1.4; the parser accepts 1.5.
   Impact: user-facing contract drift in the exact class the gap suite was built to prevent.
   Fix: extend the list to 1.5 and single-source it from the spec.

3. CORR-003 — The coproduct probe is the only probe without a ledger regression test and without a catalog-hash ratchet entry.
   Impact: the `survived` verdict and fixture hashes can rot silently.
   Fix: add `probe_coproduct.rs` and the ratchet id, mirroring naturality/composition.

**Stage-by-stage quality**
- Stage 1 (Draft): EXCELLENT — one change per capability, clean seams, no second pipeline.
- Stage 2 (Correctness): GOOD — all five tickets' contracts are honored; one over-broad suppression and two documentation/test ratchets.
- Stage 3 (Clarity): GOOD — docs gate and USING need scoping.
- Stage 4 (Edge Cases): GOOD — duplicate-path hygiene is the only live edge.
- Stage 5 (Excellence): GOOD — minor deduplication opportunities only.

**Recommended actions**
1. Resolve CORR-001 one way or the other in `add-coproduct-exhaustiveness`'s record (it is archived; a superseding change or a defect issue with the decided semantics), and add the dual-hidden fixture.
2. Fix the core README version list (CORR-002) and remove the duplicated list (EXCL-002).
3. Add `probe_coproduct.rs` + ratchet entry (CORR-003).
4. Narrow the docs-gate generic phrases (CLAR-001).
5. Optional polish: cross-capability integration test (DRAFT-001), USING recipe updates (DRAFT-002/CLAR-002), duplicate-path warning (EDGE-001), handoff/table wording (CORR-004/005).

## Verdict

**READY (WITH_NOTES)**

**Rationale:** no critical or high finding survived any stage and the full gate set is green at `08bf44f` (`just ci`, `openspec validate --all/--changes --strict`, `ah check`, four probe ledgers), so the arc is production-ready as landed; the notes are one over-broad coproduct suppression whose spec rationale overstates undecidability, one user-facing version-policy drift, one missing probe ratchet, and a docs-gate scoping risk — all MEDIUM/LOW and individually small.

**Post-remediation verdict: READY** — every finding below is closed; the
gate set is re-verified green on the remediation HEAD.

## Remediation (post-review)

All findings are closed. Documentation fixes landed in `99587df`; the
behavioural/test fixes landed in `7d918c6`, `68c2bf5`, `06ea916`, `c3a58ba`.

Documentation (commit `99587df`, review remediation `6571f8f`):

- **CORR-002 / EXCL-002** — core README version policy is 5/1.5 and now
  points at the single authoritative list in the deployed model-document
  spec.
- **CLAR-002 / DRAFT-002** — USING gains the joint-check paragraph in
  Recipe D and Recipe E for `naturality`/`instance_map`; the stale "once
  P10 lands" clause is replaced by the executed weakening witness.
- **README producer claim** — the analysis classes have real producers;
  the roadmap records P12 and the remaining work.
- **Book completeness** — SUMMARY gains the expressibility-dispositions
  reference and the four missing generated spec pages; mdBook builds.
- **THEORY** — new §10 (instance morphisms, coproducts, composition) and
  glossary entries; DESIGN/PROPOSAL annotated as historical records.

Behaviour and tests (commits `7d918c6`, `68c2bf5`, `06ea916`, `c3a58ba`):

- **CORR-001 / EDGE-002** (`7d918c6`) — an unresolved injection cell makes
  coverage undecidable, but a row in two complete images is now a decided
  disjointness violation; partial images never join a witness; the deployed
  requirement states the precise rule and notes the empty-object vacuity.
- **catherine-3en / CORR-005** (`68c2bf5`) — a non-empty `instance` object
  with neither `rows` nor `tables` is refused with the structural `syntax`
  error naming the keys (the legacy shape can no longer read as empty); the
  gap row is re-scoped to zero-row vacuity; the registration-count sentence
  is marked as history.
- **catherine-ke1 / EDGE-001** (`06ea916`) — `naturality` reports
  both-sides-undefined squares as `unresolved_naturality_square` warnings
  (arrow + row, JSON and human) instead of a silent clean; variadic
  surfaces warn `duplicate_document_input` on repeated paths without
  deduplicating findings.
- **CORR-003 / CLAR-001 / DRAFT-001 / EXCL-001 / EXCL-003 / CORR-006**
  (`c3a58ba`) — probe-coproduct regression test + catalog ratchet, the
  re-sealed control `0002-384db903` with the canonical fixture hash, shared
  probe-test plumbing, context-scoped generic gate phrases with a
  false-positive test, the combined-surface integration test, the
  absent-map requirement + contract, and the gap-runner signature cleanup.

- **CORR-004** — superseded by the follow-up handoff entry that cites the
  post-remediation HEAD and commit count.
- **EDGE-003** accepted as documented (`coproducts: []` = omitted, the keys
  precedent).

Verification at remediation HEAD: `just ci` green (fmt, clippy -D warnings,
workspace tests, check-gap-claims, wasm, pretender); `openspec validate
--all --strict` 15/15; `--changes --strict` 1/1; `ah check` clean; all four
probe ledgers valid. Defects `catherine-3en` and `catherine-ke1` closed.
