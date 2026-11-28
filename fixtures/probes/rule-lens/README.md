# Rule-lens selector-collision probe (pre-registration)

Committed BEFORE any run (RED). Ticket `catherine-8qm`; arm specifications,
exact inputs, commands and the ledger plan: `fixtures/probes/rule-lens/arms.md`.
Protocol precedents: `fixtures/probes/bench/README.md` (the three-arm bench,
kw9.6/kw9.11) and `fixtures/probes/selector-state/README.md` (the
domain-informing probe, kw9.5).

License: wave E1 (differentiation must be demonstrated) + E4 (unique-
detection claims require an executed conventional baseline); the P6 gate is
satisfied (`catherine-9vv` closed, survived-scoped). Negative control of the
license discipline: the FizzBuzz class — a computed predicate with no stored
counterpart (E6 below) that no arm may fire on; an arm firing there is a
scoring alarm, not a detection.

## Domain decision (recorded here; not re-litigated)

The bounded domain is the **bogart selector→capture state subdomain** — the
frozen 6-capture / 3-collision slice + census already committed at
`fixtures/probes/selector-state/` (kw9.5). Reused as-is; **no re-extraction**.
The frozen artifacts are cited by the hashes recorded at their extraction and
verified unchanged before and after this probe's runs (the executed model
hash is the GREEN-run loader fix, per the bench custody note).

kw9.5 fixed the defect class: **two stored capture rows sharing one artifact
`sha256` under one selector** — a selector matched by ≥2 stored capture rows
whose rows carry the same content hash. The model-check surface (path
equations + totality over stored arrows) has no cardinality claim over stored
arrows, so the class is invisible there — kw9.5 verdict `killed`
(`ledger/probes/selector-state/0002-5c22c00e`); kw9.11 extended the killed
result to the full three-arm comparison
(`crates/catherine-ledger/ledger/probes/bench/selector-state/0002-48bb05fe`).
This probe carries the check into the **rule-lens/policy layer**, where a
cardinality claim over rule tables *is* declarable: a rule table over capture
rows (one rule per row, column = the row's selector, action = the row's
artifact) with **`Policy::DeclaredDisjoint`** — the policy under which two
rows accepting one input with the same declared action is a violation class,
not an order-resolved shadow.

## The rule under test (verbatim)

> A selector is **collided** when two or more stored capture rows count for
> it (source match + edition-date or recorded alias match, the census rule
> verbatim) and those rows carry the same content hash — two stored rows for
> one artifact.

Census semantics are the prototype's, verbatim:
`prototype/bogart_storage_granularity.py --census`.

The same rule statement is available to every arm. Arm A implements it as the
census criterion (selector matched by ≥2 stored capture rows); arm B states
it as the rule-lens collision claim — two rule rows sharing one input key
under one selector with the same action (the artifact) — under the
collision-sensitive policy. Arm B's rule rows are a **transcription**, not a
computation: each rule's predicate and action are copied cell-for-cell from
the frozen instance's `capture_selector` / `capture_artifact` tables; the
cardinality evaluation (≥2 rows per selector, equal actions) is done by the
model check, never baked into the table. A rule table row that cannot be
derived byte-wise from the frozen cells would be a computed row — D3-invalid,
scored FP (the instance-custody rule of the bench, carried over).

## Custody (frozen; cited, not re-hashed)

All frozen hashes are quoted from the provenance tables in
`fixtures/probes/selector-state/README.md` and
`fixtures/probes/bench/README.md`; they are not recomputed here — except that
both are re-verified unchanged after the GREEN run (acceptance requirement).

| Artifact | Path | sha256 (as recorded) |
|---|---|---|
| Frozen census (scoring-only ground truth) | `fixtures/probes/selector-state/census.json` | `154913904e7544930b28dda7e8bca3ae903a0bfb06342e937fd67b8967e4be3d` |
| Frozen slice rows (the only state arms consume) | `fixtures/probes/selector-state/slice.json` | `840e81c843ce371fbdb73c071dda4a9080f94bbb14da016d2f72dc065a5280c5` |
| Arm A script (committed, version-pinned in kw9.11) | `prototype/bogart_selector_bench.py` | `df8106581e2ff3eec83861372f0862decebca8b3e9874e951161f50d7553c762` |
| Arm B model + instance (frozen) | `fixtures/probes/selector-state/selector-state.ct.json` | `c4bf0bbf15c2015b769e45adfad656bf48e7e70d3031441b52755f5664c9f958` (executed hash; the RED-commit hash `3cc5314d…` is the bench's loader-defect supersede, bench README custody table) |
| Floor fixture (collision-free sub-slice) | `fixtures/probes/selector-state/selector-state-floor.ct.json` | `04517b828eb42948418cafa22dacb35a60131d9eeac2f38cc832d2038908750b` |
| Floor selectors (127 zero-match negatives) | `fixtures/probes/selector-state/floor-selectors.json` | `b26c5ea178ddb381226874adbc428c00e2c357c6f11bf2e01276dc28fe6dd32b` |
| Sanity seed S1 (4th-collision model, kw9.11) | `fixtures/probes/bench/seeds/selector-state-seed-s1.ct.json` | `5f3f0b3dee2d41484d2c5bd8cd38d8b5c5bfa36a7ab606bd91be169d2f90e1a0` |
| Sanity seed S2 (dangling-reference model, kw9.11) | `fixtures/probes/bench/seeds/selector-state-seed-s2.ct.json` | `8ec7f0963b6a93fd2d1e721106fbcd7c915286eeb373a4f09ae34ccd7946ec50` |

Derived artifacts (produced at GREEN, never in place; frozen files untouched).
Each is the named frozen model **verbatim** plus an additive
`rule_presentation` section (format 1.2: `domain`, `columns`, `rules`,
`policy: "DeclaredDisjoint"`) whose rules are the transcription described
above — the derivation command is fixed in arms.md, and the produced hashes
are recorded in the ledger entries at run time:

| Derived artifact | From |
|---|---|
| `fixtures/probes/rule-lens/selector-state-rules.ct.json` | frozen `selector-state.ct.json` + 6 transcription rules |
| `fixtures/probes/rule-lens/selector-state-rules-floor.ct.json` | floor `selector-state-floor.ct.json` + empty capture table → 0 rules |
| `fixtures/probes/rule-lens/selector-state-rules-seed-s1.ct.json` | seed S1 model + 8 transcription rules (3 collided selectors + `bench-seed/s1`) |
| `fixtures/probes/rule-lens/selector-state-rules-seed-s2.ct.json` | seed S2 model, same derivation (the dangling cell stays in the instance tables) |

Custody separation (unchanged from the bench): `census.json` is scoring-only
ground truth — no arm consumes it. The slice rows in `slice.json` are the
only state arm A consumes; the frozen ct models are the only state arm B
consumes. Live databases are never opened; the probe is offline over frozen
data.

## The two arms

| Arm | Implementation | Artifact | Status at pre-registration |
|---|---|---|---|
| **A — conventional suite (the control)** | The census criterion as an offline script over the frozen slice: `prototype/bogart_selector_bench.py`, census match semantics verbatim, detections = selectors matched by ≥2 stored capture rows | committed and version-pinned in kw9.11 (`df810658…`); the committed-executed-baseline precedent is `prototype/bogart_storage_granularity.py --census` | committed; executed again in this probe (an executed baseline is required for the verdict — a cited-only baseline caps at `inconclusive`) |
| **B — catherine rule-lens/policy surface** | The new rule-lens selector-collision analysis (ticket `catherine-8qm`): over `catherine rules`, two rule rows sharing one input key under one selector with the same action is a violation finding under `Policy::DeclaredDisjoint`; under `PriorityOrder`/`FirstMatch` the pair is order-resolved (shadow-class), never a collision; scope overflow is `UNKNOWN { scope }`, never a partial finding set | the derived `*-rules.ct.json` models above + the projection bridge (`catherine rules <model> --project capture --columns capture_selector --action capture_artifact`), which fixes `DeclaredDisjoint` by construction | to be implemented and executed after this commit |

The negative control **is** arm A (the executed conventional baseline), as in
the bench. Arm B is the arm under test: the probe asks whether the policy
layer can state the cardinality claim the check surface could not.

## Pre-registered expectations (falsifiable)

Arm classes: **finding** / **silent** / **N/A** (out of scope by
construction). Detections are `check`/`rules`-surface violation findings
(exit 3 class); `underconstraint` hypotheses (exit 6) and load-time labeled
errors (exit 1) are recorded verbatim and scored as non-findings — except
where a row's registration requires a non-zero exit (S2), which is a sanity
outcome, never a detection. Exit 3 is the only detection exit for arm B.

| # | Row | Fault class | Arm A must | Arm B must | Falsified by |
|---|---|---|---|---|---|
| E1 | selector collision `nacional/primera 2026-11-03` (2 capture rows, 1 sha256) | D1 defect | **finding** (match_count 2) | **finding** — one collision violation naming the two rows and the shared input | any other class in any arm; arm B silent, or a finding naming a non-defect row |
| E2 | selector collision `pba/JUDICIAL 2026-11-03` (2 capture rows, 1 sha256) | D1 defect | **finding** | **finding** — same shape as E1 | same as E1 |
| E3 | selector collision `nacional/segunda 2026-11-13` (2 capture rows, 1 sha256; corpus-side only) | D1 defect | **finding** | **finding** — same shape as E1 | same as E1 |
| E4 | zero-match negatives — 67 recorded GT / 127 frozen (denominators named; scoring uses the frozen 127) | negative | **silent** (floor run: 0 detections over the 127) | **silent** — no collision violation naming a zero-match selector (the floor rules model has an empty capture table; any exit-6 coverage hypothesis over the declared 127-selector domain is a non-finding, recorded) | any collision violation naming a zero-match selector (FP) |
| E5 | transcription-only rule table — derived relations (match-count histograms, edition rollups) are prose intent only | silence class | **N/A** — the census IS the rule-based machinery | **silent** — the rules table carries no computed row; a collision finding attributable to a computed row is D3-invalid | any arm-B finding attributable to a computed/derived row (D3-invalid, scored FP, removed from the accounting) |
| E6 | FizzBuzz-class negative control — a computed predicate with no stored counterpart (e.g. "no selector's match count is divisible by 3") | negative control | **silent** — the census script implements no such predicate | **silent** — the rule-lens collision analysis computes no such predicate | any finding attributable to a FizzBuzz-class expectation in **any** arm = scoring alarm → entry invalid-run |
| S1 | seeded 4th collision — new selector `bench-seed/s1 2026-11-03` holding a duplicated capture pair (the kw9.11 seed, reused verbatim; sanity, outside defect accounting) | sanity seed | **finding** — must report 4 collision selectors (3 real + seed); missing the seed ⇒ arm-A scoring invalid | **finding** — must report 4 collision violations incl. `bench-seed/s1`; missing the seed ⇒ arm-B scoring invalid | a missing seed in either arm; B firing on a non-seed row it does not fire on unseeded |
| S2 | seeded dangling reference — capture #1's `sha256` repointed to a nonexistent artifact (the kw9.11 seed, reused verbatim; expressible fault) | sanity seed | **N/A** — the census makes no referential claim (scope-gap) | **finding** — must fire non-zero (the dangling `capture_artifact` cell is caught at load on this loader) | silence from B ⇒ battery vacuous ⇒ arm invalid-run |

Policy-gate expectations (arm B, pre-registered): on the same instance rows,
a rule table declared `PriorityOrder` or `FirstMatch` must produce **no
collision finding** (the pair is order-resolved — shadow-class, not a
collision; this is the expressible-policy boundary the probe pins). Recorded
as a supplementary run over a policy-variant model; its class is
**silent** on E1–E3 and its falsifier is any collision finding under a
priority policy.

## Defect accounting (two-arm shape)

Per arm X ∈ {A, B}, over rows E1–E3 (defect ids) and E4 (negatives):

- `TP_X` — defect rows on which arm X fired a real detection.
- `FP_X` — findings on non-defect rows (E4) plus D3-invalid findings (E5/E6).
- `unique_X = TP_X ∖ TP_A` — set difference over defect ids against the
  executed control, recorded in the ledger entry's
  `comparison.unique_detections`. For this probe the registered expectation
  is **parity with full coverage**: `unique_B = ∅` and `TP_B = {E1,E2,E3}`.
- The S-rows count in sanity pass/fail only, never in TP/FP (bench
  discipline).

## Decision rule (fixed in advance)

The closed verdict vocabulary: `killed | survived | inconclusive |
invalid-run`.

| Observed outcome | Verdict |
|---|---|
| B fires real collision findings on **all** of E1–E3 over instance rows verbatim from the frozen slice; `unique_B = ∅` (parity against the executed A); no D3-invalid; no FP on E4; S1/S2 pass; floor self-negates; the policy-variant run is silent | **survived** — the selector-collision cardinality claim is expressible and executed on the rule-lens/policy surface; slice-scoped, and the stored-state check surface's kw9.5 silence is unchanged (it has no such claim) |
| B fires on a **subset** of E1–E3 with zero FP | **inconclusive** — partial detection is an analysis defect; fix, supersede, re-run |
| B silent on E1–E3 with zero FP | **killed** — the rule-lens surface cannot state the claim either; the disposition row stays `gap` |
| Arm A cited but not executed at run time | **inconclusive** — uniqueness/executed-baseline ceiling per the domain-probes spec |
| Any FP on E4/E5/E6; any arm fires on the floor; A or B misses S1; B silent on S2; any E6 alarm | **invalid-run** — scoring validation failed; fix the harness, supersede, re-run |
| Tool / parse / load failure outside a registered expectation | **invalid-run** for that arm; the probe is inconclusive until re-run |
| Outcomes mix such that no row above applies | **inconclusive** |

A floor run that fires (any collision finding over the collision-free model)
means invalid-run — the floor entry self-negates first (verdict `killed`,
`control: true`), then the probe entry follows.

## Power and UNKNOWN labels (boundary)

Powered for: **existence on this slice** — is the selector-collision class a
statable, executable claim on the rule-lens/policy surface. Underpowered for,
therefore UNKNOWN-labeled wherever claimed: detection-rate generalization
(3 defect instances), domains beyond the bogart selector→capture state (one
domain, one frozen snapshot), and the stored-state check surface (kw9.5's
killed result is untouched — this probe adds no claim there). A survived
verdict here closes the *expressibility* question on this slice only; the
class is expressed over a **declared rule table** (a transcription of stored
cells under `Policy::DeclaredDisjoint`), not over stored instance rows — the
disposition row records precisely that boundary if it flips.

## Protocol Review (RED — rubric review, done at design time)

1. **Arm classes completed to the bench shape.** First draft registered only
   E1–E3. Completed with E4 (negatives), E5 (transcription-only silence), E6
   (FizzBuzz control), S1/S2 — every fault row carries a per-arm class from
   {finding, silent, N/A}, and the uniqueness column is the arithmetic
   `TP_X ∖ TP_A`, not prose.
2. **Transcription vs computation made a stated rule.** Arm B's rule rows are
   copied cells; the counting stays in the checker. Without this, the probe
   would re-state the bench's instance-custody defect (a computed row would
   launder the census through the model check — D3-invalid, FP).
3. **Policy gate pinned.** The collision is a finding only under
   `DeclaredDisjoint`; under a priority policy the same pair is
   order-resolved. Registered as its own supplementary expectation so the
   probe pins the boundary, not just the detection.
4. **Exit-code scoring enumerated in advance.** Only exit-3 violations are
   detections; exit-6 hypotheses and exit-1 load errors are non-findings —
   except S2, whose registration *requires* a non-zero exit (a sanity
   outcome, never a detection). Mirrors the bench's D2 discrepancy so this
   probe does not rediscover it as a surprise.
5. **Executed baseline required.** Arm A is executed at run time, not cited;
   a cited-only baseline would cap the verdict at `inconclusive` (the
   domain-probes spec), so the probe runs it.
6. **Floor ordering.** The floor control is entry 0001, self-negating
   (`control: true`, verdict `killed`); the probe entry 0002 follows. A floor
   firing is invalid-run, never scored.
7. **Seed reuse, no re-seeding.** S1/S2 reuse the kw9.11 seed models verbatim
   (hashes above); this probe re-derives only the rule-presentation sections
   over them. No second seeding scheme, no divergent seeds.

No prediction was edited after any run: there has been no run.

## Ledger plan

New sequence `ledger/probes/rule-lens/` (content-addressed ids
`NNNN-<8hex>.json`, sealed via the ledger's own `seal` discipline, validated
with `ledger-check` on the new directory and its neighbors):

- `0001-<id>.json` — floor control (`control: true`, self-negating, verdict
  `killed`): arm A floor run (0 detections over the 127) + arm B floor run
  (no collision finding over the collision-free rules model).
- `0002-<id>.json` — probe entry: the expectations table above, every command
  verbatim with exit codes, the two-arm accounting, the
  `comparison.conventional_baseline` (arm A artifact sha256 + its executed
  detections), `expressibility_gap[]` citing the kw9.5 ledger 0002
  (`ledger/probes/selector-state/0002-5c22c00e.json`) for why the check
  surface had no cardinality claim, `discrepancies[]` verbatim, the
  `procedure[]`, and the verdict from the decision rule.

The validator runs on the new sequence; a `throwaway` validator invocation is
acceptable where no bin flag covers a neighbor sweep (the bench precedent),
and the throwaway is deleted after the run.
