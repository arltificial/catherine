# Three-arm bench — bogart selector→capture state (pre-registration)

Committed BEFORE any run (RED). Ticket `catherine-kw9.6`; execution lives in
the sibling `catherine-kw9.11`. Protocol precedents:
`fixtures/probes/granularity-bogart/README.md` (pre-registration before the
run, decision rule fixed in advance) and `fixtures/probes/selector-state/
README.md` (the domain-informing probe). Arm specifications, exact inputs,
commands, scoring rubric and the ledger plan:
`fixtures/probes/bench/arms.md`.

License: wave E1 (differentiation must be demonstrated) + E4 (unique-
detection claims require an executed conventional baseline); the P6 gate is
satisfied (`catherine-9vv` closed, survived-scoped).

## Domain decision (recorded here; not re-litigated)

The bounded domain is the **bogart selector→capture state subdomain** — the
frozen 6-capture / 3-collision slice + census already committed at
`fixtures/probes/selector-state/` (kw9.5). Reused as-is; **no re-extraction**.

Rationale, from the kw9.5 outcome: the probe ran the committed stored-column
model over the frozen collision slice and the check fired **nothing** —
verdict `killed` (ledger/probes/selector-state/0002-5c22c00e). The triage
fixed the cause: the model-check surface (path equations + totality over
stored arrows) has **no cardinality claim**, so the selector-collision defect
class — two stored capture rows sharing one artifact `sha256` under one
selector — is not a violation of anything declarable, and declaring the
collision as an equation would be a derived relation, not stored state (the
P3 boundary). In other words, the model check is **silent on the defect
class by construction**. Meanwhile the rule-based census is the machinery
that *found* the class (three ≥2-match selectors, the pilot's 2 real
`bogart-4wt` defects plus `nacional/segunda 2026-11-13`).

The bench therefore does not ask whether the model check detects the class —
kw9.5 answered that. It asks whether **any** arm detects it, scored as a
pre-registered three-way defect accounting with the **conventional suite as
the control arm**. This is the wave's central UNKNOWN instantiated once:
no independent controlled comparison of a categorical implementation and a
competent conventional suite on detection rate exists anywhere (E4: ct-catlab
never beat a competent conventional suite on detection rate; E1:
differentiation must be demonstrated). One bounded comparison is what this
protocol supplies; it generalizes no further (see Power and UNKNOWN labels).

## The rule under test

A selector is **collided** when two or more stored capture rows count for it
(source match + edition-date or recorded alias match, the census rule
verbatim) and those rows carry the same content hash — two stored rows for
one artifact. Concrete shape in the slice: 3 collision selectors, each with
exactly 2 stored `backfill_captures` rows sharing one `sha256` and differing
only in `job_id`/`revision`. Census semantics are the prototype's, verbatim:
`prototype/bogart_storage_granularity.py --census`.

The same rule statement is available to every arm: arm A implements it as
the census criterion (selector matched by ≥2 stored capture rows); arm B
inherits it from the committed model + the kw9.5 README; arm C receives it
as pre-registered prose (verbatim in arms.md — the rule is stated, the
ground truth is not: no selector names, no counts).

## Custody (frozen; cited, not re-hashed)

All hashes are quoted from the provenance table in
`fixtures/probes/selector-state/README.md`; they were recorded there at
extraction and are not recomputed here.

| Artifact | Path | sha256 (as recorded) |
|---|---|---|
| Frozen census (ground truth for scoring) | `fixtures/probes/selector-state/census.json` | `154913904e7544930b28dda7e8bca3ae903a0bfb06342e937fd67b8967e4be3d` |
| Frozen slice rows (the only state arms consume) | `fixtures/probes/selector-state/slice.json` | `840e81c843ce371fbdb73c071dda4a9080f94bbb14da016d2f72dc065a5280c5` |
| Arm B model + instance | `fixtures/probes/selector-state/selector-state.ct.json` | `3cc5314d614d57e4eb8fa33420a2b1ee582c4e548e2268ce97bb23b72143163a` (RED commit `c1b7cb34ab`; the GREEN run's loader exposed a load-time defect, fixed on disk, executed hash `c4bf0bbf15c2015b769e45adfad656bf48e7e70d3031441b52755f5664c9f958`) |
| Floor fixture (collision-free sub-slice) | `fixtures/probes/selector-state/selector-state-floor.ct.json` | `04517b828eb42948418cafa22dacb35a60131d9eeac2f38cc832d2038908750b` |
| Serving projection (provenance only; arms never open it) | `bogart/data/backfill/delivery-2026-11-11-november/serving.sqlite` | `30ec194777efc912c4848e06faca1e7451d3fb192690856e51ef98e07a413c34` |
| Batch working corpus (provenance only; arms never open it) | `bogart/data/backfill/corpus.sqlite` | `1ac681dbe8874643c4c40d4a5bce2a4f0cefbe9bbf22deaa33c6448774f000b5` |

Custody separation (scoring hygiene): `census.json` is **scoring-only**
ground truth — no arm consumes it. The slice rows in `slice.json` are the
only state any arm consumes. Live databases are never opened by any arm;
the bench is offline over frozen data.

Denominator note (disclosed drift, carried over from kw9.5): zero-match
selectors are **67** in the recorded ground truth (2026-11-13) and **127**
in the frozen snapshot (+60 distinct `backfill_partitions` selectors added
by the running backfill after the recording; the table carries no timestamp,
so the recorded set is not reconstructable). Scoring against frozen data
uses the frozen **127**; the recorded 67 is named wherever the two could
differ, and the drift note lives in `census.json`.

## The three arms

| Arm | Implementation | Artifact | Status at pre-registration |
|---|---|---|---|
| **A — conventional suite (the control)** | The census criterion as an offline script over the frozen slice: sibling `prototype/bogart_selector_bench.py`, census match semantics verbatim (`src==s and (ed==e or e in metadata 'edition-dates')`), detections = selectors matched by ≥2 stored capture rows | Named here; **committed and version-pinned in kw9.11** — the committed, executed conventional baseline precedent is `prototype/bogart_storage_granularity.py --census` (committed before the 2026-11-13/2026-11-25 runs) | Named, not committed (by design; kw9.11 owns it) |
| **B — catherine hand-built model** | The committed stored-column model over the slice instance | `fixtures/probes/selector-state/selector-state.ct.json` (hashes above) | Committed; executed once — kw9.5, exit 0, zero findings |
| **C — LLM-drafted model, attacked** | The kw9.9 draft→attack→fix loop (docs/ref/ai-draft-attack-recipe.md): prose in, LLM drafts, battery attacks, findings back, iterate to the recipe's stopping rule. **The arm's evaluated artifact is the recipe's output** — the final `draft-v<N>.ct.json` that stopped the loop | Loop transcript + final draft, produced in kw9.11 under `fixtures/attack-demo/bench-selector-state/` (naming per the recipe) | Recipe committed (v1); prose pre-registered below |

The negative control **is** arm A: the conventional suite is what the
categorical arms are measured against, not a strawman. A seeded-defect
sanity arm (S1/S2 below) validates that the scoring is neither vacuous nor
manufacturing detections.

## Pre-registered expectations (falsifiable)

Arm classes: **finding** / **silent** / **N/A** (out of scope by
construction — scope is recorded, never counted as silence). Detections are
`check`-surface violation findings; `underconstraint` hypotheses (exit 6)
and `search` payloads (witness / contradiction / UNKNOWN) are recorded
verbatim and scored as **non-findings** (see arms.md rubric).

| # | Row | Fault class | Arm A must | Arm B must | Arm C must | Falsified by | Unique detection vs arm A (arithmetic) |
|---|---|---|---|---|---|---|---|
| E1 | selector collision `nacional/primera 2026-11-03` (2 capture rows, 1 sha256) | D1 defect | **finding** | **silent** | **silent** | any other class in any arm | unique_B = TP_B ∖ TP_A = ∅ ; unique_C = TP_C ∖ TP_A = ∅ |
| E2 | selector collision `pba/JUDICIAL 2026-11-03` (2 capture rows, 1 sha256) | D1 defect | **finding** | **silent** | **silent** | any other class in any arm | unique_B = TP_B ∖ TP_A = ∅ ; unique_C = TP_C ∖ TP_A = ∅ |
| E3 | selector collision `nacional/segunda 2026-11-13` (2 capture rows, 1 sha256; corpus-side only — no serving rows in the delivery) | D1 defect | **finding** | **silent** | **silent** | any other class in any arm | unique_B = TP_B ∖ TP_A = ∅ ; unique_C = TP_C ∖ TP_A = ∅ |
| E4 | zero-match negatives — 67 recorded GT / 127 frozen (denominators named; scoring uses the frozen 127) | negative | **silent** — no zero-match selector flagged ≥2-match | **silent** | **silent** | any finding naming a zero-match selector, in any arm (scored FP) | none — negatives carry no TP; FP_X counted per arm |
| E5 | P3 rule-based silence — derived relations (match-count histogram, derived edition rollups) are prose intent only | silence class | **N/A** — the census IS the rule-based machinery; it computes derived stats and makes no stored-state claim | **silent** — `equations: []`; a finding attributable to a derived relation is D3-invalid | **silent** — same boundary; a draft instance carrying computed/derived rows is D3-invalid (instance-custody rule, arms.md) | any model-check finding attributable to a computed/derived relation (scored D3-invalid, counted as FP, flagged) | D3-invalid findings are removed from TP before the ∖ TP_A arithmetic (laundering control) |
| E6 | FizzBuzz-class negative control — a computed predicate with no stored counterpart (e.g. "no selector's match count is divisible by 3") modeled as prose intent only | negative control | **silent** — the census script implements no such predicate | **silent** | **silent** | any finding attributable to a FizzBuzz-class expectation in **any** arm = scoring alarm → entry invalid-run | none — alarm path, outside the accounting |
| S1 | seeded 4th collision — new selector `bench-seed/s1 2026-11-03` holding a duplicated capture pair (deterministic seeding, arms.md; sanity, outside defect accounting) | sanity seed | **finding** — must report 4 collision selectors (3 real + seed); missing the seed ⇒ arm-A scoring invalid | **silent** — the seed is the same inexpressible class; recorded expected-silent (validates the pipeline does not hallucinate a B detection) | **silent** — per C's E1–E3 registration | A missing the seed; B firing on it (falsifies the E1–E3 B expectations); C firing scored per C's rows | none — seed detections count in sanity pass/fail only, never in TP/FP |
| S2 | seeded dangling reference — capture #1's `sha256` repointed to a nonexistent artifact (deterministic seeding, arms.md; expressible fault) | sanity seed | **N/A** — the census makes no referential claim; recorded as scope-gap, not silence | **finding** — must fire (dangling, non-zero exit) | **finding** — must fire (this is the recipe's mandatory overconstraint control) | silence from B or C ⇒ battery vacuous ⇒ arm invalid-run | none — validates the batteries, not uniqueness |

Mechanism note on arm C's expectation (honest, registered in advance): the
recipe's stopping rule ends the loop only when every battery verb exits
clean on the current draft, and findings are fed back for the LLM to
**address**. The loop therefore converges toward silence-on-instance by
construction — on a defect-bearing slice that is detection-blindness unless
the collision obligation is statable as an executed stored-state claim. Arm
C's registered outcomes are thus: **silent** (loop converged — the expected
class), **invalid-run** (the loop cannot converge: the LLM's draft keeps
firing true findings it cannot remove without breaking the prose — itself an
informative outcome, recorded as loop-nonconvergence), or a **finding** on
the final draft (falsifies E1–E3-C; the only route to a real unique
detection, and per E5 it must survive the instance-custody rule to count).
kw9.11 tests which occurs; this protocol does not presume it.

## Defect accounting (three-arm shape)

Per arm X ∈ {A, B, C}, over rows E1–E3 (defect ids) and E4 (negatives):

- `TP_X` — defect rows (E1–E3) on which arm X fired a real detection.
- `FP_X` — findings on non-defect rows: any E4 row, plus all noise.
- `D3inv_X` — findings attributable to derived/FizzBuzz relations (E5/E6):
  counted, flagged, and **removed from TP** before any uniqueness
  arithmetic (they are scored as FP).
- `unique_X = TP_X ∖ TP_A` — **set difference over defect ids, against arm
  A** (the control). Uniqueness is never asserted rhetorically; it is this
  subtraction, recorded in the ledger entry's
  `comparison.unique_detections`.

The ledger entry records all six accountings plus the two subtractions.
Per the domain-probes spec: a non-empty `unique_detections` is refused
unless the entry names arm A as a committed, version-pinned artifact with
its detections on the same fault set; a cited-but-not-executed arm A caps
the verdict at `inconclusive`; with no baseline arm at all, `survived` is
possible only with an empty unique set (parity ceiling).

## Decision rule (fixed in advance)

| Observed outcome | Bench verdict (ledger 0002) |
|---|---|
| A detects all of E1–E3; B and C produce zero real detections; no D3-invalid; no FP on E4; S1/S2 pass; floor silent | **killed** — parity; the null result is the deliverable. The categorical claim stays UNKNOWN-labeled beyond this slice |
| B or C produces a real E1–E3 detection absent from **executed** A's detections — custody-clean, not D3-invalid | **survived** on that arm's `unique` set — slice-scoped; generalization stays UNKNOWN-labeled |
| Arm A cited but not executed at run time (sibling script uncommitted) | **inconclusive** — uniqueness ceiling per the domain-probes spec; unique sets recorded empty, the missing execution recorded as a gap |
| No baseline arm at all | **inconclusive** at the parity ceiling — no unique-detection claim may be recorded |
| Sanity fails its own registration: A misses S1; B or C silent on S2; any arm fires on the floor; any arm raises an E6 alarm | **invalid-run** — scoring validation failed; fix the harness, supersede, re-run. Never scored as an arm result |
| Tool / parse / load failure on any arm | **invalid-run** for that arm; the bench is inconclusive until re-run |
| Outcomes mix such that neither row above applies | **inconclusive** |

## Power and UNKNOWN labels (boundary)

Powered for: **existence on this slice** — does any arm produce a real
detection the executed conventional control misses. Underpowered for, and
therefore UNKNOWN-labeled wherever claimed: detection-rate claims (3 defect
instances), generalization beyond the bogart selector→capture state (one
domain, one frozen snapshot), and arm-C draft variance (one draft lineage;
the recipe is nondeterministic). A killed verdict here kills the categorical
claim **on this slice only** — it extends kw9.5's killed model-check result
to the full three-arm comparison; it is not a categorical refutation
anywhere else.

## Protocol Review (RED — rubric review, done at design time)

The expectations table was re-read against the three-arm defect-accounting
shape (every fault row must carry a per-arm class from {finding, silent,
N/A}; the unique-detection column must be arithmetic against arm A). The
review found and fixed, in this document, before any run:

1. **Uniqueness made arithmetic.** First draft phrased the uniqueness column
   in prose ("B cannot detect"). Rewritten as the set difference
   `TP_X ∖ TP_A` per row with the expected ∅ stated — the ledger records the
   subtraction, not a sentiment (domain-probes spec).
2. **Sanity rows split out of the accounting.** First draft mixed S1/S2 into
   TP/FP. Now: seed detections count in sanity pass/fail only; a B/C firing
   on S1 is a falsification event, an A miss on S1 is an invalid-run —
   neither ever enters the defect accounting.
3. **N/A introduced for arm A on S2 and E5.** First draft registered arm A
   "silent" on the dangling seed. Silence and out-of-scope are different
   observations; the census makes no referential claim, so A is N/A
   (scope-gap), and on E5 the census computing derived stats is the
   legitimate conventional machinery, not a silence subject. Counting scope
   as silence would have mis-scored the control arm.
4. **Negatives denominator pinned to the frozen snapshot.** First draft used
   the recorded 67 against frozen data. The frozen slice carries 127
   zero-match selectors (disclosed +60 partition drift); scoring uses 127,
   with the recorded 67 named (Custody section) — the recorded set is not
   reconstructable.
5. **Instance-custody rule added (laundering control).** Without it, arm C
   could manufacture "detections" by carrying computed rows (match counts,
   tallies) in its draft instance — the conventional computation laundered
   through the model checker. Rule: every instance row in a ct arm comes
   verbatim from the frozen slice; a computed row is D3-invalid, scored FP,
   and excluded from TP before the uniqueness arithmetic. Arm B satisfies it
   by construction (committed model).
6. **Non-findings enumerated.** `underconstraint` hypotheses (exit 6) and
   `search` payloads (witness / contradiction / UNKNOWN) are recorded
   verbatim but never scored as detections; only `check` violation findings
   are. First draft left this open, which would have let hypothesized
   underconstraints inflate B/C detection counts.
7. **S1 seeding shape corrected.** First draft duplicated a capture row in
   place — that deepens an existing collision (still 3 collision selectors,
   so the "4 collision selectors" expectation was unsatisfiable). The seed
   now introduces a **new** selector holding a duplicated pair, making the
   A-must-find-4 expectation well-formed.

No prediction was edited after any run: there has been no run.

---

## Result (kw9.11 — executed 2026-11-25)

Executed per arms.md at the kw9.11 pin. Binary pinned to `target/debug/catherine` (cargo build; the granularity lint rides `check`
unconditionally). Frozen artifacts untouched; seeded/floor/attack variants
written to new files.

### Findings table (executed, per row)

| # | Row | Arm A (conventional, executed) | Arm B (committed model) | Arm C (LLM draft, attacked) |
|---|---|---|---|---|
| E1 | collision `nacional/primera 2026-11-03` | **finding** (detection, match_count 2) | **silent** (check exit 0, findings []) | **silent** (check exit 0) |
| E2 | collision `pba/JUDICIAL 2026-11-03` | **finding** (detection, match_count 2) | **silent** | **silent** |
| E3 | collision `nacional/segunda 2026-11-13` | **finding** (detection, match_count 2) | **silent** | **silent** |
| E4 | zero-match negatives (127 frozen) | **silent** (floor run: 0 detections over the 127) | **silent** | **silent** |
| E5 | derived-relations silence | **N/A** (census computes derived stats; no stored-state claim) | **silent** (`equations: []`) | **silent** (`equations: []`) |
| E6 | FizzBuzz-class negative control | **silent** — the script implements no such predicate | **silent** | **silent** |
| S1 | seeded 4th collision | **finding** — 4 collision selectors (3 real + `bench-seed/s1`) — sanity pass | **silent** (expected; rows_evaluated 605) — pass | **silent** (rows_evaluated 478) — pass |
| S2 | seeded dangling reference | **N/A** (scope-gap; unseeded output, exit 0) | **fires** — exit 1, `error[syntax]`: `capture_artifact` maps `cap_f16110174…` to `art_bench_seed_s2_dangling`, not a row of `artifact` — pass | **fires** — exit 1, same class on `draft-final-seed-s2.ct.json` — pass |
| — | floor (entry 0001) | silent (0 detections over 127 selectors) | silent (rows_evaluated 127) | silent (rows_evaluated 127) |

### Scoring (the pre-registered rubric, verbatim arithmetic)

- Only `check`-surface violations are detections; `underconstraint` (exit 6)
  and `search` payloads are non-findings; the S2/control firings are
  load-time labeled errors (exit 1) — recorded verbatim, scored as sanity
  outcomes, never as detections (see Discrepancy 2).
- `TP_A = {E1, E2, E3}`, `FP_A = 0`, `D3inv_A = 0`
- `TP_B = ∅`, `FP_B = 0`, `D3inv_B = 0`
- `TP_C = ∅`, `FP_C = 0`, `D3inv_C = 0`
- `unique_B = TP_B ∖ TP_A = ∅`; `unique_C = TP_C ∖ TP_A = ∅`
- real/total findings ratios: A 3/3; B 0/0 (undefined); C 0/0 (undefined) —
  reported for comparability only; the decision rule runs on the arithmetic.
- E6 alarm check: no FizzBuzz-class predicate exists in any arm (the census
  script implements none; both ct models declare `equations: []`) — no alarm.

### Arm C loop transcript (kw9.9 recipe, v1, followed verbatim)

- `prose.md` = the pre-registered block byte-for-byte (sha256
  `a40e6c14934cd6446ebeff66266e3fff0f9e9988b6d6c1b1d8a283e3cc1a0569`);
  `instance-slice.json` = the frozen slice (sha256 `840e81c8…`, verified).
- **Drafter disclosure (honesty rule):** the drafting agent was this
  session's own model (GLM, Z.ai — the kw9.11 worker agent), not an
  independent session. It had earlier in the same session read
  `slice.json` while building arm A and the seeding harness, so the draft
  step was not prose-blind in the strict sense. The schema was written from
  the prose block only; the instance rows were taken verbatim from
  `instance-slice.json` (the prose's designated input).
- draft-v1 battery: `check` exit 0 (silent), `underconstraint --max-path-length 3`
  exit 0 (no hypotheses), `search` witness at shrunk bounds (natural bounds
  returned UNKNOWN within the 256-candidate default — bounds shrunk per the
  recipe, never declared success). No findings came back, so no revision was
  needed; the loop converged at `draft-v1.ct.json` once both negative
  controls were demonstrated: clean control = full battery silent on the
  unmodified draft; overconstraint control = injected dangling
  `page_document` cell caught non-zero (exit 1; Discrepancy 2).
  `naturality` not run — no `instance_map` declared (the verb is conditional).
- Instance-custody diff (laundering control): every final-draft instance row
  and table cell is byte-derivable from frozen `slice.json` rows — diff
  clean; no computed/derived rows; D3-invalid count 0; arm C recorded as
  independent on the instance.
- Final draft: `fixtures/attack-demo/bench-selector-state/draft-v1.ct.json`
  (sha256 `9c984a65af40f71c96901bac979a4e263c0514b409fdd25c5c314d337d035ff9`).

### Verdict

**killed** — the first decision-rule row applies exactly: A detects all of
E1–E3; B and C produce zero real detections; no D3-invalid; no FP on E4;
S1/S2 pass; floor silent. Parity; the null result is the deliverable — the
kw9.5 killed model-check result extends to the full three-arm comparison on
this slice. The categorical claim stays UNKNOWN-labeled beyond this slice
(power boundary unchanged). The unique-detection route to `survived` was
tested, not presumed: both categorical arms were exercised on the
defect-bearing slice and on the seeded dangling fault, and their silence on
the collision class is confirmed as by-construction (no cardinality claim
over stored arrows), not accidental.

### Discrepancies (recorded, never silent)

1. **S1 seeding wording vs its own registered result.** arms.md says
   "Append one capture row"; both protocol files register the result as a
   4th collision with "2 stored capture rows, one shared sha256" (README
   review note 7: "a duplicated pair"). One row yields a 1-match selector
   and cannot satisfy the pinned A-must-report-4 expectation. kw9.11
   implemented the pair (copies of the slice's first capture under
   `bench-seed/s1`, ids `bfc:bench-seed-s1` / `bfc:bench-seed-s1-2`),
   applied identically to all three arms.
2. **Dangling faults fire at load, not on the check surface.** S2 (arms B
   and C) and the recipe's overconstraint control are caught by the
   .ct.json v1 loader as labeled errors (exit 1) naming the dangling cell,
   not as check violations (exit 3). The registrations require a non-zero
   exit / that the battery catches the fault — satisfied verbatim; exit 3
   is not reachable for dangling stored references on this loader.
3. **Pre-registration commit hash.** The arms.md ledger template pins the
   kw9.6 commit hash; the ticket forbids git commands in kw9.11, so entries
   cite the protocol by path and pin the executed artifacts by sha256.
4. **Arm-A selector-set provenance.** `slice.json` carries no selector
   table, so arm A's selector set is DISTINCT (source, edition_date) over
   the slice's capture/partition rows (exactly the 3 collision selectors);
   the 127 zero-match negatives are scored via the floor run (census
   criterion over all 127: zero detections), matching census.json.

### Ledger

- `ledger/probes/bench/selector-state/0001-fcfd0068.json` — floor control
  (control: true, self-negating, verdict killed).
- `ledger/probes/bench/selector-state/0002-48bb05fe.json` — probe entry,
  verdict killed; embeds the expectations table, executed commands, the
  uniqueness arithmetic, the expressibility gap, and the discrepancies
  above. Both entries sealed (content-addressed) and `validate_dir` green
  over the new sequence and every neighbor sequence
  (`ledger/probes/{bench/selector-state,naturality,selector-state,…}`) via a
  throwaway validator test, deleted after the run.
