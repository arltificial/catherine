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
