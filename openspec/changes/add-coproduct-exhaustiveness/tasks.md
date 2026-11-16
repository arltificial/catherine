# Tasks: add-coproduct-exhaustiveness

One feature cycle (RED → GREEN) plus one refactor cycle. Next free
additive minor at RED (4 at proposal time, 5 if `add-instance-morphism`
archived first); absent `coproducts` stays hash-neutral, so no sealed
artifact moves. Archives after `add-declarative-keys` (shared-requirement
rebase, design Decision 6).

## RED

- [ ] 1. Probe pre-registration (failing): commit `ledger/probes/
      coproduct/` catalog, payments fixtures (three variants clean; rogue
      fourth variant; dual-covered row), floor control, and the executed
      baseline arm pinning today's silent-acceptance behavior
- [ ] 2. Kernel format tests (failing): `coproducts` round-trips and
      canonicalizes (input order → sorted bytes), the new minor is
      accepted, `invalid_coproduct` cases at their nodes (undeclared
      object/arrow, arrow not `V_k → C`, empty list, duplicate names),
      absent section hashes exactly as before
- [ ] 3. Check/findings tests (failing): rogue variant row → one
      `violation`/`executed` naming the uncovered row; dual-covered row →
      one `violation`/`executed` naming the row and both injections; clean
      table → zero findings; proposed coproduct → `hypothesized`;
      equation-violation JSON bytes unchanged
- [ ] 4. Schema-diff tests (failing): added accepted coproduct →
      `accreted` naming it; dropped accepted coproduct → `weakened` with a
      witness instance that accepts a rogue variant v2-clean/v1-rejected;
      proposed edit → `cosmetic`
- [ ] 5. CLI/web tests (failing): human + `--json` rendering of both
      witness shapes; web panel renders without `undefined` fields
- [ ] 6. Contracts for the new scenarios (`ah scenario new`) and negative
      matrix rows for the malformed-coproduct surface

## GREEN

- [ ] 7. Kernel: `Coproduct` type + `Schema.coproducts`; parse pass
      (`invalid_coproduct`); canonical writer (omitted when empty);
      `KNOWN_MINOR` bump (Decision 6 sequencing: this change owns the
      versioning MODIFIED delta at apply only if no other pending change
      does); JSON Schema artifact + regenerated TS types
- [ ] 8. Checks: coverage/disjointness execution through the existing
      finding pipeline; label rule per status
- [ ] 9. Analysis: enforced-set extension (equations ∪ keys ∪
      coproducts), `accreted` naming, weakening through the existing scan;
      author the analysis MODIFIED deltas at this point, rebased onto
      `add-declarative-keys`' text as archived (shared-requirement rule,
      design Decision 6)
- [ ] 10. CLI: render branches for the two witness shapes; web union
      branch
- [ ] 11. Record the probe run against the pre-registered catalog

## REFACTOR

- [ ] 12. Gate caps: split any file/function the strict pretender gate
      rejects, without behavior change
- [ ] 13. Docs: CHANGELOG entry; README/USING note (coproducts are
      declared totality claims; no pullbacks/cones/defaults — Decision 4)

## Verification

- [ ] 14. `cargo test --workspace`; `just ci`; `openspec validate --strict
      --all`; `ah check --changes add-coproduct-exhaustiveness
      --run-tests`; gap-suite coproduct rows flipped to `closed` (requires
      `add-expressibility-gap-suite` archived first — see Impact)

## Boundary

- [ ] 15. Close note: coverage failures are violations (Decision 2), no
      limits machinery (Decision 4), probe verdict recorded whatever it
      is (Decision 5 — a `killed` probe stops the change before merge)
