# ro5u review — add-delta-migration PHASE A (catherine-alr, tasks 1–4)

Full-diff lead-side review. Diff: `crates/catherine-core` (SchemaMap type
— total maps only, explicit-drop `null` assignments; parse pass
`build_schema_map` with labeled `partial_schema_map` rejections naming
the unmapped element; KNOWN_MINOR 0→1 per convention; `partial_schema_map`
as the one named ErrorCode extension), `crates/catherine-core/schema/
ct-schema-v1.json` (additive optional section), regenerated TS types,
`crates/catherine-core/tests/schema_maps.rs` (4 tests),
`fixtures/migration/` (3 RED fixtures + README with per-fixture phase-B
expectations), `crates/catherine-cli/src/render/errors.rs` (the new
code's remediation hint), golden fixture `unknown_minor.json` (unknown
minor moved past the new KNOWN_MINOR) + its message expectations,
`bad_version.ct.json` (same), the cli version-policy test updated to
1.1. Tasks 5–10 are later phases — NOT started (maps are declared data;
pullback/reindex/lost-data execution is phase B).

The identity contract held: the hash-stability guard pins
`fixtures/parity/small/model.ct.json`'s pre-change `model_hash`
(computed BEFORE the section landed) and passes — pre-map documents
serialize byte-identically (the section is skip-serialized when absent),
so the P6 ledger's content addressing did not shift.

Orchestrator verification (live): `just ci` EXIT=0 (fmt/clippy -D
warnings/full workspace tests incl. schema_maps 4/4 + golden + roundtrip
+ parity); pretender gate green; `openspec validate
add-delta-migration --strict` valid; TS regenerated + `tsc` clean.

Five passes:

1. Correctness: totality enforced at parse with the labeled error naming
   the unmapped element; assignment targets must exist in the validated
   target schema (the target runs the same object/arrow/equation passes
   under its own pointer root); explicit drops are `null`, never silent
   defaults.
2. Contract drift (class B): the closed error-code enum gained exactly
   one variant via the named-ticket path (documented in error.rs); the
   CLI remediation hint + the version-policy test updated to the raised
   KNOWN_MINOR reality (the stale "known version is 1.0" hints/tests
   were the drift).
3. Tests: hash-stability guard + total/partial/fixed-point ratchets; the
   golden rejection table updated for the raised minor (position pins
   preserved).
4. Negative matrix: no new CLI flag (the `migrate` subcommand is phase
   B/C); the parse surface's new rejection IS the covered class.
5. Regression risk: additive optional section, kernel genesis-free;
   roundtrip/parity suites green (270s roundtrip run included).

Findings + dispositions:

- [DEFERRED, LOW, written reason] RED fixtures' phase-B outcomes are
  documented in `fixtures/migration/README.md` rather than executable
  assertions — the engine (phase B) is what makes them executable; the
  README pins the expectations now so phase B cannot goalpost-move.
- [NOTE] task 4's "JSON Schema update" is additive-only; the schema
  artifact's syntactic surface does not enforce totality (kernel-side,
  documented in the section description).

max_critical = 0. Verdict: PASS.

---

# Phase C addendum (tasks 7–9, orchestrator review)

Diff: `crates/catherine-cli` (`migrate` subcommand via main.rs dispatch +
`src/migrate.rs` runner + render envelope extension), negative_matrix.rs
rows for the new surface, variant_coverage.rs (the migrate surface joins
the executed-coverage corpus), `fixtures/migration/bogart_rehearsal.ct.json`
+ README rehearsal row, ticked tasks 7–9.

Orchestrator verification (live): `catherine migrate
bogart_rehearsal.ct.json --json` → envelope `data.lost` = **10 findings
(4 lost_row RawCapture + 6 lost_cell)**, exit 8; drop_object/field_narrowing
exits 8; lossless exit 0; exit decision consistent across human/JSON
modes (`migration_exit`); full `just ci` EXIT=0; pretender green.

Phase-C passes:

1. Correctness: the subcommand drives the frozen phase-B engine on the
   shared execution path; exit decision (lost data → 8, lossless → 0)
   consistent in both render modes.
2. Contract drift: negative-matrix rows added for the new surface (348
   discipline); the cli version-policy test updated to the raised
   KNOWN_MINOR (the deliberate contract change this phase series made).
3. Tests: cli suite green (23+11+11+7+7+1); rehearsal findings match the
   documented expectation exactly (4+6, deterministic order).
4. Negative matrix: covered (new surface rows present).
5. Regression risk: additive subcommand; exit table extended at the
   lowest severity (8) — existing class exits untouched.

Findings + dispositions:

- [RECORDED] task 6 (round-trip analysis) remains OPEN — the Δ∘Δ′
  return-identity design decision is documented in the ignored
  round-trip test (migration.rs) and blocks alr closure, not phase C.
  alr stays OPEN after this phase.
- [NOTE] envelope `data.rows` reports per-target-object row counts; the
  full `row_map` is not serialized (per-target counts + lost findings
  are the operator-facing contract).

max_critical = 0. Phase C verdict: PASS.

---

# Task-6 completion addendum (round-trip proof, orchestrator review)

The Δ∘Δ′ return-identity design decision is SETTLED and implemented: a
round-trip element is a source (object, row) pair the forward map keeps;
its forward image is computed per SOURCE OBJECT (`transport::placed_row`
— never read from the merged positional row_map, whose keys can collide
across preimage objects); its return is the (object, row) the candidate
inverse yields; identity holds iff both fields match. Missing return
place, different object, or different row are all non-identities;
offender = smallest (object, position), sibling named in the detail.
The ignored test is UN-IGNORED and passing (pins offender Wide/0,
detail naming rows 0 and 1 returning Narrow row 0); a new
identity-round-trip negative regression proves clean maps yield None.
`NonInvertibilityProof` shape unchanged.

Orchestrator verification: `cargo test -p catherine-core --test
migration` → 5/5 green; `just ci` EXIT=0; pretender green; openspec
validate valid. No README or test weakening (the pinned expectation is
entailed and stricter: smallest offender + sibling named).

Findings: none ≥ MEDIUM. max_critical = 0. Task-6 verdict: PASS.
