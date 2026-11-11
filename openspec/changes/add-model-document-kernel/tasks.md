# Tasks: add-model-document-kernel

One red→green→refactor cycle (beads `catherine-bpz`). Crate:
`crates/catherine-core`; designative modules: `model.rs`, `parse.rs`,
`canonical.rs`, `hash.rs`, `error.rs`.

## RED — malformed presentations rejected with line-accurate errors

- [ ] 1. Scaffold Rust workspace + `crates/catherine-core` (no deps beyond
      serde/serde_json + proptest); wire `just` check gate
- [ ] 2. Golden fixture: duplicate object ident → error `duplicate_ident`
      with line/column of the second occurrence
- [ ] 3. Golden fixture: duplicate arrow ident → `duplicate_ident`
- [ ] 4. Golden fixture: equation referencing unknown arrow → `unknown_arrow`
- [ ] 5. Golden fixture: ill-typed equation (path endpoints mismatch, e.g.
      `a∘b = c` where endpoints differ) → `ill_typed_equation`
- [ ] 6. Golden fixtures: `format_version` absent / unknown major / bad
      equation `status` → `unsupported_version` / `invalid_status`
- [ ] 6b. Golden fixture: identifier violating `[A-Za-z_][A-Za-z0-9_]*`
      (unicode, empty, leading digit) → `invalid_ident`
- [ ] 6c. Fixture: empty schema (no objects/arrows/equations) and empty
      instance parse cleanly
- [ ] 7. Error type: `{line, column, code, message}`, stable `code` enum

## GREEN — valid documents parse, canonicalize, round-trip, hash

- [ ] 8. Parse valid `.ct.json` v1: schema (objects, arrows, equations with
      rationale + status) + instance (rows, per-arrow tables, `null` =
      dangling preserved — never dropped)
- [ ] 9. Canonical serialization: sorted enumeration, byte-stable across
      input orders (unit test: two orderings → identical bytes)
- [ ] 10. Canonical-JSON + sha256 hash ported from wild `formats.rs`
      (normative byte-for-byte tests from wild's suite)
- [ ] 11. Round-trip proptest: `parse(serialize(arbitrary_model)) == model`;
      plus row-id-canonicalization property (same structure, different
      presentation row names → equal canonical bytes and equal hash)
- [ ] 12. Publish `ct-schema-v1.json` (JSON Schema artifact) + document
      `format_version` policy in the crate README

## REFACTOR

- [ ] 13. Extract shared parse-error type; dedupe fixture harness
- [ ] 14. Verify specodelic/wild attribution: NOTICE headers on ported code

## Validation

- [ ] 15. `just ci` green; `openspec validate add-model-document-kernel
      --strict`; `ah check --changes add-model-document-kernel`; `bd close
      catherine-bpz` + fresh export
