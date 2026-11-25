# Three-arm bench — arm specifications, scoring rubric, ledger plan

Companion to `fixtures/probes/bench/README.md` (the pre-registered
protocol). Everything here is specification for the kw9.11 run — **no run
happens before that ticket**; this file registers the exact inputs,
commands, and scoring so the pre-registration is complete.

Global invariants, all arms:

- **Offline over frozen data.** Inputs are the frozen artifacts only
  (hashes in README custody). No live database is opened; no extraction is
  re-run.
- **Custody separation.** `fixtures/probes/selector-state/census.json` is
  scoring-only ground truth (`collision_selectors`, `zero_match_selectors`).
  No arm consumes it. `fixtures/probes/selector-state/slice.json` is the
  only state any arm consumes.
- **Working copies, never in place.** Seeded and floor variants are written
  to new files; frozen artifacts are read-only for every arm.
- **Record verbatim.** Every command, its full output, and its exit code are
  recorded in the kw9.11 transcript; the ledger entry embeds them.

---

## Arm A — conventional suite (the control)

**Artifact.** `prototype/bogart_selector_bench.py` — a sibling of the
committed, executed conventional baseline precedent
`prototype/bogart_storage_granularity.py` (whose `--census` semantics it
reuses verbatim: capture `(src, ed)` matches selector `(s, e)` iff
`src == s` and (`ed == e` or `e` is in the capture's recorded edition-date
alias set)). **Committed and version-pinned in kw9.11** — it does not exist
at pre-registration time, by design; the ledger entry records its commit
hash as the baseline pin.

**Detection criterion (pinned, single criterion).** A detection is a
selector matched by **≥2 stored capture rows** — the census criterion. No
secondary modes: the duplicate-`sha256` view is a *description* of the
known defect shape (kw9.5 README), not a separate scoring criterion, and is
not implemented.

**Instance-custody (scoring hygiene).** The script reads `slice.json` rows
only. It must not read `census.json`; scoring compares its output to the
census ground truth out-of-band.

**Commands (kw9.11).**

```sh
python3 prototype/bogart_selector_bench.py \
  --instance fixtures/probes/selector-state/slice.json            # probe slice
python3 prototype/bogart_selector_bench.py --floor \
  --instance fixtures/probes/selector-state/floor-selectors.json  # floor (derived below)
python3 prototype/bogart_selector_bench.py --seed s1 \
  --instance fixtures/probes/selector-state/slice.json            # sanity S1
python3 prototype/bogart_selector_bench.py --seed s2 \
  --instance fixtures/probes/selector-state/slice.json            # sanity S2
```

- `--floor`: evaluates the census criterion over the collision-free
  sub-slice. `floor-selectors.json` is derived in kw9.11 from the frozen
  `census.json` `zero_match_selectors` (the same derivation the committed
  floor fixture discloses); derivation recorded. Expected output: zero
  detections.
- `--seed s1|s2`: applies the deterministic seeding below to an in-memory
  copy (or a written copy under the kw9.11 working directory — never to the
  frozen files) and evaluates. Expected outputs: S1 → 4 collision selectors
  (3 real + seed); S2 → same output as the unseeded run (referential state
  is outside the census criterion; the run still exits 0).

**Version pin.** The kw9.11 commit hash of
`prototype/bogart_selector_bench.py` is recorded in the ledger entry's
`comparison.conventional_baseline`. Without that committed artifact +
recorded detections, the entry cannot carry a uniqueness verdict
(domain-probes spec) — this is the E4 honesty rule the bench exists to
satisfy.

---

## Arm B — catherine hand-built model

**Artifact.** `fixtures/probes/selector-state/selector-state.ct.json`
(committed; hashes in README custody). Schema: 7 objects, 5 stored arrows,
`equations: []`; instance: 602 rows (selector 130, capture 6, partition 3,
artifact 3, read_page 454, read_document 5, edition_date 1).

**Battery.** `check` (detections come from here only) +
`underconstraint --max-path-length 3` (recorded, never scored — see
rubric). The binary invocation is pinned in kw9.11
(`target/debug/catherine` or `cargo run --quiet --bin catherine --`, one
choice, recorded once).

```sh
$CAT check           fixtures/probes/selector-state/selector-state.ct.json
$CAT underconstraint fixtures/probes/selector-state/selector-state.ct.json --max-path-length 3
$CAT check           fixtures/probes/selector-state/selector-state-floor.ct.json          # floor
$CAT check           <kw9.11 working dir>/selector-state-seed-s1.ct.json                  # sanity S1
$CAT check           <kw9.11 working dir>/selector-state-seed-s2.ct.json                  # sanity S2
```

The seeded model copies re-embed the seeded slice rows into the committed
model's instance (the seeding is defined on slice rows, below); schema and
equations untouched.

kw9.5 already executed the first command: exit 0, zero findings,
`rows_evaluated 602`, `constraints_checked 0`. The bench replicates it at
the kw9.11 pin and scores it; it does not assume the kw9.5 result carries
over.

---

## Arm C — LLM-drafted model, attacked via the kw9.9 recipe

**Recipe.** `docs/ref/ai-draft-attack-recipe.md` v1, followed verbatim:
prose in → LLM drafts (`draft-v1.ct.json`, one file per revision, never
edited in place) → battery → findings back verbatim → iterate → stop only
on the recipe's stopping rule (every battery verb exits 0 **and** both
negative controls demonstrated on this demo). Working directory:
`fixtures/attack-demo/bench-selector-state/` (recipe naming).

**The prose (pre-registered verbatim; the LLM's only input for the draft
step).** kw9.11 copies this block byte-for-byte into
`fixtures/attack-demo/bench-selector-state/prose.md` and records its hash;
any other prose is a protocol deviation:

```markdown
# Prose requirements — backfill capture queue (the LLM's only input)

A backfill queue archives snapshots of web sources. Each capture row
stores which source it archived, the edition date it was captured for,
a content hash of the archived artifact, and optionally the set of
edition dates (aliases) it counts for. A selector names one source at
one edition date. A capture counts for a selector when the sources
match and the selector's edition date is the capture's own edition
date or one of its recorded aliases. Each partition row plans one
future capture for a selector and stores the same source and edition
date. On the serving side, each published document row stores its
edition date, and each page row stores which document — by document id
and version — it belongs to.

A selector is collided when two or more stored capture rows count for
it and those rows carry the same content hash: two stored rows for one
artifact. Model which rows store which references, and make every
stored obligation you can state checkable.

The instance rows you must use are given verbatim in
`instance-slice.json` next to this file; use them exactly, and add no
row of your own.
```

Fairness note (why the rule is stated): arm A's criterion is baked into
committed code and arm B's rule statement lives in the committed kw9.5
README; arm C gets the same rule as prose, and the ground truth is not
disclosed beyond the rule (no selector names, no counts). The instance
pointer is part of the input so the draft's instance satisfies the
instance-custody rule by construction; the LLM may not invent rows.

**Battery (all four recipe verbs, on every revision).**

```sh
$CAT check           fixtures/attack-demo/bench-selector-state/draft-v<N>.ct.json --human
$CAT underconstraint fixtures/attack-demo/bench-selector-state/draft-v<N>.ct.json --max-path-length 3 --human
$CAT search          fixtures/attack-demo/bench-selector-state/draft-v<N>.ct.json --rows <declared objects' row counts> --human
$CAT naturality      fixtures/attack-demo/bench-selector-state/draft-v<N>.ct.json <target> --human   # only if an instance_map is declared
```

**Evaluated artifact.** The final `draft-v<N>.ct.json` that stopped the
loop — the recipe's output, per the protocol. Intermediate drafts are
provenance, not the arm. The bench scores the final draft's battery run on
the probe slice, plus the floor and seeded variants:

```sh
$CAT check <working dir>/draft-final-floor.ct.json    # final draft schema, floor instance
$CAT check <working dir>/draft-final-seed-s1.ct.json
$CAT check <working dir>/draft-final-seed-s2.ct.json  # = the recipe's mandatory overconstraint control
```

**Instance-custody rule (all ct arms; laundering control).** Every instance
row in the evaluated draft must come verbatim from `slice.json` (row ids
identifiable in the frozen slice). A draft instance carrying computed or
derived rows — match counts, tallies, any row not in the frozen slice — is
**D3-invalid**: the detection is the conventional computation laundered
through the model checker; it is scored as FP, excluded from TP before the
uniqueness arithmetic, and the arm is recorded as not independent. Arm B
satisfies the rule by construction (committed model). Checked in kw9.11 by
diffing the draft's instance rows against the frozen slice rows; the diff is
recorded.

**Loop non-convergence.** If the recipe's stopping rule cannot be reached
(the draft keeps firing true findings the LLM cannot remove without
violating the prose), arm C records loop-nonconvergence with the full
transcript and the bench scores C **invalid-run**. That outcome is itself a
result (the categorical side cannot certify a model over defect-bearing
state under the recipe), recorded as such — not suppressed.

---

## Seeding procedures (deterministic; sanity arm)

Defined on `slice.json` rows; the same transformation applies to arm B's
model instance and arm C's final draft instance, so all three arms consume
identical seeded state.

**S1 — seeded 4th collision.** Append one capture row: a copy of the slice's
first capture (`pba/JUDICIAL 2026-11-03`, `sha256 d9ff31c3…`) with
`capture_id = "bfc:bench-seed-s1"`, `source = "bench-seed/s1"`,
`revision = "bench-seed-s1"` (edition date, alias set, `captured_at`
unchanged), and append the selector row `("bench-seed/s1", "2026-11-03")` to
the selector set. Result: a 4th collision selector — 2 stored capture rows,
one shared sha256, same shape as the known-real defects.

**S2 — seeded dangling reference.** Copy the slice; set the first capture's
`sha256` to `"bench-seed-s2-dangling"` (no artifact row exists for it).
Result: an expressible fault — a dangling stored reference.

Scoring: S1/S2 outcomes count in **sanity pass/fail only** (README decision
rule). An arm firing on S1 in a way that matches its E1–E3 expectation is a
sanity pass recorded as expected-class-reproduced; an A miss on S1 or a B/C
silence on S2 is an invalid-run (scoring validation failed).

---

## Scoring rubric (fixed in advance)

**What counts as a detection.** A `check`-surface violation finding that
names a defect row (one of the 3 collision selectors, their 6 capture rows,
or their 3 artifact rows — the kw9.5 rubric's REAL set, unchanged). Only
`check` violations are detections.

**Non-findings (recorded, never scored as detections).**

- `underconstraint` output (exit 6): hypothesized, never a violation.
- `search` payloads: a witness is a bounded-run success, not a clean bill; a
  contradiction or UNKNOWN is recorded verbatim. Retroactive re-scoring of
  any non-finding as a detection requires a superseding ledger entry, never
  an edit.

**Per-arm accounting** (rows E1–E3 defect ids; E4 negatives; README table):

- `TP_X` = |{E1, E2, E3 detected as real findings}|; `FP_X` = findings on
  E4 rows + all other noise; `D3inv_X` = findings attributable to
  derived/FizzBuzz relations — counted, flagged, scored as FP, removed from
  TP before uniqueness arithmetic.
- `unique_X = TP_X ∖ TP_A` — set difference over defect ids against the
  control arm, recorded in the ledger entry's `comparison.unique_detections`.
- Precision-style ratio `real / total findings` is reported per arm for
  comparability with the kw9.5 rubric (0/0 = undefined), but the **decision
  rule runs on the uniqueness arithmetic, not the ratio**.

**Verdict vocabulary** — the closed set `killed | survived | inconclusive |
invalid-run`, mapped in the README decision rule (fixed before the run).

---

## Ledger plan

New sequence `ledger/probes/bench/selector-state/`, reusing the kata entry
shape and validator (domain-probes spec). The floor control entry must
self-negate before the probe entry (`NegativeControlMissing` /
`ControlDidNotSelfNegate` otherwise). Content-addressed ids
(`0001-<hash>.json`, `0002-<hash>.json`); append-only supersession; the
entry embeds its expectation rows and carries the hash of this committed
protocol (README + arms.md), so expectations cannot be retrofitted.

**0001 — floor control (control: true, verdict must be `killed` —
self-negating).** All three arms over the collision-free sub-slice: A
(census criterion over an empty capture set — zero detections), B (the
committed floor fixture `selector-state-floor.ct.json`; executed silent in
kw9.5, replicated at the kw9.11 pin), C (final-draft battery over the floor
instance). Any finding or detection on the floor means the bench
manufactures detections from defect-free state — that is an invalid-run of
the bench, not a probe result.

```json
{"seq": 1, "kind": "kata", "recorded": "<kw9.11 run date>",
 "kata": {
  "kata_id": "probe-bench-selector-state",
  "name": "probe bench floor control: all three arms score zero real detections over the collision-free sub-slice — the bench has nothing to detect there",
  "control": true,
  "series_report": "fixtures/probes/bench/README.md + fixtures/probes/bench/arms.md (pre-registered kw9.6, commit <hash>)",
  "series_verdict": "pre-registered: the floor control must self-negate — on defect-free state each arm scores zero real detections, so the bench series has no unique detection to claim on the floor; any floor finding is a scoring-alarm invalid-run",
  "fixture": "fixtures/probes/selector-state/selector-state-floor.ct.json",
  "fixture_hash": "sha256:04517b828eb42948418cafa22dacb35a60131d9eeac2f38cc832d2038908750b",
  "expected": [{"fault": "floor-all-arms-silent",
                "series_class": "behavioral",
                "catherine_class": "none",
                "detail": "every arm scores zero real detections on the collision-free sub-slice (127 zero-match selectors, no captures/partitions/artifacts/pages): A — census criterion over an empty capture set; B — committed floor model; C — final-draft battery over the floor instance; falsified by any finding or detection on the floor (scoring alarm, invalid-run)"}],
  "procedure": ["<kw9.11 records the three floor commands and outputs verbatim>"],
  "run": {"commands": ["<verbatim>"], "findings": []},
  "comparison": {"conventional_baseline": "<arm A floor run: prototype/bogart_selector_bench.py --floor, commit <hash>, output verbatim>",
                 "fired": [], "unique_detections": []},
  "verdict": "killed"}}
```

**0002 — probe (control: false).** The three arms on the probe slice +
seeded variants; expected rows are the README expectations table, one row
per fault; `catherine_class` carries arm B's registered class (the
ledger-native check arm); arms A and C's registered classes live in the row
`detail` (naturality precedent: the baseline expectation lives in the
detail).

```json
{"seq": 2, "kind": "kata", "recorded": "<kw9.11 run date>",
 "kata": {
  "kata_id": "probe-bench-selector-state",
  "name": "probe bench selector-state: three-arm defect accounting on the frozen 6-capture/3-collision slice — <filled from the executed accounting, never the expectation>",
  "control": false,
  "series_report": "fixtures/probes/bench/README.md + fixtures/probes/bench/arms.md (pre-registered kw9.6, commit <hash>; result recorded <date>)",
  "series_verdict": "pre-registered decision rule (README, fixed before the run): the bench verdict is the uniqueness arithmetic against the executed conventional control — survived only on a real, custody-clean detection absent from arm A's executed detections; parity is killed and the null result is the deliverable; cited-not-executed baseline caps at inconclusive",
  "fixture": "fixtures/probes/selector-state/selector-state.ct.json",
  "fixture_hash": "sha256:3cc5314d614d57e4eb8fa33420a2b1ee582c4e548e2268ce97bb23b72143163a",
  "expected": [
    {"fault": "collision-nacional-primera", "series_class": "behavioral", "catherine_class": "none",
     "detail": "E1 — A: finding; B: silent; C: silent; unique_B = TP_B ∖ TP_A = ∅, unique_C = TP_C ∖ TP_A = ∅; falsified by any other class in any arm"},
    {"fault": "collision-pba-judicial",     "series_class": "behavioral", "catherine_class": "none",
     "detail": "E2 — same classes and arithmetic as E1"},
    {"fault": "collision-nacional-segunda", "series_class": "behavioral", "catherine_class": "none",
     "detail": "E3 — same classes and arithmetic as E1 (corpus-side only; no serving rows)"},
    {"fault": "zero-match-negatives",       "series_class": "negative",   "catherine_class": "none",
     "detail": "E4 — all arms silent on the zero-match negatives (127 frozen; 67 recorded GT named, drift disclosed); falsified by any finding naming a zero-match selector (FP, any arm)"},
    {"fault": "derived-relations-silence",  "series_class": "negative",   "catherine_class": "none",
     "detail": "E5 — B and C silent on derived relations (equations: []); A is N/A (the census is the rule-based machinery); any model-check finding attributable to a derived relation is D3-invalid FP, removed from TP before uniqueness arithmetic"},
    {"fault": "fizzbuzz-negative-control",  "series_class": "negative",   "catherine_class": "none",
     "detail": "E6 — all arms silent on a FizzBuzz-class computed predicate (prose intent only); any finding attributable to one is a scoring alarm (invalid-run)"},
    {"fault": "seed-s1-fourth-collision",   "series_class": "sanity",     "catherine_class": "none",
     "detail": "S1 — A must report 4 collision selectors (3 real + bench-seed/s1); B expected-silent (same inexpressible class); C silent per its E1–E3 registration; sanity pass/fail only, never in TP/FP"},
    {"fault": "seed-s2-dangling-reference", "series_class": "sanity",     "catherine_class": "finding",
     "detail": "S2 — A is N/A (no referential claim in census scope; scope-gap); B must fire (dangling); C must fire (the recipe's mandatory overconstraint control); silence from B or C is battery-vacuity, invalid-run"}],
  "discrepancies": [],
  "procedure": ["<kw9.11 records the arm-by-arm procedure: A commands + baseline commit hash; B battery at the pinned binary; C loop transcript + final-draft hash + instance-custody diff; seeded + floor variants>"],
  "run": {"commands": ["<verbatim, all arms>"], "findings": ["<executed findings by id, or []>"]},
  "comparison": {
    "conventional_baseline": "arm A — prototype/bogart_selector_bench.py, committed at <kw9.11 commit hash>, census match semantics verbatim over fixtures/probes/selector-state/slice.json; detections on the same fault set: <executed list>; precedent: prototype/bogart_storage_granularity.py --census (committed, executed 2026-11-13 / 2026-11-25)",
    "fired": ["<per-arm executed detections>"],
    "unique_detections": ["<TP_B ∖ TP_A and TP_C ∖ TP_A as executed; non-empty only with the executed baseline above>"]},
  "expressibility_gap": [{"invariant": "selector→capture duplication (the 3 known-real collisions)",
                          "why": "kw9.5 ledger 0002: the check surface has no cardinality claim over stored arrows; declaring the collision would be a derived relation (the P3 boundary). The bench measures whether ANY arm detects it — the gap is the object under test, restated here because it licenses the B expectation"}],
  "verdict": "<per the pre-registered decision rule, from the executed accounting>"}}
```

Validator constraints that bind this plan (domain-probes spec): a non-empty
`unique_detections` without the executed, committed baseline arm is refused;
a cited-but-not-executed baseline caps the verdict at `inconclusive`;
unregistered expectations cannot derive a verdict; editing the sealed entry
or its catalog breaks the content address and forces supersession.
