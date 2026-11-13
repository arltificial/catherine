# Tasks: add-declarative-keys

One feature cycle (RED → GREEN) plus one refactor cycle. Format 1.3; absent
`keys` stays hash-neutral, so no sealed artifact moves.

## RED

- [x] 1. Kernel format tests (failing): `keys` round-trips and canonicalizes
      (input order → sorted bytes), `KNOWN_MINOR` accepts 1.3, `invalid_key`
      cases at their nodes (`/schema/keys/<i>`), an absent-`keys` 1.2 document
      hashes exactly as before, a 3-way collision is three findings
- [x] 2. Check/findings tests (failing): accepted key collision → one
      `violation`/`executed` per colliding pair with `KeyWitness{key, rows}`;
      clean key → zero findings; proposed key → `hypothesized`; dangling cell
      in a key arrow → only the dangling finding; JSON shape for equation
      violations unchanged (byte-pinned)
- [x] 3. Schema-diff tests (failing): added accepted key → `accreted` naming
      it; dropped accepted key → `weakened` with a key witness instance that
      re-checks clean in v2 and rejected in v1; proposed key edit →
      `cosmetic`
- [x] 4. CLI/web tests (failing): `diff` payload `added` key shape and human
      `added:` line; key violation rendered with both rows; web panel renders
      a key witness (no `undefined` fields)
- [x] 5. Contracts for the new scenarios (`ah scenario new`) and negative
      matrix rows for the malformed-key surface

## GREEN

- [x] 6. Kernel: `Key` type + `Schema.keys`; parser pass (`check_keys`,
      `invalid_key`); canonical writer (sorted, omitted when empty);
      `KNOWN_MINOR = 3`; JSON Schema artifact + regenerated TS types
- [x] 7. Findings/checks: `ViolationWitness` union + `KeyWitness`;
      `key_findings` (pairs, dangling exclusion, label rule); rule-lens and
      discovery call sites migrated to the union
- [x] 8. Analysis: `EnforcedConstraint` (equation ∪ key), enforced multiset,
      `accreted` naming, weakening witness through the existing scan
- [x] 9. CLI: `ConstraintView` union + `added_line` key branch;
      `render_violation` key branch; web `findings.ts` union branch

## REFACTOR

- [x] 10. Gate caps: split any file/function the strict pretender gate
      rejects (whole-tree `pretender check --mode gate`), without behavior
      change; re-run the gate
- [x] 11. Docs: CHANGELOG entry, help/README note on the arrow-only key scope
      (value-object convention or `rules --project` for column FDs)

## Verification

- [x] 12. `cargo test --workspace`; `just ci`; `openspec validate --strict
      --all`; `ah check --changes add-declarative-keys --run-tests`; web
      `npx vitest run` (findings panel); acceptance fixture run

## Boundary

- [x] 13. Close note: keys are arrows-only (CORR-003 decision), the witness
      union is the CORR-002 decision, absent keys are hash-neutral, and
      attribute-column keys wait for format 1.4 (step 5)
## Close note

Landed on `main` in `915f06c` (feature), `1e92e49` (negative-matrix row),
`2d10292` (web + generated types), `4ee1b09` (docs). Beads `catherine-2s6`
(feature) and `catherine-obc` (refactor).

Decisions shipped as designed: `schema.keys` is a hashed schema section
(format 1.3, `KNOWN_MINOR` 3) omitted from canonical bytes when empty, so
every pre-1.3 document keeps its exact hash (the pinned
`sha256:fb033e3c…` fixture constants stayed green); `violation` carries the
untagged `ViolationWitness` union (`KeyWitness {key, rows}`; the equation
encoding is byte-identical); `schema_diff` enforces accepted equations ∪
accepted keys (added → `accreted`, dropped → `weakened` with the key
witness, through the existing bounded scan). CORR-003 is closed as
arrow-only keys: the capture FD `(source, edition_date)` needs the
value-object convention or `rules --project` until attribute columns
(format 1.4) exist. CORR-002 is closed as the witness union rather than an
overloaded `EquationWitness`, because `check` output mixes equation and key
violations in one list where a reused `equation` index would be ambiguous.

Evidence: `cargo test --workspace` green; `just ci` green (fmt, clippy
`-D warnings`, tests, wasm check, `pretender check --mode gate`); `openspec
validate --strict --all` 13/13; `ah check --changes add-declarative-keys
--run-tests` 107 passed; `npx vitest run` 4/4; the negative matrix asserts
a malformed key surfaces as labeled `invalid_key` (exit 1), never silence.

Boundary: keys range over declared arrows only; attribute-level keys are a
later additive minor, never a reinterpretation of 1.3.
