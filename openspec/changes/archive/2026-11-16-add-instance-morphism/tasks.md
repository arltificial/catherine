# Tasks: add-instance-morphism

One feature cycle (RED → GREEN) plus one refactor cycle. Next free
additive minor at RED (4 at proposal time); absent `instance_map` stays
hash-neutral, so no sealed artifact moves. This change owns the versioning
MODIFIED delta (design Decision 6): `KNOWN_MINOR` moves to 4 and the delta
carries the updated illustrative list.

## RED

- [x] 1. Kernel format tests (failing): `instance_map` round-trips and
      canonicalizes (sorted `(object, rows)` order), the new minor is
      accepted, an absent section hashes exactly as before,
      `incomplete_instance_map` fires at the uncovered row's node,
      `unknown_map_target` fires at the cell naming a missing target row
- [x] 2. Naturality tests (failing): the same document as source and
      target → zero findings, exit 0 (design Decision 4 degenerate case);
      a commuting map → clean (exit 0);
      the crm/warehouse `a3→w3` fixture → one `violation`/`executed` with
      witness `{arrow, source_row, lhs, rhs}`; mismatched schemas →
      `schema_mismatch` reused; JSON envelope shape additive only
- [x] 3. CLI tests (failing): `naturality src.ct.json tgt.ct.json` human +
      `--json` render; witness renders arrow and both composed values;
      exit codes per the existing precedence (violation class)
- [x] 4. Contracts for the new scenarios (`ah scenario new`) and negative
      matrix rows for incomplete maps and unknown targets

## GREEN

- [x] 5. Kernel: `InstanceMap` type + parse pass (`invalid_*` codes at
      nodes), canonical writer (omitted when empty), `KNOWN_MINOR` bump
      with the versioning MODIFIED delta in this change's
      `model-document` delta (Decision 6: this change owns it — no other
      pending change does), JSON Schema artifact + regenerated TS types
- [x] 6. Analysis: `naturality(doc_a, doc_b)` — schema identity check
      (reused), target resolution, commutation sweep producing findings
      through the existing taxonomy
- [x] 7. CLI: subcommand wiring, human/JSON renderers, exit-code mapping
- [x] 8. Probe: commit `ledger/probes/naturality/` catalog + crm/warehouse
      fixtures, floor control, and the executed `distinguish` baseline
      arm; record the run

## REFACTOR

- [x] 9. Gate caps: split any file/function the strict pretender gate
      rejects, without behavior change
- [x] 10. Docs: CHANGELOG entry; README/USING note (verify-given-mappings
      only; no synthesis — the G2 scope guardrail)

## Verification

- [x] 11. `cargo test --workspace`; `just ci`; `openspec validate --strict
      --all`; `ah check --changes add-instance-morphism --run-tests`;
      gap-suite naturality rows flipped to `closed` (requires
      `add-expressibility-gap-suite` archived first — see Impact)

## Boundary

- [x] 12. Close note: totality at parse, target resolution at
      construction (Decision 2), breaks are violations (Decision 3),
      same-schema only (Decision 4), probe verdict recorded whatever it
      is (Decision 5)
